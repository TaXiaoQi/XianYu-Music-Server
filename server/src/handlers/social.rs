use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

pub async fn submit_feedback(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let mut nickname = str_of(&data, "nickname").trim().to_string();
    let title = str_of(&data, "title").trim().to_string();
    let content = str_of(&data, "content").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "请先登录");
    }
    if title.is_empty() {
        return ctx.err(400, "标题不能为空");
    }
    if content.is_empty() {
        return ctx.err(400, "内容不能为空");
    }
    if title.chars().count() > 60 {
        return ctx.err(400, "标题不能超过 60 字");
    }
    if content.chars().count() > 1000 {
        return ctx.err(400, "内容不能超过 1000 字");
    }
    if nickname.is_empty() {
        let row = sqlx::query("SELECT username FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
            .bind(&ciyuanxi_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        if let Some(r) = row {
            nickname = r.get("username");
        }
    }
    let ip = ctx.client_ip.clone();
    let result = sqlx::query("INSERT INTO user_feedback (ciyuanxi_id, nickname, title, content, ip) VALUES (?,?,?,?,?)")
        .bind(&ciyuanxi_id)
        .bind(&nickname)
        .bind(&title)
        .bind(&content)
        .bind(&ip)
        .execute(pool)
        .await;
    match result {
        Ok(r) => ctx.json(200, "提交成功", Some(json!({ "id": r.last_insert_id() }))),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn check_ciyuanxi_id(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let mut identifier = str_of(&data, "ciyuanxi_id");
    if identifier.is_empty() {
        identifier = str_of(&data, "user_id");
    }
    if identifier.is_empty() {
        return ctx.err(400, "用户标识不能为空");
    }
    let row = sqlx::query("SELECT id, ciyuanxi_id FROM app_users WHERE id = ? OR ciyuanxi_id = ? LIMIT 1")
        .bind(&identifier)
        .bind(&identifier)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let user_id: i64 = r.get("id");
            let ciyuanxi_id: String = r.get("ciyuanxi_id");
            ctx.json(200, "ok", Some(json!({ "user_id": user_id, "ciyuanxi_id": ciyuanxi_id })))
        }
        None => ctx.err(404, "用户不存在"),
    }
}

pub async fn create_ciyuanxi_id(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let mut identifier = str_of(&data, "ciyuanxi_id");
    if identifier.is_empty() {
        identifier = str_of(&data, "user_id");
    }
    if identifier.is_empty() {
        return ctx.err(400, "用户标识不能为空");
    }
    let row = sqlx::query("SELECT id, ciyuanxi_id, username FROM app_users WHERE id = ? OR ciyuanxi_id = ? LIMIT 1")
        .bind(&identifier)
        .bind(&identifier)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(row) = row else {
        return ctx.err(404, "用户不存在");
    };
    let user_id: i64 = row.get("id");
    let ciyuanxi_id: String = row.get("ciyuanxi_id");
    if !ciyuanxi_id.is_empty() {
        return ctx.json(200, "ok", Some(json!({ "user_id": user_id, "ciyuanxi_id": ciyuanxi_id })));
    }
    let new_id = format!(
        "cx_{:06}{:04}",
        user_id,
        crate::handlers::helpers::random_int(0, 9999)
    );
    let _ = sqlx::query("UPDATE app_users SET ciyuanxi_id = ? WHERE id = ?")
        .bind(&new_id)
        .bind(user_id)
        .execute(pool)
        .await;
    ctx.json(200, "创建成功", Some(json!({ "user_id": user_id, "ciyuanxi_id": new_id })))
}
