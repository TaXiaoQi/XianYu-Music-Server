use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body};

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

pub async fn delete_empty_favorite_playlists(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    let all = sqlx::query("SELECT id, user_id, name, song_count, created_at FROM user_playlists WHERE is_favorite = 1 AND name = '我喜欢的音乐' ORDER BY user_id ASC, created_at ASC, id ASC")
        .fetch_all(pool).await;
    let all = match all {
        Ok(r) => r,
        Err(_) => return err(500, "清理失败"),
    };
    let mut grouped: std::collections::BTreeMap<String, Vec<(i64, i64)>> = std::collections::BTreeMap::new();
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