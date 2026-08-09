use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

const DEFAULT_WALLPAPER_UPLOAD_LIMIT: i64 = 20;

async fn ensure_wallpapers_table(pool: &MySqlPool) {
    if let Some(stmt) = crate::schema::table_statements().iter().find(|s| s.contains("`wallpapers`")) {
        let _ = sqlx::query(stmt).execute(pool).await;
    }
}

async fn ensure_wallpaper_upload_limits_table(pool: &MySqlPool) {
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS `wallpaper_upload_limits` (
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `upload_limit` int(11) NOT NULL DEFAULT 20,
            `remark` varchar(255) NOT NULL DEFAULT '',
            `updated_by` varchar(64) NOT NULL DEFAULT '',
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`ciyuanxi_id`),
            KEY `idx_upload_limit` (`upload_limit`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
    )
    .execute(pool)
    .await;
}

async fn read_global_wallpaper_upload_limit(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT setting_value FROM server_settings WHERE setting_key = 'wallpaper_upload_limit' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .and_then(|v| v.trim().parse::<i64>().ok())
    .filter(|v| *v >= 0)
    .unwrap_or(DEFAULT_WALLPAPER_UPLOAD_LIMIT)
}

fn wallpaper_dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").join("wallpapers")
}

/// 保存压缩图片：统一转 JPG，最大宽度 max_w，质量 quality
fn compress_and_save_image(bytes: &[u8], target: &std::path::Path, max_w: u32, quality: u32) -> bool {
    use image::GenericImageView;
    let img = match image::load_from_memory(bytes) {
        Ok(i) => i,
        Err(_) => return false,
    };
    let (w, h) = img.dimensions();
    let (nw, nh) = if w > max_w {
        (max_w, (h * max_w / w).max(1))
    } else {
        (w, h)
    };
    let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgb8();
    let file = match std::fs::File::create(target) {
        Ok(f) => f,
        Err(_) => return false,
    };
    image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), quality as u8)
        .encode(&rgb, nw, nh, image::ExtendedColorType::Rgb8)
        .is_ok()
}

/// 新增壁纸（图片上传 + 压缩原图 + 缩略图）
pub async fn add_wallpaper(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    // 兼容 multipart 或 JSON base64 两种传入方式
    let data = parse_body(body);
    let title = str_of(&data, "title").trim().to_string();
    let description = str_of(&data, "description").trim().to_string();
    let mut category = str_of(&data, "category").trim().to_string();
    if category.is_empty() {
        category = "默认".into();
    }
    if title.is_empty() {
        return err(400, "请填写壁纸标题");
    }
    let image_b64 = str_of(&data, "image").to_string();
    if image_b64.is_empty() {
        return err(400, "请上传壁纸图片");
    }
    use base64::Engine;
    let bytes = match base64::engine::general_purpose::STANDARD.decode(&image_b64) {
        Ok(b) => b,
        Err(_) => return err(400, "无效的图片数据"),
    };
    let valid_ext = image::guess_format(&bytes).map(|f| matches!(f, image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP)).unwrap_or(false);
    if !valid_ext {
        return err(400, "只允许上传 JPG/PNG/WEBP 图片");
    }
    ensure_wallpapers_table(pool).await;
    let dir = wallpaper_dir();
    if let Err(_) = std::fs::create_dir_all(&dir) {
        return err(500, "无法创建上传目录");
    }
    let ins = sqlx::query(
        "INSERT INTO wallpapers (title, description, category, image_url, thumbnail_url, status, uploaded_by, uploaded_by_nickname, reviewed_at, reviewed_by) VALUES (?, ?, ?, '', '', 'normal', 'admin', ?, NOW(), ?)",
    )
    .bind(&title)
    .bind(&description)
    .bind(&category)
    .bind(&ctx.username)
    .bind(&ctx.username)
    .execute(pool)
    .await;
    let wp_id = match ins {
        Ok(r) => r.last_insert_id() as i64,
        Err(_) => return err(500, "数据库错误"),
    };
    let main_path = dir.join(format!("wallpaper_{}.jpg", wp_id));
    let thumb_path = dir.join(format!("thumb_{}.jpg", wp_id));
    if !compress_and_save_image(&bytes, &main_path, 1920, 82) {
        let _ = sqlx::query("DELETE FROM wallpapers WHERE id = ?").bind(wp_id).execute(pool).await;
        return err(500, "图片保存失败，请检查目录权限");
    }
    if !compress_and_save_image(&bytes, &thumb_path, 480, 72) {
        let _ = std::fs::copy(&main_path, &thumb_path);
    }
    let image_url = format!("/uploads/wallpapers/wallpaper_{}.jpg", wp_id);
    let thumb_url = format!("/uploads/wallpapers/thumb_{}.jpg", wp_id);
    let _ = sqlx::query("UPDATE wallpapers SET image_url = ?, thumbnail_url = ? WHERE id = ?")
        .bind(&image_url).bind(&thumb_url).bind(wp_id).execute(pool).await;
    log_operation(pool, ctx, "新增壁纸", &title, &format!("ID:{}", wp_id)).await;
    ok("上传成功", json!({ "id": wp_id }))
}

/// 壁纸列表
pub async fn list_wallpapers(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    ensure_wallpapers_table(pool).await;
    let rows = sqlx::query("SELECT * FROM wallpapers ORDER BY sort_order DESC, id DESC")
        .fetch_all(pool).await;
    match rows {
        Ok(rows) => {
            let arr: Vec<Value> = rows.iter().map(|r| crate::admin::row_to_value(r)).collect();
            log_operation(pool, ctx, "查看壁纸列表", "", "").await;
            ok("ok", json!(arr))
        }
        Err(_) => err(500, "数据库错误"),
    }
}

/// 删除壁纸
pub async fn delete_wallpaper(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "无效的壁纸ID");
    }
    let wp = sqlx::query("SELECT image_url, thumbnail_url FROM wallpapers WHERE id = ?").bind(id).fetch_optional(pool).await.ok().flatten();
    let Some(wp) = wp else {
        return err(404, "壁纸不存在");
    };
    let dir = wallpaper_dir();
    for f in ["image_url", "thumbnail_url"] {
        let rel: String = wp.try_get(f).unwrap_or_default();
        if !rel.is_empty() {
            let name = rel.rsplit('/').next().unwrap_or("").to_string();
            let p = dir.join(&name);
            if p.is_file() {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
    let _ = sqlx::query("DELETE FROM wallpapers WHERE id = ?").bind(id).execute(pool).await;
    log_operation(pool, ctx, "删除壁纸", &format!("ID:{}", id), "").await;
    ok("删除成功", Value::Null)
}

/// 修改壁纸状态
pub async fn change_wallpaper_status(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let status = str_of(&data, "status");
    let status = if status.is_empty() { "normal".to_string() } else { status };
    if id <= 0 {
        return err(400, "无效的壁纸ID");
    }
    let valid = ["normal", "disabled", "pending", "rejected"];
    if !valid.contains(&status.as_str()) {
        return err(400, "无效的状态");
    }
    ensure_wallpapers_table(pool).await;
    if status == "normal" || status == "rejected" {
        let upd = sqlx::query("UPDATE wallpapers SET status = ?, reviewed_at = NOW(), reviewed_by = ? WHERE id = ?")
            .bind(&status).bind(&ctx.username).bind(id).execute(pool).await;
        match upd {
            Ok(_) => {
                let label = if status == "normal" { "审核通过壁纸" } else { "拒绝壁纸" };
                log_operation(pool, ctx, label, &format!("ID:{}", id), &format!("审核人:{}", ctx.username)).await;
                ok("状态已更新", Value::Null)
            }
            Err(_) => err(500, "数据库错误"),
        }
    } else {
        let upd = sqlx::query("UPDATE wallpapers SET status = ? WHERE id = ?")
            .bind(&status).bind(id).execute(pool).await;
        match upd {
            Ok(_) => {
                log_operation(pool, ctx, "修改壁纸状态", &format!("ID:{}", id), &format!("状态:{}", status)).await;
                ok("状态已更新", Value::Null)
            }
            Err(_) => err(500, "数据库错误"),
        }
    }
}

/// 获取桌面端用户壁纸上传总数上限；0 表示不限制
pub async fn get_wallpaper_upload_limit(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let limit = read_global_wallpaper_upload_limit(pool).await;
    ok("ok", json!({ "wallpaper_upload_limit": limit }))
}

/// 修改桌面端用户壁纸上传总数上限；0 表示不限制
pub async fn update_wallpaper_upload_limit(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let limit = int_of(&data, "wallpaper_upload_limit");
    if limit < 0 || limit > 10000 {
        return err(400, "上传上限需在 0 到 10000 之间");
    }
    let limit_text = limit.to_string();
    let result = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description)
         VALUES ('wallpaper_upload_limit', ?, '每个用户最多可上传的壁纸数量，0 表示不限制')
         ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), description = VALUES(description)",
    )
    .bind(&limit_text)
    .execute(pool)
    .await;
    match result {
        Ok(_) => {
            log_operation(
                pool,
                ctx,
                "更新壁纸上传上限",
                "wallpaper_upload_limit",
                &format!("limit={}", limit),
            )
            .await;
            ok("保存成功", json!({ "wallpaper_upload_limit": limit }))
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 获取账号级壁纸上传限制列表
pub async fn list_wallpaper_account_limits(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    ensure_wallpaper_upload_limits_table(pool).await;
    let rows = sqlx::query(
        "SELECT
            l.ciyuanxi_id,
            l.upload_limit,
            l.remark,
            l.updated_by,
            l.updated_at,
            COALESCE(u.username, '') AS username,
            COALESCE(u.email, '') AS email,
            COALESCE(w.uploaded_count, 0) AS uploaded_count
         FROM wallpaper_upload_limits l
         LEFT JOIN app_users u ON u.ciyuanxi_id = l.ciyuanxi_id
         LEFT JOIN (
            SELECT uploaded_by, COUNT(*) AS uploaded_count
            FROM wallpapers
            GROUP BY uploaded_by
         ) w ON w.uploaded_by = l.ciyuanxi_id
         ORDER BY l.updated_at DESC",
    )
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            let arr: Vec<Value> = rows.iter().map(crate::admin::row_to_value).collect();
            ok("ok", json!(arr))
        }
        Err(_) => err(500, "数据库错误"),
    }
}

/// 保存账号级壁纸上传限制；0 表示该账号无限制
pub async fn save_wallpaper_account_limit(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    ensure_wallpaper_upload_limits_table(pool).await;
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let limit = int_of(&data, "upload_limit");
    let remark = str_of(&data, "remark").trim().to_string();

    if ciyuanxi_id.is_empty() {
        return err(400, "请填写弦予号");
    }
    if limit < 0 || limit > 10000 {
        return err(400, "账号上传上限需在 0 到 10000 之间");
    }
    let user_exists = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if !user_exists {
        return err(404, "账号不存在，请检查弦予号");
    }

    let result = sqlx::query(
        "INSERT INTO wallpaper_upload_limits (ciyuanxi_id, upload_limit, remark, updated_by)
         VALUES (?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
            upload_limit = VALUES(upload_limit),
            remark = VALUES(remark),
            updated_by = VALUES(updated_by),
            updated_at = NOW()",
    )
    .bind(&ciyuanxi_id)
    .bind(limit)
    .bind(&remark)
    .bind(&ctx.username)
    .execute(pool)
    .await;
    match result {
        Ok(_) => {
            log_operation(
                pool,
                ctx,
                "更新账号壁纸上传上限",
                &ciyuanxi_id,
                &format!("limit={}", limit),
            )
            .await;
            ok("保存成功", json!({
                "ciyuanxi_id": ciyuanxi_id,
                "upload_limit": limit,
            }))
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 删除账号级壁纸上传限制，恢复使用全局默认
pub async fn delete_wallpaper_account_limit(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    ensure_wallpaper_upload_limits_table(pool).await;
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return err(400, "请填写弦予号");
    }
    let result = sqlx::query("DELETE FROM wallpaper_upload_limits WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    match result {
        Ok(_) => {
            log_operation(pool, ctx, "删除账号壁纸上传上限", &ciyuanxi_id, "恢复全局默认").await;
            ok("已恢复全局默认", Value::Null)
        }
        Err(_) => err(500, "服务器错误"),
    }
}
