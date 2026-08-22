use axum::response::Response;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::response::ReqCtx;

const VIOLATION_WINDOW_SECONDS: i64 = 300;

/// 数据库临时封禁查询的内存负缓存间隔（秒）：
/// 每个身份在该间隔内只查一次库，避免正常请求每条都付出一次 DB 往返
const DB_BLOCK_CHECK_TTL_SECONDS: i64 = 15;

#[derive(Default)]
pub struct ApiRateLimiter {
    windows: Mutex<HashMap<String, WindowState>>,
    cooldowns: Mutex<HashMap<String, i64>>,
    temp_blocks: Mutex<HashMap<String, i64>>,
    /// 身份 -> 下次允许查库确认封禁的时间戳（负缓存）
    db_block_checks: Mutex<HashMap<String, i64>>,
    /// 上次全量清理的时间戳（秒），节流避免每请求都锁三张表全量扫描
    last_cleanup: AtomicI64,
}

#[derive(Clone, Debug)]
struct WindowState {
    started_at: i64,
    count: u32,
    violations: Vec<i64>,
}

#[derive(Clone, Copy)]
struct RateProfile {
    name: &'static str,
    window_seconds: i64,
    warn_threshold: u32,
    limit_threshold: u32,
    cooldown_seconds: i64,
    block_after_violations: usize,
    block_seconds: i64,
    allow_temp_block: bool,
}

#[derive(Clone, Debug)]
struct RateIdentity {
    identity_type: &'static str,
    identity: String,
    key: String,
}

#[derive(Clone, Debug)]
enum RateDecision {
    Allow,
    Warn { count: u32, threshold: u32 },
    Cooldown { retry_after: i64, count: u32, threshold: u32 },
    TempBlock { retry_after: i64, count: u32, threshold: u32 },
}

pub async fn check_api_rate_limit(
    limiter: &ApiRateLimiter,
    pool: Option<&MySqlPool>,
    action: &str,
    body: &str,
    ctx: &ReqCtx,
) -> Option<Response> {
    let data = serde_json::from_str::<Value>(body).unwrap_or(Value::Null);
    let identity = extract_identity(action, &data, &ctx.client_ip);
    check_with_identity(limiter, pool, action, identity, &ctx.client_ip, ctx).await
}

/// 后台登录专用限流：IP 与用户名双维度各自独立计数，任一超限即拦截。
/// 攻击者即使轮换用户名，其 IP 维度窗口仍会被计满，防止暴力破解。
pub async fn check_admin_login_rate_limit(
    limiter: &ApiRateLimiter,
    pool: Option<&MySqlPool>,
    body: &str,
    ctx: &ReqCtx,
) -> Option<Response> {
    let action = "admin_login";
    let ip = if ctx.client_ip.trim().is_empty() {
        "unknown".to_string()
    } else {
        ctx.client_ip.trim().to_string()
    };
    if let Some(resp) = check_with_identity(
        limiter,
        pool,
        action,
        build_identity(action, "ip", ip),
        &ctx.client_ip,
        ctx,
    )
    .await
    {
        return Some(resp);
    }
    if let Ok(data) = serde_json::from_str::<Value>(body) {
        if let Some(username) = first_non_empty(&data, &["username", "account", "identifier"]) {
            if let Some(resp) = check_with_identity(
                limiter,
                pool,
                action,
                build_identity(action, "identifier", username),
                &ctx.client_ip,
                ctx,
            )
            .await
            {
                return Some(resp);
            }
        }
    }
    None
}

async fn check_with_identity(
    limiter: &ApiRateLimiter,
    pool: Option<&MySqlPool>,
    action: &str,
    identity: RateIdentity,
    client_ip: &str,
    ctx: &ReqCtx,
) -> Option<Response> {
    let profile = profile_for_action(action);
    let now = now_seconds();
    let block_key = format!("{}:{}", identity.identity_type, identity.identity);

    if let Some(until) = limiter.memory_temp_block_until(&block_key, now) {
        return Some(rate_limited_response(ctx, until - now, "请求过于频繁，当前设备已被临时限制"));
    }

    // 内存无封禁时按负缓存间隔查库确认，命中则同步进内存并拦截
    if limiter.should_check_db_block(&block_key, now) {
        if let Some(db_until) = check_db_temp_block(pool, &identity).await {
            limiter.set_memory_temp_block(&block_key, db_until);
            return Some(rate_limited_response(ctx, db_until - now, "请求过于频繁，当前设备已被临时限制"));
        }
    }

    let decision = limiter.record(profile, &identity.key, &block_key, now);
    match decision {
        RateDecision::Allow => None,
        RateDecision::Warn { count, threshold } => {
            log_rate_event(pool, action, &identity, client_ip, "warning", count, threshold, profile.window_seconds, None, "请求频率达到预警阈值").await;
            None
        }
        RateDecision::Cooldown { retry_after, count, threshold } => {
            log_rate_event(pool, action, &identity, client_ip, "limited", count, threshold, profile.window_seconds, None, "请求频率超过限制，短暂冷却").await;
            Some(rate_limited_response(ctx, retry_after, "请求过于频繁，请稍后再试"))
        }
        RateDecision::TempBlock { retry_after, count, threshold } => {
            let blocked_until = now + retry_after;
            upsert_db_temp_block(pool, &identity, client_ip, blocked_until, "短时间内多次触发 API 限流").await;
            log_rate_event(pool, action, &identity, client_ip, "blocked", count, threshold, profile.window_seconds, Some(blocked_until), "短时间内多次超限，临时限制 1 小时").await;
            Some(rate_limited_response(ctx, retry_after, "请求过于频繁，当前设备已被临时限制 1 小时"))
        }
    }
}

impl ApiRateLimiter {
    fn record(&self, profile: RateProfile, key: &str, block_key: &str, now: i64) -> RateDecision {
        self.cleanup(now);

        if let Some(until) = self.memory_cooldown_until(key, now) {
            return RateDecision::Cooldown {
                retry_after: until - now,
                count: profile.limit_threshold,
                threshold: profile.limit_threshold,
            };
        }

        let mut windows = self.windows.lock().unwrap();
        let state = windows.entry(key.to_string()).or_insert(WindowState {
            started_at: now,
            count: 0,
            violations: Vec::new(),
        });

        if now - state.started_at >= profile.window_seconds {
            state.started_at = now;
            state.count = 0;
        }

        state.count = state.count.saturating_add(1);
        let count = state.count;

        if count == profile.warn_threshold {
            return RateDecision::Warn {
                count,
                threshold: profile.warn_threshold,
            };
        }

        if count < profile.limit_threshold {
            return RateDecision::Allow;
        }

        state.violations.retain(|ts| now - *ts <= VIOLATION_WINDOW_SECONDS);
        state.violations.push(now);
        let violation_count = state.violations.len();
        drop(windows);

        if profile.allow_temp_block
            && profile.block_after_violations > 0
            && violation_count >= profile.block_after_violations
        {
            let until = now + profile.block_seconds;
            self.set_memory_temp_block(block_key, until);
            return RateDecision::TempBlock {
                retry_after: profile.block_seconds,
                count,
                threshold: profile.limit_threshold,
            };
        }

        let until = now + profile.cooldown_seconds;
        self.set_memory_cooldown(key, until);
        RateDecision::Cooldown {
            retry_after: profile.cooldown_seconds,
            count,
            threshold: profile.limit_threshold,
        }
    }

    fn memory_cooldown_until(&self, key: &str, now: i64) -> Option<i64> {
        let mut cooldowns = self.cooldowns.lock().unwrap();
        match cooldowns.get(key).copied() {
            Some(until) if until > now => Some(until),
            Some(_) => {
                cooldowns.remove(key);
                None
            }
            None => None,
        }
    }

    fn set_memory_cooldown(&self, key: &str, until: i64) {
        self.cooldowns.lock().unwrap().insert(key.to_string(), until);
    }

    fn memory_temp_block_until(&self, key: &str, now: i64) -> Option<i64> {
        let mut blocks = self.temp_blocks.lock().unwrap();
        match blocks.get(key).copied() {
            Some(until) if until > now => Some(until),
            Some(_) => {
                blocks.remove(key);
                None
            }
            None => None,
        }
    }

    fn set_memory_temp_block(&self, key: &str, until: i64) {
        self.temp_blocks.lock().unwrap().insert(key.to_string(), until);
    }

    /// 是否需要查库确认封禁：距上次确认不足 TTL 时跳过（负缓存）。
    /// 封禁记录由限流触发时写入，正常请求高频率查库纯属开销。
    fn should_check_db_block(&self, block_key: &str, now: i64) -> bool {
        let mut checks = self.db_block_checks.lock().unwrap();
        match checks.get(block_key).copied() {
            Some(until) if until > now => false,
            _ => {
                checks.insert(block_key.to_string(), now + DB_BLOCK_CHECK_TTL_SECONDS);
                true
            }
        }
    }

    fn cleanup(&self, now: i64) {
        // 节流：过期条目本身有 TTL 兜底（读取时会判过期），30 秒清一次足够
        let last = self.last_cleanup.load(Ordering::Relaxed);
        if now - last < 30 {
            return;
        }
        if self
            .last_cleanup
            .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return; // 另一个请求正在清理
        }

        let mut cooldowns = self.cooldowns.lock().unwrap();
        cooldowns.retain(|_, until| *until > now);
        drop(cooldowns);

        let mut blocks = self.temp_blocks.lock().unwrap();
        blocks.retain(|_, until| *until > now);
        drop(blocks);

        let mut checks = self.db_block_checks.lock().unwrap();
        checks.retain(|_, until| *until > now);
        drop(checks);

        let mut windows = self.windows.lock().unwrap();
        if windows.len() > 10_000 {
            windows.retain(|_, state| now - state.started_at <= 600);
        }
    }
}

fn profile_for_action(action: &str) -> RateProfile {
    match action {
        "register"
        | "user_login"
        | "login_by_code"
        | "send_verify_code"
        | "reset_password"
        | "get_captcha"
        | "verify_captcha"
        | "email_send_code"
        | "email_register"
        | "email_login"
        | "email_reset_password"
        | "email_get_profile"
        | "generate_tv_login_code"
        | "poll_tv_login_status"
        | "scan_tv_login"
        | "confirm_tv_login"
        | "admin_login" => RateProfile {
            name: "auth",
            window_seconds: 60,
            warn_threshold: 5,
            limit_threshold: 10,
            cooldown_seconds: 300,
            block_after_violations: 3,
            block_seconds: 3600,
            allow_temp_block: true,
        },
        "file_sync_upload_start"
        | "file_sync_upload_chunk"
        | "file_sync_upload_finish"
        | "file_sync_download"
        | "plugin_sync_upload_one"
        | "plugin_sync_download"
        | "settings_sync_upload"
        | "settings_sync_download"
        | "upload_avatar"
        | "upload_wallpaper" => RateProfile {
            name: "bulk",
            window_seconds: 1,
            warn_threshold: 50,
            limit_threshold: 80,
            cooldown_seconds: 30,
            block_after_violations: 0,
            block_seconds: 0,
            allow_temp_block: false,
        },
        _ => RateProfile {
            name: "default",
            window_seconds: 1,
            warn_threshold: 20,
            limit_threshold: 30,
            cooldown_seconds: 60,
            block_after_violations: 3,
            block_seconds: 3600,
            allow_temp_block: true,
        },
    }
}

fn extract_identity(action: &str, data: &Value, ip: &str) -> RateIdentity {
    if let Some(device_id) = first_non_empty(data, &["device_id", "deviceId", "last_device_id", "lastDeviceId"]) {
        return build_identity(action, "device_id", device_id);
    }
    if let Some(user_id) = first_non_empty(data, &["ciyuanxi_id", "ciyuanxiId", "user_id", "userId", "uid", "id"]) {
        return build_identity(action, "user_id", user_id);
    }
    if let Some(identifier) = first_non_empty(data, &["email", "username", "account", "identifier", "phone"]) {
        return build_identity(action, "identifier", identifier);
    }
    let fallback_ip = if ip.trim().is_empty() { "unknown" } else { ip.trim() };
    build_identity(action, "ip", fallback_ip.to_string())
}

fn build_identity(action: &str, identity_type: &'static str, identity: String) -> RateIdentity {
    let profile = profile_for_action(action);
    let key = if profile.name == "auth" {
        format!("{}:{}:{}:{}", profile.name, action, identity_type, identity)
    } else {
        format!("{}:{}:{}", profile.name, identity_type, identity)
    };
    RateIdentity {
        identity_type,
        identity,
        key,
    }
}

fn first_non_empty(data: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        let value = data.get(*key).and_then(value_to_string).unwrap_or_default();
        let trimmed = value.trim();
        if !trimmed.is_empty() && trimmed != "0" && !trimmed.eq_ignore_ascii_case("null") {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

async fn check_db_temp_block(pool: Option<&MySqlPool>, identity: &RateIdentity) -> Option<i64> {
    let pool = pool?;
    let row = sqlx::query(
        "SELECT TIMESTAMPDIFF(SECOND, NOW(), expires_at) AS remain_seconds
         FROM api_temp_blocks
         WHERE identity_type = ? AND identity = ? AND expires_at > NOW()
         LIMIT 1",
    )
    .bind(identity.identity_type)
    .bind(&identity.identity)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    let remain: i64 = row.try_get("remain_seconds").unwrap_or(0);
    if remain > 0 {
        Some(now_seconds() + remain)
    } else {
        None
    }
}

async fn upsert_db_temp_block(pool: Option<&MySqlPool>, identity: &RateIdentity, ip: &str, blocked_until: i64, reason: &str) {
    let Some(pool) = pool else {
        return;
    };
    let seconds = (blocked_until - now_seconds()).max(1);
    let _ = sqlx::query(
        "INSERT INTO api_temp_blocks (identity_type, identity, ip, reason, expires_at)
         VALUES (?, ?, ?, ?, DATE_ADD(NOW(), INTERVAL ? SECOND))
         ON DUPLICATE KEY UPDATE
           ip = VALUES(ip),
           reason = VALUES(reason),
           expires_at = GREATEST(expires_at, VALUES(expires_at)),
           updated_at = CURRENT_TIMESTAMP",
    )
    .bind(identity.identity_type)
    .bind(&identity.identity)
    .bind(ip)
    .bind(reason)
    .bind(seconds)
    .execute(pool)
    .await;
}

async fn log_rate_event(
    pool: Option<&MySqlPool>,
    action: &str,
    identity: &RateIdentity,
    ip: &str,
    level: &str,
    count: u32,
    threshold: u32,
    window_seconds: i64,
    blocked_until: Option<i64>,
    reason: &str,
) {
    let Some(pool) = pool else {
        return;
    };
    let blocked_seconds = blocked_until.map(|until| (until - now_seconds()).max(1));
    let _ = sqlx::query(
        "INSERT INTO api_rate_events
         (level, action, identity_type, identity, ip, request_count, threshold_count, window_seconds, reason, blocked_until)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CASE WHEN ? IS NULL THEN NULL ELSE DATE_ADD(NOW(), INTERVAL ? SECOND) END)",
    )
    .bind(level)
    .bind(action)
    .bind(identity.identity_type)
    .bind(&identity.identity)
    .bind(ip)
    .bind(count as i64)
    .bind(threshold as i64)
    .bind(window_seconds)
    .bind(reason)
    .bind(blocked_seconds)
    .bind(blocked_seconds)
    .execute(pool)
    .await;
}

fn rate_limited_response(ctx: &ReqCtx, retry_after: i64, msg: &str) -> Response {
    ctx.json(
        429,
        msg,
        Some(json!({
            "retry_after": retry_after.max(1)
        })),
    )
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
