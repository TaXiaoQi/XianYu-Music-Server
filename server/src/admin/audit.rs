use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};
use crate::audit_policy::{self, AuditExternalConfig};
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

/// 待审核头像列表 + 统计
pub async fn list_avatar_pending(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    ensure_avatar_table(pool).await;

    // 查询待审核头像（关联 app_users 获取用户名和当前头像）
    let rows = sqlx::query(
        "SELECT p.id, p.ciyuanxi_id, p.avatar_data, p.status, p.created_at, \
         u.username, u.avatar_url AS current_avatar \
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
         u.username AS old_name \
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
    let _ = sqlx::query("UPDATE app_users SET username = ? WHERE ciyuanxi_id = ?")
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
