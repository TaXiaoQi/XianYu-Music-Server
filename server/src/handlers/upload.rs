use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;

use crate::audit_policy::{self, AuditDecision};
use crate::handlers::helpers::{parse_body, str_of};
use crate::response::ReqCtx;

/// 校验用户存在且启用，返回是否通过
async fn user_active(pool: &MySqlPool, ciyuanxi_id: &str) -> bool {
    sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? AND status = 1")
        .bind(ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some()
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

/// 头像上传（JSON base64 模式，与 PHP 模式1对应）
/// 入参：ciyuanxi_id/user_id, avatar_data(data:image/xxx;base64,...)
pub async fn upload_avatar(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    }
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "用户ID不能为空");
    }
    let avatar_data = str_of(&data, "avatar_data").to_string();
    if !avatar_data.starts_with("data:image/") {
        return ctx.err(400, "无效的图片数据格式");
    }
    if avatar_data.len() > 200 * 1024 {
        return ctx.err(400, "图片数据过大，请使用更小的图片");
    }
    if !user_active(pool, &ciyuanxi_id).await {
        return ctx.err(404, "用户不存在");
    }
    if let Some(msg) = avatar_submit_block_message(pool, &ciyuanxi_id).await {
        return ctx.err(429, msg);
    }
    let audit = audit_policy::audit_image(
        pool,
        "avatar",
        &avatar_data,
        json!({ "ciyuanxi_id": ciyuanxi_id }),
    )
    .await;
    if audit.decision == AuditDecision::Pass {
        let _ = sqlx::query("UPDATE app_users SET avatar_url = ? WHERE ciyuanxi_id = ?")
            .bind(&avatar_data)
            .bind(&ciyuanxi_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, status, reviewed_at, reviewed_by) VALUES (?, ?, 'approved', NOW(), ?)")
            .bind(&ciyuanxi_id)
            .bind(&avatar_data)
            .bind(format!("external:{}", audit.provider))
            .execute(pool)
            .await;
        return ctx.ok("头像已通过机审并立即生效", json!({ "status": "approved" }));
    }
    if audit.decision == AuditDecision::Reject {
        let _ = sqlx::query("DELETE FROM user_avatar_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
            .bind(&ciyuanxi_id)
            .execute(pool)
            .await;
        let _ = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, status, reviewed_at, reviewed_by) VALUES (?, ?, 'rejected', NOW(), ?)")
            .bind(&ciyuanxi_id)
            .bind(&avatar_data)
            .bind(format!("external:{}", audit.provider))
            .execute(pool)
            .await;
        return ctx.err(400, if audit.reason.is_empty() { "头像未通过机审" } else { &audit.reason });
    }
    let _ = sqlx::query("DELETE FROM user_avatar_pending WHERE ciyuanxi_id = ? AND status = 'pending'")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    let ins = sqlx::query("INSERT INTO user_avatar_pending (ciyuanxi_id, avatar_data, status) VALUES (?,?, 'pending')")
        .bind(&ciyuanxi_id)
        .bind(&avatar_data)
        .execute(pool)
        .await;
    match ins {
        Ok(_) => ctx.ok("头像已上传，等待管理员审核", json!({ "status": "pending" })),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}
