use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::handlers::helpers::{parse_body, str_of};
use crate::handlers::sync_diff;
use crate::handlers::sync_merge;
use crate::handlers::sync_store::{now_str, now_ts, read_snapshot, write_snapshot};
use crate::response::ReqCtx;

// 存储层已迁至 sync_store / sync_merge；旧引用路径保持可用。
pub use crate::handlers::sync_store::{backfill_legacy_sync_files, delete_user_sync_data};

// ===== 删除广播账本 =====
// 「删除全部/删除云端」后条目已从快照移除，但其他端仍持有本地副本，且会在
// 下次上传时把副本并回云端（复活）。各域快照内维护已删除条目账本（按标识
// 去重、cap 500 丢最旧），随下载响应下发；拉取端按账本裁剪本地副本，同标识
// 重新上传时从账本移除（复活即撤销广播）。
const LEDGER_CAP: usize = 500;

/// 记账：把 ids 追加进快照账本字段（field 内条目形如 {id_key, at}）。
fn ledger_append(doc: &mut Value, field: &str, id_key: &str, ids: &[String]) {
    if ids.is_empty() {
        return;
    }
    let now = now_ts();
    let mut entries: Vec<(String, i64)> = doc
        .get(field)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|e| {
                    let k = e.get(id_key)?.as_str()?.to_string();
                    let at = e.get("at").and_then(Value::as_i64).unwrap_or(0);
                    Some((k, at))
                })
                .collect()
        })
        .unwrap_or_default();
    for id in ids {
        let id = id.trim();
        if id.is_empty() {
            continue;
        }
        entries.retain(|(k, _)| k != id);
        entries.push((id.to_string(), now));
    }
    if entries.len() > LEDGER_CAP {
        entries.drain(..entries.len() - LEDGER_CAP);
    }
    doc[field] = Value::Array(
        entries
            .into_iter()
            .map(|(k, at)| {
                let mut e = serde_json::Map::new();
                e.insert(id_key.to_string(), Value::String(k));
                e.insert("at".to_string(), json!(at));
                Value::Object(e)
            })
            .collect(),
    );
}

/// 撤账：同标识重新上传时移除（复活即撤销广播）。
fn ledger_remove(doc: &mut Value, field: &str, id_key: &str, ids: &[String]) {
    if ids.is_empty() {
        return;
    }
    let rm: std::collections::HashSet<&str> = ids
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if rm.is_empty() {
        return;
    }
    let keep: Vec<Value> = doc
        .get(field)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|e| {
            let k = e.get(id_key).and_then(Value::as_str).unwrap_or("");
            !rm.contains(k)
        })
        .collect();
    doc[field] = Value::Array(keep);
}

/// 读取账本标识列表（保持账本顺序，供下载响应扁平下发）。
fn ledger_keys(doc: &Value, field: &str, id_key: &str) -> Vec<String> {
    doc.get(field)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|e| {
                    e.get(id_key)
                        .and_then(Value::as_str)
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}

pub async fn file_sync_upload_start(_body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(_body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let ok = sqlx::query("DELETE FROM user_sync_chunks WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
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
    // 客户端 maxSongsPerChunk=1500/块，256 块足够 38 万首歌；防恶意刷行数
    if !(1..=256).contains(&total_chunks) {
        return ctx.err(400, "total_chunks 超出范围");
    }
    if !(0..total_chunks).contains(&chunk_index) {
        return ctx.err(400, "chunk_index 超出范围");
    }
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
    .bind(&ciyuanxi_id)
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
    let rows: Vec<(i64, String)> = match sqlx::query_as(
        "SELECT chunk_index, content FROM user_sync_chunks WHERE ciyuanxi_id = ? ORDER BY chunk_index ASC",
    )
    .bind(&ciyuanxi_id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => return ctx.err(400, "没有分块数据"),
    };
    let do_merge = matches!(data.get("merge"), Some(Value::Bool(true)));
    // merge=false（手动覆盖上传）允许零分块：空快照即快捷清空云端歌单
    if rows.is_empty() && do_merge {
        return ctx.err(400, "没有分块数据");
    }
    let mut uploaded = sync_merge::assemble_chunk_playlists(&rows);
    let _ = sqlx::query("DELETE FROM user_sync_chunks WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let mut id_map: Vec<Value> = Vec::new();
    // 账本挂在完整快照文档上（build_snapshot_doc 只组装歌单域），读写都取全文档
    let mut doc = read_snapshot(pool, &ciyuanxi_id, "playlists.json")
        .await
        .unwrap_or_else(|_| json!({ "playlists": [] }));
    let existing: Vec<Value> = doc
        .get("playlists")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    // 重新上传视为要回该歌单：按 cloudId 撤销删除广播
    let uploaded_cloud_ids: Vec<String> = uploaded
        .iter()
        .filter_map(|p| {
            p.get("cloudId")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
        })
        .filter(|s| !s.is_empty())
        .collect();
    ledger_remove(&mut doc, "deleted", "cloudId", &uploaded_cloud_ids);
    if do_merge {
        let delete_ids: Option<Vec<String>> = data
            .get("delete_cloud_ids")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect());
        // 显式删除云端的歌单进入删除广播账本
        if let Some(ids) = &delete_ids {
            ledger_append(&mut doc, "deleted", "cloudId", ids);
        }
        let (m, im) = sync_merge::merge_into_snapshot(uploaded, existing, delete_ids.as_deref());
        id_map = im;
        doc["playlists"] = json!(m);
    } else {
        // 全量替换（手动覆盖上传 merge=false）：被清掉的存量歌单记入广播账本，
        // 其他端下载时按 cloudId 裁剪本地副本，避免其自动同步把数据合并复活
        let kept: std::collections::HashSet<String> = uploaded
            .iter()
            .filter_map(|p| {
                p.get("cloudId")
                    .and_then(Value::as_str)
                    .map(|s| s.to_string())
            })
            .filter(|s| !s.is_empty())
            .collect();
        let cleared: Vec<String> = existing
            .iter()
            .filter_map(|p| {
                p.get("cloudId")
                    .and_then(Value::as_str)
                    .map(|s| s.to_string())
            })
            .filter(|s| !s.is_empty() && !kept.contains(s))
            .collect();
        if !cleared.is_empty() {
            ledger_append(&mut doc, "deleted", "cloudId", &cleared);
        }
        // 全量替换（手动覆盖上传 merge=false）：为缺 cloudId 的歌单分配探针 id
        // 并回传 id_map，供客户端把新歌单绑定云端 id（分配规则与 merge 分支一致）
        let ts = now_ts();
        for (n, pl) in uploaded.iter_mut().enumerate() {
            let has_cloud_id = pl
                .get("cloudId")
                .and_then(|c| c.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            let key = if has_cloud_id {
                pl.get("cloudId")
                    .and_then(|c| c.as_str())
                    .unwrap()
                    .to_string()
            } else {
                format!("c{}{}", ts, n)
            };
            let local_id = pl
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            pl["cloudId"] = json!(key);
            id_map.push(json!({ "id": local_id, "cloudId": key }));
        }
        doc["playlists"] = json!(uploaded);
    }
    let merged: Vec<Value> = doc
        .get("playlists")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let mut save = sync_merge::build_snapshot_doc(merged);
    if let Some(d) = doc.get("deleted") {
        save["deleted"] = d.clone();
    }
    let ok = write_snapshot(pool, &ciyuanxi_id, "playlists.json", &save).await;
    let meta = json!({
        "last_sync": now_str(),
        "last_sync_timestamp": now_ts(),
        "playlist_count": save["stats"]["playlist_count"],
        "song_total": save["stats"]["song_total"]
    });
    let _ = write_snapshot(pool, &ciyuanxi_id, "meta.json", &meta).await;
    if ok {
        ctx.ok("同步成功", json!({
            "playlist_count": save["stats"]["playlist_count"],
            "song_total": save["stats"]["song_total"],
            "id_map": id_map
        }))
    } else {
        ctx.err(500, "同步数据写入失败")
    }
}

// ===== v2 协议 =====
// 上传三步与 v1 请求/响应完全一致（合并逻辑共用 sync_merge），仅 action 名不同，
// 用于客户端按版本灰度切换；下载改为服务端 diff（file_sync_v2_download_ops）。

pub async fn file_sync_v2_upload_start(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    file_sync_upload_start(body, ctx, pool).await
}

pub async fn file_sync_v2_upload_chunk(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    file_sync_upload_chunk(body, ctx, pool).await
}

pub async fn file_sync_v2_upload_finish(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    file_sync_upload_finish(body, ctx, pool).await
}

/// v2 下载：客户端上报本地歌单概要（local_playlists），服务端对云端快照
/// 计算最小 ops（create_playlist / add_songs / remove_songs / update_playlist_meta）。
pub async fn file_sync_v2_download_ops(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let mut reports: Vec<sync_diff::LocalPlaylistReport> = Vec::new();
    if let Some(arr) = data.get("local_playlists").and_then(Value::as_array) {
        for item in arr {
            if let Ok(r) = serde_json::from_value::<sync_diff::LocalPlaylistReport>(item.clone()) {
                reports.push(r);
            }
        }
    }
    match read_snapshot(pool, &ciyuanxi_id, "playlists.json").await {
        Ok(v) => {
            let (fixed, changed) = sync_merge::ensure_cloud_ids(v);
            if changed {
                let _ = write_snapshot(pool, &ciyuanxi_id, "playlists.json", &fixed).await;
            }
            let ops = sync_diff::compute_download_ops(&fixed, &reports);
            let stats = sync_diff::snapshot_stats(&fixed);
            let ts = fixed.get("timestamp").and_then(Value::as_i64).unwrap_or(0);
            ctx.ok("获取成功", json!({
                "ops": ops,
                "stats": stats,
                "snapshot_timestamp": ts,
                "deleted_playlist_cloud_ids": ledger_keys(&fixed, "deleted", "cloudId")
            }))
        }
        Err(_) => ctx.ok(
            "暂无同步数据",
            json!({
                "ops": [],
                "stats": { "playlist_count": 0, "song_total": 0 },
                "snapshot_timestamp": 0,
                "deleted_playlist_cloud_ids": []
            }),
        ),
    }
}

pub async fn file_sync_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(pool, &ciyuanxi_id, "playlists.json").await {
        Ok(v) => {
            let (fixed_snapshot, changed) = sync_merge::ensure_cloud_ids(v);
            if changed {
                let _ = write_snapshot(pool, &ciyuanxi_id, "playlists.json", &fixed_snapshot).await;
            }
            let deleted_ids = ledger_keys(&fixed_snapshot, "deleted", "cloudId");
            let mut out = fixed_snapshot;
            out["deleted_playlist_cloud_ids"] = json!(deleted_ids);
            ctx.ok("获取成功", out)
        }
        Err(_) => ctx.ok(
            "暂无同步数据",
            json!({ "playlists": [], "deleted_playlist_cloud_ids": [] }),
        ),
    }
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
    let mut save = json!({
        "version": 4,
        "uploaded_at": now_str(),
        "timestamp": last_ts,
        "stats": {
            "playlist_count": kept.len(),
            "song_total": song_total
        },
        "playlists": kept
    });
    // 保留既有账本，再把本次删除记入广播账本：其他端下载时按 cloudId 裁剪本地副本
    if let Some(d) = existing.get("deleted") {
        save["deleted"] = d.clone();
    }
    let ids: Vec<String> = delset.into_iter().collect();
    ledger_append(&mut save, "deleted", "cloudId", &ids);
    let ok = write_snapshot(pool, &ciyuanxi_id, "playlists.json", &save).await;
    if ok {
        ctx.ok("删除成功", json!({ "deleted": deleted }))
    } else {
        ctx.err(500, "同步数据写入失败")
    }
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
    // 上传一律按 id upsert（云端 = 各端并集 + plugin_sync_delete 显式删除）。
    // 曾经的 is_first「清空后整批替换」语义在多端插件数不对称时会互相覆盖
    // （如移动端 1 个插件的上传把桌面端 31 个清掉），已废弃；字段仅保留兼容忽略。
    let mut save_data = read_snapshot(pool, &ciyuanxi_id, "plugins.json").await.unwrap_or_else(|_| {
        json!({
            "version": 1, "uploaded_at": now_str(), "timestamp": now_ts(),
            "stats": { "plugin_count": 0, "subscription_count": 0 }, "plugins": [], "subscriptions": []
        })
    });
    let mut plugins: Vec<Value> = save_data.get("plugins").and_then(|x| x.as_array()).cloned().unwrap_or_default();
    let pid = plugin.get("id").cloned().unwrap_or(Value::Null);
    let pid_str = plugin
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let mut found = false;
    if !plugin_empty {
        // 重新上传视为要回该插件：按 id 撤销删除广播
        ledger_remove(&mut save_data, "deleted", "id", &[pid_str.clone()]);
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
        // 订阅按 url upsert（上传方条目覆盖同 url 元数据）：云端 = 各端并集，
        // 删除走 plugin_sync_delete 的 subscription_urls 显式删除。整体替换会在
        // 多端订阅集不对称时互相覆盖（同 is_first 插件清空问题），已废弃。
        let mut merged: Vec<Value> = save_data
            .get("subscriptions")
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default();
        let mut sub_urls: Vec<String> = Vec::new();
        for s in subs {
            let url = s.get("url").and_then(|u| u.as_str()).unwrap_or("").trim().to_string();
            if url.is_empty() {
                continue;
            }
            // 重新上传视为要回该订阅：按 url 撤销删除广播
            sub_urls.push(url.clone());
            let pos = merged.iter().position(|p| {
                p.get("url")
                    .and_then(|u| u.as_str())
                    .map(|u| u.trim() == url)
                    .unwrap_or(false)
            });
            match pos {
                Some(i) => merged[i] = s,
                None => merged.push(s),
            }
        }
        ledger_remove(&mut save_data, "deleted_subscriptions", "url", &sub_urls);
        let sub_count = merged.len() as i64;
        save_data["subscriptions"] = json!(merged);
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
        Ok(v) => {
            let mut out = v;
            out["deleted_plugin_ids"] = json!(ledger_keys(&out, "deleted", "id"));
            out["deleted_subscription_urls"] =
                json!(ledger_keys(&out, "deleted_subscriptions", "url"));
            ctx.ok("获取成功", out)
        }
        Err(_) => ctx.ok(
            "暂无同步数据",
            json!({
                "plugins": [],
                "deleted_plugin_ids": [],
                "deleted_subscription_urls": []
            }),
        ),
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
    // 订阅显式删除（url 维度）：配合上传 upsert 的并集语义
    let subdel: std::collections::HashSet<String> = data
        .get("subscription_urls")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    if delset.is_empty() && subdel.is_empty() {
        return ctx.ok(
            "删除成功",
            json!({ "deleted": 0, "plugin_count": 0, "deleted_subscriptions": 0 }),
        );
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
    let subs = existing
        .get("subscriptions")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let sub_before = subs.len();
    let kept_subs: Vec<Value> = subs
        .into_iter()
        .filter(|s| {
            let u = s
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            !subdel.contains(&u)
        })
        .collect();
    let deleted_subs = sub_before - kept_subs.len();
    let mut save = existing;
    save["plugins"] = json!(kept);
    save["stats"]["plugin_count"] = json!(kept.len() as i64);
    save["subscriptions"] = json!(kept_subs);
    save["stats"]["subscription_count"] = json!(kept_subs.len() as i64);
    save["uploaded_at"] = json!(now_str());
    save["timestamp"] = json!(now_ts());
    // 删除进入广播账本：其他端下载时按 id / url 裁剪本地副本
    let del_ids: Vec<String> = delset.into_iter().collect();
    ledger_append(&mut save, "deleted", "id", &del_ids);
    let sub_urls: Vec<String> = subdel.into_iter().collect();
    ledger_append(&mut save, "deleted_subscriptions", "url", &sub_urls);
    if !write_snapshot(pool, &ciyuanxi_id, "plugins.json", &save).await {
        return ctx.err(500, "同步数据写入失败");
    }
    ctx.ok(
        "删除成功",
        json!({ "deleted": deleted, "plugin_count": kept.len() as i64, "deleted_subscriptions": deleted_subs }),
    )
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

// ===== 播放历史云同步 =====
// 客户端（桌面/移动端）上传本地听歌历史（已自行合并去重），云端存
// listen_history.json；下载直接返回该文件，客户端仅补本地缺失条目。

pub async fn history_sync_upload(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let history = match data.get("history") {
        Some(Value::Array(arr)) => arr.clone(),
        _ => return ctx.err(400, "history 格式错误"),
    };
    let count = history.len();
    let save = json!({
        "version": 1,
        "uploaded_at": now_str(),
        "timestamp": now_ts(),
        "history": history
    });
    if !write_snapshot(pool, &ciyuanxi_id, "listen_history.json", &save).await {
        return ctx.err(500, "同步数据写入失败");
    }
    ctx.ok("上传成功", json!({ "history_count": count, "uploaded_at": save["uploaded_at"] }))
}

pub async fn history_sync_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(pool, &ciyuanxi_id, "listen_history.json").await {
        Ok(v) => ctx.ok("获取成功", v),
        Err(_) => ctx.ok("暂无同步数据", json!({ "history": [] })),
    }
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
    // 账本挂在完整快照文档上，读写都取全文档（重新上传按 path 撤销广播）
    let mut doc = read_snapshot(pool, &ciyuanxi_id, "favorites.json")
        .await
        .unwrap_or_else(|_| json!({ "favorites": [] }));
    if merge {
        let existing: Vec<Value> = doc
            .get("favorites")
            .and_then(|x| x.as_array())
            .cloned()
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
            let delset: Vec<String> = del
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
            // 显式删除进入广播账本：其他端下载时按 path 裁剪本地副本
            ledger_append(&mut doc, "deleted", "path", &delset);
            let delset_set: std::collections::HashSet<String> = delset.into_iter().collect();
            by_path.retain(|(k, _)| match k {
                Some(p) => !delset_set.contains(p),
                None => true,
            });
        }
        final_list = by_path.into_iter().map(|(_, it)| it).collect();
        count = final_list.len() as i64;
    } else {
        final_list = favorites.as_array().cloned().unwrap_or_default();
        count = final_list.len() as i64;
        // 全量替换（手动覆盖上传 merge=false）：被清掉的存量 path 记入广播账本，
        // 其他端下载时按账本裁剪本地副本，避免其自动同步把数据合并复活
        let kept: std::collections::HashSet<String> = final_list
            .iter()
            .filter_map(|it| it.get("path").and_then(Value::as_str).map(|s| s.to_string()))
            .collect();
        let cleared: Vec<String> = doc
            .get("favorites")
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|it| it.get("path").and_then(Value::as_str).map(|s| s.to_string()))
            .filter(|p| !p.is_empty() && !kept.contains(p))
            .collect();
        if !cleared.is_empty() {
            ledger_append(&mut doc, "deleted", "path", &cleared);
        }
    }
    let uploaded_paths: Vec<String> = final_list
        .iter()
        .filter_map(|it| {
            it.get("path")
                .and_then(Value::as_str)
                .map(|s| s.to_string())
        })
        .filter(|s| !s.is_empty())
        .collect();
    ledger_remove(&mut doc, "deleted", "path", &uploaded_paths);

    doc["favorites"] = json!(final_list);
    doc["version"] = json!(1);
    doc["uploaded_at"] = json!(now_str());
    doc["timestamp"] = json!(now_ts());
    doc["stats"] = json!({ "song_count": count });
    if !write_snapshot(pool, &ciyuanxi_id, "favorites.json", &doc).await {
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
        Ok(v) => {
            let mut out = v;
            out["deleted_paths"] = json!(ledger_keys(&out, "deleted", "path"));
            ctx.ok("获取成功", out)
        }
        Err(_) => ctx.ok(
            "暂无同步数据",
            json!({ "favorites": [], "deleted_paths": [] }),
        ),
    }
}

// ===== 网盘凭据云同步 =====
// 客户端把本地 AES 密封的网盘账号串（`xy1:<iv>:<密文>`）整包上云，
// 登录弦予账号的其他设备拉取后用各自的本地密钥解密合并。
// 服务端只见密文，不做任何解析。

/// 密封串上限：11 盘多账号 cookie 级凭据实际约几 KB～几十 KB，
/// 1MB 足够宽裕，防恶意刷大字段。
const CLOUD_CREDS_MAX_LEN: usize = 1024 * 1024;

pub async fn cloud_creds_upload(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let sealed = str_of(&data, "sealed").trim().to_string();
    if sealed.is_empty() {
        return ctx.err(400, "缺少凭据数据");
    }
    if sealed.len() > CLOUD_CREDS_MAX_LEN {
        return ctx.err(400, "凭据数据过大");
    }
    let save = json!({
        "version": 1,
        "uploaded_at": now_str(),
        "timestamp": now_ts(),
        "sealed": sealed
    });
    if !write_snapshot(pool, &ciyuanxi_id, "cloud_creds.json", &save).await {
        return ctx.err(500, "同步数据写入失败");
    }
    ctx.ok("上传成功", json!({ "uploaded_at": save["uploaded_at"] }))
}

pub async fn cloud_creds_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数错误");
    }
    match read_snapshot(pool, &ciyuanxi_id, "cloud_creds.json").await {
        Ok(v) => {
            let sealed = v.get("sealed").and_then(Value::as_str).unwrap_or("");
            if sealed.is_empty() {
                ctx.ok("暂无同步数据", json!({ "sealed": null }))
            } else {
                ctx.ok("获取成功", json!({ "sealed": sealed }))
            }
        }
        Err(_) => ctx.ok("暂无同步数据", json!({ "sealed": null })),
    }
}
