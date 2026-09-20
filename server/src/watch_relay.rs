//! 腕上端云端兜底通道（P4）：手机 ↔ 服务器 ↔ 手表 的 WebSocket 中继。
//!
//! 设计（对应 watch-app-plan.md P4，协议预留 type 0x40–0x7F）：
//! - 手机与手表各自与本服务建立 WS 长连接，凭 256-bit 随机 device_key 配对
//!   （key 经蓝牙链路的 `cloud_bind` 帧自动下发，两端各持一份即可互信）；
//! - 握手：连接后 10s 内第一条必须是 Text `{"op":"hello","role":"phone|watch","key":"<64hex>","name":"..."}`，
//!   校验失败立即断开；
//! - 配对成功后互发 `{"op":"ready","peer":<对端名>}`，任一端离开互发 `{"op":"peer_lost"}`，
//!   同角色重复连接踢旧留新（旧连接收 `{"op":"replaced"}`）；
//! - XYW1 二进制帧（客户端帧编解码后的完整帧字节）以 WS Binary 消息原样转发给对端，
//!   服务端不解析帧内容（无隐私暴露面）；心跳沿用 XYW1 的手表 3s ping（兼作 NAT 保活）；
//! - 房间（每 key 一间）数量上限 [MAX_ROOMS]，防连接滥用。
//!
//! 部署：nginx 需为本路由加 WebSocket upgrade 反代（见仓库 nginx.conf `/watch-relay`）。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::mpsc;

const MAX_ROOMS: usize = 4096;

const HELLO_TIMEOUT: Duration = Duration::from_secs(10);

const CHANNEL_CAPACITY: usize = 256;

// ===================== 房间状态 =====================

struct Peer {
    conn_id: u64,
    name: String,
    tx: mpsc::Sender<WsMessage>,
}

fn next_conn_id() -> u64 {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Default)]
struct Room {
    phone: Option<Peer>,
    watch: Option<Peer>,
}

fn rooms() -> &'static Mutex<HashMap<String, Room>> {
    static ROOMS: OnceLock<Mutex<HashMap<String, Room>>> = OnceLock::new();
    ROOMS.get_or_init(|| Mutex::new(HashMap::new()))
}

// ===================== 路由入口 =====================

pub async fn watch_relay_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

struct Hello {
    role: String,
    key: String,
    name: String,
}

fn parse_hello(text: &str) -> Option<Hello> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    if v.get("op")?.as_str()? != "hello" {
        return None;
    }
    let role = v.get("role")?.as_str()?.to_string();
    if role != "phone" && role != "watch" {
        return None;
    }
    let key = v.get("key")?.as_str()?.to_ascii_lowercase();
    if key.len() != 64 || !key.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return None;
    }
    let name = v
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or_default()
        .chars()
        .take(64)
        .collect::<String>();
    Some(Hello { role, key, name })
}

async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // ---- 握手：限时等待第一条 Text ----
    let first = tokio::time::timeout(HELLO_TIMEOUT, receiver.next()).await;
    let hello = match first {
        Ok(Some(Ok(WsMessage::Text(t)))) => parse_hello(&t),
        _ => None,
    };
    let Some(hello) = hello else {
        let _ = sender
            .send(WsMessage::Text(
                json!({"op": "error", "msg": "invalid hello"}).to_string().into(),
            ))
            .await;
        return;
    };

    // ---- 注册进房间（锁内不做任何 await）----
    let (tx, mut rx) = mpsc::channel::<WsMessage>(CHANNEL_CAPACITY);
    let conn_id = next_conn_id();
    enum Register {
        Full,
        Ok(Option<(String, mpsc::Sender<WsMessage>)>),
    }
    let outcome = {
        let mut rooms = match rooms().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let room_count = rooms.len();
        let existed = rooms.contains_key(&hello.key);
        if !existed && room_count >= MAX_ROOMS {
            Register::Full
        } else {
            let room = rooms.entry(hello.key.clone()).or_default();
            let slot = if hello.role == "phone" {
                &mut room.phone
            } else {
                &mut room.watch
            };
            if let Some(old) = slot.take() {
                let _ = old.tx.try_send(WsMessage::Text(
                    json!({"op": "replaced"}).to_string().into(),
                ));
            }
            *slot = Some(Peer {
                conn_id,
                name: hello.name.clone(),
                tx: tx.clone(),
            });
            let peer = if hello.role == "phone" {
                room.watch.as_ref()
            } else {
                room.phone.as_ref()
            };
            Register::Ok(peer.map(|p| (p.name.clone(), p.tx.clone())))
        }
    };
    let peer_ready = match outcome {
        Register::Full => {
            let _ = sender
                .send(WsMessage::Text(
                    json!({"op": "error", "msg": "relay full"}).to_string().into(),
                ))
                .await;
            return;
        }
        Register::Ok(peer) => peer,
    };

    // ---- outboard 泵：rx -> sender ----
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    if let Some((peer_name, peer_tx)) = peer_ready {
        let _ = peer_tx
            .try_send(WsMessage::Text(
                json!({"op": "ready", "peer": hello.name}).to_string().into(),
            ))
            .ok();
        let _ = tx
            .try_send(WsMessage::Text(
                json!({"op": "ready", "peer": peer_name}).to_string().into(),
            ))
            .ok();
    }

    // ---- 收循环：Binary 原样转发，Text 仅处理应用层 ping ----
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            WsMessage::Binary(bytes) => {
                let peer_tx = {
                    let rooms = match rooms().lock() {
                        Ok(g) => g,
                        Err(_) => break,
                    };
                    let Some(room) = rooms.get(&hello.key) else {
                        break;
                    };
                    let peer = if hello.role == "phone" {
                        room.watch.as_ref()
                    } else {
                        room.phone.as_ref()
                    };
                    peer.map(|p| p.tx.clone())
                };
                match peer_tx {
                    Some(tx) => {
                        if tx.send(WsMessage::Binary(bytes)).await.is_err() {
                        }
                    }
                    None => {
                    }
                }
            }
            WsMessage::Text(t) => {
                if t.contains("\"ping\"") {
                    let _ = tx
                        .send(WsMessage::Text(json!({"op": "pong"}).to_string().into()))
                        .await;
                }
            }
            WsMessage::Close(_) => break,
            _ => {}
        }
    }

    // ---- 清理：移除自身并通知对端 ----
    send_task.abort();
    let (still_mine, room_empty, peer_tx) = {
        let mut rooms = match rooms().lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        let Some(room) = rooms.get_mut(&hello.key) else {
            return;
        };
        let slot = if hello.role == "phone" {
            &mut room.phone
        } else {
            &mut room.watch
        };
        let still_mine = slot.as_ref().map(|p| p.conn_id == conn_id).unwrap_or(false);
        if still_mine {
            *slot = None;
        }
        let peer_tx = if still_mine {
            let peer = if hello.role == "phone" {
                room.watch.as_ref()
            } else {
                room.phone.as_ref()
            };
            peer.map(|p| p.tx.clone())
        } else {
            None
        };
        (still_mine, room.phone.is_none() && room.watch.is_none(), peer_tx)
    };
    if room_empty {
        rooms().lock().ok().map(|mut g| g.remove(&hello.key));
    }
    if still_mine {
        if let Some(peer_tx) = peer_tx {
            let _ = peer_tx
                .send(WsMessage::Text(
                    json!({"op": "peer_lost"}).to_string().into(),
                ))
                .await;
        }
    }
}
