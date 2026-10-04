use super::*;

fn normalize_rate_identifier(value: &str) -> String {
    value.trim().to_lowercase()
}

async fn check_login_cooldown(identifier: &str, ctx: &ReqCtx, pool: &MySqlPool) -> Option<Response> {
    let key = normalize_rate_identifier(identifier);
    if key.is_empty() {
        return None;
    }
    // 锁定键为纯 identifier（ip 固定存 ''）：跨 IP 共享失败计数，防换 IP 绕过爆破
    let row = sqlx::query(
        "SELECT TIMESTAMPDIFF(SECOND, NOW(), locked_until) AS remain_seconds FROM auth_rate_limits WHERE action = 'user_login' AND identifier = ? AND ip = '' AND locked_until IS NOT NULL AND locked_until > NOW() LIMIT 1",
    )
    .bind(&key)
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

async fn record_login_failure(identifier: &str, pool: &MySqlPool) {
    let key = normalize_rate_identifier(identifier);
    if key.is_empty() {
        return;
    }
    let current: i64 = sqlx::query(
        "SELECT failed_count FROM auth_rate_limits WHERE action = 'user_login' AND identifier = ? AND ip = '' AND updated_at > DATE_SUB(NOW(), INTERVAL ? MINUTE) LIMIT 1",
    )
    .bind(&key)
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
         VALUES ('user_login', ?, '', ?, IF(? = 1, DATE_ADD(NOW(), INTERVAL ? MINUTE), NULL), NOW())
         ON DUPLICATE KEY UPDATE failed_count = VALUES(failed_count), locked_until = VALUES(locked_until), last_failed_at = NOW()",
    )
    .bind(next)
    .bind(if lock { 1 } else { 0 })
    .bind(LOGIN_LOCK_MINUTES)
    .execute(pool)
    .await;
}

async fn clear_login_failures(identifier: &str, pool: &MySqlPool) {
    let key = normalize_rate_identifier(identifier);
    if key.is_empty() {
        return;
    }
    // 成功登录清掉该账号全部计数行（含历史真实 ip 行）
    let _ = sqlx::query("DELETE FROM auth_rate_limits WHERE action = 'user_login' AND identifier = ?")
        .bind(&key)
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

    if let Err(msg) = validate_ciyuanxi_id(&ciyuanxi_id) {
        return ctx.err(400, msg);
    }
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

    let reg_device_id = str_of(&data, "device_id").trim().to_string();
    if let Some(resp) = check_device_ban(&reg_device_id, &ctx, pool).await {
        return resp;
    }

    let _ip = ctx.client_ip.clone();

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
            let token = token::issue(pool, &ciyuanxi_id, &reg_device_id).await;
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
        Err(e) => { tracing::error!("服务器错误: {e}"); ctx.err(500, "服务器错误") },
    }
}

pub async fn user_login(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
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
        record_login_failure(&matched, pool).await;
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
        record_login_failure(&matched, pool).await;
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

    let user_id: i64 = user.try_get::<i64, _>("id").unwrap_or(0);
    let uname: String = user.try_get::<String, _>("nickname").unwrap_or_default();
    let avatar_url = crate::handlers::upload::absolutize_media_url_with(
        &ctx.base_url,
        &ctx.config.public_base_url,
        &user.try_get::<Option<String>, _>("avatar_url").ok().flatten().unwrap_or_default(),
    );
    let ciyuanxi_id: String = user.try_get::<String, _>("ciyuanxi_id").unwrap_or_default();
    let master_quota: i64 = user.try_get::<i64, _>("master_quota").unwrap_or(0);
    let token = token::issue(pool, &ciyuanxi_id, &login_device_id).await;
    clear_login_failures(&matched, pool).await;

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
    let login_device_id = str_of(&data, "device_id").trim().to_string();
    let user_id: i64 = user.get("id");
    let uname: String = user.try_get::<String, _>("nickname").unwrap_or_default();
    let avatar_url = crate::handlers::upload::absolutize_media_url_with(
        &ctx.base_url,
        &ctx.config.public_base_url,
        &user.try_get::<Option<String>, _>("avatar_url").ok().flatten().unwrap_or_default(),
    );
    let ciyuanxi_id: String = user.get("ciyuanxi_id");
    let master_quota: i64 = user.get("master_quota");
    let token = token::issue(pool, &ciyuanxi_id, &login_device_id).await;
    let payload = build_user_payload(user_id, &uname, &email, &avatar_url, &ciyuanxi_id, status, master_quota, &token, "");
    ctx.json(200, "登录成功", Some(payload))
}
