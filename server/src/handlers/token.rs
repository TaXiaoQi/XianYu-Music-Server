use axum::response::Response;
use serde_json::Value;
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::response::ReqCtx;

const TOKEN_TTL_DAYS: i64 = 30;
const TOKEN_RENEW_THRESHOLD_DAYS: i64 = 15;
const MAX_TOKENS_PER_USER: i64 = 10;
const TOKEN_CACHE_TTL_SECONDS: i64 = 15;
const TOKEN_TOUCH_INTERVAL_SECONDS: i64 = 300;
const TOKEN_CACHE_MAX_ENTRIES: usize = 4096;

#[derive(Clone)]
struct CachedToken {
    ciyuanxi_id: String,
    expires_unix: i64,
    cached_at: i64,
}

fn token_cache() -> &'static Mutex<HashMap<String, CachedToken>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedToken>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

const USER_BOUND_ACTIONS: &[&str] = &[
    // settings
    "get_user_info",
    "get_user_settings",
    "update_user_settings",
    "update_profile",
    "change_password",
    "update_ciyuanxi_id",
    "bind_email",
    "get_avatar_status",
    "get_nickname_status",
    "report_listen_stats",
    "get_listen_stats",
    "deduct_master_quota",
    "get_master_quota_usage",
    // recommend
    "get_daily_recommend",
    // social
    "submit_feedback",
    "submit_appeal",
    "get_my_feedback_notifications",
    "confirm_feedback_notification",
    "get_nickname_change_notices",
    "confirm_nickname_change_notice",
    "list_my_feedback",
    // wallpaper
    "my_wallpapers",
    "upload_wallpaper",
    // playlist
    "delete_playlist",
    // file / settings / favorites / plugin sync
    "file_sync_upload_start",
    "file_sync_upload_chunk",
    "file_sync_upload_finish",
    "file_sync_download",
    "plugin_sync_upload_one",
    "plugin_sync_download",
    "settings_sync_upload",
    "settings_sync_download",
    "favorites_sync_upload",
    "favorites_sync_download",
    "listen_stats_sync_upload",
    "listen_stats_sync_download",
    // upload
    "upload_avatar",
    // account lifecycle
    "delete_account",
    "preverify_delete_account",
    // watch 联动：命令中继 + 设备在线状态（需本账号 token）
    "watch_submit_command",
    "watch_poll_command",
    "watch_phone_ping",
    "watch_phone_query",
];

const VIEW_OTHER_ACTIONS: &[&str] = &[
    "favorites_sync_download",
    "file_sync_download",
];

pub async fn issue(pool: &MySqlPool, ciyuanxi_id: &str, device_id: &str) -> String {
    let token = crate::handlers::helpers::random_hex(32);
    let _ = sqlx::query("DELETE FROM user_tokens WHERE ciyuanxi_id = ? AND device_id = ?")
        .bind(ciyuanxi_id)
        .bind(device_id)
        .execute(pool)
        .await;
    let _ = sqlx::query(
        "INSERT INTO user_tokens (token, ciyuanxi_id, device_id, expires_at) VALUES (?, ?, ?, DATE_ADD(NOW(), INTERVAL ? DAY))",
    )
    .bind(&token)
    .bind(ciyuanxi_id)
    .bind(device_id)
    .bind(TOKEN_TTL_DAYS)
    .execute(pool)
    .await;
    prune(pool, ciyuanxi_id).await;
    token
}

pub async fn revoke_user(pool: &MySqlPool, ciyuanxi_id: &str) {
    let _ = sqlx::query("DELETE FROM user_tokens WHERE ciyuanxi_id = ?")
        .bind(ciyuanxi_id)
        .execute(pool)
        .await;
}

async fn prune(pool: &MySqlPool, ciyuanxi_id: &str) {
    let _ = sqlx::query("DELETE FROM user_tokens WHERE expires_at < NOW()")
        .execute(pool)
        .await;
    let _ = sqlx::query(
        "DELETE FROM user_tokens WHERE ciyuanxi_id = ? AND id NOT IN (
            SELECT id FROM (
                SELECT id FROM user_tokens WHERE ciyuanxi_id = ? ORDER BY id DESC LIMIT ?
            ) recent
        )",
    )
    .bind(ciyuanxi_id)
    .bind(ciyuanxi_id)
    .bind(MAX_TOKENS_PER_USER)
    .execute(pool)
    .await;
}

enum OwnerState {
    Valid,
    Expired,
    Mismatch,
    Unknown,
}

async fn verify_owner(pool: &MySqlPool, token: &str, identity: &str) -> OwnerState {
    let now = now_unix();
    if let Some(cached) = token_cache().lock().unwrap().get(token).cloned() {
        if now - cached.cached_at < TOKEN_CACHE_TTL_SECONDS {
            if cached.expires_unix <= now {
                return OwnerState::Expired;
            }
            return if cached.ciyuanxi_id == identity {
                OwnerState::Valid
            } else {
                OwnerState::Mismatch
            };
        }
    }

    let row = sqlx::query(
        "SELECT ciyuanxi_id, UNIX_TIMESTAMP(expires_at) AS expires_unix,
                (expires_at < DATE_ADD(NOW(), INTERVAL ? DAY)) AS need_renew,
                (last_used_at IS NULL OR last_used_at < DATE_SUB(NOW(), INTERVAL ? SECOND)) AS should_touch
         FROM user_tokens WHERE token = ? AND expires_at > NOW() LIMIT 1",
    )
    .bind(TOKEN_RENEW_THRESHOLD_DAYS)
    .bind(TOKEN_TOUCH_INTERVAL_SECONDS)
    .bind(token)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    if let Some(row) = row {
        let owner: String = row.get("ciyuanxi_id");
        if owner != identity {
            return OwnerState::Mismatch;
        }
        let expires_unix: i64 = row.try_get("expires_unix").unwrap_or(0);
        let need_renew: i64 = row.try_get("need_renew").unwrap_or(0);
        let should_touch: i64 = row.try_get("should_touch").unwrap_or(1);
        if need_renew == 1 {
            let _ = sqlx::query(
                "UPDATE user_tokens SET last_used_at = NOW(), expires_at = DATE_ADD(NOW(), INTERVAL ? DAY) WHERE token = ?",
            )
            .bind(TOKEN_TTL_DAYS)
            .bind(token)
            .execute(pool)
            .await;
        } else if should_touch == 1 {
            let _ = sqlx::query("UPDATE user_tokens SET last_used_at = NOW() WHERE token = ?")
                .bind(token)
                .execute(pool)
                .await;
        }
        let mut cache = token_cache().lock().unwrap();
        if cache.len() > TOKEN_CACHE_MAX_ENTRIES {
            cache.retain(|_, v| v.expires_unix > now);
        }
        cache.insert(
            token.to_string(),
            CachedToken {
                ciyuanxi_id: owner,
                expires_unix,
                cached_at: now,
            },
        );
        return OwnerState::Valid;
    }
    let exists = sqlx::query("SELECT id FROM user_tokens WHERE token = ? LIMIT 1")
        .bind(token)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if exists {
        OwnerState::Expired
    } else {
        OwnerState::Unknown
    }
}

pub async fn check_dispatch_auth(
    action: &str,
    body: &str,
    ctx: &ReqCtx,
    pool: &MySqlPool,
) -> Option<Response> {
    if !USER_BOUND_ACTIONS.contains(&action) {
        return None;
    }
    let data: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let identity = ["ciyuanxi_id", "user_id"]
        .iter()
        .map(|k| match data.get(*k) {
            Some(Value::String(s)) => s.trim().to_string(),
            Some(Value::Number(n)) => n.to_string(),
            _ => String::new(),
        })
        .find(|v| !v.is_empty());
    let identity = identity?;
    let token = data
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if token.is_empty() {
        if ctx.config.require_user_token {
            return Some(ctx.err(401, "登录状态已失效，请更新客户端后重新登录"));
        }
        return None;
    }
    match verify_owner(pool, &token, &identity).await {
        OwnerState::Valid => None,
        OwnerState::Mismatch if VIEW_OTHER_ACTIONS.contains(&action) => {
            record_view_access(pool, &token, &identity, action).await;
            None
        }
        OwnerState::Mismatch => Some(ctx.err(401, "登录状态与账号不匹配，请重新登录")),
        OwnerState::Expired => Some(ctx.err(401, "登录已过期，请重新登录")),
        OwnerState::Unknown => {
            if ctx.config.require_user_token {
                Some(ctx.err(401, "登录状态已失效，请重新登录"))
            } else {
                None
            }
        }
    }
}

async fn record_view_access(pool: &MySqlPool, token: &str, target: &str, action: &str) {
    let viewer: Option<String> = sqlx::query_scalar(
        "SELECT ciyuanxi_id FROM user_tokens WHERE token = ? LIMIT 1",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    if let Some(viewer) = viewer {
        let _ = sqlx::query(
            "INSERT INTO view_access_log (viewer_ciyuanxi_id, target_ciyuanxi_id, action) VALUES (?, ?, ?)",
        )
        .bind(&viewer)
        .bind(target)
        .bind(action)
        .execute(pool)
        .await;
    }
}
