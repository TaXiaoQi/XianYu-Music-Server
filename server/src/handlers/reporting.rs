use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;

use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::response::ReqCtx;

/// source_call 音源调用上报
pub async fn source_call(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ip = ctx.client_ip.clone();
    let result = sqlx::query(
        "INSERT INTO source_call_log (ip, device_id, source_name, action, song_name, singer, status, result_status, error_msg, duration_ms, platform, source_type, request_params) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(ip)
    .bind(str_of(&data, "device_id"))
    .bind(str_of(&data, "source_name"))
    .bind(str_of(&data, "action"))
    .bind(str_of(&data, "song_name"))
    .bind(str_of(&data, "singer"))
    .bind(int_of(&data, "status"))
    .bind(str_of(&data, "result_status"))
    .bind(str_of(&data, "error_msg"))
    .bind(int_of(&data, "duration_ms"))
    .bind(str_of(&data, "platform"))
    .bind(str_of(&data, "source_type"))
    .bind(body.to_string())
    .execute(pool)
    .await;

    match result {
        Ok(r) => ctx.json(200, "上报成功", Some(json!({ "id": r.last_insert_id() }))),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

/// login 登录上报
pub async fn login(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ip = ctx.client_ip.clone();
    let result = sqlx::query(
        "INSERT INTO login_log (ip, device_id, user_id, username, status, request_params) VALUES (?,?,?,?,?,?)",
    )
    .bind(ip)
    .bind(str_of(&data, "device_id"))
    .bind(str_of(&data, "user_id"))
    .bind(str_of(&data, "username"))
    .bind(int_of(&data, "status"))
    .bind(body.to_string())
    .execute(pool)
    .await;

    match result {
        Ok(r) => ctx.json(200, "上报成功", Some(json!({ "id": r.last_insert_id() }))),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

/// error 错误上报
pub async fn error(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ip = ctx.client_ip.clone();
    let result = sqlx::query(
        "INSERT INTO error_log (ip, device_id, app_version, os_version, device_model, error_type, error_message, error_stack, page, device_brand, platform, request_params) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(ip)
    .bind(str_of(&data, "device_id"))
    .bind(str_of(&data, "app_version"))
    .bind(str_of(&data, "os_version"))
    .bind(str_of(&data, "device_model"))
    .bind(str_of(&data, "error_type"))
    .bind(str_of(&data, "error_message"))
    .bind(str_of(&data, "error_stack"))
    .bind(str_of(&data, "page"))
    .bind(str_of(&data, "device_brand"))
    .bind(str_of(&data, "platform"))
    .bind(body.to_string())
    .execute(pool)
    .await;

    match result {
        Ok(r) => ctx.json(200, "上报成功", Some(json!({ "id": r.last_insert_id() }))),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

/// check 数据库连接检查
pub async fn check(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let conn_ok = sqlx::query("SELECT 1").execute(pool).await.is_ok();
    let tables = [
        "source_call_log", "login_log", "error_log", "app_users", "email_verify_codes",
        "admin_users", "admin_operation_log", "admin_login_log", "email_templates", "email_send_log",
    ];
    let mut table_map = serde_json::Map::new();
    for t in tables.iter() {
        let exists = sqlx::query(&format!("SHOW TABLES LIKE '{}'", t))
            .fetch_all(pool)
            .await
            .map(|r| !r.is_empty())
            .unwrap_or(false);
        table_map.insert(t.to_string(), json!(exists));
    }
    ctx.json(
        200,
        "检测完成",
        Some(json!({ "connection": conn_ok, "tables": table_map })),
    )
}

/// install 安装
pub async fn install(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let conn_ok = sqlx::query("SELECT 1").execute(pool).await.is_ok();
    if !conn_ok {
        return ctx.err(500, "数据库连接失败");
    }
    ctx.json(200, "安装完成", Some(json!([])))
}
