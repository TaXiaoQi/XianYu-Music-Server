use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

/// 获取用户的所有云端歌单（含歌曲）
pub async fn get_user_playlists(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let legacy_user_id = str_of(&data, "user_id").trim().to_string();
    let email = str_of(&data, "email").trim().to_string();
    if ciyuanxi_id.is_empty() && !legacy_user_id.is_empty() {
        if legacy_user_id.chars().all(|c| c.is_ascii_digit()) {
            let r = sqlx::query("SELECT ciyuanxi_id FROM app_users WHERE id = ? LIMIT 1").bind(&legacy_user_id).fetch_optional(pool).await.ok().flatten();
            if let Some(r) = r {
                ciyuanxi_id = r.try_get::<String, _>("ciyuanxi_id").unwrap_or_default();
            }
        } else {
            ciyuanxi_id = legacy_user_id;
        }
    }
    if ciyuanxi_id.is_empty() && !email.is_empty() {
        let r = sqlx::query("SELECT ciyuanxi_id FROM app_users WHERE email = ? LIMIT 1").bind(&email).fetch_optional(pool).await.ok().flatten();
        if let Some(r) = r {
            ciyuanxi_id = r.try_get::<String, _>("ciyuanxi_id").unwrap_or_default();
        }
    }
    if ciyuanxi_id.is_empty() {
        return err(404, "未找到对应用户或该用户无弦予号");
    }
    let user = sqlx::query("SELECT id, username, ciyuanxi_id FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id).fetch_optional(pool).await;
    let Some(user_row) = user.ok().flatten() else {
        return err(404, "用户不存在");
    };
    let uid: i64 = user_row.get("id");
    let username: String = user_row.try_get("username").unwrap_or_default();
    let playlists = sqlx::query("SELECT id, name, description, cover_url, song_count, created_at, updated_at FROM user_playlists WHERE user_id = ? OR user_id = ? ORDER BY updated_at DESC")
        .bind(&ciyuanxi_id).bind(&uid.to_string()).fetch_all(pool).await;

    let mut result: Vec<Value> = Vec::new();
    let rows = playlists.unwrap_or_default();
    for p in &rows {
        let pid: i64 = p.get("id");
        let songs = sqlx::query("SELECT id, song_hash, song_name, singer, album, cover_url, duration, source, original_id, sort_order, created_at FROM user_playlist_songs WHERE playlist_id = ? ORDER BY sort_order ASC, id ASC")
            .bind(pid).fetch_all(pool).await.unwrap_or_default();
        let songs_arr: Vec<Value> = songs.iter().map(|s| json!({
            "id": s.get::<i64, _>("id"),
            "song_name": s.try_get::<String, _>("song_name").unwrap_or_default(),
            "singer": s.try_get::<String, _>("singer").unwrap_or_default(),
            "album": s.try_get::<String, _>("album").unwrap_or_default(),
            "cover_url": s.try_get::<String, _>("cover_url").unwrap_or_default(),
            "duration": s.get::<i64, _>("duration"),
            "source": s.try_get::<String, _>("source").unwrap_or_default(),
            "sort_order": s.get::<i64, _>("sort_order"),
            "created_at": s.try_get::<String, _>("created_at").unwrap_or_default(),
        })).collect();
        result.push(json!({
            "id": pid,
            "name": p.try_get::<String, _>("name").unwrap_or_default(),
            "description": p.try_get::<String, _>("description").unwrap_or_default(),
            "cover_url": p.try_get::<String, _>("cover_url").unwrap_or_default(),
            "song_count": p.get::<i64, _>("song_count"),
            "created_at": p.try_get::<String, _>("created_at").unwrap_or_default(),
            "updated_at": p.try_get::<String, _>("updated_at").unwrap_or_default(),
            "songs": songs_arr,
        }));
    }
    log_operation(pool, ctx, "查看用户歌单", &format!("弦予号:{}", ciyuanxi_id), "").await;
    ok("ok", json!({
        "user": json!({ "id": uid, "username": username, "ciyuanxi_id": ciyuanxi_id }),
        "playlists": result,
    }))
}

/// 后台删除用户的单个云端歌单
pub async fn delete_user_playlist(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let playlist_id = int_of(&data, "playlist_id");
    if playlist_id <= 0 {
        return err(400, "参数错误");
    }
    let pl = sqlx::query("SELECT id, name, user_id, cover_path FROM user_playlists WHERE id = ?").bind(playlist_id).fetch_optional(pool).await.ok().flatten();
    let Some(pl) = pl else {
        return err(404, "歌单不存在");
    };
    let name: String = pl.try_get("name").unwrap_or_default();
    let owner: String = pl.try_get("user_id").unwrap_or_default();
    let cover: String = pl.try_get("cover_path").unwrap_or_default();
    if !cover.is_empty() {
        let abs = std::path::Path::new("uploads").join("playlists").join(&cover);
        if abs.is_file() {
            let _ = std::fs::remove_file(&abs);
        }
    }
    let _ = sqlx::query("DELETE FROM user_playlist_songs WHERE playlist_id = ?").bind(playlist_id).execute(pool).await;
    let _ = sqlx::query("DELETE FROM user_playlists WHERE id = ?").bind(playlist_id).execute(pool).await;
    log_operation(pool, ctx, "后台删除用户歌单", &format!("歌单ID:{}", playlist_id), &format!("名称:{} 用户:{}", name, owner)).await;
    ok("删除成功", Value::Null)
}

/// 一键删除空的"我喜欢的音乐"歌单
pub async fn delete_empty_favorite_playlists(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    let all = sqlx::query("SELECT id, user_id, name, song_count, created_at FROM user_playlists WHERE is_favorite = 1 AND name = '我喜欢的音乐' ORDER BY user_id ASC, created_at ASC, id ASC")
        .fetch_all(pool).await;
    let all = match all {
        Ok(r) => r,
        Err(_) => return err(500, "清理失败"),
    };
    // 按 user_id 分组
    let mut grouped: std::collections::BTreeMap<String, Vec<(i64, i64)>> = std::collections::BTreeMap::new(); // user_id -> [(id, created_ts)]
    for r in &all {
        let uid: String = r.get("user_id");
        let pid: i64 = r.get("id");
        grouped.entry(uid).or_default().push((pid, pid));
    }
    let total_scanned = all.len();
    let mut deleted_ids: Vec<i64> = Vec::new();
    let mut details: Vec<Value> = Vec::new();
    for (user_id, playlists) in grouped {
        let mut real_counts: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
        for (pid, _) in &playlists {
            let c: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_playlist_songs WHERE playlist_id = ?").bind(pid).fetch_one(pool).await.unwrap_or(0);
            real_counts.insert(*pid, c);
        }
        let non_empty: Vec<i64> = playlists.iter().map(|(pid, _)| *pid).filter(|pid| real_counts.get(pid).copied().unwrap_or(0) > 0).collect();
        let keep: i64 = if !non_empty.is_empty() { non_empty[0] } else { playlists[0].0 };
        for (pid, _) in &playlists {
            if *pid == keep {
                continue;
            }
            if real_counts.get(pid).copied().unwrap_or(0) == 0 || non_empty.is_empty() {
                let _ = sqlx::query("DELETE FROM user_playlist_songs WHERE playlist_id = ?").bind(*pid).execute(pool).await;
                let _ = sqlx::query("DELETE FROM user_playlists WHERE id = ?").bind(*pid).execute(pool).await;
                deleted_ids.push(*pid);
            }
        }
        let total = playlists.len();
        let deleted = if non_empty.is_empty() { total - 1 } else { playlists.iter().map(|(pid, _)| *pid).filter(|pid| *pid != keep && real_counts.get(pid).copied().unwrap_or(0) == 0 && !non_empty.is_empty()).count() };
        details.push(json!({ "user_id": user_id, "total": total, "deleted": deleted, "kept_id": keep }));
    }
    log_operation(pool, ctx, "一键删除空的我喜欢的音乐歌单", "全站", &format!("删除 {} 个空歌单", deleted_ids.len())).await;
    ok("清理完成", json!({
        "deleted_count": deleted_ids.len(), "total_scanned": total_scanned, "details": details
    }))
}