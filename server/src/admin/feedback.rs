use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, row_to_value, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

const DEFAULT_FEEDBACK_DAILY_LIMIT: i64 = 20;

async fn read_feedback_daily_limit(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT setting_value FROM server_settings WHERE setting_key = 'feedback_daily_limit' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .and_then(|v| v.trim().parse::<i64>().ok())
    .filter(|v| *v >= 0)
    .unwrap_or(DEFAULT_FEEDBACK_DAILY_LIMIT)
}

/// 反馈列表 + 统计
pub async fn list_feedback(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let status_filter = str_of(&data, "status_filter").trim().to_string();

    let (where_clause, binds): (String, Vec<String>) = if status_filter.is_empty() || status_filter == "all" {
        (String::new(), Vec::new())
    } else {
        ("WHERE status = ?".to_string(), vec![status_filter.clone()])
    };

    // 查询列表：不直接返回 LONGTEXT 日志正文，避免列表页过大
    let list_sql = format!(
        "SELECT id, ciyuanxi_id, nickname, title, content, status, admin_reply, replied_at, replied_by, ip, created_at, updated_at,
                log_meta,
                COALESCE(CHAR_LENGTH(error_logs), 0) AS error_logs_chars,
                COALESCE(CHAR_LENGTH(all_logs), 0) AS all_logs_chars,
                CASE WHEN error_logs IS NULL OR error_logs = '' THEN 0 ELSE 1 END AS has_error_logs,
                CASE WHEN all_logs IS NULL OR all_logs = '' THEN 0 ELSE 1 END AS has_all_logs
         FROM user_feedback {} ORDER BY created_at DESC",
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

/// 反馈详情（包含日志正文）
pub async fn get_feedback_detail(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let row = sqlx::query("SELECT * FROM user_feedback WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(pool)
        .await;
    match row {
        Ok(Some(r)) => ok("ok", row_to_value(&r)),
        Ok(None) => err(404, "反馈不存在"),
        Err(_) => err(500, "数据库错误"),
    }
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

/// 获取反馈每日提交上限
pub async fn get_feedback_limit(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let limit = read_feedback_daily_limit(pool).await;
    ok("ok", json!({ "feedback_daily_limit": limit }))
}

/// 更新反馈每日提交上限
pub async fn update_feedback_limit(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let limit = int_of(&data, "feedback_daily_limit");
    if limit < 0 || limit > 10000 {
        return err(400, "每日上限需在 0 到 10000 之间");
    }
    let limit_text = limit.to_string();
    let result = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description)
         VALUES ('feedback_daily_limit', ?, '每个用户每天可提交的问题反馈数量上限，0 表示不限制')
         ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), description = VALUES(description)",
    )
    .bind(&limit_text)
    .execute(pool)
    .await;
    match result {
        Ok(_) => {
            log_operation(
                pool,
                ctx,
                "更新反馈提交上限",
                "feedback_daily_limit",
                &format!("limit={}", limit),
            )
            .await;
            ok("保存成功", json!({ "feedback_daily_limit": limit }))
        }
        Err(_) => err(500, "服务器错误"),
    }
}
