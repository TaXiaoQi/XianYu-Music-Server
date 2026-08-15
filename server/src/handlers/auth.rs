use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{default_nickname, parse_body, str_of, validate_ciyuanxi_id, validate_nickname};
use crate::response::ReqCtx;

const CAPTCHA_TTL_MINUTES: i64 = 5;
const LOGIN_LOCK_THRESHOLD: i64 = 5;
const LOGIN_LOCK_MINUTES: i64 = 15;
const LOGIN_FAILURE_WINDOW_MINUTES: i64 = 30;

/// 检查设备是否被封禁，返回 Some(错误响应) 表示已封禁
async fn check_device_ban(device_id: &str, ctx: &ReqCtx, pool: &MySqlPool) -> Option<Response> {
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

/// 客户端心跳接口：检查账号/设备是否被封禁。
/// 返回 code=200 + data.banned，避免客户端 requestAction 抛错。
pub async fn check_ban_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let device_id = str_of(&data, "device_id").trim().to_string();

    // 账号封禁
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

    // 设备封禁
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

fn rand_token() -> String {
    crate::handlers::helpers::random_hex(32)
}

/// 根据邮箱判定角色（管理员已去除邮箱，统一返回 member）
async fn resolve_role(_pool: &MySqlPool, _email: &str) -> String {
    "member".to_string()
}

/// 构建登录用户返回 payload
fn build_user_payload(
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

pub async fn get_captcha(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let purpose = str_of(&data, "purpose").trim().to_string();
    let purpose = if purpose.is_empty() { "auth".to_string() } else { purpose };
    let ip = ctx.client_ip.clone();

    let recent_count: i64 = sqlx::query(
        "SELECT COUNT(*) AS cnt FROM human_captcha_challenges WHERE ip = ? AND created_at > DATE_SUB(NOW(), INTERVAL 60 SECOND)",
    )
    .bind(&ip)
    .fetch_one(pool)
    .await
    .map(|r| r.get("cnt"))
    .unwrap_or(0);
    if recent_count >= 20 {
        return ctx.err(429, "请求过于频繁，请稍后再试");
    }

    let _ = sqlx::query("DELETE FROM human_captcha_challenges WHERE expires_at <= NOW() OR created_at < DATE_SUB(NOW(), INTERVAL 1 DAY)")
        .execute(pool)
        .await;

    let left = crate::handlers::helpers::random_int(2, 9);
    let right = crate::handlers::helpers::random_int(1, 9);
    let captcha_id = crate::handlers::helpers::random_hex(16);
    let answer = (left + right).to_string();
    let question = format!("{} + {} = ?", left, right);

    let result = sqlx::query(
        "INSERT INTO human_captcha_challenges (captcha_id, purpose, answer, ip, expires_at) VALUES (?,?,?,?,DATE_ADD(NOW(), INTERVAL ? MINUTE))",
    )
    .bind(&captcha_id)
    .bind(&purpose)
    .bind(&answer)
    .bind(&ip)
    .bind(CAPTCHA_TTL_MINUTES)
    .execute(pool)
    .await;

    match result {
        Ok(_) => ctx.ok(
            "ok",
            json!({
                "captcha_id": captcha_id,
                "question": question,
                "expire_seconds": CAPTCHA_TTL_MINUTES * 60,
            }),
        ),
        Err(e) => ctx.err(500, &format!("验证码生成失败: {}", e)),
    }
}

pub async fn verify_captcha(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let captcha_id = str_of(&data, "captcha_id").trim().to_string();
    let captcha_answer = str_of(&data, "captcha_answer").trim().to_string();
    let purpose = str_of(&data, "purpose").trim().to_string();
    let purpose = if purpose.is_empty() { "auth".to_string() } else { purpose };

    if captcha_id.is_empty() || captcha_answer.is_empty() {
        return ctx.err(400, "请完成人机验证");
    }

    let row = sqlx::query(
        "SELECT answer, ip FROM human_captcha_challenges WHERE captcha_id = ? AND purpose = ? AND used = 0 AND expires_at > NOW() LIMIT 1",
    )
    .bind(&captcha_id)
    .bind(&purpose)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(row) = row else {
        return ctx.err(400, "人机验证已过期，请刷新后重试");
    };

    let expected: String = row.get("answer");
    let ip: String = row.get("ip");
    if ip != ctx.client_ip || expected.trim() != captcha_answer {
        return ctx.err(400, "人机验证错误，请重新输入");
    }

    ctx.ok("验证通过", json!({ "verified": true }))
}

async fn require_captcha(data: &serde_json::Value, ctx: &ReqCtx, pool: &MySqlPool, purpose: &str) -> Option<Response> {
    let captcha_config = crate::handlers::email_auth::load_captcha_config(pool, &ctx.config).await;
    let use_provider_captcha = captcha_config.enabled
        && !captcha_config.site_key.trim().is_empty()
        && !captcha_config.secret.trim().is_empty();
    if use_provider_captcha {
        let captcha_token = {
            let token = str_of(data, "captcha_token");
            if token.trim().is_empty() {
                str_of(data, "turnstile_token")
            } else {
                token
            }
        };
        match crate::handlers::email_auth::verify_captcha_token(&captcha_config, &captcha_token, &ctx.client_ip).await {
            Ok(true) => return None,
            Ok(false) => return Some(ctx.err(400, "请先完成人机验证")),
            Err(e) => {
                eprintln!("[auth] 人机验证校验失败: {}", e);
                return Some(ctx.err(500, "人机验证服务暂不可用，请稍后重试"));
            }
        }
    }

    let captcha_id = str_of(data, "captcha_id").trim().to_string();
    let captcha_answer = str_of(data, "captcha_answer").trim().to_string();
    if captcha_id.is_empty() || captcha_answer.is_empty() {
        return Some(ctx.err(400, "请完成人机验证"));
    }

    let row = sqlx::query(
        "SELECT id, answer, ip FROM human_captcha_challenges WHERE captcha_id = ? AND purpose = ? AND used = 0 AND expires_at > NOW() LIMIT 1",
    )
    .bind(&captcha_id)
    .bind(purpose)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(row) = row else {
        return Some(ctx.err(400, "人机验证已过期，请刷新后重试"));
    };

    let id: i64 = row.get("id");
    let expected: String = row.get("answer");
    let ip: String = row.get("ip");
    let _ = sqlx::query("UPDATE human_captcha_challenges SET used = 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await;

    if ip != ctx.client_ip || expected.trim() != captcha_answer {
        return Some(ctx.err(400, "人机验证错误，请刷新后重试"));
    }
    None
}

fn normalize_rate_identifier(value: &str) -> String {
    value.trim().to_lowercase()
}

async fn check_login_cooldown(identifier: &str, ctx: &ReqCtx, pool: &MySqlPool) -> Option<Response> {
    let key = normalize_rate_identifier(identifier);
    if key.is_empty() {
        return None;
    }
    let row = sqlx::query(
        "SELECT TIMESTAMPDIFF(SECOND, NOW(), locked_until) AS remain_seconds FROM auth_rate_limits WHERE action = 'user_login' AND identifier = ? AND ip = ? AND locked_until IS NOT NULL AND locked_until > NOW() LIMIT 1",
    )
    .bind(&key)
    .bind(&ctx.client_ip)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    if let Some(row) = row {
        let remain_seconds: i64 = row.try_get("remain_seconds").unwrap_or(LOGIN_LOCK_MINUTES * 60);
        let remain_minutes = ((remain_seconds.max(1) + 59) / 60).max(1);
        return Some(ctx.err(429, &format!("登录失败次数过多，请约{}分钟后再试", remain_minutes)));
    }
    None
}

async fn record_login_failure(identifier: &str, ctx: &ReqCtx, pool: &MySqlPool) {
    let key = normalize_rate_identifier(identifier);
    if key.is_empty() {
        return;
    }
    let current: i64 = sqlx::query(
        "SELECT failed_count FROM auth_rate_limits WHERE action = 'user_login' AND identifier = ? AND ip = ? AND updated_at > DATE_SUB(NOW(), INTERVAL ? MINUTE) LIMIT 1",
    )
    .bind(&key)
    .bind(&ctx.client_ip)
    .bind(LOGIN_FAILURE_WINDOW_MINUTES)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .and_then(|r| r.try_get("failed_count").ok())
    .unwrap_or(0);
    let next = current + 1;
    let lock = next >= LOGIN_LOCK_THRESHOLD;
    let _ = sqlx::query(
        "INSERT INTO auth_rate_limits (action, identifier, ip, failed_count, locked_until, last_failed_at)
         VALUES ('user_login', ?, ?, ?, IF(? = 1, DATE_ADD(NOW(), INTERVAL ? MINUTE), NULL), NOW())
         ON DUPLICATE KEY UPDATE failed_count = VALUES(failed_count), locked_until = VALUES(locked_until), last_failed_at = NOW()",
    )
    .bind(&key)
    .bind(&ctx.client_ip)
    .bind(next)
    .bind(if lock { 1 } else { 0 })
    .bind(LOGIN_LOCK_MINUTES)
    .execute(pool)
    .await;
}

async fn clear_login_failures(identifier: &str, ctx: &ReqCtx, pool: &MySqlPool) {
    let key = normalize_rate_identifier(identifier);
    if key.is_empty() {
        return;
    }
    let _ = sqlx::query("DELETE FROM auth_rate_limits WHERE action = 'user_login' AND identifier = ? AND ip = ?")
        .bind(&key)
        .bind(&ctx.client_ip)
        .execute(pool)
        .await;
}

pub async fn register(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let mut nickname = str_of(&data, "nickname").trim().to_string();
    if nickname.is_empty() {
        nickname = str_of(&data, "username").trim().to_string();
    }
    let password = str_of(&data, "password");
    let email = str_of(&data, "email").trim().to_string();
    let verify_code = str_of(&data, "verify_code").trim().to_string();
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();

    // 弦予号必填 + 微信号规则校验
    if let Err(msg) = validate_ciyuanxi_id(&ciyuanxi_id) {
        return ctx.err(400, msg);
    }
    // 昵称可选，留空默认"弦予+号"
    if nickname.is_empty() {
        nickname = default_nickname(&ciyuanxi_id);
    }
    let name_len = nickname.chars().count();
    if name_len < 2 || name_len > 32 {
        return ctx.err(400, "昵称长度需2-32个字符");
    }
    if let Err(msg) = validate_nickname(&nickname, 2, 32) {
        return ctx.err(400, msg);
    }
    // 检查昵称是否与管理员用户名冲突（大小写不敏感）
    {
        let admin_conflict = sqlx::query("SELECT id FROM admin_users WHERE LOWER(username) = LOWER(?) LIMIT 1")
            .bind(&nickname)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .is_some();
        if admin_conflict {
            return ctx.err(400, "该昵称不可使用");
        }
    }
    if password.len() < 6 {
        return ctx.err(400, "密码长度至少6位");
    }
    if !email.contains('@') {
        return ctx.err(400, "邮箱格式不正确");
    }
    if verify_code.is_empty() {
        return ctx.err(400, "请输入验证码");
    }
    if let Some(resp) = require_captcha(&data, &ctx, pool, "auth").await {
        return resp;
    }

    // 检查设备是否被封禁
    let reg_device_id = str_of(&data, "device_id").trim().to_string();
    if let Some(resp) = check_device_ban(&reg_device_id, &ctx, pool).await {
        return resp;
    }

    let _ip = ctx.client_ip.clone();

    // 验证邮箱验证码
    let code_row = sqlx::query(
        "SELECT * FROM email_verify_codes WHERE email = ? AND code = ? AND type = 'register' AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .bind(&verify_code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let Some(code_row) = code_row else {
        return ctx.err(400, "验证码无效或已过期");
    };
    let code_id: i64 = code_row.try_get("id").unwrap_or(0);
    let _ = sqlx::query("UPDATE email_verify_codes SET used = 1 WHERE id = ?")
        .bind(code_id)
        .execute(pool)
        .await;

    // 检查邮箱是否已注册
    let email_user = sqlx::query("SELECT id FROM app_users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if email_user {
        return ctx.err(400, "该邮箱已注册");
    }

    // 弦予号唯一性校验（普通用户表 + 靓号表）
    let id_dup = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    let id_pretty = sqlx::query("SELECT id FROM ciyuanxi_pretty_ids WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if id_dup || id_pretty {
        return ctx.err(400, "该弦予号已被占用");
    }

    let hashed = match bcrypt::hash(&password, 10) {
        Ok(h) => h,
        Err(_) => return ctx.err(500, "密码加密失败"),
    };
    let result = sqlx::query(
        "INSERT INTO app_users (nickname, password, email, email_verified, status, ciyuanxi_id, last_device_id) VALUES (?,?,?,1,1,?,?)",
    )
    .bind(&nickname)
    .bind(&hashed)
    .bind(&email)
    .bind(&ciyuanxi_id)
    .bind(&reg_device_id)
    .execute(pool)
    .await;

    match result {
        Ok(r) => {
            let user_id = r.last_insert_id() as i64;
            let token = rand_token();
            let role = resolve_role(pool, &email).await;
            let payload = build_user_payload(
                user_id,
                &nickname,
                &email,
                "",
                &ciyuanxi_id,
                1,
                0,
                &token,
                &role,
            );
            ctx.json(200, "注册成功", Some(payload))
        },
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn user_login(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    // 支持弦予号或邮箱登录（参考微信号设计：邮箱登录时大小写不敏感）
    let account_input = str_of(&data, "ciyuanxi_id").trim().to_string();
    let password = str_of(&data, "password");
    if account_input.is_empty() || password.is_empty() {
        return ctx.err(400, "弦予号/邮箱和密码不能为空");
    }
    if let Some(resp) = check_login_cooldown(&account_input, &ctx, pool).await {
        return resp;
    }
    if let Some(resp) = require_captcha(&data, &ctx, pool, "auth").await {
        return resp;
    }

    let is_email = account_input.contains('@');
    let (user, matched) = if is_email {
        let email_lower = account_input.to_lowercase();
        let row = sqlx::query("SELECT * FROM app_users WHERE LOWER(email) = ?")
            .bind(&email_lower)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        (row, account_input.clone())
    } else {
        let row = sqlx::query("SELECT * FROM app_users WHERE ciyuanxi_id = ?")
            .bind(&account_input)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        (row, account_input.clone())
    };
    let Some(user) = user else {
        record_login_failure(&matched, &ctx, pool).await;
        return ctx.err(401, "弦予号/邮箱或密码错误");
    };
    let stored: String = user.get("password");
    let mut password_ok = bcrypt::verify(&password, &stored).unwrap_or(false);
    if !password_ok && stored == password {
        password_ok = true;
        if let Ok(hashed) = bcrypt::hash(&password, 10) {
            let user_id: i64 = user.try_get::<i64, _>("id").unwrap_or(0);
            if user_id > 0 {
                let _ = sqlx::query("UPDATE app_users SET password = ? WHERE id = ?")
                    .bind(hashed)
                    .bind(user_id)
                    .execute(pool)
                    .await;
            }
        }
    }
    if !password_ok {
        record_login_failure(&matched, &ctx, pool).await;
        return ctx.err(401, "弦予号/邮箱或密码错误");
    }
    let status: i64 = user.get("status");
    if status == 0 {
        let ban_reason: String = user.try_get::<String, _>("ban_reason").unwrap_or_default();
        let reason = ban_reason.trim();
        if reason.is_empty() {
            return ctx.err(403, "账号已被封禁，请联系管理员");
        }
        return ctx.err(403, &format!("账号已被封禁，原因：{}。如有疑问请联系管理员", reason));
    }
    let login_device_id = str_of(&data, "device_id").trim().to_string();
    if let Some(resp) = check_device_ban(&login_device_id, &ctx, pool).await {
        return resp;
    }
    let email: String = user.try_get::<String, _>("email").unwrap_or_default();
    let role = resolve_role(pool, &email).await;
    let token = rand_token();

    let user_id: i64 = user.try_get::<i64, _>("id").unwrap_or(0);
    let uname: String = user.try_get::<String, _>("nickname").unwrap_or_default();
    let avatar_url: String = user.try_get::<Option<String>, _>("avatar_url").ok().flatten().unwrap_or_default();
    let ciyuanxi_id: String = user.try_get::<String, _>("ciyuanxi_id").unwrap_or_default();
    let master_quota: i64 = user.try_get::<i64, _>("master_quota").unwrap_or(0);
    clear_login_failures(&matched, &ctx, pool).await;

    // 记录 APP 登录日志（若请求未携带设备信息，则从 app_open_log 按 device_id 兜底补全）
    let mut log_device_model = str_of(&data, "device_model").trim().to_string();
    let mut log_app_version = str_of(&data, "app_version").trim().to_string();
    let mut log_os_version = str_of(&data, "os_version").trim().to_string();
    if !login_device_id.is_empty()
        && (log_device_model.is_empty() || log_app_version.is_empty() || log_os_version.is_empty())
    {
        if let Ok(Some(r)) = sqlx::query(
            "SELECT device_model, app_version, os_version FROM app_open_log WHERE device_id = ? AND device_id != '' ORDER BY id DESC LIMIT 1",
        )
        .bind(&login_device_id)
        .fetch_optional(pool)
        .await
        {
            if log_device_model.is_empty() {
                log_device_model = r.try_get::<String, _>("device_model").unwrap_or_default();
            }
            if log_app_version.is_empty() {
                log_app_version = r.try_get::<String, _>("app_version").unwrap_or_default();
            }
            if log_os_version.is_empty() {
                log_os_version = r.try_get::<String, _>("os_version").unwrap_or_default();
            }
        }
    }
    let _ = sqlx::query(
        "INSERT INTO admin_app_login_log (admin_id, admin_username, ip, user_agent, device_id, device_model, app_version, os_version, status, extra) VALUES (?,?,?,?,?,?,?,?,1,?)",
    )
    .bind(user_id)
    .bind(&uname)
    .bind(&ctx.client_ip)
    .bind("")
    .bind(&login_device_id)
    .bind(&log_device_model)
    .bind(&log_app_version)
    .bind(&log_os_version)
    .bind("user_login")
    .execute(pool)
    .await;

    // 更新用户最后登录设备ID
    if !login_device_id.is_empty() {
        let _ = sqlx::query("UPDATE app_users SET last_device_id = ? WHERE id = ?")
            .bind(&login_device_id)
            .bind(user_id)
            .execute(pool)
            .await;
    }

    let payload = build_user_payload(user_id, &uname, &email, &avatar_url, &ciyuanxi_id, status, master_quota, &token, &role);
    ctx.json(200, "登录成功", Some(payload))
}

pub async fn generate_tv_login_code(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return ctx.err(400, "设备标识不能为空");
    }
    let _ = sqlx::query("DELETE FROM tv_login_codes WHERE device_id = ? AND created_at < (NOW() - INTERVAL 10 MINUTE)")
        .bind(&device_id)
        .execute(pool)
        .await;
    let code = crate::handlers::helpers::random_hex(16);
    let ip = ctx.client_ip.clone();
    let result = sqlx::query("INSERT INTO tv_login_codes (code, device_id, status, ip, expires_at) VALUES (?,?,'pending',?,DATE_ADD(NOW(), INTERVAL 5 MINUTE))")
        .bind(&code)
        .bind(&device_id)
        .bind(&ip)
        .execute(pool)
        .await;
    match result {
        Ok(_) => ctx.json(200, "ok", Some(json!({ "code": code, "expire_seconds": 300 }))),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn poll_tv_login_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let code = str_of(&data, "code");
    let device_id = str_of(&data, "device_id");
    if code.is_empty() || device_id.is_empty() {
        return ctx.err(400, "code 和 device_id 不能为空");
    }
    let row = sqlx::query("SELECT * FROM tv_login_codes WHERE code = ? AND device_id = ? LIMIT 1")
        .bind(&code)
        .bind(&device_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(row) = row else {
        return ctx.err(404, "二维码无效或设备不匹配");
    };
    let status: String = row.get("status");
    if status != "logged_in" {
        return ctx.json(200, "ok", Some(json!({ "status": status })));
    }
    let ciyuanxi_id: String = row.get("ciyuanxi_id");
    let token: String = row.get("token");
    let user = sqlx::query("SELECT * FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let user_status: i64 = user.get("status");
    if user_status == 0 {
        return ctx.err(403, "账号已被禁用");
    }
    let email: String = user.get("email");
    let role = resolve_role(pool, &email).await;
    let user_id: i64 = user.get("id");
    let uname: String = user.try_get::<String, _>("nickname").unwrap_or_default();
    let avatar_url: String = user.try_get::<Option<String>, _>("avatar_url").ok().flatten().unwrap_or_default();
    let master_quota: i64 = user.try_get::<i64, _>("master_quota").unwrap_or(0);
    let mut payload = build_user_payload(user_id, &uname, &email, &avatar_url, &ciyuanxi_id, user_status, master_quota, &token, &role);
    payload["status"] = json!("logged_in");
    ctx.json(200, "登录成功", Some(payload))
}

pub async fn scan_tv_login(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let code = str_of(&data, "code");
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
    if code.is_empty() || ciyuanxi_id.is_empty() {
        return ctx.err(400, "code 和 ciyuanxi_id 不能为空");
    }
    let row = sqlx::query("SELECT * FROM tv_login_codes WHERE code = ? LIMIT 1")
        .bind(&code)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(row) = row else {
        return ctx.err(404, "二维码无效");
    };
    let status: String = row.get("status");
    if status != "pending" && status != "scanned" {
        return ctx.err(410, "二维码已被使用或已取消");
    }
    let user = sqlx::query("SELECT id, status, nickname FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let u_status: i64 = user.get("status");
    if u_status == 0 {
        return ctx.err(403, "账号已被禁用");
    }
    let nickname: String = user.try_get::<String, _>("nickname").unwrap_or_default();
    let _ = sqlx::query("UPDATE tv_login_codes SET status = 'scanned', ciyuanxi_id = ?, scanned_at = NOW() WHERE code = ? AND status IN ('pending','scanned')")
        .bind(&ciyuanxi_id)
        .bind(&code)
        .execute(pool)
        .await;
    ctx.json(
        200,
        "扫码成功，请在手机端确认登录",
        Some(json!({ "ciyuanxi_id": ciyuanxi_id, "nickname": nickname, "username": nickname })),
    )
}

pub async fn confirm_tv_login(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let code = str_of(&data, "code");
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
    if code.is_empty() || ciyuanxi_id.is_empty() {
        return ctx.err(400, "code 和 ciyuanxi_id 不能为空");
    }
    let row = sqlx::query("SELECT * FROM tv_login_codes WHERE code = ? LIMIT 1")
        .bind(&code)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(row) = row else {
        return ctx.err(404, "二维码无效");
    };
    let status: String = row.get("status");
    if status != "scanned" {
        return ctx.err(410, "请先扫码后再确认登录");
    }
    let row_ciyuanxi: String = row.get("ciyuanxi_id");
    if row_ciyuanxi != ciyuanxi_id {
        return ctx.err(403, "账号不匹配，无法确认登录");
    }
    let user = sqlx::query("SELECT id, status FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let u_status: i64 = user.get("status");
    if u_status == 0 {
        return ctx.err(403, "账号已被禁用");
    }
    let token = rand_token();
    let res = sqlx::query("UPDATE tv_login_codes SET status = 'logged_in', token = ?, logged_in_at = NOW() WHERE code = ? AND status = 'scanned'")
        .bind(&token)
        .bind(&code)
        .execute(pool)
        .await;
    match res {
        Ok(r) if r.rows_affected() > 0 => ctx.json(200, "登录成功", Some(json!({ "ciyuanxi_id": ciyuanxi_id }))),
        _ => ctx.err(410, "确认失败，二维码状态已变更"),
    }
}

pub async fn login_by_code(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let verify_code = str_of(&data, "verify_code").trim().to_string();
    if email.is_empty() || !email.contains('@') {
        return ctx.err(400, "请输入正确的邮箱");
    }
    if verify_code.is_empty() {
        return ctx.err(400, "请输入验证码");
    }
    if let Some(resp) = require_captcha(&data, &ctx, pool, "auth").await {
        return resp;
    }
    let code_row = sqlx::query(
        "SELECT * FROM email_verify_codes WHERE email = ? AND code = ? AND type = 'login' AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .bind(&verify_code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let Some(code_row) = code_row else {
        return ctx.err(400, "验证码无效或已过期");
    };
    let code_id: i64 = code_row.try_get("id").unwrap_or(0);
    let _ = sqlx::query("UPDATE email_verify_codes SET used = 1 WHERE id = ?")
        .bind(code_id)
        .execute(pool)
        .await;
    let user = sqlx::query("SELECT * FROM app_users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(401, "该邮箱未注册");
    };
    let status: i64 = user.get("status");
    if status == 0 {
        let ban_reason: String = user.try_get::<String, _>("ban_reason").unwrap_or_default();
        let reason = ban_reason.trim();
        if reason.is_empty() {
            return ctx.err(403, "账号已被封禁，请联系管理员");
        }
        return ctx.err(403, &format!("账号已被封禁，原因：{}。如有疑问请联系管理员", reason));
    }
    let token = rand_token();
    let user_id: i64 = user.get("id");
    let uname: String = user.try_get::<String, _>("nickname").unwrap_or_default();
    let avatar_url: String = user.get("avatar_url");
    let ciyuanxi_id: String = user.get("ciyuanxi_id");
    let master_quota: i64 = user.get("master_quota");
    let payload = build_user_payload(user_id, &uname, &email, &avatar_url, &ciyuanxi_id, status, master_quota, &token, "");
    ctx.json(200, "登录成功", Some(payload))
}

pub async fn send_verify_code(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let typ = str_of(&data, "type");
    let typ = if typ.is_empty() { "register".to_string() } else { typ };
    if email.is_empty() || !email.contains('@') {
        return ctx.err(400, "邮箱格式不正确");
    }
    if let Some(resp) = require_captcha(&data, &ctx, pool, "auth").await {
        return resp;
    }

    // 注册/绑定邮箱类型：发码前预检查邮箱唯一性（注册还需检查弦予号唯一性）
    if typ == "register" || typ == "bind" {
        let email_bound = sqlx::query("SELECT id FROM app_users WHERE email = ? LIMIT 1")
            .bind(&email)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .is_some();
        if email_bound {
            return ctx.err(400, "该邮箱已绑定账号，请直接登录");
        }
        if typ == "register" {
            let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
            if !ciyuanxi_id.is_empty() {
                let id_dup = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
                    .bind(&ciyuanxi_id)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten()
                    .is_some();
                let id_pretty = sqlx::query("SELECT id FROM ciyuanxi_pretty_ids WHERE ciyuanxi_id = ? LIMIT 1")
                    .bind(&ciyuanxi_id)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten()
                    .is_some();
                if id_dup || id_pretty {
                    return ctx.err(400, "该弦予号已被占用");
                }
            }
        }
    }

    let ip = ctx.client_ip.clone();
    let cnt1: i64 = sqlx::query("SELECT COUNT(*) as cnt FROM email_verify_codes WHERE email = ? AND created_at > DATE_SUB(NOW(), INTERVAL 60 SECOND)")
        .bind(&email)
        .fetch_one(pool)
        .await
        .map(|r| r.get("cnt"))
        .unwrap_or(0);
    if cnt1 > 0 {
        return ctx.err(429, "发送过于频繁，请1分钟后再试");
    }
    let cnt2: i64 = sqlx::query("SELECT COUNT(*) as cnt FROM email_verify_codes WHERE ip = ? AND created_at > DATE_SUB(NOW(), INTERVAL 1 HOUR)")
        .bind(&ip)
        .fetch_one(pool)
        .await
        .map(|r| r.get("cnt"))
        .unwrap_or(0);
    if cnt2 >= 10 {
        return ctx.err(429, "请求过于频繁，请稍后再试");
    }
    let code = format!("{:06}", crate::handlers::helpers::random_int(100000, 999999));
    let _ = sqlx::query("INSERT INTO email_verify_codes (email, code, type, ip, expired_at) VALUES (?,?,?,?,DATE_ADD(NOW(), INTERVAL 10 MINUTE))")
        .bind(&email)
        .bind(&code)
        .bind(&typ)
        .bind(&ip)
        .execute(pool)
        .await;

    // 根据 type 构造邮件标题和正文
    let type_label = match typ.as_str() {
        "login" => "登录",
        "reset_password" => "找回密码",
        "delete_account" => "注销账号",
        "bind" => "绑定邮箱",
        _ => "注册",
    };
    let title = format!("【弦予音乐】{}验证码", type_label);
    let context = format!(
        "您正在进行弦予音乐 APP 的{}操作。\n\n您的验证码是：{}\n\n验证码 10 分钟内有效，请勿泄露给他人。如非本人操作，请忽略此邮件。\n\n—— 弦予音乐",
        type_label, code
    );
    let html = crate::admin::email::build_verify_code_email_html(&type_label, &code);

    // 调用外部邮箱 API 真正发送邮件（HTML 卡片 + 纯文本兜底）
    let send_result = crate::handlers::email_auth::call_email_api_html(
        &ctx.config,
        pool,
        &title,
        &html,
        &context,
        &email,
    )
    .await;

    // 记录发送日志（status: 1=成功, 0=失败）
    let (status_val, error_msg) = match &send_result {
        Ok(()) => (1i64, String::new()),
        Err(e) => {
            eprintln!("[auth] send_verify_code 邮件发送失败 (email={}, type={}): {}", email, typ, e);
            (0i64, e.clone())
        }
    };
    let subject = format!("弦予APP - {}验证码: {}", type_label, code);
    let _ = sqlx::query(
        "INSERT INTO email_send_log (email, subject, interface_id, template_id, status, error_msg, ip) VALUES (?,?,?,?,?,?,?)",
    )
    .bind(&email)
    .bind(&subject)
    .bind(0i64)
    .bind(0i64)
    .bind(status_val)
    .bind(&error_msg)
    .bind(&ip)
    .execute(pool)
    .await;

    match send_result {
        Ok(()) => ctx.ok_empty("验证码已发送，请查收邮件"),
        Err(_) => ctx.err(500, "邮件发送失败，请稍后重试或检查邮箱地址"),
    }
}

pub async fn reset_password(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let verify_code = str_of(&data, "verify_code").trim().to_string();
    let new_password = str_of(&data, "new_password");
    if email.is_empty() || !email.contains('@') {
        return ctx.err(400, "邮箱格式不正确");
    }
    if verify_code.is_empty() {
        return ctx.err(400, "请输入验证码");
    }
    if new_password.len() < 6 {
        return ctx.err(400, "新密码长度至少6位");
    }
    if let Some(resp) = require_captcha(&data, &ctx, pool, "auth").await {
        return resp;
    }
    let exists = sqlx::query("SELECT id FROM app_users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if !exists {
        return ctx.err(400, "该邮箱未注册");
    }
    let code_row = sqlx::query(
        "SELECT * FROM email_verify_codes WHERE email = ? AND code = ? AND type = 'reset_password' AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .bind(&verify_code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let Some(code_row) = code_row else {
        return ctx.err(400, "验证码无效或已过期");
    };
    let code_id: i64 = code_row.try_get("id").unwrap_or(0);
    let _ = sqlx::query("UPDATE email_verify_codes SET used = 1 WHERE id = ?")
        .bind(code_id)
        .execute(pool)
        .await;
    let hashed = match bcrypt::hash(&new_password, 10) {
        Ok(h) => h,
        Err(_) => return ctx.err(500, "密码加密失败"),
    };
    let _ = sqlx::query("UPDATE app_users SET password = ? WHERE email = ?")
        .bind(&hashed)
        .bind(&email)
        .execute(pool)
        .await;
    ctx.ok_empty("密码修改成功")
}

pub async fn delete_account(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let email = str_of(&data, "email").trim().to_string();
    let verify_code = str_of(&data, "verify_code").trim().to_string();
    let password = str_of(&data, "password").trim().to_string();

    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "账号标识不能为空");
    }
    if email.is_empty() || !email.contains('@') {
        return ctx.err(400, "邮箱格式不正确");
    }
    if verify_code.is_empty() {
        return ctx.err(400, "请输入邮箱验证码");
    }
    if password.is_empty() {
        return ctx.err(400, "请输入登录密码");
    }

    let user = sqlx::query("SELECT id, email, password FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "账号不存在或已注销");
    };

    let user_id: i64 = user.try_get("id").unwrap_or(0);
    let registered_email: String = user.try_get("email").unwrap_or_default();
    if registered_email.trim().to_lowercase() != email.trim().to_lowercase() {
        return ctx.err(400, "邮箱与当前账号不匹配");
    }

    // 验证登录密码（双重验证：密码 + 邮箱验证码）
    let stored_password: String = user.try_get("password").unwrap_or_default();
    if !stored_password.is_empty() && !bcrypt::verify(&password, &stored_password).unwrap_or(false) {
        return ctx.err(400, "登录密码错误");
    }

    let code_row = sqlx::query(
        "SELECT * FROM email_verify_codes WHERE email = ? AND code = ? AND type = 'delete_account' AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&registered_email)
    .bind(&verify_code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let Some(code_row) = code_row else {
        return ctx.err(400, "验证码无效或已过期");
    };

    let code_id: i64 = code_row.try_get("id").unwrap_or(0);
    let _ = sqlx::query("UPDATE email_verify_codes SET used = 1 WHERE id = ?")
        .bind(code_id)
        .execute(pool)
        .await;

    let playlists = sqlx::query("SELECT id, cover_path FROM user_playlists WHERE user_id = ?")
        .bind(&ciyuanxi_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    for pl in &playlists {
        let cover: String = pl.try_get("cover_path").unwrap_or_default();
        if !cover.is_empty() {
            let abs = std::path::Path::new("uploads").join("playlists").join(cover);
            if abs.is_file() {
                let _ = std::fs::remove_file(&abs);
            }
        }
        let playlist_id: i64 = pl.try_get("id").unwrap_or(0);
        let _ = sqlx::query("DELETE FROM user_playlist_songs WHERE playlist_id = ?")
            .bind(playlist_id)
            .execute(pool)
            .await;
    }
    let _ = sqlx::query("DELETE FROM user_playlist_songs WHERE user_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_playlists WHERE user_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM play_history WHERE user_id = ? OR ciyuanxi_id = ?")
        .bind(user_id)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM admin_app_login_log WHERE admin_id = ?")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_settings WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_avatar_pending WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_nickname_pending WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_feedback WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM master_quota_usage_log WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("UPDATE ciyuanxi_pretty_ids SET assigned_user_id = '0', assigned_at = NULL WHERE assigned_user_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM auth_rate_limits WHERE identifier = ? OR identifier = ?")
        .bind(&ciyuanxi_id)
        .bind(&registered_email)
        .execute(pool)
        .await;
    let result = sqlx::query("DELETE FROM app_users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await;

    match result {
        Ok(_) => ctx.ok_empty("账号已注销"),
        Err(e) => ctx.err(500, &format!("注销失败: {}", e)),
    }
}

/// 预验证注销凭据（密码 + 邮箱验证码），仅校验不执行实际注销。
/// 用于客户端弹出二级确认弹窗时提前验证，减少用户确认后的等待时间。
pub async fn preverify_delete_account(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let email = str_of(&data, "email").trim().to_string();
    let verify_code = str_of(&data, "verify_code").trim().to_string();
    let password = str_of(&data, "password").trim().to_string();

    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "账号标识不能为空");
    }
    if email.is_empty() || !email.contains('@') {
        return ctx.err(400, "邮箱格式不正确");
    }
    if verify_code.is_empty() {
        return ctx.err(400, "请输入邮箱验证码");
    }
    if password.is_empty() {
        return ctx.err(400, "请输入登录密码");
    }

    let user = sqlx::query("SELECT id, email, password FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "账号不存在或已注销");
    };

    let registered_email: String = user.try_get("email").unwrap_or_default();
    if registered_email.trim().to_lowercase() != email.trim().to_lowercase() {
        return ctx.err(400, "邮箱与当前账号不匹配");
    }

    // 验证登录密码
    let stored_password: String = user.try_get("password").unwrap_or_default();
    if !stored_password.is_empty() && !bcrypt::verify(&password, &stored_password).unwrap_or(false) {
        return ctx.err(400, "登录密码错误");
    }

    // 验证邮箱验证码（仅校验，不标记为已使用，留给实际注销接口标记）
    let code_row = sqlx::query(
        "SELECT id FROM email_verify_codes WHERE email = ? AND code = ? AND type = 'delete_account' AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&registered_email)
    .bind(&verify_code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    if code_row.is_none() {
        return ctx.err(400, "验证码无效或已过期");
    }

    ctx.ok_empty("凭据验证通过")
}
