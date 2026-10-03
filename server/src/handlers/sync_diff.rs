use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};

use super::sync_merge::song_diff_key;

/// v2 下载协议的 diff 计算：服务端根据云端快照与客户端上报的本地歌单
/// 概要，算出最小操作集（ops），客户端按 op 类型应用到本地库。
/// 纯函数，不触碰 DB / 网络。

/// 客户端在 file_sync_v2_download_ops 请求中上报的本地歌单概要。
/// 字段名接受 camelCase（主）与 snake_case（alias 兼容）。
/// name / is_favorite / created_at 属协议字段：服务端当前不做 diff，保留以稳定协议。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct LocalPlaylistReport {
    #[serde(alias = "local_id", default)]
    pub local_id: String,
    #[serde(default)]
    pub cloud_id: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub cloud_cover_url: String,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub source_plugin_id: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub song_hashes: Vec<String>,
}

/// 计算下载 ops。匹配规则：cloudId 优先，本地 id 兜底（消除跨设备重复建单）。
/// - 云端歌单匹配不到任何本地歌单 → create_playlist（全量建单）；
/// - 匹配到 → 按需产出 add_songs（song_hash 不在上报集合的云歌曲，排除云端 tombstone）、
///   remove_songs（云端 tombstone paths）、update_playlist_meta（仅差异字段，
///   与桌面端既有下载合并行为一致：cloudCoverUrl / sourcePluginId / sourceUrl）。
pub fn compute_download_ops(snapshot: &Value, local: &[LocalPlaylistReport]) -> Vec<Value> {
    let Some(cloud_playlists) = snapshot.get("playlists").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut by_cloud: HashMap<&str, &LocalPlaylistReport> = HashMap::new();
    let mut by_local: HashMap<&str, &LocalPlaylistReport> = HashMap::new();
    for r in local {
        by_local.insert(r.local_id.as_str(), r);
        if let Some(cid) = r.cloud_id.as_deref().filter(|s| !s.is_empty()) {
            by_cloud.insert(cid, r);
        }
    }

    let mut ops: Vec<Value> = Vec::new();
    for cloud_pl in cloud_playlists {
        let cloud_id = cloud_pl.get("cloudId").and_then(Value::as_str).unwrap_or("");
        if cloud_id.is_empty() {
            continue;
        }
        let local_id = cloud_pl.get("id").and_then(Value::as_str).unwrap_or("");
        let matched = by_cloud
            .get(cloud_id)
            .copied()
            .or_else(|| by_local.get(local_id).copied());
        let Some(report) = matched else {
            ops.push(json!({ "type": "create_playlist", "playlist": cloud_pl }));
            continue;
        };

        let known: HashSet<&str> = report.song_hashes.iter().map(|s| s.as_str()).collect();
        let tombstones = tombstone_paths(cloud_pl);

        let add: Vec<Value> = cloud_pl
            .get("songs")
            .and_then(Value::as_array)
            .map(|songs| {
                songs
                    .iter()
                    .filter(|s| {
                        let path = s.get("path").and_then(Value::as_str).unwrap_or("");
                        if !path.is_empty() && tombstones.iter().any(|t| t == path) {
                            return false;
                        }
                        !known.contains(song_diff_key(s).as_str())
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        if !add.is_empty() {
            ops.push(json!({ "type": "add_songs", "cloudId": cloud_id, "id": local_id, "songs": add }));
        }

        if !tombstones.is_empty() {
            ops.push(json!({ "type": "remove_songs", "cloudId": cloud_id, "id": local_id, "paths": tombstones }));
        }

        let meta = compute_meta_patch(cloud_pl, report);
        if !meta.is_empty() {
            let mut op = json!({ "type": "update_playlist_meta", "cloudId": cloud_id, "id": local_id });
            for (k, v) in meta {
                op[k] = v;
            }
            ops.push(op);
        }
    }
    ops
}

/// 云端 tombstone（deletedSongPaths）：保序去重。
fn tombstone_paths(cloud_pl: &Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(arr) = cloud_pl.get("deletedSongPaths").and_then(Value::as_array) {
        for v in arr {
            if let Some(s) = v.as_str() {
                if !s.is_empty() && !out.iter().any(|t| t == s) {
                    out.push(s.to_string());
                }
            }
        }
    }
    out
}

/// 与桌面端既有下载合并行为对齐：已有歌单只更新
/// cloudCoverUrl / sourcePluginId / sourceUrl；name / isFavorite / createdAt 不动。
fn compute_meta_patch(cloud_pl: &Value, report: &LocalPlaylistReport) -> Map<String, Value> {
    let mut meta = Map::new();
    // 下载方向：patch 携带云端值下发客户端；云端为空时与今日下载合并一致（不覆盖本地）。
    let cloud_cover = cloud_pl.get("cloudCoverUrl").and_then(Value::as_str).unwrap_or("");
    if !cloud_cover.is_empty() && cloud_cover != report.cloud_cover_url.as_str() {
        meta.insert("cloudCoverUrl".to_string(), json!(cloud_cover));
    }
    for (key, local_v, cloud_key) in [
        ("sourcePluginId", &report.source_plugin_id, "sourcePluginId"),
        ("sourceUrl", &report.source_url, "sourceUrl"),
    ] {
        let cloud_s = cloud_pl
            .get(cloud_key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty());
        if let Some(cs) = cloud_s {
            if Some(cs) != local_v.as_deref() {
                meta.insert(key.to_string(), json!(cs));
            }
        }
    }
    meta
}

/// 快照统计（v2 响应 stats 用）。
pub fn snapshot_stats(snapshot: &Value) -> Value {
    let playlists = snapshot
        .get("playlists")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let song_total: i64 = playlists
        .iter()
        .map(|pl| pl.get("songs").and_then(|s| s.as_array()).map(|a| a.len() as i64).unwrap_or(0))
        .sum();
    json!({ "playlist_count": playlists.len(), "song_total": song_total })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::sync_merge::md5_hex;

    fn report(local_id: &str, cloud_id: Option<&str>, song_hashes: &[&str]) -> LocalPlaylistReport {
        LocalPlaylistReport {
            local_id: local_id.to_string(),
            cloud_id: cloud_id.map(|s| s.to_string()),
            name: String::new(),
            cloud_cover_url: String::new(),
            is_favorite: false,
            created_at: None,
            source_plugin_id: None,
            source_url: None,
            song_hashes: song_hashes.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn create_playlist_for_unknown_cloud_id() {
        let snapshot = json!({"playlists": [
            {"id": "l1", "cloudId": "c1", "name": "云端歌单", "songs": [{"path": "lx://wy/1"}]}
        ]});
        let ops = compute_download_ops(&snapshot, &[report("lx", None, &[])]);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0]["type"], json!("create_playlist"));
        assert_eq!(ops[0]["playlist"]["cloudId"], json!("c1"));
        assert_eq!(ops[0]["playlist"]["name"], json!("云端歌单"));
    }

    #[test]
    fn add_songs_by_hash_with_fallback_key() {
        let snapshot = json!({"playlists": [
            {"id": "l1", "cloudId": "c1", "songs": [
                {"path": "lx://wy/1", "song_hash": "h1"},
                {"path": "lx://wy/2", "song_hash": "h2"},
                // 无 song_hash：回退 md5(path)
                {"path": "lx://wy/3"}
            ]}
        ]});
        // 本地已有 h1 与 md5("lx://wy/3")
        let fallback = md5_hex("lx://wy/3");
        let hashes = ["h1", fallback.as_str()];
        let ops = compute_download_ops(&snapshot, &[report("l1", Some("c1"), &hashes)]);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0]["type"], json!("add_songs"));
        let songs = ops[0]["songs"].as_array().unwrap();
        assert_eq!(songs.len(), 1);
        assert_eq!(songs[0]["song_hash"], json!("h2"));
    }

    #[test]
    fn no_ops_when_in_sync() {
        let snapshot = json!({"playlists": [
            {"id": "l1", "cloudId": "c1", "cloudCoverUrl": "", "songs": [
                {"path": "lx://wy/1", "song_hash": "h1"}
            ]}
        ]});
        let ops = compute_download_ops(&snapshot, &[report("l1", Some("c1"), &["h1"])]);
        assert!(ops.is_empty());
    }

    #[test]
    fn remove_songs_only_contains_tombstone_paths() {
        let snapshot = json!({"playlists": [
            {"id": "l1", "cloudId": "c1", "songs": [{"path": "a", "song_hash": "h1"}], "deletedSongPaths": ["p1", "p2"]}
        ]});
        let ops = compute_download_ops(&snapshot, &[report("l1", Some("c1"), &["h1"])]);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0]["type"], json!("remove_songs"));
        assert_eq!(ops[0]["paths"], json!(["p1", "p2"]));
    }

    #[test]
    fn tombstoned_song_not_added() {
        let snapshot = json!({"playlists": [
            {"id": "l1", "cloudId": "c1", "songs": [
                {"path": "p1", "song_hash": "h1"},
                {"path": "p2", "song_hash": "h2"}
            ], "deletedSongPaths": ["p1"]}
        ]});
        let ops = compute_download_ops(&snapshot, &[report("l1", Some("c1"), &[])]);
        let add = ops.iter().find(|o| o["type"] == json!("add_songs")).unwrap();
        let songs = add["songs"].as_array().unwrap();
        assert_eq!(songs.len(), 1);
        assert_eq!(songs[0]["path"], json!("p2"));
    }

    #[test]
    fn meta_patch_only_diff_fields() {
        let snapshot = json!({"playlists": [
            {"id": "l1", "cloudId": "c1", "name": "A",
             "cloudCoverUrl": "http://cloud/cover.jpg",
             "sourcePluginId": "p1", "sourceUrl": "http://s",
             "isFavorite": true, "createdAt": 123, "songs": []}
        ]});
        let mut rep = report("l1", Some("c1"), &[]);
        rep.cloud_cover_url = "http://local/cover.jpg".to_string();
        rep.source_plugin_id = Some("p1".to_string());
        rep.source_url = Some("http://stale".to_string());
        let ops = compute_download_ops(&snapshot, &[rep]);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0]["type"], json!("update_playlist_meta"));
        assert_eq!(ops[0]["id"], json!("l1"));
        // 下载方向：patch 携带云端值
        assert_eq!(ops[0]["cloudCoverUrl"], json!("http://cloud/cover.jpg"));
        assert_eq!(ops[0]["sourceUrl"], json!("http://s"));
        // sourcePluginId 本地与云端一致 → 不入 patch
        assert!(ops[0].get("sourcePluginId").is_none());
        // name / isFavorite / createdAt 不在 patch 语义内
        assert!(ops[0].get("name").is_none());
        assert!(ops[0].get("isFavorite").is_none());
        assert!(ops[0].get("createdAt").is_none());
    }

    #[test]
    fn meta_patch_skips_empty_cloud_values() {
        // 云端封面/来源为空时与今日下载合并一致：不覆盖本地
        let snapshot = json!({"playlists": [
            {"id": "l1", "cloudId": "c1", "cloudCoverUrl": "", "songs": []}
        ]});
        let mut rep = report("l1", Some("c1"), &[]);
        rep.cloud_cover_url = "http://local/cover.jpg".to_string();
        rep.source_plugin_id = Some("p1".to_string());
        let ops = compute_download_ops(&snapshot, &[rep]);
        assert!(ops.is_empty());
    }

    #[test]
    fn match_by_local_id_when_cloud_id_absent_in_report() {
        let snapshot = json!({"playlists": [
            {"id": "l1", "cloudId": "c1", "songs": [{"path": "lx://wy/1", "song_hash": "h1"}]}
        ]});
        // 本地歌单还没有 cloudId（新设备未上传），但本地 id 与云端存的 id 一致 → 匹配而非建单
        let ops = compute_download_ops(&snapshot, &[report("l1", None, &["h1"])]);
        assert!(ops.is_empty());
    }

    #[test]
    fn empty_snapshot_yields_no_ops() {
        let ops = compute_download_ops(&json!({"playlists": []}), &[report("l1", Some("c1"), &[])]);
        assert!(ops.is_empty());
        let ops = compute_download_ops(&json!({}), &[report("l1", Some("c1"), &[])]);
        assert!(ops.is_empty());
    }

    #[test]
    fn snapshot_stats_counts_playlists_and_songs() {
        let snapshot = json!({"playlists": [
            {"songs": [{"path": "a"}, {"path": "b"}]},
            {"songs": []}
        ]});
        let stats = snapshot_stats(&snapshot);
        assert_eq!(stats["playlist_count"], json!(2));
        assert_eq!(stats["song_total"], json!(2));
    }
}
