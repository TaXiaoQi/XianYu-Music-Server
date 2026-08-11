use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;

use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::response::ReqCtx;

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

/// open 客户端启动上报
pub async fn app_open(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let device_id = str_of(&data, "device_id");
    if device_id.is_empty() {
        return ctx.err(400, "设备标识不能为空");
    }
    let result = sqlx::query(
        "INSERT INTO app_open_log (device_id, app_version, os_version, device_model, ip, ciyuanxi_id) VALUES (?,?,?,?,?,?)",
    )
    .bind(&device_id)
    .bind(str_of(&data, "app_version"))
    .bind(str_of(&data, "os_version"))
    .bind(str_of(&data, "device_model"))
    .bind(&ctx.client_ip)
    .bind(str_of(&data, "ciyuanxi_id"))
    .execute(pool)
    .await;

    match result {
        Ok(_) => ctx.ok_empty("ok"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

/// report_user_behavior 播放行为上报
///
/// 客户端播放/切歌时会上报本次播放时长。排行榜依赖 app_users.listen_duration，
/// 因此这里必须同步写入账号表；否则正式服务会把该 action 当作未知操作，
/// 播放统计被客户端静默吞掉，排行榜一直没有数据。
pub async fn report_user_behavior(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }

    let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
    if ciyuanxi_id.is_empty() {
        // 未登录播放不参与账号排行榜，但不应打断客户端播放流程。
        return ctx.ok_empty("ok");
    }

    let duration = int_of(&data, "listen_duration").max(0);
    if duration <= 0 {
        return ctx.ok_empty("ok");
    }

    let result = sqlx::query(
        "UPDATE app_users SET listen_duration = GREATEST(listen_duration, ?) WHERE ciyuanxi_id = ?",
    )
    .bind(duration)
    .bind(&ciyuanxi_id)
    .execute(pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => ctx.ok_empty("ok"),
        Ok(_) => ctx.err(404, "用户不存在"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}
