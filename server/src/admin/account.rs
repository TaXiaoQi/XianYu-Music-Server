use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};

fn str_of<'a>(v: &'a serde_json::Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

fn int_of(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key)
        .and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0)
}

/// 获取当前管理员账户信息
pub async fn get_account_info(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let row = sqlx::query("SELECT id, username, email, role, status, created_at, updated_at FROM admin_users WHERE id = ?")
        .bind(ctx.id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(admin) = row else {
        return err(404, "管理员不存在");
    };
    let username: String = admin.get("username");
    let email: String = admin.get("email");
    let role: String = admin.get("role");
    let status: i32 = admin.get("status");
    let created_at: String = admin.get("created_at");
    let updated_at: String = admin.get("updated_at");

    let operation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_operation_log WHERE admin_id = ?")
        .bind(ctx.id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let last_login: Option<(String, String)> = sqlx::query("SELECT ip, created_at FROM admin_login_log WHERE admin_id = ? ORDER BY id DESC LIMIT 1")
        .bind(ctx.id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|r| {
            let ip: String = r.get("ip");
            let t: String = r.get("created_at");
            (ip, t)
        });

    ok("ok", json!({
        "id": ctx.id,
        "username": username,
        "email": email,
        "role": role,
        "status": status,
        "created_at": created_at,
        "updated_at": updated_at,
        "operation_count": operation_count,
        "last_login_ip": last_login.as_ref().map(|(ip, _)| ip.as_str()).unwrap_or("未知"),
        "last_login_time": last_login.as_ref().map(|(_, t)| t.as_str()).unwrap_or("未知"),
    }))
}

/// 绑定邮箱
pub async fn bind_email(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let email = str_of(&data, "email").trim().to_string();
    if email.is_empty() || !crate::admin::is_valid_email(&email) {
        return err(400, "邮箱格式不正确");
    }
    let exists = sqlx::query("SELECT id FROM admin_users WHERE email = ? AND id != ?")
        .bind(&email)
        .bind(ctx.id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if exists {
        return err(400, "该邮箱已被绑定");
    }
    let _ = sqlx::query("UPDATE admin_users SET email = ? WHERE id = ?")
        .bind(&email)
        .bind(ctx.id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "绑定邮箱", &email, "").await;
    ok("邮箱绑定成功", serde_json::Value::Null)
}

/// 修改用户名
pub async fn change_username(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let new_username = str_of(&data, "new_username").trim().to_string();
    if new_username.is_empty() {
        return err(400, "用户名不能为空");
    }
    let exists = sqlx::query("SELECT id FROM admin_users WHERE username = ? AND id != ?")
        .bind(&new_username)
        .bind(ctx.id)
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
        .bind(ctx.id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "修改用户名", &new_username, "").await;
    ok("用户名修改成功", serde_json::Value::Null)
}

/// 修改用户邮箱
pub async fn change_user_email(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let user_id = int_of(&data, "user_id");
    let new_email = str_of(&data, "new_email").trim().to_string();
    if user_id <= 0 {
        return err(400, "用户参数错误");
    }
    if !new_email.is_empty() && !crate::admin::is_valid_email(&new_email) {
        return err(400, "邮箱格式不正确");
    }
    let user = sqlx::query("SELECT email, username FROM app_users WHERE id = ? LIMIT 1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return err(404, "用户不存在");
    };
    let old_email: String = user.get("email");
    let mut role_hint = "普通成员".to_string();
    if !new_email.is_empty() {
        let admin = sqlx::query("SELECT role FROM admin_users WHERE email = ? AND status = 1 LIMIT 1")
            .bind(&new_email)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        if let Some(admin) = admin {
            let role: String = admin.get("role");
            role_hint = if role == "super_admin" {
                "超级管理员".to_string()
            } else {
                "管理员".to_string()
            };
        }
    }
    let _ = sqlx::query("UPDATE app_users SET email = ? WHERE id = ?")
        .bind(&new_email)
        .bind(user_id)
        .execute(pool)
        .await;
    let detail = format!("user_id={} {} -> {} ({})", user_id, old_email, new_email, role_hint);
    log_operation(pool, ctx, "修改用户邮箱", &detail, "").await;
    ok(&format!("修改成功，身份：{}", role_hint), serde_json::json!({ "role": role_hint }))
}

/// 重置用户听歌时长与新歌数
pub async fn reset_listen_duration(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let mut ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    }
    if ciyuanxi_id.is_empty() {
        return err(400, "用户参数错误");
    }
    let _ = sqlx::query("UPDATE app_users SET listen_duration = 0, unique_songs_count = 0, listen_stats_reset_at = NOW() WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    // 清理每日统计表，确保日榜/周榜也从零开始
    let _ = sqlx::query("DELETE FROM listen_daily_stats WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "重置听歌时长", &format!("ciyuanxi_id={}", ciyuanxi_id), "").await;
    ok("重置成功", serde_json::Value::Null)
}
