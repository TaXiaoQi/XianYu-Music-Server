use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

async fn ensure_wallpapers_table(pool: &MySqlPool) {
    if let Some(stmt) = crate::schema::table_statements().iter().find(|s| s.contains("`wallpapers`")) {
        let _ = sqlx::query(stmt).execute(pool).await;
    }
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