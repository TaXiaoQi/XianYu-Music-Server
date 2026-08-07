use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::response::ReqCtx;

/// 校验用户存在且启用
pub async fn user_exists(pool: &MySqlPool, ciyuanxi_id: &str) -> bool {
    let row = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? AND status = 1")
        .bind(ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    row.is_some()
}

pub async fn create_playlist(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let name = str_of(&data, "name").trim().to_string();
    let description = str_of(&data, "description").trim().to_string();
    let cover_url = str_of(&data, "cover_url").trim().to_string();
    let cover_path = str_of(&data, "cover_path").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    if name.is_empty() {
        return ctx.err(400, "歌单名称不能为空");
    }
    if !user_exists(pool, &ciyuanxi_id).await {
        return ctx.err(404, "用户不存在或被禁用");
    }
    let result = sqlx::query(
        "INSERT INTO user_playlists (user_id, name, description, cover_url, cover_path) VALUES (?,?,?,?,?)",
    )
    .bind(&ciyuanxi_id)
    .bind(&name)
    .bind(&description)
    .bind(&cover_url)
    .bind(&cover_path)
    .execute(pool)
    .await;
    match result {
        Ok(r) => {
            let playlist_id = r.last_insert_id();
            ctx.ok(
                "创建成功",
                json!({
                    "playlist_id": playlist_id,
                    "name": name,
                    "description": description,
                    "cover_url": cover_url
                }),
            )
        }
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn get_playlists(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let rows = sqlx::query(
        "SELECT id, name, description, cover_url, song_count, is_favorite, created_at, updated_at
         FROM user_playlists WHERE user_id = ? ORDER BY is_favorite DESC, updated_at DESC",
    )
    .bind(&ciyuanxi_id)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rs) => {
            let mut list = Vec::new();
            for r in rs {
                let id: i64 = r.get("id");
                let name: String = r.get("name");
                let description: String = r.get("description");
                let cover_url: String = r.get("cover_url");
                let song_count: i64 = r.get("song_count");
                let is_favorite: i64 = r.get("is_favorite");
                let created_at: String = r.get("created_at");
                let updated_at: String = r.get("updated_at");
                list.push(json!({
                    "id": id,
                    "name": name,
                    "description": description,
                    "cover_url": cover_url,
                    "song_count": song_count,
                    "is_favorite": is_favorite,
                    "created_at": created_at,
                    "updated_at": updated_at,
                    "source": "cloud"
                }));
            }
            ctx.ok("获取成功", json!({ "playlists": list }))
        }
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn get_or_create_favorite_playlist(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    }
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let row = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? AND status = 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user_row) = row else {
        return ctx.err(404, "用户不存在或被禁用");
    };
    let legacy_user_id: i64 = user_row.get("id");
    let legacy_user_id = legacy_user_id.to_string();

    let fav = sqlx::query(
        "SELECT id, song_count, created_at FROM user_playlists WHERE (user_id = ? OR user_id = ?) AND is_favorite = 1 ORDER BY id ASC",
    )
    .bind(&ciyuanxi_id)
    .bind(&legacy_user_id)
    .fetch_all(pool)
    .await;
    let Ok(favorites) = fav else {
        return ctx.err(500, "服务器错误");
    };
    if favorites.is_empty() {
        let ins = sqlx::query(
            "INSERT INTO user_playlists (user_id, name, description, is_favorite) VALUES (?, '我喜欢的音乐', '', 1)",
        )
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
        return match ins {
            Ok(r) => ctx.ok("ok", json!({ "playlist_id": r.last_insert_id() })),
            Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
        };
    }

    let keep_id: i64 = favorites[0].get("id");
    if favorites.len() > 1 {
        // 合并重复收藏歌单：迁移歌曲后删除
        let mut tx = match pool.begin().await {
            Ok(t) => t,
            Err(e) => return ctx.err(500, &format!("服务器错误: {}", e)),
        };
        for dup in &favorites[1..] {
            let dup_id: i64 = dup.get("id");
            let songs = sqlx::query(
                "SELECT song_hash, song_name, singer, album, cover_url, duration, source, song_url, original_id, sort_order
                 FROM user_playlist_songs WHERE playlist_id = ?",
            )
            .bind(dup_id)
            .fetch_all(&mut *tx)
            .await
            .unwrap_or_default();
            let max_order_row =
                sqlx::query("SELECT COALESCE(MAX(sort_order), 0) AS m FROM user_playlist_songs WHERE playlist_id = ?")
                    .bind(keep_id)
                    .fetch_one(&mut *tx)
                    .await;
            let mut next_order: i64 = match max_order_row {
                Ok(r) => r.try_get("m").unwrap_or(0),
                Err(_) => 0,
            };
            let existing = sqlx::query("SELECT song_hash FROM user_playlist_songs WHERE playlist_id = ?")
                .bind(keep_id)
                .fetch_all(&mut *tx)
                .await
                .unwrap_or_default();
            let mut existing_hashes: std::collections::HashSet<String> =
                existing.iter().filter_map(|r| r.try_get("song_hash").ok()).collect();
            for s in songs {
                let h: String = s.get("song_hash");
                if existing_hashes.contains(&h) {
                    continue;
                }
                existing_hashes.insert(h.clone());
                next_order += 1;
                let _ = sqlx::query(
                    "INSERT INTO user_playlist_songs (playlist_id, user_id, song_hash, song_name, singer, album, cover_url, duration, source, song_url, original_id, sort_order)
                     VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
                )
                .bind(keep_id)
                .bind(&ciyuanxi_id)
                .bind(&h)
                .bind(s.try_get::<String, _>("song_name").unwrap_or_default())
                .bind(s.try_get::<String, _>("singer").unwrap_or_default())
                .bind(s.try_get::<String, _>("album").unwrap_or_default())
                .bind(s.try_get::<String, _>("cover_url").unwrap_or_default())
                .bind(s.try_get::<i64, _>("duration").unwrap_or(0))
                .bind(s.try_get::<String, _>("source").unwrap_or_default())
                .bind(s.try_get::<String, _>("song_url").unwrap_or_default())
                .bind(s.try_get::<String, _>("original_id").unwrap_or_default())
                .bind(next_order)
                .execute(&mut *tx)
                .await
                .ok();
            }
            let _ = sqlx::query("DELETE FROM user_playlist_songs WHERE playlist_id = ?")
                .bind(dup_id)
                .execute(&mut *tx)
                .await;
            let _ = sqlx::query("DELETE FROM user_playlists WHERE id = ?")
                .bind(dup_id)
                .execute(&mut *tx)
                .await;
        }
        let count_row =
            sqlx::query("SELECT COUNT(*) AS c FROM user_playlist_songs WHERE playlist_id = ?")
                .bind(keep_id)
                .fetch_one(&mut *tx)
                .await;
        let real_count: i64 = match &count_row {
            Ok(r) => r.try_get("c").unwrap_or(0),
            Err(_) => 0,
        };
        let _ = sqlx::query(
            "UPDATE user_playlists SET song_count = ?, user_id = ?, name = '我喜欢的音乐', is_favorite = 1, updated_at = NOW() WHERE id = ?",
        )
        .bind(real_count)
        .bind(&ciyuanxi_id)
        .bind(keep_id)
        .execute(&mut *tx)
        .await;
        if let Err(e) = tx.commit().await {
            return ctx.err(500, &format!("服务器错误: {}", e));
        }
        ctx.ok("ok", json!({ "playlist_id": keep_id }))
    } else {
        ctx.ok("ok", json!({ "playlist_id": keep_id }))
    }
}

pub async fn check_song_in_playlist(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    }
    let playlist_id = int_of(&data, "playlist_id");
    let song_hash = str_of(&data, "song_hash").trim().to_string();
    if ciyuanxi_id.is_empty() || playlist_id <= 0 || song_hash.is_empty() {
        return ctx.err(400, "参数错误");
    }
    let row = sqlx::query("SELECT 1 FROM user_playlist_songs WHERE playlist_id = ? AND song_hash = ? LIMIT 1")
        .bind(playlist_id)
        .bind(&song_hash)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let in_playlist = if row.is_some() { 1 } else { 0 };
    ctx.ok("ok", json!({ "in_playlist": in_playlist }))
}

pub async fn get_playlist_detail(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let playlist_id = int_of(&data, "playlist_id");
    if ciyuanxi_id.is_empty() || playlist_id <= 0 {
        return ctx.err(400, "参数错误");
    }
    let row = sqlx::query(
        "SELECT id, name, description, cover_url, song_count, created_at, updated_at FROM user_playlists WHERE id = ? AND user_id = ?",
    )
    .bind(playlist_id)
    .bind(&ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let Some(pl) = row else {
        return ctx.err(404, "歌单不存在");
    };
    let song_rows = sqlx::query(
        "SELECT id, song_hash, song_name, singer, album, cover_url, duration, source, song_url, original_id, sort_order
         FROM user_playlist_songs WHERE playlist_id = ? ORDER BY sort_order ASC, id ASC",
    )
    .bind(playlist_id)
    .fetch_all(pool)
    .await;
    let mut songs = Vec::new();
    if let Ok(rs) = song_rows {
        for s in rs {
            songs.push(json!({
                "id": s.try_get::<i64,_>("id").unwrap_or(0),
                "song_hash": s.try_get::<String,_>("song_hash").unwrap_or_default(),
                "songName": s.try_get::<String,_>("song_name").unwrap_or_default(),
                "singer": s.try_get::<String,_>("singer").unwrap_or_default(),
                "albumName": s.try_get::<String,_>("album").unwrap_or_default(),
                "cover": s.try_get::<String,_>("cover_url").unwrap_or_default(),
                "duration": s.try_get::<i64,_>("duration").unwrap_or(0),
                "source": s.try_get::<String,_>("source").unwrap_or_default(),
                "songUrl": s.try_get::<String,_>("song_url").unwrap_or_default(),
                "originalId": s.try_get::<String,_>("original_id").unwrap_or_default(),
                "sort_order": s.try_get::<i64,_>("sort_order").unwrap_or(0)
            }));
        }
    }
    ctx.ok(
        "获取成功",
        json!({
            "playlist": {
                "id": pl.try_get::<i64,_>("id").unwrap_or(0),
                "name": pl.try_get::<String,_>("name").unwrap_or_default(),
                "description": pl.try_get::<String,_>("description").unwrap_or_default(),
                "cover_url": pl.try_get::<String,_>("cover_url").unwrap_or_default(),
                "song_count": pl.try_get::<i64,_>("song_count").unwrap_or(0),
                "created_at": pl.try_get::<String,_>("created_at").unwrap_or_default(),
                "updated_at": pl.try_get::<String,_>("updated_at").unwrap_or_default(),
                "source": "cloud"
            },
            "songs": songs
        }),
    )
}

pub async fn update_playlist(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let playlist_id = int_of(&data, "playlist_id");
    let name = str_of(&data, "name").trim().to_string();
    let description = str_of(&data, "description").trim().to_string();
    let cover_url = str_of(&data, "cover_url").trim().to_string();
    let cover_path = str_of(&data, "cover_path").trim().to_string();
    if ciyuanxi_id.is_empty() || playlist_id <= 0 {
        return ctx.err(400, "参数错误");
    }
    let row = sqlx::query("SELECT id, cover_path FROM user_playlists WHERE id = ? AND user_id = ?")
        .bind(playlist_id)
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if row.is_none() {
        return ctx.err(404, "歌单不存在");
    }
    let mut updates: Vec<&str> = Vec::new();
    let mut params: Vec<Value> = Vec::new();
    if !name.is_empty() {
        updates.push("name = ?");
        params.push(json!(name));
    }
    if !description.is_empty() {
        updates.push("description = ?");
        params.push(json!(description));
    }
    if !cover_url.is_empty() {
        updates.push("cover_url = ?");
        params.push(json!(cover_url));
        updates.push("cover_path = ?");
        params.push(json!(cover_path));
        let old_path: String = row.as_ref().unwrap().try_get("cover_path").unwrap_or_default();
        if !old_path.is_empty() {
            let abs = std::path::Path::new("uploads/playlists").join(&old_path);
            let _ = std::fs::remove_file(abs);
        }
    }
    if updates.is_empty() {
        return ctx.err(400, "没有需要更新的字段");
    }
    let sql = format!("UPDATE user_playlists SET {} WHERE id = ? AND user_id = ?", updates.join(", "));
    let mut q = sqlx::query(&sql);
    for p in &params {
        match p {
            Value::String(s) => q = q.bind(s.clone()),
            _ => {}
        }
    }
    let result = q.bind(playlist_id).bind(&ciyuanxi_id).execute(pool).await;
    match result {
        Ok(_) => ctx.ok_empty("更新成功"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn delete_playlist(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let playlist_id = int_of(&data, "playlist_id");
    if ciyuanxi_id.is_empty() || playlist_id <= 0 {
        return ctx.err(400, "参数错误");
    }
    let row = sqlx::query("SELECT id, cover_path FROM user_playlists WHERE id = ? AND user_id = ?")
        .bind(playlist_id)
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if row.is_none() {
        return ctx.err(404, "歌单不存在");
    }
    let cover_path: String = row.as_ref().unwrap().try_get("cover_path").unwrap_or_default();
    if !cover_path.is_empty() {
        let abs = std::path::Path::new("uploads/playlists").join(&cover_path);
        let _ = std::fs::remove_file(abs);
    }
    let _ = sqlx::query("DELETE FROM user_playlist_songs WHERE playlist_id = ?")
        .bind(playlist_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_playlists WHERE id = ? AND user_id = ?")
        .bind(playlist_id)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    ctx.ok_empty("删除成功")
}

pub async fn add_song_to_playlist(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let playlist_id = int_of(&data, "playlist_id");
    let song = data.get("song").cloned().unwrap_or(Value::Null);
    if ciyuanxi_id.is_empty() || playlist_id <= 0 {
        return ctx.err(400, "参数错误");
    }
    if song.is_null() || !song.is_object() {
        return ctx.err(400, "歌曲信息不能为空");
    }
    let row = sqlx::query("SELECT id, song_count FROM user_playlists WHERE id = ? AND user_id = ?")
        .bind(playlist_id)
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(pl) = row else {
        return ctx.err(404, "歌单不存在");
    };
    let mut song_hash = str_of(&song, "song_hash").trim().to_string();
    if song_hash.is_empty() {
        let key = format!(
            "{}|{}|{}",
            str_of(&song, "songName"),
            str_of(&song, "singer"),
            str_of(&song, "source")
        );
        song_hash = crate::sign::md5_hex(key.as_bytes());
    }
    let dup = sqlx::query("SELECT id FROM user_playlist_songs WHERE playlist_id = ? AND song_hash = ?")
        .bind(playlist_id)
        .bind(&song_hash)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if dup.is_some() {
        return ctx.ok("歌曲已在歌单中", json!({ "duplicate": true }));
    }
    let max_row =
        sqlx::query("SELECT COALESCE(MAX(sort_order), 0) + 1 AS n FROM user_playlist_songs WHERE playlist_id = ?")
            .bind(playlist_id)
            .fetch_one(pool)
            .await;
    let next_order: i64 = match max_row {
        Ok(r) => r.try_get("n").unwrap_or(1),
        Err(_) => 1,
    };
    let _ = sqlx::query(
        "INSERT INTO user_playlist_songs (playlist_id, user_id, song_hash, song_name, singer, album, cover_url, duration, source, song_url, original_id, sort_order)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(playlist_id)
    .bind(&ciyuanxi_id)
    .bind(&song_hash)
    .bind(str_of(&song, "songName").trim())
    .bind(str_of(&song, "singer").trim())
    .bind(str_of(&song, "albumName").trim())
    .bind(str_of(&song, "cover").trim())
    .bind(int_of(&song, "duration"))
    .bind(str_of(&song, "source").trim())
    .bind(str_of(&song, "songUrl").trim())
    .bind(str_of(&song, "originalId").trim())
    .bind(next_order)
    .execute(pool)
    .await;
    let old_count: i64 = pl.try_get("song_count").unwrap_or(0);
    let new_count = old_count + 1;
    let _ = sqlx::query("UPDATE user_playlists SET song_count = ?, updated_at = NOW() WHERE id = ?")
        .bind(new_count)
        .bind(playlist_id)
        .execute(pool)
        .await;
    ctx.ok("添加成功", json!({ "song_count": new_count }))
}

fn song_extract(song: &Value) -> String {
    let mut h = str_of(song, "song_hash").trim().to_string();
    if h.is_empty() {
        let n = {
            let a = str_of(song, "songName");
            if a.is_empty() {
                str_of(song, "song_name")
            } else {
                a
            }
        };
        let key = format!("{}|{}|{}", n, str_of(song, "singer"), str_of(song, "source"));
        h = crate::sign::md5_hex(key.as_bytes());
    }
    h
}

async fn batch_add_songs_inner(
    body: &str,
    ctx: &ReqCtx,
    pool: &MySqlPool,
    cap: usize,
    msg: &str,
) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let playlist_id = int_of(&data, "playlist_id");
    let songs = data.get("songs").cloned().unwrap_or_else(|| json!([]));
    if ciyuanxi_id.is_empty() || playlist_id <= 0 {
        return ctx.err(400, "参数错误");
    }
    if !songs.is_array() || songs.as_array().unwrap().is_empty() {
        return ctx.err(400, "歌曲列表不能为空");
    }
    let row = sqlx::query("SELECT id, song_count FROM user_playlists WHERE id = ? AND user_id = ?")
        .bind(playlist_id)
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if row.is_none() {
        return ctx.err(404, "歌单不存在");
    }
    let total = songs.as_array().unwrap().len();
    let mut items: Vec<&Value> = songs.as_array().unwrap().iter().collect();
    if items.len() > cap {
        items.truncate(cap);
    }
    let max_row = sqlx::query("SELECT COALESCE(MAX(sort_order), 0) AS n FROM user_playlist_songs WHERE playlist_id = ?")
        .bind(playlist_id)
        .fetch_one(pool)
        .await;
    let mut next_order: i64 = match max_row {
        Ok(r) => r.try_get("n").unwrap_or(0),
        Err(_) => 0,
    };
    let mut added = 0i64;
    let mut duplicates = 0i64;

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return ctx.err(500, &format!("服务器错误: {}", e)),
    };
    for song in items {
        if !song.is_object() {
            continue;
        }
        let song_hash = song_extract(song);
        let dup = sqlx::query("SELECT id FROM user_playlist_songs WHERE playlist_id = ? AND song_hash = ?")
            .bind(playlist_id)
            .bind(&song_hash)
            .fetch_optional(&mut *tx)
            .await
            .ok()
            .flatten();
        if dup.is_some() {
            duplicates += 1;
            continue;
        }
        next_order += 1;
        let _ = sqlx::query(
            "INSERT INTO user_playlist_songs (playlist_id, user_id, song_hash, song_name, singer, album, cover_url, duration, source, song_url, original_id, sort_order)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(playlist_id)
        .bind(&ciyuanxi_id)
        .bind(&song_hash)
        .bind(song_get_str(song, "songName", "song_name"))
        .bind(str_of(song, "singer").trim())
        .bind(song_get_str(song, "albumName", "album"))
        .bind(song_get_str(song, "cover", "cover_url"))
        .bind(int_of(song, "duration"))
        .bind(str_of(song, "source").trim())
        .bind(song_get_str(song, "songUrl", "song_url"))
        .bind(song_get_str(song, "originalId", "original_id"))
        .bind(next_order)
        .execute(&mut *tx)
        .await
        .ok();
        added += 1;
    }
    if added > 0 {
        let _ = sqlx::query("UPDATE user_playlists SET song_count = song_count + ?, updated_at = NOW() WHERE id = ?")
            .bind(added)
            .bind(playlist_id)
            .execute(&mut *tx)
            .await;
    }
    match tx.commit().await {
        Ok(_) => ctx.ok(
            msg,
            json!({ "added": added, "duplicates": duplicates, "total": total }),
        ),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

fn song_get_str(v: &Value, a: &str, b: &str) -> String {
    let x = str_of(v, a);
    if x.is_empty() {
        str_of(v, b).trim().to_string()
    } else {
        x.trim().to_string()
    }
}

pub async fn batch_add_songs(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    batch_add_songs_inner(body, &ctx, pool, 100, "批量添加成功").await
}

pub async fn batch_add_songs_large(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    batch_add_songs_inner(body, &ctx, pool, 1000, "超大歌单批量添加成功").await
}

pub async fn remove_song_from_playlist(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let playlist_id = int_of(&data, "playlist_id");
    let song_id = int_of(&data, "song_id");
    if ciyuanxi_id.is_empty() || playlist_id <= 0 || song_id <= 0 {
        return ctx.err(400, "参数错误");
    }
    let row = sqlx::query("SELECT id, song_count FROM user_playlists WHERE id = ? AND user_id = ?")
        .bind(playlist_id)
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(pl) = row else {
        return ctx.err(404, "歌单不存在");
    };
    let old_count: i64 = pl.try_get("song_count").unwrap_or(0);
    let result = sqlx::query("DELETE FROM user_playlist_songs WHERE id = ? AND playlist_id = ?")
        .bind(song_id)
        .bind(playlist_id)
        .execute(pool)
        .await;
    let new_count = match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                let nc = std::cmp::max(0, old_count - 1);
                let _ = sqlx::query("UPDATE user_playlists SET song_count = ?, updated_at = NOW() WHERE id = ?")
                    .bind(nc)
                    .bind(playlist_id)
                    .execute(pool)
                    .await;
                nc
            } else {
                old_count
            }
        }
        Err(e) => return ctx.err(500, &format!("服务器错误: {}", e)),
    };
    ctx.ok("删除成功", json!({ "song_count": new_count }))
}