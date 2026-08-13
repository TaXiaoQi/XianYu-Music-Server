use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, is_valid_email, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{bool_of, int_of, parse_body, str_of};

/// 通知板块白名单（用于动态列名与全局设置，防止注入）
pub const NOTIFY_MODULES: [&str; 4] = ["wallpaper", "avatar", "nickname", "feedback"];

async fn ensure_email_tables(pool: &MySqlPool) {
    for t in crate::schema::table_statements() {
        if t.contains("`notification_emails`") || t.contains("`email_test_users`") {
            let _ = sqlx::query(t).execute(pool).await;
        }
    }
    for m in NOTIFY_MODULES.iter() {
        let _ = sqlx::query(&format!("ALTER TABLE `notification_emails` ADD COLUMN IF NOT EXISTS `notify_{}` tinyint(1) NOT NULL DEFAULT 1", m))
            .execute(pool)
            .await;
    }
}

/// 读取某板块的全局通知开关（server_settings），默认开启
pub async fn notify_module_enabled(pool: &MySqlPool, module: &str) -> bool {
    let key = format!("notify_module_{}", module);
    sqlx::query_scalar::<_, String>("SELECT setting_value FROM server_settings WHERE setting_key = ?")
        .bind(&key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|v| v != "0")
        .unwrap_or(true)
}

/// 向开启该板块通知的邮箱发送邮件，返回成功发送的邮箱列表
#[allow(dead_code)]
pub async fn notify_external_emails_for_module(
    pool: &MySqlPool,
    config: &crate::config::Config,
    ip: &str,
    module: &str,
    subject: &str,
    context: &str,
) -> Vec<String> {
    if !NOTIFY_MODULES.contains(&module) {
        return Vec::new();
    }
    if !notify_module_enabled(pool, module).await {
        return Vec::new();
    }
    let col = format!("notify_{}", module);
    let q = format!("SELECT email FROM notification_emails WHERE status = 1 AND {} = 1", col);
    let rows = sqlx::query(&q).fetch_all(pool).await.unwrap_or_default();
    let mut sent: Vec<String> = Vec::new();
    for r in rows {
        let email: String = r.try_get("email").unwrap_or_default();
        if email.is_empty() || !is_valid_email(&email) {
            continue;
        }
        match crate::handlers::email_auth::call_email_api(config, pool, subject, context, &email).await {
            Ok(_) => {
                let _ = sqlx::query("INSERT INTO email_send_log (email, subject, interface_id, template_id, status, error_msg, ip) VALUES (?,?,0,0,1,'',?)")
                    .bind(&email).bind(subject).bind(ip).execute(pool).await;
                sent.push(email);
            }
            Err(e) => {
                let _ = sqlx::query("INSERT INTO email_send_log (email, subject, interface_id, template_id, status, error_msg, ip) VALUES (?,?,0,0,2,?,?)")
                    .bind(&email).bind(subject).bind(&e).bind(ip).execute(pool).await;
            }
        }
    }
    sent
}

/// 通知邮箱列表
pub async fn list_notification_emails(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    ensure_email_tables(pool).await;
    let list = sqlx::query("SELECT id, email, remark, status, notify_wallpaper, notify_avatar, notify_nickname, notify_feedback, created_at FROM notification_emails ORDER BY id ASC")
        .fetch_all(pool)
        .await;
    match list {
        Ok(rows) => {
            let arr: Vec<Value> = rows.iter().map(row_to_email).collect();
            log_operation(pool, ctx, "查看通知邮箱", "", "").await;
            ok("", json!(arr))
        }
        Err(_) => super::err(500, "数据库错误"),
    }
}

fn row_to_email(r: &sqlx::mysql::MySqlRow) -> Value {
    json!({
        "id": r.get::<i64, _>("id"),
        "email": r.get::<String, _>("email"),
        "remark": r.get::<String, _>("remark"),
        "status": r.get::<i64, _>("status"),
        "notify_wallpaper": r.try_get::<i64, _>("notify_wallpaper").unwrap_or(1),
        "notify_avatar": r.try_get::<i64, _>("notify_avatar").unwrap_or(1),
        "notify_nickname": r.try_get::<i64, _>("notify_nickname").unwrap_or(1),
        "notify_feedback": r.try_get::<i64, _>("notify_feedback").unwrap_or(1),
        "created_at": r.try_get::<String, _>("created_at").unwrap_or_else(|_| "".into()),
    })
}

/// 新增通知邮箱
pub async fn add_notification_email(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let remark = str_of(&data, "remark").trim().to_string();
    if email.is_empty() || !is_valid_email(&email) {
        return err(400, "邮箱格式不正确");
    }
    ensure_email_tables(pool).await;
    let dup = sqlx::query("SELECT id FROM notification_emails WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if dup.is_some() {
        return err(400, "该邮箱已存在");
    }
    let nw = bool_of(&data, "notify_wallpaper");
    let na = bool_of(&data, "notify_avatar");
    let nn = bool_of(&data, "notify_nickname");
    let nf = bool_of(&data, "notify_feedback");
    let ins = sqlx::query("INSERT INTO notification_emails (email, remark, status, notify_wallpaper, notify_avatar, notify_nickname, notify_feedback) VALUES (?, ?, 1, ?, ?, ?, ?)")
        .bind(&email)
        .bind(&remark)
        .bind(nw as i32)
        .bind(na as i32)
        .bind(nn as i32)
        .bind(nf as i32)
        .execute(pool)
        .await;
    match ins {
        Ok(_) => {
            log_operation(pool, ctx, "新增通知邮箱", &email, &remark).await;
            ok("添加成功", Value::Null)
        }
        Err(_) => err(500, "数据库错误"),
    }
}

/// 更新通知邮箱的备注与板块开关
pub async fn update_notification_email(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let remark = str_of(&data, "remark").trim().to_string();
    let nw = bool_of(&data, "notify_wallpaper");
    let na = bool_of(&data, "notify_avatar");
    let nn = bool_of(&data, "notify_nickname");
    let nf = bool_of(&data, "notify_feedback");
    let upd = sqlx::query("UPDATE notification_emails SET remark = ?, notify_wallpaper = ?, notify_avatar = ?, notify_nickname = ?, notify_feedback = ? WHERE id = ?")
        .bind(&remark)
        .bind(nw as i32)
        .bind(na as i32)
        .bind(nn as i32)
        .bind(nf as i32)
        .bind(id)
        .execute(pool)
        .await;
    match upd {
        Ok(r) if r.rows_affected() > 0 => {
            log_operation(pool, ctx, "更新通知邮箱设置", &format!("ID:{}", id), &format!("壁纸:{} 头像:{} 昵称:{} 反馈:{}", nw, na, nn, nf)).await;
            ok("已更新", Value::Null)
        }
        Ok(_) => err(404, "邮箱不存在"),
        Err(_) => err(500, "数据库错误"),
    }
}

/// 快捷导入管理员邮箱
pub async fn import_admin_emails(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    ensure_email_tables(pool).await;
    let rows = sqlx::query("SELECT username, email FROM admin_users WHERE email IS NOT NULL AND TRIM(email) <> ''")
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    let mut imported: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    for r in rows {
        let username: String = r.try_get("username").unwrap_or_default();
        let email: String = r.try_get("email").unwrap_or_default();
        let email = email.trim().to_string();
        if email.is_empty() || !is_valid_email(&email) {
            continue;
        }
        let dup = sqlx::query("SELECT id FROM notification_emails WHERE email = ?")
            .bind(&email)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        if dup.is_some() {
            skipped.push(email);
            continue;
        }
        let _ = sqlx::query("INSERT INTO notification_emails (email, remark, status, notify_wallpaper, notify_avatar, notify_nickname, notify_feedback) VALUES (?,?,1,1,1,1,1)")
            .bind(&email)
            .bind(format!("管理员：{}", username))
            .execute(pool)
            .await;
        imported.push(email);
    }
    log_operation(pool, ctx, "导入管理员邮箱", "", &format!("导入:{}个 跳过:{}个", imported.len(), skipped.len())).await;
    ok(&format!("成功导入 {} 个管理员邮箱{}", imported.len(), if skipped.is_empty() { String::new() } else { format!("，{} 个已存在跳过", skipped.len()) }), json!({ "imported": imported, "skipped": skipped }))
}

/// 获取全局通知板块设置
pub async fn get_notification_modules(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let mut modules = json!({});
    for m in NOTIFY_MODULES.iter() {
        modules[m] = json!(notify_module_enabled(pool, m).await);
    }
    ok("", modules)
}

/// 保存全局通知板块设置
pub async fn update_notification_modules(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut changed: Vec<String> = Vec::new();
    for m in NOTIFY_MODULES.iter() {
        let key = format!("notify_module_{}", m);
        if let Some(v) = data.get(m).and_then(|x| x.as_bool()) {
            let _ = sqlx::query("INSERT INTO server_settings (setting_key, setting_value, description) VALUES (?, ?, '') ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value)")
                .bind(&key)
                .bind(if v { "1" } else { "0" })
                .execute(pool)
                .await;
            changed.push(format!("{}:{}", m, if v { "开" } else { "关" }));
        }
    }
    log_operation(pool, ctx, "更新通知板块设置", "", &changed.join(" ")).await;
    ok("已保存", Value::Null)
}

/// 删除通知邮箱
pub async fn delete_notification_email(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let row = sqlx::query("SELECT email FROM notification_emails WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let email: String = r.get("email");
            let _ = sqlx::query("DELETE FROM notification_emails WHERE id = ?").bind(id).execute(pool).await;
            log_operation(pool, ctx, "删除通知邮箱", &email, "").await;
            ok("删除成功", Value::Null)
        }
        None => err(404, "邮箱不存在"),
    }
}

/// 启用/禁用通知邮箱
pub async fn toggle_notification_email(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let upd = sqlx::query("UPDATE notification_emails SET status = 1 - status WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await;
    match upd {
        Ok(_) => {
            log_operation(pool, ctx, "切换通知邮箱状态", &format!("ID:{}", id), "").await;
            ok("操作成功", Value::Null)
        }
        Err(_) => err(500, "数据库错误"),
    }
}

/// 发送测试邮件（此处记录日志，实际发送依赖邮箱接口配置）
pub async fn test_notification_email(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    if email.is_empty() || !is_valid_email(&email) {
        return err(400, "邮箱格式不正确");
    }
    // 尚无真实 SMTP 对接，模拟发送成功并记录
    let _ = sqlx::query("INSERT INTO email_send_log (email, subject, interface_id, template_id, status, error_msg, ip) VALUES (?,?,0,0,1,'',?)")
        .bind(&email)
        .bind("【弦予后台】通知邮箱测试")
        .bind(&ctx.ip)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "发送通知邮箱测试", &email, "").await;
    ok("测试邮件已发送，请查收", Value::Null)
}

/// 邮箱测试用户列表（分页）
pub async fn email_users_list(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page");
    let page = if page < 1 { 1 } else { page };
    let keyword = str_of(&data, "keyword").trim().to_string();
    let page_size = 15;
    let offset = (page - 1) * page_size;
    let (where_sql, kw) = if keyword.is_empty() {
        (String::new(), String::new())
    } else {
        let kw = format!("%{}%", keyword);
        (" WHERE email LIKE ? OR nickname LIKE ?".to_string(), kw)
    };
    let total: i64 = if where_sql.is_empty() {
        sqlx::query_scalar("SELECT COUNT(*) FROM email_test_users").fetch_one(pool).await.unwrap_or(0)
    } else {
        sqlx::query_scalar(&format!("SELECT COUNT(*) FROM email_test_users {}", where_sql))
            .bind(&kw).bind(&kw).fetch_one(pool).await.unwrap_or(0)
    };
    let q = if where_sql.is_empty() {
        format!("SELECT id, email, nickname, status, created_at, last_login FROM email_test_users ORDER BY id DESC LIMIT {} OFFSET {}", page_size, offset)
    } else {
        format!("SELECT id, email, nickname, status, created_at, last_login FROM email_test_users {} ORDER BY id DESC LIMIT {} OFFSET {}", where_sql, page_size, offset)
    };
    let rows_result = if where_sql.is_empty() {
        sqlx::query(&q).fetch_all(pool).await
    } else {
        sqlx::query(&q).bind(&kw).bind(&kw).fetch_all(pool).await
    };
    match rows_result {
        Ok(rows) => {
            let arr: Vec<Value> = rows.iter().map(|r| json!({
                "id": r.get::<i64, _>("id"),
                "email": r.get::<String, _>("email"),
                "nickname": r.get::<String, _>("nickname"),
                "status": r.get::<i64, _>("status"),
                "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
                "last_login": r.try_get::<Option<String>, _>("last_login").ok().flatten().unwrap_or_default(),
            })).collect();
            let pages = ((total as f64) / (page_size as f64)).ceil() as i64;
            ok("", json!({ "data": arr, "total": total, "page": page, "pages": if pages < 1 { 1 } else { pages } }))
        }
        Err(_) => err(500, "查询失败"),
    }
}

/// 切换邮箱测试用户状态
pub async fn email_users_toggle(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let upd = sqlx::query("UPDATE email_test_users SET status = 1 - status WHERE id = ?").bind(id).execute(pool).await;
    match upd {
        Ok(_) => {
            log_operation(pool, ctx, "邮箱测试用户-切换状态", &format!("ID:{}", id), "").await;
            ok("操作成功", Value::Null)
        }
        Err(_) => err(500, "操作失败"),
    }
}

/// 删除邮箱测试用户（级联清理）
pub async fn email_users_delete(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let u = sqlx::query("SELECT email FROM email_test_users WHERE id = ?").bind(id)
        .fetch_optional(pool).await.ok().flatten();
    let Some(u) = u else {
        return err(404, "用户不存在");
    };
    let email: String = u.get("email");
    let _ = sqlx::query("DELETE FROM email_test_logs WHERE user_id = ?").bind(id).execute(pool).await;
    let _ = sqlx::query("DELETE FROM email_test_codes WHERE email = ?").bind(&email).execute(pool).await;
    let _ = sqlx::query("DELETE FROM email_test_users WHERE id = ?").bind(id).execute(pool).await;

    let app_user = sqlx::query("SELECT id, ciyuanxi_id FROM app_users WHERE email = ?").bind(&email)
        .fetch_optional(pool).await.ok().flatten();
    let mut app_user_deleted = false;
    if let Some(au) = app_user {
        let app_user_id: i64 = au.get("id");
        let ciyuanxi_id: String = au.try_get("ciyuanxi_id").unwrap_or_default();
        if !ciyuanxi_id.is_empty() {
            let playlists = sqlx::query("SELECT id, cover_path FROM user_playlists WHERE user_id = ?").bind(&ciyuanxi_id).fetch_all(pool).await.unwrap_or_default();
            for pl in &playlists {
                let cover: String = pl.try_get("cover_path").unwrap_or_default();
                if !cover.is_empty() {
                    let abs = std::path::Path::new("uploads").join("playlists").join(cover);
                    if abs.is_file() {
                        let _ = std::fs::remove_file(&abs);
                    }
                }
                let pid: i64 = pl.get("id");
                let _ = sqlx::query("DELETE FROM user_playlist_songs WHERE playlist_id = ?").bind(pid).execute(pool).await;
            }
            let _ = sqlx::query("DELETE FROM user_playlists WHERE user_id = ?").bind(&ciyuanxi_id).execute(pool).await;
        }
        let _ = sqlx::query("DELETE FROM play_history WHERE user_id = ?").bind(app_user_id).execute(pool).await;
        let _ = sqlx::query("DELETE FROM app_users WHERE id = ?").bind(app_user_id).execute(pool).await;
        app_user_deleted = true;
    }
    let detail = format!("邮箱:{} {}", email, if app_user_deleted { "(同步删除app_users记录)" } else { "" });
    log_operation(pool, ctx, "邮箱注册用户-删除", &format!("ID:{}", id), &detail).await;
    ok(&format!("删除成功{}", if app_user_deleted { "（已同步删除主用户及歌单数据）" } else { "" }), Value::Null)
}

/// 邮箱测试用户日志
pub async fn email_users_logs(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let q = if id > 0 {
        format!("SELECT * FROM email_test_logs WHERE user_id = {} ORDER BY id DESC LIMIT 50", id)
    } else {
        "SELECT * FROM email_test_logs ORDER BY id DESC LIMIT 50".to_string()
    };
    let rows = sqlx::query(&q).fetch_all(pool).await;
    match rows {
        Ok(rows) => {
            let arr: Vec<Value> = rows.iter().map(|r| crate::admin::row_to_value(r)).collect();
            ok("", json!(arr))
        }
        Err(_) => err(500, "查询失败"),
    }
}

/// 邮箱测试用户统计
pub async fn email_users_stats(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_test_users").fetch_one(pool).await.unwrap_or(0);
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_test_users WHERE status = 1").fetch_one(pool).await.unwrap_or(0);
    let disabled: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_test_users WHERE status = 0").fetch_one(pool).await.unwrap_or(0);
    let today: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_test_users WHERE DATE(created_at) = CURDATE()").fetch_one(pool).await.unwrap_or(0);
    ok("", json!({
        "total_users": total, "active_users": active, "disabled_users": disabled, "today_users": today
    }))
}

// ============================================================
//  邮箱 API 配置管理
// ============================================================

/// 写入或更新 server_settings 值（不存在则 INSERT）
async fn upsert_setting(pool: &MySqlPool, key: &str, value: &str) {
    let _ = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description) VALUES (?, ?, '') ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value)",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await;
}

fn smtp_account_key(account: &crate::handlers::email_auth::SmtpAccount) -> String {
    format!(
        "{}|{}|{}",
        account.sender.trim().to_lowercase(),
        account.host.trim().to_lowercase(),
        account.username.trim().to_lowercase()
    )
}

fn mask_smtp_accounts(accounts: &[crate::handlers::email_auth::SmtpAccount]) -> Vec<Value> {
    accounts
        .iter()
        .map(|a| {
            json!({
                "sender": a.sender.clone(),
                "host": a.host.clone(),
                "port": a.port,
                "username": a.username.clone(),
                "password": if a.password.is_empty() { "" } else { "********" },
                "has_password": !a.password.is_empty(),
                "enabled": a.enabled,
                "remark": a.remark.clone(),
            })
        })
        .collect()
}

fn parse_smtp_accounts_from_body(
    data: &Value,
    old_accounts: &[crate::handlers::email_auth::SmtpAccount],
) -> Vec<crate::handlers::email_auth::SmtpAccount> {
    let Some(items) = data.get("smtp_accounts").and_then(|v| v.as_array()) else {
        return old_accounts.to_vec();
    };

    let mut accounts = Vec::new();
    for item in items {
        let mut account = crate::handlers::email_auth::SmtpAccount {
            sender: item.get("sender").and_then(|v| v.as_str()).unwrap_or("").trim().to_string(),
            host: item.get("host").and_then(|v| v.as_str()).unwrap_or("").trim().to_string(),
            port: item
                .get("port")
                .and_then(|v| {
                    if let Some(n) = v.as_u64() {
                        Some(n as u16)
                    } else {
                        v.as_str().and_then(|s| s.parse::<u16>().ok())
                    }
                })
                .unwrap_or(465),
            username: item.get("username").and_then(|v| v.as_str()).unwrap_or("").trim().to_string(),
            password: item.get("password").and_then(|v| v.as_str()).unwrap_or("").trim().to_string(),
            enabled: item.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
            remark: item.get("remark").and_then(|v| v.as_str()).unwrap_or("").trim().to_string(),
        };

        if account.sender.is_empty() && account.username.contains('@') {
            account.sender = account.username.clone();
        }
        if account.username.is_empty() {
            account.username = account.sender.clone();
        }

        if account.password.is_empty() || account.password == "********" {
            let key = smtp_account_key(&account);
            if let Some(old) = old_accounts.iter().find(|old| smtp_account_key(old) == key) {
                account.password = old.password.clone();
            }
        }

        if !account.sender.is_empty() || !account.host.is_empty() || !account.username.is_empty() {
            accounts.push(account);
        }
    }
    accounts
}

/// 获取邮箱机配置（密码字段脱敏）
pub async fn get_email_config(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let cfg = crate::handlers::email_auth::load_email_config(pool, &ctx.config).await;
    let has_password = !cfg.password.is_empty();
    let has_smtp_password = !cfg.smtp_password.is_empty();
    let smtp_accounts = mask_smtp_accounts(&cfg.smtp_accounts);

    ok("ok", json!({
        "email_provider": cfg.provider,
        "email_channel_general": cfg.channel_general,
        "email_channel_api": cfg.channel_api,
        "email_channel_pool": cfg.channel_pool,
        "email_api_primary": cfg.api_primary,
        "email_api_backup": cfg.api_backup,
        "email_sender": cfg.sender,
        "email_password": if has_password { "********".to_string() } else { String::new() },
        "has_password": has_password,
        "smtp_host": cfg.smtp_host,
        "smtp_port": cfg.smtp_port,
        "smtp_username": cfg.smtp_username,
        "smtp_password": if has_smtp_password { "********".to_string() } else { String::new() },
        "has_smtp_password": has_smtp_password,
        "smtp_accounts": smtp_accounts,
    }))
}

/// 从请求体解析布尔开关（支持 true/false/1/0）
fn parse_bool_setting(data: &serde_json::Value, key: &str) -> Option<bool> {
    match data.get(key) {
        Some(serde_json::Value::Bool(b)) => Some(*b),
        Some(serde_json::Value::String(s)) => Some(s == "1" || s.eq_ignore_ascii_case("true")),
        Some(serde_json::Value::Number(n)) => Some(n.as_i64().unwrap_or(0) == 1),
        _ => None,
    }
}

/// 保存邮箱机配置（密码为空时保留原值）
pub async fn update_email_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);

    let provider = str_of(&data, "email_provider").trim().to_string();
    let provider = if provider.is_empty() { "builtin".to_string() } else { provider };
    if !matches!(provider.as_str(), "builtin" | "http_api" | "smtp") {
        return err(400, "邮箱机发送方式不正确");
    }

    // 通道开关
    let channel_general = parse_bool_setting(&data, "email_channel_general").unwrap_or(true);
    let channel_api = parse_bool_setting(&data, "email_channel_api").unwrap_or(false);
    let channel_pool = parse_bool_setting(&data, "email_channel_pool").unwrap_or(true);

    let api_primary = str_of(&data, "email_api_primary").trim().to_string();
    let api_backup = str_of(&data, "email_api_backup").trim().to_string();
    let sender = str_of(&data, "email_sender").trim().to_string();
    let password = str_of(&data, "email_password").trim().to_string();

    let smtp_host = str_of(&data, "smtp_host").trim().to_string();
    let smtp_port = str_of(&data, "smtp_port").trim().to_string();
    let smtp_username = str_of(&data, "smtp_username").trim().to_string();
    let smtp_password = str_of(&data, "smtp_password").trim().to_string();
    let current_cfg = crate::handlers::email_auth::load_email_config(pool, &ctx.config).await;
    let smtp_accounts = parse_smtp_accounts_from_body(&data, &current_cfg.smtp_accounts);
    let smtp_accounts_json = serde_json::to_string(&smtp_accounts).unwrap_or_else(|_| "[]".to_string());

    upsert_setting(pool, "email_provider", &provider).await;
    upsert_setting(pool, "email_channel_general", if channel_general { "1" } else { "0" }).await;
    upsert_setting(pool, "email_channel_api", if channel_api { "1" } else { "0" }).await;
    upsert_setting(pool, "email_channel_pool", if channel_pool { "1" } else { "0" }).await;
    upsert_setting(pool, "email_api_primary", &api_primary).await;
    upsert_setting(pool, "email_api_backup", &api_backup).await;
    upsert_setting(pool, "email_sender", &sender).await;
    upsert_setting(pool, "smtp_host", &smtp_host).await;
    upsert_setting(pool, "smtp_port", &smtp_port).await;
    upsert_setting(pool, "smtp_username", &smtp_username).await;
    upsert_setting(pool, "smtp_accounts", &smtp_accounts_json).await;

    // 通用密码为空时保留原值
    if !password.is_empty() && password != "********" {
        upsert_setting(pool, "email_password", &password).await;
    }
    // SMTP 密码为空时保留原值
    if !smtp_password.is_empty() && smtp_password != "********" {
        upsert_setting(pool, "smtp_password", &smtp_password).await;
    }

    log_operation(
        pool, ctx, "保存邮箱配置",
        &format!("通用:{} 外部API:{} 账号池:{} 账号池账号:{}个", channel_general, channel_api, channel_pool, smtp_accounts.len()),
        "",
    ).await;

    ok("邮箱机配置已保存", Value::Null)
}

/// 发送测试邮件（使用当前配置）
pub async fn test_email_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let recipient = str_of(&data, "email").trim().to_string();

    if recipient.is_empty() || !is_valid_email(&recipient) {
        return err(400, "请输入合法的测试收件邮箱");
    }

    let title = "【弦予音乐】测试邮件";
    let context = "这是一封来自弦予音乐后台管理系统的测试邮件。\n\n如果您收到此邮件，说明当前邮箱机配置正常工作。\n\n—— 弦予音乐";

    match crate::handlers::email_auth::call_email_api(
        &ctx.config,
        pool,
        title,
        context,
        &recipient,
    )
    .await
    {
        Ok(()) => {
            log_operation(pool, ctx, "发送测试邮件", &recipient, "").await;
            ok("测试邮件已发送，请查收", Value::Null)
        }
        Err(e) => {
            eprintln!("[admin] test_email_config 发送失败: {}", e);
            err(500, &format!("邮件发送失败: {}", e))
        }
    }
}
