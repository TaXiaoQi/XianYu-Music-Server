pub(crate) use super::{err, log_operation, ok, AdminCtx};
pub(crate) use super::email;

mod api;
mod client;
mod server;
mod state;
mod webhook;

pub(crate) use state::{check_token, now_str, parse_events, push_log, read_setting};
pub(crate) use client::ws_client_connect_impl;
pub(crate) use webhook::upsert_setting;

pub use api::{
    comm_auth_config, comm_auth_save_config, comm_client_add, comm_client_delete, comm_client_list,
    comm_client_toggle, comm_get_status, comm_http_clear, comm_http_client, comm_http_logs,
    comm_sse_push, comm_service_config, comm_ws_server_broadcast, comm_ws_server_list, comm_ws_server_send,
};
pub use client::{
    comm_ws_clear, comm_ws_client_config, comm_ws_client_connect, comm_ws_client_disconnect,
    comm_ws_client_logs, comm_ws_client_save_config, comm_ws_client_send, ws_client_loop,
};
pub use server::comm_server_loop;
pub use state::{comm_state, SseClient, WsClientHandle, WsServerClient};
pub use webhook::{broadcast_event, get_webhook_config, save_webhook_config, test_webhook};
