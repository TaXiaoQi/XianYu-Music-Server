use axum::extract::ws::Message as WsMessage;
use axum::http::HeaderMap;
use serde_json::Value;
use sqlx::MySqlPool;
use sqlx::Row;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc;


// ===================== 全局通信工具状态 =====================

#[derive(Clone)]
pub struct CommState {
    pub http_logs: Arc<Mutex<VecDeque<Value>>>,
    pub sse_clients: Arc<Mutex<HashMap<String, SseClient>>>,
    pub ws_server_clients: Arc<Mutex<HashMap<String, WsServerClient>>>,
    pub ws_client: Arc<Mutex<Option<WsClientHandle>>>,
    pub ws_client_logs: Arc<Mutex<VecDeque<Value>>>,
    pub server_running: Arc<Mutex<bool>>,
    pub server_port: Arc<Mutex<u16>>,
    pub token: Arc<Mutex<String>>,
}

pub struct WsServerClient {
    pub id: String,
    pub addr: String,
    pub connected_at: String,
    pub events: HashSet<String>,
    pub tx: mpsc::Sender<WsMessage>,
}

pub struct SseClient {
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

pub(crate) fn check_token(query: &HashMap<String, String>, headers: &HeaderMap) -> bool {
    let expected = comm_state().token.lock().unwrap().clone();
    if expected.is_empty() {
        return true;
    }
    if let Some(t) = query.get("token") {
        if t == &expected {
            return true;
        }
    }
    if let Some(v) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(s) = v.to_str() {
            if let Some(bearer) = s.strip_prefix("Bearer ") {
                if bearer == expected {
                    return true;
                }
            }
        }
    }
    if let Some(v) = headers.get("x-token") {
        if let Ok(s) = v.to_str() {
            if s == expected {
                return true;
            }
        }
    }
    false
}

pub(crate) fn parse_events(query: &HashMap<String, String>) -> HashSet<String> {
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

pub(crate) fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub(crate) fn push_log(queue: &Mutex<VecDeque<Value>>, entry: Value, max: usize) {
    let mut q = queue.lock().unwrap();
    q.push_front(entry);
    if q.len() > max {
        q.truncate(max);
    }
}

// ===================== 通信工具服务（独立端口） =====================

pub(crate) async fn read_setting(pool: &MySqlPool, key: &str) -> String {
    sqlx::query("SELECT setting_value FROM server_settings WHERE setting_key = ? LIMIT 1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .and_then(|r| r.try_get::<Option<String>, _>(0).ok().flatten())
        .unwrap_or_default()
}