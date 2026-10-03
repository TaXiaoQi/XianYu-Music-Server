use serde_json::{json, Value};

use super::sync_store::{now_str, now_ts};

/// 歌单同步的纯合并逻辑：从 file_sync_upload_finish 中抽出，
/// 旧 v1 action 与新 v2 action 共用。全部函数不触碰 DB / 网络。

pub(crate) fn md5_hex(input: &str) -> String {
    format!("{:x}", md5::compute(input.as_bytes()))
}

/// 把分块行（DB 读出的 (chunk_index, content)）拼回歌单条目全集，
/// 并按歌单本地 id 去重合并：同 id 的 songs 跨 chunk 顺序拼接。
pub fn assemble_chunk_playlists(rows: &[(i64, String)]) -> Vec<Value> {
    let mut all_playlists: Vec<Value> = Vec::new();
    for (_, content) in rows {
        if let Ok(chunk) = serde_json::from_str::<Value>(content) {
            if let Some(items) = chunk.get("chunk_data").and_then(|x| x.as_array()) {
                all_playlists.extend(items.clone());
            }
        }
    }
    dedupe_playlists_by_id(all_playlists)
}

pub fn dedupe_playlists_by_id(items: Vec<Value>) -> Vec<Value> {
    let mut map: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    for pl in &items {
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
    map.into_values().collect()
}

/// 上传歌单与云端快照合并：
/// - cloudId 继承：上传歌单无 cloudId 时按本地 id 回溯云端旧条目；
/// - 新歌单分配探针 id `c{ts}{n}`（同批次内递增保证唯一）；
/// - tombstone 并集再排除本次重新上传的 path（重新上传视为恢复）；
/// - 按 delete_cloud_ids 移除云端歌单。
/// 返回 (合并后的歌单全集, 本地id → cloudId 映射)。
pub fn merge_into_snapshot(
    uploaded: Vec<Value>,
    existing: Vec<Value>,
    delete_cloud_ids: Option<&[String]>,
) -> (Vec<Value>, Vec<Value>) {
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
    let ts = now_ts();
    let mut id_map: Vec<Value> = Vec::new();
    for mut pl in uploaded {
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
            key = Some(format!("c{}{}", ts, probe));
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
    if let Some(delset_ids) = delete_cloud_ids {
        let delset: std::collections::HashSet<&String> = delset_ids.iter().collect();
        by_cloud.retain(|(k, _)| match k {
            Some(c) => !delset.contains(c),
            None => true,
        });
    }
    let merged = by_cloud.into_iter().map(|(_, it)| it).collect();
    (merged, id_map)
}

/// 用合并结果构造 playlists.json 快照文档。
pub fn build_snapshot_doc(playlists: Vec<Value>) -> Value {
    let song_total: i64 = playlists
        .iter()
        .map(|pl| pl.get("songs").and_then(|s| s.as_array()).map(|a| a.len() as i64).unwrap_or(0))
        .sum();
    json!({
        "version": 4,
        "uploaded_at": now_str(),
        "timestamp": now_ts(),
        "stats": {
            "playlist_count": playlists.len(),
            "song_total": song_total
        },
        "playlists": playlists
    })
}

/// 补齐快照中缺失 cloudId 的歌单（历史上传可能不带），返回 (修复后快照, 是否有修改)。
pub fn ensure_cloud_ids(snapshot: Value) -> (Value, bool) {
    let mut changed = false;
    let Some(playlists) = snapshot.get("playlists").cloned() else {
        return (snapshot, changed);
    };
    let Some(playlists) = playlists.as_array() else {
        return (snapshot, changed);
    };
    let mut probe: i64 = 0;
    let ts = now_ts();
    let mut out = Vec::with_capacity(playlists.len());
    for mut pl in playlists.clone() {
        let has_cloud_id = pl
            .get("cloudId")
            .and_then(|c| c.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        if !has_cloud_id {
            pl["cloudId"] = json!(format!("c{}{}", ts, probe));
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

/// 歌曲在下载 diff 中的比较键。
/// 优先使用客户端上传时生成并随 payload 存储的 song_hash；
/// 缺失时回退计算，算法与客户端 playlistSyncSong.generateSongHash 逐字对齐：
/// - 在线歌曲（source_type=remote/plugin 或 path 前缀 remote:// lx:// plugin:// http:// https://）
///   且有 path：md5(path)
/// - 其余：md5("{title||name}|{artist}|local")
pub(crate) fn song_diff_key(song: &Value) -> String {
    if let Some(h) = song.get("song_hash").and_then(Value::as_str) {
        if !h.is_empty() {
            return h.to_string();
        }
    }
    let path = song.get("path").and_then(Value::as_str).unwrap_or("");
    if !path.is_empty() && fallback_is_online(song) {
        return md5_hex(path);
    }
    let name = first_non_empty_str(song, &["title", "name"]);
    let artist = song.get("artist").and_then(Value::as_str).unwrap_or("");
    md5_hex(&format!("{}|{}|local", name, artist))
}

fn first_non_empty_str<'a>(song: &'a Value, keys: &[&str]) -> &'a str {
    for k in keys {
        if let Some(s) = song.get(*k).and_then(Value::as_str) {
            if !s.is_empty() {
                return s;
            }
        }
    }
    ""
}

fn fallback_is_online(song: &Value) -> bool {
    match song.get("source_type").and_then(Value::as_str) {
        Some("remote") | Some("plugin") => return true,
        _ => {}
    }
    let path = song.get("path").and_then(Value::as_str).unwrap_or("");
    path.starts_with("remote://")
        || path.starts_with("lx://")
        || path.starts_with("plugin://")
        || path.starts_with("http://")
        || path.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn md5_matches_known_vectors() {
        assert_eq!(md5_hex("abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(md5_hex(""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn song_diff_key_uses_client_hash_when_present() {
        let song = json!({"path": "lx://wy/1", "song_hash": "client-hash"});
        assert_eq!(song_diff_key(&song), "client-hash");
    }

    #[test]
    fn song_diff_key_online_falls_back_to_path_md5() {
        let song = json!({"path": "lx://wy/1", "syncType": "online"});
        assert_eq!(song_diff_key(&song), md5_hex("lx://wy/1"));
        // source_type=remote 时 remote:// 前缀同样视为在线
        let song2 = json!({"source_type": "remote", "path": "remote://a"});
        assert_eq!(song_diff_key(&song2), md5_hex("remote://a"));
        // 无 source_type 但 path 是 https 也算在线
        let song3 = json!({"path": "https://example.com/a.mp3"});
        assert_eq!(song_diff_key(&song3), md5_hex("https://example.com/a.mp3"));
    }

    #[test]
    fn song_diff_key_local_falls_back_to_name_artist() {
        let song = json!({"path": "C:/m/f.mp3", "title": "Song", "name": "Song", "artist": "A"});
        assert_eq!(song_diff_key(&song), md5_hex("Song|A|local"));
        // title 为空时回退 name
        let song2 = json!({"path": "C:/m/f.mp3", "title": "", "name": "Song2", "artist": ""});
        assert_eq!(song_diff_key(&song2), md5_hex("Song2||local"));
    }

    #[test]
    fn dedupe_playlists_across_chunks() {
        let items = vec![
            json!({"id": "l1", "name": "A", "songs": [{"path": "p1"}]}),
            json!({"id": "l1", "name": "A", "songs": [{"path": "p2"}]}),
            json!({"id": "l2", "name": "B", "songs": []}),
        ];
        let merged = dedupe_playlists_by_id(items);
        assert_eq!(merged.len(), 2);
        let l1 = merged.iter().find(|p| p["id"] == "l1").unwrap();
        assert_eq!(l1["songs"].as_array().unwrap().len(), 2);
        let l2 = merged.iter().find(|p| p["id"] == "l2").unwrap();
        assert_eq!(l2["songs"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn merge_inherits_cloud_id_from_existing() {
        let existing = vec![json!({"id": "l1", "cloudId": "c1", "name": "A", "songs": []})];
        let uploaded = vec![json!({"id": "l1", "name": "A-new", "songs": []})];
        let (merged, id_map) = merge_into_snapshot(uploaded, existing, None);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0]["cloudId"], json!("c1"));
        assert_eq!(merged[0]["name"], json!("A-new"));
        assert_eq!(id_map, vec![json!({"id": "l1", "cloudId": "c1"})]);
    }

    #[test]
    fn merge_assigns_unique_probe_ids() {
        let uploaded = vec![
            json!({"id": "l1", "name": "A", "songs": []}),
            json!({"id": "l2", "name": "B", "songs": []}),
        ];
        let (merged, id_map) = merge_into_snapshot(uploaded, vec![], None);
        assert_eq!(merged.len(), 2);
        let c1 = id_map[0]["cloudId"].as_str().unwrap();
        let c2 = id_map[1]["cloudId"].as_str().unwrap();
        assert_ne!(c1, c2);
        assert!(c1.starts_with('c'));
    }

    #[test]
    fn merge_tombstone_union_minus_uploaded() {
        let existing = vec![json!({
            "id": "l1", "cloudId": "c1", "songs": [{"path": "a"}, {"path": "b"}],
            "deletedSongPaths": ["a", "x"]
        })];
        let uploaded = vec![json!({
            "id": "l1", "cloudId": "c1", "songs": [{"path": "b"}],
            "deletedSongPaths": ["a"]
        })];
        let (merged, _) = merge_into_snapshot(uploaded, existing, None);
        let pl = &merged[0];
        // tombstone = {a,x} ∪ {a} − {b} = {a,x}；songs 中移除 a,x
        let deleted: Vec<&str> = pl["deletedSongPaths"].as_array().unwrap()
            .iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(deleted, vec!["a", "x"]);
        let songs: Vec<&str> = pl["songs"].as_array().unwrap()
            .iter().filter_map(|s| s["path"].as_str()).collect();
        assert_eq!(songs, vec!["b"]);
    }

    #[test]
    fn merge_clears_tombstone_when_empty() {
        let existing = vec![json!({
            "id": "l1", "cloudId": "c1", "songs": [{"path": "a"}],
            "deletedSongPaths": ["x"]
        })];
        // 重新上传 x：视为恢复，tombstone 清空
        let uploaded = vec![json!({
            "id": "l1", "cloudId": "c1", "songs": [{"path": "a"}, {"path": "x"}]
        })];
        let (merged, _) = merge_into_snapshot(uploaded, existing, None);
        assert!(merged[0].get("deletedSongPaths").is_none());
        assert_eq!(merged[0]["songs"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn merge_delete_cloud_ids_removes_playlists() {
        let existing = vec![
            json!({"id": "l1", "cloudId": "c1", "songs": []}),
            json!({"id": "l2", "cloudId": "c2", "songs": []}),
        ];
        let uploaded: Vec<Value> = vec![];
        let (merged, _) = merge_into_snapshot(uploaded, existing, Some(&["c1".to_string()]));
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0]["cloudId"], json!("c2"));
    }

    #[test]
    fn ensure_cloud_ids_fills_missing() {
        let snapshot = json!({"version": 4, "playlists": [
            {"id": "l1", "cloudId": "c1", "songs": []},
            {"id": "l2", "songs": []}
        ]});
        let (fixed, changed) = ensure_cloud_ids(snapshot);
        assert!(changed);
        let arr = fixed["playlists"].as_array().unwrap();
        assert_eq!(arr[0]["cloudId"], json!("c1"));
        assert!(!arr[1]["cloudId"].as_str().unwrap().is_empty());
    }
}
