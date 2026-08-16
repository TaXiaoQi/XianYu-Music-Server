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
///
/// 性能要点：
/// - 用 datetime 范围查询替代 `DATE(col) = ?`，命中索引，避免对日志表全表扫描
/// - 所有查询通过 `tokio::join!` 并发执行，由串行 22 次往返降为一次并发
pub async fn dashboard_stats(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let now = chrono::Local::now();
    let today = now.format("%Y-%m-%d").to_string();
    let yesterday = (now - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let tomorrow = (now + chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    // 以 datetime 范围替代 DATE() 函数谓词，保证索引可用
    let today_start = format!("{} 00:00:00", today);
    let tomorrow_start = format!("{} 00:00:00", tomorrow);
    let yesterday_start = format!("{} 00:00:00", yesterday);

    // 预构造 SQL 字符串，避免并发宏中借用临时值
    let sql_total_users = "SELECT COUNT(*) FROM app_users".to_string();
    let sql_today_users = format!(
        "SELECT COUNT(*) FROM app_users WHERE created_at >= '{}' AND created_at < '{}'",
        today_start, tomorrow_start
    );
    let sql_yesterday_users = format!(
        "SELECT COUNT(*) FROM app_users WHERE created_at >= '{}' AND created_at < '{}'",
        yesterday_start, today_start
    );
    let sql_total_admins = "SELECT COUNT(*) FROM admin_users".to_string();
    let sql_total_source_calls = "SELECT COUNT(*) FROM source_call_log".to_string();
    let sql_today_source_calls = format!(
        "SELECT COUNT(*) FROM source_call_log WHERE call_time >= '{}' AND call_time < '{}'",
        today_start, tomorrow_start
    );
    let sql_yesterday_source_calls = format!(
        "SELECT COUNT(*) FROM source_call_log WHERE call_time >= '{}' AND call_time < '{}'",
        yesterday_start, today_start
    );
    let sql_today_source_success = format!(
        "SELECT COUNT(*) FROM source_call_log WHERE call_time >= '{}' AND call_time < '{}' AND status = 1",
        today_start, tomorrow_start
    );
    let sql_total_source_success = "SELECT COUNT(*) FROM source_call_log WHERE status = 1".to_string();
    let sql_total_errors = "SELECT COUNT(*) FROM error_log".to_string();
    let sql_today_errors = format!(
        "SELECT COUNT(*) FROM error_log WHERE error_time >= '{}' AND error_time < '{}'",
        today_start, tomorrow_start
    );
    let sql_yesterday_errors = format!(
        "SELECT COUNT(*) FROM error_log WHERE error_time >= '{}' AND error_time < '{}'",
        yesterday_start, today_start
    );
    let sql_total_shares = "SELECT COUNT(*) FROM share_log".to_string();
    let sql_today_shares = format!(
        "SELECT COUNT(*) FROM share_log WHERE created_at >= '{}' AND created_at < '{}'",
        today_start, tomorrow_start
    );
    let sql_yesterday_shares = format!(
        "SELECT COUNT(*) FROM share_log WHERE created_at >= '{}' AND created_at < '{}'",
        yesterday_start, today_start
    );
    let sql_active_users = format!(
        "SELECT COUNT(DISTINCT device_id) FROM app_open_log WHERE created_at >= '{}' AND created_at < '{}' AND device_id != ''",
        today_start, tomorrow_start
    );
    let sql_source_distribution = format!(
        "SELECT COALESCE(NULLIF(source_name, ''), NULLIF(source_type, ''), '未知音源') AS source_name, COUNT(*) AS count \
         FROM source_call_log \
         WHERE call_time >= '{}' AND call_time < '{}' \
         GROUP BY COALESCE(NULLIF(source_name, ''), NULLIF(source_type, ''), '未知音源') \
         ORDER BY count DESC \
         LIMIT 8",
        today_start, tomorrow_start
    );
    let sql_hot_search = format!(
        "SELECT song_name, COUNT(*) AS count \
         FROM source_call_log \
         WHERE call_time >= '{}' AND call_time < '{}' AND action = 'search' AND song_name <> '' \
         GROUP BY song_name \
         ORDER BY count DESC, MAX(call_time) DESC \
         LIMIT 1",
        today_start, tomorrow_start
    );
    let sql_pending_wallpapers = "SELECT COUNT(*) FROM wallpapers WHERE status = 'pending'".to_string();
    let sql_pending_avatars = "SELECT COUNT(*) FROM user_avatar_pending WHERE status = 'pending'".to_string();
    let sql_pending_nicknames = "SELECT COUNT(*) FROM user_nickname_pending WHERE status = 'pending'".to_string();
    let sql_pending_feedback =
        "SELECT COUNT(*) FROM user_feedback WHERE status = 'pending' AND deleted_at IS NULL".to_string();

    // 并发执行所有计数查询
    let (
        total_users,
        today_users,
        yesterday_users,
        total_admins,
        total_source_calls,
        today_source_calls,
        yesterday_source_calls,
        today_source_success,
        total_source_success,
        total_errors,
        today_errors,
        yesterday_errors,
        total_shares,
        today_shares,
        yesterday_shares,
        active_users,
        pending_wallpapers,
        pending_avatars,
        pending_nicknames,
        pending_feedback,
    ) = tokio::join!(
        safe_count(pool, &sql_total_users),
        safe_count(pool, &sql_today_users),
        safe_count(pool, &sql_yesterday_users),
        safe_count(pool, &sql_total_admins),
        safe_count(pool, &sql_total_source_calls),
        safe_count(pool, &sql_today_source_calls),
        safe_count(pool, &sql_yesterday_source_calls),
        safe_count(pool, &sql_today_source_success),
        safe_count(pool, &sql_total_source_success),
        safe_count(pool, &sql_total_errors),
        safe_count(pool, &sql_today_errors),
        safe_count(pool, &sql_yesterday_errors),
        safe_count(pool, &sql_total_shares),
        safe_count(pool, &sql_today_shares),
        safe_count(pool, &sql_yesterday_shares),
        safe_count(pool, &sql_active_users),
        safe_count(pool, &sql_pending_wallpapers),
        safe_count(pool, &sql_pending_avatars),
        safe_count(pool, &sql_pending_nicknames),
        safe_count(pool, &sql_pending_feedback),
    );

    // 并发执行两条分组查询
    let (source_rows, hot_row) = tokio::join!(
        sqlx::query(&sql_source_distribution).fetch_all(pool),
        sqlx::query(&sql_hot_search).fetch_optional(pool),
    );

    let source_distribution: Vec<_> = match source_rows {
        Ok(rows) => rows
            .iter()
            .map(|row| {
                json!({
                    "source_name": row.try_get::<String, _>("source_name").unwrap_or_else(|_| "未知音源".to_string()),
                    "count": row.try_get::<i64, _>("count").unwrap_or(0),
                })
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let (today_hot_search_keyword, today_hot_search_count) = match hot_row {
        Ok(Some(row)) => (
            row.try_get::<String, _>("song_name").unwrap_or_default(),
            row.try_get::<i64, _>("count").unwrap_or(0),
        ),
        _ => (String::new(), 0),
    };

    let stats = json!({
        "total_users": total_users,
        "today_users": today_users,
        "yesterday_users": yesterday_users,
        "total_admins": total_admins,
        "total_source_calls": total_source_calls,
        "today_source_calls": today_source_calls,
        "yesterday_source_calls": yesterday_source_calls,
        "today_source_success": today_source_success,
        "total_source_success": total_source_success,
        "total_errors": total_errors,
        "today_errors": today_errors,
        "yesterday_errors": yesterday_errors,
        "total_shares": total_shares,
        "today_shares": today_shares,
        "yesterday_shares": yesterday_shares,
        "active_users": active_users,
        "source_distribution": source_distribution,
        "today_hot_search_keyword": today_hot_search_keyword,
        "today_hot_search_count": today_hot_search_count,
        "pending_wallpapers": pending_wallpapers,
        "pending_avatars": pending_avatars,
        "pending_nicknames": pending_nicknames,
        "pending_feedback": pending_feedback,
        "api_secret": ctx.config.api_secret.clone(),
    });

    ok("ok", stats)
}