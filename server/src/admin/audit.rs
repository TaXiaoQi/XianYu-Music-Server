use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};
use crate::audit_policy::{self, AuditExternalConfig, BannedWordsConfig};
use crate::handlers::helpers::{bool_of, int_of, parse_body, str_of};

/// 确保头像审核表存在
async fn ensure_avatar_table(pool: &MySqlPool) {
    for stmt in crate::schema::table_statements().iter() {
        if stmt.contains("`user_avatar_pending`") || stmt.contains("`user_nickname_pending`") {
            let _ = sqlx::query(stmt).execute(pool).await;
        }
    }
}

pub async fn get_audit_external_config(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let mut cfg = audit_policy::load_config(pool).await;
    cfg.api_key = String::new();
    ok("ok", json!(cfg))
}

pub async fn save_audit_external_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let current = audit_policy::load_config(pool).await;
    let mut cfg = AuditExternalConfig {
        enabled: bool_of(&data, "enabled"),
        provider: str_of(&data, "provider").trim().to_string(),
        endpoint: str_of(&data, "endpoint").trim().to_string(),
        api_key: str_of(&data, "api_key").trim().to_string(),
        nickname_enabled: bool_of(&data, "nickname_enabled"),
        avatar_enabled: bool_of(&data, "avatar_enabled"),
        wallpaper_enabled: bool_of(&data, "wallpaper_enabled"),
        timeout_ms: int_of(&data, "timeout_ms").max(1000).min(30000) as u64,
        fail_to_manual: bool_of(&data, "fail_to_manual"),
    };
    if cfg.provider.is_empty() {
        cfg.provider = "generic".to_string();
    }
    if cfg.timeout_ms == 0 {
        cfg.timeout_ms = 5000;
    }
    if cfg.api_key.is_empty() {
        cfg.api_key = current.api_key;
    }
    if let Err(e) = audit_policy::save_config(pool, &cfg).await {
        return err(500, &format!("保存审核配置失败: {}", e));
    }
    log_operation(pool, ctx, "保存外部审核配置", &format!("启用:{} 服务:{}", cfg.enabled, cfg.provider), "").await;
    ok("审核配置已保存", json!(cfg))
}

pub async fn test_audit_external_config(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let text = str_of(&data, "text");
    let text = if text.trim().is_empty() { "弦予音乐测试内容" } else { text.as_str() };
    let result = audit_policy::audit_text(pool, "nickname", text, json!({ "source": "admin_test" })).await;
    ok("测试完成", json!({
        "decision": match result.decision {
            crate::audit_policy::AuditDecision::Pass => "pass",
            crate::audit_policy::AuditDecision::Reject => "reject",
            crate::audit_policy::AuditDecision::Manual => "manual",
        },
        "reason": result.reason,
        "provider": result.provider,
    }))
}

pub async fn get_banned_words_config(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let cfg = audit_policy::load_banned_words(pool).await;
    let count = cfg.words.iter().filter(|w| !w.trim().is_empty()).count();
    ok("ok", json!({
        "enabled": cfg.enabled,
        "words": cfg.words,
        "count": count,
    }))
}

pub async fn save_banned_words_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let enabled = bool_of(&data, "enabled");
    let mut words: Vec<String> = data
        .get("words")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    // 去重（保留首次出现顺序）
    let mut seen = std::collections::HashSet::new();
    words.retain(|w| seen.insert(w.clone()));
    // 限制词条数量，防止配置过大
    if words.len() > 5000 {
        words.truncate(5000);
    }
    let cfg = BannedWordsConfig { enabled, words };
    if let Err(e) = audit_policy::save_banned_words(pool, &cfg).await {
        return err(500, &format!("保存违禁词库失败: {}", e));
    }
    log_operation(pool, ctx, "保存内置违禁词库", &format!("启用:{} 词数:{}", enabled, cfg.words.len()), "").await;
    ok("违禁词库已保存", json!({
        "enabled": cfg.enabled,
        "words": cfg.words,
        "count": cfg.words.len(),
    }))
}

pub async fn test_banned_words(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let text = str_of(&data, "text");
    let cfg = audit_policy::load_banned_words(pool).await;
    let hit = cfg
        .words
        .iter()
        .find(|w| !w.trim().is_empty() && text.contains(w.trim()));
    match hit {
        Some(w) => ok("测试完成", json!({
            "decision": "reject",
            "reason": format!("命中违禁词「{}」", w.trim()),
            "provider": "banned_words",
        })),
        None => ok("测试完成", json!({
            "decision": "pass",
            "reason": "未命中违禁词",
            "provider": "banned_words",
        })),
    }
}

/// 待审核头像列表 + 统计
pub async fn list_avatar_pending(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    ensure_avatar_table(pool).await;

    // 查询待审核头像（关联 app_users 获取用户名和当前头像）
    let rows = sqlx::query(
        "SELECT p.id, p.ciyuanxi_id, p.avatar_data, p.status, p.created_at, \
         u.nickname AS username, u.avatar_url AS current_avatar \
         FROM user_avatar_pending p \
         LEFT JOIN app_users u ON u.ciyuanxi_id = p.ciyuanxi_id \
         WHERE p.status = 'pending' \
         ORDER BY p.created_at DESC",
    )
    .fetch_all(pool)
    .await;

    let list = match rows {
        Ok(rows) => rows
            .iter()
            .map(|r| crate::admin::row_to_value(r))
            .collect::<Vec<Value>>(),
        Err(_) => return err(500, "数据库错误"),
    };

    // 统计各状态数量
    let stats_row = sqlx::query(
        "SELECT \
         SUM(status = 'pending') AS pending, \
         SUM(status = 'approved') AS approved, \
         SUM(status = 'rejected') AS rejected \
         FROM user_avatar_pending",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let stats = if let Some(ref r) = stats_row {
        json!({
            "pending": r.try_get::<i64, _>("pending").unwrap_or(0),
            "approved": r.try_get::<i64, _>("approved").unwrap_or(0),
            "rejected": r.try_get::<i64, _>("rejected").unwrap_or(0),
        })
    } else {
        json!({ "pending": 0, "approved": 0, "rejected": 0 })
    };

    log_operation(pool, ctx, "查看头像审核列表", "", "").await;
    ok("ok", json!({ "list": list, "stats": stats }))
}

/// 待审核改名申请列表
pub async fn list_nickname_pending(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    ensure_avatar_table(pool).await;

    let rows = sqlx::query(
        "SELECT n.id, n.ciyuanxi_id, n.nickname AS new_name, n.created_at, \
         u.nickname AS old_name \
         FROM user_nickname_pending n \
         LEFT JOIN app_users u ON u.ciyuanxi_id = n.ciyuanxi_id \
         WHERE n.status = 'pending' \
         ORDER BY n.created_at DESC",
    )
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows
                .iter()
                .map(|r| crate::admin::row_to_value(r))
                .collect();
            log_operation(pool, ctx, "查看改名审核列表", "", "").await;
            ok("ok", json!(list))
        }
        Err(_) => err(500, "数据库错误"),
    }
}

/// 统一审核记录列表（按状态，头像 + 改名）
pub async fn list_audit_records(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let status = str_of(&data, "status").trim().to_string();
    let status = if status.is_empty() { "pending".to_string() } else { status };
    if !["pending", "approved", "rejected"].contains(&status.as_str()) {
        return err(400, "无效的状态");
    }
    ensure_avatar_table(pool).await;

    let avatar_rows = sqlx::query(
        "SELECT p.id, p.ciyuanxi_id, p.avatar_data, p.status, p.created_at, p.reviewed_at, p.reviewed_by, \
         u.nickname AS username, u.avatar_url AS current_avatar \
         FROM user_avatar_pending p \
         LEFT JOIN app_users u ON u.ciyuanxi_id = p.ciyuanxi_id \
         WHERE p.status = ? \
         ORDER BY p.created_at DESC",
    )
    .bind(&status)
    .fetch_all(pool)
    .await;

    let nickname_rows = sqlx::query(
        "SELECT n.id, n.ciyuanxi_id, n.nickname AS new_name, n.status, n.created_at, n.reviewed_at, n.reviewed_by, \
         u.nickname AS old_name \
         FROM user_nickname_pending n \
         LEFT JOIN app_users u ON u.ciyuanxi_id = n.ciyuanxi_id \
         WHERE n.status = ? \
         ORDER BY n.created_at DESC",
    )
    .bind(&status)
    .fetch_all(pool)
    .await;

    let mut list: Vec<Value> = Vec::new();
    if let Ok(rows) = avatar_rows {
        for r in rows.iter() {
            list.push(json!({
                "type": "avatar",
                "id": r.try_get::<i64, _>("id").unwrap_or(0),
                "ciyuanxi_id": r.try_get::<String, _>("ciyuanxi_id").unwrap_or_default(),
                "username": r.try_get::<String, _>("username").unwrap_or_default(),
                "avatar_data": r.try_get::<String, _>("avatar_data").unwrap_or_default(),
                "current_avatar": r.try_get::<String, _>("current_avatar").unwrap_or_default(),
                "status": r.try_get::<String, _>("status").unwrap_or_default(),
                "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
                "reviewed_at": r.try_get::<String, _>("reviewed_at").unwrap_or_default(),
                "reviewed_by": r.try_get::<String, _>("reviewed_by").unwrap_or_default(),
            }));
        }
    }
    if let Ok(rows) = nickname_rows {
        for r in rows.iter() {
            list.push(json!({
                "type": "nickname",
                "id": r.try_get::<i64, _>("id").unwrap_or(0),
                "ciyuanxi_id": r.try_get::<String, _>("ciyuanxi_id").unwrap_or_default(),
                "old_name": r.try_get::<String, _>("old_name").unwrap_or_default(),
                "new_name": r.try_get::<String, _>("new_name").unwrap_or_default(),
                "status": r.try_get::<String, _>("status").unwrap_or_default(),
                "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
                "reviewed_at": r.try_get::<String, _>("reviewed_at").unwrap_or_default(),
                "reviewed_by": r.try_get::<String, _>("reviewed_by").unwrap_or_default(),
            }));
        }
    }

    // 统计（头像 + 改名合并）
    let stats_row = sqlx::query(
        "SELECT \
         (SELECT COUNT(*) FROM user_avatar_pending WHERE status='pending') + (SELECT COUNT(*) FROM user_nickname_pending WHERE status='pending') AS pending, \
         (SELECT COUNT(*) FROM user_avatar_pending WHERE status='approved') + (SELECT COUNT(*) FROM user_nickname_pending WHERE status='approved') AS approved, \
         (SELECT COUNT(*) FROM user_avatar_pending WHERE status='rejected') + (SELECT COUNT(*) FROM user_nickname_pending WHERE status='rejected') AS rejected",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let stats = if let Some(ref r) = stats_row {
        json!({
            "pending": r.try_get::<i64, _>("pending").unwrap_or(0),
            "approved": r.try_get::<i64, _>("approved").unwrap_or(0),
            "rejected": r.try_get::<i64, _>("rejected").unwrap_or(0),
        })
    } else {
        json!({ "pending": 0, "approved": 0, "rejected": 0 })
    };

    ok("ok", json!({ "list": list, "stats": stats }))
}

/// 审核通过头像
pub async fn approve_avatar(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "无效的ID");
    }
    let row = sqlx::query("SELECT id, ciyuanxi_id, avatar_data FROM user_avatar_pending WHERE id = ? AND status = 'pending' LIMIT 1")
        .bind(id)
        .fetch_optional(pool)
        .await;
    let Some(row) = row.ok().flatten() else {
        return err(404, "待审核记录不存在或已处理");
    };
    let ciyuanxi_id: String = row.get("ciyuanxi_id");
    let avatar_data: String = row.get("avatar_data");
    let _ = sqlx::query("UPDATE app_users SET avatar_url = ? WHERE ciyuanxi_id = ?")
        .bind(&avatar_data)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("UPDATE user_feedback SET nickname = (SELECT nickname FROM app_users WHERE ciyuanxi_id = ?) WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("UPDATE user_avatar_pending SET status = 'approved', reviewed_at = NOW(), reviewed_by = ? WHERE id = ?")
        .bind(&ctx.username)
        .bind(id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "审核通过头像", &format!("弦予号:{}", ciyuanxi_id), &format!("审核人:{} ID:{}", ctx.username, id)).await;
    ok("审核通过", Value::Null)
}

/// 审核拒绝头像
pub async fn reject_avatar(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "无效的ID");
    }
    let row = sqlx::query("SELECT id, ciyuanxi_id FROM user_avatar_pending WHERE id = ? AND status = 'pending' LIMIT 1")
        .bind(id)
        .fetch_optional(pool)
        .await;
    let Some(row) = row.ok().flatten() else {
        return err(404, "待审核记录不存在或已处理");
    };
    let ciyuanxi_id: String = row.get("ciyuanxi_id");
    let _ = sqlx::query("UPDATE user_avatar_pending SET status = 'rejected', reviewed_at = NOW(), reviewed_by = ? WHERE id = ?")
        .bind(&ctx.username)
        .bind(id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "拒绝头像", &format!("弦予号:{}", ciyuanxi_id), &format!("审核人:{} ID:{}", ctx.username, id)).await;
    ok("已拒绝", Value::Null)
}

/// 审核通过改名
pub async fn approve_nickname(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "无效的ID");
    }
    let row = sqlx::query("SELECT id, ciyuanxi_id, nickname FROM user_nickname_pending WHERE id = ? AND status = 'pending' LIMIT 1")
        .bind(id)
        .fetch_optional(pool)
        .await;
    let Some(row) = row.ok().flatten() else {
        return err(404, "待审核记录不存在或已处理");
    };
    let ciyuanxi_id: String = row.get("ciyuanxi_id");
    let nickname: String = row.get("nickname");
    let _ = sqlx::query("UPDATE app_users SET nickname = ? WHERE ciyuanxi_id = ?")
        .bind(&nickname)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("UPDATE user_feedback SET nickname = ? WHERE ciyuanxi_id = ?")
        .bind(&nickname)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("UPDATE user_nickname_pending SET status = 'approved', reviewed_at = NOW(), reviewed_by = ? WHERE id = ?")
        .bind(&ctx.username)
        .bind(id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "审核通过改名", &format!("弦予号:{}", ciyuanxi_id), &format!("新用户名:{} 审核人:{} ID:{}", nickname, ctx.username, id)).await;
    ok("审核通过", Value::Null)
}

/// 审核拒绝改名
pub async fn reject_nickname(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "无效的ID");
    }
    let row = sqlx::query("SELECT id, ciyuanxi_id, nickname FROM user_nickname_pending WHERE id = ? AND status = 'pending' LIMIT 1")
        .bind(id)
        .fetch_optional(pool)
        .await;
    let Some(row) = row.ok().flatten() else {
        return err(404, "待审核记录不存在或已处理");
    };
    let ciyuanxi_id: String = row.get("ciyuanxi_id");
    let nickname: String = row.get("nickname");
    let _ = sqlx::query("UPDATE user_nickname_pending SET status = 'rejected', reviewed_at = NOW(), reviewed_by = ? WHERE id = ?")
        .bind(&ctx.username)
        .bind(id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "拒绝改名", &format!("弦予号:{}", ciyuanxi_id), &format!("申请用户名:{} 审核人:{} ID:{}", nickname, ctx.username, id)).await;
    ok("已拒绝", Value::Null)
}
