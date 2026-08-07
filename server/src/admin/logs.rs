use axum::response::Response;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};

use super::{err, log_operation, ok, row_to_value, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

/// 获取报错日志列表（分页 + 多条件筛选）
pub async fn list_error_logs(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 20 } else { ps.clamp(1, 100) }
    };
    let error_type = str_of(&data, "error_type").trim().to_string();
    let platform = str_of(&data, "platform").trim().to_string();
    let device_brand = str_of(&data, "device_brand").trim().to_string();
    let keyword = str_of(&data, "keyword").trim().to_string();
    let offset = (page - 1) * page_size;

    // 构建条件
    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<String> = Vec::new();
    if !error_type.is_empty() {
        conditions.push("error_type LIKE ?".to_string());
        binds.push(format!("%{}%", error_type));
    }
    if !platform.is_empty() {
        conditions.push("platform LIKE ?".to_string());
        binds.push(format!("%{}%", platform));
    }
    if !device_brand.is_empty() {
        conditions.push("device_brand LIKE ?".to_string());
        binds.push(format!("%{}%", device_brand));
    }
    if !keyword.is_empty() {
        conditions.push("(error_message LIKE ? OR error_stack LIKE ? OR device_model LIKE ?)".to_string());
        let pat = format!("%{}%", keyword);
        binds.push(pat.clone());
        binds.push(pat.clone());
        binds.push(pat);
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // 查询总数
    let count_sql = format!("SELECT COUNT(*) FROM error_log {}", where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        count_query = count_query.bind(b);
    }
    let total = count_query.fetch_one(pool).await.unwrap_or(0);

    // 查询列表
    let list_sql = format!(
        "SELECT * FROM error_log {} ORDER BY error_time DESC LIMIT ? OFFSET ?",
        where_clause
    );
    let mut list_query = sqlx::query(&list_sql);
    for b in &binds {
        list_query = list_query.bind(b);
    }
    list_query = list_query.bind(page_size).bind(offset);

    match list_query.fetch_all(pool).await {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_value).collect();
            let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
            ok("ok", json!({
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
                "list": list,
            }))
        }
        Err(e) => err(500, &format!("查询失败: {}", e)),
    }
}

/// 获取报错日志统计（按 error_type 分组 + 总计）
pub async fn get_error_stats(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    // 按类型分组统计
    let rows = sqlx::query("SELECT error_type, COUNT(*) as cnt FROM error_log GROUP BY error_type ORDER BY cnt DESC")
        .fetch_all(pool)
        .await;

    let stats: Vec<Value> = match rows {
        Ok(rows) => rows
            .iter()
            .map(|r| {
                let et: String = r.try_get("error_type").unwrap_or_default();
                let cnt: i64 = r.try_get("cnt").unwrap_or(0);
                json!({ "error_type": et, "count": cnt })
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM error_log")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    ok("ok", json!({ "stats": stats, "total": total }))
}

/// 获取单条崩溃日志详情
pub async fn get_error_detail(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    match sqlx::query("SELECT * FROM error_log WHERE id = ?").bind(id).fetch_optional(pool).await {
        Ok(Some(row)) => {
            log_operation(pool, ctx, "查看崩溃日志", &format!("ID:{}", id), "").await;
            ok("", crate::admin::row_to_value(&row))
        }
        Ok(None) => err(404, "记录不存在"),
        Err(e) => err(500, &format!("服务器错误: {}", e)),
    }
}

/// 删除单条崩溃日志
pub async fn delete_error(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    sqlx::query("DELETE FROM error_log WHERE id = ?").bind(id).execute(pool).await.unwrap_or_default();
    log_operation(pool, ctx, "删除崩溃日志", &format!("ID:{}", id), "").await;
    ok("删除成功", Value::Null)
}

/// 获取APP登录日志列表（分页 + 搜索 + 筛选 + 统计）
pub async fn list_app_login_log(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 20 } else { ps.clamp(1, 100) }
    };
    let keyword = str_of(&data, "keyword").trim().to_string();
    let status_filter = str_of(&data, "status_filter").trim().to_string();
    let offset = (page - 1) * page_size;

    // 构建条件
    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<String> = Vec::new();
    if !keyword.is_empty() {
        conditions.push("(admin_username LIKE ? OR ip LIKE ? OR device_id LIKE ? OR device_model LIKE ?)".to_string());
        let pat = format!("%{}%", keyword);
        binds.push(pat.clone());
        binds.push(pat.clone());
        binds.push(pat.clone());
        binds.push(pat);
    }
    if status_filter == "success" {
        conditions.push("status = 1".to_string());
    } else if status_filter == "failed" {
        conditions.push("status = 0".to_string());
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // 查询总数
    let count_sql = format!("SELECT COUNT(*) FROM admin_app_login_log {}", where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        count_query = count_query.bind(b);
    }
    let filtered_total = count_query.fetch_one(pool).await.unwrap_or(0);

    // 查询列表
    let list_sql = format!(
        "SELECT * FROM admin_app_login_log {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );
    let mut list_query = sqlx::query(&list_sql);
    for b in &binds {
        list_query = list_query.bind(b);
    }
    list_query = list_query.bind(page_size).bind(offset);

    let list: Vec<Value> = match list_query.fetch_all(pool).await {
        Ok(rows) => rows.iter().map(row_to_value).collect(),
        Err(e) => return err(500, &format!("查询失败: {}", e)),
    };

    // 统计数据
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_app_login_log")
        .fetch_one(pool).await.unwrap_or(0);
    let today_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_app_login_log WHERE DATE(created_at) = CURDATE()")
        .fetch_one(pool).await.unwrap_or(0);
    let today_success: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_app_login_log WHERE DATE(created_at) = CURDATE() AND status = 1")
        .fetch_one(pool).await.unwrap_or(0);
    let yesterday_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_app_login_log WHERE DATE(created_at) = DATE_SUB(CURDATE(), INTERVAL 1 DAY)")
        .fetch_one(pool).await.unwrap_or(0);
    let distinct_ips: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT ip) FROM admin_app_login_log")
        .fetch_one(pool).await.unwrap_or(0);
    let distinct_devices: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT device_id) FROM admin_app_login_log WHERE device_id != ''")
        .fetch_one(pool).await.unwrap_or(0);

    let today_trend = if yesterday_count > 0 {
        ((today_count - yesterday_count) as f64 / yesterday_count as f64 * 100.0).round() as i64
    } else if today_count > 0 {
        100
    } else {
        0
    };

    let total_pages = ((filtered_total as f64) / (page_size as f64)).ceil() as i64;
    ok("ok", json!({
        "total": total,
        "filtered_total": filtered_total,
        "page": page,
        "page_size": page_size,
        "total_pages": total_pages,
        "list": list,
        "stats": {
            "today_count": today_count,
            "today_success": today_success,
            "today_failed": today_count - today_success,
            "yesterday_count": yesterday_count,
            "today_trend": today_trend,
            "distinct_ips": distinct_ips,
            "distinct_devices": distinct_devices,
        }
    }))
}

/// 清空所有崩溃日志
pub async fn clear_all_errors(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let _ = body;
    sqlx::query("TRUNCATE TABLE error_log").execute(pool).await.unwrap_or_default();
    log_operation(pool, ctx, "清空崩溃日志", "", "").await;
    ok("已清空所有崩溃日志", Value::Null)
}

/// 后台操作日志列表（分页 + 搜索）
pub async fn list_operation_logs(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 20 } else { ps.clamp(1, 100) }
    };
    let keyword = str_of(&data, "keyword").trim().to_string();
    let offset = (page - 1) * page_size;

    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<String> = Vec::new();
    if !keyword.is_empty() {
        conditions.push("(admin_username LIKE ? OR action LIKE ? OR target LIKE ? OR ip LIKE ?)".to_string());
        let pat = format!("%{}%", keyword);
        binds.push(pat.clone());
        binds.push(pat.clone());
        binds.push(pat.clone());
        binds.push(pat);
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) FROM admin_operation_log {}", where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        count_query = count_query.bind(b);
    }
    let total = count_query.fetch_one(pool).await.unwrap_or(0);

    let list_sql = format!(
        "SELECT * FROM admin_operation_log {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );
    let mut list_query = sqlx::query(&list_sql);
    for b in &binds {
        list_query = list_query.bind(b);
    }
    list_query = list_query.bind(page_size).bind(offset);

    let list: Vec<Value> = match list_query.fetch_all(pool).await {
        Ok(rows) => rows.iter().map(row_to_value).collect(),
        Err(e) => return err(500, &format!("查询失败: {}", e)),
    };

    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
    ok("ok", json!({
        "total": total,
        "page": page,
        "page_size": page_size,
        "total_pages": total_pages,
        "list": list,
    }))
}

/// 后台登录日志列表（分页 + 搜索）
pub async fn list_admin_login_logs(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 20 } else { ps.clamp(1, 100) }
    };
    let keyword = str_of(&data, "keyword").trim().to_string();
    let status_filter = str_of(&data, "status_filter").trim().to_string();
    let offset = (page - 1) * page_size;

    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<String> = Vec::new();
    if !keyword.is_empty() {
        conditions.push("(admin_username LIKE ? OR ip LIKE ?)".to_string());
        let pat = format!("%{}%", keyword);
        binds.push(pat.clone());
        binds.push(pat);
    }
    if status_filter == "success" {
        conditions.push("status = 1".to_string());
    } else if status_filter == "failed" {
        conditions.push("status = 0".to_string());
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!("SELECT COUNT(*) FROM admin_login_log {}", where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        count_query = count_query.bind(b);
    }
    let total = count_query.fetch_one(pool).await.unwrap_or(0);

    let list_sql = format!(
        "SELECT * FROM admin_login_log {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_clause
    );
    let mut list_query = sqlx::query(&list_sql);
    for b in &binds {
        list_query = list_query.bind(b);
    }
    list_query = list_query.bind(page_size).bind(offset);

    let list: Vec<Value> = match list_query.fetch_all(pool).await {
        Ok(rows) => rows.iter().map(row_to_value).collect(),
        Err(e) => return err(500, &format!("查询失败: {}", e)),
    };

    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
    ok("ok", json!({
        "total": total,
        "page": page,
        "page_size": page_size,
        "total_pages": total_pages,
        "list": list,
    }))
}