use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::response::ReqCtx;

/// 手表端可下发的合法控制命令
const ALLOWED_OPS: &[&str] = &[
    "toggle",
    "play",
    "pause",
    "next",
    "prev",
    "favorite",
    "un_favorite",
    "seek",
    "play_mode",
];

fn op_allowed(op: &str) -> bool {
    ALLOWED_OPS.contains(&op)
}

/// 手表端提交控制命令 → 写入 watch_commands 队列，等待手机端轮询执行。
pub async fn watch_submit_command(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let device_id = str_of(&data, "device_id").trim().to_string();
    let request_id = str_of(&data, "request_id").trim().to_string();
    let op = str_of(&data, "op").trim().to_string();
    if ciyuanxi_id.is_empty() || device_id.is_empty() || request_id.is_empty() || op.is_empty() {
        return ctx.err(400, "参数不完整");
    }
    if !op_allowed(&op) {
        return ctx.err(400, &format!("不支持的控制命令: {}", op));
    }
    let payload = data.get("payload").cloned().unwrap_or(Value::Null);
    let payload_str = payload.to_string();
    // request_id 唯一冲突时 INSERT IGNORE 幂等跳过，手机端按序取走即可
    let result = sqlx::query(
        "INSERT IGNORE INTO watch_commands (request_id, ciyuanxi_id, from_device_id, op, payload, status) VALUES (?,?,?,?,?,'pending')",
    )
    .bind(&request_id)
    .bind(&ciyuanxi_id)
    .bind(&device_id)
    .bind(&op)
    .bind(&payload_str)
    .execute(pool)
    .await;
    match result {
        Ok(_) => ctx.ok("已提交", json!({ "request_id": request_id, "op": op })),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

/// 手机端轮询本账号未消费的手表命令，取出后标记 consumed。
/// 返回最多 20 条，手机端按序执行。
pub async fn watch_poll_command(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数不完整");
    }
    let rows = sqlx::query(
        "SELECT id, request_id, from_device_id, op, payload FROM watch_commands
         WHERE ciyuanxi_id = ? AND status = 'pending' ORDER BY id ASC LIMIT 20",
    )
    .bind(&ciyuanxi_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut commands: Vec<Value> = Vec::new();
    let mut ids: Vec<i64> = Vec::new();
    for row in &rows {
        let id: i64 = row.try_get("id").unwrap_or(0);
        ids.push(id);
        let request_id: String = row.try_get("request_id").unwrap_or_default();
        let from_device_id: String = row.try_get("from_device_id").unwrap_or_default();
        let op: String = row.try_get("op").unwrap_or_default();
        let payload: Option<String> = row.try_get("payload").ok().flatten();
        commands.push(json!({
            "request_id": request_id,
            "from_device_id": from_device_id,
            "op": op,
            "payload": payload.and_then(|s| serde_json::from_str::<Value>(&s).ok()).unwrap_or(Value::Null),
        }));
    }
    if !ids.is_empty() {
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "UPDATE watch_commands SET status = 'consumed', consumed_at = NOW() WHERE id IN ({}) AND status = 'pending'",
            placeholders
        );
        let mut q = sqlx::query(&sql);
        for id in &ids {
            q = q.bind(*id);
        }
        let _ = q.execute(pool).await;
    }
    ctx.ok("ok", json!({ "commands": commands }))
}

/// 手机端心跳 + 上报当前播放信息（presence），手表端据此判断手机在线并展示曲目信息。
pub async fn watch_phone_ping(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let device_id = str_of(&data, "device_id").trim().to_string();
    if ciyuanxi_id.is_empty() || device_id.is_empty() {
        return ctx.err(400, "参数不完整");
    }
    let device_model = str_of(&data, "device_model").trim().to_string();
    let app_version = str_of(&data, "app_version").trim().to_string();
    let playing_title = str_of(&data, "playing_title");
    let playing_artist = str_of(&data, "playing_artist");
    let playing_album = str_of(&data, "playing_album");
    let playing_cover = str_of(&data, "playing_cover");
    let is_playing = int_of(&data, "is_playing");
    let is_favorite = int_of(&data, "is_favorite");
    let _ = sqlx::query(
        "INSERT INTO device_presence
            (ciyuanxi_id, device_id, device_type, device_model, app_version,
             playing_title, playing_artist, playing_album, playing_cover, is_playing, is_favorite, updated_at)
         VALUES (?,?,'mobile',?,?,?,?,?,?,?,?,NOW())
         ON DUPLICATE KEY UPDATE
            updated_at = NOW(),
            playing_title = VALUES(playing_title),
            playing_artist = VALUES(playing_artist),
            playing_album = VALUES(playing_album),
            playing_cover = VALUES(playing_cover),
            is_playing = VALUES(is_playing),
            is_favorite = VALUES(is_favorite)",
    )
    .bind(&ciyuanxi_id)
    .bind(&device_id)
    .bind(&device_model)
    .bind(&app_version)
    .bind(&playing_title)
    .bind(&playing_artist)
    .bind(&playing_album)
    .bind(&playing_cover)
    .bind(is_playing)
    .bind(is_favorite)
    .execute(pool)
    .await;
    // 顺带清理该账号过期的手机 presence（> 15 分钟未心跳视为离线）
    let _ = sqlx::query(
        "DELETE FROM device_presence WHERE ciyuanxi_id = ? AND device_type = 'mobile' AND updated_at < DATE_SUB(NOW(), INTERVAL 15 MINUTE)",
    )
    .bind(&ciyuanxi_id)
    .execute(pool)
    .await;
    ctx.ok("ok", json!({ "online": true }))
}

/// 手表端查询本账号在线手机设备及其当前播放信息。
pub async fn watch_phone_query(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "参数不完整");
    }
    let rows = sqlx::query(
        "SELECT device_id, device_model, app_version, playing_title, playing_artist, playing_album,
                playing_cover, is_playing, is_favorite,
                (updated_at >= DATE_SUB(NOW(), INTERVAL 60 SECOND)) AS online
         FROM device_presence WHERE ciyuanxi_id = ? AND device_type = 'mobile'
         ORDER BY updated_at DESC",
    )
    .bind(&ciyuanxi_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let mut phones: Vec<Value> = Vec::new();
    for row in &rows {
        let device_id: String = row.try_get("device_id").unwrap_or_default();
        let device_model: String = row.try_get("device_model").unwrap_or_default();
        let app_version: String = row.try_get("app_version").unwrap_or_default();
        let playing_title: String = row.try_get("playing_title").unwrap_or_default();
        let playing_artist: String = row.try_get("playing_artist").unwrap_or_default();
        let playing_album: String = row.try_get("playing_album").unwrap_or_default();
        let playing_cover: String = row.try_get("playing_cover").unwrap_or_default();
        let is_playing: i64 = row.try_get("is_playing").unwrap_or(0);
        let is_favorite: i64 = row.try_get("is_favorite").unwrap_or(0);
        let online: i64 = row.try_get("online").unwrap_or(0);
        phones.push(json!({
            "device_id": device_id,
            "device_model": device_model,
            "app_version": app_version,
            "playing": {
                "title": playing_title,
                "artist": playing_artist,
                "album": playing_album,
                "cover": playing_cover,
                "is_playing": is_playing == 1,
                "is_favorite": is_favorite == 1,
            },
            "online": online == 1,
        }));
    }
    ctx.ok("ok", json!({ "phones": phones }))
}