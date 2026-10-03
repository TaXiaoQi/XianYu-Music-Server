use super::*;

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

    let send_result = crate::handlers::email_auth::call_email_api_html(
        &ctx.config,
        pool,
        &title,
        &html,
        &context,
        &email,
    )
    .await;

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
    if let Ok(rows) = sqlx::query("SELECT ciyuanxi_id FROM app_users WHERE email = ?")
        .bind(&email)
        .fetch_all(pool)
        .await
    {
        for row in rows {
            if let Ok(id) = row.try_get::<String, _>("ciyuanxi_id") {
                token::revoke_user(pool, &id).await;
            }
        }
    }
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
    crate::handlers::sync::delete_user_sync_data(pool, &ciyuanxi_id).await;
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
    token::revoke_user(pool, &ciyuanxi_id).await;
    let result = sqlx::query("DELETE FROM app_users WHERE id = ?")
        .bind(user_id)
        .execute(pool)
        .await;

    match result {
        Ok(_) => ctx.ok_empty("账号已注销"),
        Err(e) => { tracing::error!("注销失败: {e}"); ctx.err(500, "注销失败") },
    }
}

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

    let stored_password: String = user.try_get("password").unwrap_or_default();
    if !stored_password.is_empty() && !bcrypt::verify(&password, &stored_password).unwrap_or(false) {
        return ctx.err(400, "登录密码错误");
    }

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
