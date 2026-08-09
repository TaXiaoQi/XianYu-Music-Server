use axum::response::Response;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};

use crate::audit_policy::{self, AuditDecision};
use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

const DEFAULT_WALLPAPER_UPLOAD_LIMIT: i64 = 20;

fn wallpaper_dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").join("wallpapers")
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

async fn read_effective_wallpaper_upload_limit(pool: &MySqlPool, ciyuanxi_id: &str) -> i64 {
    let user_limit = sqlx::query_scalar::<_, i64>(
        "SELECT upload_limit FROM wallpaper_upload_limits WHERE ciyuanxi_id = ? LIMIT 1",
    )
    .bind(ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match user_limit {
        Some(limit) if limit >= 0 => limit,
        _ => read_global_wallpaper_upload_limit(pool).await,
    }
}

fn public_url(ctx: &ReqCtx, url: String) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url;
    }
    if ctx.base_url.is_empty() {
        return url;
    }
    format!("{}{}", ctx.base_url.trim_end_matches('/'), url)
}

fn data_url_to_bytes(data_url: &str) -> Option<Vec<u8>> {
    let raw = data_url.split_once(',').map(|(_, v)| v).unwrap_or(data_url);
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(raw).ok()
}

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

fn row_to_wallpaper(ctx: &ReqCtx, row: &sqlx::mysql::MySqlRow) -> Value {
    let id: i64 = row.try_get::<i64, _>("id").unwrap_or_else(|_| {
        row.try_get::<i32, _>("id").map(|v| v as i64).unwrap_or_default()
    });
    let image_url = public_url(ctx, row.try_get::<String, _>("image_url").unwrap_or_default());
    let thumbnail_url = public_url(ctx, row.try_get::<String, _>("thumbnail_url").unwrap_or_default());
    json!({
        "id": id,
        "title": row.try_get::<String, _>("title").unwrap_or_default(),
        "description": row.try_get::<String, _>("description").unwrap_or_default(),
        "imageUrl": image_url,
        "thumbnailUrl": thumbnail_url,
        "category": row.try_get::<String, _>("category").unwrap_or_default(),
        "uploaderId": row.try_get::<String, _>("uploaded_by").unwrap_or_default(),
        "uploaderNickname": row.try_get::<String, _>("uploaded_by_nickname").unwrap_or_default(),
        "status": row.try_get::<String, _>("status").unwrap_or_default(),
        "reviewedAt": row.try_get::<String, _>("reviewed_at").ok(),
        "reviewedBy": row.try_get::<String, _>("reviewed_by").unwrap_or_default(),
        "createdAt": row.try_get::<String, _>("created_at").ok(),
    })
}

pub async fn list_wallpapers(_body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let rows = sqlx::query(
        "SELECT * FROM wallpapers WHERE status = 'normal' ORDER BY sort_order DESC, id DESC",
    )
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(|r| row_to_wallpaper(&ctx, r)).collect();
            ctx.ok("ok", list)
        }
        Err(_) => ctx.err(500, "数据库错误"),
    }
}

pub async fn my_wallpapers(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let rows = sqlx::query(
        "SELECT * FROM wallpapers WHERE uploaded_by = ? ORDER BY id DESC",
    )
    .bind(&ciyuanxi_id)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(|r| row_to_wallpaper(&ctx, r)).collect();
            ctx.ok("ok", list)
        }
        Err(_) => ctx.err(500, "数据库错误"),
    }
}

pub async fn upload_wallpaper(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let nickname = str_of(&data, "nickname").trim().to_string();
    let title = str_of(&data, "title").trim().to_string();
    let description = str_of(&data, "description").trim().to_string();
    let mut category = str_of(&data, "category").trim().to_string();
    let image_data = str_of(&data, "image_data");

    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "请先登录");
    }
    if title.is_empty() {
        return ctx.err(400, "请填写壁纸标题");
    }
    if image_data.is_empty() {
        return ctx.err(400, "请选择壁纸图片");
    }
    if category.is_empty() {
        category = "用户上传".to_string();
    }

    let user_exists = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? AND status = 1 LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if !user_exists {
        return ctx.err(404, "用户不存在");
    }

    let upload_limit = read_effective_wallpaper_upload_limit(pool, &ciyuanxi_id).await;
    if upload_limit > 0 {
        let current_count: i64 = match sqlx::query_scalar(
            "SELECT COUNT(*) FROM wallpapers WHERE uploaded_by = ?",
        )
        .bind(&ciyuanxi_id)
        .fetch_one(pool)
        .await {
            Ok(v) => v,
            Err(_) => return ctx.err(500, "数据库错误"),
        };
        if current_count >= upload_limit {
            return ctx.err(400, &format!("每个用户最多只能上传 {} 张壁纸", upload_limit));
        }
    }

    let Some(bytes) = data_url_to_bytes(&image_data) else {
        return ctx.err(400, "无效的图片数据");
    };
    let valid_ext = image::guess_format(&bytes)
        .map(|f| matches!(f, image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP | image::ImageFormat::Gif))
        .unwrap_or(false);
    if !valid_ext {
        return ctx.err(400, "只支持 JPG / PNG / WEBP / GIF 格式");
    }

    let dir = wallpaper_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return ctx.err(500, "无法创建上传目录");
    }

    let audit = audit_policy::audit_image(
        pool,
        "wallpaper",
        &image_data,
        json!({ "ciyuanxi_id": ciyuanxi_id, "title": title, "category": category }),
    )
    .await;
    let initial_status = match audit.decision {
        AuditDecision::Pass => "normal",
        AuditDecision::Reject => "rejected",
        AuditDecision::Manual => "pending",
    };
    let reviewed_by = if initial_status == "pending" {
        String::new()
    } else {
        format!("external:{}", audit.provider)
    };

    let ins = sqlx::query(
        "INSERT INTO wallpapers (title, description, category, image_url, thumbnail_url, status, uploaded_by, uploaded_by_nickname, reviewed_at, reviewed_by) VALUES (?, ?, ?, '', '', ?, ?, ?, IF(? = 'pending', NULL, NOW()), ?)",
    )
    .bind(&title)
    .bind(&description)
    .bind(&category)
    .bind(initial_status)
    .bind(&ciyuanxi_id)
    .bind(&nickname)
    .bind(initial_status)
    .bind(&reviewed_by)
    .execute(pool)
    .await;
    let wp_id = match ins {
        Ok(r) => r.last_insert_id() as i64,
        Err(_) => return ctx.err(500, "数据库错误"),
    };

    let main_path = dir.join(format!("wallpaper_{}.jpg", wp_id));
    let thumb_path = dir.join(format!("thumb_{}.jpg", wp_id));
    if !compress_and_save_image(&bytes, &main_path, 1920, 82) {
        let _ = sqlx::query("DELETE FROM wallpapers WHERE id = ?").bind(wp_id).execute(pool).await;
        return ctx.err(500, "图片保存失败，请检查目录权限");
    }
    if !compress_and_save_image(&bytes, &thumb_path, 480, 72) {
        let _ = std::fs::copy(&main_path, &thumb_path);
    }

    let image_url = format!("/uploads/wallpapers/wallpaper_{}.jpg", wp_id);
    let thumb_url = format!("/uploads/wallpapers/thumb_{}.jpg", wp_id);
    let _ = sqlx::query("UPDATE wallpapers SET image_url = ?, thumbnail_url = ? WHERE id = ?")
        .bind(&image_url)
        .bind(&thumb_url)
        .bind(wp_id)
        .execute(pool)
        .await;

    let msg = match initial_status {
        "normal" => "上传成功，已通过机审",
        "rejected" => if audit.reason.is_empty() { "上传成功，但未通过机审" } else { audit.reason.as_str() },
        _ => "上传成功，等待管理员审核",
    };
    ctx.ok(msg, json!({
        "id": wp_id,
        "status": initial_status,
        "imageUrl": public_url(&ctx, image_url),
        "thumbnailUrl": public_url(&ctx, thumb_url),
    }))
}
