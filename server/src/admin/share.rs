use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body};

/// 查看分享详情
pub async fn view_share_detail(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return ok("无效的ID", Value::Null);
    }
    match sqlx::query("SELECT * FROM share_log WHERE id = ?").bind(id).fetch_optional(pool).await {
        Ok(Some(row)) => {
            let map = crate::admin::row_to_value(&row);
            log_operation(pool, ctx, "查看分享详情", &format!("id={}", id), "").await;
            ok("", map)
        }
        Ok(None) => super::err(404, "记录不存在"),
        Err(_) => super::err(500, "查询失败"),
    }
}

fn remove_dir_all(path: &std::path::Path) {
    if let Ok(entries) = std::fs::read_dir(path) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                remove_dir_all(&p);
                let _ = std::fs::remove_dir(&p);
            } else {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
}

/// 删除过期分享文件
pub async fn delete_expired_shares(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    let rows = sqlx::query("SELECT id, share_id, song_name, expired_at FROM share_log WHERE expired_at < NOW() ORDER BY expired_at ASC")
        .fetch_all(pool)
        .await;
    let expired = match rows {
        Ok(r) => r,
        Err(_) => return super::err(500, "删除失败"),
    };
    if expired.is_empty() {
        log_operation(pool, ctx, "清理过期分享", "", "无过期分享").await;
        return ok("没有过期分享", json!({ "deleted_count": 0, "details": [] }));
    }
    let shares_base = std::path::Path::new("fenxiang").join("public").join("1");
    let mut deleted_count = 0;
    let mut missing_count = 0;
    let mut details: Vec<String> = Vec::new();
    for r in &expired {
        let share_id: String = r.get("share_id");
        let song_name: String = r.get("song_name");
        if share_id.is_empty() {
            continue;
        }
        let dir = shares_base.join(&share_id);
        let mut deleted = false;
        let mut reason = String::new();
        if dir.is_dir() {
            // 先清空再删除目录，失败回退到重命名
            remove_dir_all(&dir);
            if std::fs::remove_dir(&dir).is_ok() || !dir.exists() {
                deleted = true;
                deleted_count += 1;
                let expired_at: String = r.get("expired_at");
                details.push(format!("已删除: share_id={} ({}) 过期时间: {}", share_id, song_name, expired_at));
            } else {
                // 尝试重命名延迟删除
                let renamed = shares_base.join(format!("{}_deleted_{}", share_id, std::process::id()));
                if std::fs::rename(&dir, &renamed).is_ok() {
                    let _ = std::fs::remove_dir_all(&renamed);
                    deleted = true;
                    deleted_count += 1;
                } else {
                    reason = "目录删除失败".into();
                }
            }
        } else {
            missing_count += 1;
            reason = "目录不存在".into();
        }
        if !deleted && !reason.is_empty() {
            details.push(format!("跳过: share_id={} 原因: {}", share_id, reason));
        }
    }
    let msg = format!(
        "删除完成：共删除 {} 个过期分享文件{}",
        deleted_count,
        if missing_count > 0 { format!("，{} 个目录不存在", missing_count) } else { String::new() }
    );
    log_operation(pool, ctx, "清理过期分享", "", &format!("删除{} 缺失{}", deleted_count, missing_count)).await;
    ok(&msg, json!({
        "deleted_count": deleted_count,
        "missing_count": missing_count,
        "total_expired": expired.len(),
        "details": details,
    }))
}