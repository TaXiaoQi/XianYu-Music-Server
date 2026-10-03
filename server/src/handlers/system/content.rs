use super::*;

fn announcements_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("announcement.json")
}

fn privacy_policy_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("privacy_policy.json")
}

fn read_announcements() -> Vec<serde_json::Value> {
    let path = announcements_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(arr) = v.as_array() {
                return arr.clone();
            }
        }
    }
    Vec::new()
}

pub async fn get_fallback_modules(ctx: ReqCtx) -> Response {
    let modules = crate::admin::fallback::enabled_modules_payload();
    ctx.json(200, "ok", Some(json!({ "modules": modules })))
}

fn announcement_version(item: &serde_json::Value) -> String {
    item.get("updated_at")
        .or_else(|| item.get("updatedAt"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

async fn announcement_confirmed(pool: &MySqlPool, ciyuanxi_id: &str, device_id: &str, announcement_id: &str, version: &str) -> bool {
    if ciyuanxi_id.is_empty() && device_id.is_empty() {
        return false;
    }
    sqlx::query(
        "SELECT id FROM user_announcement_confirmations
         WHERE announcement_id = ? AND announcement_updated_at = ?
           AND ((? <> '' AND ciyuanxi_id = ?) OR (? <> '' AND device_id = ?))
         LIMIT 1",
    )
    .bind(announcement_id)
    .bind(version)
    .bind(ciyuanxi_id)
    .bind(ciyuanxi_id)
    .bind(device_id)
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .is_some()
}

pub async fn get_announcement(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let device_id = str_of(&data, "device_id").trim().to_string();
    let platform = {
        let p = str_of(&data, "platform").trim().to_string();
        if p.is_empty() { "desktop".to_string() } else { p }
    };
    let mut list: Vec<serde_json::Value> = read_announcements()
        .into_iter()
        .filter(|item| item.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false))
        .filter(|item| {
            let p = item.get("platform").and_then(|v| v.as_str()).unwrap_or("desktop");
            p == platform
        })
        .collect();
    list.sort_by(|a, b| {
        let ta = a.get("updated_at")
            .or_else(|| a.get("created_at"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tb = b.get("updated_at")
            .or_else(|| b.get("created_at"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        tb.cmp(ta)
    });

    let Some(item) = list.into_iter().next() else {
        return ctx.json::<serde_json::Value>(200, "ok", None);
    };

    let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let updated_at = announcement_version(&item);
    if id.is_empty() || title.is_empty() || content.is_empty() {
        return ctx.json::<serde_json::Value>(200, "ok", None);
    }
    if announcement_confirmed(pool, &ciyuanxi_id, &device_id, &id, &updated_at).await {
        return ctx.json::<serde_json::Value>(200, "ok", None);
    }

    ctx.json(
        200,
        "ok",
        Some(json!({
            "id": id,
            "title": title,
            "content": content,
            "type": item.get("type").and_then(|v| v.as_str()).unwrap_or("info"),
            "date": item.get("date").and_then(|v| v.as_str()).unwrap_or(""),
            "actionUrl": item.get("actionUrl").and_then(|v| v.as_str()).unwrap_or(""),
            "actionText": item.get("actionText").and_then(|v| v.as_str()).unwrap_or(""),
            "updatedAt": updated_at,
        })),
    )
}

pub async fn confirm_announcement(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let announcement_id = str_of(&data, "announcement_id").trim().to_string();
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let device_id = str_of(&data, "device_id").trim().to_string();
    let client_updated_at = str_of(&data, "announcement_updated_at").trim().to_string();

    if announcement_id.is_empty() {
        return ctx.err(400, "公告 ID 不能为空");
    }
    if ciyuanxi_id.is_empty() && device_id.is_empty() {
        return ctx.err(400, "缺少用户或设备标识");
    }

    let list = read_announcements();
    let Some(item) = list
        .iter()
        .find(|item| item.get("id").and_then(|v| v.as_str()).unwrap_or("") == announcement_id)
    else {
        return ctx.err(404, "公告不存在");
    };

    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let updated_at = announcement_version(item);
    if !client_updated_at.is_empty() && client_updated_at != updated_at {
        return ctx.err(409, "公告已更新，请重新阅读");
    }

    let result = sqlx::query(
        "INSERT INTO user_announcement_confirmations
         (ciyuanxi_id, device_id, announcement_id, announcement_title, announcement_updated_at, ip)
         VALUES (?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE confirmed_at = CURRENT_TIMESTAMP, announcement_title = VALUES(announcement_title), ip = VALUES(ip)",
    )
    .bind(&ciyuanxi_id)
    .bind(&device_id)
    .bind(&announcement_id)
    .bind(&title)
    .bind(&updated_at)
    .bind(&ctx.client_ip)
    .execute(pool)
    .await;

    match result {
        Ok(_) => ctx.ok("确认成功", json!({ "announcement_id": announcement_id, "updatedAt": updated_at })),
        Err(e) => { tracing::error!("记录公告确认失败: {e}"); ctx.err(500, "记录公告确认失败") },
    }
}

/// 获取服务器下发的隐私政策（api/privacy_policy.json）。
/// enabled=false 或文件缺失时返回 None，客户端使用内置默认版本。
/// 是否已确认由客户端本地 fingerprint（id + updatedAt）判断，同公告机制。
pub async fn get_privacy_policy(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let _ = (body, pool);
    let path = privacy_policy_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return ctx.json::<serde_json::Value>(200, "ok", None);
    };
    let Ok(item) = serde_json::from_str::<serde_json::Value>(&content) else {
        return ctx.json::<serde_json::Value>(200, "ok", None);
    };
    let enabled = item.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let policy = item.get("content").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let updated_at = item.get("updatedAt").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    if !enabled || id.is_empty() || policy.is_empty() || updated_at.is_empty() {
        return ctx.json::<serde_json::Value>(200, "ok", None);
    }
    ctx.json(
        200,
        "ok",
        Some(json!({
            "id": id,
            "content": policy,
            "updatedAt": updated_at,
        })),
    )
}

/// 记录隐私政策确认（复用 user_announcement_confirmations 表，
/// announcement_id 存政策 id）。仅作留存上报，弹窗与否由客户端判断。
pub async fn confirm_privacy_policy(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let policy_id = str_of(&data, "policy_id").trim().to_string();
    let policy_updated_at = str_of(&data, "policy_updated_at").trim().to_string();
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let device_id = str_of(&data, "device_id").trim().to_string();

    if policy_id.is_empty() {
        return ctx.err(400, "政策 ID 不能为空");
    }
    if ciyuanxi_id.is_empty() && device_id.is_empty() {
        return ctx.err(400, "缺少用户或设备标识");
    }

    let result = sqlx::query(
        "INSERT INTO user_announcement_confirmations
         (ciyuanxi_id, device_id, announcement_id, announcement_title, announcement_updated_at, ip)
         VALUES (?, ?, ?, 'privacy-policy', ?, ?)
         ON DUPLICATE KEY UPDATE confirmed_at = CURRENT_TIMESTAMP, announcement_updated_at = VALUES(announcement_updated_at), ip = VALUES(ip)",
    )
    .bind(&ciyuanxi_id)
    .bind(&device_id)
    .bind(&policy_id)
    .bind(&policy_updated_at)
    .bind(&ctx.client_ip)
    .execute(pool)
    .await;

    match result {
        Ok(_) => ctx.ok("确认成功", json!({ "policy_id": policy_id, "updatedAt": policy_updated_at })),
        Err(e) => { tracing::error!("记录隐私政策确认失败: {e}"); ctx.err(500, "记录隐私政策确认失败") },
    }
}

pub async fn get_site_logo(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let url = crate::admin::site_config::read_site_logo(pool).await;
    ctx.json(200, "ok", Some(json!({ "logo_url": url })))
}

pub async fn get_user_agreement(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let (title, content) = crate::admin::agreement::load_user_agreement(pool).await;
    ctx.json(200, "ok", Some(json!({ "title": title, "content": content })))
}

pub async fn get_deploy_doc(ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let (title, content) = crate::admin::deploy_doc::load_deploy_doc(pool).await;
    ctx.json(200, "ok", Some(json!({ "title": title, "content": content })))
}
