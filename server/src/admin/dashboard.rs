use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;

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

    let source_distribution_rows = sqlx::query(
        "SELECT COALESCE(NULLIF(source_name, ''), NULLIF(source_type, ''), '未知音源') AS source_name, COUNT(*) AS count \
         FROM source_call_log \
         WHERE DATE(call_time) = ? \
         GROUP BY COALESCE(NULLIF(source_name, ''), NULLIF(source_type, ''), '未知音源') \
         ORDER BY count DESC \
         LIMIT 8",
    )
    .bind(&today)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let source_distribution: Vec<_> = source_distribution_rows
        .iter()
        .map(|row| {
            json!({
                "source_name": row.try_get::<String, _>("source_name").unwrap_or_else(|_| "未知音源".to_string()),
                "count": row.try_get::<i64, _>("count").unwrap_or(0),
            })
        })
        .collect();

    let hot_search_row = sqlx::query(
        "SELECT song_name, COUNT(*) AS count \
         FROM source_call_log \
         WHERE DATE(call_time) = ? AND action = 'search' AND song_name <> '' \
         GROUP BY song_name \
         ORDER BY count DESC, MAX(call_time) DESC \
         LIMIT 1",
    )
    .bind(&today)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let (today_hot_search_keyword, today_hot_search_count) = if let Some(row) = hot_search_row {
        (
            row.try_get::<String, _>("song_name").unwrap_or_default(),
            row.try_get::<i64, _>("count").unwrap_or(0),
        )
    } else {
        (String::new(), 0)
    };

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
        "active_users": safe_count(pool, "SELECT COUNT(DISTINCT user_id) FROM play_history WHERE user_id > 0").await,
        "source_distribution": source_distribution,
        "today_hot_search_keyword": today_hot_search_keyword,
        "today_hot_search_count": today_hot_search_count,
        "pending_wallpapers": safe_count(pool, "SELECT COUNT(*) FROM wallpapers WHERE status = 'pending'").await,
        "pending_avatars": safe_count(pool, "SELECT COUNT(*) FROM user_avatar_pending WHERE status = 'pending'").await,
        "pending_nicknames": safe_count(pool, "SELECT COUNT(*) FROM user_nickname_pending WHERE status = 'pending'").await,
        "pending_feedback": safe_count(pool, "SELECT COUNT(*) FROM user_feedback WHERE status = 'pending'").await,
    });

    ok("ok", stats)
}
