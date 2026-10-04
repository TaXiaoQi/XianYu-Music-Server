use axum::extract::ws::Message as WsMessage;
use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;
use std::time::Duration;
use tokio::sync::mpsc;

use super::*;
use crate::handlers::helpers::{int_of, parse_body, str_of};

pub async fn comm_get_status(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let st = comm_state();
    let running = *st.server_running.lock().unwrap();
    let port = *st.server_port.lock().unwrap();
    let ws_count = st.ws_server_clients.lock().unwrap().len();
    let sse_count = st.sse_clients.lock().unwrap().len();
    let token_enabled = !st.token.lock().unwrap().is_empty();
    let ws_client = {
        let g = st.ws_client.lock().unwrap();
        g.as_ref().map(|c| (c.url.clone(), c.connected_at.clone()))
    };
    let cfg_enabled = read_setting(pool, "commtool_enabled").await == "1";
    let cfg_port = read_setting(pool, "commtool_port").await.parse::<u16>().unwrap_or(8090);
    let cfg_url = read_setting(pool, "ws_client_url").await;
    let cfg_auto = read_setting(pool, "ws_client_auto_reconnect").await == "1";
    let cfg_reconnect = read_setting(pool, "ws_client_reconnect_interval").await;
    let cfg_heartbeat = read_setting(pool, "ws_client_heartbeat_interval").await;
    ok(
        "",
        json!({
            "server_running": running,
            "server_port": port,
            "server_enabled": cfg_enabled,
            "server_port_config": cfg_port,
            "ws_server_count": ws_count,
            "sse_count": sse_count,
            "token_enabled": token_enabled,
            "ws_client": ws_client.map(|(url, ca)| json!({
                "url": url,
                "connected_at": ca,
            })),
            "ws_client_config": json!({
                "url": cfg_url,
                "auto_reconnect": cfg_auto,
                "reconnect_interval": cfg_reconnect,
                "heartbeat_interval": cfg_heartbeat,
            }),
        }),
    )
}

pub async fn comm_service_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let enabled = data.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let port = int_of(&data, "port").clamp(1024, 65535) as u16;
    upsert_setting(pool, "commtool_enabled", if enabled { "1" } else { "0" }, "通信工具服务开关").await;
    upsert_setting(pool, "commtool_port", &port.to_string(), "通信工具服务端口").await;
    super::log_operation(pool, ctx, "更新通信工具服务配置", &format!("{}:{}", if enabled { "启用" } else { "禁用" }, port), "").await;
    ok("已保存", Value::Null)
}

pub async fn comm_http_logs(body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let limit = int_of(&data, "limit");
    let limit = if limit > 0 { limit as usize } else { 100 };
    let logs = comm_state().http_logs.lock().unwrap();
    let arr: Vec<Value> = logs.iter().take(limit).cloned().collect();
    ok("", json!(arr))
}

pub async fn comm_http_clear(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    comm_state().http_logs.lock().unwrap().clear();
    super::log_operation(pool, ctx, "清空通信工具HTTP日志", "", "").await;
    ok("已清空", Value::Null)
}

pub async fn comm_http_client(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let url = str_of(&data, "url").trim().to_string();
    if url.is_empty() {
        return err(400, "请输入请求地址");
    }
    // 防 SSRF：仅允许公网 http(s) 目标
    if let Err(e) = crate::audit_policy::validate_external_endpoint(&url).await {
        return err(400, &format!("地址不可用: {}", e));
    }
    let method = str_of(&data, "method").trim().to_uppercase();
    let method = if method.is_empty() { "GET".to_string() } else { method };
    let headers_raw = str_of(&data, "headers");
    let req_body = str_of(&data, "body");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default();
    let mut req = client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET), &url);

    for line in headers_raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            req = req.header(k.trim(), v.trim());
        }
    }
    if !req_body.trim().is_empty() {
        req = req
            .header("content-type", "application/json")
            .body(req_body.to_string());
    }

    let started = std::time::Instant::now();
    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let resp_headers: serde_json::Map<String, Value> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), json!(v.to_str().unwrap_or(""))))
                .collect();
            let resp_body = resp.text().await.unwrap_or_default();
            let elapsed = started.elapsed().as_millis();
            super::log_operation(pool, ctx, "通信工具HTTP客户端", &url, &format!("{} {}", status, elapsed)).await;
            ok(
                "",
                json!({
                    "status": status,
                    "headers": resp_headers,
                    "body": resp_body,
                    "elapsed_ms": elapsed,
                }),
            )
        }
        Err(e) => { tracing::error!("请求失败: {e}"); err(500, "请求失败") },
    }
}

pub async fn comm_sse_push(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let msg = str_of(&data, "message");
    if msg.is_empty() {
        return err(400, "请输入推送内容");
    }
    push_sse_all(&msg);
    super::log_operation(pool, ctx, "通信工具SSE推送", "", &msg).await;
    ok("已推送", Value::Null)
}

fn push_sse_all(msg: &str) {
    let targets: Vec<mpsc::Sender<String>> = {
        let clients = comm_state().sse_clients.lock().unwrap();
        clients.values().map(|c| c.tx.clone()).collect()
    };
    for tx in targets {
        let _ = tx.try_send(msg.to_string());
    }
}

pub async fn comm_ws_server_list(_body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let clients = comm_state().ws_server_clients.lock().unwrap();
    let arr: Vec<Value> = clients
        .values()
        .map(|c| {
            let mut ev: Vec<&str> = c.events.iter().map(|s| s.as_str()).collect();
            ev.sort();
            json!({
                "id": c.id,
                "addr": c.addr,
                "connected_at": c.connected_at,
                "events": ev,
            })
        })
        .collect();
    ok("", json!(arr))
}

pub async fn comm_ws_server_send(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = str_of(&data, "id");
    let msg = str_of(&data, "message");
    if id.is_empty() || msg.is_empty() {
        return err(400, "参数错误");
    }
    let tx = {
        let clients = comm_state().ws_server_clients.lock().unwrap();
        match clients.get(&id) {
            Some(c) => c.tx.clone(),
            None => return err(404, "连接不存在"),
        }
    };
    let _ = tx.send(WsMessage::Text(msg.clone().into())).await;
    push_log(
        &comm_state().ws_client_logs,
        json!({
            "time": now_str(),
            "direction": "out",
            "client": id,
            "type": "text",
            "data": msg,
        }),
        200,
    );
    super::log_operation(pool, ctx, "通信工具WS发送", &id, &msg).await;
    ok("已发送", Value::Null)
}

pub async fn comm_ws_server_broadcast(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let msg = str_of(&data, "message");
    if msg.is_empty() {
        return err(400, "请输入广播内容");
    }
    let senders: Vec<mpsc::Sender<WsMessage>> = {
        let clients = comm_state().ws_server_clients.lock().unwrap();
        clients.values().map(|c| c.tx.clone()).collect()
    };
    let mut sent = 0usize;
    for tx in senders {
        if tx.send(WsMessage::Text(msg.clone().into())).await.is_ok() {
            sent += 1;
        }
    }
    push_log(
        &comm_state().ws_client_logs,
        json!({
            "time": now_str(),
            "direction": "out",
            "client": "broadcast",
            "type": "text",
            "data": msg,
        }),
        200,
    );
    super::log_operation(pool, ctx, "通信工具WS广播", &format!("{}个连接", sent), &msg).await;
    ok(&format!("已广播到 {} 个连接", sent), Value::Null)
}

pub async fn comm_auth_config(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let token = read_setting(pool, "commtool_token").await;
    ok(
        "",
        json!({
            "token": token,
            "token_enabled": !token.is_empty(),
        }),
    )
}

pub async fn comm_auth_save_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let token = str_of(&data, "token").trim().to_string();
    // 连接鉴权 fail-closed：不允许清空 token（服务监听 0.0.0.0，空 token 等于向公网开放）
    if token.is_empty() {
        return err(400, "token 不能为空，请设置后保存");
    }
    *comm_state().token.lock().unwrap() = token.clone();
    upsert_setting(pool, "commtool_token", &token, "通信工具连接鉴权令牌").await;
    super::log_operation(pool, ctx, "更新连接鉴权配置", "", "设置鉴权令牌").await;
    ok("已保存", Value::Null)
}

// ===================== 外部客户端管理 =====================

pub async fn comm_client_list(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let rows = sqlx::query("SELECT id, name, type, url, events, enabled, created_at FROM comm_clients ORDER BY id DESC")
        .fetch_all(pool)
        .await;
    match rows {
        Ok(rows) => {
            let arr: Vec<Value> = rows
                .iter()
                .map(|r| {
                    json!({
                        "id": r.try_get::<i64, _>("id").unwrap_or(0),
                        "name": r.try_get::<String, _>("name").unwrap_or_default(),
                        "type": r.try_get::<String, _>("type").unwrap_or_default(),
                        "url": r.try_get::<String, _>("url").unwrap_or_default(),
                        "events": r.try_get::<String, _>("events").unwrap_or_default(),
                        "enabled": r.try_get::<i8, _>("enabled").unwrap_or(0) == 1,
                        "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
                    })
                })
                .collect();
            ok("", json!(arr))
        }
        Err(e) => { tracing::error!("数据库错误: {e}"); err(500, "数据库错误") },
    }
}

pub async fn comm_client_add(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let name = str_of(&data, "name").trim().to_string();
    let url = str_of(&data, "url").trim().to_string();
    let client_type = str_of(&data, "type").trim().to_lowercase();
    let events = str_of(&data, "events").trim().to_string();

    if name.is_empty() || url.is_empty() {
        return err(400, "名称和连接地址不能为空");
    }
    if !["ws", "http", "sse"].contains(&client_type.as_str()) {
        return err(400, "类型仅支持 ws / http / sse");
    }

    let res = sqlx::query("INSERT INTO comm_clients (name, type, url, events, enabled) VALUES (?, ?, ?, ?, 1)")
        .bind(&name)
        .bind(&client_type)
        .bind(&url)
        .bind(&events)
        .execute(pool)
        .await;
    match res {
        Ok(_) => {
            super::log_operation(pool, ctx, "添加通信客户端", &name, &format!("类型:{} 地址:{}", client_type, url)).await;
            if client_type == "ws" {
                upsert_setting(pool, "ws_client_url", &url, "WS客户端连接地址").await;
                upsert_setting(pool, "ws_client_auto_reconnect", "1", "WS客户端自动重连开关").await;
                let connected = comm_state().ws_client.lock().unwrap().is_some();
                if !connected {
                    let heartbeat: u64 = read_setting(pool, "ws_client_heartbeat_interval").await.parse().unwrap_or(30);
                    let _ = ws_client_connect_impl(url.clone(), heartbeat).await;
                }
            }
            ok("添加成功", Value::Null)
        }
        Err(e) => { tracing::error!("数据库错误: {e}"); err(500, "数据库错误") },
    }
}

pub async fn comm_client_delete(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let res = sqlx::query("DELETE FROM comm_clients WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await;
    match res {
        Ok(_) => {
            super::log_operation(pool, ctx, "删除通信客户端", &format!("#{}", id), "").await;
            ok("已删除", Value::Null)
        }
        Err(e) => { tracing::error!("数据库错误: {e}"); err(500, "数据库错误") },
    }
}

pub async fn comm_client_toggle(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let enabled = data.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    if id <= 0 {
        return err(400, "参数错误");
    }
    let res = sqlx::query("UPDATE comm_clients SET enabled = ? WHERE id = ?")
        .bind(if enabled { 1 } else { 0 })
        .bind(id)
        .execute(pool)
        .await;
    match res {
        Ok(_) => {
            super::log_operation(pool, ctx, if enabled { "启用通信客户端" } else { "停用通信客户端" }, &format!("#{}", id), "").await;
            ok("已更新", Value::Null)
        }
        Err(e) => { tracing::error!("数据库错误: {e}"); err(500, "数据库错误") },
    }
}
