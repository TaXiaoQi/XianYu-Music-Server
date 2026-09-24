use axum::response::Response;
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use std::path::PathBuf;

use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

/// 旧版把用户同步数据按文件存放在 data/sync/<弦予号>/*.json，
/// 现已全部迁入 user_sync_files / user_sync_chunks 两张表。
/// 以下路径函数仅供旧文件迁移回填与遗留清理使用。
fn sync_root() -> PathBuf {
    PathBuf::from("data/sync")
}

fn sync_dir(ciyuanxi_id: &str) -> PathBuf {
    let digits: String = ciyuanxi_id.chars().filter(|c| c.is_ascii_digit()).collect();
    sync_root().join(digits)
}

fn sync_user_id(ciyuanxi_id: &str) -> String {
    ciyuanxi_id.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn now_str() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn now_ts() -> i64 {
    Utc::now().timestamp()
}

/// 整包读用户同步文件：先查 DB，未命中时尝试旧文件懒迁移（导入后删除旧文件）。
async fn read_snapshot(pool: &MySqlPool, ciyuanxi_id: &str, name: &str) -> Result<Value, ()> {
    let uid = sync_user_id(ciyuanxi_id);
    let row: Option<(String,)> =
        sqlx::query_as("SELECT content FROM user_sync_files WHERE ciyuanxi_id = ? AND file_name = ?")
            .bind(&uid)
            .bind(name)
            .fetch_optional(pool)
            .await
            .map_err(|_| ())?;
    if let Some((content,)) = row {
        return serde_json::from_str(&content).map_err(|_| ());
    }
    // 旧文件回退：命中即导入 DB 并移除旧文件
    let file = sync_dir(ciyuanxi_id).join(name);
    if !file.exists() {
        return Err(());
    }
    let content = std::fs::read_to_string(&file).map_err(|_| ())?;
    let v: Value = serde_json::from_str(&content).map_err(|_| ())?;
    let _ = sqlx::query("INSERT IGNORE INTO user_sync_files (ciyuanxi_id, file_name, content, content_size) VALUES (?, ?, ?, ?)")
        .bind(&uid)
        .bind(name)
        .bind(&content)
        .bind(content.len() as i64)
        .execute(pool)
        .await;
    let _ = std::fs::remove_file(&file);
    Ok(v)
}

/// 整包写用户同步文件（DB upsert），成功后清掉可能残留的旧文件。
async fn write_snapshot(pool: &MySqlPool, ciyuanxi_id: &str, name: &str, data: &Value) -> bool {
    let uid = sync_user_id(ciyuanxi_id);
    let content = serde_json::to_string(data).unwrap_or_default();
    let ok = sqlx::query(
        "INSERT INTO user_sync_files (ciyuanxi_id, file_name, content, content_size) VALUES (?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE content = VALUES(content), content_size = VALUES(content_size)",
    )
    .bind(&uid)
    .bind(name)
    .bind(&content)
    .bind(content.len() as i64)
    .execute(pool)
    .await
    .is_ok();
    if ok {
        let _ = std::fs::remove_file(sync_dir(ciyuanxi_id).join(name));
    }
    ok
}

/// 注销/清理用户时删除其全部同步数据（DB + 遗留目录兜底）。
pub async fn delete_user_sync_data(pool: &MySqlPool, ciyuanxi_id: &str) {
    let uid = sync_user_id(ciyuanxi_id);
    let _ = sqlx::query("DELETE FROM user_sync_files WHERE ciyuanxi_id = ?")
        .bind(&uid)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_sync_chunks WHERE ciyuanxi_id = ?")
        .bind(&uid)
        .execute(pool)
        .await;
    let _ = std::fs::remove_dir_all(sync_dir(ciyuanxi_id));
}

/// 启动回填：把 data/sync 下遗留的旧文件（含中断上传的分块）导入 DB，
/// 导入成功的文件即删除；幂等，可重复执行。
pub async fn backfill_legacy_sync_files(pool: &MySqlPool) {
    let root = sync_root();
    let mut migrated = 0usize;
    let entries = match std::fs::read_dir(&root) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let uid = e.file_name().to_string_lossy().to_string();
            let files = match std::fs::read_dir(&p) {
                Ok(f) => f,
                Err(_) => continue,
            };
            for f in files.flatten() {
                let fp = f.path();
                if fp.is_dir() {
                    // chunks 目录：遗留的分块文件导入 user_sync_chunks
                    if fp.file_name().map(|n| n == "chunks").unwrap_or(false) {
                        if let Ok(cs) = std::fs::read_dir(&fp) {
                            for c in cs.flatten() {
                                let cp = c.path();
                                let cname = c.file_name().to_string_lossy().to_string();
                                if let Some(idx) = parse_chunk_index(&cname) {
                                    if let Ok(content) = std::fs::read_to_string(&cp) {
                                        let ins = sqlx::query(
                                            "INSERT IGNORE INTO user_sync_chunks (ciyuanxi_id, chunk_index, total_chunks, content) VALUES (?, ?, 0, ?)",
                                        )
                                        .bind(&uid)
                                        .bind(idx)
                                        .bind(&content)
                                        .execute(pool)
                                        .await;
                                        if ins.map(|r| r.rows_affected() > 0).unwrap_or(false) {
                                            let _ = std::fs::remove_file(&cp);
                                            migrated += 1;
                                        }
                                    }
                                }
                            }
                            let _ = std::fs::remove_dir(&fp);
                        }
                    }
                } else {
                    let name = f.file_name().to_string_lossy().to_string();
                    if !name.ends_with(".json") {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(&fp) {
                        let size = content.len() as i64;
                        let ins = sqlx::query(
                            "INSERT IGNORE INTO user_sync_files (ciyuanxi_id, file_name, content, content_size) VALUES (?, ?, ?, ?)",
                        )
                        .bind(&uid)
                        .bind(&name)
                        .bind(&content)
                        .bind(size)
                        .execute(pool)
                        .await;
                        if ins.map(|r| r.rows_affected() > 0).unwrap_or(false) {
                            let _ = std::fs::remove_file(&fp);
                            migrated += 1;
                        }
                    }
                }
            }
            let _ = std::fs::remove_dir(&p);
        } else {
            // 根级文件：属于无数字字符的异常弦予号（旧逻辑落到根目录）
            let name = e.file_name().to_string_lossy().to_string();
            if !name.ends_with(".json") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&p) {
                let size = content.len() as i64;
                let ins = sqlx::query(
                    "INSERT IGNORE INTO user_sync_files (ciyuanxi_id, file_name, content, content_size) VALUES ('', ?, ?, ?)",
                )
                .bind(&name)
                .bind(&content)
                .bind(size)
                .execute(pool)
                .await;
                if ins.map(|r| r.rows_affected() > 0).unwrap_or(false) {
                    let _ = std::fs::remove_file(&p);
                    migrated += 1;
                }
            }
        }
    }
    // 清理超过 7 天的遗留分块（客户端中断上传的残渣）
    let _ = sqlx::query("DELETE FROM user_sync_chunks WHERE updated_at < NOW() - INTERVAL 7 DAY")
        .execute(pool)
        .await;
    if migrated > 0 {
        tracing::info!("[sync] backfilled {} legacy sync items into db", migrated);
    }
}

pub async fn file_sync_upload_start(_body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(_body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let uid = sync_user_id(&ciyuanxi_id);
    let ok = sqlx::query("DELETE FROM user_sync_chunks WHERE ciyuanxi_id = ?")
        .bind(&uid)
        .execute(pool)
        .await
        .is_ok();
    if !ok {
        return ctx.err(500, "重置分块失败");
    }
    ctx.ok("ok", json!({ "chunk_dir_ready": true }))
}

pub async fn file_sync_upload_chunk(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let chunk_index = data.get("chunk_index").and_then(|v| v.as_i64()).unwrap_or(0);
    let total_chunks = data.get("total_chunks").and_then(|v| v.as_i64()).unwrap_or(1);
    let chunk_data = data.get("chunk_data").cloned().unwrap_or_else(|| json!([]));
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let uid = sync_user_id(&ciyuanxi_id);
    let payload = json!({
        "chunk_index": chunk_index,
        "total_chunks": total_chunks,
        "chunk_data": chunk_data
    });
    let content = serde_json::to_string(&payload).unwrap_or_default();
    let ok = sqlx::query(
        "INSERT INTO user_sync_chunks (ciyuanxi_id, chunk_index, total_chunks, content) VALUES (?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE total_chunks = VALUES(total_chunks), content = VALUES(content)",
    )
    .bind(&uid)
    .bind(chunk_index)
    .bind(total_chunks)
    .bind(&content)
    .execute(pool)
    .await
    .is_ok();
    if ok {
        ctx.ok("ok", json!({ "chunk_index": chunk_index, "total_chunks": total_chunks }))
    } else {
        ctx.err(500, &format!("分块 {} 写入失败", chunk_index))
    }
}

pub async fn file_sync_upload_finish(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let uid = sync_user_id(&ciyuanxi_id);
    let rows: Vec<(i64, String)> = match sqlx::query_as(
        "SELECT chunk_index, content FROM user_sync_chunks WHERE ciyuanxi_id = ? ORDER BY chunk_index ASC",
    )
    .bind(&uid)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => return ctx.err(400, "没有分块数据"),
    };
    if rows.is_empty() {
        return ctx.err(400, "没有分块数据");
    }
    let mut all_playlists: Vec<Value> = Vec::new();
    for (_, content) in &rows {
        if let Ok(chunk) = serde_json::from_str::<Value>(content) {
            if let Some(items) = chunk.get("chunk_data").and_then(|x| x.as_array()) {
                all_playlists.extend(items.clone());
            }
        }
    }
    let _ = sqlx::query("DELETE FROM user_sync_chunks WHERE ciyuanxi_id = ?")
        .bind(&uid)
        .execute(pool)
        .await;
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
    let do_merge = matches!(data.get("merge"), Some(Value::Bool(true)));
    let mut id_map: Vec<Value> = Vec::new();
    if do_merge {
        let existing: Vec<Value> = read_snapshot(pool, &ciyuanxi_id, "playlists.json")
            .await
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
            let mut inherited: Option<String> = None;
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
    let ok = write_snapshot(pool, &ciyuanxi_id, "playlists.json", &save).await;
    let meta = json!({
        "last_sync": now_str(),
        "last_sync_timestamp": now_ts(),
        "playlist_count": merged.len(),
        "song_total": song_total
    });
    let _ = write_snapshot(pool, &ciyuanxi_id, "meta.json", &meta).await;
    if ok {
        ctx.ok("同步成功", json!({
            "playlist_count": merged.len(),
            "song_total": song_total,
            "id_map": id_map
        }))
    } else {
        ctx.err(500, "同步数据写入失败")
    }
}

fn parse_chunk_index(name: &str) -> Option<i64> {
    let trimmed = name.strip_prefix("chunk_")?.strip_suffix(".json")?;
    trimmed.parse().ok()
}

pub async fn file_sync_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(pool, &ciyuanxi_id, "playlists.json").await {
        Ok(v) => {
            let (fixed_snapshot, changed) = ensure_cloud_ids(v);
            if changed {
                let _ = write_snapshot(pool, &ciyuanxi_id, "playlists.json", &fixed_snapshot).await;
            }
            ctx.ok("获取成功", fixed_snapshot)
        }
        Err(_) => ctx.ok("暂无同步数据", json!({ "playlists": [] })),
    }
}

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

pub async fn file_sync_delete_playlist(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
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
    let existing = read_snapshot(pool, &ciyuanxi_id, "playlists.json")
        .await
        .ok()
        .unwrap_or_else(|| json!({ "playlists": [] }));
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
    let ok = write_snapshot(pool, &ciyuanxi_id, "playlists.json", &save).await;
    if ok {
        ctx.ok("删除成功", json!({ "deleted": deleted }))
    } else {
        ctx.err(500, "同步数据写入失败")
    }
}

pub async fn write_listen_stats_reset(pool: &MySqlPool, ciyuanxi_id: &str, reason: &str) -> bool {
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
    write_snapshot(pool, ciyuanxi_id, "listen_stats.json", &save).await
}

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

pub async fn plugin_sync_upload_one(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let plugin = data.get("plugin").cloned().unwrap_or(Value::Null);
    if !plugin.is_object() {
        return ctx.err(400, "plugin 格式错误");
    }
    let plugin_empty = plugin.get("id").and_then(|v| v.as_str()).unwrap_or("").is_empty()
        && plugin.get("script").and_then(|v| v.as_str()).unwrap_or("").is_empty();
    if plugin_empty && data.get("subscriptions").is_none() {
        return ctx.err(400, "缺少插件或订阅数据");
    }
    let is_first = matches!(data.get("is_first"), Some(Value::Bool(true)));
    let mut save_data = read_snapshot(pool, &ciyuanxi_id, "plugins.json").await.unwrap_or_else(|_| {
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
    if let Some(subs) = data.get("subscriptions").and_then(sanitize_subscriptions) {
        let sub_count = subs.len() as i64;
        save_data["subscriptions"] = json!(subs);
        save_data["stats"]["subscription_count"] = json!(sub_count);
    }
    save_data["uploaded_at"] = json!(now_str());
    save_data["timestamp"] = json!(now_ts());
    if !write_snapshot(pool, &ciyuanxi_id, "plugins.json", &save_data).await {
        return ctx.err(500, "同步数据写入失败");
    }
    ctx.ok("上传成功", json!({ "plugin_count": count }))
}

pub async fn plugin_sync_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(pool, &ciyuanxi_id, "plugins.json").await {
        Ok(v) => ctx.ok("获取成功", v),
        Err(_) => ctx.ok("暂无同步数据", json!({ "plugins": [] })),
    }
}

pub async fn plugin_sync_delete(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
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
    let existing = read_snapshot(pool, &ciyuanxi_id, "plugins.json")
        .await
        .ok()
        .unwrap_or_else(|| {
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
    if !write_snapshot(pool, &ciyuanxi_id, "plugins.json", &save).await {
        return ctx.err(500, "同步数据写入失败");
    }
    ctx.ok("删除成功", json!({ "deleted": deleted, "plugin_count": kept.len() as i64 }))
}

fn settings_file_name(platform: &str) -> String {
    match platform {
        "desktop" => "settings_desktop.json".to_string(),
        "mobile" => "settings_mobile.json".to_string(),
        _ => "settings.json".to_string(),
    }
}

pub async fn settings_sync_upload(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
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
    if !write_snapshot(pool, &ciyuanxi_id, &file_name, &save).await {
        return ctx.err(500, "同步数据写入失败");
    }
    ctx.ok("上传成功", json!({ "uploaded_at": save["uploaded_at"] }))
}

pub async fn settings_sync_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let platform = str_of(&data, "platform").trim().to_string();
    let file_name = settings_file_name(&platform);
    if platform == "desktop" || platform == "mobile" {
        if let Ok(v) = read_snapshot(pool, &ciyuanxi_id, &file_name).await {
            return ctx.ok("获取成功", v);
        }
        if let Ok(v) = read_snapshot(pool, &ciyuanxi_id, "settings.json").await {
            return ctx.ok("获取成功", v);
        }
    } else if let Ok(v) = read_snapshot(pool, &ciyuanxi_id, &file_name).await {
        return ctx.ok("获取成功", v);
    }
    ctx.ok("暂无同步数据", json!({ "settings": null }))
}

pub async fn favorites_sync_upload(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
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
        let existing: Vec<Value> = read_snapshot(pool, &ciyuanxi_id, "favorites.json")
            .await
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
    if !write_snapshot(pool, &ciyuanxi_id, "favorites.json", &save).await {
        return ctx.err(500, "同步数据写入失败");
    }
    ctx.ok("上传成功", json!({ "song_count": count }))
}

pub async fn favorites_sync_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(pool, &ciyuanxi_id, "favorites.json").await {
        Ok(v) => ctx.ok("获取成功", v),
        Err(_) => ctx.ok("暂无同步数据", json!({ "favorites": [] })),
    }
}

pub async fn listen_stats_sync_upload(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
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
    let mut reset_at = data.get("reset_at").and_then(Value::as_i64).unwrap_or(0);
    if reset_at == 0 {
        reset_at = read_snapshot(pool, &ciyuanxi_id, "listen_stats.json")
            .await
            .ok()
            .and_then(|v| v.get("reset_at").and_then(Value::as_i64))
            .unwrap_or(0);
    }
    let mut reason = data
        .get("reason")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_default();
    if reason.trim().is_empty() {
        reason = read_snapshot(pool, &ciyuanxi_id, "listen_stats.json")
            .await
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
    if !write_snapshot(pool, &ciyuanxi_id, "listen_stats.json", &save).await {
        return ctx.err(500, "同步数据写入失败");
    }
    ctx.ok("上传成功", json!({ "updated": true }))
}

pub async fn listen_stats_sync_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(pool, &ciyuanxi_id, "listen_stats.json").await {
        Ok(v) => ctx.ok("获取成功", v),
        Err(_) => ctx.ok("暂无同步数据", json!({ "listen_stats": null })),
    }
}
