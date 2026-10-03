use super::*;

pub async fn send_code(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let captcha_token = {
        let token = str_of(&data, "captcha_token");
        if token.trim().is_empty() {
            str_of(&data, "turnstile_token")
        } else {
            token
        }
    };

    if email.is_empty() || !is_valid_email(&email) {
        return ctx.err(400, "邮箱地址格式不正确");
    }

    let captcha_config = load_captcha_config(pool, &ctx.config).await;
    match verify_captcha_token(&captcha_config, &captcha_token, &ctx.client_ip).await {
        Ok(true) => {}
        Ok(false) => return ctx.err(400, "请先完成人机验证"),
        Err(e) => {
            eprintln!("[email_auth] 人机验证校验失败: {}", e);
            return ctx.err(500, "人机验证服务暂不可用，请稍后重试");
        }
    }

    let recent = sqlx::query(
        "SELECT id FROM email_test_codes WHERE email = ? AND created_at > DATE_SUB(NOW(), INTERVAL 60 SECOND) ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if recent.is_some() {
        return ctx.err(400, "发送过于频繁，请 60 秒后再试");
    }

    let code = format!("{:06}", random_int(0, 999999));

    let _ = sqlx::query(
        "INSERT INTO email_test_codes (email, code, type, expired_at) VALUES (?, ?, 'register', DATE_ADD(NOW(), INTERVAL 5 MINUTE))",
    )
    .bind(&email)
    .bind(&code)
    .execute(pool)
    .await;

    let title = "【弦予】您的邮箱验证码";
    let context = format!(
        "您正在进行弦予测试系统的注册/登录操作。\n\n您的验证码是：{}\n\n验证码 5 分钟内有效，请勿泄露给他人。\n\n—— 弦予邮箱注册登录测试系统",
        code
    );

    match call_email_api(&ctx.config, pool, title, &context, &email).await {
        Ok(()) => ctx.ok("验证码已发送，请查收邮件", Value::Null),
        Err(e) => {
            eprintln!("[email_auth] send_code 发送失败: {}", e);
            ctx.err(500, "邮件发送失败，请稍后重试")
        }
    }
}

pub async fn register(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let code = str_of(&data, "code").trim().to_string();
    let password = str_of(&data, "password");
    let password2 = str_of(&data, "password2");
    let nickname = str_of(&data, "nickname").trim().to_string();

    if email.is_empty() || !is_valid_email(&email) {
        return ctx.err(400, "请输入合法的邮箱地址");
    }
    if code.is_empty() {
        return ctx.err(400, "请输入邮箱验证码");
    }
    if password.len() < 6 || password.len() > 32 {
        return ctx.err(400, "密码长度需为 6-32 位");
    }
    if password != password2 {
        return ctx.err(400, "两次输入的密码不一致");
    }

    let code_row = sqlx::query(
        "SELECT id FROM email_test_codes WHERE email = ? AND code = ? AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .bind(&code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(code_row) = code_row else {
        return ctx.err(400, "验证码不正确或已过期");
    };
    let code_id: i64 = code_row.try_get("id").unwrap_or(0);

    let existing = sqlx::query("SELECT id FROM email_test_users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if existing.is_some() {
        return ctx.err(400, "该邮箱已注册，请直接登录");
    }

    let hash = match bcrypt::hash(&password, 10) {
        Ok(h) => h,
        Err(_) => return ctx.err(500, "密码加密失败"),
    };

    let ins = sqlx::query("INSERT INTO email_test_users (email, password, nickname) VALUES (?, ?, ?)")
        .bind(&email)
        .bind(&hash)
        .bind(&nickname)
        .execute(pool)
        .await;

    match ins {
        Ok(result) => {
            let uid = result.last_insert_id() as i64;
            let _ = sqlx::query("UPDATE email_test_codes SET used = 1 WHERE id = ?")
                .bind(code_id)
                .execute(pool)
                .await;
            log_action(pool, uid, &email, "register", &ctx.client_ip).await;

            ctx.ok("注册成功", Value::Null)
        }
        Err(_) => ctx.err(500, "注册失败，请稍后重试"),
    }
}

pub async fn login(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let password = str_of(&data, "password");

    if email.is_empty() || password.is_empty() {
        return ctx.err(400, "请输入邮箱和密码");
    }

    let row = sqlx::query("SELECT id, email, password, nickname, status FROM email_test_users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    let Some(row) = row else {
        return ctx.err(400, "邮箱或密码不正确");
    };

    let hash: String = row.try_get("password").unwrap_or_default();
    if !bcrypt::verify(&password, &hash).unwrap_or(false) {
        return ctx.err(400, "邮箱或密码不正确");
    }

    let status: i64 = row.try_get("status").unwrap_or(1);
    if status == 0 {
        return ctx.err(403, "账号已被禁用，请联系管理员");
    }

    let login_device_id = str_of(&data, "device_id").trim().to_string();
    if !login_device_id.is_empty() {
        let banned = sqlx::query("SELECT reason FROM banned_devices WHERE device_id = ? LIMIT 1")
            .bind(&login_device_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        if let Some(row) = banned {
            let reason: String = row.try_get::<String, _>("reason").unwrap_or_default();
            let reason = reason.trim();
            if reason.is_empty() {
                return ctx.err(403, "该设备已被封禁，请联系管理员");
            }
            return ctx.err(403, &format!("该设备已被封禁，原因：{}。如有疑问请联系管理员", reason));
        }
    }

    let uid: i64 = row.try_get("id").unwrap_or(0);
    let user_email: String = row.try_get("email").unwrap_or_default();
    let nickname: String = row.try_get("nickname").unwrap_or_default();

    let _ = sqlx::query("UPDATE email_test_users SET last_login = NOW() WHERE id = ?")
        .bind(uid)
        .execute(pool)
        .await;

    log_action(pool, uid, &user_email, "login", &ctx.client_ip).await;

    let token = sign_email_token(&ctx.config, uid, &user_email);

    ctx.ok(
        "登录成功",
        json!({
            "token": token,
            "user": {
                "id": uid,
                "email": user_email,
                "nickname": nickname,
            }
        }),
    )
}

pub async fn reset_password(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let code = str_of(&data, "code").trim().to_string();
    let password = str_of(&data, "password");
    let password2 = str_of(&data, "password2");

    if email.is_empty() || !is_valid_email(&email) {
        return ctx.err(400, "请输入合法的邮箱地址");
    }
    if code.is_empty() {
        return ctx.err(400, "请输入邮箱验证码");
    }
    if password.len() < 6 || password.len() > 32 {
        return ctx.err(400, "密码长度需为 6-32 位");
    }
    if password != password2 {
        return ctx.err(400, "两次输入的密码不一致");
    }

    let code_row = sqlx::query(
        "SELECT id FROM email_test_codes WHERE email = ? AND code = ? AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .bind(&code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(code_row) = code_row else {
        return ctx.err(400, "验证码不正确或已过期");
    };
    let code_id: i64 = code_row.try_get("id").unwrap_or(0);

    let user_row = sqlx::query("SELECT id FROM email_test_users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user_row) = user_row else {
        return ctx.err(400, "该邮箱尚未注册，请先注册");
    };
    let user_id: i64 = user_row.try_get("id").unwrap_or(0);

    let hash = match bcrypt::hash(&password, 10) {
        Ok(h) => h,
        Err(_) => return ctx.err(500, "密码加密失败"),
    };
    let _ = sqlx::query("UPDATE email_test_users SET password = ? WHERE id = ?")
        .bind(&hash)
        .bind(user_id)
        .execute(pool)
        .await;

    let _ = sqlx::query("UPDATE email_test_codes SET used = 1 WHERE id = ?")
        .bind(code_id)
        .execute(pool)
        .await;

    log_action(pool, user_id, &email, "reset_password", &ctx.client_ip).await;

    ctx.ok("密码已重置成功", Value::Null)
}

pub async fn get_profile(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let token = str_of(&data, "token");

    let claims = match verify_email_token(&ctx.config, &token) {
        Some(c) => c,
        None => return ctx.err(401, "未登录或登录已过期"),
    };

    let row = sqlx::query(
        "SELECT id, email, nickname, status, created_at, last_login FROM email_test_users WHERE id = ?",
    )
    .bind(claims.sub)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(row) = row else {
        return ctx.err(404, "用户不存在");
    };

    let status: i64 = row.try_get("status").unwrap_or(1);
    if status == 0 {
        return ctx.err(403, "账号已被禁用");
    }

    let logs = sqlx::query(
        "SELECT action, detail, created_at FROM email_test_logs WHERE user_id = ? ORDER BY id DESC LIMIT 8",
    )
    .bind(claims.sub)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let logs_arr: Vec<Value> = logs
        .iter()
        .map(|r| {
            json!({
                "action": r.try_get::<String, _>("action").unwrap_or_default(),
                "detail": r.try_get::<String, _>("detail").unwrap_or_default(),
                "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
            })
        })
        .collect();

    ctx.ok(
        "",
        json!({
            "id": row.try_get::<i64, _>("id").unwrap_or(0),
            "email": row.try_get::<String, _>("email").unwrap_or_default(),
            "nickname": row.try_get::<String, _>("nickname").unwrap_or_default(),
            "status": status,
            "created_at": row.try_get::<String, _>("created_at").unwrap_or_default(),
            "last_login": row.try_get::<Option<String>, _>("last_login").ok().flatten().unwrap_or_default(),
            "logs": logs_arr,
        }),
    )
}

// ============================================================
//  单元测试
// ============================================================

