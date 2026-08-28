use axum::response::Response;
use chrono::Utc;
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

/// data/sync/{digits}/ 目录（与 PHP fileSyncDir 一致）
fn sync_root() -> PathBuf {
    PathBuf::from("data/sync")
}

fn sync_dir(ciyuanxi_id: &str) -> PathBuf {
    let digits: String = ciyuanxi_id.chars().filter(|c| c.is_ascii_digit()).collect();
    sync_root().join(digits)
}

fn chunk_dir(ciyuanxi_id: &str) -> PathBuf {
    sync_dir(ciyuanxi_id).join("chunks")
}

fn now_str() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn now_ts() -> i64 {
    Utc::now().timestamp()
}

pub async fn file_sync_upload_start(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let dir = chunk_dir(&ciyuanxi_id);
    if std::fs::create_dir_all(&dir).is_err() {
        return ctx.err(500, "创建目录失败");
    }
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            if e.path().extension().map(|x| x == "json").unwrap_or(false) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    ctx.ok("ok", json!({ "chunk_dir_ready": true }))
}

pub async fn file_sync_upload_chunk(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let chunk_index = data.get("chunk_index").and_then(|v| v.as_i64()).unwrap_or(0);
    let total_chunks = data.get("total_chunks").and_then(|v| v.as_i64()).unwrap_or(1);
    let chunk_data = data.get("chunk_data").cloned().unwrap_or_else(|| json!([]));
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let dir = chunk_dir(&ciyuanxi_id);
    if std::fs::create_dir_all(&dir).is_err() {
        return ctx.err(500, "创建目录失败");
    }
    let file = dir.join(format!("chunk_{}.json", chunk_index));
    let payload = json!({
        "chunk_index": chunk_index,
        "total_chunks": total_chunks,
        "chunk_data": chunk_data
    });
    match std::fs::write(&file, serde_json::to_string(&payload).unwrap_or_default()) {
        Ok(_) => ctx.ok("ok", json!({ "chunk_index": chunk_index, "total_chunks": total_chunks })),
        Err(_) => ctx.err(500, &format!("分块 {} 写入失败", chunk_index)),
    }
}

pub async fn file_sync_upload_finish(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let dir = sync_dir(&ciyuanxi_id);
    let chunkdir = chunk_dir(&ciyuanxi_id);
    if !chunkdir.is_dir() {
        return ctx.err(400, "没有分块数据");
    }
    let mut all_playlists: Vec<Value> = Vec::new();
    let mut files: Vec<(i64, PathBuf)> = Vec::new();
    let mut err = false;
    if let Ok(rd) = std::fs::read_dir(&chunkdir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(idx) = parse_chunk_index(&name) {
                files.push((idx, e.path()));
            }
        }
    } else {
        err = true;
    }
    if err || files.is_empty() {
        return ctx.err(400, "没有分块文件");
    }
    files.sort_by_key(|(i, _)| *i);
    for (_, path) in &files {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(chunk) = serde_json::from_str::<Value>(&content) {
                if let Some(items) = chunk.get("chunk_data").and_then(|x| x.as_array()) {
                    all_playlists.extend(items.clone());
                }
            }
        }
        let _ = std::fs::remove_file(path);
    }
    let _ = std::fs::remove_dir(&chunkdir);
    // 合并同 ID 歌单（songs 合并）
    let mut map: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for pl in &all_playlists {
        let id = pl.get("id").map(|v| v.as_str().unwrap_or("").to_string()).unwrap_or_default();
        let songs = pl.get("songs").cloned().unwrap_or_else(|| json!([]));
        let new_arr = songs.as_array().cloned().unwrap_or_default();
        let entry = map.entry(id).or_insert_with(|| {
            let mut base = pl.clone();
            base["songs"] = json!([]);
            base
        });
        let mut existing_arr = entry.get("songs").and_then(|s| s.as_array()).cloned().unwrap_or_default();
        existing_arr.extend(new_arr);
        entry["songs"] = json!(existing_arr);
    }
    let merged: Vec<Value> = map.into_values().collect();
    let song_total: i64 = merged
        .iter()
        .map(|pl| pl.get("songs").and_then(|s| s.as_array()).map(|a| a.len() as i64).unwrap_or(0))
        .sum();
    let save = json!({
        "version": 4,
        "uploaded_at": now_str(),
        "timestamp": now_ts(),
        "stats": {
            "playlist_count": merged.len(),
            "song_total": song_total
        },
        "playlists": merged
    });
    let file = dir.join("playlists.json");
    let ok = std::fs::write(&file, serde_json::to_string(&save).unwrap_or_default()).is_ok();
    // 更新元信息
    let meta = json!({
        "last_sync": now_str(),
        "last_sync_timestamp": now_ts(),
        "playlist_count": merged.len(),
        "song_total": song_total
    });
    let _ = std::fs::write(dir.join("meta.json"), serde_json::to_string(&meta).unwrap_or_default());
    if ok {
        ctx.ok("同步成功", json!({
            "playlist_count": merged.len(),
            "song_total": song_total
        }))
    } else {
        ctx.err(500, "写入文件失败")
    }
}

fn parse_chunk_index(name: &str) -> Option<i64> {
    let trimmed = name.strip_prefix("chunk_")?.strip_suffix(".json")?;
    trimmed.parse().ok()
}

pub async fn file_sync_download(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let file = sync_dir(&ciyuanxi_id).join("playlists.json");
    if !file.exists() {
        return ctx.ok("暂无同步数据", json!({ "playlists": [] }));
    }
    match std::fs::read_to_string(&file) {
        Ok(content) => match serde_json::from_str::<Value>(&content) {
            Ok(v) => ctx.ok("获取成功", v),
            Err(_) => ctx.ok("数据读取失败", json!({ "playlists": [] })),
        },
        Err(_) => ctx.ok("数据读取失败", json!({ "playlists": [] })),
    }
}

/// 读取插件/设置类的 JSON 快照文件
fn read_snapshot(ciyuanxi_id: &str, name: &str) -> Result<Value, ()> {
    let file = sync_dir(ciyuanxi_id).join(name);
    if !file.exists() {
        return Err(());
    }
    std::fs::read_to_string(&file)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .ok_or(())
}

fn write_snapshot(ciyuanxi_id: &str, name: &str, data: &Value) -> bool {
    let dir = sync_dir(ciyuanxi_id);
    if std::fs::create_dir_all(&dir).is_err() {
        return false;
    }
    std::fs::write(dir.join(name), serde_json::to_string(data).unwrap_or_default()).is_ok()
}

/// 后台重置听歌时长时写入「清零快照」。
/// reset_at 记录本次清零时间点：客户端通过它做一次性下发（仅当云端更重置时间新于本地
/// 已应用的 reset_at 才清零一次），避免永久屏蔽用户后续重新累计。
/// reason 为管理员填写的清除原因，随快照下发供客户端弹窗展示。
pub fn write_listen_stats_reset(ciyuanxi_id: &str, reason: &str) -> bool {
    let trimmed = reason.trim();
    let save = json!({
        "version": 2,
        "uploaded_at": now_str(),
        "timestamp": now_ts(),
        "merged": true,
        "cleared": true,
        "reset_at": now_ts(),
        "reason": if trimmed.is_empty() { "违规" } else { trimmed },
        "listen_stats": {
            "global": {
                "total_play_count": 0,
                "total_play_time_ms": 0,
                "first_played_at": null,
                "last_played_at": null
            },
            "daily": []
        }
    });
    write_snapshot(ciyuanxi_id, "listen_stats.json", &save)
}

/// 清洗订阅列表：仅保留带有效 url 的对象，其余字段透传（id/name/addedAt 等）。
fn sanitize_subscriptions(raw: &Value) -> Option<Vec<Value>> {
    let arr = raw.as_array()?;
    let list: Vec<Value> = arr
        .iter()
        .filter(|item| {
            item.get("url")
                .and_then(|u| u.as_str())
                .map(|u| !u.trim().is_empty())
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    Some(list)
}

pub async fn plugin_sync_upload_one(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let plugin = data.get("plugin").cloned().unwrap_or(Value::Null);
    if !plugin.is_object() {
        return ctx.err(400, "plugin 格式错误");
    }
    // 纯订阅同步（本地无插件）时客户端会传空 plugin 仅携带 subscriptions。
    let plugin_empty = plugin.get("id").and_then(|v| v.as_str()).unwrap_or("").is_empty()
        && plugin.get("script").and_then(|v| v.as_str()).unwrap_or("").is_empty();
    if plugin_empty && data.get("subscriptions").is_none() {
        return ctx.err(400, "缺少插件或订阅数据");
    }
    let is_first = matches!(data.get("is_first"), Some(Value::Bool(true)));
    let mut save_data = read_snapshot(&ciyuanxi_id, "plugins.json").unwrap_or_else(|_| {
        json!({
            "version": 1, "uploaded_at": now_str(), "timestamp": now_ts(),
            "stats": { "plugin_count": 0, "subscription_count": 0 }, "plugins": [], "subscriptions": []
        })
    });
    if is_first {
        save_data["plugins"] = json!([]);
        save_data["stats"]["plugin_count"] = json!(0);
    }
    let mut plugins: Vec<Value> = save_data.get("plugins").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    let pid = plugin.get("id").cloned().unwrap_or(Value::Null);
    let mut found = false;
    if !plugin_empty {
        for p in plugins.iter_mut() {
            if p.get("id").cloned().unwrap_or(Value::Null) == pid {
                *p = plugin.clone();
                found = true;
                break;
            }
        }
        if !found {
            plugins.push(plugin.clone());
        }
    }
    let count = plugins.len() as i64;
    save_data["plugins"] = json!(plugins);
    save_data["stats"]["plugin_count"] = json!(count);
    // 订阅链接列表整包替换（与插件同步语义一致：上传端为权威）。
    if let Some(subs) = data.get("subscriptions").and_then(sanitize_subscriptions) {
        let sub_count = subs.len() as i64;
        save_data["subscriptions"] = json!(subs);
        save_data["stats"]["subscription_count"] = json!(sub_count);
    }
    save_data["uploaded_at"] = json!(now_str());
    save_data["timestamp"] = json!(now_ts());
    if !write_snapshot(&ciyuanxi_id, "plugins.json", &save_data) {
        return ctx.err(500, "文件写入失败");
    }
    ctx.ok("上传成功", json!({ "plugin_count": count }))
}

pub async fn plugin_sync_download(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(&ciyuanxi_id, "plugins.json") {
        Ok(v) => ctx.ok("获取成功", v),
        Err(_) => ctx.ok("暂无同步数据", json!({ "plugins": [] })),
    }
}

/// 平台标识对应的快照文件名：desktop / mobile 设置结构不同，分开存储互不覆盖。
/// 未携带 platform（旧客户端）沿用旧的 settings.json，下载时平台文件不存在也回退到它。
fn settings_file_name(platform: &str) -> String {
    match platform {
        "desktop" => "settings_desktop.json".to_string(),
        "mobile" => "settings_mobile.json".to_string(),
        _ => "settings.json".to_string(),
    }
}

pub async fn settings_sync_upload(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let settings = data.get("settings").cloned().unwrap_or(Value::Null);
    if !settings.is_object() {
        return ctx.err(400, "settings 格式错误");
    }
    let file_name = settings_file_name(str_of(&data, "platform").trim());
    let save = json!({
        "version": 1,
        "uploaded_at": now_str(),
        "timestamp": now_ts(),
        "settings": settings
    });
    if !write_snapshot(&ciyuanxi_id, &file_name, &save) {
        return ctx.err(500, "文件写入失败");
    }
    ctx.ok("上传成功", json!({ "uploaded_at": save["uploaded_at"] }))
}

pub async fn settings_sync_download(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let platform = str_of(&data, "platform").trim().to_string();
    let file_name = settings_file_name(&platform);
    // 平台文件优先；不存在时回退旧版共用 settings.json（首次升级迁移）
    if platform == "desktop" || platform == "mobile" {
        if let Ok(v) = read_snapshot(&ciyuanxi_id, &file_name) {
            return ctx.ok("获取成功", v);
        }
        if let Ok(v) = read_snapshot(&ciyuanxi_id, "settings.json") {
            return ctx.ok("获取成功", v);
        }
    } else if let Ok(v) = read_snapshot(&ciyuanxi_id, &file_name) {
        return ctx.ok("获取成功", v);
    }
    ctx.ok("暂无同步数据", json!({ "settings": null }))
}

/// 上传当前用户收藏歌曲列表（文件快照：data/sync/{id}/favorites.json）
pub async fn favorites_sync_upload(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let favorites = data.get("favorites").cloned().unwrap_or_else(|| json!([]));
    if !favorites.is_array() {
        return ctx.err(400, "favorites 格式错误");
    }
    let count = favorites.as_array().map(|a| a.len() as i64).unwrap_or(0);
    let save = json!({
        "version": 1,
        "uploaded_at": now_str(),
        "timestamp": now_ts(),
        "stats": { "song_count": count },
        "favorites": favorites
    });
    if !write_snapshot(&ciyuanxi_id, "favorites.json", &save) {
        return ctx.err(500, "文件写入失败");
    }
    ctx.ok("上传成功", json!({ "song_count": count }))
}

/// 下载指定用户的收藏歌曲列表（排行榜"查看"用户详情用）
pub async fn favorites_sync_download(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(&ciyuanxi_id, "favorites.json") {
        Ok(v) => ctx.ok("获取成功", v),
        Err(_) => ctx.ok("暂无同步数据", json!({ "favorites": [] })),
    }
}

/// 上传当前用户听歌统计快照（文件快照：data/sync/{id}/listen_stats.json）。
/// 载荷中 listen_stats 为统计对象（累计时长/首数 + 每日明细），整份存储。
/// merged = 本地离线数据是否已被一次性地并入服务端累计；cleared = 服务端后台是否已清零（制裁）。
pub async fn listen_stats_sync_upload(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let stats = data.get("listen_stats").cloned().unwrap_or_else(|| json!(null));
    if stats.is_null() {
        return ctx.err(400, "listen_stats 不能为空");
    }
    let merged = matches!(data.get("merged"), Some(Value::Bool(true)));
    let cleared = matches!(data.get("cleared"), Some(Value::Bool(true)));
    // reset_at（后台清零时间点）尽量沿用客户端上报值，缺失时保留既有文件值，避免覆盖丢失。
    let mut reset_at = data.get("reset_at").and_then(Value::as_i64).unwrap_or(0);
    if reset_at == 0 {
        reset_at = read_snapshot(&ciyuanxi_id, "listen_stats.json")
            .ok()
            .and_then(|v| v.get("reset_at").and_then(Value::as_i64))
            .unwrap_or(0);
    }
    // reason（清零原因）客户端一般不回传，沿用既有文件值，确保后续设备仍能读到原因。
    let mut reason = data
        .get("reason")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_default();
    if reason.trim().is_empty() {
        reason = read_snapshot(&ciyuanxi_id, "listen_stats.json")
            .ok()
            .and_then(|v| v.get("reason").and_then(Value::as_str).map(String::from))
            .unwrap_or_default();
    }
    let save = json!({
        "version": 2,
        "uploaded_at": now_str(),
        "timestamp": now_ts(),
        "merged": merged,
        "cleared": cleared,
        "reset_at": reset_at,
        "reason": reason,
        "listen_stats": stats
    });
    if !write_snapshot(&ciyuanxi_id, "listen_stats.json", &save) {
        return ctx.err(500, "文件写入失败");
    }
    ctx.ok("上传成功", json!({ "updated": true }))
}

/// 下载当前用户听歌统计快照。
pub async fn listen_stats_sync_download(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(&ciyuanxi_id, "listen_stats.json") {
        Ok(v) => ctx.ok("获取成功", v),
        Err(_) => ctx.ok("暂无同步数据", json!({ "listen_stats": null })),
    }
}

