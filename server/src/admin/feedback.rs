use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, row_to_value, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

/// 反馈列表 + 统计
pub async fn list_feedback(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let status_filter = str_of(&data, "status_filter").trim().to_string();

    let (where_clause, binds): (String, Vec<String>) = if status_filter.is_empty() || status_filter == "all" {
        (String::new(), Vec::new())
    } else {
        ("WHERE status = ?".to_string(), vec![status_filter.clone()])
    };

    // 查询列表
    let list_sql = format!(
        "SELECT * FROM user_feedback {} ORDER BY created_at DESC",
        where_clause
    );
    let mut list_query = sqlx::query(&list_sql);
    for b in &binds {
        list_query = list_query.bind(b);
    }
    let list: Vec<Value> = match list_query.fetch_all(pool).await {
        Ok(rows) => rows.iter().map(row_to_value).collect(),
        Err(_) => return err(500, "数据库错误"),
    };

    // 统计各状态数量
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback")
        .fetch_one(pool).await.unwrap_or(0);
    let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback WHERE status = 'pending'")
        .fetch_one(pool).await.unwrap_or(0);
    let processing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback WHERE status = 'processing'")
        .fetch_one(pool).await.unwrap_or(0);
    let resolved: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback WHERE status = 'resolved'")
        .fetch_one(pool).await.unwrap_or(0);
    let rejected: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback WHERE status = 'rejected'")
        .fetch_one(pool).await.unwrap_or(0);

    ok("ok", json!({
        "list": list,
        "stats": {
            "total": total,
            "pending": pending,
            "processing": processing,
            "resolved": resolved,
            "rejected": rejected,
        }
    }))
}

/// 回复反馈
pub async fn reply_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let reply = str_of(&data, "reply").trim().to_string();
    if id <= 0 || reply.is_empty() {
        return err(400, "参数错误");
    }
    let admin_name = &ctx.username;
    let upd = sqlx::query(
        "UPDATE user_feedback SET admin_reply = ?, replied_by = ?, replied_at = NOW(), status = 'processing', updated_at = NOW() WHERE id = ?",
    )
    .bind(&reply)
    .bind(admin_name)
    .bind(id)
    .execute(pool)
    .await;
    match upd {
        Ok(_) => {
            log_operation(pool, ctx, "回复反馈", &format!("id={}", id), &reply).await;
            ok("回复成功", serde_json::Value::Null)
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 更新反馈状态
pub async fn update_feedback_status(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let status = str_of(&data, "status").trim().to_string();
    let valid = ["pending", "processing", "resolved", "rejected"];
    if id <= 0 || !valid.contains(&status.as_str()) {
        return err(400, "参数错误");
    }
    let upd = sqlx::query("UPDATE user_feedback SET status = ?, updated_at = NOW() WHERE id = ?")
        .bind(&status)
        .bind(id)
        .execute(pool)
        .await;
    match upd {
        Ok(_) => {
            log_operation(pool, ctx, "更新反馈状态", &format!("id={}", id), &format!("status={}", status)).await;
            ok("状态已更新", serde_json::Value::Null)
        }
        Err(_) => err(500, "服务器错误"),
    }
}