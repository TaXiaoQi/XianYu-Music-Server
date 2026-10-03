use axum::body::Body;
use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::Query;
use axum::http::{HeaderMap, Method, Request, StatusCode, Uri};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Router;
use futures_util::StreamExt;
use futures_util::SinkExt;
use serde_json::json;
use sqlx::MySqlPool;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::sync::mpsc;

use super::*;

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
    comm_state().sse_clients.lock().unwrap().clear();
    comm_state().ws_server_clients.lock().unwrap().clear();
    *comm_state().server_running.lock().unwrap() = false;
}

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

async fn sse_handler(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    if !check_token(&query, &headers) {
        return (StatusCode::UNAUTHORIZED, "未授权：token 无效").into_response();
    }
    let events = parse_events(&query);
    let id = uuid::Uuid::new_v4().simple().to_string();
    let (tx, rx) = mpsc::channel::<String>(200);
    comm_state().sse_clients.lock().unwrap().insert(
        id.clone(),
        SseClient {
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
