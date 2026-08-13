use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{parse_body, str_of};

/// 站点 logo 在 server_settings 表中的 key
const SITE_LOGO_KEY: &str = "site_logo_url";

/// logo 存储目录与固定文件名（统一覆盖，保证 URL 稳定）
fn logo_dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").join("logos")
}

fn logo_relative_url() -> String {
    "/uploads/logos/logo.png".to_string()
}

/// 读取当前站点 logo URL（未配置返回空串）
pub async fn read_site_logo(pool: &MySqlPool) -> String {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT setting_value FROM server_settings WHERE setting_key = ? LIMIT 1",
    )
    .bind(SITE_LOGO_KEY)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_default()
}

/// 将相对路径拼接为完整 URL（优先 base_url，其次 config_public_base_url）
fn full_url(base_url: &str, config_public_base_url: &str, url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    let base = if !base_url.is_empty() {
        base_url
    } else if !config_public_base_url.is_empty() {
        config_public_base_url
    } else {
        return url.to_string();
    };
    format!("{}{}", base.trim_end_matches('/'), url)
}

/// 保存 logo 为 PNG（含透明通道，限制最大边长 512）
fn save_logo_png(bytes: &[u8], target: &std::path::Path) -> bool {
    use image::GenericImageView;
    let img = match image::load_from_memory(bytes) {
        Ok(i) => i,
        Err(_) => return false,
    };
    let (w, h) = img.dimensions();
    let max = 512u32;
    let (nw, nh) = if w > max || h > max {
        let scale = (max as f64 / w.max(h) as f64).min(1.0);
        (((w as f64 * scale) as u32).max(1), ((h as f64 * scale) as u32).max(1))
    } else {
        (w, h)
    };
    let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    let file = match std::fs::File::create(target) {
        Ok(f) => f,
        Err(_) => return false,
    };
    resized
        .write_to(&mut std::io::BufWriter::new(file), image::ImageFormat::Png)
        .is_ok()
}

/// 获取站点 logo（后台）
pub async fn get_site_logo(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let stored = read_site_logo(pool).await;
    let url = if stored.is_empty() { "" } else { &stored };
    ok("ok", json!({ "logo_url": url }))
}

/// 上传并更新站点 logo（覆盖旧图片）
pub async fn upload_site_logo(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let image_b64 = str_of(&data, "image").to_string();
    if image_b64.is_empty() {
        return err(400, "请选择 logo 图片");
    }
    // 兼容 data URL 前缀
    let b64 = if let Some(idx) = image_b64.find(',') {
        if image_b64.starts_with("data:") {
            &image_b64[idx + 1..]
        } else {
            &image_b64
        }
    } else {
        &image_b64
    };
    use base64::Engine;
    let bytes = match base64::engine::general_purpose::STANDARD.decode(b64.trim()) {
        Ok(b) => b,
        Err(_) => return err(400, "无效的图片数据"),
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
        return err(400, "只允许上传 JPG/PNG/WEBP/GIF 图片");
    }
    let dir = logo_dir();
    if let Err(_) = std::fs::create_dir_all(&dir) {
        return err(500, "无法创建上传目录");
    }
    let target = dir.join("logo.png");
    if !save_logo_png(&bytes, &target) {
        return err(500, "图片保存失败，请检查目录权限");
    }
    let relative = logo_relative_url();
    let full = full_url(&ctx.base_url, &ctx.config.public_base_url, &relative);
    let _ = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value) VALUES (?, ?)
         ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value)",
    )
    .bind(SITE_LOGO_KEY)
    .bind(&full)
    .execute(pool)
    .await;
    log_operation(pool, ctx, "上传站点Logo", "站点配置", &full).await;
    ok("站点 Logo 已更新", json!({ "logo_url": full }))
}