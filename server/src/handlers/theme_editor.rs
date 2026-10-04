use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};

use crate::handlers::theme::{persist_new_theme, public_url};
use crate::response::ReqCtx;

/// GET /theme-editor：编辑器已剥离为 admin-web/public/theme-editor/ 纯静态站，
/// vite build 随 public/ 拷入 dist/，生产 static_dir 指向 dist 即同源自带
pub async fn serve_page(static_dir: &str) -> Response {
    let path = format!(
        "{}/theme-editor/index.html",
        static_dir.trim_end_matches(|c| c == '/' || c == '\\')
    );
    match tokio::fs::read(&path).await {
        Ok(bytes) => file_response(bytes, "text/html; charset=utf-8"),
        Err(_) => not_found(),
    }
}

/// GET /theme-editor/:filename：仅放行已知资源名，杜绝路径穿越
pub async fn serve_asset(static_dir: &str, filename: &str) -> Response {
    const ASSETS: [(&str, &str); 3] = [
        ("style.css", "text/css; charset=utf-8"),
        ("app.js", "application/javascript; charset=utf-8"),
        ("slots.js", "application/javascript; charset=utf-8"),
    ];
    let Some((_, ctype)) = ASSETS.iter().find(|(name, _)| *name == filename) else {
        return not_found();
    };
    let path = format!(
        "{}/theme-editor/{}",
        static_dir.trim_end_matches(|c| c == '/' || c == '\\'),
        filename
    );
    match tokio::fs::read(&path).await {
        Ok(bytes) => file_response(bytes, ctype),
        Err(_) => not_found(),
    }
}

fn file_response(bytes: Vec<u8>, content_type: &str) -> Response {
    (
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, content_type),
            (axum::http::header::CACHE_CONTROL, "no-cache"),
        ],
        axum::body::Body::from(bytes),
    )
        .into_response()
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        "not found",
    )
        .into_response()
}

pub fn qrcode_svg(data: &str) -> Option<String> {
    use qrcode::render::svg;
    use qrcode::{EcLevel, QrCode, Version};
    if data.is_empty() {
        return None;
    }
    let code = QrCode::with_version(data.as_bytes(), Version::Normal(6), EcLevel::M).ok()?;
    Some(
        code.render::<svg::Color>()
            .dark_color(svg::Color("#111111"))
            .light_color(svg::Color("#ffffff"))
            .quiet_zone(true)
            .build(),
    )
}

/// POST /theme-editor/upload：网页编辑器上传，token 真验（user_tokens 表），
/// 落库与客户端 upload_theme 同一套 persist_new_theme。
pub async fn upload(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let token = data
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if token.is_empty() {
        return ctx.err(401, "请先扫码登录");
    }
    let owner = crate::handlers::token::resolve_owner(pool, &token).await;
    let Some(ciyuanxi_id) = owner else {
        return ctx.err(401, "登录状态已失效，请重新扫码登录");
    };
    let user = sqlx::query("SELECT nickname, status FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(401, "账号不存在，请重新登录");
    };
    let status: i64 = user.get("status");
    if status == 0 {
        return ctx.err(403, "账号已被禁用");
    }
    let nickname: String = user
        .try_get::<String, _>("nickname")
        .unwrap_or_else(|_| ciyuanxi_id.clone());

    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let description = data
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let platform = data
        .get("platform")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if !matches!(platform.as_str(), "mobile" | "desktop") {
        return ctx.err(400, "平台仅支持 mobile / desktop");
    }
    if name.is_empty() {
        return ctx.err(400, "请填写主题名称");
    }
    if name.chars().count() > 64 {
        return ctx.err(400, "主题名称过长（最多 64 字）");
    }
    if description.chars().count() > 200 {
        return ctx.err(400, "主题简介过长（最多 200 字）");
    }
    let payload = match data.get("payload").and_then(|v| v.as_object()) {
        Some(obj) => obj.clone(),
        None => return ctx.err(400, "主题数据不能为空"),
    };
    if payload.get("accentColor").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
        return ctx.err(400, "主题缺少强调色配置");
    }
    // surfaces 值格式白名单：c=hex 颜色、o=0~1 透明度（审核编辑器会把 c 插入
    // innerHTML，这里挡住任意字符串注入）
    if let Some(surfaces) = payload.get("surfaces").and_then(|v| v.as_object()) {
        for (slot, v) in surfaces {
            let Some(obj) = v.as_object() else {
                return ctx.err(400, &format!("组件色块 {} 数据无效", slot));
            };
            for (k, val) in obj {
                match k.as_str() {
                    "c" => {
                        let c = val.as_str().unwrap_or("");
                        let hex = c.strip_prefix('#').unwrap_or("*");
                        let ok = c.is_empty()
                            || matches!(hex.len(), 3 | 4 | 6 | 8)
                                && hex.bytes().all(|b| b.is_ascii_hexdigit());
                        if !ok {
                            return ctx.err(400, &format!("组件色块 {} 颜色格式无效", slot));
                        }
                    }
                    "o" => {
                        let o = val.as_f64().unwrap_or(-1.0);
                        if !(0.0..=1.0).contains(&o) {
                            return ctx.err(400, &format!("组件色块 {} 透明度无效", slot));
                        }
                    }
                    _ => return ctx.err(400, &format!("组件色块 {} 含未知字段 {}", slot, k)),
                }
            }
        }
    }
    let preview = data
        .get("preview")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let wallpaper_id = data
        .get("wallpaperRef")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    match persist_new_theme(
        &ctx,
        pool,
        &ciyuanxi_id,
        &nickname,
        &name,
        &description,
        &platform,
        &payload,
        &preview,
        wallpaper_id,
    )
    .await
    {
        Ok((id, status, preview_url)) => ctx.ok(
            match status.as_str() {
                "normal" => "上传成功，已通过机审",
                "rejected" => "上传成功，但未通过机审",
                _ => "上传成功，等待管理员审核",
            },
            json!({
                "id": id,
                "status": status,
                "previewUrl": public_url(&ctx, preview_url),
            }),
        ),
        Err((code, msg)) => ctx.err(code, &msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qrcode_svg_renders_for_tv_login_payload() {
        let svg = qrcode_svg("xianyumusic://tvlogin/0123456789abcdef0123456789abcdef").expect("svg");
        assert!(svg.contains("<svg"));
        assert!(qrcode_svg("").is_none());
    }
}