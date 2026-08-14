use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use crate::audit_policy::{self, AuditDecision};
use crate::handlers::helpers::{parse_body, str_of, validate_ciyuanxi_id, validate_nickname};
use crate::response::ReqCtx;

const SETTINGS_FIELDS: [&str; 8] = [
    "stream_cache_enabled",
    "startup_play_enabled",
    "bluetooth_lyric_enabled",
    "download_lyric_enabled",
    "download_cover_enabled",
    "download_artist_enabled",
    "search_board_enabled",
    "page_animation_enabled",
];

fn extract_id(data: &Value) -> String {
    let ciyuanxi_id = str_of(data, "ciyuanxi_id");
    if !ciyuanxi_id.is_empty() {
        return ciyuanxi_id;
    }
    str_of(data, "user_id")
}

pub async fn get_user_info(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let user = sqlx::query("SELECT * FROM app_users WHERE ciyuanxi_id = ? OR id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let email: String = user.get("email");
    let role = crate::handlers::helpers::resolve_role_by_email(pool, &email).await;
    let payload = json!({
        "user_id": user.get::<i64,_>("id"),
        "nickname": user.get::<String,_>("nickname"),
        "username": user.get::<String,_>("nickname"),
        "email": email,
        "role": role,
        "avatar_url": user.try_get::<Option<String>, _>("avatar_url").unwrap_or(None).unwrap_or_default(),
        "ciyuanxi_id": user.get::<String,_>("ciyuanxi_id"),
    });
    ctx.json(200, "ok", Some(payload))
}

pub async fn get_user_settings(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let _ = sqlx::query("INSERT INTO user_settings (ciyuanxi_id) VALUES (?) ON DUPLICATE KEY UPDATE ciyuanxi_id = ciyuanxi_id")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let row = sqlx::query("SELECT * FROM user_settings WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(row) = row else {
        return ctx.json(200, "ok", Some(default_settings()));
    };
    let payload = json!({
        "stream_cache_enabled": row.get::<i64,_>("stream_cache_enabled"),
        "startup_play_enabled": row.get::<i64,_>("startup_play_enabled"),
        "bluetooth_lyric_enabled": row.get::<i64,_>("bluetooth_lyric_enabled"),
        "download_lyric_enabled": row.get::<i64,_>("download_lyric_enabled"),
        "download_cover_enabled": row.get::<i64,_>("download_cover_enabled"),
        "download_artist_enabled": row.get::<i64,_>("download_artist_enabled"),
        "search_board_enabled": row.get::<i64,_>("search_board_enabled"),
        "page_animation_enabled": row.get::<i64,_>("page_animation_enabled"),
        "default_quality": row.try_get::<Option<String>, _>("default_quality").unwrap_or(None).unwrap_or_else(|| "standard".to_string()),
    });
    ctx.json(200, "ok", Some(payload))
}

fn default_settings() -> Value {
    json!({
        "stream_cache_enabled": 1,
        "startup_play_enabled": 0,
        "bluetooth_lyric_enabled": 0,
        "download_lyric_enabled": 1,
        "download_cover_enabled": 1,
        "download_artist_enabled": 0,
        "search_board_enabled": 1,
        "page_animation_enabled": 1,
        "default_quality": "standard"
    })
}

pub async fn update_user_settings(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let _ = sqlx::query("INSERT INTO user_settings (ciyuanxi_id) VALUES (?) ON DUPLICATE KEY UPDATE ciyuanxi_id = ciyuanxi_id")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;

    let mut sets = vec![];
    for field in SETTINGS_FIELDS.iter() {
        if let Some(v) = data.get(*field) {
            let val = async_of_i64(v);
            sets.push(format!("`{}` = {}", field, val));
        }
    }
    if let Some(v) = data.get("default_quality") {
        if let Some(q) = v.as_str() {
            let valid = [
                "low", "standard", "high", "super", "ciyuanxi_master", "zhen_master",
                "panorama_master", "ai_master", "exclusive_ai_master",
            ];
            if valid.contains(&q) {
                sets.push(format!("`default_quality` = '{}'", q));
            }
        }
    }
    if sets.is_empty() {
        return ctx.err(400, "没有需要更新的字段");
    }
    let sql = format!("UPDATE user_settings SET {} WHERE ciyuanxi_id = ?", sets.join(", "));
    let result = sqlx::query(&sql).bind(&ciyuanxi_id).execute(pool).await;
    match result {
        Ok(_) => ctx.ok_empty("ok"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

fn async_of_i64(v: &Value) -> i64 {
    match v {
        Value::Number(n) => {
            if n.as_i64().unwrap_or(0) != 0 {
                1
            } else {
                0
            }
        }
        Value::String(s) => {
            if s == "1" || s.eq_ignore_ascii_case("true") {
                1
            } else {
                0
            }
        }
        Value::Bool(b) => {
            if *b {
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

async fn nickname_submit_block_message(pool: &MySqlPool, ciyuanxi_id: &str) -> Option<&'static str> {
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM user_nickname_pending WHERE ciyuanxi_id = ? AND created_at >= CURDATE() AND created_at < CURDATE() + INTERVAL 1 DAY ORDER BY id DESC LIMIT 1",
    )
    .bind(ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match status.as_deref() {
        Some("pending") => Some("昵称正在审核中哦"),
        Some(_) => Some("今日已修改过啦"),
        None => None,
    }
}

async fn avatar_submit_block_message(pool: &MySqlPool, ciyuanxi_id: &str) -> Option<&'static str> {
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM user_avatar_pending WHERE ciyuanxi_id = ? AND created_at >= CURDATE() AND created_at < CURDATE() + INTERVAL 1 DAY ORDER BY id DESC LIMIT 1",
    )
    .bind(ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match status.as_deref() {
        Some("pending") => Some("头像正在审核中哦"),
        Some(_) => Some("今日已修改过啦"),
        None => None,
    }
}

pub async fn update_profile(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let user = sqlx::query("SELECT nickname FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let current_nickname: String = user.get("nickname");

    let mut nickname_submitted = false;
    let mut avatar_submitted = false;
    let mut nickname_auto_approved = false;
    let mut avatar_auto_approved = false;
    if data.get("nickname").and_then(|v| v.as_str()).is_some() {
        let nickname = str_of(&data, "nickname").trim().to_string();
        let len = nickname.chars().count();
        if nickname.is_empty() {
            return ctx.err(400, "昵称不能为空");
        }
        if len < 2 || len > 20 {
            return ctx.err(400, "昵称长度需为 2 到 20 个字符");
        }
        if let Err(msg) = validate_nickname(&nickname, 2, 20) {
            return ctx.err(400, msg);
        }
        if nickname == current_nickname {
            return ctx.err(400, "新昵称不能与当前昵称相同");
        }
        let exists = sqlx::query("SELECT id FROM app_users WHERE nickname = ? AND ciyuanxi_id != ? LIMIT 1")
            .bind(&nickname)
            .bind(&ciyuanxi_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .is_some();
        if exists {
            return ctx.err(400, "昵称已被使用");
        }
        // 检查昵称是否与管理员用户名冲突（大小写不敏感）
        {
            let admin_conflict = sqlx::query("SELECT id FROM admin_users WHERE LOWER(username) = LOWER(?) LIMIT 1")
                .bind(&nickname)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten()
                .is_some();
            if admin_conflict {
                return ctx.err(400, "该昵称不可使用");
            }
        }
        if let Some(msg) = nickname_submit_block_message(pool, &ciyuanxi_id).await {
            return ctx.err(429, msg);
        }
        if data.get("avatar_url").and_then(|v| v.as_str()).is_some() {
            let avatar_url = str_of(&data, "avatar_url").trim().to_string();
            if avatar_url.is_empty() {
                return ctx.err(400, "头像不能为空");
            }
            if let Some(msg) = avatar_submit_block_message(pool, &ciyuanxi_id).await {
                return ctx.err(429, msg);
            }
        }
        let audit = audit_policy::audit_text(
            pool,
            "nickname",
            &nickname,
            json!({ "ciyuanxi_id": ciyuanxi_id, "old_name": current_nickname }),
        )
        .await;
        if audit.decision == AuditDecision::Pass {
            let _ = sqlx::query("UPDATE app_users SET nickname = ? WHERE ciyuanxi_id = ?")
                .bind(&nickname)
                .bind(&ciyuanxi_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("INSERT INTO user_nickname_pending (ciyuanxi_id, nickname, status, reviewed_at, reviewed_by) VALUES (?, ?, 'approved', NOW(), ?)")
                .bind(&ciyuanxi_id)
                .bind(&nickname)
                .bind(format!("external:{}", audit.provider))
                .execute(pool)
                .await;
            nickname_auto_approved = true;
        } else if audit.decision == AuditDecision::Reject {
            let _ = sqlx::query("DELETE FROM user_nickname_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
                .bind(&ciyuanxi_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("INSERT INTO user_nickname_pending (ciyuanxi_id, nickname, status, reviewed_at, reviewed_by) VALUES (?, ?, 'rejected', NOW(), ?)")
                .bind(&ciyuanxi_id)
                .bind(&nickname)
                .bind(format!("external:{}", audit.provider))
                .execute(pool)
                .await;
            return ctx.err(400, if audit.reason.is_empty() { "改名未通过机审" } else { &audit.reason });
        } else {
            let _ = sqlx::query("DELETE FROM user_nickname_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
                .bind(&ciyuanxi_id)
                .execute(pool)
                .await;
            let result = sqlx::query("INSERT INTO user_nickname_pending (ciyuanxi_id, nickname, status) VALUES (?, ?, 'pending')")
                .bind(&ciyuanxi_id)
                .bind(&nickname)
                .execute(pool)
                .await;
            if let Err(e) = result {
                return ctx.err(500, &format!("服务器错误: {}", e));
            }
            crate::admin::email::notify_external_emails_for_module(
                pool,
                &ctx.config,
                &ctx.client_ip,
                "nickname",
                "【弦予后台】新昵称待审核",
                &format!("用户 {} 申请改名为「{}」，请及时审核。", ciyuanxi_id, nickname),
                "",
                &ctx.base_url,
            ).await;
            nickname_submitted = true;
        }
    }
    if data.get("avatar_url").and_then(|v| v.as_str()).is_some() {
        let avatar_url = str_of(&data, "avatar_url").trim().to_string();
        if avatar_url.is_empty() {
            return ctx.err(400, "头像不能为空");
        }
        if let Some(msg) = avatar_submit_block_message(pool, &ciyuanxi_id).await {
            return ctx.err(429, msg);
        }
        let audit = audit_policy::audit_image(
            pool,
            "avatar",
            &avatar_url,
            json!({ "ciyuanxi_id": ciyuanxi_id }),
        )
        .await;
        if audit.decision == AuditDecision::Pass {
            let _ = sqlx::query("UPDATE app_users SET avatar_url = ? WHERE ciyuanxi_id = ?")
                .bind(&avatar_url)
                .bind(&ciyuanxi_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("UPDATE user_feedback SET nickname = (SELECT nickname FROM app_users WHERE ciyuanxi_id = ?) WHERE ciyuanxi_id = ?")
                .bind(&ciyuanxi_id)
                .bind(&ciyuanxi_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, status, reviewed_at, reviewed_by) VALUES (?, ?, 'approved', NOW(), ?)")
                .bind(&ciyuanxi_id)
                .bind(&avatar_url)
                .bind(format!("external:{}", audit.provider))
                .execute(pool)
                .await;
            avatar_auto_approved = true;
        } else if audit.decision == AuditDecision::Reject {
            let _ = sqlx::query("DELETE FROM user_avatar_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
                .bind(&ciyuanxi_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, status, reviewed_at, reviewed_by) VALUES (?, ?, 'rejected', NOW(), ?)")
                .bind(&ciyuanxi_id)
                .bind(&avatar_url)
                .bind(format!("external:{}", audit.provider))
                .execute(pool)
                .await;
            return ctx.err(400, if audit.reason.is_empty() { "头像未通过机审" } else { &audit.reason });
        } else {
            let _ = sqlx::query("DELETE FROM user_avatar_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
                .bind(&ciyuanxi_id)
                .execute(pool)
                .await;
            let result = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, status) VALUES (?, ?, 'pending')")
                .bind(&ciyuanxi_id)
                .bind(&avatar_url)
                .execute(pool)
                .await;
            if let Err(e) = result {
                return ctx.err(500, &format!("服务器错误: {}", e));
            }
            crate::admin::email::notify_external_emails_for_module(
                pool,
                &ctx.config,
                &ctx.client_ip,
                "avatar",
                "【弦予后台】新头像待审核",
                &format!("用户 {} 提交了新头像，请及时审核。", ciyuanxi_id),
                &avatar_url,
                &ctx.base_url,
            ).await;
            avatar_submitted = true;
        }
    }
    if nickname_auto_approved || avatar_auto_approved {
        return match (nickname_auto_approved, avatar_auto_approved, nickname_submitted, avatar_submitted) {
            (true, true, _, _) => ctx.ok("头像和改名已通过机审并立即生效", json!({ "status": "approved" })),
            (true, false, false, false) => ctx.ok("改名已通过机审并立即生效", json!({ "status": "approved" })),
            (false, true, false, false) => ctx.ok("头像已通过机审并立即生效", json!({ "status": "approved" })),
            (true, false, _, true) => ctx.ok("改名已通过机审，头像已提交人工审核", json!({ "status": "partial" })),
            (false, true, true, _) => ctx.ok("头像已通过机审，改名已提交人工审核", json!({ "status": "partial" })),
            _ => ctx.ok("资料已处理", json!({ "status": "partial" })),
        };
    }
    match (nickname_submitted, avatar_submitted) {
        (true, true) => ctx.ok("头像和改名申请已提交，等待管理员审核", json!({ "status": "pending" })),
        (true, false) => ctx.ok("改名申请已提交，等待管理员审核", json!({ "status": "pending" })),
        (false, true) => ctx.ok("头像已上传，等待管理员审核", json!({ "status": "pending" })),
        (false, false) => ctx.err(400, "没有需要更新的字段"),
    }
}

#[allow(dead_code)]
fn sql_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

pub async fn check_username(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let _nickname = str_of(&data, "nickname").trim().to_string();
    let username = if _nickname.is_empty() {
        str_of(&data, "username").trim().to_string()
    } else {
        _nickname
    };
    let exists = if username.is_empty() {
        false
    } else {
        sqlx::query("SELECT id FROM app_users WHERE nickname = ? LIMIT 1")
            .bind(&username)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .is_some()
    };
    ctx.json(200, "ok", Some(json!({ "available": !exists })))
}

pub async fn change_password(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let old_password = str_of(&data, "old_password");
    let new_password = str_of(&data, "new_password");
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    if new_password.len() < 6 {
        return ctx.err(400, "新密码长度不能少于6位");
    }
    let user = sqlx::query("SELECT * FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let stored: String = user.get("password");
    if !bcrypt::verify(&old_password, &stored).unwrap_or(false) {
        return ctx.err(400, "原密码错误");
    }
    let hashed = match bcrypt::hash(&new_password, 10) {
        Ok(h) => h,
        Err(_) => return ctx.err(500, "密码加密失败"),
    };
    let uid: i64 = user.get("id");
    let _ = sqlx::query("UPDATE app_users SET password = ? WHERE id = ?")
        .bind(&hashed)
        .bind(uid)
        .execute(pool)
        .await;
    ctx.ok_empty("密码修改成功")
}

/// 修改弦予号（每月限一次 + 唯一性校验）
/// 参考微信号设计：弦予号是用户唯一登录标识，可修改但每月仅限一次。
pub async fn update_ciyuanxi_id(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let old_ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let new_ciyuanxi_id = str_of(&data, "new_ciyuanxi_id").trim().to_string();
    let password = str_of(&data, "password");
    if old_ciyuanxi_id.is_empty() {
        return ctx.err(400, "当前弦予号不能为空");
    }
    if new_ciyuanxi_id.is_empty() {
        return ctx.err(400, "请输入新弦予号");
    }
    if let Err(msg) = validate_ciyuanxi_id(&new_ciyuanxi_id) {
        return ctx.err(400, msg);
    }
    if new_ciyuanxi_id == old_ciyuanxi_id {
        return ctx.err(400, "新弦予号不能与当前弦予号相同");
    }

    let user = sqlx::query("SELECT * FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&old_ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };

    // 校验密码
    let stored: String = user.get("password");
    if !bcrypt::verify(&password, &stored).unwrap_or(false) {
        return ctx.err(400, "密码错误");
    }

    // 每月限一次
    if let Ok(Some(last)) = user.try_get::<Option<chrono::NaiveDateTime>, _>("ciyuanxi_id_updated_at") {
        let now = chrono::Utc::now().naive_utc();
        let diff_days = (now - last).num_days();
        if diff_days < 30 {
            let remain = 30 - diff_days;
            return ctx.err(429, &format!("弦予号每月只能修改一次，请{}天后再试", remain));
        }
    }

    // 唯一性校验（app_users + pretty_ids）
    let dup_user = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? AND ciyuanxi_id != ? LIMIT 1")
        .bind(&new_ciyuanxi_id)
        .bind(&old_ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    let dup_pretty = sqlx::query("SELECT id FROM ciyuanxi_pretty_ids WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&new_ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if dup_user || dup_pretty {
        return ctx.err(400, "该弦予号已被占用");
    }

    let uid: i64 = user.get("id");
    let _ = sqlx::query("UPDATE app_users SET ciyuanxi_id = ?, ciyuanxi_id_updated_at = NOW() WHERE id = ?")
        .bind(&new_ciyuanxi_id)
        .bind(uid)
        .execute(pool)
        .await;

    ctx.ok("弦予号修改成功", json!({ "ciyuanxi_id": new_ciyuanxi_id }))
}

pub async fn get_avatar_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let user_exists = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if user_exists.is_none() {
        return ctx.err(404, "用户不存在");
    }
    let today_status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM user_avatar_pending WHERE ciyuanxi_id = ? AND created_at >= CURDATE() AND created_at < CURDATE() + INTERVAL 1 DAY ORDER BY id DESC LIMIT 1",
    )
    .bind(&ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let today_block_message = match today_status.as_deref() {
        Some("pending") => "头像正在审核中哦",
        Some(_) => "今日已修改过啦",
        None => "",
    };
    let row = sqlx::query(
        "SELECT status, created_at FROM user_avatar_pending WHERE ciyuanxi_id = ? AND status IN ('pending', 'rejected') ORDER BY id DESC LIMIT 1",
    )
    .bind(&ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match row {
        Some(r) => ctx.json(200, "ok", Some(json!({
            "status": r.get::<String, _>("status"),
            "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
            "today_blocked": today_status.is_some(),
            "block_message": today_block_message,
        }))),
        None => ctx.json(200, "ok", Some(json!({
            "status": "none",
            "today_blocked": today_status.is_some(),
            "block_message": today_block_message,
        }))),
    }
}

pub async fn get_nickname_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let user = sqlx::query("SELECT nickname FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let today_status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM user_nickname_pending WHERE ciyuanxi_id = ? AND created_at >= CURDATE() AND created_at < CURDATE() + INTERVAL 1 DAY ORDER BY id DESC LIMIT 1",
    )
    .bind(&ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let today_block_message = match today_status.as_deref() {
        Some("pending") => "昵称正在审核中哦",
        Some(_) => "今日已修改过啦",
        None => "",
    };
    let row = sqlx::query(
        "SELECT status, nickname, created_at FROM user_nickname_pending WHERE ciyuanxi_id = ? AND status IN ('pending', 'rejected') ORDER BY id DESC LIMIT 1",
    )
    .bind(&ciyuanxi_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match row {
        Some(r) => ctx.json(200, "ok", Some(json!({
            "status": r.get::<String, _>("status"),
            "nickname": r.get::<String, _>("nickname"),
            "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
            "today_blocked": today_status.is_some(),
            "block_message": today_block_message,
        }))),
        None => ctx.json(200, "ok", Some(json!({
            "status": "none",
            "nickname": user.get::<String, _>("nickname"),
            "today_blocked": today_status.is_some(),
            "block_message": today_block_message,
        }))),
    }
}

pub async fn report_listen_stats(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    // 兼容 duration / listen_duration 字段，同时支持数字和字符串
    let seconds = data
        .get("duration")
        .or_else(|| data.get("listen_duration"))
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok())))
        .unwrap_or(0.0)
        .max(0.0) as i64;
    // 当日听歌时长（秒），用于日榜/周榜
    let daily_seconds = data
        .get("daily_duration")
        .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok())))
        .unwrap_or(0.0)
        .max(0.0) as i64;
    let unique_songs_count = data
        .get("unique_songs_count")
        .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0)
        .max(0);

    // 检查是否存在待处理的听歌统计重置信号
    let row = sqlx::query(
        "SELECT listen_stats_reset_at, listen_duration_offset, unique_songs_offset FROM app_users WHERE ciyuanxi_id = ?",
    )
    .bind(&ciyuanxi_id)
    .fetch_optional(pool)
    .await;

    let (reset_at, dur_offset, songs_offset): (Option<String>, i64, i64) = match row {
        Ok(Some(r)) => {
            use sqlx::Row;
            let reset: Option<String> = r.try_get("listen_stats_reset_at").unwrap_or(None);
            let dur_off: i64 = r.try_get("listen_duration_offset").unwrap_or(0);
            let songs_off: i64 = r.try_get("unique_songs_offset").unwrap_or(0);
            (reset, dur_off, songs_off)
        }
        _ => return ctx.err(404, "用户不存在"),
    };

    if let Some(ts) = reset_at {
        // 重置信号存在：将客户端当前上报的累计值作为基准偏移量存储
        // 后续上报需减去偏移量才是重置后的真实增量
        let _ = sqlx::query(
            "UPDATE app_users SET listen_stats_reset_at = NULL, listen_duration = 0, unique_songs_count = 0, listen_duration_offset = ?, unique_songs_offset = ? WHERE ciyuanxi_id = ?",
        )
        .bind(seconds)
        .bind(unique_songs_count)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;

        // 同时清理每日统计，确保日榜/周榜从零开始
        let _ = sqlx::query("DELETE FROM listen_daily_stats WHERE ciyuanxi_id = ?")
            .bind(&ciyuanxi_id)
            .execute(pool)
            .await;

        return ctx.ok("ok", Some(json!({ "reset_at": ts })));
    }

    if seconds <= 0 && unique_songs_count <= 0 {
        return ctx.ok_empty("ok");
    }

    // 正常流程：减去重置偏移量得到真实增量，再与当前值取 GREATEST
    let actual_seconds = (seconds - dur_offset).max(0);
    let actual_songs = (unique_songs_count - songs_offset).max(0);

    let result = sqlx::query(
        "UPDATE app_users \
         SET listen_duration = GREATEST(listen_duration, ?), \
             unique_songs_count = GREATEST(unique_songs_count, ?) \
         WHERE ciyuanxi_id = ?",
    )
    .bind(actual_seconds)
    .bind(actual_songs)
    .bind(&ciyuanxi_id)
    .execute(pool)
    .await;

    // 同步写入每日统计（用于日榜/周榜），使用客户端上报的当日时长而非累计总时长
    let _ = sqlx::query(
        "INSERT INTO listen_daily_stats (ciyuanxi_id, stat_date, listen_duration, unique_songs_count) \
         VALUES (?, CURDATE(), ?, ?) \
         ON DUPLICATE KEY UPDATE \
             listen_duration = GREATEST(listen_duration, VALUES(listen_duration)), \
             unique_songs_count = GREATEST(unique_songs_count, VALUES(unique_songs_count))",
    )
    .bind(&ciyuanxi_id)
    .bind(daily_seconds)
    .bind(actual_songs)
    .execute(pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => ctx.ok_empty("ok"),
        Ok(_) => ctx.err(404, "用户不存在"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn deduct_master_quota(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let amount = data.get("amount").and_then(|v| v.as_i64()).unwrap_or(1);
    let res = sqlx::query("UPDATE app_users SET master_quota = GREATEST(master_quota - ?, 0) WHERE ciyuanxi_id = ?")
        .bind(amount)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    match res {
        Ok(_) => ctx.ok_empty("ok"),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn get_master_quota_usage(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let row = sqlx::query("SELECT master_quota FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let quota: i64 = r.get("master_quota");
            ctx.json(200, "ok", Some(json!({ "master_quota": quota })))
        }
        None => ctx.err(404, "用户不存在"),
    }
}
