use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, row_to_value, AdminCtx};

fn str_of<'a>(v: &'a serde_json::Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

fn int_of(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key)
        .and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0)
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
/// 支持分级：普通管理员只能修改自己的密码，超级管理员可修改任意管理员密码。
/// 修改自己的密码需要校验旧密码；超管修改他人密码时无需旧密码。
pub async fn change_password(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let admin_id = int_of(&data, "admin_id");
    let old_password = str_of(&data, "old_password").to_string();
    let new_password = str_of(&data, "new_password").to_string();
    let confirm_password = str_of(&data, "confirm_password").to_string();
    if new_password.is_empty() {
        return err(400, "请填写新密码");
    }
    if new_password != confirm_password {
        return err(400, "两次输入的新密码不一致");
    }
    if new_password.len() < 6 {
        return err(400, "新密码长度不能少于6位");
    }
    // 目标管理员：默认自己
    let target_id = if admin_id > 0 { admin_id } else { ctx.id };
    // 权限分级：修改他人密码仅超管可操作
    if target_id != ctx.id && ctx.role != "super_admin" {
        return err(403, "仅超级管理员可以修改其他管理员密码");
    }
    // 修改自己密码需校验旧密码；超管修改他人时无需旧密码
    if target_id == ctx.id && old_password.is_empty() {
        return err(400, "请填写旧密码");
    }
    let row = sqlx::query("SELECT password, username FROM admin_users WHERE id = ?")
        .bind(target_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(admin) = row else {
        return err(404, "管理员不存在");
    };
    if target_id == ctx.id {
        let stored: String = admin.get("password");
        if !bcrypt::verify(&old_password, &stored).unwrap_or(false) {
            return err(400, "旧密码不正确");
        }
    }
    let hashed = match bcrypt::hash(&new_password, 10) {
        Ok(h) => h,
        Err(_) => return err(500, "加密失败"),
    };
    let _ = sqlx::query("UPDATE admin_users SET password = ? WHERE id = ?")
        .bind(hashed)
        .bind(target_id)
        .execute(pool)
        .await;
    let target_username: String = admin.get("username");
    log_operation(pool, ctx, "修改密码", &format!("管理员:{}", target_username), &format!("id={}", target_id)).await;
    ok("密码修改成功", serde_json::Value::Null)
}

/// 获取可修改密码的管理员列表（用于修改密码页面的选择器）
/// 超管返回所有启用管理员，普通管理员仅返回自己
pub async fn list_password_targets(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    if ctx.role == "super_admin" {
        let rows = sqlx::query(
            "SELECT id, username, role, status FROM admin_users WHERE status = 1 ORDER BY id",
        )
        .fetch_all(pool)
        .await;
        let list: Vec<Value> = match rows {
            Ok(rows) => rows.iter().map(row_to_value).collect(),
            Err(_) => return err(500, "数据库错误"),
        };
        ok("ok", json!({ "list": list, "current_id": ctx.id, "role": ctx.role }))
    } else {
        ok("ok", json!({ "list": [{
            "id": ctx.id,
            "username": ctx.username,
            "role": ctx.role,
            "status": 1
        }], "current_id": ctx.id, "role": ctx.role }))
    }
}