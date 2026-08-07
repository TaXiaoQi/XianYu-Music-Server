use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{int_of, parse_body, resolve_role_by_email, str_of};
use crate::response::ReqCtx;

/// 确保主频道存在，返回 (id, name, description)
async fn ensure_main_channel(pool: &MySqlPool) -> (i64, String, String) {
    let q = "SELECT id, name, description FROM chat_channels WHERE channel_key = 'main' LIMIT 1";
    let row = sqlx::query(q).fetch_optional(pool).await.ok().flatten();
    if let Some(r) = row {
        return (
            r.try_get("id").unwrap_or(0),
            r.try_get("name").unwrap_or_default(),
            r.try_get("description").unwrap_or_default(),
        );
    }
    let _ = sqlx::query("INSERT INTO chat_channels (channel_key, name, description) VALUES ('main', '弦予音乐频道', '')")
        .execute(pool)
        .await;
    let row = sqlx::query(q).fetch_optional(pool).await.ok().flatten();
    if let Some(r) = row {
        (
            r.try_get("id").unwrap_or(0),
            r.try_get("name").unwrap_or_default(),
            r.try_get("description").unwrap_or_default(),
        )
    } else {
        (0, String::new(), String::new())
    }
}

/// 根据弦予号查 app_users，返回 (username, email, avatar_url, status)
async fn resolve_app_user(pool: &MySqlPool, ciyuanxi_id: &str) -> Option<(String, String, String, i64)> {
    sqlx::query("SELECT username, email, avatar_url, status FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|r| {
            (
                r.try_get("username").unwrap_or_default(),
                r.try_get("email").unwrap_or_default(),
                r.try_get("avatar_url").unwrap_or_default(),
                r.try_get("status").unwrap_or(1),
            )
        })
}

pub async fn chat_get_messages(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let (channel_id, channel_name, channel_desc) = ensure_main_channel(pool).await;
    let since_id = int_of(&data, "since_id");
    let before_id = int_of(&data, "before_id");
    let all = str_of(&data, "all").eq_ignore_ascii_case("1")
        || str_of(&data, "all").eq_ignore_ascii_case("true");
    let raw_limit = int_of(&data, "limit");
    let limit: i64 = if all { 10000 } else { raw_limit.clamp(1, 100) };
    let base = "SELECT id, channel_id, ciyuanxi_id, username, avatar_url, role, msg_type, content, extra, created_at FROM chat_messages WHERE channel_id = ?";
    let (sql, is_before) = if since_id > 0 {
        (format!("{} AND id > ? ORDER BY id ASC LIMIT ?", base), false)
    } else if before_id > 0 {
        (format!("{} AND id < ? ORDER BY id DESC LIMIT ?", base), true)
    } else {
        (format!("{} ORDER BY id DESC LIMIT ?", base), false)
    };
    let mut q = sqlx::query(&sql).bind(channel_id);
    if since_id > 0 {
        q = q.bind(since_id);
    } else if before_id > 0 {
        q = q.bind(before_id);
    }
    q = q.bind(limit);
    let mut messages = Vec::new();
    if let Ok(rs) = q.fetch_all(pool).await {
        for r in rs {
            let raw_extra: String = r.try_get("extra").unwrap_or_default();
            let extra: Value = serde_json::from_str(&raw_extra).unwrap_or(Value::Null);
            messages.push(json!({
                "id": r.try_get::<i64,_>("id").unwrap_or(0),
                "ciyuanxi_id": r.try_get::<String,_>("ciyuanxi_id").unwrap_or_default(),
                "username": r.try_get::<String,_>("username").unwrap_or_default(),
                "avatar_url": r.try_get::<String,_>("avatar_url").unwrap_or_default(),
                "role": r.try_get::<String,_>("role").unwrap_or_default(),
                "msg_type": r.try_get::<String,_>("msg_type").unwrap_or_default(),
                "content": r.try_get::<String,_>("content").unwrap_or_default(),
                "extra": extra,
                "created_at": r.try_get::<String,_>("created_at").unwrap_or_default()
            }));
        }
        if is_before {
            messages.reverse();
        }
    }
    ctx.ok(
        "ok",
        json!({
            "channel": { "id": channel_id, "name": channel_name, "description": channel_desc },
            "messages": messages,
            "total": messages.len()
        }),
    )
}

pub async fn chat_send_message(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let msg_type = str_of(&data, "msg_type").trim().to_string();
    if msg_type != "text" && msg_type != "song_card" {
        return ctx.err(400, "不支持的消息类型");
    }
    let mut content = str_of(&data, "content").trim().to_string();
    if msg_type == "text" && content.is_empty() {
        return ctx.err(400, "消息内容不能为空");
    }
    if content.chars().count() > 1000 {
        content = content.chars().take(1000).collect();
    }
    let raw_extra: Value = data.get("extra").cloned().unwrap_or(Value::Null);
    let extra_json = if raw_extra.is_null() {
        String::new()
    } else {
        serde_json::to_string(&raw_extra).unwrap_or_default()
    };
    let Some((username, email, avatar_url, status)) = resolve_app_user(pool, &ciyuanxi_id).await else {
        return ctx.err(404, "用户不存在");
    };
    if status != 1 {
        return ctx.err(403, "账号已被禁用");
    }
    let (channel_id, _, _) = ensure_main_channel(pool).await;
    let role = resolve_role_by_email(pool, &email).await;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = sqlx::query(
        "INSERT INTO chat_messages (channel_id, ciyuanxi_id, username, avatar_url, role, msg_type, content, extra) VALUES (?,?,?,?,?,?,?,?)",
    )
    .bind(channel_id)
    .bind(&ciyuanxi_id)
    .bind(&username)
    .bind(&avatar_url)
    .bind(&role)
    .bind(&msg_type)
    .bind(&content)
    .bind(&extra_json)
    .execute(pool)
    .await;
    let id = sqlx::query("SELECT LAST_INSERT_ID() AS id")
        .fetch_one(pool)
        .await
        .ok()
        .and_then(|r| r.try_get::<i64, _>("id").ok())
        .unwrap_or(0);
    let return_extra: Value = if extra_json.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&extra_json).unwrap_or(Value::Null)
    };
    ctx.ok(
        "ok",
        json!({
            "id": id,
            "ciyuanxi_id": ciyuanxi_id,
            "username": username,
            "avatar_url": avatar_url,
            "role": role,
            "msg_type": msg_type,
            "content": content,
            "extra": return_extra,
            "created_at": now
        }),
    )
}

pub async fn chat_get_channel_info(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let (id, name, description) = ensure_main_channel(pool).await;
    let mut my_role = "member".to_string();
    if let Some((_, email, _, _)) = resolve_app_user(pool, &ciyuanxi_id).await {
        my_role = resolve_role_by_email(pool, &email).await;
    }
    ctx.ok(
        "ok",
        json!({
            "id": id,
            "name": name,
            "description": description,
            "my_role": my_role
        }),
    )
}

pub async fn chat_update_channel_name(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let mut name = str_of(&data, "name").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    if name.is_empty() {
        return ctx.err(400, "频道名称不能为空");
    }
    if name.chars().count() > 100 {
        name = name.chars().take(100).collect();
    }
    let Some((_, email, _, _)) = resolve_app_user(pool, &ciyuanxi_id).await else {
        return ctx.err(404, "用户不存在");
    };
    let role = resolve_role_by_email(pool, &email).await;
    if role != "admin" && role != "super_admin" {
        return ctx.err(403, "仅管理员或超管可修改频道名称");
    }
    let (channel_id, _, _) = ensure_main_channel(pool).await;
    let _ = sqlx::query("UPDATE chat_channels SET name = ? WHERE id = ?")
        .bind(&name)
        .bind(channel_id)
        .execute(pool)
        .await;
    ctx.ok("ok", json!({ "name": name }))
}

pub async fn chat_get_members(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let admin_map: std::collections::HashMap<String, String> = sqlx::query(
        "SELECT email, role FROM admin_users WHERE status = 1",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .filter_map(|r| {
        r.try_get::<String, _>("email")
            .ok()
            .zip(r.try_get::<String, _>("role").ok())
    })
    .collect();
    let users = sqlx::query(
        "SELECT ciyuanxi_id, username, avatar_url, email, listen_duration FROM app_users WHERE status = 1 AND ciyuanxi_id <> '' ORDER BY listen_duration DESC, id ASC LIMIT 500",
    )
    .fetch_all(pool)
    .await;
    let mut members = Vec::new();
    if let Ok(rs) = users {
        for r in rs {
            let uemail: String = r.try_get("email").unwrap_or_default();
            let role = admin_map
                .get(&uemail)
                .filter(|v| **v == "admin" || **v == "super_admin")
                .cloned()
                .unwrap_or_else(|| "member".to_string());
            members.push(json!({
                "ciyuanxi_id": r.try_get::<String,_>("ciyuanxi_id").unwrap_or_default(),
                "username": r.try_get::<String,_>("username").unwrap_or_default(),
                "avatar_url": r.try_get::<String,_>("avatar_url").unwrap_or_default(),
                "role": role,
                "email": uemail,
                "listen_duration": r.try_get::<i64,_>("listen_duration").unwrap_or(0)
            }));
        }
    }
    ctx.ok("ok", json!({ "members": members }))
}

pub async fn chat_get_remark(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let owner_id = str_of(&data, "owner_ciyuanxi_id").trim().to_string();
    let target_id = str_of(&data, "target_ciyuanxi_id").trim().to_string();
    if owner_id.is_empty() || target_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let row = sqlx::query("SELECT remark FROM chat_user_remarks WHERE owner_ciyuanxi_id = ? AND target_ciyuanxi_id = ? LIMIT 1")
        .bind(&owner_id)
        .bind(&target_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let remark: String = row
        .as_ref()
        .and_then(|r| r.try_get("remark").ok())
        .unwrap_or_default();
    ctx.ok("ok", json!({ "remark": remark }))
}

pub async fn chat_update_remark(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let owner_id = str_of(&data, "owner_ciyuanxi_id").trim().to_string();
    let target_id = str_of(&data, "target_ciyuanxi_id").trim().to_string();
    let mut remark = str_of(&data, "remark").trim().to_string();
    if owner_id.is_empty() || target_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    if remark.chars().count() > 64 {
        remark = remark.chars().take(64).collect();
    }
    let _ = sqlx::query(
        "INSERT INTO chat_user_remarks (owner_ciyuanxi_id, target_ciyuanxi_id, remark) VALUES (?,?,?) ON DUPLICATE KEY UPDATE remark = VALUES(remark)",
    )
    .bind(&owner_id)
    .bind(&target_id)
    .bind(&remark)
    .execute(pool)
    .await;
    ctx.ok("ok", json!({ "remark": remark }))
}

pub async fn chat_get_channel_remark(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let (channel_id, _, _) = ensure_main_channel(pool).await;
    let row = sqlx::query("SELECT remark FROM chat_channel_remarks WHERE ciyuanxi_id = ? AND channel_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .bind(channel_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let remark: String = row
        .as_ref()
        .and_then(|r| r.try_get("remark").ok())
        .unwrap_or_default();
    ctx.ok("ok", json!({ "remark": remark }))
}

pub async fn chat_update_channel_remark(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let mut remark = str_of(&data, "remark").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    if remark.chars().count() > 64 {
        remark = remark.chars().take(64).collect();
    }
    let (channel_id, _, _) = ensure_main_channel(pool).await;
    if remark.is_empty() {
        let _ = sqlx::query("DELETE FROM chat_channel_remarks WHERE ciyuanxi_id = ? AND channel_id = ?")
            .bind(&ciyuanxi_id)
            .bind(channel_id)
            .execute(pool)
            .await;
    } else {
        let _ = sqlx::query(
            "INSERT INTO chat_channel_remarks (ciyuanxi_id, channel_id, remark) VALUES (?,?,?) ON DUPLICATE KEY UPDATE remark = VALUES(remark)",
        )
        .bind(&ciyuanxi_id)
        .bind(channel_id)
        .bind(&remark)
        .execute(pool)
        .await;
    }
    ctx.ok("ok", json!({ "remark": remark }))
}

pub async fn chat_recall_message(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let message_id = int_of(&data, "message_id");
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    if message_id <= 0 {
        return ctx.err(400, "消息ID错误");
    }
    let (channel_id, _, _) = ensure_main_channel(pool).await;
    let Some((_, email, _, _)) = resolve_app_user(pool, &ciyuanxi_id).await else {
        return ctx.err(404, "用户不存在");
    };
    let role = resolve_role_by_email(pool, &email).await;
    let is_admin = role == "admin" || role == "super_admin";
    let msg = sqlx::query("SELECT id, channel_id, ciyuanxi_id, created_at FROM chat_messages WHERE id = ? LIMIT 1")
        .bind(message_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(m) = msg else {
        return ctx.err(404, "消息不存在或已被撤回");
    };
    let m_channel: i64 = m.try_get("channel_id").unwrap_or(0);
    if m_channel != channel_id {
        return ctx.err(403, "不能撤回其他频道的消息");
    }
    if !is_admin {
        let m_ciyuanxi: String = m.try_get("ciyuanxi_id").unwrap_or_default();
        if m_ciyuanxi != ciyuanxi_id {
            return ctx.err(403, "只能撤回自己的消息");
        }
        let created_at: String = m.try_get("created_at").unwrap_or_default();
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%d %H:%M:%S") {
            let now = chrono::Utc::now().naive_utc();
            if (now - dt).num_seconds() > 120 {
                return ctx.err(403, "超过2分钟，无法撤回");
            }
        }
    }
    let _ = sqlx::query("DELETE FROM chat_messages WHERE id = ?")
        .bind(message_id)
        .execute(pool)
        .await;
    ctx.ok("ok", json!({ "message_id": message_id }))
}