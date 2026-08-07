use axum::response::Response;
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};

fn str_of<'a>(v: &'a serde_json::Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

/// 后台管理员登录（不需要登录态）
pub async fn admin_login(body: &str, cfg: &crate::config::Config, pool: &MySqlPool, ip: &str) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let username = str_of(&data, "username").trim().to_string();
    let password = str_of(&data, "password").to_string();
    if username.is_empty() || password.is_empty() {
        return err(400, "请输入用户名和密码");
    }
    let row = sqlx::query("SELECT id, password, role FROM admin_users WHERE username = ? AND status = 1")
        .bind(&username)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(admin) = row else {
        let _ = sqlx::query("INSERT INTO admin_login_log (admin_id, admin_username, ip, user_agent, status) VALUES (0, ?, ?, ?, 0)")
            .bind(&username)
            .bind(ip)
            .bind("")
            .execute(pool)
            .await;
        return err(401, "用户名或密码错误");
    };
    let id: i64 = admin.get("id");
    let stored: String = admin.get("password");
    if !bcrypt::verify(&password, &stored).unwrap_or(false) {
        let _ = sqlx::query("INSERT INTO admin_login_log (admin_id, admin_username, ip, user_agent, status) VALUES (0, ?, ?, ?, 0)")
            .bind(&username)
            .bind(ip)
            .bind("")
            .execute(pool)
            .await;
        return err(401, "用户名或密码错误");
    }
    let role: String = admin.get("role");
    let token = super::sign_token(cfg, id, &username, &role);
    let _ = sqlx::query("INSERT INTO admin_login_log (admin_id, admin_username, ip, user_agent, status) VALUES (?, ?, ?, ?, 1)")
        .bind(id)
        .bind(&username)
        .bind(ip)
        .bind("")
        .execute(pool)
        .await;
    ok("登录成功", serde_json::json!({
        "token": token,
        "admin_id": id,
        "username": username,
        "role": role,
        "expires_in": 86400
    }))
}

/// 退出登录（JWT 无服务端状态，仅记录操作日志）
pub async fn admin_logout(ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    log_operation(pool, ctx, "退出登录", "", "").await;
    ok("已退出", serde_json::Value::Null)
}

/// 修改密码（需要登录态）
pub async fn change_password(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let old_password = str_of(&data, "old_password").to_string();
    let new_password = str_of(&data, "new_password").to_string();
    let confirm_password = str_of(&data, "confirm_password").to_string();
    if old_password.is_empty() || new_password.is_empty() {
        return err(400, "请填写旧密码和新密码");
    }
    if new_password != confirm_password {
        return err(400, "两次输入的新密码不一致");
    }
    if new_password.len() < 6 {
        return err(400, "新密码长度不能少于6位");
    }
    let row = sqlx::query("SELECT password FROM admin_users WHERE id = ?")
        .bind(ctx.id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(admin) = row else {
        return err(400, "管理员不存在");
    };
    let stored: String = admin.get("password");
    if !bcrypt::verify(&old_password, &stored).unwrap_or(false) {
        return err(400, "旧密码不正确");
    }
    let hashed = match bcrypt::hash(&new_password, 10) {
        Ok(h) => h,
        Err(_) => return err(500, "加密失败"),
    };
    let _ = sqlx::query("UPDATE admin_users SET password = ? WHERE id = ?")
        .bind(hashed)
        .bind(ctx.id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "修改密码", "", "").await;
    ok("密码修改成功", serde_json::Value::Null)
}