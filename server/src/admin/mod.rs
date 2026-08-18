use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Column;
use sqlx::MySqlPool;

pub mod account;
pub mod about;
pub mod agreement;
pub mod admins;
pub mod announcement;
pub mod audit;
pub mod auth;
pub mod config_file;
pub mod dashboard;
pub mod db;
pub mod email;
pub mod feedback;
pub mod logs;
pub mod playlist;
pub mod prettyid;
pub mod proxy;
pub mod share;
pub mod site_config;
pub mod source;
pub mod turnstile;
pub mod users;
pub mod version;
pub mod wallpaper;
pub mod commtool;

/// JWT 载荷：管理员身份
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminClaims {
    pub sub: i64,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

/// 当前登录管理员上下文
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AdminCtx {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub ip: String,
    pub config: crate::config::Config,
    pub base_url: String,
}

/// 签发 JWT（24 小时时效）
pub fn sign_token(cfg: &crate::config::Config, id: i64, username: &str, role: &str) -> String {
    let exp = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 86400) as usize;
    let claims = AdminClaims {
        sub: id,
        username: username.to_string(),
        role: role.to_string(),
        exp,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .unwrap_or_default()
}

/// 解析 Bearer JWT
pub fn verify_token(cfg: &crate::config::Config, header: Option<&str>) -> Option<AdminClaims> {
    let bearer = header?.strip_prefix("Bearer ")?;
    let data = jsonwebtoken::decode::<AdminClaims>(
        bearer,
        &jsonwebtoken::DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .ok()?;
    Some(data.claims)
}

/// 统一 JSON 响应（后台不使用加密）
pub fn json(code: i32, msg: &str, data: Option<Value>) -> Response {
    let payload =
        serde_json::to_string(&json!({ "code": code, "msg": msg, "data": data })).unwrap_or_default();
    let status = if (400..600).contains(&code) {
        StatusCode::from_u16(code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        StatusCode::OK
    };
    (
        status,
        [("content-type", "application/json; charset=utf-8")],
        Body::from(payload),
    )
        .into_response()
}

pub fn ok(msg: &str, data: Value) -> Response {
    json(200, msg, Some(data))
}

pub fn err(code: i32, msg: &str) -> Response {
    json(code, msg, None)
}

/// 代理专用：返回透传 JSON（不强制 {code,msg,data} 结构），附加 http_code / raw
pub fn proxy_response(payload: Option<Value>) -> Response {
    let body = serde_json::to_string(&payload.unwrap_or(Value::Null)).unwrap_or_default();
    (
        StatusCode::OK,
        [("content-type", "application/json; charset=utf-8")],
        Body::from(body),
    )
        .into_response()
}

/// 记录后台操作日志
pub async fn log_operation(pool: &MySqlPool, ctx: &AdminCtx, action: &str, target: &str, detail: &str) {
    let _ = sqlx::query(
        "INSERT INTO admin_operation_log (admin_id, admin_username, action, target, detail, ip) VALUES (?,?,?,?,?,?)",
    )
    .bind(ctx.id)
    .bind(&ctx.username)
    .bind(action)
    .bind(target)
    .bind(detail)
    .bind(&ctx.ip)
    .execute(pool)
    .await;
}

/// 提取客户端 IP
pub fn client_ip(headers: &HeaderMap) -> String {
    crate::sign::get_client_ip(
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
        headers.get("x-real-ip").and_then(|v| v.to_str().ok()),
        None,
    )
}

/// 简单的邮箱格式校验
pub fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() || !email.contains('@') {
        return false;
    }
    let mut parts = email.split('@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    !local.is_empty() && !domain.is_empty() && domain.contains('.')
}

/// 将一行转换为 JSON 对象（列名 -> 值），兼容文本/数字/二进制/NULL
pub fn row_to_value(row: &sqlx::mysql::MySqlRow) -> Value {
    use sqlx::Row;
    let mut map = serde_json::Map::new();
    for i in 0..row.columns().len() {
        let name = row.column(i).name().to_string();
        let v = match row.try_get::<Option<String>, _>(i) {
            Ok(Some(s)) => Value::String(s),
            Ok(None) => Value::Null,
            Err(_) => match row.try_get::<Option<chrono::NaiveDateTime>, _>(i) {
                Ok(Some(dt)) => Value::String(dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                Ok(None) => Value::Null,
                Err(_) => match row.try_get::<i64, _>(i) {
                    Ok(n) => Value::Number(n.into()),
                    Err(_) => match row.try_get::<u32, _>(i) {
                        Ok(n) => Value::Number(n.into()),
                        Err(_) => match row.try_get::<u64, _>(i) {
                            Ok(n) => match serde_json::Number::from_u128(n as u128) {
                                Some(v) => Value::Number(v),
                                None => Value::Null,
                            },
                            Err(_) => match row.try_get::<Option<Vec<u8>>, _>(i) {
                                Ok(Some(b)) => Value::String(String::from_utf8_lossy(&b).into_owned()),
                                Ok(None) => Value::Null,
                                Err(_) => Value::Null,
                            },
                        },
                    },
                },
            },
        };
        map.insert(name, v);
    }
    Value::Object(map)
}

/// 后台 action 分发（仅登录态调用）
pub async fn dispatch(action: &str, body: &str, ctx: AdminCtx, pool: &MySqlPool) -> Response {
    match action {
        // dashboard
        "dashboard_stats" => dashboard::dashboard_stats(body, &ctx, pool).await,
        // users
        "get_users" => users::get_users(body, &ctx, pool).await,
        // auth / account / admins
        "admin_logout" => auth::admin_logout(&ctx, pool).await,
        "change_password" => auth::change_password(body, &ctx, pool).await,
        "change_login" => auth::change_login(body, &ctx, pool).await,
        "list_password_targets" => auth::list_password_targets(body, &ctx, pool).await,
        "upload_admin_avatar" => account::upload_admin_avatar(body, &ctx, pool).await,
        "change_username" => account::change_username(body, &ctx, pool).await,
        "get_account_info" => account::get_account_info(body, &ctx, pool).await,
        "get_about_config_admin" => about::get(body, &ctx, pool).await,
        "save_about_config" => about::save(body, &ctx, pool).await,
        "get_site_logo" => site_config::get_site_logo(body, &ctx, pool).await,
        "upload_site_logo" => site_config::upload_site_logo(body, &ctx, pool).await,
        "get_server_config_file" => config_file::get(body, &ctx, pool).await,
        "save_server_config_file" => config_file::save(body, &ctx, pool).await,
        "migrate_local_cache_to_database" => config_file::migrate_local_cache_to_database(body, &ctx).await,
        "get_user_agreement_admin" => agreement::get(body, &ctx, pool).await,
        "save_user_agreement" => agreement::save(body, &ctx, pool).await,
        "add_admin" => admins::add_admin(body, &ctx, pool).await,
        "delete_admin" => admins::delete_admin(body, &ctx, pool).await,
        "list_admins" => admins::list_admins(body, &ctx, pool).await,
        "toggle_admin_status" => admins::toggle_admin_status(body, &ctx, pool).await,
        // users
        "toggle_user_status" => users::toggle_user_status(body, &ctx, pool).await,
        "batch_toggle_user_status" => users::batch_toggle_user_status(body, &ctx, pool).await,
        "delete_user" => users::delete_user(body, &ctx, pool).await,
        "add_user" => users::add_user(body, &ctx, pool).await,
        "set_user_master_quota" => users::set_user_master_quota(body, &ctx, pool).await,
        "batch_set_master_quota" => users::batch_set_master_quota(body, &ctx, pool).await,
        "delete_user_avatar" => users::delete_user_avatar(body, &ctx, pool).await,
        "change_user_nickname" => users::change_user_nickname(body, &ctx, pool).await,
        "get_user_plugins" => users::get_user_plugins(body, &ctx, pool).await,
        "replace_user_id_to_ciyuanxi" => users::replace_user_id_to_ciyuanxi(body, &ctx, pool).await,
        "change_user_email" => account::change_user_email(body, &ctx, pool).await,
        "reset_listen_duration" => account::reset_listen_duration(body, &ctx, pool).await,
        "change_ciyuanxi_id" => prettyid::change_ciyuanxi_id(body, &ctx, pool).await,
        "list_banned_devices" => users::list_banned_devices(body, &ctx, pool).await,
        "list_all_devices" => users::list_all_devices(body, &ctx, pool).await,
        "ban_device" => users::ban_device(body, &ctx, pool).await,
        "unban_device" => users::unban_device(body, &ctx, pool).await,
        "get_user_devices" => users::get_user_devices(body, &ctx, pool).await,
        "get_device_detail" => users::get_device_detail(body, &ctx, pool).await,
        "reset_device_listen_stats" => users::reset_device_listen_stats(body, &ctx, pool).await,
        "delete_device_record" => users::delete_device_record(body, &ctx, pool).await,
        "batch_delete_devices" => users::batch_delete_devices(body, &ctx, pool).await,
        "get_device_plugins" => users::get_device_plugins(body, &ctx, pool).await,
        // db
        "repair_database" => db::repair_database(body, &ctx, pool).await,
        "view_table" => db::view_table(body, &ctx, pool).await,
        "backup_db" => db::backup_db(body, &ctx, pool).await,
        "view_backup" => db::view_backup(body, &ctx, pool).await,
        "download_backup" => db::download_backup(body, &ctx, pool).await,
        "restore_backup" => db::restore_backup(body, &ctx, pool).await,
        "delete_backup" => db::delete_backup(body, &ctx, pool).await,
        "list_tables" => db::list_tables(body, &ctx, pool).await,
        "list_backups" => db::list_backups(body, &ctx, pool).await,
        "import_db" => db::import_db(body, &ctx, pool).await,
        "get_auto_backup_config" => db::get_auto_backup_config(body, &ctx, pool).await,
        "save_auto_backup_config" => db::save_auto_backup_config(body, &ctx, pool).await,
        // logs / feedback / share
        "list_error_logs" => logs::list_error_logs(body, &ctx, pool).await,
        "get_error_stats" => logs::get_error_stats(body, &ctx, pool).await,
        "get_error_detail" => logs::get_error_detail(body, &ctx, pool).await,
        "delete_error" => logs::delete_error(body, &ctx, pool).await,
        "clear_all_errors" => logs::clear_all_errors(body, &ctx, pool).await,
        "list_app_login_log" => logs::list_app_login_log(body, &ctx, pool).await,
        "list_operation_logs" => logs::list_operation_logs(body, &ctx, pool).await,
        "list_admin_login_logs" => logs::list_admin_login_logs(body, &ctx, pool).await,
        "list_feedback" => feedback::list_feedback(body, &ctx, pool).await,
        "get_feedback_detail" => feedback::get_feedback_detail(body, &ctx, pool).await,
        "update_feedback_status" => feedback::update_feedback_status(body, &ctx, pool).await,
        "claim_feedback" => feedback::claim_feedback(body, &ctx, pool).await,
        "abandon_feedback" => feedback::abandon_feedback(body, &ctx, pool).await,
        "resolve_feedback" => feedback::resolve_feedback(body, &ctx, pool).await,
        "add_collaborator" => feedback::add_collaborator(body, &ctx, pool).await,
        "poll_collab_requests" => feedback::poll_collab_requests(body, &ctx, pool).await,
        "respond_collab_request" => feedback::respond_collab_request(body, &ctx, pool).await,
        "poll_admin_notifications" => feedback::poll_admin_notifications(body, &ctx, pool).await,
        "mark_notifications_read" => feedback::mark_notifications_read(body, &ctx, pool).await,
        "collaborator_complete" => feedback::collaborator_complete(body, &ctx, pool).await,
        "create_feedback" => feedback::create_feedback(body, &ctx, pool).await,
        "feedback_admin_stats" => feedback::feedback_admin_stats(body, &ctx, pool).await,
        "get_feedback_limit" => feedback::get_feedback_limit(body, &ctx, pool).await,
        "update_feedback_limit" => feedback::update_feedback_limit(body, &ctx, pool).await,
        "batch_delete_feedback" => feedback::batch_delete_feedback(body, &ctx, pool).await,
        "list_recycle_bin" => feedback::list_recycle_bin(body, &ctx, pool).await,
        "restore_feedback" => feedback::restore_feedback(body, &ctx, pool).await,
        "view_share_detail" => share::view_share_detail(body, &ctx, pool).await,
        "delete_expired_shares" => share::delete_expired_shares(body, &ctx, pool).await,
        // source / prettyid
        "get_source_config" => source::get_source(body, &ctx, pool).await,
        "toggle_source" => source::toggle_source(body, &ctx, pool).await,
        // version / wallpaper / announcement
        "list_versions" => version::list_versions(body, &ctx, pool).await,
        "add_version" => version::add_version(body, &ctx, pool).await,
        "update_version" => version::update_version(body, &ctx, pool).await,
        "change_version_status" => version::change_version_status(body, &ctx, pool).await,
        "delete_version" => version::delete_version(body, &ctx, pool).await,
        "get_desktop_version" => version::get_desktop_version(body, &ctx, pool).await,
        "save_desktop_version" => version::save_desktop_version(body, &ctx, pool).await,
        "add_wallpaper" => wallpaper::add_wallpaper(body, &ctx, pool).await,
        "list_wallpapers" => wallpaper::list_wallpapers(body, &ctx, pool).await,
        "delete_wallpaper" => wallpaper::delete_wallpaper(body, &ctx, pool).await,
        "change_wallpaper_status" => wallpaper::change_wallpaper_status(body, &ctx, pool).await,
        "get_wallpaper_upload_limit" => wallpaper::get_wallpaper_upload_limit(body, &ctx, pool).await,
        "update_wallpaper_upload_limit" => wallpaper::update_wallpaper_upload_limit(body, &ctx, pool).await,
        "list_wallpaper_account_limits" => wallpaper::list_wallpaper_account_limits(body, &ctx, pool).await,
        "save_wallpaper_account_limit" => wallpaper::save_wallpaper_account_limit(body, &ctx, pool).await,
        "delete_wallpaper_account_limit" => wallpaper::delete_wallpaper_account_limit(body, &ctx, pool).await,
        "add_announcement" => announcement::add(body, &ctx, pool).await,
        "update_announcement" => announcement::update(body, &ctx, pool).await,
        "delete_announcement" => announcement::delete(body, &ctx, pool).await,
        "toggle_announcement" => announcement::toggle(body, &ctx, pool).await,
        "list_announcements" => announcement::list(body, &ctx, pool).await,
        // audit (avatar / nickname / notification email)
        "list_avatar_pending" => audit::list_avatar_pending(body, &ctx, pool).await,
        "list_nickname_pending" => audit::list_nickname_pending(body, &ctx, pool).await,
        "list_audit_records" => audit::list_audit_records(body, &ctx, pool).await,
        "approve_avatar" => audit::approve_avatar(body, &ctx, pool).await,
        "reject_avatar" => audit::reject_avatar(body, &ctx, pool).await,
        "approve_nickname" => audit::approve_nickname(body, &ctx, pool).await,
        "reject_nickname" => audit::reject_nickname(body, &ctx, pool).await,
        "get_audit_external_config" => audit::get_audit_external_config(body, &ctx, pool).await,
        "save_audit_external_config" => audit::save_audit_external_config(body, &ctx, pool).await,
        "test_audit_external_config" => audit::test_audit_external_config(body, &ctx, pool).await,
        "get_banned_words_config" => audit::get_banned_words_config(body, &ctx, pool).await,
        "save_banned_words_config" => audit::save_banned_words_config(body, &ctx, pool).await,
        "test_banned_words" => audit::test_banned_words(body, &ctx, pool).await,
        "list_notification_emails" => email::list_notification_emails(body, &ctx, pool).await,
        "add_notification_email" => email::add_notification_email(body, &ctx, pool).await,
        "update_notification_email" => email::update_notification_email(body, &ctx, pool).await,
        "delete_notification_email" => email::delete_notification_email(body, &ctx, pool).await,
        "toggle_notification_email" => email::toggle_notification_email(body, &ctx, pool).await,
        "test_notification_email" => email::test_notification_email(body, &ctx, pool).await,
        "import_admin_emails" => email::import_admin_emails(body, &ctx, pool).await,
        "get_notification_modules" => email::get_notification_modules(body, &ctx, pool).await,
        "update_notification_modules" => email::update_notification_modules(body, &ctx, pool).await,
        "email_users_list" => email::email_users_list(body, &ctx, pool).await,
        "email_users_toggle" => email::email_users_toggle(body, &ctx, pool).await,
        "email_users_delete" => email::email_users_delete(body, &ctx, pool).await,
        "email_users_logs" => email::email_users_logs(body, &ctx, pool).await,
        "email_users_stats" => email::email_users_stats(body, &ctx, pool).await,
        // email API config
        "get_email_config" => email::get_email_config(body, &ctx, pool).await,
        "update_email_config" => email::update_email_config(body, &ctx, pool).await,
        "test_email_config" => email::test_email_config(body, &ctx, pool).await,
        // comm tool (HTTP / SSE / WebSocket)
        "comm_get_status" => commtool::comm_get_status(body, &ctx, pool).await,
        "comm_service_config" => commtool::comm_service_config(body, &ctx, pool).await,
        "comm_http_logs" => commtool::comm_http_logs(body, &ctx, pool).await,
        "comm_http_clear" => commtool::comm_http_clear(body, &ctx, pool).await,
        "comm_http_client" => commtool::comm_http_client(body, &ctx, pool).await,
        "comm_sse_push" => commtool::comm_sse_push(body, &ctx, pool).await,
        "comm_ws_server_list" => commtool::comm_ws_server_list(body, &ctx, pool).await,
        "comm_ws_server_send" => commtool::comm_ws_server_send(body, &ctx, pool).await,
        "comm_ws_server_broadcast" => commtool::comm_ws_server_broadcast(body, &ctx, pool).await,
        "comm_ws_client_connect" => commtool::comm_ws_client_connect(body, &ctx, pool).await,
        "comm_ws_client_send" => commtool::comm_ws_client_send(body, &ctx, pool).await,
        "comm_ws_client_disconnect" => commtool::comm_ws_client_disconnect(body, &ctx, pool).await,
        "comm_ws_client_logs" => commtool::comm_ws_client_logs(body, &ctx, pool).await,
        "comm_ws_clear" => commtool::comm_ws_clear(body, &ctx, pool).await,
        // ws client 自动重连配置 + 连接鉴权
        "comm_ws_client_config" => commtool::comm_ws_client_config(body, &ctx, pool).await,
        "comm_ws_client_save_config" => commtool::comm_ws_client_save_config(body, &ctx, pool).await,
        "comm_auth_config" => commtool::comm_auth_config(body, &ctx, pool).await,
        "comm_auth_save_config" => commtool::comm_auth_save_config(body, &ctx, pool).await,
        // 外部客户端管理
        "comm_client_list" => commtool::comm_client_list(body, &ctx, pool).await,
        "comm_client_add" => commtool::comm_client_add(body, &ctx, pool).await,
        "comm_client_delete" => commtool::comm_client_delete(body, &ctx, pool).await,
        "comm_client_toggle" => commtool::comm_client_toggle(body, &ctx, pool).await,
        // webhook notification
        "get_webhook_config" => commtool::get_webhook_config(body, &ctx, pool).await,
        "save_webhook_config" => commtool::save_webhook_config(body, &ctx, pool).await,
        "test_webhook" => commtool::test_webhook(body, &ctx, pool).await,
        // captcha config
        "get_captcha_config" => turnstile::get_captcha_config(body, &ctx, pool).await,
        "save_captcha_config" => turnstile::save_captcha_config(body, &ctx, pool).await,
        "get_turnstile_config" => turnstile::get_turnstile_config(body, &ctx, pool).await,
        "save_turnstile_config" => turnstile::save_turnstile_config(body, &ctx, pool).await,
        // playlist
        "get_user_playlists" => playlist::get_user_playlists(body, &ctx, pool).await,
        "delete_user_playlist" => playlist::delete_user_playlist(body, &ctx, pool).await,
        "delete_empty_favorite_playlists" => playlist::delete_empty_favorite_playlists(body, &ctx, pool).await,
        // proxy
        "proxy_api_test" => proxy::proxy_api_test(body, &ctx, pool).await,
        "proxy_app_api_test" => proxy::proxy_app_api_test(body, &ctx, pool).await,
        _ => err(404, "未知操作"),
    }
}
