use super::*;

pub async fn generate_tv_login_code(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return ctx.err(400, "设备标识不能为空");
    }
    let location = str_of(&data, "location").trim().to_string();
    let _ = sqlx::query("DELETE FROM tv_login_codes WHERE device_id = ? AND created_at < (NOW() - INTERVAL 10 MINUTE)")
        .bind(&device_id)
        .execute(pool)
        .await;
    let code = crate::handlers::helpers::random_hex(16);
    let ip = ctx.client_ip.clone();
    let result = sqlx::query("INSERT INTO tv_login_codes (code, device_id, status, ip, location, expires_at) VALUES (?,?,'pending',?,?,DATE_ADD(NOW(), INTERVAL 5 MINUTE))")
        .bind(&code)
        .bind(&device_id)
        .bind(&ip)
        .bind(&location)
        .execute(pool)
        .await;
    match result {
        Ok(_) => ctx.json(200, "ok", Some(json!({ "code": code, "expire_seconds": 300, "location": location }))),
        Err(e) => { tracing::error!("服务器错误: {e}"); ctx.err(500, "服务器错误") },
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
    let email: String = user.try_get("email").unwrap_or_default();
    let role = resolve_role(pool, &email).await;
    let user_id: i64 = user.get("id");
    let uname: String = user.try_get::<String, _>("nickname").unwrap_or_default();
    let avatar_url = crate::handlers::upload::absolutize_media_url_with(
        &ctx.base_url,
        &ctx.config.public_base_url,
        &user.try_get::<Option<String>, _>("avatar_url").ok().flatten().unwrap_or_default(),
    );
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
    // 扫码必须由本人发起：token 归属校验（fail-closed，防冒领任意账号）
    let owner = token::resolve_owner(pool, str_of(&data, "token").trim()).await.unwrap_or_default();
    if owner.is_empty() {
        return ctx.err(401, "请先登录后再扫码");
    }
    if owner != ciyuanxi_id {
        return ctx.err(403, "登录状态与账号不匹配，请重新登录");
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
    let device_id: String = row.try_get::<String, _>("device_id").unwrap_or_default();
    let location: String = row.try_get::<String, _>("location").unwrap_or_default();
    ctx.json(
        200,
        "扫码成功，请在手机端确认登录",
        Some(json!({
            "app_name": "弦予.桌面版",
            "device_id": device_id,
            "location": location,
            "ciyuanxi_id": ciyuanxi_id,
            "nickname": nickname,
            "username": nickname,
        })),
    )
}

pub async fn confirm_tv_login(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let code = str_of(&data, "code");
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
    if code.is_empty() || ciyuanxi_id.is_empty() {
        return ctx.err(400, "code 和 ciyuanxi_id 不能为空");
    }
    // 确认登录必须由本人发起：token 归属校验（fail-closed，防冒领任意账号）
    let owner = token::resolve_owner(pool, str_of(&data, "token").trim()).await.unwrap_or_default();
    if owner.is_empty() {
        return ctx.err(401, "请先登录后再确认");
    }
    if owner != ciyuanxi_id {
        return ctx.err(403, "登录状态与账号不匹配，请重新登录");
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
    let tv_device_id: String = row.try_get::<String, _>("device_id").unwrap_or_default();
    let token = token::issue(pool, &ciyuanxi_id, &tv_device_id).await;
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
