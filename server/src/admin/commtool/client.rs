use axum::response::Response;
use futures_util::StreamExt;
use futures_util::SinkExt;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use std::time::Duration;
use tokio::sync::mpsc;

use super::*;
use crate::handlers::helpers::{int_of, parse_body, str_of};

pub async fn ws_client_loop(pool: MySqlPool) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let enabled = read_setting(&pool, "ws_client_url").await;
        let auto = read_setting(&pool, "ws_client_auto_reconnect").await == "1";
        if enabled.trim().is_empty() || !auto {
            continue;
        }
        let connected = comm_state().ws_client.lock().unwrap().is_some();
        if connected {
            continue;
        }
        let heartbeat: u64 = read_setting(&pool, "ws_client_heartbeat_interval")
            .await
            .parse()
            .unwrap_or(30);
        let url = enabled.trim().to_string();
        if let Err(e) = ws_client_connect_impl(url.clone(), heartbeat).await {
            tracing::warn!("WS客户端自动重连失败: {} {}", url, e);
        }
    }
}

pub(crate) async fn ws_client_connect_impl(url: String, heartbeat_secs: u64) -> Result<(), String> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| format!("连接失败: {}", e))?;
    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel::<tokio_tungstenite::tungstenite::Message>(100);

    let send_task = tokio::spawn(async move {
        let mut hb = tokio::time::interval(Duration::from_secs(heartbeat_secs.max(1)));
        hb.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Some(m) => {
                            if write.send(m).await.is_err() { break; }
                        }
                        None => break,
                    }
                }
                _ = hb.tick() => {
                    if write
                        .send(tokio_tungstenite::tungstenite::Message::Ping(vec![].into()))
                        .await
                        .is_err()
                    { break; }
                }
            }
        }
    });

    let recv_tx = tx.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            match msg {
                tokio_tungstenite::tungstenite::Message::Text(t) => {
                    push_log(
                        &comm_state().ws_client_logs,
                        json!({
                            "time": now_str(),
                            "direction": "in",
                            "client": "ws-client",
                            "type": "text",
                            "data": t.to_string(),
                        }),
                        200,
                    );
                }
                tokio_tungstenite::tungstenite::Message::Binary(b) => {
                    push_log(
                        &comm_state().ws_client_logs,
                        json!({
                            "time": now_str(),
                            "direction": "in",
                            "client": "ws-client",
                            "type": "binary",
                            "data": format!("{:?}", b),
                        }),
                        200,
                    );
                }
                tokio_tungstenite::tungstenite::Message::Ping(_) => {
                    let _ = recv_tx
                        .send(tokio_tungstenite::tungstenite::Message::Pong(vec![].into()))
                        .await;
                }
                tokio_tungstenite::tungstenite::Message::Close(_) => break,
                _ => {}
            }
        }
        *comm_state().ws_client.lock().unwrap() = None;
    });

    *comm_state().ws_client.lock().unwrap() = Some(WsClientHandle {
        url: url.clone(),
        connected_at: now_str(),
        tx,
    });
    let _ = (send_task, recv_task);
    Ok(())
}

pub async fn comm_ws_client_connect(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let url = str_of(&data, "url").trim().to_string();
    if url.is_empty() {
        return err(400, "请输入连接地址");
    }
    {
        let guard = comm_state().ws_client.lock().unwrap();
        if guard.is_some() {
            return err(400, "已有连接，请先断开");
        }
    }
    let heartbeat: u64 = read_setting(pool, "ws_client_heartbeat_interval")
        .await
        .parse()
        .unwrap_or(30);
    match ws_client_connect_impl(url.clone(), heartbeat).await {
        Ok(()) => {
            super::log_operation(pool, ctx, "通信工具WS客户端连接", &url, "").await;
            ok("连接成功", Value::Null)
        }
        Err(e) => err(500, &e),
    }
}

pub async fn comm_ws_client_send(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let msg = str_of(&data, "message");
    if msg.is_empty() {
        return err(400, "请输入消息内容");
    }
    let tx = {
        let guard = comm_state().ws_client.lock().unwrap();
        match guard.as_ref() {
            Some(h) => h.tx.clone(),
            None => return err(400, "未连接"),
        }
    };
    let _ = tx
        .send(tokio_tungstenite::tungstenite::Message::Text(msg.clone().into()))
        .await;
    push_log(
        &comm_state().ws_client_logs,
        json!({
            "time": now_str(),
            "direction": "out",
            "client": "ws-client",
            "type": "text",
            "data": msg,
        }),
        200,
    );
    super::log_operation(pool, ctx, "通信工具WS客户端发送", "", &msg).await;
    ok("已发送", Value::Null)
}

pub async fn comm_ws_client_disconnect(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let had = {
        let mut guard = comm_state().ws_client.lock().unwrap();
        if guard.is_none() {
            false
        } else {
            guard.take();
            true
        }
    };
    if !had {
        return err(400, "未连接");
    }
    super::log_operation(pool, ctx, "通信工具WS客户端断开", "", "").await;
    ok("已断开", Value::Null)
}

pub async fn comm_ws_client_logs(body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let limit = int_of(&data, "limit");
    let limit = if limit > 0 { limit as usize } else { 100 };
    let logs = comm_state().ws_client_logs.lock().unwrap();
    let arr: Vec<Value> = logs.iter().take(limit).cloned().collect();
    ok("", json!(arr))
}

pub async fn comm_ws_clear(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    comm_state().ws_client_logs.lock().unwrap().clear();
    super::log_operation(pool, ctx, "清空通信工具WS日志", "", "").await;
    ok("已清空", Value::Null)
}

// ===================== Webhook 通知 =====================

pub async fn comm_ws_client_config(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    ok(
        "",
        json!({
            "url": read_setting(pool, "ws_client_url").await,
            "auto_reconnect": read_setting(pool, "ws_client_auto_reconnect").await == "1",
            "reconnect_interval": read_setting(pool, "ws_client_reconnect_interval").await,
            "heartbeat_interval": read_setting(pool, "ws_client_heartbeat_interval").await,
        }),
    )
}

pub async fn comm_ws_client_save_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let url = str_of(&data, "url").trim().to_string();
    let auto_reconnect = data.get("auto_reconnect").and_then(|v| v.as_bool()).unwrap_or(false);
    let reconnect_interval = str_of(&data, "reconnect_interval");
    let reconnect_interval = if reconnect_interval.trim().is_empty() {
        "10"
    } else {
        reconnect_interval.trim()
    };
    let heartbeat_interval = str_of(&data, "heartbeat_interval");
    let heartbeat_interval = if heartbeat_interval.trim().is_empty() {
        "30"
    } else {
        heartbeat_interval.trim()
    };
    upsert_setting(pool, "ws_client_url", &url, "WS客户端连接地址").await;
    upsert_setting(pool, "ws_client_auto_reconnect", if auto_reconnect { "1" } else { "0" }, "WS客户端自动重连开关").await;
    upsert_setting(pool, "ws_client_reconnect_interval", reconnect_interval, "WS客户端重连间隔(秒)").await;
    upsert_setting(pool, "ws_client_heartbeat_interval", heartbeat_interval, "WS客户端心跳间隔(秒)").await;
    super::log_operation(pool, ctx, "更新WS客户端配置", &url, &format!("自动重连:{}, 重连间隔:{}s", auto_reconnect, reconnect_interval)).await;
    ok("已保存", Value::Null)
}

// ===================== 连接鉴权 Token =====================

