use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

const SOURCE_NAMES: [(&str, &str); 5] = [
    ("kg", "酷狗音乐"),
    ("tx", "QQ音乐"),
    ("kw", "酷我音乐"),
    ("mg", "咪咕音乐"),
    ("wy", "网易音乐"),
];

async fn seed_sources(pool: &MySqlPool) {
    let _ = sqlx::query(
        "INSERT IGNORE INTO `music_source_config` (source_name, source_code, is_enabled) VALUES \
         ('酷狗音乐','kg',1),('QQ音乐','tx',1),('酷我音乐','kw',1),('咪咕音乐','mg',1),('网易音乐','wy',1)",
    )
    .execute(pool)
    .await;
}

/// 获取音源开关配置
pub async fn get_source(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    seed_sources(pool).await;
    let rows = sqlx::query("SELECT source_name, source_code, is_enabled FROM music_source_config")
        .fetch_all(pool)
        .await;
    match rows {
Ok(rows) => {
            let list: Vec<Value> = rows
                .iter()
                .map(|r| json!({
                    "source_name": r.get::<String, _>("source_name"),
                    "source_code": r.get::<String, _>("source_code"),
                    "is_enabled": r.get::<i64, _>("is_enabled"),
                }))
                .collect();
            log_operation(pool, ctx, "查看音源配置", "", "").await;
            ok("", json!(list))
        }
        Err(_) => err(500, "查询失败"),
    }
}

/// 切换音源开关
pub async fn toggle_source(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let source_code = str_of(&data, "source_code").trim().to_string();
    let enabled = int_of(&data, "enabled");
    if enabled != 0 && enabled != 1 {
        return err(400, "参数错误");
    }
    if !SOURCE_NAMES.iter().any(|(c, _)| *c == source_code.as_str()) {
        return err(400, "无效的音源");
    }
    seed_sources(pool).await;
    let upd = sqlx::query("UPDATE music_source_config SET is_enabled = ? WHERE source_code = ?")
        .bind(enabled)
        .bind(&source_code)
        .execute(pool)
        .await;
    match upd {
        Ok(_) => {
            let name = SOURCE_NAMES.iter().find(|(c, _)| *c == source_code.as_str()).map(|(_, n)| *n).unwrap_or(source_code.as_str());
            let suffix = if enabled == 1 { "已启用" } else { "已禁用" };
            log_operation(pool, ctx, "音源开关", &format!("{}{}", name, suffix), "").await;
            ok(&format!("{}{}", name, suffix), Value::Null)
        }
        Err(_) => err(500, "操作失败"),
    }
}