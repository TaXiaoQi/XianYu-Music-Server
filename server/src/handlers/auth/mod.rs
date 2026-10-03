mod account;
mod captcha;
mod login;
mod tv;

pub(crate) use axum::response::Response;
pub(crate) use serde_json::json;
pub(crate) use sqlx::MySqlPool;
pub(crate) use sqlx::Row;

pub(crate) use crate::handlers::helpers::{
    default_nickname, parse_body, str_of, validate_ciyuanxi_id, validate_nickname,
};
pub(crate) use crate::handlers::token;
pub(crate) use crate::response::ReqCtx;

pub use account::{delete_account, preverify_delete_account, reset_password, send_verify_code};
pub use captcha::{get_captcha, verify_captcha};
pub(crate) use captcha::require_captcha;
pub use login::{login_by_code, register, user_login};
pub use tv::{confirm_tv_login, generate_tv_login_code, poll_tv_login_status, scan_tv_login};
const CAPTCHA_TTL_MINUTES: i64 = 5;

const LOGIN_LOCK_THRESHOLD: i64 = 5;

const LOGIN_LOCK_MINUTES: i64 = 15;

const LOGIN_FAILURE_WINDOW_MINUTES: i64 = 30;

pub(crate) async fn check_device_ban(device_id: &str, ctx: &ReqCtx, pool: &MySqlPool) -> Option<Response> {
    if device_id.trim().is_empty() {
        return None;
    }
    let banned = sqlx::query("SELECT reason FROM banned_devices WHERE device_id = ? LIMIT 1")
        .bind(device_id.trim())
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if let Some(row) = banned {
        let reason: String = row.try_get::<String, _>("reason").unwrap_or_default();
        let reason = reason.trim();
        if reason.is_empty() {
            return Some(ctx.err(403, "该设备已被封禁，请联系管理员"));
        }
        return Some(ctx.err(403, &format!("该设备已被封禁，原因：{}。如有疑问请联系管理员", reason)));
    }
    None
}

pub async fn check_ban_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let device_id = str_of(&data, "device_id").trim().to_string();

    if !ciyuanxi_id.is_empty() {
        if let Ok(Some(row)) = sqlx::query("SELECT status, ban_reason FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
            .bind(&ciyuanxi_id)
            .fetch_optional(pool)
            .await
        {
            let status: i64 = row.try_get("status").unwrap_or(1);
            if status == 0 {
                let reason: String = row.try_get("ban_reason").unwrap_or_default();
                return ctx.ok("ok", json!({ "banned": true, "type": "account", "reason": reason }));
            }
        }
    }

    if !device_id.is_empty() {
        if let Ok(Some(row)) = sqlx::query("SELECT reason FROM banned_devices WHERE device_id = ? LIMIT 1")
            .bind(&device_id)
            .fetch_optional(pool)
            .await
        {
            let reason: String = row.try_get("reason").unwrap_or_default();
            return ctx.ok("ok", json!({ "banned": true, "type": "device", "reason": reason }));
        }
    }

    ctx.ok("ok", json!({ "banned": false }))
}

async fn resolve_role(_pool: &MySqlPool, _email: &str) -> String {
    "member".to_string()
}

pub(crate) fn build_user_payload(
    user_id: i64,
    nickname: &str,
    email: &str,
    avatar_url: &str,
    ciyuanxi_id: &str,
    status: i64,
    master_quota: i64,
    token: &str,
    role: &str,

) -> serde_json::Value {
    json!({
        "user_id": user_id,
        "nickname": nickname,
        "username": nickname,
        "email": email,
        "token": token,
        "role": role,
        "avatar_url": avatar_url,
        "ciyuanxi_id": ciyuanxi_id,
        "master_quota": master_quota,
        "status": if status == 1 { "enabled" } else { "disabled" }
    })
}
