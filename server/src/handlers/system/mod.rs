mod about;
mod content;
mod leaderboard;
mod version;

pub(crate) use axum::response::Response;
pub(crate) use ed25519_dalek::Signer;
pub(crate) use serde_json::json;
pub(crate) use serde_json::Value;
pub(crate) use sqlx::MySqlPool;
pub(crate) use sqlx::Row;
pub(crate) use std::collections::HashMap;
pub(crate) use std::sync::{Mutex, OnceLock};
pub(crate) use std::time::{Duration, Instant};

pub(crate) use crate::handlers::helpers::{compare_version_code, parse_body, str_of};
pub(crate) use crate::response::ReqCtx;

pub use about::{
    apply_mobile_about_overrides, apply_platform_about_overrides, apply_watch_about_overrides,
    get_about_config,
};
pub use content::{
    confirm_announcement, confirm_privacy_policy, get_announcement, get_deploy_doc,
    get_fallback_modules, get_privacy_policy, get_site_logo, get_user_agreement,
};
pub use leaderboard::get_leaderboard;
pub use version::{check_beta_access, get_latest_version, get_version_status, share_download};

pub async fn get_source_status(ctx: ReqCtx, pool: &MySqlPool) -> Response {
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
