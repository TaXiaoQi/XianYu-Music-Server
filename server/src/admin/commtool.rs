use axum::body::Body;
use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::Query;
use axum::http::{HeaderMap, Method, Request, StatusCode, Uri};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Router;
use futures_util::StreamExt;
use futures_util::SinkExt;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::mpsc;

use super::{err, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

// ===================== 全局通信工具状态 =====================

#[derive(Clone)]
pub struct CommState {
    /// HTTP 服务器收到的请求日志（最新在前）
    pub http_logs: Arc<Mutex<VecDeque<Value>>>,
    /// SSE 订阅连接（每连接独立 channel + 订阅事件集合）
    pub sse_clients: Arc<Mutex<HashMap<String, SseClient>>>,
    /// WS 服务器连接列表
    pub ws_server_clients: Arc<Mutex<HashMap<String, WsServerClient>>>,
    /// WS 客户端（连接外部服务）
    pub ws_client: Arc<Mutex<Option<WsClientHandle>>>,
    /// WS 客户端收到的消息日志
    pub ws_client_logs: Arc<Mutex<VecDeque<Value>>>,
    /// 通信工具服务是否正在运行
    pub server_running: Arc<Mutex<bool>>,
    pub server_port: Arc<Mutex<u16>>,
    /// 连接鉴权令牌（空 = 不开启鉴权）
    pub token: Arc<Mutex<String>>,
}

pub struct WsServerClient {
    pub id: String,
    pub addr: String,
    pub connected_at: String,
    /// 订阅的事件类型（空集合 = 订阅全部事件）
    pub events: HashSet<String>,
    pub tx: mpsc::Sender<WsMessage>,
}

pub struct SseClient {
    pub id: String,
    pub connected_at: String,
    /// 订阅的事件类型（空集合 = 订阅全部事件）
    pub events: HashSet<String>,
    pub tx: mpsc::Sender<String>,
}

pub struct WsClientHandle {
    pub url: String,
    pub connected_at: String,
    pub tx: mpsc::Sender<tokio_tungstenite::tungstenite::Message>,
}

pub static COMM_STATE: OnceLock<CommState> = OnceLock::new();

pub fn comm_state() -> &'static CommState {
    COMM_STATE.get_or_init(|| {
        CommState {
            http_logs: Arc::new(Mutex::new(VecDeque::new())),
            sse_clients: Arc::new(Mutex::new(HashMap::new())),
            ws_server_clients: Arc::new(Mutex::new(HashMap::new())),
            ws_client: Arc::new(Mutex::new(None)),
            ws_client_logs: Arc::new(Mutex::new(VecDeque::new())),
            server_running: Arc::new(Mutex::new(false)),
            server_port: Arc::new(Mutex::new(0)),
            token: Arc::new(Mutex::new(String::new())),
        }
    })
}

/// 连接鉴权：校验 token。token 为空表示不开启鉴权。
/// 支持 query 参数 `token`、请求头 `Authorization: Bearer <token>`、请求头 `X-Token`。
fn check_token(query: &HashMap<String, String>, headers: &HeaderMap) -> bool {
    let expected = comm_state().token.lock().unwrap().clone();
    if expected.is_empty() {
        return true;
    }
    // query ?token=
    if let Some(t) = query.get("token") {
        if t == &expected {
            return true;
        }
    }
    // Authorization: Bearer xxx
    if let Some(v) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(s) = v.to_str() {
            if let Some(bearer) = s.strip_prefix("Bearer ") {
                if bearer == expected {
                    return true;
                }
            }
        }
    }
    // X-Token
    if let Some(v) = headers.get("x-token") {
        if let Ok(s) = v.to_str() {
            if s == expected {
                return true;
            }
        }
    }
    false
}

/// 解析事件订阅列表（query 参数 events，逗号分隔；空 = 订阅全部）
fn parse_events(query: &HashMap<String, String>) -> HashSet<String> {
    let mut set = HashSet::new();
    if let Some(ev) = query.get("events") {
        for e in ev.split(',') {
            let e = e.trim();
            if !e.is_empty() {
                set.insert(e.to_string());
            }
        }
    }
    set
}

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn push_log(queue: &Mutex<VecDeque<Value>>, entry: Value, max: usize) {
    let mut q = queue.lock().unwrap();
    q.push_front(entry);
    if q.len() > max {
        q.truncate(max);
    }
}

// ===================== 通信工具服务（独立端口） =====================

/// 后台循环：每 10s 检查 server_settings 配置，动态启停通信工具服务并同步鉴权令牌
pub async fn comm_server_loop(pool: MySqlPool) {
    let mut running = false;
    let mut abort: Option<tokio::task::JoinHandle<()>> = None;
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let enabled = read_setting(&pool, "commtool_enabled").await == "1";
        let port = read_setting(&pool, "commtool_port")
            .await
            .parse::<u16>()
            .unwrap_or(8090);
        // 同步连接鉴权令牌
        let token = read_setting(&pool, "commtool_token").await;
        *comm_state().token.lock().unwrap() = token;
        if enabled && !running {
            let h = tokio::spawn(async move {
                run_comm_server(port).await;
            });
            abort = Some(h);
            running = true;
            *comm_state().server_running.lock().unwrap() = true;
            *comm_state().server_port.lock().unwrap() = port;
        } else if !enabled && running {
            if let Some(h) = abort.take() {
                h.abort();
            }
            running = false;
            *comm_state().server_running.lock().unwrap() = false;
        }
    }
}

/// 后台循环：WS 客户端自动重连（参考 napcat 反向 WS）。
/// 每 reconnect_interval 秒检查一次：配置了 url 且开启自动重连且当前未连接，则自动连接。
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

async fn read_setting(pool: &MySqlPool, key: &str) -> String {
    sqlx::query("SELECT setting_value FROM server_settings WHERE setting_key = ? LIMIT 1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .and_then(|r| r.try_get::<Option<String>, _>(0).ok().flatten())
        .unwrap_or_default()
}

async fn run_comm_server(port: u16) {
    let app = Router::new()
        .route("/sse", axum::routing::get(sse_handler))
        .route("/ws", axum::routing::get(ws_server_handler))
        .fallback(http_server_handler)
        .with_state(());

    let addr = format!("0.0.0.0:{}", port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("通信工具服务启动失败: {}", e);
            *comm_state().server_running.lock().unwrap() = false;
            return;
        }
    };
    tracing::info!("通信工具服务已启动，监听 {}", addr);
    if let Err(e) = axum::serve(listener, app).await {
        tracing::warn!("通信工具服务退出: {}", e);
    }
    // 服务退出时清空连接与状态
    comm_state().sse_clients.lock().unwrap().clear();
    comm_state().ws_server_clients.lock().unwrap().clear();
    *comm_state().server_running.lock().unwrap() = false;
}

/// 解析 query 字符串为键值对（简单解码，支持 token/events）
fn parse_query_str(q: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in q.split('&') {
        if pair.is_empty() {
            continue;
        }
        if let Some((k, v)) = pair.split_once('=') {
            map.insert(url_decode(k).unwrap_or_default(), url_decode(v).unwrap_or_default());
        }
    }
    map
}

fn url_decode(s: &str) -> Option<String> {
    let bytes: Vec<u8> = s.replace('+', " ").as_bytes().to_vec();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// HTTP 服务器：校验鉴权后记录所有收到的请求
async fn http_server_handler(
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    req: Request<Body>,
) -> Response {
    let query: HashMap<String, String> = match uri.query() {
        Some(q) => parse_query_str(q),
        None => HashMap::new(),
    };
    if !check_token(&query, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            [(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")],
            Body::from(r#"{"code":401,"msg":"未授权：token 无效"}"#),
        )
            .into_response();
    }
    let body_bytes = axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024)
        .await
        .unwrap_or_default();
    let body_str = String::from_utf8_lossy(&body_bytes).into_owned();
    let mut headers_map = serde_json::Map::new();
    for (k, v) in headers.iter() {
        if let Ok(s) = v.to_str() {
            headers_map.insert(k.to_string(), json!(s));
        }
    }
    let entry = json!({
        "time": now_str(),
        "method": method.as_str(),
        "path": uri.path(),
        "query": uri.query().unwrap_or(""),
        "headers": headers_map,
        "body": body_str,
    });
    push_log(&comm_state().http_logs, entry, 200);
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Body::from(r#"{"code":0,"msg":"ok"}"#),
    )
        .into_response()
}

/// SSE 服务器：校验鉴权，按订阅事件类型接收广播
async fn sse_handler(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if !check_token(&query, &headers) {
        return (StatusCode::UNAUTHORIZED, "未授权：token 无效").into_response();
    }
    let events = parse_events(&query);
    let id = uuid::Uuid::new_v4().simple().to_string();
    let (tx, mut rx) = mpsc::channel::<String>(200);
    comm_state().sse_clients.lock().unwrap().insert(
        id.clone(),
        SseClient {
            id: id.clone(),
            connected_at: now_str(),
            events,
            tx,
        },
    );

    let stream = futures_util::stream::unfold((id, rx), |(cid, mut rx)| async move {
        loop {
            match rx.recv().await {
                Some(msg) => {
                    return Some((Ok::<Event, std::convert::Infallible>(Event::default().data(msg)), (cid, rx)));
                }
                None => {
                    comm_state().sse_clients.lock().unwrap().remove(&cid);
                    return None;
                }
            }
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default()).into_response()
}

/// WebSocket 服务器：校验鉴权，支持事件订阅
async fn ws_server_handler(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    ws: WebSocketUpgrade,
) -> Response {
    if !check_token(&query, &headers) {
        return (StatusCode::UNAUTHORIZED, "未授权：token 无效").into_response();
    }
    let events = parse_events(&query);
    ws.on_upgrade(move |socket| handle_ws_server(socket, events))
}

async fn handle_ws_server(socket: WebSocket, events: HashSet<String>) {
    let id = uuid::Uuid::new_v4().simple().to_string();
    let (tx, mut rx) = mpsc::channel::<WsMessage>(100);
    let addr = "ws-server".to_string();
    comm_state()
        .ws_server_clients
        .lock()
        .unwrap()
        .insert(
            id.clone(),
            WsServerClient {
                id: id.clone(),
                addr: addr.clone(),
                connected_at: now_str(),
                events,
                tx,
            },
        );

    let (mut sender, mut receiver) = socket.split();

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            WsMessage::Text(t) => {
                push_log(
                    &comm_state().ws_client_logs,
                    json!({
                        "time": now_str(),
                        "direction": "in",
                        "client": addr,
                        "type": "text",
                        "data": t.to_string(),
                    }),
                    200,
                );
            }
            WsMessage::Binary(b) => {
                push_log(
                    &comm_state().ws_client_logs,
                    json!({
                        "time": now_str(),
                        "direction": "in",
                        "client": addr,
                        "type": "binary",
                        "data": format!("{:?}", b),
                    }),
                    200,
                );
            }
            WsMessage::Close(_) => break,
            _ => {}
        }
    }

    comm_state().ws_server_clients.lock().unwrap().remove(&id);
    send_task.abort();
}

// ===================== Admin API =====================

/// 获取通信工具状态
pub async fn comm_get_status(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let st = comm_state();
    let running = *st.server_running.lock().unwrap();
    let port = *st.server_port.lock().unwrap();
    let ws_count = st.ws_server_clients.lock().unwrap().len();
    let sse_count = st.sse_clients.lock().unwrap().len();
    let token_enabled = !st.token.lock().unwrap().is_empty();
    // 提前提取 ws_client 信息，避免 MutexGuard 跨 await 点
    let ws_client = {
        let g = st.ws_client.lock().unwrap();
        g.as_ref().map(|c| (c.url.clone(), c.connected_at.clone()))
    };
    // 读取配置（await）
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

/// 保存通信工具服务配置（启用状态 + 监听端口）
pub async fn comm_service_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let enabled = data.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let port = int_of(&data, "port").clamp(1024, 65535) as u16;
    upsert_setting(pool, "commtool_enabled", if enabled { "1" } else { "0" }, "通信工具服务开关").await;
    upsert_setting(pool, "commtool_port", &port.to_string(), "通信工具服务端口").await;
    super::log_operation(pool, ctx, "更新通信工具服务配置", &format!("{}:{}", if enabled { "启用" } else { "禁用" }, port), "").await;
    ok("已保存", Value::Null)
}

/// 获取 HTTP 服务器收到的请求日志
pub async fn comm_http_logs(body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let limit = int_of(&data, "limit");
    let limit = if limit > 0 { limit as usize } else { 100 };
    let logs = comm_state().http_logs.lock().unwrap();
    let arr: Vec<Value> = logs.iter().take(limit).cloned().collect();
    ok("", json!(arr))
}

/// 清空 HTTP 服务器日志
pub async fn comm_http_clear(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    comm_state().http_logs.lock().unwrap().clear();
    super::log_operation(pool, ctx, "清空通信工具HTTP日志", "", "").await;
    ok("已清空", Value::Null)
}

/// HTTP 客户端：发送 HTTP 请求
pub async fn comm_http_client(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let url = str_of(&data, "url").trim().to_string();
    if url.is_empty() {
        return err(400, "请输入请求地址");
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

    // 解析 headers（每行 "Key: Value"）
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
        Err(e) => err(500, &format!("请求失败: {}", e)),
    }
}

/// SSE 推送（手动广播给所有 SSE 连接）
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

/// 向所有 SSE 连接推送消息
fn push_sse_all(msg: &str) {
    let targets: Vec<mpsc::Sender<String>> = {
        let clients = comm_state().sse_clients.lock().unwrap();
        clients.values().map(|c| c.tx.clone()).collect()
    };
    for tx in targets {
        let _ = tx.try_send(msg.to_string());
    }
}

/// WS 服务器连接列表
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

/// WS 服务器向指定连接发送消息
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

/// WS 服务器广播
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

/// WS 客户端：连接外部服务（可复用，供手动连接与自动重连共用）
async fn ws_client_connect_impl(url: String, heartbeat_secs: u64) -> Result<(), String> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| format!("连接失败: {}", e))?;
    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel::<tokio_tungstenite::tungstenite::Message>(100);

    // 发送任务（含心跳保活）
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

    // 接收任务（记录消息，响应 Ping，断线清理状态）
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

/// WS 客户端：连接外部服务（手动）
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

/// WS 客户端：发送消息
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

/// WS 客户端：断开连接
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

/// WS 客户端消息日志
pub async fn comm_ws_client_logs(body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let limit = int_of(&data, "limit");
    let limit = if limit > 0 { limit as usize } else { 100 };
    let logs = comm_state().ws_client_logs.lock().unwrap();
    let arr: Vec<Value> = logs.iter().take(limit).cloned().collect();
    ok("", json!(arr))
}

/// 清空 WS 消息日志
pub async fn comm_ws_clear(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    comm_state().ws_client_logs.lock().unwrap().clear();
    super::log_operation(pool, ctx, "清空通信工具WS日志", "", "").await;
    ok("已清空", Value::Null)
}

// ===================== Webhook 通知 =====================

const WH_KEY_ENABLED: &str = "webhook_enabled";
const WH_KEY_URL: &str = "webhook_url";
const WH_KEY_METHOD: &str = "webhook_method";
const WH_KEY_HEADERS: &str = "webhook_headers";
const WH_KEY_BODY_TEMPLATE: &str = "webhook_body_template";
const WH_KEY_MODULES: &str = "webhook_modules";

/// 获取 Webhook 配置
pub async fn get_webhook_config(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let enabled = read_setting(pool, WH_KEY_ENABLED).await == "1";
    let url = read_setting(pool, WH_KEY_URL).await;
    let method = read_setting(pool, WH_KEY_METHOD).await;
    let headers = read_setting(pool, WH_KEY_HEADERS).await;
    let body_template = read_setting(pool, WH_KEY_BODY_TEMPLATE).await;
    let modules_raw = read_setting(pool, WH_KEY_MODULES).await;
    let mut modules = json!({});
    for m in super::email::NOTIFY_MODULES.iter() {
        let key = format!("wh_{}", m);
        let on = modules_raw
            .split(',')
            .any(|s| s.trim() == *m);
        modules[&key] = json!(on);
    }
    ok(
        "",
        json!({
            "enabled": enabled,
            "url": url,
            "method": if method.is_empty() { "POST".to_string() } else { method },
            "headers": headers,
            "body_template": body_template,
            "modules": modules,
        }),
    )
}

/// 保存 Webhook 配置
pub async fn save_webhook_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let enabled = data.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let url = str_of(&data, "url").trim().to_string();
    let method = str_of(&data, "method").trim().to_uppercase();
    let method = if method.is_empty() { "POST".to_string() } else { method };
    let headers = str_of(&data, "headers");
    let body_template = str_of(&data, "body_template");

    if enabled && url.is_empty() {
        return err(400, "启用 Webhook 时请输入回调地址");
    }
    let mut modules_on: Vec<String> = Vec::new();
    if let Some(mods) = data.get("modules").and_then(|v| v.as_object()) {
        for (k, v) in mods {
            if let Some(m) = k.strip_prefix("wh_") {
                if v.as_bool().unwrap_or(false) {
                    modules_on.push(m.to_string());
                }
            }
        }
    }
    upsert_setting(pool, WH_KEY_ENABLED, if enabled { "1" } else { "0" }, "通用Webhook开关").await;
    upsert_setting(pool, WH_KEY_URL, &url, "通用Webhook回调地址").await;
    upsert_setting(pool, WH_KEY_METHOD, &method, "通用Webhook请求方法").await;
    upsert_setting(pool, WH_KEY_HEADERS, &headers, "通用Webhook自定义请求头").await;
    upsert_setting(pool, WH_KEY_BODY_TEMPLATE, &body_template, "通用Webhook请求体模板").await;
    upsert_setting(pool, WH_KEY_MODULES, &modules_on.join(","), "通用Webhook触发板块").await;
    super::log_operation(pool, ctx, "更新Webhook配置", &url, &format!("方法:{} 板块:{}", method, modules_on.join(","))).await;
    ok("已保存", Value::Null)
}

async fn upsert_setting(pool: &MySqlPool, key: &str, value: &str, desc: &str) {
    let _ = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), description = VALUES(description)",
    )
    .bind(key)
    .bind(value)
    .bind(desc)
    .execute(pool)
    .await;
}

/// 测试 Webhook：发送一条测试通知
pub async fn test_webhook(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let url = str_of(&data, "url").trim().to_string();
    let method = str_of(&data, "method").trim().to_uppercase();
    let method = if method.is_empty() { "POST".to_string() } else { method };
    let headers_raw = str_of(&data, "headers");
    let body_template = str_of(&data, "body_template");

    if url.is_empty() {
        return err(400, "请输入回调地址");
    }
    let payload = if body_template.trim().is_empty() {
        json!({
            "event": "test",
            "message": "这是一条测试通知",
            "time": now_str(),
        })
        .to_string()
    } else {
        render_template(&body_template, "test", "测试通知", "", "", "")
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    let mut req = client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::POST), &url);
    for line in headers_raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            req = req.header(k.trim(), v.trim());
        }
    }
    if !payload.is_empty() {
        req = req
            .header("content-type", "application/json")
            .body(payload);
    }
    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let resp_body = resp.text().await.unwrap_or_default();
            super::log_operation(pool, ctx, "测试Webhook", &url, &format!("{} {}", status, resp_body)).await;
            ok("", json!({ "status": status, "body": resp_body }))
        }
        Err(e) => err(500, &format!("请求失败: {}", e)),
    }
}

/// 渲染 Webhook 请求体模板（替换占位符）
pub fn render_template(template: &str, event: &str, title: &str, detail: &str, image_url: &str, link: &str) -> String {
    let mut s = template.to_string();
    let map = [
        ("{{event}}", event),
        ("{{title}}", title),
        ("{{detail}}", detail),
        ("{{image_url}}", image_url),
        ("{{link}}", link),
        ("{{time}}", &now_str()),
    ];
    for (k, v) in map {
        s = s.replace(k, v);
    }
    s
}

/// 触发 Webhook 通知（供审核/反馈等事件调用）
pub async fn notify_webhook(
    pool: &MySqlPool,
    module: &str,
    title: &str,
    detail: &str,
    image_url: &str,
    link: &str,
) {
    if read_setting(pool, WH_KEY_ENABLED).await != "1" {
        return;
    }
    let modules_raw = read_setting(pool, WH_KEY_MODULES).await;
    if !modules_raw.split(',').any(|s| s.trim() == module) {
        return;
    }
    let url = read_setting(pool, WH_KEY_URL).await;
    if url.is_empty() {
        return;
    }
    let method = read_setting(pool, WH_KEY_METHOD).await;
    let method = if method.is_empty() { "POST".to_string() } else { method };
    let headers_raw = read_setting(pool, WH_KEY_HEADERS).await;
    let body_template = read_setting(pool, WH_KEY_BODY_TEMPLATE).await;
    let payload = if body_template.trim().is_empty() {
        json!({
            "event": module,
            "title": title,
            "detail": detail,
            "image_url": image_url,
            "link": link,
            "time": now_str(),
        })
        .to_string()
    } else {
        render_template(&body_template, module, title, detail, image_url, link)
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_default();
    let mut req = client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::POST), &url);
    for line in headers_raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            req = req.header(k.trim(), v.trim());
        }
    }
    if !payload.is_empty() {
        req = req
            .header("content-type", "application/json")
            .body(payload);
    }
    let _ = req.send().await;
}

// ===================== 统一事件广播（参考 napcat 事件分发） =====================

/// 统一事件入口：将内部事件同时分发到多个通道。
/// 1. Webhook（HTTP 上报）
/// 2. WS 服务器已订阅该事件的连接
/// 3. SSE 已订阅该事件的连接
/// `module` 为事件类型（如 wallpaper/avatar/nickname/feedback）。
pub async fn broadcast_event(
    pool: &MySqlPool,
    module: &str,
    title: &str,
    detail: &str,
    image_url: &str,
    link: &str,
) {
    // 1. Webhook
    notify_webhook(pool, module, title, detail, image_url, link).await;

    // 2. WS 服务器订阅者
    let payload = json!({
        "event": module,
        "title": title,
        "detail": detail,
        "image_url": image_url,
        "link": link,
        "time": now_str(),
        "channel": "event",
    })
    .to_string();
    let ws_targets: Vec<mpsc::Sender<WsMessage>> = {
        let clients = comm_state().ws_server_clients.lock().unwrap();
        clients
            .values()
            .filter(|c| c.events.is_empty() || c.events.contains(module))
            .map(|c| c.tx.clone())
            .collect()
    };
    for tx in ws_targets {
        let _ = tx.send(WsMessage::Text(payload.clone().into())).await;
    }

    // 3. SSE 订阅者
    let sse_targets: Vec<mpsc::Sender<String>> = {
        let clients = comm_state().sse_clients.lock().unwrap();
        clients
            .values()
            .filter(|c| c.events.is_empty() || c.events.contains(module))
            .map(|c| c.tx.clone())
            .collect()
    };
    for tx in sse_targets {
        let _ = tx.try_send(payload.clone());
    }
}

// ===================== WS 客户端自动重连配置 =====================

/// 获取 WS 客户端配置
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

/// 保存 WS 客户端配置
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

/// 获取连接鉴权配置
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

/// 保存连接鉴权配置（token 为空 = 关闭鉴权）
pub async fn comm_auth_save_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let token = str_of(&data, "token").trim().to_string();
    *comm_state().token.lock().unwrap() = token.clone();
    upsert_setting(pool, "commtool_token", &token, "通信工具连接鉴权令牌（空=不鉴权）").await;
    super::log_operation(pool, ctx, "更新连接鉴权配置", "", if token.is_empty() { "关闭鉴权" } else { "设置鉴权令牌" }).await;
    ok("已保存", Value::Null)
}

// ===================== 外部客户端管理 =====================

/// 获取已添加的外部客户端列表
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
        Err(e) => err(500, &format!("数据库错误: {}", e)),
    }
}

/// 添加外部客户端
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
            // WS 类型客户端：同步为当前 WS 客户端配置并尝试连接
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
        Err(e) => err(500, &format!("数据库错误: {}", e)),
    }
}

/// 删除外部客户端
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
        Err(e) => err(500, &format!("数据库错误: {}", e)),
    }
}

/// 启用/停用外部客户端
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
        Err(e) => err(500, &format!("数据库错误: {}", e)),
    }
}
