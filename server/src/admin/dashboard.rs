use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;

use super::{ok, AdminCtx};

/// 安全计数：查询失败返回 0
async fn safe_count(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

/// 后台仪表盘统计数据
pub async fn dashboard_stats(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let stats = json!({
        "total_users": safe_count(pool, "SELECT COUNT(*) FROM app_users").await,
        "today_users": safe_count(pool, &format!("SELECT COUNT(*) FROM app_users WHERE DATE(created_at) = '{}'", today)).await,
        "yesterday_users": safe_count(pool, &format!("SELECT COUNT(*) FROM app_users WHERE DATE(created_at) = '{}'", yesterday)).await,
        "total_admins": safe_count(pool, "SELECT COUNT(*) FROM admin_users").await,
        "total_source_calls": safe_count(pool, "SELECT COUNT(*) FROM source_call_log").await,
        "today_source_calls": safe_count(pool, &format!("SELECT COUNT(*) FROM source_call_log WHERE DATE(call_time) = '{}'", today)).await,
        "yesterday_source_calls": safe_count(pool, &format!("SELECT COUNT(*) FROM source_call_log WHERE DATE(call_time) = '{}'", yesterday)).await,
        "today_source_success": safe_count(pool, &format!("SELECT COUNT(*) FROM source_call_log WHERE DATE(call_time) = '{}' AND status = 1", today)).await,
        "total_source_success": safe_count(pool, "SELECT COUNT(*) FROM source_call_log WHERE status = 1").await,
        "total_errors": safe_count(pool, "SELECT COUNT(*) FROM error_log").await,
        "today_errors": safe_count(pool, &format!("SELECT COUNT(*) FROM error_log WHERE DATE(error_time) = '{}'", today)).await,
        "yesterday_errors": safe_count(pool, &format!("SELECT COUNT(*) FROM error_log WHERE DATE(error_time) = '{}'", yesterday)).await,
        "total_shares": safe_count(pool, "SELECT COUNT(*) FROM share_log").await,
        "today_shares": safe_count(pool, &format!("SELECT COUNT(*) FROM share_log WHERE DATE(created_at) = '{}'", today)).await,
        "yesterday_shares": safe_count(pool, &format!("SELECT COUNT(*) FROM share_log WHERE DATE(created_at) = '{}'", yesterday)).await,
        "total_logins": safe_count(pool, "SELECT COUNT(*) FROM login_log").await,
        "today_logins": safe_count(pool, &format!("SELECT COUNT(*) FROM login_log WHERE DATE(login_time) = '{}'", today)).await,
    });

    ok("ok", stats)
}
