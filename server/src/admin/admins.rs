use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, row_to_value, AdminCtx};

fn str_of<'a>(v: &'a serde_json::Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

fn int_of(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key)
        .and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0)
}

/// 管理员列表 + 统计
pub async fn list_admins(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    // 查询列表（不返回 password 字段）
    let list: Vec<Value> = match sqlx::query(
        "SELECT id, username, email, avatar_url, role, status, created_at, updated_at FROM admin_users ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows.iter().map(row_to_value).collect(),
        Err(_) => return err(500, "数据库错误"),
    };

    // 统计
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users")
        .fetch_one(pool).await.unwrap_or(0);
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE status = 1")
        .fetch_one(pool).await.unwrap_or(0);
    let super_admin: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE role = 'super_admin'")
        .fetch_one(pool).await.unwrap_or(0);
    let admin: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE role = 'admin'")
        .fetch_one(pool).await.unwrap_or(0);

    ok("ok", json!({
        "list": list,
        "stats": {
            "total": total,
            "active": active,
            "disabled": total - active,
            "super_admin": super_admin,
            "admin": admin,
        }
    }))
}

/// 切换管理员状态（启用/禁用）
pub async fn toggle_admin_status(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    if ctx.role != "super_admin" {
        return err(403, "仅超级管理员可以管理管理员账号");
    }
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let id = int_of(&data, "id");
    if id == 0 {
        return err(400, "参数错误");
    }
    if id == ctx.id {
        return err(400, "不能修改自己的状态");
    }
    // 查询当前状态
    let current: Option<i32> = sqlx::query_scalar("SELECT status FROM admin_users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match current {
        Some(status) => {
            let new_status = if status == 1 { 0 } else { 1 };
            // 禁止禁用最后一个启用中的超级管理员
            if status == 1 && new_status == 0 {
                let role: Option<String> = sqlx::query_scalar("SELECT role FROM admin_users WHERE id = ?")
                    .bind(id)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten();
                if role.as_deref() == Some("super_admin") {
                    let active_super: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE role = 'super_admin' AND status = 1")
                        .fetch_one(pool)
                        .await
                        .unwrap_or(0);
                    if active_super <= 1 {
                        return err(400, "系统必须保留至少一个启用的超级管理员");
                    }
                }
            }
            let _ = sqlx::query("UPDATE admin_users SET status = ? WHERE id = ?")
                .bind(new_status)
                .bind(id)
                .execute(pool)
                .await;
            log_operation(pool, ctx, "切换管理员状态", &format!("id={}", id), &format!("{}->{}", status, new_status)).await;
            ok("状态已更新", serde_json::Value::Null)
        }
        None => err(404, "管理员不存在"),
    }
}

pub async fn add_admin(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    if ctx.role != "super_admin" {
        return err(403, "仅超级管理员可以新增管理员账号");
    }
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let username = str_of(&data, "username").trim().to_string();
    let password = str_of(&data, "password").to_string();
    let role = str_of(&data, "role").to_string();
    let email = str_of(&data, "email").trim().to_string();
    if username.is_empty() || password.is_empty() {
        return err(400, "用户名和密码不能为空");
    }
    if !email.is_empty() && !super::is_valid_email(&email) {
        return err(400, "邮箱格式不正确");
    }
    // 超级管理员全局只能有一个
    if role == "super_admin" {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE role = 'super_admin'")
            .fetch_one(pool)
            .await
            .unwrap_or(0);
        if count >= 1 {
            return err(400, "超级管理员已存在，全局最多只能有一个超级管理员");
        }
    }
    let exists = sqlx::query("SELECT id FROM admin_users WHERE username = ?")
        .bind(&username)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if exists {
        return err(400, "用户名已存在");
    }
    let hashed = match bcrypt::hash(&password, 10) {
        Ok(h) => h,
        Err(_) => return err(500, "加密失败"),
    };
    let _ = sqlx::query("INSERT INTO admin_users (username, password, email, avatar_url, role, status) VALUES (?,?,?,?,?,1)")
        .bind(&username)
        .bind(hashed)
        .bind(&email)
        .bind("")
        .bind(&role)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "新增管理员", &username, &format!("角色:{} 邮箱:{}", role, email)).await;
    ok("添加成功", serde_json::Value::Null)
}

pub async fn delete_admin(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    if ctx.role != "super_admin" {
        return err(403, "仅超级管理员可以删除管理员账号");
    }
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let id = int_of(&data, "id");
    if id == ctx.id {
        return err(400, "不能删除自己");
    }
    // 禁止删除最后一个超级管理员
    let role: Option<String> = sqlx::query_scalar("SELECT role FROM admin_users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if role.as_deref() == Some("super_admin") {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE role = 'super_admin'")
            .fetch_one(pool)
            .await
            .unwrap_or(0);
        if count <= 1 {
            return err(400, "系统必须保留至少一个超级管理员");
        }
    }
    let _ = sqlx::query("DELETE FROM admin_users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "删除管理员", &format!("管理员ID:{}", id), "").await;
    ok("删除成功", serde_json::Value::Null)
}
