use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, row_to_value, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

const DEFAULT_FEEDBACK_DAILY_LIMIT: i64 = 20;
const MAX_ADMIN_FEEDBACK_IMAGES: usize = 6;
const MAX_ADMIN_FEEDBACK_IMAGE_BYTES: usize = 8 * 1024 * 1024;

/// 解析 collaborators JSON 数组（不含 assignee 的额外协作者）
fn parse_collaborators(v: Option<&str>) -> Vec<String> {
    let s = v.unwrap_or("").trim();
    if s.is_empty() || s == "[]" {
        return Vec::new();
    }
    serde_json::from_str::<Vec<String>>(s).unwrap_or_default()
}

/// 解析 completed_by JSON 数组（[{admin, note}]）
fn parse_completed_by(v: Option<&str>) -> Vec<Value> {
    let s = v.unwrap_or("").trim();
    if s.is_empty() || s == "[]" {
        return Vec::new();
    }
    serde_json::from_str::<Vec<Value>>(s).unwrap_or_default()
}

/// 反馈参与人列表（assignee + 协作者，去重）
fn participants(assignee: &str, collaborators: &[String]) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    if !assignee.is_empty() {
        list.push(assignee.to_string());
    }
    for c in collaborators {
        if c != assignee && !list.contains(c) {
            list.push(c.clone());
        }
    }
    list
}

/// 写入一条管理员通知
async fn push_admin_notification(pool: &MySqlPool, feedback_id: i64, to_admin: &str, from_admin: &str, ntype: &str, content: &str) {
    let _ = sqlx::query(
        "INSERT INTO feedback_admin_notifications (feedback_id, to_admin, from_admin, type, content) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(feedback_id)
    .bind(to_admin)
    .bind(from_admin)
    .bind(ntype)
    .bind(content)
    .execute(pool)
    .await;
}

fn admin_feedback_img_dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").join("feedback")
}

fn admin_data_url_to_bytes(data_url: &str) -> Option<Vec<u8>> {
    // 兼容 data:image/xxx;base64,... 前缀
    let raw = data_url.split_once(',').map(|(_, v)| v).unwrap_or(data_url);
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(raw).ok()
}

/// 后台创建反馈用的图片压缩保存逻辑（复用与 APP 端一致的 uploads/feedback/ 目录）
fn compress_and_save_admin_feedback_image(bytes: &[u8], name: &str, max_w: u32, quality: u32) -> Option<String> {
    use image::GenericImageView;
    let img = image::load_from_memory(bytes).ok()?;
    let (w, h) = img.dimensions();
    let (nw, nh) = if w > max_w {
        (max_w, (h * max_w / w).max(1))
    } else {
        (w, h)
    };
    let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgb8();
    let dir = admin_feedback_img_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let path = dir.join(name);
    let file = std::fs::File::create(&path).ok()?;
    if image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), quality as u8)
        .encode(&rgb, nw, nh, image::ExtendedColorType::Rgb8)
        .is_err()
    {
        return None;
    }
    Some(format!("/uploads/feedback/{}", name))
}

fn admin_feedback_img_url(ctx: &AdminCtx, url: String) -> String {
    if url.starts_with("http://") || url.starts_with("https://") || url.is_empty() {
        return url;
    }
    let base = if !ctx.base_url.is_empty() {
        &ctx.base_url
    } else if !ctx.config.public_base_url.is_empty() {
        &ctx.config.public_base_url
    } else {
        return url;
    };
    format!("{}{}", base.trim_end_matches('/'), url)
}

async fn read_feedback_daily_limit(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT setting_value FROM server_settings WHERE setting_key = 'feedback_daily_limit' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .and_then(|v| v.trim().parse::<i64>().ok())
    .filter(|v| *v >= 0)
    .unwrap_or(DEFAULT_FEEDBACK_DAILY_LIMIT)
}

/// 反馈列表 + 统计
pub async fn list_feedback(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let status_filter = str_of(&data, "status_filter").trim().to_string();
    // 排序：post_time_desc（按提交时间倒序，默认）/ post_time_asc / update_desc
    let sort = str_of(&data, "sort").trim().to_string();
    let order_sql = match sort.as_str() {
        "post_time_asc" => "ORDER BY created_at ASC",
        "update_desc" => "ORDER BY updated_at DESC",
        _ => "ORDER BY created_at DESC",
    };

    // 排除已软删除的记录
    let (where_clause, binds): (String, Vec<String>) = if status_filter.is_empty() || status_filter == "all" {
        ("WHERE f.deleted_at IS NULL".to_string(), Vec::new())
    } else {
        ("WHERE f.deleted_at IS NULL AND f.status = ?".to_string(), vec![status_filter.clone()])
    };

    // 查询列表：不直接返回 LONGTEXT 日志正文，避免列表页过大
    let list_sql = format!(
        "SELECT f.id, f.ciyuanxi_id, COALESCE(u.nickname, f.nickname) AS nickname, f.title, f.content, f.status, f.category, f.feedback_type, f.images, f.admin_reply, f.replied_at, f.replied_by, f.assignee, f.collaborators, f.completed_by, f.resolve_note, f.ip, f.created_at, f.updated_at, f.claimed_at, f.resolved_at,
                f.log_meta,
                COALESCE(CHAR_LENGTH(f.error_logs), 0) AS error_logs_chars,
                COALESCE(CHAR_LENGTH(f.all_logs), 0) AS all_logs_chars,
                CASE WHEN f.error_logs IS NULL OR f.error_logs = '' THEN 0 ELSE 1 END AS has_error_logs,
                CASE WHEN f.all_logs IS NULL OR f.all_logs = '' THEN 0 ELSE 1 END AS has_all_logs,
                COALESCE(u.avatar_url, au.avatar_url) AS avatar_url
         FROM user_feedback f LEFT JOIN app_users u ON u.ciyuanxi_id = f.ciyuanxi_id LEFT JOIN admin_users au ON au.username = f.nickname AND f.ciyuanxi_id = '' {} {}",
        where_clause, order_sql
    );
    let mut list_query = sqlx::query(&list_sql);
    for b in &binds {
        list_query = list_query.bind(b);
    }
    let list: Vec<Value> = match list_query.fetch_all(pool).await {
        Ok(rows) => rows.iter().map(row_to_value).collect(),
        Err(_) => return err(500, "数据库错误"),
    };

    // 统计各状态数量（排除已软删除的记录）
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback WHERE deleted_at IS NULL")
        .fetch_one(pool).await.unwrap_or(0);
    let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback WHERE deleted_at IS NULL AND status = 'pending'")
        .fetch_one(pool).await.unwrap_or(0);
    let processing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback WHERE deleted_at IS NULL AND status = 'processing'")
        .fetch_one(pool).await.unwrap_or(0);
    let resolved: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback WHERE deleted_at IS NULL AND status = 'resolved'")
        .fetch_one(pool).await.unwrap_or(0);
    let rejected: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_feedback WHERE deleted_at IS NULL AND status = 'rejected'")
        .fetch_one(pool).await.unwrap_or(0);

    ok("ok", json!({
        "list": list,
        "stats": {
            "total": total,
            "pending": pending,
            "processing": processing,
            "resolved": resolved,
            "rejected": rejected,
        }
    }))
}

/// 反馈详情（包含日志正文）
pub async fn get_feedback_detail(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let row = sqlx::query(
        "SELECT f.*, COALESCE(u.nickname, f.nickname) AS nickname, COALESCE(u.avatar_url, au.avatar_url) AS avatar_url FROM user_feedback f LEFT JOIN app_users u ON u.ciyuanxi_id = f.ciyuanxi_id LEFT JOIN admin_users au ON au.username = f.nickname AND f.ciyuanxi_id = '' WHERE f.id = ? LIMIT 1",
    )
        .bind(id)
        .fetch_optional(pool)
        .await;
    match row {
        Ok(Some(r)) => ok("ok", row_to_value(&r)),
        Ok(None) => err(404, "反馈不存在"),
        Err(_) => err(500, "数据库错误"),
    }
}

/// 更新反馈状态
pub async fn update_feedback_status(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let status = str_of(&data, "status").trim().to_string();
    let reason = str_of(&data, "reason").trim().to_string();
    let valid = ["pending", "processing", "resolved", "rejected"];
    if id <= 0 || !valid.contains(&status.as_str()) {
        return err(400, "参数错误");
    }
    // 拒绝时记录操作人（与认领一致，打上 assignee 和 replied_by）
    // 归属校验：仅认领人可拒绝自己的反馈；未认领的（空认领人）任意管理员可拒绝
    if status == "rejected" {
        // 拒绝理由必填（与完成说明保持一致），并作为拒绝通知展示给用户
        if reason.is_empty() {
            return err(400, "拒绝理由不能为空");
        }
        if reason.chars().count() > 1000 {
            return err(400, "拒绝理由不能超过 1000 字");
        }
        let cur = sqlx::query_as::<_, (String, String)>(
            "SELECT status, COALESCE(assignee, '') FROM user_feedback WHERE id = ?",
        )
            .bind(id)
            .fetch_optional(pool)
            .await;
        match cur {
            Ok(Some((cur_status, assignee))) => {
                if !assignee.is_empty() && assignee != ctx.username {
                    return err(403, &format!("该反馈由 {} 认领，仅认领人可拒绝", assignee));
                }
                // 终态不可再变更
                if cur_status == "resolved" || cur_status == "rejected" {
                    return err(409, "该反馈已处于终态，无法变更");
                }
            }
            Ok(None) => return err(404, "反馈不存在"),
            Err(_) => return err(500, "服务器错误"),
        }
    }
    let upd = if status == "rejected" {
        sqlx::query(
            "UPDATE user_feedback SET status = ?,
                    assignee = ?, replied_by = ?, replied_at = NOW(),
                    reject_reason = ?, notified_at = NULL,
                    resolved_at = CASE WHEN ? = 'resolved' THEN COALESCE(resolved_at, NOW()) ELSE resolved_at END,
                    updated_at = NOW() WHERE id = ?",
        )
        .bind(&status)
        .bind(&ctx.username)
        .bind(&ctx.username)
        .bind(&reason)
        .bind(&status)
        .bind(id)
        .execute(pool)
        .await
    } else {
        sqlx::query(
            "UPDATE user_feedback SET status = ?,
                    claimed_at = CASE WHEN ? = 'processing' THEN COALESCE(claimed_at, NOW()) ELSE claimed_at END,
                    resolved_at = CASE WHEN ? = 'resolved' THEN COALESCE(resolved_at, NOW()) ELSE resolved_at END,
                    updated_at = NOW() WHERE id = ?",
        )
        .bind(&status)
        .bind(&status)
        .bind(&status)
        .bind(id)
        .execute(pool)
        .await
    };
    match upd {
        Ok(_) => {
            log_operation(pool, ctx, "更新反馈状态", &format!("id={}", id), &format!("status={} 操作人={}", status, ctx.username)).await;
            ok("状态已更新", serde_json::Value::Null)
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 获取反馈每日提交上限
pub async fn get_feedback_limit(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let limit = read_feedback_daily_limit(pool).await;
    ok("ok", json!({ "feedback_daily_limit": limit }))
}

/// 更新反馈每日提交上限
pub async fn update_feedback_limit(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let limit = int_of(&data, "feedback_daily_limit");
    if limit < 0 || limit > 10000 {
        return err(400, "每日上限需在 0 到 10000 之间");
    }
    let limit_text = limit.to_string();
    let result = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description)
         VALUES ('feedback_daily_limit', ?, '每个用户每天可提交的问题反馈数量上限，0 表示不限制')
         ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), description = VALUES(description)",
    )
    .bind(&limit_text)
    .execute(pool)
    .await;
    match result {
        Ok(_) => {
            log_operation(
                pool,
                ctx,
                "更新反馈提交上限",
                "feedback_daily_limit",
                &format!("limit={}", limit),
            )
            .await;
            ok("保存成功", json!({ "feedback_daily_limit": limit }))
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 认领反馈：将 pending 状态的反馈归属到当前管理员并置为 processing。
/// 仅待处理状态可认领，且只能由发起请求的管理员本人认领。
/// 处理中且当前认领人不是自己时，可"转认领"到自己名下，并通知原认领人。
pub async fn claim_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    // 先查当前状态与认领人，判断是否转认
    let cur = sqlx::query_as::<_, (String, String, String)>(
        "SELECT status, COALESCE(assignee, ''), COALESCE(title, '') FROM user_feedback WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;
    let (is_transfer, old_assignee, title) = match cur {
        Ok(Some((st, asg, t))) => (st == "processing" && !asg.is_empty() && asg != ctx.username, asg, t),
        Ok(None) => return err(404, "反馈不存在"),
        Err(_) => return err(500, "服务器错误"),
    };
    // 认领：pending 状态可直接认领；processing 且当前认领人不是自己时，可"转认领"到自己名下
    let upd = sqlx::query(
        "UPDATE user_feedback SET status = 'processing', assignee = ?, replied_by = ?, replied_at = NOW(), claimed_at = NOW(), collaborators = '', completed_by = '', updated_at = NOW()
         WHERE id = ? AND (status = 'pending' OR (status = 'processing' AND assignee != ?))",
    )
    .bind(&ctx.username)
    .bind(&ctx.username)
    .bind(id)
    .bind(&ctx.username)
    .execute(pool)
    .await;
    match upd {
        Ok(r) => {
            if r.rows_affected() == 0 {
                return err(409, "该反馈不存在、已被认领或当前不可认领，请刷新后重试");
            }
            // 转认时通知原认领人
            if is_transfer && !old_assignee.is_empty() {
                push_admin_notification(
                    pool,
                    id,
                    &old_assignee,
                    &ctx.username,
                    "transfer",
                    &format!("{} 已将您认领的反馈「{}」转认到自己名下", ctx.username, title),
                )
                .await;
            }
            log_operation(pool, ctx, "认领反馈", &format!("id={}", id), &format!("assignee={}", ctx.username)).await;
            ok("认领成功，已置为处理中", json!({ "id": id, "assignee": ctx.username, "status": "processing" }))
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 放弃认领反馈：仅放弃当前管理员自己的账号。
/// - 唯一认领人放弃：回归未认领（pending）状态，清空认领人、认领时间。
/// - 认领人放弃但仍有协作者：认领权移交给第一位协作者。
/// - 协作者放弃：仅将自己从协作者列表移除。
pub async fn abandon_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let cur = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT status, COALESCE(assignee, ''), COALESCE(collaborators, ''), COALESCE(completed_by, '') FROM user_feedback WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;
    let (status_val, assignee, collab_json, completed_json) = match cur {
        Ok(Some(v)) => v,
        Ok(None) => return err(404, "反馈不存在"),
        Err(_) => return err(500, "服务器错误"),
    };
    if status_val != "processing" {
        return err(409, "该反馈不是处理中状态，无法放弃");
    }
    let mut collabs = parse_collaborators(Some(&collab_json));
    let completed = parse_completed_by(Some(&completed_json));
    let is_assignee = assignee == ctx.username;
    let is_collab = collabs.iter().any(|c| c == &ctx.username);
    if !is_assignee && !is_collab {
        return err(403, "您未参与该反馈，无法放弃");
    }
    // 从已完成列表中移除自己
    let completed_filtered: Vec<Value> = completed
        .into_iter()
        .filter(|v| v.get("admin").and_then(|a| a.as_str()).unwrap_or("") != ctx.username)
        .collect();
    let new_completed_json = json!(completed_filtered).to_string();
    if is_assignee {
        if collabs.is_empty() {
            // 唯一认领人放弃，回归未认领
            let upd = sqlx::query(
                "UPDATE user_feedback SET status = 'pending', assignee = '', replied_by = '', replied_at = NULL, claimed_at = NULL, collaborators = '', completed_by = '', updated_at = NOW() WHERE id = ? AND status = 'processing'",
            )
            .bind(id)
            .execute(pool)
            .await;
            match upd {
                Ok(r) => {
                    if r.rows_affected() == 0 {
                        return err(409, "该反馈不存在或状态已变化，请刷新后重试");
                    }
                    log_operation(pool, ctx, "放弃认领反馈", &format!("id={}", id), "回归未认领状态").await;
                    ok("已放弃认领，回归未认领状态", json!({ "id": id, "status": "pending" }))
                }
                Err(_) => err(500, "服务器错误"),
            }
        } else {
            // 认领人放弃但仍有协作者：认领权移交给第一位协作者
            let new_assignee = collabs.remove(0);
            let new_collab_json = json!(collabs).to_string();
            let upd = sqlx::query(
                "UPDATE user_feedback SET assignee = ?, collaborators = ?, completed_by = ?, updated_at = NOW() WHERE id = ? AND status = 'processing'",
            )
            .bind(&new_assignee)
            .bind(&new_collab_json)
            .bind(&new_completed_json)
            .bind(id)
            .execute(pool)
            .await;
            match upd {
                Ok(_) => {
                    log_operation(pool, ctx, "放弃认领反馈", &format!("id={}", id), &format!("认领权移交={}", new_assignee)).await;
                    ok("已放弃认领，认领权已移交给其他协作者", json!({ "id": id, "status": "processing" }))
                }
                Err(_) => err(500, "服务器错误"),
            }
        }
    } else {
        // 协作者放弃：仅移除自己
        collabs.retain(|c| c != &ctx.username);
        let new_collab_json = json!(collabs).to_string();
        let upd = sqlx::query(
            "UPDATE user_feedback SET collaborators = ?, completed_by = ?, updated_at = NOW() WHERE id = ? AND status = 'processing'",
        )
        .bind(&new_collab_json)
        .bind(&new_completed_json)
        .bind(id)
        .execute(pool)
        .await;
        match upd {
            Ok(_) => {
                log_operation(pool, ctx, "退出协同", &format!("id={}", id), &format!("退出人={}", ctx.username)).await;
                ok("已退出协同，仅放弃自己的账号", json!({ "id": id, "status": "processing" }))
            }
            Err(_) => err(500, "服务器错误"),
        }
    }
}

/// 完成反馈：必填完成说明，将 processing 状态反馈置为 resolved 并记录说明，
/// 同时清空 notified_at 以便用户端拉取到该反馈的完成通知。
pub async fn resolve_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let note = str_of(&data, "note").trim().to_string();
    if id <= 0 {
        return err(400, "参数错误");
    }
    if note.is_empty() {
        return err(400, "完成说明不能为空");
    }
    if note.chars().count() > 1000 {
        return err(400, "完成说明不能超过 1000 字");
    }
    // 仅认领人可完成该反馈
    let cur = sqlx::query_as::<_, (String, String)>(
        "SELECT status, COALESCE(assignee, '') FROM user_feedback WHERE id = ?",
    )
        .bind(id)
        .fetch_optional(pool)
        .await;
    match cur {
        Ok(Some((status_val, assignee))) => {
            if status_val != "processing" {
                return err(409, "该反馈不是处理中状态，无法完成，请刷新后重试");
            }
            if !assignee.is_empty() && assignee != ctx.username {
                return err(403, &format!("该反馈由 {} 认领，仅认领人可完成", assignee));
            }
        }
        Ok(None) => return err(404, "反馈不存在"),
        Err(_) => return err(500, "服务器错误"),
    }
    let upd = sqlx::query(
        "UPDATE user_feedback SET status = 'resolved', resolve_note = ?, assignee = ?, replied_by = ?, replied_at = NOW(), resolved_at = NOW(), notified_at = NULL, updated_at = NOW() WHERE id = ? AND status = 'processing'",
    )
    .bind(&note)
    .bind(&ctx.username)
    .bind(&ctx.username)
    .bind(id)
    .execute(pool)
    .await;
    match upd {
        Ok(r) => {
            if r.rows_affected() == 0 {
                return err(409, "该反馈不是处理中状态，无法完成，请刷新后重试");
            }
            log_operation(pool, ctx, "完成反馈", &format!("id={}", id), &format!("note={}", note)).await;
            ok("已标记为已完成", json!({ "id": id, "status": "resolved" }))
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 发起协同：当前管理员请求加入某已认领反馈的协同处理。
/// 需先由认领人（assignee）弹窗同意，同意后才正式加入协作者列表。
pub async fn add_collaborator(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let cur = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT status, COALESCE(assignee, ''), COALESCE(collaborators, ''), COALESCE(title, '') FROM user_feedback WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;
    let (status_val, assignee, collab_json, title) = match cur {
        Ok(Some(v)) => v,
        Ok(None) => return err(404, "反馈不存在"),
        Err(_) => return err(500, "服务器错误"),
    };
    if status_val != "processing" {
        return err(409, "仅处理中的反馈可发起协同");
    }
    if assignee.is_empty() {
        return err(409, "该反馈尚未被认领，无法协同");
    }
    if assignee == ctx.username {
        return err(409, "您已是该反馈的认领人，无需协同");
    }
    let collabs = parse_collaborators(Some(&collab_json));
    if collabs.iter().any(|c| c == &ctx.username) {
        return err(409, "您已参与该反馈的协同");
    }
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM feedback_collab_requests WHERE feedback_id = ? AND requester = ? AND status = 'pending'",
    )
    .bind(id)
    .bind(&ctx.username)
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    if pending > 0 {
        return err(409, "您的协同请求正在等待认领人确认");
    }
    let ins = sqlx::query(
        "INSERT INTO feedback_collab_requests (feedback_id, feedback_title, requester, assignee) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&title)
    .bind(&ctx.username)
    .bind(&assignee)
    .execute(pool)
    .await;
    match ins {
        Ok(_) => {
            log_operation(pool, ctx, "发起协同请求", &format!("id={}", id), &format!("请求协同人={}", ctx.username)).await;
            ok("协同请求已发送，等待认领人确认", json!({ "requested": true }))
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 轮询待处理的协同请求（当前管理员作为认领人收到的请求）
pub async fn poll_collab_requests(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let rows = sqlx::query(
        "SELECT id, feedback_id, feedback_title, requester, created_at FROM feedback_collab_requests WHERE assignee = ? AND status = 'pending' ORDER BY created_at DESC LIMIT 50",
    )
    .bind(&ctx.username)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_value).collect();
            ok("ok", json!({ "list": list }))
        }
        Err(_) => err(500, "数据库错误"),
    }
}

/// 处理协同请求：认领人同意/拒绝。同意后 requester 加入协作者列表。
pub async fn respond_collab_request(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let request_id = int_of(&data, "request_id");
    let approve = int_of(&data, "approve") != 0;
    if request_id <= 0 {
        return err(400, "参数错误");
    }
    let cur = sqlx::query_as::<_, (i64, String, String, String, String)>(
        "SELECT feedback_id, requester, assignee, status, COALESCE(feedback_title, '') FROM feedback_collab_requests WHERE id = ?",
    )
    .bind(request_id)
    .fetch_optional(pool)
    .await;
    let (feedback_id, requester, assignee, req_status, title) = match cur {
        Ok(Some(v)) => v,
        Ok(None) => return err(404, "请求不存在"),
        Err(_) => return err(500, "服务器错误"),
    };
    if assignee != ctx.username {
        return err(403, "仅认领人可处理该请求");
    }
    if req_status != "pending" {
        return err(409, "该请求已处理");
    }
    if approve {
        let cur2 = sqlx::query_as::<_, (String, String)>(
            "SELECT COALESCE(collaborators, ''), COALESCE(assignee, '') FROM user_feedback WHERE id = ?",
        )
        .bind(feedback_id)
        .fetch_optional(pool)
        .await;
        match cur2 {
            Ok(Some((collab_json, fb_assignee))) => {
                if fb_assignee != ctx.username {
                    return err(409, "该反馈的认领人已变更，请刷新后重试");
                }
                let mut collabs = parse_collaborators(Some(&collab_json));
                if !collabs.iter().any(|c| c == &requester) {
                    collabs.push(requester.clone());
                }
                let new_json = json!(collabs).to_string();
                let upd = sqlx::query("UPDATE user_feedback SET collaborators = ?, updated_at = NOW() WHERE id = ?")
                    .bind(&new_json)
                    .bind(feedback_id)
                    .execute(pool)
                    .await;
                if upd.is_err() {
                    return err(500, "服务器错误");
                }
                push_admin_notification(
                    pool,
                    feedback_id,
                    &requester,
                    &ctx.username,
                    "collab_approved",
                    &format!("{} 已同意您协同处理反馈「{}」", ctx.username, title),
                )
                .await;
            }
            Ok(None) => return err(404, "反馈不存在"),
            Err(_) => return err(500, "服务器错误"),
        }
    } else {
        push_admin_notification(
            pool,
            feedback_id,
            &requester,
            &ctx.username,
            "collab_rejected",
            &format!("{} 拒绝了您协同处理反馈「{}」的请求", ctx.username, title),
        )
        .await;
    }
    let upd = sqlx::query("UPDATE feedback_collab_requests SET status = ?, responded_at = NOW() WHERE id = ?")
        .bind(if approve { "approved" } else { "rejected" })
        .bind(request_id)
        .execute(pool)
        .await;
    match upd {
        Ok(_) => {
            log_operation(
                pool,
                ctx,
                if approve { "同意协同请求" } else { "拒绝协同请求" },
                &format!("request_id={}", request_id),
                &format!("requester={}", requester),
            )
            .await;
            ok(if approve { "已同意协同" } else { "已拒绝协同" }, json!({ "approved": approve }))
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 轮询当前管理员未读通知（转认告知 / 协同结果 / 协同完成）
pub async fn poll_admin_notifications(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let rows = sqlx::query(
        "SELECT id, feedback_id, from_admin, type, content, created_at FROM feedback_admin_notifications WHERE to_admin = ? AND read_at IS NULL ORDER BY created_at DESC LIMIT 50",
    )
    .bind(&ctx.username)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_value).collect();
            ok("ok", json!({ "list": list }))
        }
        Err(_) => err(500, "数据库错误"),
    }
}

/// 标记通知为已读
pub async fn mark_notifications_read(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ids: Vec<i64> = data
        .get("ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();
    if ids.is_empty() {
        return ok("ok", json!({ "updated": 0 }));
    }
    let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
    let sql = format!(
        "UPDATE feedback_admin_notifications SET read_at = NOW() WHERE to_admin = ? AND id IN ({}) AND read_at IS NULL",
        placeholders.join(",")
    );
    let mut query = sqlx::query(&sql).bind(&ctx.username);
    for id in &ids {
        query = query.bind(id);
    }
    match query.execute(pool).await {
        Ok(r) => ok("ok", json!({ "updated": r.rows_affected() })),
        Err(_) => err(500, "服务器错误"),
    }
}

/// 协同完成确认：当前参与人点击"完成"，记录其完成状态与说明。
/// 当所有参与人（认领人 + 协作者）都确认完成后，反馈才真正置为已解决，
/// 并给所有仍在参与的账号 +1 统计（统计由 feedback_admin_stats 动态计算）。
pub async fn collaborator_complete(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let note = str_of(&data, "note").trim().to_string();
    if id <= 0 {
        return err(400, "参数错误");
    }
    if note.is_empty() {
        return err(400, "完成说明不能为空");
    }
    if note.chars().count() > 1000 {
        return err(400, "完成说明不能超过 1000 字");
    }
    let cur = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT status, COALESCE(assignee, ''), COALESCE(collaborators, ''), COALESCE(completed_by, ''), COALESCE(title, '') FROM user_feedback WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;
    let (status_val, assignee, collab_json, completed_json, title) = match cur {
        Ok(Some(v)) => v,
        Ok(None) => return err(404, "反馈不存在"),
        Err(_) => return err(500, "服务器错误"),
    };
    if status_val != "processing" {
        return err(409, "该反馈不是处理中状态，无法完成，请刷新后重试");
    }
    let collabs = parse_collaborators(Some(&collab_json));
    let mut completed = parse_completed_by(Some(&completed_json));
    let is_assignee = assignee == ctx.username;
    let is_collab = collabs.iter().any(|c| c == &ctx.username);
    if !is_assignee && !is_collab {
        return err(403, "您未参与该反馈，无法完成");
    }
    if completed.iter().any(|v| v.get("admin").and_then(|a| a.as_str()).unwrap_or("") == ctx.username) {
        return err(409, "您已完成确认，请等待其他参与人");
    }
    completed.push(json!({ "admin": ctx.username, "note": note }));
    let all_participants = participants(&assignee, &collabs);
    let total = all_participants.len().max(1);
    let done = completed.len();
    let all_done = done >= total;
    if all_done {
        let resolve_note = completed
            .iter()
            .filter_map(|v| v.get("note").and_then(|n| n.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        let new_completed_json = json!(completed).to_string();
        let upd = sqlx::query(
            "UPDATE user_feedback SET status = 'resolved', resolve_note = ?, replied_by = ?, replied_at = NOW(), resolved_at = NOW(), completed_by = ?, notified_at = NULL, updated_at = NOW() WHERE id = ? AND status = 'processing'",
        )
        .bind(&resolve_note)
        .bind(&ctx.username)
        .bind(&new_completed_json)
        .bind(id)
        .execute(pool)
        .await;
        match upd {
            Ok(r) => {
                if r.rows_affected() == 0 {
                    return err(409, "该反馈不是处理中状态，无法完成，请刷新后重试");
                }
                // 通知其他参与人协同已完成
                for p in &all_participants {
                    if p == &ctx.username {
                        continue;
                    }
                    push_admin_notification(
                        pool,
                        id,
                        p,
                        &ctx.username,
                        "collab_completed",
                        &format!("协同反馈「{}」已由全体参与人共同完成", title),
                    )
                    .await;
                }
                log_operation(pool, ctx, "协同完成反馈", &format!("id={}", id), &format!("参与人={:?}", all_participants)).await;
                ok("协同反馈已全部完成", json!({ "id": id, "status": "resolved", "completed": done, "total": total, "resolved": true }))
            }
            Err(_) => err(500, "服务器错误"),
        }
    } else {
        let new_completed_json = json!(completed).to_string();
        let upd = sqlx::query(
            "UPDATE user_feedback SET completed_by = ?, updated_at = NOW() WHERE id = ? AND status = 'processing'",
        )
        .bind(&new_completed_json)
        .bind(id)
        .execute(pool)
        .await;
        match upd {
            Ok(_) => {
                log_operation(pool, ctx, "确认协同完成", &format!("id={}", id), &format!("完成进度 {}/{}", done, total)).await;
                ok("已确认完成，等待其他参与人", json!({ "id": id, "status": "processing", "completed": done, "total": total, "resolved": false }))
            }
            Err(_) => err(500, "服务器错误"),
        }
    }
}

/// 后台新增反馈/建议事项（由管理员发起，非用户提交）
/// 入参：feedback_type（problem/suggestion）、title、content、images（base64 data URL 数组）、notify_external（是否外部同步通知）
/// 支持问题反馈与功能建议两种类型。勾选外部同步通知后，发布时会向所有启用中的通知邮箱发送提醒邮件。
pub async fn create_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let mut feedback_type = str_of(&data, "feedback_type").trim().to_string();
    if feedback_type.is_empty() {
        feedback_type = "problem".to_string();
    }
    if feedback_type != "problem" && feedback_type != "suggestion" && feedback_type != "appeal" {
        return err(400, "反馈类型不正确");
    }
    // 封禁申诉类型使用 category='appeal'，其余使用 category='feedback'
    let category = if feedback_type == "appeal" { "appeal" } else { "feedback" };
    let title = str_of(&data, "title").trim().to_string();
    let content = str_of(&data, "content").trim().to_string();
    let notify_external = int_of(&data, "notify_external") != 0;
    if title.is_empty() {
        return err(400, "标题不能为空");
    }
    if content.is_empty() {
        return err(400, "内容不能为空");
    }
    if title.chars().count() > 60 {
        return err(400, "标题不能超过 60 字");
    }
    if content.chars().count() > 1000 {
        return err(400, "内容不能超过 1000 字");
    }
    // 处理图片（base64 data URL 数组）
    let raw_images = data.get("images").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if raw_images.len() > MAX_ADMIN_FEEDBACK_IMAGES {
        return err(400, &format!("最多上传 {} 张图片", MAX_ADMIN_FEEDBACK_IMAGES));
    }
    let mut image_urls: Vec<String> = Vec::new();
    if !raw_images.is_empty() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        for (i, img_val) in raw_images.iter().enumerate() {
            let data_url = img_val.as_str().unwrap_or("");
            if data_url.is_empty() || data_url.len() > MAX_ADMIN_FEEDBACK_IMAGE_BYTES {
                continue;
            }
            let bytes = match admin_data_url_to_bytes(data_url) {
                Some(b) if b.len() <= MAX_ADMIN_FEEDBACK_IMAGE_BYTES => b,
                _ => continue,
            };
            let name = format!("admin_feedback_{}_{}.jpg", ts, i);
            if let Some(url) = compress_and_save_admin_feedback_image(&bytes, &name, 1600, 82) {
                image_urls.push(admin_feedback_img_url(ctx, url));
            }
        }
    }
    let images_json = json!(image_urls).to_string();
    // 后台创建：昵称显示为发起的管理员，ciyuanxi_id 留空标识为后台创建
    let result = sqlx::query(
        "INSERT INTO user_feedback (ciyuanxi_id, nickname, title, content, feedback_type, images, status, category, ip) VALUES ('', ?, ?, ?, ?, ?, 'pending', ?, ?)",
    )
        .bind(&ctx.username)
        .bind(&title)
        .bind(&content)
        .bind(&feedback_type)
        .bind(&images_json)
        .bind(&category)
        .bind(&ctx.ip)
        .execute(pool)
        .await;
    let new_id = match result {
        Ok(r) => r.last_insert_id() as i64,
        Err(_) => return err(500, "服务器错误"),
    };
    log_operation(pool, ctx, "后台新增反馈", &format!("id={}", new_id), &format!("类型:{}", feedback_type)).await;
    // 外部同步通知：向所有启用中的通知邮箱发送提醒
    if notify_external {
        let sent = notify_external_emails(pool, ctx, &feedback_type, &title, &content).await;
        return ok(
            "创建成功",
            json!({ "id": new_id, "notify_sent": sent, "notify_total": sent.len() }),
        );
    }
    ok("创建成功", json!({ "id": new_id }))
}

/// 向所有启用中的通知邮箱统一发送邮件提醒，返回成功发送的邮箱列表
async fn notify_external_emails(pool: &MySqlPool, ctx: &AdminCtx, feedback_type: &str, title: &str, content: &str) -> Vec<String> {
    let type_label = if feedback_type == "suggestion" { "功能建议" } else { "问题反馈" };
    let subject = format!("【弦予后台】新增{}：{}", type_label, title);
    let context = if content.len() > 200 {
        format!(
            "后台新增了一条{}事项，请及时查看处理。\n\n标题：{}\n内容：{}…",
            type_label,
            title,
            content.chars().take(200).collect::<String>()
        )
    } else {
        format!("后台新增了一条{}事项，请及时查看处理。\n\n标题：{}\n内容：{}", type_label, title, content)
    };
    crate::admin::email::notify_external_emails_for_module(pool, &ctx.config, &ctx.ip, "feedback", &subject, &context, "", &ctx.base_url).await
}

/// 各管理账号处理反馈量统计
/// 统计每个管理员（认领人 assignee + 协作者）处理了多少反馈，及其处理结果分布。
/// 协同反馈完成时，所有仍在参与的账号都会 +1。
pub async fn feedback_admin_stats(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let rows = sqlx::query(
        "SELECT COALESCE(assignee, '') AS assignee, COALESCE(collaborators, '') AS collaborators, status FROM user_feedback WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await;
    let mut map: std::collections::BTreeMap<String, [i64; 5]> = std::collections::BTreeMap::new();
    let mut grand_total: i64 = 0;
    match rows {
        Ok(rows) => {
            for r in rows {
                let assignee: String = r.get("assignee");
                let collab_json: String = r.get("collaborators");
                let status: String = r.get("status");
                let collabs = parse_collaborators(Some(&collab_json));
                let mut names: Vec<String> = Vec::new();
                if !assignee.is_empty() {
                    names.push(assignee);
                }
                for c in collabs {
                    if !names.contains(&c) {
                        names.push(c);
                    }
                }
                if names.is_empty() {
                    continue;
                }
                grand_total += 1;
                for name in names {
                    let entry = map.entry(name).or_insert([0; 5]);
                    entry[0] += 1;
                    match status.as_str() {
                        "processing" => entry[1] += 1,
                        "resolved" => entry[2] += 1,
                        "rejected" => entry[3] += 1,
                        "pending" => entry[4] += 1,
                        _ => {}
                    }
                }
            }
        }
        Err(_) => return err(500, "数据库错误"),
    }
    let mut list: Vec<Value> = map
        .iter()
        .map(|(name, arr)| {
            json!({
                "admin_name": name,
                "total": arr[0],
                "processing": arr[1],
                "resolved": arr[2],
                "rejected": arr[3],
                "pending": arr[4],
            })
        })
        .collect();
    list.sort_by(|a, b| {
        b.get("total")
            .and_then(|t| t.as_i64())
            .unwrap_or(0)
            .cmp(&a.get("total").and_then(|t| t.as_i64()).unwrap_or(0))
    });
    // 未认领统计
    let unclaimed_total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_feedback WHERE deleted_at IS NULL AND assignee = '' AND (collaborators IS NULL OR collaborators = '' OR collaborators = '[]')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    let unclaimed_pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_feedback WHERE deleted_at IS NULL AND status = 'pending' AND assignee = ''",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    if unclaimed_total > 0 {
        list.insert(
            0,
            json!({
                "admin_name": "未认领",
                "total": unclaimed_total,
                "processing": 0,
                "resolved": 0,
                "rejected": 0,
                "pending": unclaimed_pending,
            }),
        );
    }
    log_operation(pool, ctx, "查看反馈处理统计", "", "").await;
    ok("ok", json!({ "list": list, "grand_total": grand_total }))
}

/// 批量软删除反馈记录（移入回收站，14天后自动过期）
/// 入参：ids（数组）
pub async fn batch_delete_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ids: Vec<i64> = data
        .get("ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .filter(|id| *id > 0)
                .collect()
        })
        .unwrap_or_default();
    if ids.is_empty() {
        return err(400, "请选择要删除的记录");
    }
    // 构建占位符
    let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
    let sql = format!(
        "UPDATE user_feedback SET deleted_at = NOW(), deleted_by = ? WHERE id IN ({}) AND deleted_at IS NULL",
        placeholders.join(",")
    );
    let mut query = sqlx::query(&sql).bind(&ctx.username);
    for id in &ids {
        query = query.bind(id);
    }
    match query.execute(pool).await {
        Ok(r) => {
            let affected = r.rows_affected();
            log_operation(pool, ctx, "批量删除反馈", &format!("ids={:?}", ids), &format!("删除{}条, 操作人={}", affected, ctx.username)).await;
            ok("删除成功", json!({ "deleted": affected }))
        }
        Err(_) => err(500, "服务器错误"),
    }
}

/// 回收站列表：展示已软删除的记录（14天内可恢复）
pub async fn list_recycle_bin(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let list_sql = "SELECT id, ciyuanxi_id, nickname, title, content, status, category, feedback_type,
                           assignee, deleted_at, deleted_by, created_at,
                           COALESCE(TIMESTAMPDIFF(HOUR, deleted_at, NOW()), 0) AS hours_since_deleted
                    FROM user_feedback
                    WHERE deleted_at IS NOT NULL AND deleted_at >= DATE_SUB(NOW(), INTERVAL 14 DAY)
                    ORDER BY deleted_at DESC";
    let list: Vec<Value> = match sqlx::query(list_sql).fetch_all(pool).await {
        Ok(rows) => rows.iter().map(row_to_value).collect(),
        Err(_) => return err(500, "数据库错误"),
    };
    // 计算每条记录的剩余可恢复小时数
    let items: Vec<Value> = list
        .iter()
        .map(|v| {
            let hours = v.get("hours_since_deleted").and_then(|h| h.as_i64()).unwrap_or(0);
            let remaining_hours = (14 * 24 - hours).max(0);
            let mut obj = v.clone();
            if let Some(m) = obj.as_object_mut() {
                m.insert("remaining_hours".to_string(), json!(remaining_hours));
            }
            obj
        })
        .collect();
    ok("ok", json!({ "list": items }))
}

/// 从回收站恢复反馈记录
/// 入参：id
pub async fn restore_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let upd = sqlx::query(
        "UPDATE user_feedback SET deleted_at = NULL, deleted_by = '' WHERE id = ? AND deleted_at IS NOT NULL",
    )
    .bind(id)
    .execute(pool)
    .await;
    match upd {
        Ok(r) => {
            if r.rows_affected() == 0 {
                return err(404, "记录不存在或已不在回收站中");
            }
            log_operation(pool, ctx, "恢复反馈记录", &format!("id={}", id), &format!("操作人={}", ctx.username)).await;
            ok("恢复成功", serde_json::Value::Null)
        }
        Err(_) => err(500, "服务器错误"),
    }
}
