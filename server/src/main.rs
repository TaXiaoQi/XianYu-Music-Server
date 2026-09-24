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
mod watch_relay;

use axum::body::Body;
use axum::extract::{Path, Query, State};
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

fn service_unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [("content-type", "application/json; charset=utf-8")],
        Body::from(r#"{"code":503,"msg":"数据库不可用，请稍后重试","data":null}"#),
    )
        .into_response()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let config = Arc::new(Config::load()?);
    config
        .validate_security()
        .map_err(|reason| anyhow::anyhow!("安全配置校验失败，拒绝启动：{reason}"))?;
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

    let _ = std::fs::create_dir_all("uploads/wallpapers");
    let _ = std::fs::create_dir_all("uploads/covers");

    let app = Router::new()
        .route("/api", get(handle_api).post(handle_api))
        .route("/api/", get(handle_api).post(handle_api))
        .route("/s/:share_id", get(share_landing))
        .route("/s/:share_id/", get(share_landing))
        .route(
            "/.well-known/apple-app-site-association",
            get(serve_apple_app_site_association),
        )
        .route("/admin/api", get(handle_admin_api).post(handle_admin_api))
        .route("/admin/api/", get(handle_admin_api).post(handle_admin_api))
        .route("/watch-relay", get(watch_relay::watch_relay_handler))
        .route("/uploads/covers/:filename", get(serve_cover))
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
        tokio::spawn(async move {
            db::ping(&cfg, &pool2).await;
            schema::ensure_schema(&pool2).await;
            handlers::sync::backfill_legacy_sync_files(&pool2).await;
        });
        let auto_pool = pool.clone();
        tokio::spawn(async move {
            admin::db::auto_backup_loop(&auto_pool).await;
        });
        let comm_pool = pool.clone();
        tokio::spawn(async move {
            admin::commtool::comm_server_loop(comm_pool).await;
        });
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

    let body_bytes = match axum::body::to_bytes(req.into_body(), 128 * 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(_) => Vec::new(),
    };
    let raw_body = String::from_utf8_lossy(&body_bytes).into_owned();

    let ctx = response::ReqCtx::new((*state.config).clone(), &headers);

    let no_sign: [&str; 19] = [
        "install", "check", "get_source_status", "upload_avatar",
        "deduct_master_quota", "get_master_quota_usage",
        "get_captcha", "verify_captcha", "email_send_code", "email_get_captcha_config", "email_get_turnstile_config", "email_register", "email_login", "email_reset_password", "email_get_profile",
        "open", "get_user_agreement", "get_site_logo", "share_download",
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
            let ua = headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("-");
            tracing::warn!(
                "sign verify failed: action={} ip={} ua={} ts={:?} skew_s={} body_len={} has_iv={}",
                action,
                admin::client_ip(&headers),
                ua,
                timestamp,
                sign::now_ts() - timestamp.parse::<i64>().unwrap_or(0),
                raw_body.len(),
                headers.contains_key("x-encrypted-iv")
            );
            return ctx.err(403, "签名验证失败");
        }
    }

    let body = if let Some(iv) = headers
        .get("x-encrypted-iv")
        .map(|v| v.to_str().unwrap_or("").to_string())
    {
        sign::aes_decrypt(&raw_body, &iv, &state.config.api_secret).unwrap_or_default()
    } else {
        raw_body
    };

    if !state.db_ready {
        if !state.config.local_debug_no_db {
            return service_unavailable();
        }
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

    let body_bytes = match axum::body::to_bytes(req.into_body(), 384 * 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(_) => Vec::new(),
    };
    let raw_body = String::from_utf8_lossy(&body_bytes).into_owned();
    let ip = admin::client_ip(&headers);

    if action == "admin_login" {
        let req_ctx = response::ReqCtx::new((*state.config).clone(), &headers);
        let pool = if state.db_ready { Some(&state.pool) } else { None };
        if let Some(resp) =
            rate_limit::check_admin_login_rate_limit(&state.rate_limiter, pool, &raw_body, &req_ctx).await
        {
            return resp;
        }
        if !state.db_ready {
            if state.config.local_debug_no_db {
                return debug::handle_admin_login(&raw_body, &state.config);
            }
            return service_unavailable();
        }
        return admin::auth::admin_login(&raw_body, &state.config, &state.pool, &ip).await;
    }

    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());
    let claims = match admin::verify_token(
        &state.config,
        auth_header.as_deref(),
        if state.db_ready { Some(&state.pool) } else { None },
    )
    .await
    {
        Some(c) => c,
        None => return admin::err(401, "未登录或登录已过期"),
    };
    if !state.db_ready {
        if state.config.local_debug_no_db {
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
        return service_unavailable();
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

async fn serve_apple_app_site_association() -> Response {
    static AASA: &str = include_str!("apple-app-site-association.json");
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        AASA,
    )
        .into_response()
}

async fn spa_fallback(State(state): State<AppState>, req: Request<Body>) -> Response {
    let static_dir = state.config.static_dir.clone();
    let index_file = format!(
        "{}/index.html",
        static_dir.trim_end_matches(|c| c == '/' || c == '\\')
    );

    let path = req.uri().path().trim_start_matches('/');
    let file_path = if path.contains('.') {
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
        index_file.clone()
    };

    match tokio::fs::read(&file_path).await {
        Ok(bytes) => {
            let ctype = mime_of(&file_path);
            let is_html = ctype.starts_with("text/html");
            let mut hm = axum::http::HeaderMap::new();
            hm.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_str(ctype).expect("valid content-type"),
            );
            hm.insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static(if is_html {
                    "no-cache"
                } else {
                    "public, max-age=31536000, immutable"
                }),
            );
            (StatusCode::OK, hm, bytes).into_response()
        }
        Err(_) => {
            match tokio::fs::read(&index_file).await {
                Ok(bytes) => (
                    StatusCode::OK,
                    [
                        (axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8"),
                        (axum::http::header::CACHE_CONTROL, "no-cache"),
                    ],
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

async fn serve_cover(
    Path(filename): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    let file_path = std::path::Path::new("uploads/covers").join(&filename);
    let Ok(bytes) = std::fs::read(&file_path) else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    let w = params
        .get("w")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    if (1..=2000).contains(&w) {
        if let Ok(img) = image::load_from_memory(&bytes) {
            let (ow, oh) = (img.width(), img.height());
            if ow > 0 && w < ow {
                let h = ((oh as f32 * w as f32 / ow as f32).round().max(1.0)) as u32;
                let resized = img.resize(w, h, image::imageops::FilterType::Triangle);
                let rgb = resized.to_rgb8();
                let mut out = std::io::Cursor::new(Vec::new());
                let mut enc =
                    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 82);
                let _ = enc.encode(
                    &rgb,
                    rgb.width(),
                    rgb.height(),
                    image::ExtendedColorType::Rgb8,
                );
                return (
                    StatusCode::OK,
                    [
                        (axum::http::header::CONTENT_TYPE, "image/jpeg"),
                        (axum::http::header::CACHE_CONTROL, "public, max-age=86400"),
                    ],
                    out.into_inner(),
                )
                    .into_response();
            }
        }
    }
    let ext = filename.rsplit('.').next().unwrap_or("png");
    let mime = mime_of(ext);
    (StatusCode::OK, [(axum::http::header::CONTENT_TYPE, mime)], bytes).into_response()
}

async fn share_landing(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(share_id): Path<String>,
) -> Response {
    if !state.db_ready {
        return share_404();
    }
    let row = sqlx::query("SELECT * FROM share_log WHERE share_id = ? AND expired_at > NOW()")
        .bind(&share_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten();
    let Some(row) = row else {
        return share_404();
    };

    let _ = sqlx::query("UPDATE share_log SET view_count = view_count + 1 WHERE share_id = ?")
        .bind(&share_id)
        .execute(&state.pool)
        .await;

    let _ = sqlx::query("INSERT INTO share_views (share_id) VALUES (?)")
        .bind(&share_id)
        .execute(&state.pool)
        .await;

    let row_val = crate::admin::row_to_value(&row);
    let params_str = row_val
        .get("request_params")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let params_val = serde_json::from_str(params_str).unwrap_or(serde_json::Value::Null);

    let base = response::ReqCtx::new((*state.config).clone(), &headers)
        .base_url
        .trim_end_matches('/')
        .to_string();
    let download_api = format!("{}/api?action=share_download", base);
    let html = handlers::share::render_landing_page(&row_val, &params_val, &download_api);
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8"), (axum::http::header::CACHE_CONTROL, "no-cache")],
        Body::from(html),
    )
        .into_response()
}

fn share_404() -> Response {
    (
        StatusCode::NOT_FOUND,
        [
            (axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (axum::http::header::CACHE_CONTROL, "no-store"),
        ],
        Body::from("分享不存在或已过期"),
    )
        .into_response()
}
