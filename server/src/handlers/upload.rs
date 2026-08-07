use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

/// 校验用户存在且启用，返回是否通过
async fn user_active(pool: &MySqlPool, ciyuanxi_id: &str) -> bool {
    sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? AND status = 1")
        .bind(ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some()
}

/// 头像上传（JSON base64 模式，与 PHP 模式1对应）
/// 入参：ciyuanxi_id/user_id, avatar_data(data:image/xxx;base64,...)
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
    let _ = sqlx::query("DELETE FROM user_avatar_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let ins = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, status) VALUES (?,?, 'pending')")
        .bind(&ciyuanxi_id)
        .bind(&avatar_data)
        .execute(pool)
        .await;
    match ins {
        Ok(_) => ctx.ok("头像已上传，等待管理员审核", json!({ "status": "pending" })),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

/// 个人背景上传（JSON base64 模式适配，updates app_users.background_url）
/// 入参：ciyuanxi_id/user_id, background_data(data url base64), ext(可选 jpg/png/webp)
pub async fn upload_background(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    }
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "用户ID不能为空");
    }
    let b64 = str_of(&data, "background_data").to_string();
    if b64.is_empty() || !b64.starts_with("data:image/") {
        return ctx.err(400, "无效的图片数据格式");
    }
    if !user_active(pool, &ciyuanxi_id).await {
        return ctx.err(404, "用户不存在");
    }
    // 简易文件落盘（保持与 PHP 同名路径约定）
    let ext = image_ext(&b64).unwrap_or("png");
    let filename = format!("{}_background.{}", ciyuanxi_id, ext);
    let dir = std::path::Path::new("uploads").join("backgrounds");
    let _ = std::fs::create_dir_all(&dir);
    let target = dir.join(&filename);
    if let Some(bytes) = decode_data_url(&b64) {
        if std::fs::write(&target, bytes).is_err() {
            return ctx.err(500, "文件保存失败");
        }
    } else {
        return ctx.err(500, "图片数据解析失败");
    }
    let url = format!("/uploads/backgrounds/{}", filename);
    let _ = sqlx::query("UPDATE app_users SET background_url = ? WHERE ciyuanxi_id = ?")
        .bind(&url)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    ctx.ok("上传成功", json!({ "background_url": url }))
}

fn image_ext(data_url: &str) -> Option<&'static str> {
    if data_url.starts_with("data:image/jpeg") || data_url.starts_with("data:image/jpg") {
        Some("jpg")
    } else if data_url.starts_with("data:image/png") {
        Some("png")
    } else if data_url.starts_with("data:image/webp") {
        Some("webp")
    } else {
        None
    }
}

fn decode_data_url(data_url: &str) -> Option<Vec<u8>> {
    let idx = data_url.find(',')?;
    let payload = &data_url[idx + 1..];
    let payload = if let Some(rem) = payload.strip_prefix("base64;") {
        rem
    } else {
        // 兼容 "data:image/png;base64,xxx" 结构
        payload
    };
    // 兼容 mime;base64 前缀解析：分割出分号后函数体
    let payload = raw_base64(payload);
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(payload).ok()
}

fn raw_base64(s: &str) -> &str {
    // 若出现两次分号（如 data:image/webp;base64,），base64 段已由主函数截取，无需再处理
    s
}

/// 查询当前用户头像审核状态（供 dispatch 复用；也可走 settings::get_avatar_status）
#[allow(dead_code)]
pub async fn get_avatar_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    }
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "用户ID不能为空");
    }
    let row = sqlx::query(
        "SELECT status, created_at FROM user_avatar_pending WHERE ciyuanxi_id = ? AND status != 'approved' ORDER BY id DESC LIMIT 1",
    )
    .bind(&ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match row {
        Some(r) => {
            let status: String = r.try_get("status").unwrap_or_default();
            let created_at: String = r.try_get("created_at").unwrap_or_default();
            ctx.ok("成功", json!({ "status": status, "created_at": created_at }))
        }
        None => ctx.ok("成功", json!({ "status": "none" })),
    }
}

/// 供处理流程使用的值填充（避免 unused import）
#[allow(dead_code)]
fn _unused(v: &Value) {
    let _ = v;
}

/// 上传歌单封面（JSON base64 模式适配）
/// 入参：user_id(ciyuanxi_id), cover_data(data url base64)
/// 返回：cover_url, cover_path
pub async fn upload_playlist_cover(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    if !user_active(pool, &ciyuanxi_id).await {
        return ctx.err(404, "用户不存在");
    }
    let b64 = str_of(&data, "cover_data").to_string();
    if b64.is_empty() || !b64.starts_with("data:image/") {
        return ctx.err(400, "无效的图片数据格式");
    }
    let ext = image_ext(&b64).unwrap_or("png");
    let ts = chrono::Utc::now().timestamp();
    let filename = format!("pl_{}_{}_{}.{}", ciyuanxi_id, ts, rand_suffix(), ext);
    let dir = std::path::Path::new("uploads").join("playlists");
    let _ = std::fs::create_dir_all(&dir);
    let target = dir.join(&filename);
    let Some(bytes) = decode_data_url(&b64) else {
        return ctx.err(500, "文件保存失败");
    };
    if std::fs::write(&target, bytes).is_err() {
        return ctx.err(500, "文件保存失败");
    }
    let cover_url = format!("/uploads/playlists/{}", filename);
    ctx.ok(
        "上传成功",
        json!({ "cover_url": cover_url, "cover_path": filename }),
    )
}

fn rand_suffix() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 3] = rng.gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}