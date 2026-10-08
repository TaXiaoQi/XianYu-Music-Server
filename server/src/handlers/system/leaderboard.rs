use super::*;

struct LeaderboardCacheEntry {
    entries: Vec<Value>,
    total_users: u32,
    at: Instant,
    ttl: Duration,
}

static LEADERBOARD_CACHE: OnceLock<Mutex<HashMap<String, LeaderboardCacheEntry>>> = OnceLock::new();
const LEADERBOARD_CACHE_MAX_ENTRIES: usize = 64;

// TTL 按周期分级：daily 榜变化快用短缓存，weekly/total 变化慢用长缓存
fn leaderboard_cache_ttl(period: &str) -> Duration {
    match period {
        "daily" => Duration::from_secs(60),
        "weekly" => Duration::from_secs(300),
        _ => Duration::from_secs(600),
    }
}

fn leaderboard_cache_get(key: &str) -> Option<(Vec<Value>, u32)> {
    let cache = LEADERBOARD_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().ok()?;
    match guard.get(key) {
        Some(e) if e.at.elapsed() <= e.ttl => Some((e.entries.clone(), e.total_users)),
        Some(_) => {
            guard.remove(key);
            None
        }
        None => None,
    }
}

fn leaderboard_cache_put(key: String, entries: Vec<Value>, total_users: u32, ttl: Duration) {
    let cache = LEADERBOARD_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = cache.lock() {
        if guard.len() >= LEADERBOARD_CACHE_MAX_ENTRIES {
            guard.clear();
        }
        guard.insert(key, LeaderboardCacheEntry { entries, total_users, at: Instant::now(), ttl });
    }
}

fn build_leaderboard_sql(kind: &str, period: &str) -> (String, String, String, String) {
    let order_col = if kind == "listen" { "listen_duration" } else { "unique_songs_count" };

    match period {
        "daily" => {
            let day_filter = "stat_date = DATE(NOW() + INTERVAL 8 HOUR)";
            (
                format!(
                    "SELECT d.ciyuanxi_id, u.nickname, u.avatar_url, CAST(SUM(d.{}) AS SIGNED) AS value \
                     FROM listen_daily_stats d \
                     INNER JOIN app_users u ON u.ciyuanxi_id = d.ciyuanxi_id AND u.status = 1 \
                     WHERE {} \
                     GROUP BY d.ciyuanxi_id, u.nickname, u.avatar_url \
                     HAVING value > 0 \
                     ORDER BY value DESC, u.nickname ASC LIMIT ?",
                    order_col, day_filter
                ),
                format!(
                    "SELECT COUNT(*) AS cnt FROM ( \
                     SELECT d.ciyuanxi_id \
                     FROM listen_daily_stats d \
                     INNER JOIN app_users u ON u.ciyuanxi_id = d.ciyuanxi_id AND u.status = 1 \
                     WHERE {} \
                     GROUP BY d.ciyuanxi_id \
                     HAVING CAST(SUM(d.{}) AS SIGNED) > 0 \
                     ) AS sub",
                    day_filter, order_col
                ),
                format!(
                    "SELECT u.nickname, u.avatar_url, CAST(COALESCE(SUM(d.{}), 0) AS SIGNED) AS value \
                     FROM app_users u \
                     LEFT JOIN listen_daily_stats d ON d.ciyuanxi_id = u.ciyuanxi_id AND {} \
                     WHERE u.ciyuanxi_id = ? AND u.status = 1 \
                     GROUP BY u.ciyuanxi_id, u.nickname, u.avatar_url",
                    order_col, day_filter
                ),
                format!(
                    "SELECT COUNT(*) AS cnt FROM ( \
                     SELECT d.ciyuanxi_id \
                     FROM listen_daily_stats d \
                     INNER JOIN app_users u ON u.ciyuanxi_id = d.ciyuanxi_id AND u.status = 1 \
                     WHERE {} \
                     GROUP BY d.ciyuanxi_id \
                     HAVING CAST(SUM(d.{}) AS SIGNED) > ? \
                     ) AS sub",
                    day_filter, order_col
                ),
            )
        }
        "weekly" => {
            let week_filter = "stat_date >= DATE_SUB(DATE(NOW() + INTERVAL 8 HOUR), INTERVAL WEEKDAY(DATE(NOW() + INTERVAL 8 HOUR)) DAY) AND stat_date <= DATE(NOW() + INTERVAL 8 HOUR)";
            (
                format!(
                    "SELECT d.ciyuanxi_id, u.nickname, u.avatar_url, CAST(SUM(d.{}) AS SIGNED) AS value \
                     FROM listen_daily_stats d \
                     INNER JOIN app_users u ON u.ciyuanxi_id = d.ciyuanxi_id AND u.status = 1 \
                     WHERE {} \
                     GROUP BY d.ciyuanxi_id, u.nickname, u.avatar_url \
                     HAVING value > 0 \
                     ORDER BY value DESC, u.nickname ASC LIMIT ?",
                    order_col, week_filter
                ),
                format!(
                    "SELECT COUNT(*) AS cnt FROM ( \
                     SELECT d.ciyuanxi_id \
                     FROM listen_daily_stats d \
                     INNER JOIN app_users u ON u.ciyuanxi_id = d.ciyuanxi_id AND u.status = 1 \
                     WHERE {} \
                     GROUP BY d.ciyuanxi_id \
                     HAVING CAST(SUM(d.{}) AS SIGNED) > 0 \
                     ) AS sub",
                    week_filter, order_col
                ),
                format!(
                    "SELECT u.nickname, u.avatar_url, CAST(COALESCE(SUM(d.{}), 0) AS SIGNED) AS value \
                     FROM app_users u \
                     LEFT JOIN listen_daily_stats d ON d.ciyuanxi_id = u.ciyuanxi_id AND {} \
                     WHERE u.ciyuanxi_id = ? AND u.status = 1 \
                     GROUP BY u.ciyuanxi_id, u.nickname, u.avatar_url",
                    order_col, week_filter
                ),
                format!(
                    "SELECT COUNT(*) AS cnt FROM ( \
                     SELECT d.ciyuanxi_id \
                     FROM listen_daily_stats d \
                     INNER JOIN app_users u ON u.ciyuanxi_id = d.ciyuanxi_id AND u.status = 1 \
                     WHERE {} \
                     GROUP BY d.ciyuanxi_id \
                     HAVING CAST(SUM(d.{}) AS SIGNED) > ? \
                     ) AS sub",
                    week_filter, order_col
                ),
            )
        }
        _ => {
            (
                format!(
                    "SELECT ciyuanxi_id, nickname, avatar_url, CAST({} AS SIGNED) AS value \
                     FROM app_users WHERE status = 1 AND CAST({} AS SIGNED) > 0 \
                     ORDER BY {} DESC, nickname ASC LIMIT ?",
                    order_col, order_col, order_col
                ),
                format!(
                    "SELECT COUNT(*) AS cnt FROM app_users WHERE status = 1 AND CAST({} AS SIGNED) > 0",
                    order_col
                ),
                format!(
                    "SELECT nickname, avatar_url, CAST({} AS SIGNED) AS value \
                     FROM app_users WHERE ciyuanxi_id = ? AND status = 1 LIMIT 1",
                    order_col
                ),
                format!(
                    "SELECT COUNT(*) AS cnt FROM app_users WHERE status = 1 AND CAST({} AS SIGNED) > ?",
                    order_col
                ),
            )
        }
    }
}

async fn fetch_leaderboard_period(
    pool: &MySqlPool,
    kind: &str,
    period: &str,
    limit: i64,
    ciyuanxi_id: &str,
    base_url: &str,
    public_base: &str,
) -> Result<Value, String> {
    let cache_key = format!("{}|{}|{}", kind, period, limit);

    let (entries, total_users) = if let Some((e, tu)) = leaderboard_cache_get(&cache_key) {
        (e, tu)
    } else {
        let (top_sql, count_sql, _, _) = build_leaderboard_sql(kind, period);
        let rows = sqlx::query(&top_sql).bind(limit).fetch_all(pool).await
            .map_err(|e| format!("查询失败: {}", e))?;
        let mut entries: Vec<Value> = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            let uid: String = row.get("ciyuanxi_id");
            let username: String = row.get("nickname");
            let avatar: String = row.get::<Option<String>, _>("avatar_url").unwrap_or_default();
            let value: i64 = row.get("value");
            entries.push(json!({
                "rank": (i + 1) as u32,
                "username": username,
                "nickname": username,
                "ciyuanxi_id": uid,
                "avatar": crate::handlers::upload::absolutize_media_url_with(base_url, public_base, &avatar),
                "duration": value,
            }));
        }
        let total_users = sqlx::query(&count_sql).fetch_one(pool).await
            .map(|r| r.get::<i64, _>("cnt") as u32)
            .unwrap_or(entries.len() as u32);
        leaderboard_cache_put(cache_key, entries.clone(), total_users, leaderboard_cache_ttl(period));
        (entries, total_users)
    };

    let mut me: Option<Value> = None;
    let mut leaderboard: Vec<Value> = Vec::with_capacity(entries.len());
    for e in entries {
        let is_me = !ciyuanxi_id.is_empty() && e["ciyuanxi_id"].as_str() == Some(ciyuanxi_id);
        let mut v = e;
        v["is_me"] = json!(is_me);
        if is_me {
            me = Some(v.clone());
        }
        leaderboard.push(v);
    }
    if !ciyuanxi_id.is_empty() && me.is_none() {
        let (_, _, me_sql, me_count_sql) = build_leaderboard_sql(kind, period);
        let user_row = sqlx::query(&me_sql).bind(ciyuanxi_id).fetch_optional(pool).await
            .map_err(|e| format!("查询失败: {}", e))?;
        if let Some(row) = user_row {
            let username: String = row.get("nickname");
            let avatar: String = row.get::<Option<String>, _>("avatar_url").unwrap_or_default();
            let value: i64 = row.get("value");
            if value > 0 {
                let rank_row = sqlx::query(&me_count_sql).bind(value).fetch_one(pool).await;
                let rank = if let Ok(r) = rank_row {
                    r.get::<i64, _>("cnt") as u32 + 1
                } else {
                    0
                };
                me = Some(json!({
                    "rank": rank,
                    "username": username,
                    "nickname": username,
                    "ciyuanxi_id": ciyuanxi_id,
                    "avatar": crate::handlers::upload::absolutize_media_url_with(base_url, public_base, &avatar),
                    "duration": value,
                    "is_me": true,
                }));
            }
        }
    }

    Ok(json!({
        "leaderboard": leaderboard,
        "me": me,
        "total_users": total_users,
        "period": period,
    }))
}

pub async fn get_leaderboard(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let kind = data.get("type").and_then(|v| v.as_str()).unwrap_or("listen").to_string();
    let period = data.get("period").and_then(|v| v.as_str()).unwrap_or("total").to_string();
    let limit = data.get("limit").and_then(|v| v.as_i64()).unwrap_or(50).clamp(1, 100);
    let ciyuanxi_id = data.get("ciyuanxi_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    if period == "all" {
        let mut leaderboards = serde_json::Map::new();
        for p in ["daily", "weekly", "total"] {
            match fetch_leaderboard_period(pool, &kind, p, limit, &ciyuanxi_id, &ctx.base_url, &ctx.config.public_base_url).await {
                Ok(v) => {
                    leaderboards.insert(p.to_string(), v);
                }
                Err(e) => return ctx.err(500, &e),
            }
        }
        return ctx.json(200, "ok", Some(json!({ "leaderboards": leaderboards })));
    }

    match fetch_leaderboard_period(pool, &kind, &period, limit, &ciyuanxi_id, &ctx.base_url, &ctx.config.public_base_url).await {
        Ok(v) => ctx.json(200, "ok", Some(v)),
        Err(e) => ctx.err(500, &e),
    }
}
