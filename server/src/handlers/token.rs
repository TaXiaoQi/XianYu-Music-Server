use axum::response::Response;
use serde_json::Value;
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::response::ReqCtx;

/// token 有效期（天）
const TOKEN_TTL_DAYS: i64 = 30;
/// 剩余有效期低于该天数时滑动续期
const TOKEN_RENEW_THRESHOLD_DAYS: i64 = 15;
/// 单用户最多保留的 token 数（多设备），超出删除最旧
const MAX_TOKENS_PER_USER: i64 = 10;
/// token 属主校验内存缓存有效期（秒）：命中时跳过数据库查询。
/// 撤销 token（改密/注销）最多延迟该秒数生效
const TOKEN_CACHE_TTL_SECONDS: i64 = 15;
/// last_used_at 落库节流间隔（秒）：活跃 token 无需每次请求都写库，
/// 也避免分片同步等并发请求对同一行反复加锁
const TOKEN_TOUCH_INTERVAL_SECONDS: i64 = 300;
/// token 缓存容量上限，超出时清理已过期条目
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

/// 需要校验 token 属主的 action（请求体含 ciyuanxi_id / user_id 的用户资源操作）
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
    // upload
    "upload_avatar",
    // account lifecycle
    "delete_account",
    "preverify_delete_account",
];

/// 签发用户 token 并落库；同设备旧 token 立即失效
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

/// 撤销用户全部 token（改密 / 重置密码 / 改弦予号 / 注销后调用，已签发 token 全部失效）
pub async fn revoke_user(pool: &MySqlPool, ciyuanxi_id: &str) {
    let _ = sqlx::query("DELETE FROM user_tokens WHERE ciyuanxi_id = ?")
        .bind(ciyuanxi_id)
        .execute(pool)
        .await;
}

/// 清理该用户过期与超量的 token
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
    /// token 有效且属主匹配（已滑动续期）
    Valid,
    /// token 存在但已过期
    Expired,
    /// token 有效但属主与请求身份不符（token 被挪用/伪造）
    Mismatch,
    /// token 不在库中（旧版服务端签发的遗留 token 或伪造值）
    Unknown,
}

async fn verify_owner(pool: &MySqlPool, token: &str, identity: &str) -> OwnerState {
    let now = now_unix();
    // 快路径：命中缓存直接判定，跳过数据库往返（token 撤销最多延迟 TTL 秒生效）
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
            // 剩余有效期不足一半时滑动续期，避免活跃用户被登出
            let _ = sqlx::query(
                "UPDATE user_tokens SET last_used_at = NOW(), expires_at = DATE_ADD(NOW(), INTERVAL ? DAY) WHERE token = ?",
            )
            .bind(TOKEN_TTL_DAYS)
            .bind(token)
            .execute(pool)
            .await;
        } else if should_touch == 1 {
            // last_used_at 落库节流：仅距上次使用超过阈值才写库
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

/// dispatch 层统一鉴权：用户资源操作必须由 token 属主本人发起。
///
/// - 请求携带 token：严格校验属主与有效期，不匹配立即 401
/// - 请求未携带 token：
///   - 软模式（require_user_token=false，默认）：放行，兼容未携带 token 的存量客户端
///   - 硬模式（require_user_token=true）：拒绝，要求更新客户端
/// - 软模式下库中不存在的 token（旧版服务端签发、未落库）放行，避免存量用户被误杀
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
    // 身份字段：不同 action 使用 ciyuanxi_id 或 user_id（可能为数字），取先出现且非空者
    let identity = ["ciyuanxi_id", "user_id"]
        .iter()
        .map(|k| match data.get(*k) {
            Some(Value::String(s)) => s.trim().to_string(),
            Some(Value::Number(n)) => n.to_string(),
            _ => String::new(),
        })
        .find(|v| !v.is_empty());
    // 无身份字段：交给各 handler 自行报"请先登录"
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
