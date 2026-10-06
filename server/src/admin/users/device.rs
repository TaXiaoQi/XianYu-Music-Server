use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::*;
use crate::handlers::helpers::{int_of, parse_body, str_of};


pub async fn list_banned_devices(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 20 } else { ps.clamp(1, 100) }
    };
    let offset = (page - 1) * page_size;
    let keyword = str_of(&data, "keyword").trim().to_string();

    let base_sql = "
        SELECT 
            b.*,
            a.device_model,
            a.os_version,
            a.app_version,
            a.ciyuanxi_id,
            u.nickname
        FROM banned_devices b
        LEFT JOIN app_open_log a ON a.id = (
            SELECT o.id FROM app_open_log o
            WHERE o.device_id = b.device_id
            ORDER BY o.created_at DESC, o.id DESC LIMIT 1
        )
        LEFT JOIN app_users u ON a.ciyuanxi_id = u.ciyuanxi_id
    ";

    let (total, rows) = if keyword.is_empty() {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM banned_devices")
            .fetch_one(pool).await.unwrap_or(0);
        let rows = sqlx::query(&format!("{} ORDER BY b.created_at DESC LIMIT ? OFFSET ?", base_sql))
            .bind(page_size).bind(offset).fetch_all(pool).await;
        (total, rows)
    } else {
        let pat = format!("%{}%", keyword);
        let where_clause = "WHERE b.device_id LIKE ? OR b.reason LIKE ? OR a.ciyuanxi_id LIKE ? OR u.nickname LIKE ?";
        let count_sql = format!(
            "SELECT COUNT(*) FROM banned_devices b
             LEFT JOIN app_open_log a ON a.id = (
                 SELECT o.id FROM app_open_log o
                 WHERE o.device_id = b.device_id
                 ORDER BY o.created_at DESC, o.id DESC LIMIT 1
             )
             LEFT JOIN app_users u ON a.ciyuanxi_id = u.ciyuanxi_id
             {}", where_clause);
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(&pat).bind(&pat).bind(&pat).bind(&pat)
            .fetch_one(pool).await.unwrap_or(0);
        let rows = sqlx::query(&format!("{} {} ORDER BY b.created_at DESC LIMIT ? OFFSET ?", base_sql, where_clause))
            .bind(&pat).bind(&pat).bind(&pat).bind(&pat).bind(page_size).bind(offset).fetch_all(pool).await;
        (total, rows)
    };

    match rows {
        Ok(rows) => {
            let mut list: Vec<Value> = rows.iter().map(row_to_value).collect();
            list = super::mask_sensitive(&_ctx.role, list);
            let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
            ok("ok", json!({ "total": total, "page": page, "page_size": page_size, "total_pages": total_pages, "list": list }))
        }
        Err(e) => { tracing::error!("查询失败: {e}"); err(500, "查询失败") },
    }
}

fn normalize_platform(platform: &str, os_version: &str) -> &'static str {
    match platform {
        "desktop" | "windows" => "desktop",
        "watch" => "watch",
        "mobile" | "android" | "ios" => "mobile",
        "" => {
            if os_version.to_lowercase().contains("windows") {
                "desktop"
            } else {
                "mobile"
            }
        }
        _ => "mobile",
    }
}

const PLATFORM_CASE_SQL: &str = "CASE \
    WHEN a.platform IN ('desktop', 'windows') THEN 'desktop' \
    WHEN a.platform = 'watch' THEN 'watch' \
    WHEN a.platform IN ('mobile', 'android', 'ios') THEN 'mobile' \
    WHEN (a.platform IS NULL OR a.platform = '') AND a.os_version LIKE '%Windows%' THEN 'desktop' \
    ELSE 'mobile' END";

pub async fn list_all_devices(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 20 } else { ps.clamp(1, 100) }
    };
    let offset = (page - 1) * page_size;
    let keyword = str_of(&data, "keyword").trim().to_string();
    let platform_filter = match str_of(&data, "platform").trim() {
        "desktop" => Some("desktop".to_string()),
        "mobile" => Some("mobile".to_string()),
        "watch" => Some("watch".to_string()),
        _ => None,
    };

    let base_sql = format!("
        SELECT
            a.device_id,
            a.device_name,
            a.device_brand,
            a.device_model,
            a.os_version,
            a.app_version,
            a.ciyuanxi_id,
            a.ip,
            a.created_at,
            {} AS platform,
            COALESCE(
                (SELECT u2.nickname FROM app_users u2 WHERE u2.last_device_id = a.device_id ORDER BY u2.id DESC LIMIT 1),
                u.nickname
            ) AS nickname,
            COALESCE(
                (SELECT u2.ciyuanxi_id FROM app_users u2 WHERE u2.last_device_id = a.device_id ORDER BY u2.id DESC LIMIT 1),
                a.ciyuanxi_id
            ) AS ciyuanxi_id,
            b.id AS ban_id,
            b.reason AS ban_reason,
            (SELECT COUNT(*) FROM (
                SELECT ciyuanxi_id FROM app_open_log WHERE device_id = a.device_id AND ciyuanxi_id != ''
                UNION
                SELECT ciyuanxi_id FROM app_users WHERE last_device_id = a.device_id AND ciyuanxi_id != ''
            ) t) AS account_count,
            (SELECT COUNT(*) FROM app_users WHERE last_device_id = a.device_id) AS current_account_count
        FROM app_open_log a
        INNER JOIN (
            SELECT device_id, MAX(id) AS max_id FROM app_open_log GROUP BY device_id
        ) latest ON a.id = latest.max_id
        LEFT JOIN app_users u ON a.ciyuanxi_id = u.ciyuanxi_id
        LEFT JOIN banned_devices b ON b.device_id = a.device_id
    ", PLATFORM_CASE_SQL);

    let counts_sql = format!(
        "SELECT {} AS plat, COUNT(*) AS c FROM app_open_log a \
         INNER JOIN (SELECT device_id, MAX(id) AS max_id FROM app_open_log GROUP BY device_id) latest \
         ON a.id = latest.max_id GROUP BY plat",
        PLATFORM_CASE_SQL
    );
    let mut platform_counts = serde_json::Map::new();
    platform_counts.insert("all".into(), json!(0));
    platform_counts.insert("desktop".into(), json!(0));
    platform_counts.insert("mobile".into(), json!(0));
    platform_counts.insert("watch".into(), json!(0));
    if let Ok(rows) = sqlx::query(&counts_sql).fetch_all(pool).await {
        for r in rows.iter() {
            use sqlx::Row;
            let plat: String = r.try_get("plat").unwrap_or_default();
            let c: i64 = r.try_get("c").unwrap_or(0);
            platform_counts.insert(plat, json!(c));
        }
        let sum: i64 = ["desktop", "mobile", "watch"]
            .iter()
            .filter_map(|k| platform_counts.get(*k).and_then(|v| v.as_i64()))
            .sum();
        platform_counts.insert("all".into(), json!(sum));
    }
    let platform_counts = Value::Object(platform_counts);

    let (total, rows) = if keyword.is_empty() {
        let latest_join = "FROM app_open_log a INNER JOIN (SELECT device_id, MAX(id) AS max_id FROM app_open_log GROUP BY device_id) latest ON a.id = latest.max_id";
        let total: i64 = match &platform_filter {
            Some(_) => {
                let sql = format!("SELECT COUNT(*) {} WHERE {} = ?", latest_join, PLATFORM_CASE_SQL);
                sqlx::query_scalar(&sql).bind(platform_filter.as_deref().unwrap_or(""))
                    .fetch_one(pool).await.unwrap_or(0)
            }
            None => sqlx::query_scalar("SELECT COUNT(DISTINCT device_id) FROM app_open_log")
                .fetch_one(pool).await.unwrap_or(0),
        };
        let list_sql = match &platform_filter {
            Some(_) => format!("{} WHERE {} = ? ORDER BY a.created_at DESC LIMIT ? OFFSET ?", base_sql, PLATFORM_CASE_SQL),
            None => format!("{} ORDER BY a.created_at DESC LIMIT ? OFFSET ?", base_sql),
        };
        let mut q = sqlx::query(&list_sql);
        if let Some(p) = &platform_filter { q = q.bind(p); }
        let rows = q.bind(page_size).bind(offset).fetch_all(pool).await;
        (total, rows)
    } else {
        let pat = format!("%{}%", keyword);
        let platform_clause = if platform_filter.is_some() {
            format!(" AND {} = ?", PLATFORM_CASE_SQL)
        } else {
            String::new()
        };
        let where_clause = format!(
            "WHERE a.device_id LIKE ? OR a.device_name LIKE ? OR a.device_model LIKE ? OR a.ciyuanxi_id LIKE ? OR u.nickname LIKE ? OR (SELECT u2.nickname FROM app_users u2 WHERE u2.last_device_id = a.device_id ORDER BY u2.id DESC LIMIT 1) LIKE ?{}",
            platform_clause
        );
        let count_sql = format!(
            "SELECT COUNT(*) FROM (
                SELECT 1 FROM app_open_log a
                INNER JOIN (
                    SELECT device_id, MAX(id) AS max_id FROM app_open_log GROUP BY device_id
                ) latest ON a.id = latest.max_id
                LEFT JOIN app_users u ON a.ciyuanxi_id = u.ciyuanxi_id
                {}
            ) t", where_clause);
        let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
        cq = cq.bind(&pat).bind(&pat).bind(&pat).bind(&pat).bind(&pat).bind(&pat);
        if let Some(p) = &platform_filter { cq = cq.bind(p); }
        let total: i64 = cq.fetch_one(pool).await.unwrap_or(0);
        let list_sql = format!("{} {} ORDER BY a.created_at DESC LIMIT ? OFFSET ?", base_sql, where_clause);
        let mut q = sqlx::query(&list_sql);
        q = q.bind(&pat).bind(&pat).bind(&pat).bind(&pat).bind(&pat).bind(&pat);
        if let Some(p) = &platform_filter { q = q.bind(p); }
        let rows = q.bind(page_size).bind(offset).fetch_all(pool).await;
        (total, rows)
    };

    match rows {
        Ok(rows) => {
            let mut list: Vec<Value> = rows.iter().map(row_to_value).collect();
            list = super::mask_sensitive(&_ctx.role, list);
            let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
            ok("ok", json!({ "total": total, "page": page, "page_size": page_size, "total_pages": total_pages, "platform_counts": platform_counts, "list": list }))
        }
        Err(e) => { tracing::error!("查询失败: {e}"); err(500, "查询失败") },
    }
}

pub async fn ban_device(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    let reason = str_of(&data, "reason").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }
    if reason.is_empty() {
        return err(400, "封禁原因不能为空");
    }
    let result = sqlx::query("INSERT IGNORE INTO banned_devices (device_id, reason, banned_by) VALUES (?, ?, ?)")
        .bind(&device_id)
        .bind(&reason)
        .bind(&ctx.username)
        .execute(pool)
        .await;
    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return ok("该设备已在封禁列表中", Value::Null);
            }
            log_operation(pool, ctx, "封禁设备", &format!("设备ID:{}", device_id), &format!("原因:{} 操作人:{}", reason, ctx.username)).await;
            ok("已封禁", Value::Null)
        }
        Err(e) => { tracing::error!("操作失败: {e}"); err(500, "操作失败") },
    }
}

pub async fn unban_device(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    let id = int_of(&data, "id");
    if device_id.is_empty() && id <= 0 {
        return err(400, "需要提供设备ID或记录ID");
    }
    let result = if id > 0 {
        sqlx::query("DELETE FROM banned_devices WHERE id = ?").bind(id).execute(pool).await
    } else {
        sqlx::query("DELETE FROM banned_devices WHERE device_id = ?").bind(&device_id).execute(pool).await
    };
    match result {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return ok("该设备不在封禁列表中", Value::Null);
            }
            log_operation(pool, ctx, "解封设备", &format!("设备ID:{} ID:{}", device_id, id), &format!("操作人:{}", ctx.username)).await;
            ok("已解封", Value::Null)
        }
        Err(e) => { tracing::error!("操作失败: {e}"); err(500, "操作失败") },
    }
}

pub async fn get_user_devices(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let user_id = int_of(&data, "user_id");
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if user_id <= 0 && ciyuanxi_id.is_empty() {
        return err(400, "需要提供用户ID或弦予号");
    }

    let user_row = if user_id > 0 {
        sqlx::query("SELECT ciyuanxi_id, nickname, last_device_id FROM app_users WHERE id = ? LIMIT 1")
            .bind(user_id).fetch_optional(pool).await
    } else {
        sqlx::query("SELECT ciyuanxi_id, nickname, last_device_id FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
            .bind(&ciyuanxi_id).fetch_optional(pool).await
    };

    let user_row = match user_row {
        Ok(Some(r)) => r,
        _ => return err(404, "用户不存在"),
    };

    let username: String = user_row.get("nickname");
    let user_ciyuanxi_id: String = user_row.get("ciyuanxi_id");
    let last_device_id: String = user_row.get("last_device_id");

    let login_logs = if !last_device_id.is_empty() {
        sqlx::query("SELECT device_id, ip, created_at FROM admin_app_login_log WHERE device_id = ? ORDER BY created_at DESC LIMIT 20")
            .bind(&last_device_id).fetch_all(pool).await
    } else {
        Ok(vec![])
    };

    let open_logs = if !last_device_id.is_empty() {
        sqlx::query("SELECT device_id, ip, app_version, created_at FROM app_open_log WHERE device_id = ? ORDER BY created_at DESC LIMIT 20")
            .bind(&last_device_id).fetch_all(pool).await
    } else {
        Ok(vec![])
    };

    let is_banned = if !last_device_id.is_empty() {
        sqlx::query("SELECT id FROM banned_devices WHERE device_id = ? LIMIT 1")
            .bind(&last_device_id).fetch_optional(pool).await
            .ok().flatten().is_some()
    } else {
        false
    };

    let login_list: Vec<Value> = login_logs.unwrap_or_default().iter().map(|r| json!({
        "device_id": r.try_get::<String, _>("device_id").unwrap_or_default(),
        "ip": r.try_get::<String, _>("ip").unwrap_or_default(),
        "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
    })).collect();

    let open_list: Vec<Value> = open_logs.unwrap_or_default().iter().map(|r| json!({
        "device_id": r.try_get::<String, _>("device_id").unwrap_or_default(),
        "ip": r.try_get::<String, _>("ip").unwrap_or_default(),
        "app_version": r.try_get::<String, _>("app_version").unwrap_or_default(),
        "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
    })).collect();

    let mut devices: Vec<Value> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let device_rows = sqlx::query(
        "SELECT a.device_id, a.device_name, a.device_brand, a.device_model, a.os_version, a.app_version, a.platform, a.created_at, \
                (b.id IS NOT NULL) AS is_banned \
         FROM app_open_log a \
         INNER JOIN (SELECT device_id, MAX(id) AS max_id FROM app_open_log WHERE ciyuanxi_id = ? GROUP BY device_id) l \
             ON a.id = l.max_id \
         LEFT JOIN banned_devices b ON b.device_id = a.device_id \
         ORDER BY a.created_at DESC",
    )
    .bind(&user_ciyuanxi_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for r in device_rows.iter() {
        let device_id: String = r.try_get("device_id").unwrap_or_default();
        if device_id.is_empty() || !seen.insert(device_id.clone()) {
            continue;
        }
        let platform_raw: String = r.try_get("platform").unwrap_or_default();
        let os_version: String = r.try_get("os_version").unwrap_or_default();
        let platform = normalize_platform(&platform_raw, &os_version);
        devices.push(json!({
            "device_id": device_id,
            "device_name": r.try_get::<String, _>("device_name").unwrap_or_default(),
            "device_brand": r.try_get::<String, _>("device_brand").unwrap_or_default(),
            "device_model": r.try_get::<String, _>("device_model").unwrap_or_default(),
            "os_version": os_version,
            "app_version": r.try_get::<String, _>("app_version").unwrap_or_default(),
            "platform": platform,
            "is_banned": r.try_get::<bool, _>("is_banned").unwrap_or(false),
            "is_last": false,
            "last_active": r.try_get::<String, _>("created_at").unwrap_or_default(),
        }));
    }
    if !last_device_id.is_empty() && seen.insert(last_device_id.clone()) {
        let extra = sqlx::query(
            "SELECT a.device_id, a.device_name, a.device_brand, a.device_model, a.os_version, a.app_version, a.platform, a.created_at, \
                    (b.id IS NOT NULL) AS is_banned \
             FROM app_open_log a \
             LEFT JOIN banned_devices b ON b.device_id = a.device_id \
             WHERE a.device_id = ? ORDER BY a.id DESC LIMIT 1",
        )
        .bind(&last_device_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
        if let Some(r) = extra {
            let platform_raw: String = r.try_get("platform").unwrap_or_default();
            let os_version: String = r.try_get("os_version").unwrap_or_default();
            let platform = normalize_platform(&platform_raw, &os_version);
            devices.push(json!({
                "device_id": last_device_id,
                "device_name": r.try_get::<String, _>("device_name").unwrap_or_default(),
                "device_brand": r.try_get::<String, _>("device_brand").unwrap_or_default(),
                "device_model": r.try_get::<String, _>("device_model").unwrap_or_default(),
                "os_version": os_version,
                "app_version": r.try_get::<String, _>("app_version").unwrap_or_default(),
                "platform": platform,
                "is_banned": r.try_get::<bool, _>("is_banned").unwrap_or(false),
                "is_last": true,
                "last_active": r.try_get::<String, _>("created_at").unwrap_or_default(),
            }));
        } else {
            devices.push(json!({
                "device_id": last_device_id,
                "device_model": "",
                "os_version": "",
                "app_version": "",
                "platform": "",
                "is_banned": false,
                "is_last": true,
                "last_active": "",
            }));
        }
    }
    devices.sort_by_key(|d| d.get("is_last").and_then(|v| v.as_bool()).unwrap_or(false) == false);

    ok("ok", json!({
        "nickname": username,
        "ciyuanxi_id": user_ciyuanxi_id,
        "last_device_id": last_device_id,
        "is_banned": is_banned,
        "devices": devices,
        "login_logs": login_list,
        "open_logs": open_list,
    }))
}

pub async fn get_device_detail(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }

    let latest = sqlx::query(
        "SELECT device_id, device_model, os_version, app_version, ip, ciyuanxi_id, created_at
         FROM app_open_log WHERE device_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(&device_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let ban_row = sqlx::query("SELECT id, reason, banned_by, created_at FROM banned_devices WHERE device_id = ? LIMIT 1")
        .bind(&device_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let is_banned = ban_row.is_some();
    let ban_info = ban_row.map(|r| json!({
        "id": r.try_get::<i64, _>("id").unwrap_or_default(),
        "reason": r.try_get::<String, _>("reason").unwrap_or_default(),
        "banned_by": r.try_get::<String, _>("banned_by").unwrap_or_default(),
        "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
    })).unwrap_or(Value::Null);

    let account_rows = sqlx::query(
        "SELECT DISTINCT ciyuanxi_id FROM (
            SELECT a.ciyuanxi_id FROM app_open_log a WHERE a.device_id = ? AND a.ciyuanxi_id != ''
            UNION
            SELECT u.ciyuanxi_id FROM app_users u WHERE u.last_device_id = ? AND u.ciyuanxi_id != ''
        ) t",
    )
    .bind(&device_id)
    .bind(&device_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let current_account = sqlx::query(
        "SELECT id, ciyuanxi_id, nickname, listen_duration, unique_songs_count, avatar_url
         FROM app_users WHERE last_device_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(&device_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let current_ciyuanxi_id: String = current_account
        .as_ref()
        .and_then(|r| r.try_get::<String, _>("ciyuanxi_id").ok())
        .unwrap_or_default();

    let mut accounts: Vec<Value> = Vec::new();
    for row in &account_rows {
        let cid: String = row.try_get("ciyuanxi_id").unwrap_or_default();
        if cid.is_empty() {
            continue;
        }
        let user_info = sqlx::query(
            "SELECT id, nickname, listen_duration, unique_songs_count, avatar_url, last_device_id
             FROM app_users WHERE ciyuanxi_id = ? LIMIT 1",
        )
        .bind(&cid)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
        if let Some(u) = user_info {
            let is_current = u.try_get::<String, _>("last_device_id").unwrap_or_default() == device_id;
            let avatar_raw = u.try_get::<Option<String>, _>("avatar_url").unwrap_or_default().unwrap_or_default();
            accounts.push(json!({
                "ciyuanxi_id": cid,
                "nickname": u.try_get::<String, _>("nickname").unwrap_or_default(),
                "listen_duration": u.try_get::<u32, _>("listen_duration").unwrap_or_default(),
                "unique_songs_count": u.try_get::<u32, _>("unique_songs_count").unwrap_or_default(),
                "avatar_url": crate::handlers::upload::absolutize_media_url_with(&ctx.base_url, &ctx.config.public_base_url, &avatar_raw),
                "is_current": is_current,
            }));
        }
    }

    let has_current = accounts.iter().any(|a| a["ciyuanxi_id"].as_str() == Some(&current_ciyuanxi_id));
    if !has_current && !current_ciyuanxi_id.is_empty() {
        if let Some(u) = &current_account {
            let avatar_raw = u.try_get::<Option<String>, _>("avatar_url").unwrap_or_default().unwrap_or_default();
            accounts.insert(0, json!({
                "ciyuanxi_id": current_ciyuanxi_id,
                "nickname": u.try_get::<String, _>("nickname").unwrap_or_default(),
                "listen_duration": u.try_get::<u32, _>("listen_duration").unwrap_or_default(),
                "unique_songs_count": u.try_get::<u32, _>("unique_songs_count").unwrap_or_default(),
                "avatar_url": crate::handlers::upload::absolutize_media_url_with(&ctx.base_url, &ctx.config.public_base_url, &avatar_raw),
                "is_current": true,
            }));
        }
    }

    let device_info = latest.map(|r| json!({
        "device_model": r.try_get::<String, _>("device_model").unwrap_or_default(),
        "os_version": r.try_get::<String, _>("os_version").unwrap_or_default(),
        "app_version": r.try_get::<String, _>("app_version").unwrap_or_default(),
        "ip": r.try_get::<String, _>("ip").unwrap_or_default(),
        "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
    })).unwrap_or(Value::Null);

    ok("ok", json!({
        "device_id": device_id,
        "device_info": device_info,
        "is_banned": is_banned,
        "ban_info": ban_info,
        "associated_accounts": accounts,
        "account_count": accounts.len(),
        "current_ciyuanxi_id": current_ciyuanxi_id,
    }))
}

pub async fn reset_device_listen_stats(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }

    let rows = sqlx::query(
        "SELECT DISTINCT ciyuanxi_id FROM app_open_log WHERE device_id = ? AND ciyuanxi_id != ''",
    )
    .bind(&device_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let current = sqlx::query("SELECT ciyuanxi_id FROM app_users WHERE last_device_id = ?")
        .bind(&device_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    let mut ciyuanxi_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in &rows {
        let cid: String = r.try_get("ciyuanxi_id").unwrap_or_default();
        if !cid.is_empty() {
            ciyuanxi_ids.insert(cid);
        }
    }
    for r in &current {
        let cid: String = r.try_get("ciyuanxi_id").unwrap_or_default();
        if !cid.is_empty() {
            ciyuanxi_ids.insert(cid);
        }
    }

    if ciyuanxi_ids.is_empty() {
        return ok("该设备未关联任何账号，无需重置", Value::Null);
    }

    let mut reset_count = 0u32;
    for cid in &ciyuanxi_ids {
        let _ = sqlx::query(
            "UPDATE app_users SET listen_duration = 0, unique_songs_count = 0,
             listen_stats_reset_at = NOW(), listen_duration_offset = 0, unique_songs_offset = 0
             WHERE ciyuanxi_id = ?",
        )
        .bind(cid)
        .execute(pool)
        .await;
        let _ = sqlx::query("DELETE FROM listen_daily_stats WHERE ciyuanxi_id = ?")
            .bind(cid)
            .execute(pool)
            .await;
        reset_count += 1;
    }

    log_operation(
        pool, ctx, "重置设备听歌统计",
        &format!("device_id={} 关联{}个账号", device_id, reset_count), "",
    ).await;

    ok(&format!("已重置 {} 个关联账号的听歌统计", reset_count), Value::Null)
}

pub async fn delete_device_record(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }

    let _ = sqlx::query("DELETE FROM app_open_log WHERE device_id = ?")
        .bind(&device_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM banned_devices WHERE device_id = ?")
        .bind(&device_id)
        .execute(pool)
        .await;

    log_operation(pool, ctx, "删除设备记录", &format!("device_id={}", device_id), "").await;

    ok("设备记录已删除", Value::Null)
}

pub async fn batch_delete_devices(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_ids: Vec<String> = data.get("device_ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    if device_ids.is_empty() {
        return err(400, "请选择要删除的设备");
    }

    let mut total_deleted = 0u64;
    for did in &device_ids {
        let r1 = sqlx::query("DELETE FROM app_open_log WHERE device_id = ?")
            .bind(did)
            .execute(pool)
            .await;
        let r2 = sqlx::query("DELETE FROM banned_devices WHERE device_id = ?")
            .bind(did)
            .execute(pool)
            .await;
        if r1.is_ok() || r2.is_ok() {
            total_deleted += 1;
        }
    }

    log_operation(
        pool, ctx, "批量删除设备记录",
        &format!("删除{}台设备", total_deleted), "",
    ).await;

    ok(&format!("已删除 {} 台设备的记录", total_deleted), Value::Null)
}

pub async fn batch_ban_devices(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_ids: Vec<String> = data.get("device_ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let reason = str_of(&data, "reason").trim().to_string();

    if device_ids.is_empty() {
        return err(400, "请选择要封禁的设备");
    }
    if reason.is_empty() {
        return err(400, "封禁原因不能为空");
    }

    let mut total = 0u64;
    for did in &device_ids {
        let r = sqlx::query("INSERT IGNORE INTO banned_devices (device_id, reason, banned_by) VALUES (?, ?, ?)")
            .bind(did)
            .bind(&reason)
            .bind(&ctx.username)
            .execute(pool)
            .await;
        if r.map(|res| res.rows_affected() > 0).unwrap_or(false) {
            total += 1;
        }
    }

    log_operation(pool, ctx, "批量封禁设备", &format!("封禁{}台设备", total), &format!("原因:{}", reason)).await;
    ok(&format!("已封禁 {} 台设备", total), Value::Null)
}

pub async fn get_device_plugins(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }

    let row = sqlx::query("SELECT ciyuanxi_id, nickname FROM app_users WHERE last_device_id = ? ORDER BY id DESC LIMIT 1")
        .bind(&device_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    let Some(row) = row else {
        return ok("ok", json!({ "device_id": device_id, "nickname": null, "plugins": [], "plugin_count": 0, "uploaded_at": Value::Null, "message": "该设备未关联账号" }));
    };

    let ciyuanxi_id: String = row.get("ciyuanxi_id");
    let nickname: String = row.get("nickname");

    if ciyuanxi_id.is_empty() {
        return ok("ok", json!({ "device_id": device_id, "nickname": nickname, "plugins": [], "plugin_count": 0, "uploaded_at": Value::Null }));
    }

    // 插件快照已迁入 user_sync_files 表，经 read_snapshot 读取（含旧文件懒迁移）
    let save_data = match crate::handlers::sync_store::read_snapshot(pool, &ciyuanxi_id, "plugins.json").await {
        Ok(v) => v,
        Err(_) => {
            return ok("ok", json!({ "device_id": device_id, "nickname": nickname, "ciyuanxi_id": ciyuanxi_id, "plugins": [], "plugin_count": 0, "uploaded_at": Value::Null }));
        }
    };
    let mut plugins: Vec<Value> = Vec::new();
    let uploaded_at = save_data.get("uploaded_at").cloned().unwrap_or(Value::Null);
    if let Some(list) = save_data.get("plugins").and_then(|x| x.as_array()) {
        for p in list {
            let script_size = p.get("script").and_then(|s| s.as_str()).map(|s| s.len()).unwrap_or(0);
            plugins.push(json!({
                "name": p.get("name").and_then(|x| x.as_str()).unwrap_or("(未知)"),
                "format": p.get("format").and_then(|x| x.as_str()).unwrap_or("unknown"),
                "version": p.get("version").and_then(|x| x.as_str()).unwrap_or(""),
                "author": p.get("author").and_then(|x| x.as_str()).unwrap_or(""),
                "description": p.get("description").and_then(|x| x.as_str()).unwrap_or(""),
                "enabled": p.get("enabled").and_then(|x| x.as_bool()).unwrap_or(false),
                "filePath": p.get("filePath").and_then(|x| x.as_str()).unwrap_or(""),
                "importedAt": p.get("importedAt").and_then(|x| x.as_i64()).unwrap_or(0),
                "scriptSize": script_size,
            }));
        }
    }
    let plugin_count = plugins.len();
    ok("ok", json!({
        "device_id": device_id,
        "nickname": nickname,
        "ciyuanxi_id": ciyuanxi_id,
        "plugins": plugins,
        "plugin_count": plugin_count,
        "uploaded_at": uploaded_at,
    }))
}

