use super::*;

pub async fn get_version_status(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let version = data.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if version.is_empty() {
        return ctx.err(400, "版本号不能为空");
    }

    let current_row = sqlx::query("SELECT * FROM app_versions WHERE version_code = ? ORDER BY id DESC LIMIT 1")
        .bind(&version)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    if let Some(row) = current_row {
        let status: String = row.get("status");
        if status == "disabled" || status == "crash" || status == "group_update" {
            let latest: String = row.get("version_code");
            let content: String = row.get("update_content");
            let url: String = row.get("download_url");
            let size: i64 = row.get("file_size");
            let message: String = row.get("message");
            let msg2: String = if !message.is_empty() { message } else { content.clone() };
            return ctx.json(
                200,
                "ok",
                Some(json!({
                    "status": status,
                    "latest_version": latest,
                    "update_content": content,
                    "download_url": url,
                    "file_size": size,
                    "message": msg2
                })),
            );
        }
    }

    match sqlx::query("SELECT * FROM app_versions WHERE status IN ('update', 'force_update') ORDER BY id DESC")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let mut latest_row: Option<sqlx::mysql::MySqlRow> = None;
            for row in rows {
                let row_version: String = row.get("version_code");
                let row_status: String = row.get("status");
                let cmp = compare_version_code(&row_version, &version);
                if cmp > 0 || (cmp == 0 && row_status == "force_update") {
                    latest_row = Some(row);
                    break;
                }
            }
            if let Some(row) = latest_row {
                let status: String = row.get("status");
                let latest: String = row.get("version_code");
                let content: String = row.get("update_content");
                let url: String = row.get("download_url");
                let size: i64 = row.get("file_size");
                let message: String = row.get("message");
                return ctx.json(
                    200,
                    "ok",
                    Some(json!({
                        "status": status,
                        "latest_version": latest,
                        "update_content": content,
                        "download_url": url,
                        "file_size": size,
                        "message": message
                    })),
                );
            }
            ctx.json(200, "ok", Some(json!({ "status": "normal" })))
        }
        Err(_) => ctx.json(200, "ok", Some(json!({ "status": "normal" }))),
    }
}

fn default_system(platform: &str) -> &'static str {
    match platform {
        "mobile" => "android",
        "watch" => "wearos",
        _ => "windows",
    }
}

fn arch_tier(item: &serde_json::Value, arch: &str) -> u8 {
    let raw = item.get("arch").and_then(|v| v.as_str()).unwrap_or("").trim();
    if raw.is_empty() {
        // 未标注架构的旧记录：与请求一致或请求未指定架构时视为完全匹配
        return if arch.is_empty() { 0 } else { 1 };
    }
    if raw == arch {
        0
    } else if arch.is_empty() {
        2
    } else {
        u8::MAX // 架构不匹配，直接淘汰
    }
}

fn item_system<'a>(item: &'a serde_json::Value, platform: &str) -> &'a str {
    let raw = item.get("system").and_then(|v| v.as_str()).unwrap_or("");
    if raw.is_empty() {
        default_system(platform)
    } else {
        raw
    }
}

fn system_label(platform: &str, system: &str) -> &'static str {
    match (platform, system) {
        ("mobile", "harmonyos") => "鸿蒙 HarmonyOS",
        ("mobile", "ios") => "iOS",
        ("mobile", _) => "Android",
        ("watch", "wearos") => "WearOS",
        ("watch", "ohos") => "鸿蒙 HarmonyOS",
        ("watch", "watchos") => "watchOS",
        (_, "linux") => "Linux",
        (_, "macos") => "macOS",
        _ => "Windows",
    }
}

fn latest_enabled_platform_version(platform: &str, channel: &str, system: &str, arch: &str) -> Option<serde_json::Value> {
    let path = std::path::Path::new("api").join("version.json");
    let content = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let arr = value.as_array()?;
    let platform = match platform {
        "mobile" => "mobile",
        "watch" => "watch",
        _ => "desktop",
    };
    let effective_system = if system.is_empty() { default_system(platform) } else { system };
    let mut best: Option<serde_json::Value> = None;
    let mut best_tier: u8 = u8::MAX;
    for item in arr {
        let item_platform = item.get("platform").and_then(|v| v.as_str()).unwrap_or("desktop");
        if item_platform != platform {
            continue;
        }
        let item_channel = item.get("channel").and_then(|v| v.as_str()).unwrap_or("stable");
        if item_channel != channel {
            continue;
        }
        let enabled = item.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
        if !enabled {
            continue;
        }
        if item_system(item, platform) != effective_system {
            continue;
        }
        let tier = arch_tier(item, arch);
        if tier == u8::MAX {
            continue;
        }
        let item_ver = item.get("version").and_then(|v| v.as_str()).unwrap_or("");
        let take = match (&best, tier.cmp(&best_tier)) {
            (_, std::cmp::Ordering::Less) => true,
            (_, std::cmp::Ordering::Greater) => false,
            (Some(cur), std::cmp::Ordering::Equal) => {
                let cur_ver = cur.get("version").and_then(|v| v.as_str()).unwrap_or("");
                compare_version_code(item_ver, cur_ver) > 0
            }
            (None, _) => true,
        };
        if take {
            best = Some(item.clone());
            best_tier = tier;
        }
    }
    best
}

async fn beta_device_allowed(pool: &MySqlPool, device_id: &str) -> bool {
    if device_id.is_empty() {
        return false;
    }
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM beta_testers WHERE device_id = ?")
        .bind(device_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
        > 0
}

/// 内测资格有效期：签发后 24 小时内可离线凭缓存通过，过期必须重新联网验证。
/// 同时决定移出名单的最迟生效时间（离线设备最多延迟一个 TTL 被锁）。
const BETA_ACCESS_TTL_SECS: i64 = 24 * 3600;

/// 与客户端 fallback_verify.rs::beta_access_message 逐字一致。
fn beta_access_message(device_id: &str, allowed: bool, pending: bool, exp: i64) -> Vec<u8> {
    format!(
        "xianyu-beta-access-v1\x00{device_id}\x00{}\x00{}\x00{exp}",
        allowed as u8,
        pending as u8
    )
    .into_bytes()
}

pub async fn check_beta_access(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let raw = parse_body(body);
    let device_id = str_of(&raw, "device_id").trim().to_string();
    let allowed = beta_device_allowed(pool, &device_id).await;
    let mut pending = false;
    if !allowed && !device_id.is_empty() {
        pending = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_feedback WHERE device_id = ? AND feedback_type = 'beta' AND status IN ('pending', 'processing')",
        )
        .bind(&device_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
            > 0;
    }
    // 响应签名：客户端 fail-closed 验签，无签名将被视为不可信而拒绝放行
    let exp = chrono::Utc::now().timestamp() + BETA_ACCESS_TTL_SECS;
    let signature = match crate::admin::fallback::load_signing_private_key() {
        Some(key) => hex::encode(key.sign(&beta_access_message(&device_id, allowed, pending, exp)).to_bytes()),
        None => {
            tracing::warn!("未配置内测资格签名密钥（FALLBACK_SIGN_PRIVATE_KEY 或 api/fallback_sign_key.txt），check_beta_access 响应未签名");
            String::new()
        }
    };
    ctx.json(200, "ok", Some(json!({ "allowed": allowed, "pending": pending, "exp": exp, "device_id": device_id, "sig": signature })))
}

pub async fn get_latest_version(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let raw = parse_body(body);
    let platform = str_of(&raw, "platform").trim().to_string();
    let system = str_of(&raw, "system").trim().to_string();
    let arch = str_of(&raw, "arch").trim().to_string();
    let device_id = str_of(&raw, "device_id").trim().to_string();

    let mut selected: Option<serde_json::Value> =
        latest_enabled_platform_version(&platform, "stable", &system, &arch);
    if beta_device_allowed(pool, &device_id).await {
        if let Some(beta) = latest_enabled_platform_version(&platform, "beta", &system, &arch) {
            let beta_newer = match &selected {
                Some(stable) => {
                    let bv = beta.get("version").and_then(|v| v.as_str()).unwrap_or("");
                    let sv = stable.get("version").and_then(|v| v.as_str()).unwrap_or("");
                    compare_version_code(bv, sv) > 0
                }
                None => true,
            };
            if beta_newer {
                selected = Some(beta);
            }
        }
    }

    if let Some(item) = selected {
        let is_beta = item.get("channel").and_then(|v| v.as_str()) == Some("beta");
        let version = item.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let arch = item.get("arch").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let content = item.get("updateContent").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let url = item.get("downloadUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let updated_at = item.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let app_name = match platform.as_str() {
            "mobile" => "弦予音乐移动端",
            "watch" => "弦予音乐腕上端",
            _ => "弦予音乐桌面端",
        };
        return ctx.json(
            200,
            "ok",
            Some(json!({
                "id": 0,
                "app_name": app_name,
                "version": version,
                "arch": arch,
                "content": content,
                "download_url": url,
                "file_size": 0,
                "status": "normal",
                "channel": if is_beta { "beta" } else { "stable" },
                "updated_at": updated_at
            })),
        );
    }
    let row = sqlx::query("SELECT * FROM app_versions WHERE status != 'disabled' ORDER BY id DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let id: i64 = r.get("id");
            let app_name: String = r.get("app_name");
            let version: String = r.get("version_code");
            let content: String = r.get("update_content");
            let url: String = r.get("download_url");
            let size: i64 = r.get("file_size");
            let status: String = r.get("status");
            ctx.json(
                200,
                "ok",
                Some(json!({
                    "id": id,
                    "app_name": app_name,
                    "version": version,
                    "content": content,
                    "download_url": url,
                    "file_size": size,
                    "status": status
                })),
            )
        }
        None => ctx.json::<serde_json::Value>(200, "ok", None),
    }
}

pub async fn share_download(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let raw = parse_body(body);
    let req_platform = str_of(&raw, "platform");
    let platform = if req_platform.is_empty() {
        "mobile".to_string()
    } else {
        req_platform
    };
    let arch = str_of(&raw, "arch");
    let app_name = match platform.as_str() {
        "mobile" => "弦予音乐移动端",
        "watch" => "弦予音乐腕上端",
        _ => "弦予音乐桌面端",
    };

    let system_keys: Vec<&str> = match platform.as_str() {
        "mobile" => vec!["android", "harmonyos", "ios"],
        "watch" => vec!["wearos", "ohos", "watchos"],
        _ => vec!["windows", "linux", "macos"],
    };
    let mut systems: Vec<serde_json::Value> = Vec::new();
    let mut first: Option<serde_json::Value> = None;
    for sys_key in &system_keys {
        if let Some(item) = latest_enabled_platform_version(&platform, "stable", sys_key, &arch) {
            let sys = if sys_key.is_empty() {
                app_name.to_string()
            } else {
                system_label(&platform, sys_key).to_string()
            };
            let entry = json!({
                "system": sys_key,
                "label": sys,
                "arch": item.get("arch").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                "version": item.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                "content": item.get("updateContent").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                "download_url": item.get("downloadUrl").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                "store_url": item.get("storeUrl").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            });
            if first.is_none() {
                first = Some(entry.clone());
            }
            systems.push(entry);
        }
    }

    if let Some(f) = first {
        return ctx.ok(
            "ok",
            json!({
                "platform": platform,
                "app_name": app_name,
                "systems": systems,
                "version": f.get("version").cloned().unwrap_or(Value::String(String::new())),
                "content": f.get("content").cloned().unwrap_or(Value::Null),
                "download_url": f.get("download_url").cloned().unwrap_or(Value::Null),
                "store_url": f.get("store_url").cloned().unwrap_or(Value::Null),
            }),
        );
    }

    let row = sqlx::query("SELECT * FROM app_versions WHERE status != 'disabled' ORDER BY id DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let version: String = r.get("version_code");
            let url: String = r.get("download_url");
            ctx.ok(
                "ok",
                json!({
                    "platform": platform,
                    "app_name": app_name,
                    "systems": [],
                    "version": version,
                    "content": "",
                    "download_url": url,
                    "store_url": "",
                }),
            )
        }
        None => ctx.err(404, "暂无可用下载"),
    }
}
