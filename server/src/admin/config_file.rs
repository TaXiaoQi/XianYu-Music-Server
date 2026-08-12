use axum::response::Response;
use serde_json::{json, Map, Number, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{bool_of, int_of, parse_body, str_of};

fn config_path() -> std::path::PathBuf {
    std::path::Path::new("config.json").to_path_buf()
}

fn local_cache_path() -> std::path::PathBuf {
    std::path::Path::new("data").join("debug").join("state.json")
}

fn default_config(ctx: &AdminCtx) -> Value {
    json!({
        "db_host": ctx.config.db_host.clone(),
        "db_port": ctx.config.db_port,
        "db_name": ctx.config.db_name.clone(),
        "db_user": ctx.config.db_user.clone(),
        "db_pass": ctx.config.db_pass.clone(),
        "db_charset": ctx.config.db_charset.clone(),
        "api_secret": ctx.config.api_secret.clone(),
        "api_timestamp_tolerance": ctx.config.api_timestamp_tolerance,
        "admin_username": ctx.config.admin_username.clone(),
        "admin_password": ctx.config.admin_password.clone(),
        "listen_addr": ctx.config.listen_addr.clone(),
        "jwt_secret": ctx.config.jwt_secret.clone(),
        "email_api_primary": ctx.config.email_api_primary.clone(),
        "email_api_backup": ctx.config.email_api_backup.clone(),
        "email_sender": ctx.config.email_sender.clone(),
        "email_password": ctx.config.email_password.clone(),
        "captcha_secret": ctx.config.captcha_secret.clone(),
        "turnstile_secret": ctx.config.turnstile_secret.clone(),
        "hcaptcha_secret": ctx.config.hcaptcha_secret.clone(),
        "static_dir": "../admin-web/dist",
        "local_debug_no_db": ctx.config.local_debug_no_db,
        "public_base_url": ctx.config.public_base_url.clone(),
    })
}

fn read_config_value(ctx: &AdminCtx) -> Value {
    let defaults = default_config(ctx);
    let Ok(content) = std::fs::read_to_string(config_path()) else {
        return defaults;
    };
    let Ok(Value::Object(saved)) = serde_json::from_str::<Value>(&content) else {
        return defaults;
    };
    let mut merged = defaults.as_object().cloned().unwrap_or_default();
    for (key, value) in saved {
        merged.insert(key, value);
    }
    Value::Object(merged)
}

fn has_non_empty(map: &Map<String, Value>, key: &str) -> bool {
    map.get(key)
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

fn public_config(mut config: Value) -> Value {
    let Some(map) = config.as_object_mut() else {
        return json!({});
    };
    let sensitive_keys = [
        "db_pass",
        "api_secret",
        "admin_password",
        "jwt_secret",
        "email_password",
        "captcha_secret",
        "turnstile_secret",
        "hcaptcha_secret",
    ];
    let snapshot = map.clone();
    for key in sensitive_keys {
        let has_key = has_non_empty(&snapshot, key);
        map.insert(key.to_string(), Value::String(String::new()));
        map.insert(format!("has_{}", key), Value::Bool(has_key));
    }
    config
}

fn set_string(map: &mut Map<String, Value>, data: &Value, key: &str) {
    if data.get(key).is_some() {
        map.insert(key.to_string(), Value::String(str_of(data, key).trim().to_string()));
    }
}

fn set_secret(map: &mut Map<String, Value>, data: &Value, key: &str) {
    if data.get(key).is_none() {
        return;
    }
    let value = str_of(data, key).trim().to_string();
    if !value.is_empty() {
        map.insert(key.to_string(), Value::String(value));
    }
}

fn set_i64(map: &mut Map<String, Value>, data: &Value, key: &str, min: i64, max: i64) -> Result<(), String> {
    if data.get(key).is_none() {
        return Ok(());
    }
    let value = int_of(data, key);
    if value < min || value > max {
        return Err(format!("{} 超出允许范围", key));
    }
    map.insert(key.to_string(), Value::Number(Number::from(value)));
    Ok(())
}

fn write_config(config: &Value) -> std::io::Result<()> {
    let path = config_path();
    let json = serde_json::to_string_pretty(config).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(&path, json)
}

fn merge_config_from_body(body: &str, ctx: &AdminCtx) -> Result<Value, String> {
    let data = parse_body(body);
    let mut config = read_config_value(ctx);
    let Some(map) = config.as_object_mut() else {
        return Err("配置文件格式异常".to_string());
    };

    let string_keys = [
        "db_host",
        "db_name",
        "db_user",
        "db_charset",
        "listen_addr",
        "admin_username",
        "email_api_primary",
        "email_api_backup",
        "email_sender",
        "static_dir",
        "public_base_url",
    ];
    for key in string_keys {
        set_string(map, &data, key);
    }

    let secret_keys = [
        "db_pass",
        "api_secret",
        "admin_password",
        "jwt_secret",
        "email_password",
        "captcha_secret",
        "turnstile_secret",
        "hcaptcha_secret",
    ];
    for key in secret_keys {
        set_secret(map, &data, key);
    }

    set_i64(map, &data, "db_port", 1, 65535)?;
    set_i64(map, &data, "api_timestamp_tolerance", 1, 86400)?;
    if data.get("local_debug_no_db").is_some() {
        map.insert("local_debug_no_db".to_string(), Value::Bool(bool_of(&data, "local_debug_no_db")));
    }

    let db_host = map.get("db_host").and_then(|v| v.as_str()).unwrap_or("").trim();
    let db_name = map.get("db_name").and_then(|v| v.as_str()).unwrap_or("").trim();
    let db_user = map.get("db_user").and_then(|v| v.as_str()).unwrap_or("").trim();
    let db_charset = map.get("db_charset").and_then(|v| v.as_str()).unwrap_or("").trim();
    let listen_addr = map.get("listen_addr").and_then(|v| v.as_str()).unwrap_or("").trim();
    if db_host.is_empty() || db_name.is_empty() || db_user.is_empty() || db_charset.is_empty() || listen_addr.is_empty() {
        return Err("数据库主机、数据库名、用户名、字符集和监听地址不能为空".to_string());
    }
    Ok(config)
}

pub async fn get(_body: &str, ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    ok("ok", public_config(read_config_value(ctx)))
}

pub async fn get_no_db(_body: &str, ctx: &AdminCtx) -> Response {
    ok("ok", public_config(read_config_value(ctx)))
}

pub async fn save(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let config = match merge_config_from_body(body, ctx) {
        Ok(v) => v,
        Err(msg) => return err(400, &msg),
    };

    if write_config(&config).is_err() {
        return err(500, "写入 config.json 失败，请检查服务端目录权限");
    }

    log_operation(pool, ctx, "保存配置文件", "config.json", "更新服务端基础配置，需重启服务端后完全生效").await;
    ok("保存成功，重启服务端后生效", public_config(config))
}

pub async fn save_no_db(body: &str, ctx: &AdminCtx) -> Response {
    let config = match merge_config_from_body(body, ctx) {
        Ok(v) => v,
        Err(msg) => return err(400, &msg),
    };
    if write_config(&config).is_err() {
        return err(500, "写入 config.json 失败，请检查服务端目录权限");
    }
    ok("保存成功，重启服务端后生效", public_config(config))
}

fn value_str(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

fn value_i64(v: &Value, key: &str, default: i64) -> i64 {
    v.get(key)
        .and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(default)
}

fn load_local_cache() -> Value {
    std::fs::read_to_string(local_cache_path())
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        .unwrap_or_else(|| json!({}))
}

async fn migrate_users(pool: &MySqlPool, state: &Value) -> (usize, usize) {
    let mut ok_count = 0;
    let mut fail_count = 0;
    let users = state.get("users").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    for user in users {
        let username = value_str(&user, "username");
        let email = value_str(&user, "email");
        if username.is_empty() || email.is_empty() {
            continue;
        }
        let raw_password = value_str(&user, "password");
        let password = if raw_password.starts_with("$2") {
            raw_password
        } else {
            bcrypt::hash(if raw_password.is_empty() { "123456" } else { &raw_password }, 10).unwrap_or_default()
        };
        let status = value_i64(&user, "status", 1);
        let ciyuanxi_id = value_str(&user, "ciyuanxi_id");
        let avatar_url = value_str(&user, "avatar_url");
        let master_quota = value_i64(&user, "master_quota", 0);
        let result = sqlx::query(
            "INSERT INTO app_users
             (username, password, email, email_verified, status, ciyuanxi_id, avatar_url, master_quota)
             VALUES (?, ?, ?, 1, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
               password = VALUES(password),
               status = VALUES(status),
               ciyuanxi_id = VALUES(ciyuanxi_id),
               avatar_url = VALUES(avatar_url),
               master_quota = VALUES(master_quota)",
        )
        .bind(username)
        .bind(password)
        .bind(email)
        .bind(status)
        .bind(ciyuanxi_id)
        .bind(avatar_url)
        .bind(master_quota)
        .execute(pool)
        .await;
        if result.is_ok() {
            ok_count += 1;
        } else {
            fail_count += 1;
        }
    }
    (ok_count, fail_count)
}

async fn migrate_settings(pool: &MySqlPool, state: &Value) -> (usize, usize) {
    let mut ok_count = 0;
    let mut fail_count = 0;
    let Some(settings) = state.get("settings").and_then(|v| v.as_object()) else {
        return (0, 0);
    };
    for (ciyuanxi_id, item) in settings {
        let result = sqlx::query(
            "INSERT INTO user_settings
             (ciyuanxi_id, stream_cache_enabled, startup_play_enabled, bluetooth_lyric_enabled,
              download_lyric_enabled, download_cover_enabled, download_artist_enabled,
              search_board_enabled, page_animation_enabled, default_quality)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
              stream_cache_enabled = VALUES(stream_cache_enabled),
              startup_play_enabled = VALUES(startup_play_enabled),
              bluetooth_lyric_enabled = VALUES(bluetooth_lyric_enabled),
              download_lyric_enabled = VALUES(download_lyric_enabled),
              download_cover_enabled = VALUES(download_cover_enabled),
              download_artist_enabled = VALUES(download_artist_enabled),
              search_board_enabled = VALUES(search_board_enabled),
              page_animation_enabled = VALUES(page_animation_enabled),
              default_quality = VALUES(default_quality)",
        )
        .bind(ciyuanxi_id)
        .bind(value_i64(item, "stream_cache_enabled", 1))
        .bind(value_i64(item, "startup_play_enabled", 0))
        .bind(value_i64(item, "bluetooth_lyric_enabled", 0))
        .bind(value_i64(item, "download_lyric_enabled", 1))
        .bind(value_i64(item, "download_cover_enabled", 1))
        .bind(value_i64(item, "download_artist_enabled", 0))
        .bind(value_i64(item, "search_board_enabled", 1))
        .bind(value_i64(item, "page_animation_enabled", 1))
        .bind(value_str(item, "default_quality"))
        .execute(pool)
        .await;
        if result.is_ok() {
            ok_count += 1;
        } else {
            fail_count += 1;
        }
    }
    (ok_count, fail_count)
}

async fn migrate_feedback(pool: &MySqlPool, state: &Value) -> (usize, usize) {
    let mut ok_count = 0;
    let mut fail_count = 0;
    let rows = state.get("feedback").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    for row in rows {
        let data = row.get("data").unwrap_or(&row);
        let title = {
            let t = value_str(data, "title");
            if t.is_empty() { "本地缓存反馈".to_string() } else { t }
        };
        let content = value_str(data, "content");
        let ciyuanxi_id = value_str(data, "ciyuanxi_id");
        let nickname = value_str(data, "nickname");
        let error_logs = data.get("error_logs").cloned().unwrap_or(Value::Null).to_string();
        let all_logs = data.get("all_logs").cloned().unwrap_or(Value::Null).to_string();
        let status = value_str(&row, "status");
        let status = if status.is_empty() { "pending".to_string() } else { status };
        let result = sqlx::query(
            "INSERT INTO user_feedback
             (ciyuanxi_id, nickname, title, content, error_logs, all_logs, status)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(ciyuanxi_id)
        .bind(nickname)
        .bind(title)
        .bind(content)
        .bind(error_logs)
        .bind(all_logs)
        .bind(status)
        .execute(pool)
        .await;
        if result.is_ok() {
            ok_count += 1;
        } else {
            fail_count += 1;
        }
    }
    (ok_count, fail_count)
}

pub async fn migrate_local_cache_to_database(_body: &str, ctx: &AdminCtx) -> Response {
    let config_value = read_config_value(ctx);
    let cfg: crate::config::Config = match serde_json::from_value(config_value.clone()) {
        Ok(v) => v,
        Err(e) => return err(400, &format!("配置文件无法解析: {}", e)),
    };
    let pool = match crate::db::connect(&cfg).await {
        Ok(v) => v,
        Err(e) => return err(400, &format!("数据库连接配置无效: {}", e)),
    };
    match tokio::time::timeout(std::time::Duration::from_secs(8), sqlx::query("SELECT 1").execute(&pool)).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => return err(400, &format!("数据库连接失败: {}", e)),
        Err(_) => return err(400, "数据库连接超时"),
    }

    crate::schema::ensure_schema(&pool).await;
    let state = load_local_cache();
    let (users_ok, users_fail) = migrate_users(&pool, &state).await;
    let (settings_ok, settings_fail) = migrate_settings(&pool, &state).await;
    let (feedback_ok, feedback_fail) = migrate_feedback(&pool, &state).await;

    ok("本地缓存迁移完成，重启服务端后将使用数据库模式", json!({
        "users": { "ok": users_ok, "failed": users_fail },
        "settings": { "ok": settings_ok, "failed": settings_fail },
        "feedback": { "ok": feedback_ok, "failed": feedback_fail },
        "cache_file": local_cache_path().to_string_lossy(),
        "need_restart": true
    }))
}
