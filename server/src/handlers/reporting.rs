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

/// 写入一条音源调用记录（source_call_log）。
/// dashboard 的音源调用分布与热搜均依赖此表。
/// fire-and-forget：写入失败静默，不阻塞主流程。
async fn record_source_call(
    pool: &MySqlPool,
    data: &serde_json::Value,
    ctx: &ReqCtx,
    action: &str,
    source_fallback: &str,
) {
    let info = data;
    let source_name = str_of(info, "source").trim().to_string();
    let source_name = if source_name.is_empty() {
        source_fallback.to_string()
    } else {
        source_name
    };
    let song_name = str_of(info, "song_name");
    let song_name = if song_name.is_empty() {
        str_of(info, "keyword")
    } else {
        song_name
    };
    let _ = sqlx::query(
        "INSERT INTO source_call_log (ip, device_id, source_name, action, song_name, singer, status, platform, source_type, duration_ms, request_params) \
         VALUES (?,?,?,?,?,?,1,?,?,?,?)",
    )
    .bind(&ctx.client_ip)
    .bind(str_of(info, "device_id"))
    .bind(&source_name)
    .bind(action)
    .bind(&song_name)
    .bind(str_of(info, "singer"))
    .bind(str_of(info, "platform"))
    .bind(str_of(info, "source_type"))
    .bind(int_of(info, "duration_ms").max(0))
    .bind(info.to_string())
    .execute(pool)
    .await;
}

/// search 搜索上报
///
/// 客户端 Search.vue 在完成一次音源搜索后上报（fire-and-forget）。
/// 写入 source_call_log（action='search'），dashboard 据此统计热搜关键词。
pub async fn search(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    record_source_call(pool, &data, &ctx, "search", "未知音源").await;
    ctx.ok_empty("ok")
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

    // 无论是否登录都记录音源调用（音源调用分布统计不依赖账号）
    let ua_action = {
        let a = str_of(&data, "action");
        if a.is_empty() { "play" } else { &a }.to_string()
    };
    record_source_call(pool, &data, &ctx, &ua_action, "未知音源").await;

    let ciyuanxi_id = str_of(&data, "ciyuanxi_id");
    if ciyuanxi_id.is_empty() {
        // 未登录播放不参与账号排行榜，但不应打断客户端播放流程。
        return ctx.ok_empty("ok");
    }

    let duration = int_of(&data, "listen_duration").max(0);
    if duration <= 0 {
        return ctx.ok_empty("ok");
    }

    // 检查是否存在待处理的听歌统计重置信号，如有则跳过本次更新（由 report_listen_stats 统一处理重置）
    let has_reset: Option<String> = sqlx::query_scalar(
        "SELECT listen_stats_reset_at FROM app_users WHERE ciyuanxi_id = ?",
    )
    .bind(&ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if has_reset.is_some() {
        return ctx.ok_empty("ok");
    }

    let result = sqlx::query(
        "UPDATE app_users SET listen_duration = listen_duration + ? WHERE ciyuanxi_id = ?",
    )
    .bind(duration)
    .bind(&ciyuanxi_id)
    .execute(pool)
    .await;

    // 同步写入每日统计（用于日榜/周榜）
    let _ = sqlx::query(
        "INSERT INTO listen_daily_stats (ciyuanxi_id, stat_date, listen_duration) \
         VALUES (?, CURDATE(), ?) \
         ON DUPLICATE KEY UPDATE listen_duration = listen_duration + VALUES(listen_duration)",
    )
    .bind(&ciyuanxi_id)
    .bind(duration)
    .execute(pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => ctx.ok_empty("ok"),
        Ok(_) => ctx.err(404, "用户不存在"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}
