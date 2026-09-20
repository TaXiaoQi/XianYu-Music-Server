use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, int_of, log_operation, ok, row_to_value, str_of, AdminCtx};

pub async fn admin_login(body: &str, cfg: &crate::config::Config, pool: &MySqlPool, ip: &str) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let username = str_of(&data, "username").trim().to_string();
    let password = str_of(&data, "password").to_string();
    if username.is_empty() || password.is_empty() {
        return err(400, "请输入用户名和密码");
    }
    let row = sqlx::query("SELECT id, password, role, avatar_url FROM admin_users WHERE username = ? AND status = 1")
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
    let avatar_url: String = admin.get("avatar_url");
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
    let must_change_password = username == "admin" && password == "adminadmin";
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
    let _ = sqlx::query("UPDATE admin_users SET password = ?, token_invalid_before = UNIX_TIMESTAMP() WHERE id = ?")
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