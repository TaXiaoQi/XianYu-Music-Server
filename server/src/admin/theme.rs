use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

fn themes_dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").join("themes")
}

async fn ensure_themes_table(pool: &MySqlPool) {
    if let Some(stmt) = crate::schema::table_statements().iter().find(|s| s.contains("`themes`")) {
        let _ = sqlx::query(stmt).execute(pool).await;
    }
}

fn full_url(base_url: &str, config_public_base_url: &str, url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }
    let base = if !base_url.is_empty() {
        base_url
    } else if !config_public_base_url.is_empty() {
        config_public_base_url
    } else {
        return url.to_string();
    };
    format!("{}{}", base.trim_end_matches('/'), url)
}

fn platform_label(platform: &str) -> &'static str {
    match platform {
        "mobile" => "移动端",
        _ => "桌面端",
    }
}

pub async fn list_themes(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let platform = str_of(&data, "platform").trim().to_string();
    ensure_themes_table(pool).await;
    let rows = if platform == "mobile" || platform == "desktop" {
        sqlx::query("SELECT * FROM themes WHERE platform = ? ORDER BY id DESC")
            .bind(&platform)
            .fetch_all(pool)
            .await
    } else {
        sqlx::query("SELECT * FROM themes ORDER BY id DESC").fetch_all(pool).await
    };
    match rows {
        Ok(rows) => {
            let arr: Vec<Value> = rows
                .iter()
                .map(|r| {
                    let mut v = crate::admin::row_to_value(r);
                    if let Some(obj) = v.as_object_mut() {
                        for key in ["preview_url", "thumbnail_url"] {
                            if let Some(url) = obj.get(key).and_then(|v| v.as_str()) {
                                obj.insert(key.to_string(), Value::String(full_url(&ctx.base_url, &ctx.config.public_base_url, url)));
                            }
                        }
                        // 管理端可直接查看主题包 JSON
                        if let Some(text) = obj.get("payload").and_then(|v| v.as_str()) {
                            obj.insert("payload_json".to_string(), serde_json::from_str::<Value>(text).unwrap_or(Value::Null));
                        }
                    }
                    v
                })
                .collect();
            log_operation(pool, ctx, "查看主题列表", "", "").await;
            ok("ok", json!(arr))
        }
        Err(_) => err(500, "数据库错误"),
    }
}

pub async fn change_theme_status(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let status = str_of(&data, "status");
    let status = if status.is_empty() { "normal".to_string() } else { status };
    if id <= 0 {
        return err(400, "无效的主题ID");
    }
    let valid = ["normal", "disabled", "pending", "rejected"];
    if !valid.contains(&status.as_str()) {
        return err(400, "无效的状态");
    }
    let theme = sqlx::query("SELECT name, platform FROM themes WHERE id = ?").bind(id).fetch_optional(pool).await.ok().flatten();
    if theme.is_none() {
        return err(404, "主题不存在");
    }
    ensure_themes_table(pool).await;
    if status == "normal" || status == "rejected" {
        let upd = sqlx::query("UPDATE themes SET status = ?, reviewed_at = NOW(), reviewed_by = ? WHERE id = ?")
            .bind(&status)
            .bind(&ctx.username)
            .bind(id)
            .execute(pool)
            .await;
        match upd {
            Ok(_) => {
                let label = if status == "normal" { "审核通过主题" } else { "拒绝主题" };
                log_operation(pool, ctx, label, &format!("ID:{}", id), &format!("审核人:{}", ctx.username)).await;
                ok("状态已更新", Value::Null)
            }
            Err(_) => err(500, "数据库错误"),
        }
    } else {
        let upd = sqlx::query("UPDATE themes SET status = ? WHERE id = ?")
            .bind(&status)
            .bind(id)
            .execute(pool)
            .await;
        match upd {
            Ok(_) => {
                log_operation(pool, ctx, "修改主题状态", &format!("ID:{}", id), &format!("状态:{}", status)).await;
                ok("状态已更新", Value::Null)
            }
            Err(_) => err(500, "数据库错误"),
        }
    }
}

pub async fn delete_theme(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "无效的主题ID");
    }
    let theme = sqlx::query("SELECT name, platform FROM themes WHERE id = ?").bind(id).fetch_optional(pool).await.ok().flatten();
    let Some(theme) = theme else {
        return err(404, "主题不存在");
    };
    let name: String = theme.try_get("name").unwrap_or_default();
    let platform: String = theme.try_get("platform").unwrap_or_default();
    let _ = std::fs::remove_dir_all(themes_dir().join(format!("theme_{}", id)));
    for f in ["preview_", "thumb_"] {
        let p = themes_dir().join(format!("{}{}.jpg", f, id));
        if p.is_file() {
            let _ = std::fs::remove_file(&p);
        }
    }
    let _ = sqlx::query("DELETE FROM themes WHERE id = ?").bind(id).execute(pool).await;
    log_operation(pool, ctx, &format!("删除{}主题", platform_label(&platform)), &name, &format!("ID:{}", id)).await;
    ok("删除成功", Value::Null)
}
