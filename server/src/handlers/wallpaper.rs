use axum::response::Response;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};

use crate::admin::wallpaper::{is_mp4, read_global_wallpaper_video_max, sha256_hex};
use crate::audit_policy::{self, AuditDecision};
use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

const DEFAULT_WALLPAPER_UPLOAD_LIMIT: i64 = 20;

fn wallpaper_dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").join("wallpapers")
}

fn normalize_platform(raw: &str) -> String {
    match raw {
        "mobile" => "mobile".to_string(),
        "watch" => "watch".to_string(),
        _ => "desktop".to_string(),
    }
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
    let base = if !ctx.base_url.is_empty() {
        &ctx.base_url
    } else if !ctx.config.public_base_url.is_empty() {
        &ctx.config.public_base_url
    } else {
        return url;
    };
    format!("{}{}", base.trim_end_matches('/'), url)
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
    // 存量视频壁纸 image_url/thumbnail_url 为空，回退到视频封面
    let image_raw: String = row.try_get::<String, _>("image_url").unwrap_or_default();
    let thumb_raw: String = row.try_get::<String, _>("thumbnail_url").unwrap_or_default();
    let poster_raw: String = row.try_get::<String, _>("video_poster").unwrap_or_default();
    let image_url = public_url(ctx, if image_raw.is_empty() { poster_raw.clone() } else { image_raw });
    let thumbnail_url = public_url(ctx, if thumb_raw.is_empty() { poster_raw.clone() } else { thumb_raw });
    let video_poster = public_url(ctx, poster_raw);
    let video_url = public_url(ctx, row.try_get::<String, _>("video_url").unwrap_or_default());
    json!({
        "id": id,
        "title": row.try_get::<String, _>("title").unwrap_or_default(),
        "description": row.try_get::<String, _>("description").unwrap_or_default(),
        "mediaType": row.try_get::<String, _>("media_type").unwrap_or_default(),
        "imageUrl": image_url,
        "thumbnailUrl": thumbnail_url,
        "videoUrl": video_url,
        "videoPoster": video_poster,
        "videoDuration": row.try_get::<i64, _>("video_duration").unwrap_or_else(|_| row.try_get::<i32, _>("video_duration").unwrap_or(0) as i64),
        "videoSize": row.try_get::<i64, _>("video_size").unwrap_or_else(|_| row.try_get::<u32, _>("video_size").unwrap_or(0) as i64),
        "videoSha256": row.try_get::<String, _>("video_sha256").unwrap_or_default(),
        "category": row.try_get::<String, _>("category").unwrap_or_default(),
        "platform": row.try_get::<String, _>("platform").unwrap_or_else(|_| "desktop".to_string()),
        "uploaderId": row.try_get::<String, _>("uploaded_by").unwrap_or_default(),
        "uploaderNickname": row.try_get::<String, _>("uploaded_by_nickname").unwrap_or_default(),
        "status": row.try_get::<String, _>("status").unwrap_or_default(),
        "reviewedAt": row.try_get::<String, _>("reviewed_at").ok(),
        "reviewedBy": row.try_get::<String, _>("reviewed_by").unwrap_or_default(),
        "createdAt": row.try_get::<String, _>("created_at").ok(),
    })
}

pub async fn list_wallpapers(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let platform = normalize_platform(str_of(&data, "platform").trim());
    let media_type = str_of(&data, "media_type").trim().to_string();
    let rows = if media_type == "image" || media_type == "video" {
        sqlx::query(
            "SELECT * FROM wallpapers WHERE status = 'normal' AND platform = ? AND media_type = ? ORDER BY sort_order DESC, id DESC",
        )
        .bind(&platform)
        .bind(media_type)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            "SELECT * FROM wallpapers WHERE status = 'normal' AND platform = ? ORDER BY sort_order DESC, id DESC",
        )
        .bind(&platform)
        .fetch_all(pool)
        .await
    };
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
    let platform = normalize_platform(str_of(&data, "platform").trim());
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let rows = sqlx::query(
        "SELECT * FROM wallpapers WHERE uploaded_by = ? AND platform = ? ORDER BY id DESC",
    )
    .bind(&ciyuanxi_id)
    .bind(&platform)
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
    let platform = normalize_platform(str_of(&data, "platform").trim());
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

    let video_data = str_of(&data, "video_data").trim().to_string();
    if !video_data.is_empty() {
        // 展示模式：缺省 video 兼容旧客户端
        let media_type = match data.get("media_type").and_then(|v| v.as_str()).map(str::trim) {
            Some("image") => "image",
            _ => "video",
        };
        return upload_wallpaper_video(
            ctx,
            pool,
            &ciyuanxi_id,
            &nickname,
            &title,
            &description,
            &category,
            &platform,
            &image_data,
            &video_data,
            data.get("video_duration").and_then(|v| v.as_i64()).unwrap_or(0),
            media_type,
        )
        .await;
    }

    let Some(bytes) = data_url_to_bytes(&image_data) else {
        return ctx.err(400, "无效的图片数据");
    };
    if bytes.len() > 8 * 1024 * 1024 {
        return ctx.err(400, "图片过大，请控制在 8MB 以内");
    }
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
        json!({ "ciyuanxi_id": ciyuanxi_id, "title": title, "category": category, "platform": platform }),
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
        "INSERT INTO wallpapers (title, description, category, platform, image_url, thumbnail_url, status, uploaded_by, uploaded_by_nickname, reviewed_at, reviewed_by) VALUES (?, ?, ?, ?, '', '', ?, ?, ?, IF(? = 'pending', NULL, NOW()), ?)",
    )
    .bind(&title)
    .bind(&description)
    .bind(&category)
    .bind(&platform)
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

    if initial_status == "pending" {
        crate::admin::email::notify_external_emails_for_module(
            pool,
            &ctx.config,
            &ctx.client_ip,
            "wallpaper",
            "【弦予后台】新壁纸待审核",
            &format!("用户 {} 上传了壁纸「{}」，请及时审核。", ciyuanxi_id, title),
            &public_url(&ctx, image_url.clone()),
            &ctx.base_url,
        ).await;
    }

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

#[allow(clippy::too_many_arguments)]
async fn upload_wallpaper_video(
    ctx: ReqCtx,
    pool: &MySqlPool,
    ciyuanxi_id: &str,
    nickname: &str,
    title: &str,
    description: &str,
    category: &str,
    platform: &str,
    image_data: &str,
    video_data: &str,
    video_duration: i64,
    media_type: &str,
) -> Response {
    let Some(video_bytes) = data_url_to_bytes(video_data) else {
        return ctx.err(400, "无效的视频数据");
    };
    let max_mb = read_global_wallpaper_video_max(pool).await;
    if (video_bytes.len() as i64) > max_mb * 1024 * 1024 {
        return ctx.err(400, &format!("视频过大，请控制在 {}MB 以内", max_mb));
    }
    if !is_mp4(&video_bytes) {
        return ctx.err(400, "仅支持 MP4 视频");
    }
    if video_duration < 0 || video_duration > 86400 {
        return ctx.err(400, "视频时长无效");
    }
    // image_data 作为视频封面（poster + 列表缩略图）
    let Some(poster_bytes) = data_url_to_bytes(image_data) else {
        return ctx.err(400, "无效的封面图片数据");
    };
    if poster_bytes.len() > 8 * 1024 * 1024 {
        return ctx.err(400, "封面图片过大，请控制在 8MB 以内");
    }
    let poster_valid = image::guess_format(&poster_bytes)
        .map(|f| matches!(f, image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP | image::ImageFormat::Gif))
        .unwrap_or(false);
    if !poster_valid {
        return ctx.err(400, "封面只支持 JPG / PNG / WEBP / GIF 格式");
    }

    let dir = wallpaper_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return ctx.err(500, "无法创建上传目录");
    }

    let audit = audit_policy::audit_image(
        pool,
        "wallpaper",
        image_data,
        json!({ "ciyuanxi_id": ciyuanxi_id, "title": title, "category": category, "platform": platform }),
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
        "INSERT INTO wallpapers (title, description, category, platform, media_type, video_url, video_poster, video_duration, video_size, video_sha256, image_url, thumbnail_url, status, uploaded_by, uploaded_by_nickname, reviewed_at, reviewed_by) VALUES (?, ?, ?, ?, ?, '', '', ?, ?, ?, '', '', ?, ?, ?, IF(? = 'pending', NULL, NOW()), ?)",
    )
    .bind(title)
    .bind(description)
    .bind(category)
    .bind(platform)
    .bind(media_type)
    .bind(video_duration)
    .bind(video_bytes.len() as i64)
    .bind(sha256_hex(&video_bytes))
    .bind(initial_status)
    .bind(ciyuanxi_id)
    .bind(nickname)
    .bind(initial_status)
    .bind(&reviewed_by)
    .execute(pool)
    .await;
    let wp_id = match ins {
        Ok(r) => r.last_insert_id() as i64,
        Err(_) => return ctx.err(500, "数据库错误"),
    };

    let video_path = dir.join(format!("wallpaper_{}.mp4", wp_id));
    if std::fs::write(&video_path, &video_bytes).is_err() {
        let _ = sqlx::query("DELETE FROM wallpapers WHERE id = ?").bind(wp_id).execute(pool).await;
        return ctx.err(500, "视频保存失败，请检查目录权限");
    }
    let poster_path = dir.join(format!("poster_{}.jpg", wp_id));
    let thumb_path = dir.join(format!("thumb_{}.jpg", wp_id));
    if !compress_and_save_image(&poster_bytes, &poster_path, 1920, 82) {
        let _ = sqlx::query("DELETE FROM wallpapers WHERE id = ?").bind(wp_id).execute(pool).await;
        let _ = std::fs::remove_file(&video_path);
        return ctx.err(500, "封面保存失败，请检查目录权限");
    }
    if !compress_and_save_image(&poster_bytes, &thumb_path, 480, 72) {
        let _ = std::fs::copy(&poster_path, &thumb_path);
    }

    let video_url = format!("/uploads/wallpapers/wallpaper_{}.mp4", wp_id);
    let poster_url = format!("/uploads/wallpapers/poster_{}.jpg", wp_id);
    let thumb_url = format!("/uploads/wallpapers/thumb_{}.jpg", wp_id);
    // image_url/thumbnail_url 指向封面与缩略图，保证「展示图片」模式下客户端可下载静帧
    let _ = sqlx::query("UPDATE wallpapers SET video_url = ?, video_poster = ?, image_url = ?, thumbnail_url = ? WHERE id = ?")
        .bind(&video_url)
        .bind(&poster_url)
        .bind(&poster_url)
        .bind(&thumb_url)
        .bind(wp_id)
        .execute(pool)
        .await;

    if initial_status == "pending" {
        crate::admin::email::notify_external_emails_for_module(
            pool,
            &ctx.config,
            &ctx.client_ip,
            "wallpaper",
            "【弦予后台】新视频壁纸待审核",
            &format!("用户 {} 上传了视频壁纸「{}」，请及时审核。", ciyuanxi_id, title),
            &public_url(&ctx, poster_url.clone()),
            &ctx.base_url,
        ).await;
    }

    let msg = match initial_status {
        "normal" => "上传成功，已通过机审",
        "rejected" => if audit.reason.is_empty() { "上传成功，但未通过机审" } else { audit.reason.as_str() },
        _ => "上传成功，等待管理员审核",
    };
    ctx.ok(msg, json!({
        "id": wp_id,
        "status": initial_status,
        "videoUrl": public_url(&ctx, video_url),
        "videoPoster": public_url(&ctx, poster_url),
    }))
}
