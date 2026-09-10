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
    let mut merged: Vec<Value> = map.into_values().collect();
    // merge 模式（新客户端）：按 cloudId 逐条 upsert 到已有快照、保留其他设备新增、
    // 并按 delete_cloud_ids 删除，防止整包重建把其他设备新增的歌单抹掉。
    // 旧客户端不带 merge 时保持原有整包覆盖（向后兼容）。
    let do_merge = matches!(data.get("merge"), Some(Value::Bool(true)));
    // 云端歌单统一使用「字符串 cloudId」作为稳定键：
    // 上传端回传 [ {id: 本地id, cloudId: 云端id} ]，使上传端能写回本地、跨设备稳定定位。
    let mut id_map: Vec<Value> = Vec::new();
    if do_merge {
        let existing: Vec<Value> = read_snapshot(&ciyuanxi_id, "playlists.json")
            .ok()
            .and_then(|v| v.get("playlists").cloned())
            .and_then(|x| x.as_array().cloned())
            .unwrap_or_default();
        let mut by_cloud: Vec<(Option<String>, Value)> = existing
            .into_iter()
            .map(|it| {
                let k = it
                    .get("cloudId")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty());
                (k, it)
            })
            .collect();
        let mut probe: i64 = 0;
        for mut pl in merged {
            let local_id = pl.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            // 同本地 id 的行视为同一歌单：合并后无需保留旧行，并继承其已有云端 id（若有）
            let mut inherited: Option<String> = None;
            // 已有行的歌曲级删除墓碑（歌单内移除单曲），合并时保留
            let mut prev_deleted: std::collections::HashSet<String> = std::collections::HashSet::new();
            by_cloud.retain(|(k, r)| {
                if r.get("id").and_then(|v| v.as_str()) == Some(local_id.as_str()) {
                    if let Some(ck) = k {
                        inherited = Some(ck.clone());
                    }
                    if let Some(Value::Array(dl)) = r.get("deletedSongPaths") {
                        for d in dl {
                            if let Some(s) = d.as_str() {
                                prev_deleted.insert(s.to_string());
                            }
                        }
                    }
                    false
                } else {
                    true
                }
            });
            // cloudId 统一为字符串稳定键：已有则沿用；否则为本地上传歌单分配新云端 id，
            // 并写入快照行，回传给上传端写回本地（保证同歌单再次上传可定位到同一份）。
            let mut key = pl
                .get("cloudId")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty());
            if key.is_none() {
                key = inherited;
            }
            if key.is_none() {
                key = Some(format!("c{}{}", now_ts(), probe));
                probe += 1;
            }
            if let Some(k) = &key {
                pl["cloudId"] = json!(k);
            }
            // 歌曲级删除墓碑合并（歌单内移除单曲的全端传播）：
            // deleted = 已有墓碑 ∪ 本次上报 − 本次实际上传的歌曲 path（重新添加自动解除删除）；
            // songs 按删除集裁剪，避免其他端下载时已删歌曲回流；随快照存储并在下载响应带出。
            let client_deleted: std::collections::HashSet<String> = pl
                .get("deletedSongPaths")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let uploaded_paths: std::collections::HashSet<String> = pl
                .get("songs")
                .and_then(|s| s.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|it| it.get("path").and_then(|p| p.as_str()))
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let mut deleted: Vec<String> = prev_deleted
                .union(&client_deleted)
                .filter(|p| !uploaded_paths.contains(*p))
                .cloned()
                .collect();
            deleted.sort();
            deleted.dedup();
            if !deleted.is_empty() {
                if let Some(songs) = pl.get_mut("songs").and_then(|s| s.as_array_mut()) {
                    songs.retain(|it| {
                        it.get("path")
                            .and_then(|p| p.as_str())
                            .map(|p| !deleted.contains(&p.to_string()))
                            .unwrap_or(true)
                    });
                }
                pl["deletedSongPaths"] = json!(deleted);
            } else if let Some(obj) = pl.as_object_mut() {
                obj.remove("deletedSongPaths");
            }
            by_cloud.push((key.clone(), pl));
            id_map.push(json!({ "id": local_id, "cloudId": key }));
        }
        if let Some(Value::Array(del)) = data.get("delete_cloud_ids") {
            let delset: std::collections::HashSet<String> = del
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            by_cloud.retain(|(k, _)| match k {
                Some(c) => !delset.contains(c),
                None => true,
            });
        }
        merged = by_cloud.into_iter().map(|(_, it)| it).collect();
    }
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
        // 回传 id_map：本地 id → 云端字符串 cloudId，供上传端写回本地，
        // 保证同歌单再次上传可定位到同一份、跨设备稳定定位。
        ctx.ok("同步成功", json!({
            "playlist_count": merged.len(),
            "song_total": song_total,
            "id_map": id_map
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
            Ok(v) => {
                // 自愈：给缺失 cloudId 的云端歌单分配稳定字符串 id 并回写，
                // 保证修复前的历史快照下载后也能稳定定位、删除范围选项可用。
                let (fixed_snapshot, changed) = ensure_cloud_ids(v);
                if changed {
                    let _ = write_snapshot(&ciyuanxi_id, "playlists.json", &fixed_snapshot);
                }
                ctx.ok("获取成功", fixed_snapshot)
            }
            Err(_) => ctx.ok("数据读取失败", json!({ "playlists": [] })),
        },
        Err(_) => ctx.ok("数据读取失败", json!({ "playlists": [] })),
    }
}

/// 给快照中缺少 cloudId 的歌单分配稳定字符串云端 id（复用上传端相同格式），返回（新快照，是否变更）。
fn ensure_cloud_ids(snapshot: Value) -> (Value, bool) {
    let mut changed = false;
    let Some(playlists) = snapshot.get("playlists").cloned() else {
        return (snapshot, changed);
    };
    let Some(playlists) = playlists.as_array() else {
        return (snapshot, changed);
    };
    let mut probe: i64 = 0;
    let mut out = Vec::with_capacity(playlists.len());
    for mut pl in playlists.clone() {
        let has_cloud_id = pl
            .get("cloudId")
            .and_then(|c| c.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        if !has_cloud_id {
            pl["cloudId"] = json!(format!("c{}{}", now_ts(), probe));
            probe += 1;
            changed = true;
        }
        out.push(pl);
    }
    let mut s = snapshot.clone();
    if let Some(arr) = s["playlists"].as_array_mut() {
        *arr = out;
    }
    (s, changed)
}

/// 从文件存储快照中按云端字符串 cloudId 删除歌单（「删除全部/仅保留本地」的云端落盘操作）。
/// 仅删除，不重建；保留其余歌单与其他端新增。传空列表时为无操作。
pub async fn file_sync_delete_playlist(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let delset: std::collections::HashSet<String> = data
        .get("cloud_ids")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();
    if delset.is_empty() {
        return ctx.ok("删除成功", json!({ "deleted": 0 }));
    }
    let existing = read_snapshot(&ciyuanxi_id, "playlists.json").ok().unwrap_or_else(|| json!({ "playlists": [] }));
    let last_ts = existing.get("timestamp").and_then(|v| v.as_i64()).unwrap_or_else(now_ts);
    let playlists = existing
        .get("playlists")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let before = playlists.len();
    let kept: Vec<Value> = playlists
        .into_iter()
        .filter(|pl| {
            let c = pl.get("cloudId").and_then(|v| v.as_str()).unwrap_or("");
            !delset.contains(c)
        })
        .collect();
    let deleted = before - kept.len();
    let song_total: i64 = kept
        .iter()
        .map(|pl| {
            pl.get("songs")
                .and_then(|s| s.as_array())
                .map(|a| a.len() as i64)
                .unwrap_or(0)
        })
        .sum();
    let save = json!({
        "version": 4,
        "uploaded_at": now_str(),
        "timestamp": last_ts,
        "stats": {
            "playlist_count": kept.len(),
            "song_total": song_total
        },
        "playlists": kept
    });
    let ok = write_snapshot(&ciyuanxi_id, "playlists.json", &save);
    if ok {
        ctx.ok("删除成功", json!({ "deleted": deleted }))
    } else {
        ctx.err(500, "写入文件失败")
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
            if p.get("id").cloned().unwrap_or(Value::Null) != pid {
                continue;
            }
            // 旧客户端可能不带用户变量加密块：合并时保留已存的密文，
            // 避免新端上传的加密变量被无字段的上传覆盖丢失（服务端仅存密文，不解密）。
            let mut merged = plugin.clone();
            if merged.get("userVariablesEncrypted").is_none() {
                if let Some(old_enc) = p.get("userVariablesEncrypted").cloned() {
                    merged["userVariablesEncrypted"] = old_enc;
                }
            }
            *p = merged;
            found = true;
            break;
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

/// 按 id 从云端插件快照中删除插件（「删除全部/仅保留本地/仅删云端」的云端落盘操作）。
/// 仅删除指定 id，不影响其余插件与订阅列表。传空列表时为无操作。
pub async fn plugin_sync_delete(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let delset: std::collections::HashSet<String> = data
        .get("plugin_ids")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();
    if delset.is_empty() {
        return ctx.ok("删除成功", json!({ "deleted": 0, "plugin_count": 0 }));
    }
    let existing = read_snapshot(&ciyuanxi_id, "plugins.json").ok().unwrap_or_else(|| {
        json!({
            "version": 1, "uploaded_at": now_str(), "timestamp": now_ts(),
            "stats": { "plugin_count": 0, "subscription_count": 0 }, "plugins": [], "subscriptions": []
        })
    });
    let plugins = existing
        .get("plugins")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let before = plugins.len();
    let kept: Vec<Value> = plugins
        .into_iter()
        .filter(|p| {
            let pid = p.get("id").and_then(|v| v.as_str()).unwrap_or("");
            !delset.contains(pid)
        })
        .collect();
    let deleted = before - kept.len();
    let mut save = existing;
    save["plugins"] = json!(kept);
    save["stats"]["plugin_count"] = json!(kept.len() as i64);
    save["uploaded_at"] = json!(now_str());
    save["timestamp"] = json!(now_ts());
    if !write_snapshot(&ciyuanxi_id, "plugins.json", &save) {
        return ctx.err(500, "文件写入失败");
    }
    ctx.ok("删除成功", json!({ "deleted": deleted, "plugin_count": kept.len() as i64 }))
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
///
/// 合并模式（merge: true）：逐条按 path upsert + delete_paths 删除，保留未涉及条目，
/// 供客户端实现收藏按键合并，避免整包覆盖把其他设备新增的收藏抹掉。
/// 旧客户端不带 merge 时不走合并，保持整包覆盖语义（向后兼容）。
pub async fn favorites_sync_upload(body: &str, ctx: ReqCtx) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let merge = matches!(data.get("merge"), Some(Value::Bool(true)));
    let favorites = data.get("favorites").cloned().unwrap_or_else(|| json!([]));
    if !favorites.is_array() {
        return ctx.err(400, "favorites 格式错误");
    }

    let final_list: Vec<Value>;
    let count: i64;
    if merge {
        let existing: Vec<Value> = read_snapshot(&ciyuanxi_id, "favorites.json")
            .ok()
            .and_then(|v| v.get("favorites").cloned())
            .and_then(|x| x.as_array().cloned())
            .unwrap_or_default();
        let mut by_path: Vec<(Option<String>, Value)> = existing
            .into_iter()
            .map(|it| {
                let p = it.get("path").and_then(|x| x.as_str()).map(|s| s.to_string());
                (p, it)
            })
            .collect();
        // upsert：同 path 覆盖，新 path 追加
        for it in favorites.as_array().cloned().unwrap_or_default() {
            let item = it;
            let p = item.get("path").and_then(|x| x.as_str()).map(|s| s.to_string());
            if let Some(ref pk) = p {
                if let Some(pos) = by_path.iter().position(|(k, _)| k.as_ref().map(|s| s == pk).unwrap_or(false)) {
                    by_path[pos].1 = item;
                } else {
                    by_path.push((p, item));
                }
            } else {
                by_path.push((None, item));
            }
        }
        // delete_paths：删除指定 path 的收藏
        if let Some(Value::Array(del)) = data.get("delete_paths") {
            let delset: std::collections::HashSet<String> = del
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            by_path.retain(|(k, _)| match k {
                Some(p) => !delset.contains(p),
                None => true,
            });
        }
        final_list = by_path.into_iter().map(|(_, it)| it).collect();
        count = final_list.len() as i64;
    } else {
        final_list = favorites.as_array().cloned().unwrap_or_default();
        count = final_list.len() as i64;
    }

    let save = json!({
        "version": 1,
        "uploaded_at": now_str(),
        "timestamp": now_ts(),
        "stats": { "song_count": count },
        "favorites": final_list
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

