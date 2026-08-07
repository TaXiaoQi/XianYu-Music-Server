use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

const SETTINGS_FIELDS: [&str; 8] = [
    "stream_cache_enabled",
    "startup_play_enabled",
    "bluetooth_lyric_enabled",
    "download_lyric_enabled",
    "download_cover_enabled",
    "download_artist_enabled",
    "search_board_enabled",
    "page_animation_enabled",
];

fn extract_id(data: &Value) -> String {
    let ciyuanxi_id = str_of(data, "ciyuanxi_id");
    if !ciyuanxi_id.is_empty() {
        return ciyuanxi_id;
    }
    str_of(data, "user_id")
}

pub async fn get_user_info(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let user = sqlx::query("SELECT * FROM app_users WHERE ciyuanxi_id = ? OR id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let email: String = user.get("email");
    let role = crate::handlers::helpers::resolve_role_by_email(pool, &email).await;
    let payload = json!({
        "user_id": user.get::<i64,_>("id"),
        "username": user.get::<String,_>("username"),
        "email": email,
        "role": role,
        "avatar_url": user.get::<String,_>("avatar_url"),
        "ciyuanxi_id": user.get::<String,_>("ciyuanxi_id"),
    });
    ctx.json(200, "ok", Some(payload))
}

pub async fn get_user_settings(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let _ = sqlx::query("INSERT INTO user_settings (ciyuanxi_id) VALUES (?) ON DUPLICATE KEY UPDATE ciyuanxi_id = ciyuanxi_id")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let row = sqlx::query("SELECT * FROM user_settings WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(row) = row else {
        return ctx.json(200, "ok", Some(default_settings()));
    };
    let payload = json!({
        "stream_cache_enabled": row.get::<i64,_>("stream_cache_enabled"),
        "startup_play_enabled": row.get::<i64,_>("startup_play_enabled"),
        "bluetooth_lyric_enabled": row.get::<i64,_>("bluetooth_lyric_enabled"),
        "download_lyric_enabled": row.get::<i64,_>("download_lyric_enabled"),
        "download_cover_enabled": row.get::<i64,_>("download_cover_enabled"),
        "download_artist_enabled": row.get::<i64,_>("download_artist_enabled"),
        "search_board_enabled": row.get::<i64,_>("search_board_enabled"),
        "page_animation_enabled": row.get::<i64,_>("page_animation_enabled"),
        "default_quality": row.get::<String,_>("default_quality"),
    });
    ctx.json(200, "ok", Some(payload))
}

fn default_settings() -> Value {
    json!({
        "stream_cache_enabled": 1,
        "startup_play_enabled": 0,
        "bluetooth_lyric_enabled": 0,
        "download_lyric_enabled": 1,
        "download_cover_enabled": 1,
        "download_artist_enabled": 0,
        "search_board_enabled": 1,
        "page_animation_enabled": 1,
        "default_quality": "standard"
    })
}

pub async fn update_user_settings(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let _ = sqlx::query("INSERT INTO user_settings (ciyuanxi_id) VALUES (?) ON DUPLICATE KEY UPDATE ciyuanxi_id = ciyuanxi_id")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;

    let mut sets = vec![];
    for field in SETTINGS_FIELDS.iter() {
        if let Some(v) = data.get(*field) {
            let val = async_of_i64(v);
            sets.push(format!("`{}` = {}", field, val));
        }
    }
    if let Some(v) = data.get("default_quality") {
        if let Some(q) = v.as_str() {
            let valid = [
                "low", "standard", "high", "super", "ciyuanxi_master", "zhen_master",
                "panorama_master", "ai_master", "exclusive_ai_master",
            ];
            if valid.contains(&q) {
                sets.push(format!("`default_quality` = '{}'", q));
            }
        }
    }
    if sets.is_empty() {
        return ctx.err(400, "没有需要更新的字段");
    }
    let sql = format!("UPDATE user_settings SET {} WHERE ciyuanxi_id = ?", sets.join(", "));
    let result = sqlx::query(&sql).bind(&ciyuanxi_id).execute(pool).await;
    match result {
        Ok(_) => ctx.ok_empty("ok"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

fn async_of_i64(v: &Value) -> i64 {
    match v {
        Value::Number(n) => {
            if n.as_i64().unwrap_or(0) != 0 {
                1
            } else {
                0
            }
        }
        Value::String(s) => {
            if s == "1" || s.eq_ignore_ascii_case("true") {
                1
            } else {
                0
            }
        }
        Value::Bool(b) => {
            if *b {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

pub async fn update_profile(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let user = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let user_id: i64 = user.get("id");

    let mut sets: Vec<String> = vec![];
    if data.get("nickname").and_then(|v| v.as_str()).is_some() {
        let v = str_of(&data, "nickname");
        sets.push(format!("username = '{}'", v));
    }
    if data.get("avatar_url").and_then(|v| v.as_str()).is_some() {
        let v = str_of(&data, "avatar_url");
        sets.push(format!("avatar_url = '{}'", sql_escape(&v)));
    }
    if sets.is_empty() {
        return ctx.err(400, "没有需要更新的字段");
    }
    let sql = format!("UPDATE app_users SET {} WHERE id = ?", sets.join(", "));
    let result = sqlx::query(&sql).bind(user_id).execute(pool).await;
    match result {
        Ok(_) => ctx.ok_empty("更新成功"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

fn sql_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

pub async fn check_username(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let username = str_of(&data, "username").trim().to_string();
    let exists = if username.is_empty() {
        false
    } else {
        sqlx::query("SELECT id FROM app_users WHERE username = ? LIMIT 1")
            .bind(&username)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .is_some()
    };
    ctx.json(200, "ok", Some(json!({ "available": !exists })))
}

pub async fn change_password(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let old_password = str_of(&data, "old_password");
    let new_password = str_of(&data, "new_password");
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    if new_password.len() < 6 {
        return ctx.err(400, "新密码长度不能少于6位");
    }
    let user = sqlx::query("SELECT * FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let stored: String = user.get("password");
    if !bcrypt::verify(&old_password, &stored).unwrap_or(false) {
        return ctx.err(400, "原密码错误");
    }
    let hashed = match bcrypt::hash(&new_password, 10) {
        Ok(h) => h,
        Err(_) => return ctx.err(500, "密码加密失败"),
    };
    let uid: i64 = user.get("id");
    let _ = sqlx::query("UPDATE app_users SET password = ? WHERE id = ?")
        .bind(&hashed)
        .bind(uid)
        .execute(pool)
        .await;
    ctx.ok_empty("密码修改成功")
}

pub async fn get_avatar_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let row = sqlx::query("SELECT avatar_url FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let avatar: String = r.get("avatar_url");
            ctx.json(200, "ok", Some(json!({ "has_avatar": !avatar.is_empty() })))
        }
        None => ctx.err(404, "用户不存在"),
    }
}

pub async fn get_nickname_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let row = sqlx::query("SELECT username FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let username: String = r.get("username");
            ctx.json(200, "ok", Some(json!({ "nickname": username })))
        }
        None => ctx.err(404, "用户不存在"),
    }
}

pub async fn report_listen_stats(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    if let Some(v) = data.get("duration") {
        let seconds = v.as_f64().unwrap_or(0.0) as i64;
        let _ = sqlx::query("UPDATE app_users SET listen_duration = listen_duration + ? WHERE ciyuanxi_id = ?")
            .bind(seconds)
            .bind(&ciyuanxi_id)
            .execute(pool)
            .await;
    }
    ctx.ok_empty("ok")
}

pub async fn deduct_master_quota(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let amount = data.get("amount").and_then(|v| v.as_i64()).unwrap_or(1);
    let res = sqlx::query("UPDATE app_users SET master_quota = GREATEST(master_quota - ?, 0) WHERE ciyuanxi_id = ?")
        .bind(amount)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    match res {
        Ok(_) => ctx.ok_empty("ok"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn get_master_quota_usage(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let row = sqlx::query("SELECT master_quota FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let quota: i64 = r.get("master_quota");
            ctx.json(200, "ok", Some(json!({ "master_quota": quota })))
        }
        None => ctx.err(404, "用户不存在"),
    }
}