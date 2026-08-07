use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

/// 修改用户弦予号（普通号码，不进入靓号表）
pub async fn change_ciyuanxi_id(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let user_id = int_of(&data, "user_id");
    let new_id = str_of(&data, "new_ciyuanxi_id").trim().to_string();
    if user_id <= 0 {
        return err(400, "用户参数错误");
    }
    if new_id.is_empty() || !new_id.chars().all(|c| c.is_ascii_digit()) {
        return err(400, "弦予号必须为纯数字");
    }
    let check = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? AND id != ? LIMIT 1")
        .bind(&new_id).bind(user_id).fetch_optional(pool).await.ok().flatten();
    if check.is_some() {
        return err(400, "该弦予号已被其他用户使用");
    }
    let pretty = sqlx::query("SELECT id, assigned_user_id FROM ciyuanxi_pretty_ids WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&new_id).fetch_optional(pool).await.ok().flatten();
    if let Some(p) = pretty {
        let assigned: Option<i64> = p.try_get("assigned_user_id").ok();
        if assigned.unwrap_or(0) != user_id {
            return err(400, "该号码为靓号，请通过靓号管理分配");
        }
    }
    let u = sqlx::query("SELECT ciyuanxi_id FROM app_users WHERE id = ? LIMIT 1").bind(user_id).fetch_optional(pool).await.ok().flatten();
    let Some(u) = u else {
        return err(404, "用户不存在");
    };
    let old_id: String = u.try_get("ciyuanxi_id").unwrap_or_default();
    if !old_id.is_empty() {
        let _ = sqlx::query("UPDATE ciyuanxi_pretty_ids SET assigned_user_id = 0, assigned_at = NULL WHERE ciyuanxi_id = ? AND assigned_user_id = ?")
            .bind(&old_id).bind(&user_id.to_string()).execute(pool).await;
    }
    let _ = sqlx::query("UPDATE app_users SET ciyuanxi_id = ? WHERE id = ?").bind(&new_id).bind(user_id).execute(pool).await;
    log_operation(pool, ctx, "修改弦予号", &format!("user_id={} {} -> {}", user_id, old_id, new_id), "").await;
    ok("修改成功", json!({ "old_id": old_id, "new_id": new_id }))
}