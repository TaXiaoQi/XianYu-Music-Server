mod admin;
mod audit_policy;
mod config;
mod db;
mod debug;
mod handlers;
mod rate_limit;
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
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Arc<Config>,
    pub db_ready: bool,
    pub rate_limiter: Arc<rate_limit::ApiRateLimiter>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Arc::new(Config::load()?);
    let pool = db::connect(&config).await?;
    let db_ready = if config.local_debug_no_db {
        false
    } else {
        match tokio::time::timeout(Duration::from_secs(3), sqlx::query("SELECT 1").execute(&pool)).await {
            Ok(Ok(_)) => true,
            Ok(Err(e)) => {
                tracing::warn!("database unavailable, fallback to local cache mode: {}", e);
                false
            }
            Err(_) => {
                tracing::warn!("database unavailable, fallback to local cache mode: connection timeout");
                false
            }
        }
    };
    let state = AppState {
        pool,
        config: config.clone(),
        db_ready,
        rate_limiter: Arc::new(rate_limit::ApiRateLimiter::default()),
    };

    let cors = CorsLayer::permissive();
    let pool = state.pool.clone();
    let static_dir = config.static_dir.clone();

    // 确保 uploads 目录及子目录存在
    let _ = std::fs::create_dir_all("uploads/wallpapers");

    let app = Router::new()
        .route("/api", get(handle_api).post(handle_api))
        .route("/api/", get(handle_api).post(handle_api))
        .route("/admin/api", get(handle_admin_api).post(handle_admin_api))
        .route("/admin/api/", get(handle_admin_api).post(handle_admin_api))
        .nest_service("/uploads", ServeDir::new("uploads"))
        .fallback(spa_fallback)
        .layer(cors)
        .with_state(state);

    let addr = config.listen_addr.clone();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {}", addr);

    let cfg = config.clone();
    let pool2 = pool.clone();
    if !db_ready {
        tracing::warn!("database is not ready: use local cache mode and skip schema initialization");
    } else {
        // 启动后台保证核心表存在（不阻塞 / 不等数据库就绪）
        tokio::spawn(async move {
            db::ping(&cfg, &pool2).await;
            schema::ensure_schema(&pool2).await;
        });
        // 启动后台自动备份调度任务
        let auto_pool = pool.clone();
        tokio::spawn(async move {
            admin::db::auto_backup_loop(&auto_pool).await;
        });
        // 启动通信工具服务调度任务（HTTP/SSE/WebSocket，按配置动态启停）
        let comm_pool = pool.clone();
        tokio::spawn(async move {
            admin::commtool::comm_server_loop(comm_pool).await;
        });
        // 启动 WS 客户端自动重连调度任务
        let ws_pool = pool.clone();
        tokio::spawn(async move {
            admin::commtool::ws_client_loop(ws_pool).await;
        });
    }

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
    let no_sign: [&str; 19] = [
        "install", "check", "get_source_status", "upload_avatar",
        "debug_sign", "deduct_master_quota", "get_master_quota_usage",
        "get_captcha", "verify_captcha", "email_send_code", "email_get_captcha_config", "email_get_turnstile_config", "email_register", "email_login", "email_reset_password", "email_get_profile",
        "open", "get_user_agreement", "get_site_logo",
    ];
    if !state.config.local_debug_no_db && !no_sign.contains(&action.as_str()) {
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

    if !state.db_ready {
        if let Some(resp) = rate_limit::check_api_rate_limit(
            &state.rate_limiter,
            None,
            &action,
            &body,
            &ctx,
        )
        .await
        {
            return resp;
        }
        return debug::handle_api(&action, &body, ctx);
    }

    if let Some(resp) = rate_limit::check_api_rate_limit(
        &state.rate_limiter,
        Some(&state.pool),
        &action,
        &body,
        &ctx,
    )
    .await
    {
        return resp;
    }

    handlers::dispatch(&action, &body, ctx, &state.pool).await
}

/// 从请求头构造 base_url，用于后台拼接完整图片 URL
/// 优先使用请求头中的 Host，若无法获取则使用 config.public_base_url 兜底
fn build_base_url(headers: &HeaderMap, config: &Config) -> String {
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(axum::http::header::HOST))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if host.is_empty() {
        return config.public_base_url.clone();
    }
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').next().unwrap_or(v).trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "http".to_string());
    format!("{}://{}", scheme, host)
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
    let body_bytes = match axum::body::to_bytes(req.into_body(), 384 * 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(_) => Vec::new(),
    };
    let raw_body = String::from_utf8_lossy(&body_bytes).into_owned();
    let ip = admin::client_ip(&headers);

    // 登录接口免鉴权
    if action == "admin_login" {
        if !state.db_ready {
            return debug::handle_admin_login(&raw_body, &state.config);
        }
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
    if !state.db_ready {
        let base_url = build_base_url(&headers, &state.config);
        let config_ctx = admin::AdminCtx {
            id: claims.sub,
            username: claims.username,
            role: claims.role,
            ip: ip.clone(),
            config: (*state.config).clone(),
            base_url,
        };
        return match action.as_str() {
            "get_server_config_file" => admin::config_file::get_no_db(&raw_body, &config_ctx).await,
            "save_server_config_file" => admin::config_file::save_no_db(&raw_body, &config_ctx).await,
            "migrate_local_cache_to_database" => admin::config_file::migrate_local_cache_to_database(&raw_body, &config_ctx).await,
            _ => debug::handle_admin_api(&action),
        };
    }
    let base_url = build_base_url(&headers, &state.config);
    let ctx = admin::AdminCtx {
        id: claims.sub,
        username: claims.username,
        role: claims.role,
        ip,
        config: (*state.config).clone(),
        base_url,
    };
    if state.config.local_debug_no_db {
        return debug::handle_admin_api(&action);
    }
    admin::dispatch(&action, &raw_body, ctx, &state.pool).await
}

/// SPA 静态资源 fallback：
/// - 已存在的静态文件（/assets/*、/logo.png 等）正常返回
/// - 其余路径（SPA 深层路由，如 /dashboard、/m/dashboard）返回 index.html 且状态码 200，
///   避免前端 history 路由刷新时 404
async fn spa_fallback(State(state): State<AppState>, req: Request<Body>) -> Response {
    let static_dir = state.config.static_dir.clone();
    let index_file = format!(
        "{}/index.html",
        static_dir.trim_end_matches(|c| c == '/' || c == '\\')
    );

    let path = req.uri().path().trim_start_matches('/');
    let file_path = if path.contains('.') {
        // 有扩展名的资源路径：映射到 static_dir 下，防路径穿越
        let mut base = static_dir.trim_end_matches(|c| c == '/' || c == '\\').to_string();
        for seg in path.split('/') {
            if seg.is_empty() || seg == "." {
                continue;
            }
            if seg == ".." {
                return (
                    StatusCode::BAD_REQUEST,
                    [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    "bad request",
                )
                    .into_response();
            }
            base.push('/');
            base.push_str(seg);
        }
        base
    } else {
        // 无扩展名：SPA 深层路由，返回 index.html
        index_file.clone()
    };

    match tokio::fs::read(&file_path).await {
        Ok(bytes) => {
            let ctype = mime_of(&file_path);
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, ctype)],
                bytes,
            )
                .into_response()
        }
        Err(_) => {
            // 资源不存在时仍回退 index.html（200），保证 SPA 可用
            match tokio::fs::read(&index_file).await {
                Ok(bytes) => (
                    StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    bytes,
                )
                    .into_response(),
                Err(_) => (
                    StatusCode::NOT_FOUND,
                    [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    "not found",
                )
                    .into_response(),
            }
        }
    }
}

fn mime_of(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        "txt" => "text/plain; charset=utf-8",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "exe" => "application/octet-stream",
        "apk" => "application/vnd.android.package-archive",
        _ => "application/octet-stream",
    }
}
