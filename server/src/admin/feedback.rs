use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, row_to_value, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

const DEFAULT_FEEDBACK_DAILY_LIMIT: i64 = 20;
const MAX_ADMIN_FEEDBACK_IMAGES: usize = 6;
const MAX_ADMIN_FEEDBACK_IMAGE_BYTES: usize = 8 * 1024 * 1024;

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
        "SELECT f.id, f.ciyuanxi_id, COALESCE(u.nickname, f.nickname) AS nickname, f.title, f.content, f.status, f.category, f.feedback_type, f.images, f.admin_reply, f.replied_at, f.replied_by, f.assignee, f.resolve_note, f.ip, f.created_at, f.updated_at, f.claimed_at, f.resolved_at,
                f.log_meta,
                COALESCE(CHAR_LENGTH(f.error_logs), 0) AS error_logs_chars,
                COALESCE(CHAR_LENGTH(f.all_logs), 0) AS all_logs_chars,
                CASE WHEN f.error_logs IS NULL OR f.error_logs = '' THEN 0 ELSE 1 END AS has_error_logs,
                CASE WHEN f.all_logs IS NULL OR f.all_logs = '' THEN 0 ELSE 1 END AS has_all_logs,
                u.avatar_url AS avatar_url
         FROM user_feedback f LEFT JOIN app_users u ON u.ciyuanxi_id = f.ciyuanxi_id {} {}",
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
        "SELECT f.*, COALESCE(u.nickname, f.nickname) AS nickname, u.avatar_url AS avatar_url FROM user_feedback f LEFT JOIN app_users u ON u.ciyuanxi_id = f.ciyuanxi_id WHERE f.id = ? LIMIT 1",
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
    let valid = ["pending", "processing", "resolved", "rejected"];
    if id <= 0 || !valid.contains(&status.as_str()) {
        return err(400, "参数错误");
    }
    // 拒绝时记录操作人（与认领一致，打上 assignee 和 replied_by）
    // 归属校验：仅认领人可拒绝自己的反馈；未认领的（空认领人）任意管理员可拒绝
    if status == "rejected" {
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
                    resolved_at = CASE WHEN ? = 'resolved' THEN COALESCE(resolved_at, NOW()) ELSE resolved_at END,
                    updated_at = NOW() WHERE id = ?",
        )
        .bind(&status)
        .bind(&ctx.username)
        .bind(&ctx.username)
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
pub async fn claim_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    // 认领：pending 状态可直接认领；processing 且当前认领人不是自己时，可"转认领"到自己名下
    let upd = sqlx::query(
        "UPDATE user_feedback SET status = 'processing', assignee = ?, replied_by = ?, replied_at = NOW(), claimed_at = NOW(), updated_at = NOW()
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
            log_operation(pool, ctx, "认领反馈", &format!("id={}", id), &format!("assignee={}", ctx.username)).await;
            ok("认领成功，已置为处理中", json!({ "id": id, "assignee": ctx.username, "status": "processing" }))
        }
        Err(_) => err(500, "服务器错误"),
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
/// 统计每个管理员（认领人 assignee）处理了多少反馈，及其处理结果分布。
pub async fn feedback_admin_stats(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let rows = sqlx::query(
        "SELECT COALESCE(NULLIF(assignee, ''), '未认领') AS admin_name,
                COUNT(*) AS total,
                CAST(SUM(CASE WHEN status = 'processing' THEN 1 ELSE 0 END) AS SIGNED) AS processing,
                CAST(SUM(CASE WHEN status = 'resolved' THEN 1 ELSE 0 END) AS SIGNED) AS resolved,
                CAST(SUM(CASE WHEN status = 'rejected' THEN 1 ELSE 0 END) AS SIGNED) AS rejected,
                CAST(SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) AS SIGNED) AS pending
         FROM user_feedback
         WHERE deleted_at IS NULL
         GROUP BY admin_name
         ORDER BY total DESC",
    )
        .fetch_all(pool)
        .await;
    let list: Vec<Value> = match rows {
        Ok(rows) => rows
            .iter()
            .map(|r| {
                json!({
                    "admin_name": r.get::<String, _>("admin_name"),
                    "total": r.get::<i64, _>("total"),
                    "processing": r.get::<i64, _>("processing"),
                    "resolved": r.get::<i64, _>("resolved"),
                    "rejected": r.get::<i64, _>("rejected"),
                    "pending": r.get::<i64, _>("pending"),
                })
            })
            .collect(),
        Err(_) => return err(500, "数据库错误"),
    };
    let grand_total: i64 = list.iter().map(|v| v.get("total").and_then(|t| t.as_i64()).unwrap_or(0)).sum();
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
