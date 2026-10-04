use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use crate::audit_policy::{self, AuditDecision};
use crate::handlers::helpers::{extract_id, parse_body, str_of, validate_ciyuanxi_id, validate_nickname};
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
    // email/role 仅在请求 token 能核验为本人时返回，防止遍历弦予号探测他人邮箱
    let token = str_of(&data, "token");
    let owner = crate::handlers::token::resolve_owner(pool, &token)
        .await
        .unwrap_or_default();
    let self_ciyuanxi_id: String = user.get("ciyuanxi_id");
    let owned = !owner.is_empty() && (owner == self_ciyuanxi_id || owner == ciyuanxi_id);
    let email: String = if owned {
        user.try_get("email").unwrap_or_default()
    } else {
        String::new()
    };
    let role = if owned {
        crate::handlers::helpers::resolve_role_by_email(pool, &email).await
    } else {
        "member".to_string()
    };
    let payload = json!({
        "user_id": user.get::<i64,_>("id"),
        "nickname": user.get::<String,_>("nickname"),
        "username": user.get::<String,_>("nickname"),
        "email": email,
        "role": role,
        "avatar_url": crate::handlers::upload::absolutize_media_url_with(
            &ctx.base_url,
            &ctx.config.public_base_url,
            &user.try_get::<Option<String>, _>("avatar_url").unwrap_or(None).unwrap_or_default(),
        ),
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
        Err(e) => { tracing::error!("服务器错误: {e}"); ctx.err(500, "服务器错误") },
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
        "SELECT status FROM user_nickname_pending WHERE ciyuanxi_id = ? AND created_at >= DATE(NOW() + INTERVAL 8 HOUR) - INTERVAL 8 HOUR AND created_at < DATE(NOW() + INTERVAL 8 HOUR) + INTERVAL 16 HOUR ORDER BY id DESC LIMIT 1",
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
        "SELECT status FROM user_avatar_pending WHERE ciyuanxi_id = ? AND created_at >= DATE(NOW() + INTERVAL 8 HOUR) - INTERVAL 8 HOUR AND created_at < DATE(NOW() + INTERVAL 8 HOUR) + INTERVAL 16 HOUR ORDER BY id DESC LIMIT 1",
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
            let _ = sqlx::query("INSERT INTO user_nickname_pending (ciyuanxi_id, nickname, old_name, status, reviewed_at, reviewed_by) VALUES (?, ?, ?, 'approved', NOW(), ?)")
                .bind(&ciyuanxi_id)
                .bind(&nickname)
                .bind(&current_nickname)
                .bind(format!("external:{}", audit.provider))
                .execute(pool)
                .await;
            nickname_auto_approved = true;
        } else if audit.decision == AuditDecision::Reject {
            let _ = sqlx::query("DELETE FROM user_nickname_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
                .bind(&ciyuanxi_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("INSERT INTO user_nickname_pending (ciyuanxi_id, nickname, old_name, status, reviewed_at, reviewed_by) VALUES (?, ?, ?, 'rejected', NOW(), ?)")
                .bind(&ciyuanxi_id)
                .bind(&nickname)
                .bind(&current_nickname)
                .bind(format!("external:{}", audit.provider))
                .execute(pool)
                .await;
            return ctx.err(400, if audit.reason.is_empty() { "改名未通过机审" } else { &audit.reason });
        } else {
            let _ = sqlx::query("DELETE FROM user_nickname_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
                .bind(&ciyuanxi_id)
                .execute(pool)
                .await;
            let result = sqlx::query("INSERT INTO user_nickname_pending (ciyuanxi_id, nickname, old_name, status) VALUES (?, ?, ?, 'pending')")
                .bind(&ciyuanxi_id)
                .bind(&nickname)
                .bind(&current_nickname)
                .execute(pool)
                .await;
            if let Err(e) = result {
                { tracing::error!("服务器错误: {e}"); return ctx.err(500, "服务器错误"); }
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
                { tracing::error!("服务器错误: {e}"); return ctx.err(500, "服务器错误"); }
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
    crate::handlers::token::revoke_user(pool, &ciyuanxi_id).await;
    ctx.ok_empty("密码修改成功")
}

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

    let stored: String = user.get("password");
    if !bcrypt::verify(&password, &stored).unwrap_or(false) {
        return ctx.err(400, "密码错误");
    }

    if let Ok(Some(last)) = user.try_get::<Option<chrono::NaiveDateTime>, _>("ciyuanxi_id_updated_at") {
        let now = chrono::Utc::now().naive_utc();
        let diff_days = (now - last).num_days();
        if diff_days < 30 {
            let remain = 30 - diff_days;
            return ctx.err(429, &format!("弦予号每月只能修改一次，请{}天后再试", remain));
        }
    }

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
    crate::handlers::token::revoke_user(pool, &old_ciyuanxi_id).await;

    ctx.ok("弦予号修改成功", json!({ "ciyuanxi_id": new_ciyuanxi_id }))
}

pub async fn bind_email(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let email = str_of(&data, "email").trim().to_string();
    let verify_code = str_of(&data, "verify_code").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "当前账号不能为空");
    }
    if email.is_empty() || !email.contains('@') || email.contains(' ') {
        return ctx.err(400, "邮箱格式不正确");
    }
    if verify_code.is_empty() {
        return ctx.err(400, "请输入邮箱验证码");
    }
    let user = sqlx::query("SELECT id, email FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(404, "用户不存在");
    };
    let current_email: String = user.try_get("email").unwrap_or_default();
    if !current_email.is_empty() {
        return ctx.err(400, "当前账号已绑定邮箱");
    }
    let dup = sqlx::query("SELECT id FROM app_users WHERE email = ? LIMIT 1")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if dup {
        return ctx.err(400, "该邮箱已绑定其他账号");
    }
    let code_row = sqlx::query(
        "SELECT id FROM email_verify_codes WHERE email = ? AND code = ? AND type = 'bind' AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .bind(&verify_code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let Some(code_row) = code_row else {
        return ctx.err(400, "验证码无效或已过期");
    };
    let code_id: i64 = code_row.get("id");
    let uid: i64 = user.get("id");
    let _ = sqlx::query("UPDATE app_users SET email = ?, email_verified = 1 WHERE id = ?")
        .bind(&email)
        .bind(uid)
        .execute(pool)
        .await;
    let _ = sqlx::query("UPDATE email_verify_codes SET used = 1 WHERE id = ?")
        .bind(code_id)
        .execute(pool)
        .await;
    ctx.ok("邮箱绑定成功", json!({ "email": email }))
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
        "SELECT status FROM user_avatar_pending WHERE ciyuanxi_id = ? AND created_at >= DATE(NOW() + INTERVAL 8 HOUR) - INTERVAL 8 HOUR AND created_at < DATE(NOW() + INTERVAL 8 HOUR) + INTERVAL 16 HOUR ORDER BY id DESC LIMIT 1",
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
        "SELECT status FROM user_nickname_pending WHERE ciyuanxi_id = ? AND created_at >= DATE(NOW() + INTERVAL 8 HOUR) - INTERVAL 8 HOUR AND created_at < DATE(NOW() + INTERVAL 8 HOUR) + INTERVAL 16 HOUR ORDER BY id DESC LIMIT 1",
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

    // ===== 增量上报协议（stats_mode = "delta"）=====
    if data.get("stats_mode").and_then(|v| v.as_str()) == Some("delta") {
        return report_listen_stats_delta(&data, &ciyuanxi_id, ctx, pool).await;
    }
    ctx.err(426, "客户端版本过旧，听歌统计上报协议已升级，请更新到最新版本")
}

async fn report_listen_stats_delta(
    data: &serde_json::Value,
    ciyuanxi_id: &str,
    ctx: ReqCtx,
    pool: &MySqlPool,
) -> Response {
    use serde_json::json;

    let parse_sec = |key: &str| -> i64 {
        data.get(key)
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok())))
            .unwrap_or(0.0)
            .max(0.0) as i64
    };
    // 单次上报上限的基准值：正常 30 秒~几分钟上报一次，物理上不可能超 4 小时
    const MAX_SINGLE_DELTA_SECS: i64 = 4 * 3600;
    // 客户端自报的自上次成功上报以来的墙钟秒数（-1 = 未提供），仅用于交叉比对，
    // 服务端以自身 listen_reported_at 独立核算为准
    let client_elapsed_secs = data
        .get("elapsed_secs")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);
    let mut delta_total = parse_sec("delta_duration");
    let mut delta_daily = parse_sec("delta_daily_duration");
    let delta_songs = data
        .get("delta_songs")
        .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0)
        .max(0);

    let reset_row = sqlx::query(
        "SELECT listen_stats_reset_at, listen_stats_reset_reason, listen_reported_at FROM app_users WHERE ciyuanxi_id = ?",
    )
    .bind(ciyuanxi_id)
    .fetch_optional(pool)
    .await;
    let reported_at_sec: i64 = match &reset_row {
        Ok(Some(row)) => {
            use sqlx::Row;
            row.try_get::<i64, _>("listen_reported_at").unwrap_or(0)
        }
        _ => 0,
    };
    let now_sec: i64 = sqlx::query_scalar("SELECT UNIX_TIMESTAMP()")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    if let Ok(Some(r)) = reset_row {
        use sqlx::Row;
        let reset: Option<String> = r.try_get("listen_stats_reset_at").unwrap_or(None);
        if let Some(ts) = reset {
            let reason: String = r.try_get("listen_stats_reset_reason").unwrap_or_default();
            let _ = sqlx::query(
                "UPDATE app_users SET listen_stats_reset_at = NULL, listen_stats_reset_reason = '', listen_duration = 0, unique_songs_count = 0, listen_duration_offset = 0, unique_songs_offset = 0, listen_reported_at = 0 WHERE ciyuanxi_id = ?",
            )
            .bind(ciyuanxi_id)
            .execute(pool)
            .await;
            let _ = sqlx::query("DELETE FROM listen_daily_stats WHERE ciyuanxi_id = ?")
                .bind(ciyuanxi_id)
                .execute(pool)
                .await;
            let _ = sqlx::query("DELETE FROM play_history WHERE ciyuanxi_id = ?")
                .bind(ciyuanxi_id)
                .execute(pool)
                .await;
            return ctx.ok("ok", Some(json!({ "reset_at": ts, "reason": reason })));
        }
    } else {
        return ctx.err(404, "用户不存在");
    }

    if delta_total <= 0 && delta_daily <= 0 && delta_songs <= 0 {
        return read_listen_stats_snapshot(ciyuanxi_id, ctx, pool).await;
    }

    // ===== 双重核验（服务端独立核算）=====
    // 用服务端自己的 listen_reported_at（上次成功上报时刻，数据库时钟）推算
    // 墙钟跨度得出物理上限；客户端 elapsed 与服务端核算一致（±120 秒）才按
    // 两者较严的上限放行，不一致（虚报/时钟漂移/缺字段）一律按服务端上限截断
    // ——两边对上账才入库。
    let server_elapsed = if reported_at_sec > 0 {
        (now_sec - reported_at_sec).clamp(0, 30 * 86400)
    } else {
        0
    };
    let server_max = if reported_at_sec > 0 {
        // 物理硬约束：播放时长增速不可能超过墙钟（+120 秒容差）。
        // 旧的 ×3+600 倍速上限在客户端 baseline 丢失（每次把全量本地累计
        // 当 delta 报）时，会被逐笔按 3 倍墙钟截断入库，数小时虚增出几十小时
        (server_elapsed + 120).min(MAX_SINGLE_DELTA_SECS)
    } else {
        // 首报兜底：客户端 baseline 丢失时会把全量本地累计当 delta 报上来，
        // 7200 的旧兜底曾造成单笔虚增 2 小时，收紧到 600 秒；
        // 正常新设备登录后几分钟内就会首次上报，不会被误伤
        600
    };
    let has_client_elapsed = client_elapsed_secs >= 0;
    let cross_ok =
        has_client_elapsed && (client_elapsed_secs - server_elapsed).abs() <= 120;
    // 客户端报了 elapsed 但与服务端核算差超容差 = 两边账脱钩（baseline 丢失/
    // 时钟异常），此时 delta 不可信：整笔丢弃，宁缺勿滥——虚增不可逆，
    // 丢笔最多丢一两个心跳的真实播放
    let effective_max = if cross_ok {
        // delta 是同窗播放秒数，不可能超过墙钟：超过客户端自报 elapsed 即自相矛盾
        // （baseline 丢失/本地统计器虚涨），整笔丢弃——截断反而给虚报留下
        // 每笔 +容差 的合法入账通道（+120 被 60s 高频心跳放大成 3 倍墙钟速）
        if delta_total > client_elapsed_secs {
            0
        } else {
            delta_total
        }
    } else if has_client_elapsed {
        0
    } else {
        // 客户端未带 elapsed（旧版本），退回服务端墙钟上限
        server_max
    };
    if delta_total > effective_max {
        tracing::info!(
            "listen delta clamped: id={} in_delta={} effective_max={} client_elapsed={} server_elapsed={}",
            ciyuanxi_id, delta_total, effective_max, client_elapsed_secs, server_elapsed
        );
        delta_total = effective_max;
    }
    if delta_daily > delta_total {
        delta_daily = delta_total;
    }

    let result = sqlx::query(
        "UPDATE app_users \
         SET listen_duration = listen_duration + ?, \
             unique_songs_count = unique_songs_count + ? \
         WHERE ciyuanxi_id = ?",
    )
    .bind(delta_total)
    .bind(delta_songs)
    .bind(ciyuanxi_id)
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "UPDATE app_users SET listen_reported_at = UNIX_TIMESTAMP() WHERE ciyuanxi_id = ?",
    )
    .bind(ciyuanxi_id)
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "INSERT INTO listen_daily_stats (ciyuanxi_id, stat_date, listen_duration, unique_songs_count) \
         VALUES (?, DATE(NOW() + INTERVAL 8 HOUR), ?, ?) \
         ON DUPLICATE KEY UPDATE \
             listen_duration = LEAST(listen_duration + VALUES(listen_duration), 86400), \
             unique_songs_count = unique_songs_count + VALUES(unique_songs_count)",
    )
    .bind(ciyuanxi_id)
    .bind(delta_daily)
    .bind(delta_songs)
    .execute(pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            read_listen_stats_snapshot(ciyuanxi_id, ctx, pool).await
        }
        Ok(_) => ctx.err(404, "用户不存在"),
        Err(e) => { tracing::error!("服务器错误: {e}"); ctx.err(500, "服务器错误") },
    }
}

async fn read_listen_stats_snapshot(ciyuanxi_id: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    use serde_json::json;
    let total: i64 = sqlx::query_scalar(
        "SELECT listen_duration FROM app_users WHERE ciyuanxi_id = ?",
    )
    .bind(ciyuanxi_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let daily: i64 = sqlx::query_scalar(
        "SELECT listen_duration FROM listen_daily_stats \
         WHERE ciyuanxi_id = ? AND stat_date = DATE(NOW() + INTERVAL 8 HOUR)",
    )
    .bind(ciyuanxi_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let weekly: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(listen_duration), 0) FROM listen_daily_stats \
         WHERE ciyuanxi_id = ? \
           AND stat_date >= DATE(NOW() + INTERVAL 8 HOUR) - INTERVAL 6 DAY",
    )
    .bind(ciyuanxi_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    ctx.ok(
        "ok",
        Some(json!({
            "server_total_duration": total.max(0),
            "server_daily_duration": daily.max(0),
            "server_weekly_duration": weekly.max(0),
        })),
    )
}

/// 登录/同步后主动拉取云端现算听歌统计（总/今日/近七日），替代已删除的快照下发。
pub async fn get_listen_stats_summary(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    read_listen_stats_snapshot(&ciyuanxi_id, ctx, pool).await
}

pub async fn get_listen_stats(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }

    let row = sqlx::query(
        "SELECT listen_duration, unique_songs_count FROM app_users WHERE ciyuanxi_id = ?",
    )
    .bind(&ciyuanxi_id)
    .fetch_optional(pool)
    .await;

    match row {
        Ok(Some(r)) => {
            use sqlx::Row;
            let duration: i64 = r.try_get("listen_duration").unwrap_or(0);
            let songs: i64 = r.try_get("unique_songs_count").unwrap_or(0);
            ctx.ok(
                "ok",
                Some(json!({ "total_duration": duration.max(0), "unique_songs_count": songs.max(0) })),
            )
        }
        Ok(None) => ctx.err(404, "用户不存在"),
        Err(e) => { tracing::error!("服务器错误: {e}"); ctx.err(500, "服务器错误") },
    }
}

pub async fn deduct_master_quota(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = extract_id(&data);
    let amount = data.get("amount").and_then(|v| v.as_i64()).unwrap_or(1);
    // 负数或 0 会经 GREATEST 反向增加配额，必须拒绝；上限防单次刷穿
    if !(1..=1000).contains(&amount) {
        return ctx.err(400, "无效的扣减数量");
    }
    let res = sqlx::query("UPDATE app_users SET master_quota = GREATEST(master_quota - ?, 0) WHERE ciyuanxi_id = ?")
        .bind(amount)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    match res {
        Ok(_) => ctx.ok_empty("ok"),
        Err(e) => { tracing::error!("服务器错误: {e}"); ctx.err(500, "服务器错误") },
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
