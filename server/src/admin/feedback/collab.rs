use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::*;
use crate::handlers::helpers::{int_of, parse_body, str_of};

pub async fn claim_feedback(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
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
    let completed_filtered: Vec<Value> = completed
        .into_iter()
        .filter(|v| v.get("admin").and_then(|a| a.as_str()).unwrap_or("") != ctx.username)
        .collect();
    let new_completed_json = json!(completed_filtered).to_string();
    if is_assignee {
        if collabs.is_empty() {
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
        let resolve_images_json = save_admin_resolve_images(&data, ctx);
        let new_completed_json = json!(completed).to_string();
        let upd = sqlx::query(
            "UPDATE user_feedback SET status = 'resolved', resolve_note = ?, resolve_images = ?, replied_by = ?, replied_at = NOW(), resolved_at = NOW(), completed_by = ?, notified_at = NULL, updated_at = NOW() WHERE id = ? AND status = 'processing'",
        )
        .bind(&resolve_note)
        .bind(&resolve_images_json)
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

