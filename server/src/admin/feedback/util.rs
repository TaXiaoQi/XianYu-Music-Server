use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::AdminCtx;

const DEFAULT_FEEDBACK_DAILY_LIMIT: i64 = 20;
pub(crate) const MAX_ADMIN_FEEDBACK_IMAGES: usize = 6;
pub(crate) const MAX_ADMIN_FEEDBACK_IMAGE_BYTES: usize = 8 * 1024 * 1024;

pub(crate) fn parse_collaborators(v: Option<&str>) -> Vec<String> {
    let s = v.unwrap_or("").trim();
    if s.is_empty() || s == "[]" {
        return Vec::new();
    }
    serde_json::from_str::<Vec<String>>(s).unwrap_or_default()
}

pub(crate) fn parse_completed_by(v: Option<&str>) -> Vec<Value> {
    let s = v.unwrap_or("").trim();
    if s.is_empty() || s == "[]" {
        return Vec::new();
    }
    serde_json::from_str::<Vec<Value>>(s).unwrap_or_default()
}

pub(crate) fn participants(assignee: &str, collaborators: &[String]) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    if !assignee.is_empty() {
        list.push(assignee.to_string());
    }
    for c in collaborators {
        if c != assignee && !list.contains(c) {
            list.push(c.clone());
        }
    }
    list
}

pub(crate) async fn push_admin_notification(pool: &MySqlPool, feedback_id: i64, to_admin: &str, from_admin: &str, ntype: &str, content: &str) {
    let _ = sqlx::query(
        "INSERT INTO feedback_admin_notifications (feedback_id, to_admin, from_admin, type, content) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(feedback_id)
    .bind(to_admin)
    .bind(from_admin)
    .bind(ntype)
    .bind(content)
    .execute(pool)
    .await;
}

fn admin_feedback_img_dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").join("feedback")
}

pub(crate) fn admin_data_url_to_bytes(data_url: &str) -> Option<Vec<u8>> {
    let raw = data_url.split_once(',').map(|(_, v)| v).unwrap_or(data_url);
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(raw).ok()
}

pub(crate) fn compress_and_save_admin_feedback_image(bytes: &[u8], name: &str, max_w: u32, quality: u32) -> Option<String> {
    use image::GenericImageView;
    let img = image::load_from_memory(bytes).ok()?;
    let (w, h) = img.dimensions();
    let (nw, nh) = if w > max_w {
        (max_w, (h * max_w / w).max(1))
    } else {
        (w, h)
    };
    let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgb8();
    let dir = admin_feedback_img_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let path = dir.join(name);
    let file = std::fs::File::create(&path).ok()?;
    if image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), quality as u8)
        .encode(&rgb, nw, nh, image::ExtendedColorType::Rgb8)
        .is_err()
    {
        return None;
    }
    Some(format!("/uploads/feedback/{}", name))
}

pub(crate) fn admin_feedback_img_url(ctx: &AdminCtx, url: String) -> String {
    if url.starts_with("http://") || url.starts_with("https://") || url.is_empty() {
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

pub(crate) fn save_admin_resolve_images(data: &Value, ctx: &AdminCtx) -> String {
    let arr = data.get("images").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if arr.is_empty() {
        return json!(Vec::<String>::new()).to_string();
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let mut urls: Vec<String> = Vec::new();
    for (i, img_val) in arr.iter().enumerate() {
        if urls.len() >= MAX_ADMIN_FEEDBACK_IMAGES {
            break;
        }
        let data_url = img_val.as_str().unwrap_or("").to_string();
        if data_url.is_empty() {
            continue;
        }
        if data_url.starts_with("http://") || data_url.starts_with("https://") || data_url.starts_with('/') {
            urls.push(data_url);
            continue;
        }
        if data_url.len() > MAX_ADMIN_FEEDBACK_IMAGE_BYTES {
            continue;
        }
        let bytes = match admin_data_url_to_bytes(&data_url) {
            Some(b) if b.len() <= MAX_ADMIN_FEEDBACK_IMAGE_BYTES => b,
            _ => continue,
        };
        let name = format!("resolve_{}_{}.jpg", ts, i);
        if let Some(url) = compress_and_save_admin_feedback_image(&bytes, &name, 1600, 82) {
            urls.push(admin_feedback_img_url(ctx, url));
        }
    }
    json!(urls).to_string()
}

pub(crate) async fn read_feedback_daily_limit(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT setting_value FROM server_settings WHERE setting_key = 'feedback_daily_limit' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .and_then(|v| v.trim().parse::<i64>().ok())
    .filter(|v| *v >= 0)
    .unwrap_or(DEFAULT_FEEDBACK_DAILY_LIMIT)
}
