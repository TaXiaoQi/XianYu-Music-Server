use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, int_of, log_operation, ok, row_to_value, str_of, AdminCtx};

// V-02（渗透修复）：admin_login 账号级失败锁定（跨 IP 生效，防绕过 IP 限流的
// 分布式爆破）。与用户侧 auth_rate_limits 同表复用：identifier=用户名、ip=''。
const ADMIN_LOGIN_LOCK_WINDOW_MINUTES: i64 = 15;
const ADMIN_LOGIN_LOCK_THRESHOLD: i64 = 5;
const ADMIN_LOGIN_LOCK_MINUTES: i64 = 30;

async fn admin_login_locked(username: &str, pool: &MySqlPool) -> bool {
    let locked: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM auth_rate_limits WHERE action = 'admin_login' AND identifier = ? AND ip = '' AND locked_until IS NOT NULL AND locked_until > NOW()",
    )
    .bind(username.trim().to_lowercase())
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    locked > 0
}

async fn record_admin_login_failure(username: &str, pool: &MySqlPool) {
    let key = username.trim().to_lowercase();
    let current: i64 = sqlx::query_scalar(
        "SELECT failed_count FROM auth_rate_limits WHERE action = 'admin_login' AND identifier = ? AND ip = '' AND updated_at > DATE_SUB(NOW(), INTERVAL ? MINUTE) LIMIT 1",
    )
    .bind(&key)
    .bind(ADMIN_LOGIN_LOCK_WINDOW_MINUTES)
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    let next = current + 1;
    let lock = if next >= ADMIN_LOGIN_LOCK_THRESHOLD { 1 } else { 0 };
    let _ = sqlx::query(
        "INSERT INTO auth_rate_limits (action, identifier, ip, failed_count, locked_until, last_failed_at) VALUES ('admin_login', ?, '', ?, IF(? = 1, DATE_ADD(NOW(), INTERVAL ? MINUTE), NULL), NOW()) ON DUPLICATE KEY UPDATE failed_count = VALUES(failed_count), locked_until = VALUES(locked_until), last_failed_at = NOW()",
    )
    .bind(&key)
    .bind(next)
    .bind(lock)
    .bind(ADMIN_LOGIN_LOCK_MINUTES)
    .execute(pool)
    .await;
}

async fn clear_admin_login_failures(username: &str, pool: &MySqlPool) {
    let _ = sqlx::query("DELETE FROM auth_rate_limits WHERE action = 'admin_login' AND identifier = ? AND ip = ''")
        .bind(username.trim().to_lowercase())
        .execute(pool)
        .await;
}

pub async fn admin_login(body: &str, cfg: &crate::config::Config, pool: &MySqlPool, ip: &str) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let username = str_of(&data, "username").trim().to_string();
    let password = str_of(&data, "password").to_string();
    if username.is_empty() || password.is_empty() {
        return err(400, "请输入用户名和密码");
    }
    if admin_login_locked(&username, pool).await {
        return err(429, "失败次数过多，请 30 分钟后再试");
    }
    let row = sqlx::query("SELECT id, password, role, avatar_url, must_change_password FROM admin_users WHERE username = ? AND status = 1")
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
        record_admin_login_failure(&username, pool).await;
        return err(401, "用户名或密码错误");
    };
    let id: i64 = admin.get("id");
    let stored: String = admin.get("password");
    let avatar_url: String = admin.get("avatar_url");
    if !bcrypt::verify(&password, &stored).unwrap_or(false) {
        let _ = sqlx::query("INSERT INTO admin_login_log (admin_id, admin_username, ip, user_agent, status) VALUES (0, ?, ?, ?, 0)")
            .bind(&username)
            .bind(ip)
            .bind("")
            .execute(pool)
            .await;
        record_admin_login_failure(&username, pool).await;
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
    clear_admin_login_failures(&username, pool).await;
    // V-02（渗透修复）：首启强制改密改由 DB 标记（must_change_password）驱动，
    // 源码不再内置任何默认口令
    let must_change_password: i64 = admin.try_get("must_change_password").unwrap_or(0);
    let must_change_password = must_change_password != 0;
    ok("登录成功", serde_json::json!({
        "token": token,
        "admin_id": id,
        "username": username,
        "role": role,
        "avatar_url": avatar_url,
        "expires_in": 86400,
        "must_change_password": must_change_password
    }))
}

pub async fn admin_logout(ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    log_operation(pool, ctx, "退出登录", "", "").await;
    ok("已退出", serde_json::Value::Null)
}

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
    let target_id = if admin_id > 0 { admin_id } else { ctx.id };
    if target_id != ctx.id && ctx.role != "super_admin" {
        return err(403, "仅超级管理员可以修改其他管理员密码");
    }
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
    let _ = sqlx::query("UPDATE admin_users SET password = ?, token_invalid_before = UNIX_TIMESTAMP(), must_change_password = 0 WHERE id = ?")
        .bind(hashed)
        .bind(target_id)
        .execute(pool)
        .await;
    let target_username: String = admin.get("username");
    log_operation(pool, ctx, "修改密码", &format!("管理员:{}", target_username), &format!("id={}", target_id)).await;
    ok("密码修改成功", serde_json::Value::Null)
}

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

pub async fn change_login(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let admin_id = int_of(&data, "admin_id");
    let old_password = str_of(&data, "old_password").to_string();
    let new_username = str_of(&data, "new_username").trim().to_string();
    let new_email = str_of(&data, "new_email").trim().to_string();
    let new_password = str_of(&data, "new_password").to_string();
    let confirm_password = str_of(&data, "confirm_password").to_string();
    if new_username.is_empty() {
        return err(400, "用户名不能为空");
    }
    if !new_email.is_empty() && !super::is_valid_email(&new_email) {
        return err(400, "邮箱格式不正确");
    }
    let target_id = if admin_id > 0 { admin_id } else { ctx.id };
    if target_id != ctx.id && ctx.role != "super_admin" {
        return err(403, "仅超级管理员可以修改其他管理员登录信息");
    }
    let change_pwd = !new_password.is_empty() || !confirm_password.is_empty();
    if change_pwd {
        if new_password.is_empty() {
            return err(400, "请填写新密码");
        }
        if new_password != confirm_password {
            return err(400, "两次输入的新密码不一致");
        }
        if new_password.len() < 6 {
            return err(400, "新密码长度不能少于6位");
        }
        if target_id == ctx.id && old_password.is_empty() {
            return err(400, "请填写当前密码");
        }
    }
    let row = sqlx::query("SELECT password, username, email FROM admin_users WHERE id = ?")
        .bind(target_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(admin) = row else {
        return err(404, "管理员不存在");
    };
    if change_pwd && target_id == ctx.id {
        let stored: String = admin.get("password");
        if !bcrypt::verify(&old_password, &stored).unwrap_or(false) {
            return err(400, "当前密码不正确");
        }
    }
    let exists = sqlx::query("SELECT id FROM admin_users WHERE username = ? AND id != ?")
        .bind(&new_username)
        .bind(target_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if exists {
        return err(400, "用户名已存在");
    }
    let _ = sqlx::query("UPDATE admin_users SET username = ? WHERE id = ?")
        .bind(&new_username)
        .bind(target_id)
        .execute(pool)
        .await;
    let old_email: String = admin.get("email");
    if !new_email.is_empty() && new_email != old_email {
        let _ = sqlx::query("UPDATE admin_users SET email = ? WHERE id = ?")
            .bind(&new_email)
            .bind(target_id)
            .execute(pool)
            .await;
    }
    if change_pwd {
        let hashed = match bcrypt::hash(&new_password, 10) {
            Ok(h) => h,
            Err(_) => return err(500, "加密失败"),
        };
        let _ = sqlx::query("UPDATE admin_users SET password = ?, token_invalid_before = UNIX_TIMESTAMP() WHERE id = ?")
            .bind(hashed)
            .bind(target_id)
            .execute(pool)
            .await;
    }
    let old_username: String = admin.get("username");
    let detail = format!("{} -> {} 邮箱:{}", old_username, new_username, if new_email.is_empty() { "未变".to_string() } else { new_email });
    log_operation(pool, ctx, if change_pwd { "修改登录信息" } else { "修改用户名" }, &detail, &format!("id={}", target_id)).await;
    let msg = if change_pwd { "登录信息修改成功" } else { "用户名修改成功" };
    ok(msg, serde_json::Value::Null)
}