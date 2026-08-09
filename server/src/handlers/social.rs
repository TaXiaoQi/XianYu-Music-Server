use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

const MAX_FEEDBACK_LOG_CHARS: usize = 500_000;
const DEFAULT_FEEDBACK_DAILY_LIMIT: i64 = 20;

fn trim_feedback_log(value: String) -> String {
    if value.chars().count() <= MAX_FEEDBACK_LOG_CHARS {
        return value;
    }
    value.chars().take(MAX_FEEDBACK_LOG_CHARS).collect()
}

async fn get_feedback_daily_limit(pool: &MySqlPool) -> i64 {
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

pub async fn submit_feedback(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let mut nickname = str_of(&data, "nickname").trim().to_string();
    let title = str_of(&data, "title").trim().to_string();
    let content = str_of(&data, "content").trim().to_string();
    let raw_error_logs = str_of(&data, "error_logs");
    let raw_all_logs = str_of(&data, "all_logs");
    let error_logs = trim_feedback_log(raw_error_logs.clone());
    let all_logs = trim_feedback_log(raw_all_logs.clone());
    let log_meta = json!({
        "has_error_logs": !error_logs.is_empty(),
        "has_all_logs": !all_logs.is_empty(),
        "error_logs_chars": error_logs.chars().count(),
        "all_logs_chars": all_logs.chars().count(),
        "error_logs_truncated": raw_error_logs.chars().count() > error_logs.chars().count(),
        "all_logs_truncated": raw_all_logs.chars().count() > all_logs.chars().count(),
    })
    .to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "请先登录");
    }
    if title.is_empty() {
        return ctx.err(400, "标题不能为空");
    }
    if content.is_empty() {
        return ctx.err(400, "内容不能为空");
    }
    if title.chars().count() > 60 {
        return ctx.err(400, "标题不能超过 60 字");
    }
    if content.chars().count() > 1000 {
        return ctx.err(400, "内容不能超过 1000 字");
    }
    let daily_limit = get_feedback_daily_limit(pool).await;
    if daily_limit > 0 {
        let submitted_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_feedback WHERE ciyuanxi_id = ? AND created_at >= CURDATE() AND created_at < CURDATE() + INTERVAL 1 DAY",
        )
        .bind(&ciyuanxi_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        if submitted_today >= daily_limit {
            return ctx.err(
                429,
                &format!("今日反馈提交次数已达上限（{} 条），请明天再试", daily_limit),
            );
        }
    }
    if nickname.is_empty() {
        let row = sqlx::query("SELECT username FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
            .bind(&ciyuanxi_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        if let Some(r) = row {
            nickname = r.get("username");
        }
    }
    let ip = ctx.client_ip.clone();
    let result = sqlx::query("INSERT INTO user_feedback (ciyuanxi_id, nickname, title, content, error_logs, all_logs, log_meta, ip) VALUES (?,?,?,?,?,?,?,?)")
        .bind(&ciyuanxi_id)
        .bind(&nickname)
        .bind(&title)
        .bind(&content)
        .bind(&error_logs)
        .bind(&all_logs)
        .bind(&log_meta)
        .bind(&ip)
        .execute(pool)
        .await;
    match result {
        Ok(r) => ctx.json(200, "提交成功", Some(json!({ "id": r.last_insert_id() }))),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn check_ciyuanxi_id(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let mut identifier = str_of(&data, "ciyuanxi_id");
    if identifier.is_empty() {
        identifier = str_of(&data, "user_id");
    }
    if identifier.is_empty() {
        return ctx.err(400, "用户标识不能为空");
    }
    let row = sqlx::query("SELECT id, ciyuanxi_id FROM app_users WHERE id = ? OR ciyuanxi_id = ? LIMIT 1")
        .bind(&identifier)
        .bind(&identifier)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let user_id: i64 = r.get("id");
            let ciyuanxi_id: String = r.get("ciyuanxi_id");
            ctx.json(200, "ok", Some(json!({ "user_id": user_id, "ciyuanxi_id": ciyuanxi_id })))
        }
        None => ctx.err(404, "用户不存在"),
    }
}
