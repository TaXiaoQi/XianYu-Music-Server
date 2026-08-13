use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{compare_version_code, parse_body, str_of};
use crate::response::ReqCtx;

fn announcements_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("announcement.json")
}

fn about_config_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("about_config.json")
}

fn default_about_config() -> serde_json::Value {
    json!({
        "officialSiteUrl": "https://xymusic.cc",
        "officialSiteText": "前往官网",
        "updateEnabled": true,
        "updateText": "检查更新",
        "projectUrl": "https://github.com/TaXiaoQi/XianYu-Music-Desktop",
        "projectText": "开源地址",
        "referenceProjectUrl": "https://github.com/Billy636/XianYuMusic",
        "referenceProjectText": "参考项目",
        "joinGroupUrl": "https://qm.qq.com/q/kvteWSD8yY",
        "joinGroupText": "加入群组"
    })
}

fn read_about_config() -> serde_json::Value {
    let defaults = default_about_config();
    let path = about_config_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return defaults;
    };
    let Ok(serde_json::Value::Object(saved)) = serde_json::from_str::<serde_json::Value>(&content) else {
        return defaults;
    };
    let mut merged = defaults.as_object().cloned().unwrap_or_default();
    for (key, value) in saved {
        merged.insert(key, value);
    }
    serde_json::Value::Object(merged)
}

fn read_announcements() -> Vec<serde_json::Value> {
    let path = announcements_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(arr) = v.as_array() {
                return arr.clone();
            }
        }
    }
    Vec::new()
}

pub async fn get_source_status(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    // 确保音源配置存在
    let _ = sqlx::query("DELETE FROM music_source_config")
        .execute(pool)
        .await;
    let sources = [
        ("酷狗音乐", "kg"),
        ("QQ音乐", "tx"),
        ("酷我音乐", "kw"),
        ("咪咕音乐", "mg"),
        ("网易音乐", "wy"),
    ];
    for (name, code) in sources.iter() {
        let _ = sqlx::query("INSERT IGNORE INTO music_source_config (source_name, source_code, is_enabled) VALUES (?, ?, 1)")
            .bind(name)
            .bind(code)
            .execute(pool)
            .await;
    }

    match sqlx::query("SELECT source_name, source_code, is_enabled FROM music_source_config")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let mut map = serde_json::Map::new();
            let mut kg_enabled = true;
            for row in rows {
                let name: String = row.get("source_name");
                let code: String = row.get("source_code");
                let enabled: i64 = row.get("is_enabled");
                let enabled_bool = enabled == 1;
                map.insert(
                    code.clone(),
                    json!({ "source_name": name, "is_enabled": enabled_bool }),
                );
                if code == "kg" {
                    kg_enabled = enabled_bool;
                }
            }
            ctx.json(
                200,
                "ok",
                Some(json!({
                    "source_name": "kg",
                    "is_enabled": kg_enabled,
                    "sources": map
                })),
            )
        }
        Err(_) => ctx.json(
            200,
            "ok",
            Some(json!({
                "source_name": "kg",
                "is_enabled": true,
                "sources": {
                    "kg": {"source_name": "酷狗音乐", "is_enabled": true},
                    "tx": {"source_name": "QQ音乐", "is_enabled": true},
                    "kw": {"source_name": "酷我音乐", "is_enabled": true},
                    "mg": {"source_name": "咪咕音乐", "is_enabled": true},
                    "wy": {"source_name": "网易音乐", "is_enabled": true}
                }
            })),
        ),
    }
}

pub async fn get_version_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let version = data.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if version.is_empty() {
        return ctx.err(400, "版本号不能为空");
    }

    let current_row = sqlx::query("SELECT * FROM app_versions WHERE version_code = ? ORDER BY id DESC LIMIT 1")
        .bind(&version)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    if let Some(row) = current_row {
        let status: String = row.get("status");
        if status == "disabled" || status == "crash" || status == "group_update" {
            let latest: String = row.get("version_code");
            let content: String = row.get("update_content");
            let url: String = row.get("download_url");
            let size: i64 = row.get("file_size");
            let message: String = row.get("message");
            let msg2: String = if !message.is_empty() { message } else { content.clone() };
            return ctx.json(
                200,
                "ok",
                Some(json!({
                    "status": status,
                    "latest_version": latest,
                    "update_content": content,
                    "download_url": url,
                    "file_size": size,
                    "message": msg2
                })),
            );
        }
    }

    match sqlx::query("SELECT * FROM app_versions WHERE status IN ('update', 'force_update') ORDER BY id DESC")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let mut latest_row: Option<sqlx::mysql::MySqlRow> = None;
            for row in rows {
                let row_version: String = row.get("version_code");
                let row_status: String = row.get("status");
                let cmp = compare_version_code(&row_version, &version);
                if cmp > 0 || (cmp == 0 && row_status == "force_update") {
                    latest_row = Some(row);
                    break;
                }
            }
            if let Some(row) = latest_row {
                let status: String = row.get("status");
                let latest: String = row.get("version_code");
                let content: String = row.get("update_content");
                let url: String = row.get("download_url");
                let size: i64 = row.get("file_size");
                let message: String = row.get("message");
                return ctx.json(
                    200,
                    "ok",
                    Some(json!({
                        "status": status,
                        "latest_version": latest,
                        "update_content": content,
                        "download_url": url,
                        "file_size": size,
                        "message": message
                    })),
                );
            }
            ctx.json(200, "ok", Some(json!({ "status": "normal" })))
        }
        Err(_) => ctx.json(200, "ok", Some(json!({ "status": "normal" }))),
    }
}

pub async fn get_latest_version(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let row = sqlx::query("SELECT * FROM app_versions WHERE status != 'disabled' ORDER BY id DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let id: i64 = r.get("id");
            let app_name: String = r.get("app_name");
            let version: String = r.get("version_code");
            let content: String = r.get("update_content");
            let url: String = r.get("download_url");
            let size: i64 = r.get("file_size");
            let status: String = r.get("status");
            ctx.json(
                200,
                "ok",
                Some(json!({
                    "id": id,
                    "app_name": app_name,
                    "version": version,
                    "content": content,
                    "download_url": url,
                    "file_size": size,
                    "status": status
                })),
            )
        }
        None => ctx.json(200, "ok", Some(json!([]))),
    }
}

fn announcement_version(item: &serde_json::Value) -> String {
    item.get("updated_at")
        .or_else(|| item.get("updatedAt"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

async fn announcement_confirmed(pool: &MySqlPool, ciyuanxi_id: &str, device_id: &str, announcement_id: &str, version: &str) -> bool {
    if ciyuanxi_id.is_empty() && device_id.is_empty() {
        return false;
    }
    sqlx::query(
        "SELECT id FROM user_announcement_confirmations
         WHERE announcement_id = ? AND announcement_updated_at = ?
           AND ((? <> '' AND ciyuanxi_id = ?) OR (? <> '' AND device_id = ?))
         LIMIT 1",
    )
    .bind(announcement_id)
    .bind(version)
    .bind(ciyuanxi_id)
    .bind(ciyuanxi_id)
    .bind(device_id)
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .is_some()
}

pub async fn get_announcement(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let device_id = str_of(&data, "device_id").trim().to_string();
    let mut list: Vec<serde_json::Value> = read_announcements()
        .into_iter()
        .filter(|item| item.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false))
        .collect();
    list.sort_by(|a, b| {
        let ta = a.get("updated_at")
            .or_else(|| a.get("created_at"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tb = b.get("updated_at")
            .or_else(|| b.get("created_at"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        tb.cmp(ta)
    });

    let Some(item) = list.into_iter().next() else {
        return ctx.json::<serde_json::Value>(200, "ok", None);
    };

    let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let updated_at = announcement_version(&item);
    if id.is_empty() || title.is_empty() || content.is_empty() {
        return ctx.json::<serde_json::Value>(200, "ok", None);
    }
    if announcement_confirmed(pool, &ciyuanxi_id, &device_id, &id, &updated_at).await {
        return ctx.json::<serde_json::Value>(200, "ok", None);
    }

    ctx.json(
        200,
        "ok",
        Some(json!({
            "id": id,
            "title": title,
            "content": content,
            "type": item.get("type").and_then(|v| v.as_str()).unwrap_or("info"),
            "date": item.get("date").and_then(|v| v.as_str()).unwrap_or(""),
            "actionUrl": item.get("actionUrl").and_then(|v| v.as_str()).unwrap_or(""),
            "actionText": item.get("actionText").and_then(|v| v.as_str()).unwrap_or(""),
            "updatedAt": updated_at,
        })),
    )
}

pub async fn confirm_announcement(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let announcement_id = str_of(&data, "announcement_id").trim().to_string();
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let device_id = str_of(&data, "device_id").trim().to_string();
    let client_updated_at = str_of(&data, "announcement_updated_at").trim().to_string();

    if announcement_id.is_empty() {
        return ctx.err(400, "公告 ID 不能为空");
    }
    if ciyuanxi_id.is_empty() && device_id.is_empty() {
        return ctx.err(400, "缺少用户或设备标识");
    }

    let list = read_announcements();
    let Some(item) = list
        .iter()
        .find(|item| item.get("id").and_then(|v| v.as_str()).unwrap_or("") == announcement_id)
    else {
        return ctx.err(404, "公告不存在");
    };

    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let updated_at = announcement_version(item);
    if !client_updated_at.is_empty() && client_updated_at != updated_at {
        return ctx.err(409, "公告已更新，请重新阅读");
    }

    let result = sqlx::query(
        "INSERT INTO user_announcement_confirmations
         (ciyuanxi_id, device_id, announcement_id, announcement_title, announcement_updated_at, ip)
         VALUES (?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE confirmed_at = CURRENT_TIMESTAMP, announcement_title = VALUES(announcement_title), ip = VALUES(ip)",
    )
    .bind(&ciyuanxi_id)
    .bind(&device_id)
    .bind(&announcement_id)
    .bind(&title)
    .bind(&updated_at)
    .bind(&ctx.client_ip)
    .execute(pool)
    .await;

    match result {
        Ok(_) => ctx.ok("确认成功", json!({ "announcement_id": announcement_id, "updatedAt": updated_at })),
        Err(e) => ctx.err(500, &format!("记录公告确认失败: {}", e)),
    }
}

pub async fn get_about_config(ctx: ReqCtx) -> Response {
    ctx.json(200, "ok", Some(read_about_config()))
}

pub async fn get_user_agreement(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let (title, content) = crate::admin::agreement::load_user_agreement(pool).await;
    ctx.json(200, "ok", Some(json!({ "title": title, "content": content })))
}

pub async fn get_server_load(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let q = sqlx::query("SELECT COUNT(*) as cnt FROM app_users")
        .fetch_one(pool)
        .await;
    let user_count: i64 = match q {
        Ok(row) => row.get("cnt"),
        Err(_) => 0,
    };
    let (cpu, mem) = loadavg();
    ctx.json(
        200,
        "ok",
        Some(json!({ "cpu": cpu, "memory": mem, "user_count": user_count })),
    )
}

fn loadavg() -> (f64, f64) {
    let cpu = std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .unwrap_or(0.0);
    let mem = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .map(|s| {
            let mut total = 0f64;
            let mut avail = 0f64;
            for line in s.lines().take(10) {
                if line.starts_with("MemTotal:") {
                    total = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0.0);
                }
                if line.starts_with("MemAvailable:") {
                    avail = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0.0);
                }
            }
            if total > 0.0 {
                (total - avail) / total * 100.0
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);
    (cpu, mem)
}

pub async fn get_leaderboard(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let kind = data.get("type").and_then(|v| v.as_str()).unwrap_or("listen").to_string();
    let period = data.get("period").and_then(|v| v.as_str()).unwrap_or("total").to_string();
    let limit = data.get("limit").and_then(|v| v.as_i64()).unwrap_or(50).clamp(1, 100);
    let ciyuanxi_id = data.get("ciyuanxi_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let order_col = if kind == "listen" { "listen_duration" } else { "unique_songs_count" };

    // 根据 period 构建不同的查询
    let (top_sql, count_sql, me_sql, me_count_sql) = match period.as_str() {
        "daily" => {
            let day_filter = "stat_date = CURDATE()";
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
            // 本周一 ~ 今天
            let week_filter = "stat_date >= DATE_SUB(CURDATE(), INTERVAL WEEKDAY(CURDATE()) DAY) AND stat_date <= CURDATE()";
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
            // total
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
    };

    // 查询 Top N 用户
    let rows = match sqlx::query(&top_sql).bind(limit).fetch_all(pool).await {
        Ok(rows) => rows,
        Err(e) => return ctx.err(500, &format!("查询失败: {}", e)),
    };

    let mut leaderboard: Vec<serde_json::Value> = Vec::new();
    let mut me_in_list: Option<serde_json::Value> = None;

    for (i, row) in rows.iter().enumerate() {
        let rank = (i + 1) as u32;
        let uid: String = row.get("ciyuanxi_id");
        let username: String = row.get("nickname");
        let avatar: String = row.get::<Option<String>, _>("avatar_url")
            .unwrap_or_default();
        let value: i64 = row.get("value");
        let is_me = !ciyuanxi_id.is_empty() && uid == ciyuanxi_id;

        let entry = json!({
            "rank": rank,
            "username": username,
            "nickname": username,
            "avatar": avatar,
            "duration": value,
            "is_me": is_me,
        });
        if is_me {
            me_in_list = Some(entry.clone());
        }
        leaderboard.push(entry);
    }

    // 当前用户不在 Top N 时，单独查询排名
    let mut me = me_in_list;
    if !ciyuanxi_id.is_empty() && me.is_none() {
        let user_row = sqlx::query(&me_sql)
            .bind(&ciyuanxi_id)
            .fetch_optional(pool)
            .await;

        if let Ok(Some(row)) = user_row {
            let username: String = row.get("nickname");
            let avatar: String = row.get::<Option<String>, _>("avatar_url")
                .unwrap_or_default();
            let value: i64 = row.get("value");

            if value > 0 {
                let rank_row = sqlx::query(&me_count_sql)
                    .bind(value)
                    .fetch_one(pool)
                    .await;

                let rank = if let Ok(r) = rank_row {
                    r.get::<i64, _>("cnt") as u32 + 1
                } else {
                    0
                };

                me = Some(json!({
                    "rank": rank,
                    "username": username,
                    "nickname": username,
                    "avatar": avatar,
                    "duration": value,
                    "is_me": true,
                }));
            }
        }
    }

    // 统计参与排行的总用户数
    let total_users = sqlx::query(&count_sql)
        .fetch_one(pool)
        .await
        .map(|r| r.get::<i64, _>("cnt") as u32)
        .unwrap_or(leaderboard.len() as u32);

    ctx.json(
        200,
        "ok",
        Some(json!({
            "leaderboard": leaderboard,
            "me": me,
            "total_users": total_users,
            "period": period,
        })),
    )
}
