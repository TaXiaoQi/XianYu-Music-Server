use axum::extract::ws::Message as WsMessage;
use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use std::time::Duration;
use tokio::sync::mpsc;

use super::*;
use crate::handlers::helpers::{parse_body, str_of};

const WH_KEY_ENABLED: &str = "webhook_enabled";
const WH_KEY_URL: &str = "webhook_url";
const WH_KEY_METHOD: &str = "webhook_method";
const WH_KEY_HEADERS: &str = "webhook_headers";
const WH_KEY_BODY_TEMPLATE: &str = "webhook_body_template";
const WH_KEY_MODULES: &str = "webhook_modules";

pub async fn get_webhook_config(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let enabled = read_setting(pool, WH_KEY_ENABLED).await == "1";
    let url = read_setting(pool, WH_KEY_URL).await;
    let method = read_setting(pool, WH_KEY_METHOD).await;
    let headers = read_setting(pool, WH_KEY_HEADERS).await;
    let body_template = read_setting(pool, WH_KEY_BODY_TEMPLATE).await;
    let modules_raw = read_setting(pool, WH_KEY_MODULES).await;
    let mut modules = json!({});
    for m in super::email::NOTIFY_MODULES.iter() {
        let key = format!("wh_{}", m);
        let on = modules_raw
            .split(',')
            .any(|s| s.trim() == *m);
        modules[&key] = json!(on);
    }
    ok(
        "",
        json!({
            "enabled": enabled,
            "url": url,
            "method": if method.is_empty() { "POST".to_string() } else { method },
            "headers": headers,
            "body_template": body_template,
            "modules": modules,
        }),
    )
}

pub async fn save_webhook_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let enabled = data.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let url = str_of(&data, "url").trim().to_string();
    let method = str_of(&data, "method").trim().to_uppercase();
    let method = if method.is_empty() { "POST".to_string() } else { method };
    let headers = str_of(&data, "headers");
    let body_template = str_of(&data, "body_template");

    if enabled && url.is_empty() {
        return err(400, "启用 Webhook 时请输入回调地址");
    }
    // 防 SSRF：仅允许公网 http(s) 目标
    if !url.is_empty() {
        if let Err(e) = crate::audit_policy::validate_external_endpoint(&url).await {
            return err(400, &format!("回调地址不可用: {}", e));
        }
    }
    let mut modules_on: Vec<String> = Vec::new();
    if let Some(mods) = data.get("modules").and_then(|v| v.as_object()) {
        for (k, v) in mods {
            if let Some(m) = k.strip_prefix("wh_") {
                if v.as_bool().unwrap_or(false) {
                    modules_on.push(m.to_string());
                }
            }
        }
    }
    upsert_setting(pool, WH_KEY_ENABLED, if enabled { "1" } else { "0" }, "通用Webhook开关").await;
    upsert_setting(pool, WH_KEY_URL, &url, "通用Webhook回调地址").await;
    upsert_setting(pool, WH_KEY_METHOD, &method, "通用Webhook请求方法").await;
    upsert_setting(pool, WH_KEY_HEADERS, &headers, "通用Webhook自定义请求头").await;
    upsert_setting(pool, WH_KEY_BODY_TEMPLATE, &body_template, "通用Webhook请求体模板").await;
    upsert_setting(pool, WH_KEY_MODULES, &modules_on.join(","), "通用Webhook触发板块").await;
    super::log_operation(pool, ctx, "更新Webhook配置", &url, &format!("方法:{} 板块:{}", method, modules_on.join(","))).await;
    ok("已保存", Value::Null)
}

pub(crate) async fn upsert_setting(pool: &MySqlPool, key: &str, value: &str, desc: &str) {
    let _ = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), description = VALUES(description)",
    )
    .bind(key)
    .bind(value)
    .bind(desc)
    .execute(pool)
    .await;
}

pub async fn test_webhook(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let url = str_of(&data, "url").trim().to_string();
    let method = str_of(&data, "method").trim().to_uppercase();
    let method = if method.is_empty() { "POST".to_string() } else { method };
    let headers_raw = str_of(&data, "headers");
    let body_template = str_of(&data, "body_template");

    if url.is_empty() {
        return err(400, "请输入回调地址");
    }
    if let Err(e) = crate::audit_policy::validate_external_endpoint(&url).await {
        return err(400, &format!("回调地址不可用: {}", e));
    }
    let payload = if body_template.trim().is_empty() {
        json!({
            "event": "test",
            "message": "这是一条测试通知",
            "time": now_str(),
        })
        .to_string()
    } else {
        render_template(&body_template, "test", "测试通知", "", "", "")
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    let mut req = client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::POST), &url);
    for line in headers_raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            req = req.header(k.trim(), v.trim());
        }
    }
    if !payload.is_empty() {
        req = req
            .header("content-type", "application/json")
            .body(payload);
    }
    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let resp_body = resp.text().await.unwrap_or_default();
            // 日志只留响应体摘要，防内网数据经操作日志外泄/膨胀
            let log_body: String = resp_body.chars().take(512).collect();
            super::log_operation(pool, ctx, "测试Webhook", &url, &format!("{} {}", status, log_body)).await;
            ok("", json!({ "status": status, "body": resp_body }))
        }
        Err(e) => { tracing::error!("请求失败: {e}"); err(500, "请求失败") },
    }
}

pub fn render_template(template: &str, event: &str, title: &str, detail: &str, image_url: &str, link: &str) -> String {
    let mut s = template.to_string();
    let map = [
        ("{{event}}", event),
        ("{{title}}", title),
        ("{{detail}}", detail),
        ("{{image_url}}", image_url),
        ("{{link}}", link),
        ("{{time}}", &now_str()),
    ];
    for (k, v) in map {
        s = s.replace(k, v);
    }
    s
}

pub async fn notify_webhook(
    pool: &MySqlPool,
    module: &str,
    title: &str,
    detail: &str,
    image_url: &str,
    link: &str,
) {
    if read_setting(pool, WH_KEY_ENABLED).await != "1" {
        return;
    }
    let modules_raw = read_setting(pool, WH_KEY_MODULES).await;
    if !modules_raw.split(',').any(|s| s.trim() == module) {
        return;
    }
    let url = read_setting(pool, WH_KEY_URL).await;
    if url.is_empty() {
        return;
    }
    // 运行时兜底：存量配置若指向内网则跳过外呼
    if crate::audit_policy::validate_external_endpoint(&url).await.is_err() {
        tracing::warn!("webhook 地址未通过公网校验，跳过推送: {}", url);
        return;
    }
    let method = read_setting(pool, WH_KEY_METHOD).await;
    let method = if method.is_empty() { "POST".to_string() } else { method };
    let headers_raw = read_setting(pool, WH_KEY_HEADERS).await;
    let body_template = read_setting(pool, WH_KEY_BODY_TEMPLATE).await;
    let payload = if body_template.trim().is_empty() {
        json!({
            "event": module,
            "title": title,
            "detail": detail,
            "image_url": image_url,
            "link": link,
            "time": now_str(),
        })
        .to_string()
    } else {
        render_template(&body_template, module, title, detail, image_url, link)
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_default();
    let mut req = client.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::POST), &url);
    for line in headers_raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            req = req.header(k.trim(), v.trim());
        }
    }
    if !payload.is_empty() {
        req = req
            .header("content-type", "application/json")
            .body(payload);
    }
    let _ = req.send().await;
}

// ===================== 统一事件广播（参考 napcat 事件分发） =====================

pub async fn broadcast_event(
    pool: &MySqlPool,
    module: &str,
    title: &str,
    detail: &str,
    image_url: &str,
    link: &str,
) {
    notify_webhook(pool, module, title, detail, image_url, link).await;

    let payload = json!({
        "event": module,
        "title": title,
        "detail": detail,
        "image_url": image_url,
        "link": link,
        "time": now_str(),
        "channel": "event",
    })
    .to_string();
    let ws_targets: Vec<mpsc::Sender<WsMessage>> = {
        let clients = comm_state().ws_server_clients.lock().unwrap();
        clients
            .values()
            .filter(|c| c.events.is_empty() || c.events.contains(module))
            .map(|c| c.tx.clone())
            .collect()
    };
    for tx in ws_targets {
        let _ = tx.send(WsMessage::Text(payload.clone().into())).await;
    }

    let sse_targets: Vec<mpsc::Sender<String>> = {
        let clients = comm_state().sse_clients.lock().unwrap();
        clients
            .values()
            .filter(|c| c.events.is_empty() || c.events.contains(module))
            .map(|c| c.tx.clone())
            .collect()
    };
    for tx in sse_targets {
        let _ = tx.try_send(payload.clone());
    }
}

// ===================== WS 客户端自动重连配置 =====================

