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
    if rows.is_empty() {
        return ctx.err(400, "没有分块数据");
    }
    let uploaded = sync_merge::assemble_chunk_playlists(&rows);
    let _ = sqlx::query("DELETE FROM user_sync_chunks WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let do_merge = matches!(data.get("merge"), Some(Value::Bool(true)));
    let mut id_map: Vec<Value> = Vec::new();
    let merged = if do_merge {
        let existing: Vec<Value> = read_snapshot(pool, &ciyuanxi_id, "playlists.json")
            .await
            .ok()
            .and_then(|v| v.get("playlists").cloned())
            .and_then(|x| x.as_array().cloned())
            .unwrap_or_default();
        let delete_ids: Option<Vec<String>> = data
            .get("delete_cloud_ids")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect());
        let (m, im) = sync_merge::merge_into_snapshot(uploaded, existing, delete_ids.as_deref());
        id_map = im;
        m
    } else {
        uploaded
    };
    let save = sync_merge::build_snapshot_doc(merged);
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
                "snapshot_timestamp": ts
            }))
        }
        Err(_) => ctx.ok(
            "暂无同步数据",
            json!({
                "ops": [],
                "stats": { "playlist_count": 0, "song_total": 0 },
                "snapshot_timestamp": 0
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
            ctx.ok("获取成功", fixed_snapshot)
        }
        Err(_) => ctx.ok("暂无同步数据", json!({ "playlists": [] })),
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
