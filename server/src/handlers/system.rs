use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{compare_version_code, parse_body};
use crate::response::ReqCtx;

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
    let limit = data.get("limit").and_then(|v| v.as_i64()).unwrap_or(20).min(100);
    let rows = match kind.as_str() {
        "listen" => sqlx::query(
            "SELECT ciyuanxi_id, username, listen_duration FROM app_users WHERE status = 1 AND listen_duration > 0 ORDER BY listen_duration DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await,
        _ => sqlx::query(
            "SELECT ciyuanxi_id, username, unique_songs_count FROM app_users WHERE status = 1 AND unique_songs_count > 0 ORDER BY unique_songs_count DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await,
    };
    match rows {
        Ok(rows) => {
            let mut list = vec![];
            for row in rows {
                let ciyuanxi_id: String = row.get("ciyuanxi_id");
                let username: String = row.get("username");
                let value: i64 = if kind == "listen" {
                    row.get("listen_duration")
                } else {
                    row.get("unique_songs_count")
                };
                list.push(json!({ "ciyuanxi_id": ciyuanxi_id, "username": username, "value": value }));
            }
            ctx.json(200, "ok", Some(json!({ "type": kind, "list": list })))
        }
        Err(_) => ctx.err(500, "查询失败"),
    }
}
