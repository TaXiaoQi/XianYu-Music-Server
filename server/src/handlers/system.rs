use axum::response::Response;
use serde_json::json;
use serde_json::Value;
use sqlx::MySqlPool;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::handlers::helpers::{compare_version_code, parse_body, str_of};
use crate::response::ReqCtx;

fn announcements_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("announcement.json")
}

fn about_config_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("about_config.json")
}

/// 平台专属关于页配置文件：desktop / mobile 配置结构独立，互不覆盖。
fn platform_about_config_path(platform: &str) -> Option<std::path::PathBuf> {
    match platform {
        "desktop" => Some(std::path::Path::new("api").join("about_config_desktop.json")),
        "mobile" => Some(std::path::Path::new("api").join("about_config_mobile.json")),
        _ => None,
    }
}

fn default_about_config() -> serde_json::Value {
    json!({
        "officialSiteUrl": "https://xianyumusic.cn",
        "updateEnabled": true,
        "projectUrl": "https://github.com/TaXiaoQi/XianYu-Music-Desktop",
        "referenceProjectUrl": "https://github.com/Billy636/XianYuMusic",
        "joinGroupUrl": "https://qm.qq.com/q/kvteWSD8yY",
        "acknowledgements": [
            { "name": "@Billy636", "url": "https://github.com/Billy636" },
            { "name": "@Zencok", "url": "https://github.com/Zencok" },
            { "name": "@kiomosu", "url": "https://github.com/kiomosu" }
        ]
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

/// 读取指定平台指定渠道（stable/beta）的最新已启用版本配置。
/// 旧数据（无 platform/channel 字段）视为正式版配置，保证分平台/分渠道上线前的数据继续生效。
fn latest_enabled_platform_version(platform: &str, channel: &str) -> Option<serde_json::Value> {
    let path = std::path::Path::new("api").join("version.json");
    let content = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let arr = value.as_array()?;
    let platform = match platform {
        "mobile" => "mobile",
        "watch" => "watch",
        _ => "desktop",
    };
    let mut best: Option<serde_json::Value> = None;
    for item in arr {
        let item_platform = item.get("platform").and_then(|v| v.as_str()).unwrap_or("desktop");
        if item_platform != platform {
            continue;
        }
        // 渠道过滤：未标注 channel 的历史条目一律视为正式版
        let item_channel = item.get("channel").and_then(|v| v.as_str()).unwrap_or("stable");
        if item_channel != channel {
            continue;
        }
        let enabled = item.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
        if !enabled {
            continue;
        }
        let item_ver = item.get("version").and_then(|v| v.as_str()).unwrap_or("");
        match &best {
            Some(cur) => {
                let cur_ver = cur.get("version").and_then(|v| v.as_str()).unwrap_or("");
                if compare_version_code(item_ver, cur_ver) > 0 {
                    best = Some(item.clone());
                }
            }
            None => best = Some(item.clone()),
        }
    }
    best
}

/// 设备是否在内测名单中（beta_testers 表）。
async fn beta_device_allowed(pool: &MySqlPool, device_id: &str) -> bool {
    if device_id.is_empty() {
        return false;
    }
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM beta_testers WHERE device_id = ?")
        .bind(device_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
        > 0
}

/// 内测资格检查（客户端开屏门槛）：
/// 客户端自行判定本地版本号含 beta 预发布段即为内测构建，携带 device_id 上报，
/// 服务端只查 beta_testers 名单返回 allowed；不在名单 → 客户端拦截并弹内测申请窗。
/// 不在名单且该设备存在待审核的内测申请时附 `pending: true`，
/// 客户端改为弹「审核中」窗（仅退出软件，不再提供申请入口）。
pub async fn check_beta_access(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let raw = parse_body(body);
    let device_id = str_of(&raw, "device_id").trim().to_string();
    let allowed = beta_device_allowed(pool, &device_id).await;
    let mut pending = false;
    if !allowed && !device_id.is_empty() {
        pending = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_feedback WHERE device_id = ? AND feedback_type = 'beta' AND status IN ('pending', 'processing')",
        )
        .bind(&device_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
            > 0;
    }
    ctx.json(200, "ok", Some(json!({ "allowed": allowed, "pending": pending })))
}

pub async fn get_latest_version(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let raw = parse_body(body);
    let platform = str_of(&raw, "platform").trim().to_string();
    let device_id = str_of(&raw, "device_id").trim().to_string();

    // 渠道细分：普通设备只拉正式版；内测名单中的设备额外参与测试版比价，
    // 测试版版本号更高时才下发（正式版发布后高于测试版则自然回正）。
    let mut selected: Option<serde_json::Value> =
        latest_enabled_platform_version(&platform, "stable");
    if beta_device_allowed(pool, &device_id).await {
        if let Some(beta) = latest_enabled_platform_version(&platform, "beta") {
            let beta_newer = match &selected {
                Some(stable) => {
                    let bv = beta.get("version").and_then(|v| v.as_str()).unwrap_or("");
                    let sv = stable.get("version").and_then(|v| v.as_str()).unwrap_or("");
                    compare_version_code(bv, sv) > 0
                }
                None => true,
            };
            if beta_newer {
                selected = Some(beta);
            }
        }
    }

    if let Some(item) = selected {
        let is_beta = item.get("channel").and_then(|v| v.as_str()) == Some("beta");
        let version = item.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let content = item.get("updateContent").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let url = item.get("downloadUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let updated_at = item.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let app_name = match platform.as_str() {
            "mobile" => "弦予音乐移动端",
            "watch" => "弦予音乐腕上端",
            _ => "弦予音乐桌面端",
        };
        return ctx.json(
            200,
            "ok",
            Some(json!({
                "id": 0,
                "app_name": app_name,
                "version": version,
                "content": content,
                "download_url": url,
                "file_size": 0,
                "status": "normal",
                "channel": if is_beta { "beta" } else { "stable" },
                "updated_at": updated_at
            })),
        );
    }
    // 兜底：version.json 无已启用版本时，回退到历史 app_versions 数据
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
        // 无任何已发布版本：data 返回 null（不要返回数组，客户端按对象解析）
        None => ctx.json::<serde_json::Value>(200, "ok", None),
    }
}

/// 分享落地页「去下载」用的免签接口：返回指定平台服务器发布的最新版本下载信息。
/// 与官网对齐——下载来源统一取服务器发布的版本（version.json → app_versions 兜底）。
pub async fn share_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let raw = parse_body(body);
    let req_platform = str_of(&raw, "platform");
    let platform = if req_platform.is_empty() {
        "mobile".to_string()
    } else {
        req_platform
    };
    let app_name = match platform.as_str() {
        "mobile" => "弦予音乐移动端",
        "watch" => "弦予音乐腕上端",
        _ => "弦予音乐桌面端",
    };

    // 优先：分平台 version.json 中已启用且版本号最大的正式版（下载页/分享页不对内测渠道开放）
    if let Some(item) = latest_enabled_platform_version(&platform, "stable") {
        let version = item.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let url = item.get("downloadUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let content = item.get("updateContent").and_then(|v| v.as_str()).unwrap_or("").to_string();
        // 商店页链接（可选）：桌面端配置微软商店后，官网下载页展示「从微软商店获取」次要入口
        let store_url = item
            .get("storeUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        return ctx.ok(
            "ok",
            json!({
                "platform": platform,
                "app_name": app_name,
                "version": version,
                "content": content,
                "download_url": url,
                "store_url": store_url,
            }),
        );
    }

    // 兜底：历史 app_versions 中最新的非禁用版本
    let row = sqlx::query("SELECT * FROM app_versions WHERE status != 'disabled' ORDER BY id DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let version: String = r.get("version_code");
            let url: String = r.get("download_url");
            ctx.ok(
                "ok",
                json!({
                    "platform": platform,
                    "app_name": app_name,
                    "version": version,
                    "content": "",
                    "download_url": url,
                    "store_url": "",
                }),
            )
        }
        None => ctx.err(404, "暂无可用下载"),
    }
}

/// 下发兜底模块：桌面端启动及每 30 分钟拉取一次，
/// 返回已启用模块的 {moduleKey, name, version, digest, code, updatedAt} 列表
pub async fn get_fallback_modules(ctx: ReqCtx) -> Response {
    let modules = crate::admin::fallback::enabled_modules_payload();
    ctx.json(200, "ok", Some(json!({ "modules": modules })))
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
    // 根据平台单独推送，如版本/壁纸：请求方传入 platform，只取该平台可用的公告。
    // 旧数据（无 platform 字段）视为桌面端配置，保证分平台上线前的公告继续生效。
    let platform = {
        let p = str_of(&data, "platform").trim().to_string();
        if p.is_empty() { "desktop".to_string() } else { p }
    };
    let mut list: Vec<serde_json::Value> = read_announcements()
        .into_iter()
        .filter(|item| item.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false))
        .filter(|item| {
            let p = item.get("platform").and_then(|v| v.as_str()).unwrap_or("desktop");
            p == platform
        })
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

/// 移动端专属链接覆盖：开源地址指向移动端仓库，
/// 参考项目改为桌面端仓库（即配置中的 projectUrl）。
pub fn apply_mobile_about_overrides(config: &mut Value) {
    let Some(obj) = config.as_object_mut() else { return };
    let desktop_url = obj
        .get("projectUrl")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("https://github.com/TaXiaoQi/XianYu-Music-Desktop")
        .to_string();
    obj.insert("projectUrl".into(), json!("https://github.com/TaXiaoQi/XianYu-Music-Mobile"));
    obj.insert("referenceProjectUrl".into(), json!(desktop_url));
}

pub fn apply_platform_about_overrides(config: &mut Value, body: &str) {
    if str_of(&parse_body(body), "platform") != "mobile" {
        return;
    }
    apply_mobile_about_overrides(config);
}

/// 读取平台专属关于页配置；文件不存在返回 None（调用方回退默认配置）。
/// 移动端存档以移动端默认值（开源=移动端仓库、参考=桌面端仓库）为基底合并，
/// 缺省字段不会被桌面端默认值污染。
fn read_platform_about_config(platform: &str) -> Option<serde_json::Value> {
    let path = platform_about_config_path(platform)?;
    let content = std::fs::read_to_string(path).ok()?;
    let saved = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    let obj = saved.as_object()?.clone();
    let mut base = default_about_config();
    if platform == "mobile" {
        apply_mobile_about_overrides(&mut base);
    }
    let mut merged = base.as_object().cloned().unwrap_or_default();
    for (key, value) in obj {
        merged.insert(key, value);
    }
    Some(serde_json::Value::Object(merged))
}

pub async fn get_about_config(body: &str, ctx: ReqCtx) -> Response {
    let platform = str_of(&parse_body(body), "platform").trim().to_string();
    // 平台专属配置优先；无平台配置时回退默认配置（移动端叠加链接覆盖，保持旧行为）
    if let Some(config) = read_platform_about_config(&platform) {
        return ctx.json(200, "ok", Some(config));
    }
    let mut config = read_about_config();
    apply_platform_about_overrides(&mut config, body);
    ctx.json(200, "ok", Some(config))
}

/// 获取站点 Logo（公开接口，供后台登录页等无需登录场景使用）
pub async fn get_site_logo(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let url = crate::admin::site_config::read_site_logo(pool).await;
    ctx.json(200, "ok", Some(json!({ "logo_url": url })))
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

/// 排行榜公开部分缓存（Top N + 总人数）：对所有用户一致，30s 内复用
struct LeaderboardCacheEntry {
    entries: Vec<Value>,
    total_users: u32,
    at: Instant,
}

static LEADERBOARD_CACHE: OnceLock<Mutex<HashMap<String, LeaderboardCacheEntry>>> = OnceLock::new();
const LEADERBOARD_CACHE_TTL: Duration = Duration::from_secs(30);
const LEADERBOARD_CACHE_MAX_ENTRIES: usize = 64;

fn leaderboard_cache_get(key: &str) -> Option<(Vec<Value>, u32)> {
    let cache = LEADERBOARD_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().ok()?;
    match guard.get(key) {
        Some(e) if e.at.elapsed() <= LEADERBOARD_CACHE_TTL => Some((e.entries.clone(), e.total_users)),
        Some(_) => {
            guard.remove(key);
            None
        }
        None => None,
    }
}

fn leaderboard_cache_put(key: String, entries: Vec<Value>, total_users: u32) {
    let cache = LEADERBOARD_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = cache.lock() {
        if guard.len() >= LEADERBOARD_CACHE_MAX_ENTRIES {
            guard.clear();
        }
        guard.insert(key, LeaderboardCacheEntry { entries, total_users, at: Instant::now() });
    }
}

/// 构建单个周期的 4 条 SQL（top/count/me/me_count）
fn build_leaderboard_sql(kind: &str, period: &str) -> (String, String, String, String) {
    let order_col = if kind == "listen" { "listen_duration" } else { "unique_songs_count" };

    // 连接统一 UTC，日期计算统一按北京时间(UTC+8)切日，保证日榜/周榜在本地 0 点重置
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
            // 本周一 ~ 今天（按北京时间切日）
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
    }
}

/// 获取单个周期的排行榜（公开部分走缓存，个人排名实时查询）
async fn fetch_leaderboard_period(
    pool: &MySqlPool,
    kind: &str,
    period: &str,
    limit: i64,
    ciyuanxi_id: &str,
) -> Result<Value, String> {
    let cache_key = format!("{}|{}|{}", kind, period, limit);

    // 公开部分（Top N + 总人数）走缓存，对所有用户一致
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
                "avatar": avatar,
                "duration": value,
            }));
        }
        let total_users = sqlx::query(&count_sql).fetch_one(pool).await
            .map(|r| r.get::<i64, _>("cnt") as u32)
            .unwrap_or(entries.len() as u32);
        leaderboard_cache_put(cache_key, entries.clone(), total_users);
        (entries, total_users)
    };

    // 个人排名（me）不缓存，按用户实时查询
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
                    "avatar": avatar,
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

    // period=all：一次请求返回日/周/总三榜，减少客户端多次往返
    if period == "all" {
        let mut leaderboards = serde_json::Map::new();
        for p in ["daily", "weekly", "total"] {
            match fetch_leaderboard_period(pool, &kind, p, limit, &ciyuanxi_id).await {
                Ok(v) => {
                    leaderboards.insert(p.to_string(), v);
                }
                Err(e) => return ctx.err(500, &e),
            }
        }
        return ctx.json(200, "ok", Some(json!({ "leaderboards": leaderboards })));
    }

    match fetch_leaderboard_period(pool, &kind, &period, limit, &ciyuanxi_id).await {
        Ok(v) => ctx.json(200, "ok", Some(v)),
        Err(e) => ctx.err(500, &e),
    }
}
