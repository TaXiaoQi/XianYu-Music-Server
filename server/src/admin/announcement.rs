use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

/// 公告 JSON 文件路径（相对 serve 根，兼容原 PHP api 目录）
fn announcements_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("announcement.json")
}

fn read_announcements() -> Vec<Value> {
    let path = announcements_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Value>(&content) {
            if let Some(arr) = v.as_array() {
                return arr.clone();
            }
        }
    }
    Vec::new()
}

fn write_announcements(list: &[Value]) -> std::io::Result<()> {
    let path = announcements_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let json = serde_json::to_string_pretty(list).unwrap_or_else(|_| "[]".to_string());
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)
}

fn now_ymd() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn now_ymd_hms() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn valid_type(t: &str) -> &str {
    if t == "warning" || t == "update" {
        t
    } else {
        "info"
    }
}

/// 获取公告列表（按创建时间倒序）
pub async fn list(_body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let mut list = read_announcements();
    list.sort_by(|a, b| {
        let ta = a.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
        let tb = b.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
        tb.cmp(ta)
    });
    ok("ok", Value::Array(list))
}

/// 新增公告
pub async fn add(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let title = str_of(&data, "title").trim().to_string();
    let content = str_of(&data, "content").trim().to_string();
    let mut typ = str_of(&data, "type");
    if typ.is_empty() {
        typ = "info".into();
    }
    let action_url = str_of(&data, "action_url").trim().to_string();
    let enabled = int_of(&data, "enabled") != 0;
    if title.is_empty() || content.is_empty() {
        return err(400, "标题和内容不能为空");
    }
    let typ = valid_type(&typ).to_string();
    let mut list = read_announcements();
    let max_id = list.iter().map(|a| a.get("id").and_then(|v| v.as_str()).and_then(|s| s.parse::<i64>().ok()).unwrap_or(0)).max().unwrap_or(0);
    let new_id = (max_id + 1).to_string();
    list.push(json!({
        "id": new_id, "title": title, "content": content, "type": typ,
        "date": now_ymd(), "actionUrl": action_url, "actionText": "",
        "enabled": enabled, "created_at": now_ymd_hms(), "updated_at": now_ymd_hms(),
    }));
    if write_announcements(&list).is_err() {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    log_operation(pool, ctx, "新增公告", &title, &format!("类型:{} 启用:{}", typ, if enabled { "是" } else { "否" })).await;
    ok("添加成功", Value::Null)
}

/// 编辑公告
pub async fn update(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = str_of(&data, "id").trim().to_string();
    let title = str_of(&data, "title").trim().to_string();
    let content = str_of(&data, "content").trim().to_string();
    let typ = str_of(&data, "type");
    let typ = if typ.is_empty() { "info".to_string() } else { valid_type(&typ).to_string() };
    let action_url = str_of(&data, "action_url").trim().to_string();
    if id.is_empty() || title.is_empty() || content.is_empty() {
        return err(400, "参数错误");
    }
    let mut list = read_announcements();
    let mut found = false;
    for a in list.iter_mut() {
        if a.get("id").and_then(|v| v.as_str()).unwrap_or("") == id {
            a["title"] = json!(title);
            a["content"] = json!(content);
            a["type"] = json!(typ);
            a["actionUrl"] = json!(action_url);
            a["updated_at"] = json!(now_ymd_hms());
            found = true;
            break;
        }
    }
    if !found {
        return err(404, "公告不存在");
    }
    if write_announcements(&list).is_err() {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    log_operation(pool, ctx, "编辑公告", &format!("ID:{}", id), &format!("标题:{}", title)).await;
    ok("修改成功", Value::Null)
}

/// 删除公告
pub async fn delete(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = str_of(&data, "id").trim().to_string();
    if id.is_empty() {
        return err(400, "参数错误");
    }
    let list = read_announcements();
    let mut new_list: Vec<Value> = Vec::new();
    let mut found_title = String::new();
    for a in &list {
        if a.get("id").and_then(|v| v.as_str()).unwrap_or("") == id {
            found_title = a.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            continue;
        }
        new_list.push(a.clone());
    }
    if new_list.len() == list.len() {
        return err(404, "公告不存在");
    }
    if write_announcements(&new_list).is_err() {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    log_operation(pool, ctx, "删除公告", &format!("ID:{}", id), &found_title).await;
    ok("删除成功", Value::Null)
}

/// 切换公告状态
pub async fn toggle(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = str_of(&data, "id").trim().to_string();
    let enabled = int_of(&data, "enabled") != 0;
    if id.is_empty() {
        return err(400, "参数错误");
    }
    let mut list = read_announcements();
    let mut found = false;
    for a in list.iter_mut() {
        if a.get("id").and_then(|v| v.as_str()).unwrap_or("") == id {
            a["enabled"] = json!(enabled);
            found = true;
            break;
        }
    }
    if !found {
        return err(404, "公告不存在");
    }
    if write_announcements(&list).is_err() {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    log_operation(pool, ctx, "切换公告状态", &format!("ID:{}", id), if enabled { "启用" } else { "禁用" }).await;
    ok("操作成功", Value::Null)
}