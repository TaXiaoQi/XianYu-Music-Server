use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, row_to_value, AdminCtx};
use crate::handlers::helpers::{bool_of, int_of, parse_body, str_of};
use super::*;

pub async fn list_feedback(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let status_filter = str_of(&data, "status_filter").trim().to_string();
    let sort = str_of(&data, "sort").trim().to_string();
    let order_sql = match sort.as_str() {
        "post_time_asc" => "ORDER BY created_at ASC",
        "update_desc" => "ORDER BY updated_at DESC",
        _ => "ORDER BY created_at DESC",
    };

    let (where_clause, binds): (String, Vec<String>) = if status_filter.is_empty() || status_filter == "all" {
        ("WHERE f.deleted_at IS NULL".to_string(), Vec::new())
    } else {
        ("WHERE f.deleted_at IS NULL AND f.status = ?".to_string(), vec![status_filter.clone()])
    };

    let list_sql = format!(
        "SELECT f.id, f.ciyuanxi_id, COALESCE(u.nickname, f.nickname) AS nickname, f.title, f.content, f.status, f.category, f.feedback_type, f.platform, f.app_version, f.images, f.admin_reply, f.replied_at, f.replied_by, f.assignee, f.collaborators, f.completed_by, f.resolve_note, f.resolve_images, f.reject_reason, f.ip, f.created_at, f.updated_at, f.claimed_at, f.resolved_at,
                f.log_meta,
                f.device_id, f.device_brand, f.device_model, f.os_version, f.architecture, f.machine_name,
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

pub async fn resolve_beta_application(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let note = str_of(&data, "note").trim().to_string();
    let device_note = str_of(&data, "device_note").trim().to_string();
    if id <= 0 {
        return err(400, "参数错误");
    }
    if note.is_empty() {
        return err(400, "回执内容不能为空");
    }
    if note.chars().count() > 1000 {
        return err(400, "回执内容不能超过 1000 字");
    }
    if device_note.chars().count() > 255 {
        return err(400, "设备备注不能超过 255 字");
    }
    let cur = sqlx::query_as::<_, (String, String, String)>(
        "SELECT status, COALESCE(feedback_type, ''), COALESCE(device_id, '') FROM user_feedback WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;
    let (status_val, fb_type, device_id) = match cur {
        Ok(Some(v)) => v,
        Ok(None) => return err(404, "反馈不存在"),
        Err(_) => return err(500, "服务器错误"),
    };
    if fb_type != "beta" {
        return err(400, "该反馈不是内测申请");
    }
    if status_val == "resolved" || status_val == "rejected" {
        return err(409, "该申请已处于终态，无法变更");
    }
    let upd = sqlx::query(
        "UPDATE user_feedback SET status = 'resolved', resolve_note = ?, resolve_images = '[]', assignee = ?, replied_by = ?, replied_at = NOW(), resolved_at = NOW(), notified_at = NULL, updated_at = NOW() WHERE id = ?",
    )
    .bind(&note)
    .bind(&ctx.username)
    .bind(&ctx.username)
    .bind(id)
    .execute(pool)
    .await;
    match upd {
        Ok(r) if r.rows_affected() > 0 => {
            let device_added = if device_id.is_empty() {
                false
            } else {
                let final_note = if device_note.is_empty() {
                    format!("内测申请自动加入（反馈#{})", id)
                } else {
                    device_note.clone()
                };
                sqlx::query("INSERT IGNORE INTO beta_testers (device_id, note) VALUES (?, ?)")
                    .bind(&device_id)
                    .bind(final_note)
                    .execute(pool)
                    .await
                    .map(|r| r.rows_affected() > 0)
                    .unwrap_or(false)
            };
            log_operation(
                pool,
                ctx,
                "同意内测申请",
                &format!("id={}", id),
                &format!(
                    "设备={} 备注={} 加入名单={} 操作人={}",
                    device_id,
                    if device_note.is_empty() { "-" } else { &device_note },
                    device_added,
                    ctx.username
                ),
            )
            .await;
            ok("已同意该内测申请", json!({ "id": id, "device_added": device_added }))
        }
        Ok(_) => err(409, "该申请状态已变化，请刷新后重试"),
        Err(e) => { tracing::error!("服务器错误: {e}"); err(500, "服务器错误") },
    }
}

pub async fn update_feedback_status(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let status = str_of(&data, "status").trim().to_string();
    let reason = str_of(&data, "reason").trim().to_string();
    let valid = ["pending", "processing", "resolved", "rejected"];
    if id <= 0 || !valid.contains(&status.as_str()) {
        return err(400, "参数错误");
    }
    if status == "rejected" {
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

pub async fn get_feedback_limit(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let limit = read_feedback_daily_limit(pool).await;
    ok("ok", json!({ "feedback_daily_limit": limit }))
}

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
    let resolve_images_json = save_admin_resolve_images(&data, ctx);
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
        "UPDATE user_feedback SET status = 'resolved', resolve_note = ?, resolve_images = ?, assignee = ?, replied_by = ?, replied_at = NOW(), resolved_at = NOW(), notified_at = NULL, updated_at = NOW() WHERE id = ? AND status = 'processing'",
    )
    .bind(&note)
    .bind(&resolve_images_json)
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

pub async fn create_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let feedback_type = str_of(&data, "feedback_type").trim().to_string();
    if feedback_type.is_empty() {
        return err(400, "请选择反馈类型");
    }
    if feedback_type != "problem" && feedback_type != "suggestion" && feedback_type != "appeal" {
        return err(400, "反馈类型不正确");
    }
    let mut platform = str_of(&data, "platform").trim().to_string();
    if platform.is_empty() {
        return err(400, "请选择平台版本");
    }
    if platform != "desktop" && platform != "mobile" && platform != "watch" {
        return err(400, "平台版本不正确");
    }
    if feedback_type == "appeal" {
        platform = "desktop".to_string();
    }
    let app_version = str_of(&data, "app_version").trim().to_string();
    let category = if feedback_type == "appeal" { "appeal" } else { "feedback" };
    let title = str_of(&data, "title").trim().to_string();
    let content = str_of(&data, "content").trim().to_string();
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
    let result = sqlx::query(
        "INSERT INTO user_feedback (ciyuanxi_id, nickname, title, content, feedback_type, platform, app_version, images, status, category, ip) VALUES ('', ?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?)",
    )
        .bind(&ctx.username)
        .bind(&title)
        .bind(&content)
        .bind(&feedback_type)
        .bind(&platform)
        .bind(&app_version)
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
    if bool_of(&data, "notify_external") {
        let type_label = match feedback_type.as_str() {
            "suggestion" => "功能建议",
            "appeal" => "账号申诉",
            _ => "问题反馈",
        };
        let image = image_urls.first().map(|s| s.as_str()).unwrap_or("");
        crate::admin::email::notify_external_emails_for_module(
            pool,
            &ctx.config,
            &ctx.ip,
            "feedback",
            "【弦予后台】新反馈待处理",
            &format!("管理员 {} 后台新建了{}「{}」，请及时处理。", ctx.username, type_label, title),
            image,
            &ctx.base_url,
        ).await;
    }
    ok("创建成功", json!({ "id": new_id }))
}

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

pub async fn list_recycle_bin(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let list_sql = "SELECT id, ciyuanxi_id, nickname, title, content, status, category, feedback_type, platform,
                           assignee, deleted_at, deleted_by, created_at,
                           COALESCE(TIMESTAMPDIFF(HOUR, deleted_at, NOW()), 0) AS hours_since_deleted
                    FROM user_feedback
                    WHERE deleted_at IS NOT NULL AND deleted_at >= DATE_SUB(NOW(), INTERVAL 14 DAY)
                    ORDER BY deleted_at DESC";
    let list: Vec<Value> = match sqlx::query(list_sql).fetch_all(pool).await {
        Ok(rows) => rows.iter().map(row_to_value).collect(),
        Err(_) => return err(500, "数据库错误"),
    };
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
