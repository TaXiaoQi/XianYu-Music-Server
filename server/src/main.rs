mod admin;
mod config;
mod db;
mod handlers;
mod response;
mod schema;
mod sign;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use config::Config;
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Arc<Config>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Arc::new(Config::load()?);
    let pool = db::connect(&config).await?;
    let state = AppState {
        pool,
        config: config.clone(),
    };

    let cors = CorsLayer::permissive();
    let pool = state.pool.clone();

    // 静态文件托管目录（空则不托管）
    let static_dir = if state.config.static_dir.is_empty() {
        "../admin-web/dist".to_string()
    } else {
        state.config.static_dir.clone()
    };
    let index_path = format!("{}/index.html", static_dir);
    let serve_dir = ServeDir::new(&static_dir).fallback(ServeFile::new(&index_path));

    let app = Router::new()
        .route("/api", get(handle_api).post(handle_api))
        .route("/api/", get(handle_api).post(handle_api))
        .route("/admin/api", get(handle_admin_api).post(handle_admin_api))
        .fallback_service(serve_dir)
        .layer(cors)
        .with_state(state);

    let addr = config.listen_addr.clone();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {}", addr);

    let cfg = config.clone();
    let pool2 = pool.clone();
    // 启动后台保证核心表存在（不阻塞 / 不等数据库就绪）
    tokio::spawn(async move {
        db::ping(&cfg, &pool2).await;
        schema::ensure_schema(&pool2).await;
    });

    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_api(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    req: Request<Body>,
) -> Response {
    let action = params.get("action").cloned().unwrap_or_default();
    if action.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")],
            Body::from(r#"{"code":404,"msg":"未知操作","data":null}"#),
        )
            .into_response();
    }

    // 读取原始请求体
    let body_bytes = match axum::body::to_bytes(req.into_body(), 128 * 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(_) => Vec::new(),
    };
    let raw_body = String::from_utf8_lossy(&body_bytes).into_owned();

    let ctx = response::ReqCtx::new((*state.config).clone(), &headers);

    // 签名验证（免签操作放行）
    let no_sign: [&str; 14] = [
        "install", "check", "get_source_status", "upload_avatar", "upload_background",
        "upload_playlist_cover", "debug_sign", "deduct_master_quota", "get_master_quota_usage",
        "email_send_code", "email_register", "email_login", "email_reset_password", "email_get_profile",
    ];
    if !no_sign.contains(&action.as_str()) {
        let timestamp = headers
            .get("x-timestamp")
            .map(|v| v.to_str().unwrap_or("").to_string())
            .unwrap_or_default();
        let nonce = headers
            .get("x-nonce")
            .map(|v| v.to_str().unwrap_or("").to_string())
            .unwrap_or_default();
        let signature = headers
            .get("x-sign")
            .map(|v| v.to_str().unwrap_or("").to_string())
            .unwrap_or_default();
        let secret = &state.config.api_secret;
        let tolerance = state.config.api_timestamp_tolerance;
        if !sign::verify(&timestamp, &nonce, &signature, &raw_body, secret, tolerance) {
            return ctx.err(403, "签名验证失败");
        }
    }

    // 解密请求体
    let body = if let Some(iv) = headers
        .get("x-encrypted-iv")
        .map(|v| v.to_str().unwrap_or("").to_string())
    {
        sign::aes_decrypt(&raw_body, &iv, &state.config.api_secret).unwrap_or_default()
    } else {
        raw_body
    };

    handlers::dispatch(&action, &body, ctx, &state.pool).await
}

/// 后台接口统一入口：`admin_login` 免鉴权，其余需 Bearer JWT
async fn handle_admin_api(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    req: Request<Body>,
) -> Response {
    let action = params.get("action").cloned().unwrap_or_default();
    if action.is_empty() {
        return admin::err(404, "未知操作");
    }

    // 读取原始请求体（后台请求体不加密，直接按 JSON 表单解析）
    let body_bytes = match axum::body::to_bytes(req.into_body(), 64 * 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(_) => Vec::new(),
    };
    let raw_body = String::from_utf8_lossy(&body_bytes).into_owned();
    let ip = admin::client_ip(&headers);

    // 登录接口免鉴权
    if action == "admin_login" {
        return admin::auth::admin_login(&raw_body, &state.config, &state.pool, &ip).await;
    }

    // 其余全部要求 Bearer JWT
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());
    let claims = match admin::verify_token(&state.config, auth_header.as_deref()) {
        Some(c) => c,
        None => return admin::err(401, "未登录或登录已过期"),
    };
    let ctx = admin::AdminCtx {
        id: claims.sub,
        username: claims.username,
        role: claims.role,
        ip,
        config: (*state.config).clone(),
    };
    admin::dispatch(&action, &raw_body, ctx, &state.pool).await
}
