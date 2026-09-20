use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;

use crate::audit_policy::{self, AuditDecision};
use crate::handlers::helpers::{parse_body, random_hex, str_of};
use crate::response::ReqCtx;

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

pub async fn upload_cover(body: &str, ctx: ReqCtx, _pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let image_data = str_of(&data, "image_data").to_string();
    if !image_data.starts_with("data:image/") {
        return ctx.err(400, "无效的图片数据格式");
    }
    if image_data.len() > 2 * 1024 * 1024 {
        return ctx.err(400, "图片数据过大，请使用更小的图片");
    }
    let Some(bytes) = data_url_to_bytes(&image_data) else {
        return ctx.err(400, "无效的图片数据");
    };
    let valid_ext = image::guess_format(&bytes)
        .map(|f| {
            matches!(
                f,
                image::ImageFormat::Jpeg
                    | image::ImageFormat::Png
                    | image::ImageFormat::WebP
                    | image::ImageFormat::Gif
            )
        })
        .unwrap_or(false);
    if !valid_ext {
        return ctx.err(400, "只支持 JPG / PNG / WEBP / GIF 格式");
    }

    let dir = std::path::Path::new("uploads").join("covers");
    if std::fs::create_dir_all(&dir).is_err() {
        return ctx.err(500, "无法创建上传目录");
    }

    let file_name = format!("cover_{}.jpg", random_hex(12));
    let path = dir.join(&file_name);
    if !compress_and_save_image(&bytes, &path, 800, 82) {
        return ctx.err(500, "图片保存失败，请检查目录权限");
    }

    let url = format!("/uploads/covers/{}", file_name);
    ctx.ok("ok", json!({ "cover_url": public_url(&ctx, url) }))
}

async fn user_active(pool: &MySqlPool, ciyuanxi_id: &str) -> bool {
    sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? AND status = 1")
        .bind(ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some()
}

async fn avatar_submit_block_message(pool: &MySqlPool, ciyuanxi_id: &str) -> Option<&'static str> {
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM user_avatar_pending WHERE ciyuanxi_id = ? AND created_at >= DATE(NOW() + INTERVAL 8 HOUR) - INTERVAL 8 HOUR AND created_at < DATE(NOW() + INTERVAL 8 HOUR) + INTERVAL 16 HOUR ORDER BY id DESC LIMIT 1",
    )
    .bind(ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match status.as_deref() {
        Some("pending") => Some("头像正在审核中哦"),
        Some(_) => Some("今日已修改过啦"),
        None => None,
    }
}

pub async fn upload_avatar(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    }
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "用户ID不能为空");
    }
    let avatar_data = str_of(&data, "avatar_data").to_string();
    if !avatar_data.starts_with("data:image/") {
        return ctx.err(400, "无效的图片数据格式");
    }
    if avatar_data.len() > 200 * 1024 {
        return ctx.err(400, "图片数据过大，请使用更小的图片");
    }
    if !user_active(pool, &ciyuanxi_id).await {
        return ctx.err(404, "用户不存在");
    }
    if let Some(msg) = avatar_submit_block_message(pool, &ciyuanxi_id).await {
        return ctx.err(429, msg);
    }
    let old_avatar: String = sqlx::query_scalar("SELECT COALESCE(avatar_url, '') FROM app_users WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .fetch_one(pool)
        .await
        .unwrap_or_default();
    let audit = audit_policy::audit_image(
        pool,
        "avatar",
        &avatar_data,
        json!({ "ciyuanxi_id": ciyuanxi_id }),
    )
    .await;
    if audit.decision == AuditDecision::Pass {
        let _ = sqlx::query("UPDATE app_users SET avatar_url = ? WHERE ciyuanxi_id = ?")
            .bind(&avatar_data)
            .bind(&ciyuanxi_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("UPDATE user_feedback SET nickname = (SELECT nickname FROM app_users WHERE ciyuanxi_id = ?) WHERE ciyuanxi_id = ?")
            .bind(&ciyuanxi_id)
            .bind(&ciyuanxi_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, old_avatar, status, reviewed_at, reviewed_by) VALUES (?, ?, ?, 'approved', NOW(), ?)")
            .bind(&ciyuanxi_id)
            .bind(&avatar_data)
            .bind(&old_avatar)
            .bind(format!("external:{}", audit.provider))
            .execute(pool)
            .await;
        return ctx.ok("头像已通过机审并立即生效", json!({ "status": "approved" }));
    }
    if audit.decision == AuditDecision::Reject {
        let _ = sqlx::query("DELETE FROM user_avatar_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
            .bind(&ciyuanxi_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, old_avatar, status, reviewed_at, reviewed_by) VALUES (?, ?, ?, 'rejected', NOW(), ?)")
            .bind(&ciyuanxi_id)
            .bind(&avatar_data)
            .bind(&old_avatar)
            .bind(format!("external:{}", audit.provider))
            .execute(pool)
            .await;
        return ctx.err(400, if audit.reason.is_empty() { "头像未通过机审" } else { &audit.reason });
    }
    let _ = sqlx::query("DELETE FROM user_avatar_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let ins = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, old_avatar, status) VALUES (?,?, ?, 'pending')")
        .bind(&ciyuanxi_id)
        .bind(&avatar_data)
        .bind(&old_avatar)
        .execute(pool)
        .await;
    match ins {
        Ok(_) => {
            crate::admin::email::notify_external_emails_for_module(
                pool,
                &ctx.config,
                &ctx.client_ip,
                "avatar",
                "【弦予后台】新头像待审核",
                &format!("用户 {} 提交了新头像，请及时审核。", ciyuanxi_id),
                &avatar_data,
                &ctx.base_url,
            ).await;
            ctx.ok("头像已上传，等待管理员审核", json!({ "status": "pending" }))
        }
        Err(e) => { tracing::error!("服务器错误: {e}"); ctx.err(500, "服务器错误") },
    }
}
