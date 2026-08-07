use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, proxy_response, AdminCtx};
use crate::handlers::helpers::{parse_body, str_of};
use crate::sign;

fn api_base_url() -> String {
    std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://api.ciyuanxi.bzxhkj.com".into())
}

fn api_secret() -> String {
    std::env::var("API_SECRET").unwrap_or_default()
}

fn nonce() -> String {
    uuid::Uuid::new_v4().simple().to_string().chars().take(16).collect()
}

fn sign_headers(timestamp: &str, nonce: &str, target: &str, secret: &str) -> Vec<(String, String)> {
    let sig = sign::md5_hex(format!("{}{}{}{}", timestamp, nonce, target, secret).as_bytes());
    vec![
        ("X-Timestamp".to_string(), timestamp.to_string()),
        ("X-Nonce".to_string(), nonce.to_string()),
        ("X-Sign".to_string(), sig),
    ]
}

/// 普通接口代理测试（GET/POST -> http://api.ciyuanxi.bzxhkj.com/index.php?action=）
pub async fn proxy_api_test(body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let api_action = str_of(&data, "api_action").trim().to_string();
    let method = str_of(&data, "method").to_uppercase();
    let req_body = str_of(&data, "body").to_string();
    let need_sign = str_of(&data, "need_sign") == "1";
    if api_action.is_empty() {
        return err(400, "缺少接口名称");
    }
    let url = format!("{}/index.php?action={}", api_base_url(), api_action);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    let result = if method == "POST" {
        let mut req = client
            .post(&url)
            .body(req_body.clone())
            .header("content-type", "application/json; charset=utf-8");
        if need_sign {
            let ts = sign::now_ts().to_string();
            let n = nonce();
            for (k, v) in sign_headers(&ts, &n, &req_body, &api_secret()) {
                req = req.header(k, v);
            }
        }
        req.send().await
    } else {
        let mut req = client.get(&url);
        if need_sign {
            let ts = sign::now_ts().to_string();
            let n = nonce();
            for (k, v) in sign_headers(&ts, &n, "", &api_secret()) {
                req = req.header(k, v);
            }
        }
        req.send().await
    };

    match result {
        Ok(resp) => {
            let http_code = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            match serde_json::from_str::<Value>(&text) {
                Ok(mut v) => {
                    if let Some(map) = v.as_object_mut() {
                        map.insert("http_code".into(), json!(http_code));
                    }
                    proxy_response(Some(v))
                }
                Err(_) => {
                    let short: String = text.chars().take(200).collect();
                    let raw: String = text.chars().take(500).collect();
                    proxy_response(Some(json!({
                        "code": 500, "msg": format!("返回非JSON: {}", short),
                        "http_code": http_code, "raw": raw,
                    })))
                }
            }
        }
        Err(e) => err(500, &format!("请求失败: {}", e)),
    }
}

/// APP 接口代理测试（/api/app.php，支持签名 + AES 加密）
pub async fn proxy_app_api_test(body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let api_action = str_of(&data, "api_action").trim().to_string();
    let method = str_of(&data, "method").to_uppercase();
    let mut req_body = str_of(&data, "body").to_string();
    let need_sign_raw = str_of(&data, "need_sign");
    let need_encrypt = str_of(&data, "need_encrypt") == "1";
    let admin_token = str_of(&data, "admin_token").to_string();
    let device_id = if str_of(&data, "device_id").is_empty() { "web_test_device".to_string() } else { str_of(&data, "device_id") };
    let device_model = if str_of(&data, "device_model").is_empty() { "WebTester".to_string() } else { str_of(&data, "device_model") };
    let app_version = if str_of(&data, "app_version").is_empty() { "1.0.0".to_string() } else { str_of(&data, "app_version") };
    let os_version = if str_of(&data, "os_version").is_empty() { "web".to_string() } else { str_of(&data, "os_version") };
    if api_action.is_empty() {
        return err(400, "缺少接口名称");
    }
    let no_sign = ["app_check", "app_install"];
    let need_sign = (need_sign_raw == "1" || need_sign_raw.is_empty()) && !no_sign.contains(&api_action.as_str());
    let secret = api_secret();
    let url = format!("{}/api/app.php?action={}", api_base_url(), api_action);

    let mut headers: Vec<(String, String)> = vec![
        ("user-agent".to_string(), "CiyuanxiAdmin/1.0 (Android)".to_string()),
        ("X-Device-Id".to_string(), device_id),
        ("X-Device-Model".to_string(), device_model),
        ("X-App-Version".to_string(), app_version),
        ("X-Os-Version".to_string(), os_version),
    ];
    if !admin_token.is_empty() {
        headers.push(("X-Admin-Token".to_string(), admin_token));
    }

    let mut is_plain = false;
    if need_encrypt && need_sign && !req_body.is_empty() {
        if let Some((iv, ct)) = sign::aes_encrypt(req_body.as_bytes(), &secret) {
            req_body = ct;
            headers.push(("X-Encrypted-IV".to_string(), iv));
            headers.push(("content-type".to_string(), "text/plain; charset=utf-8".to_string()));
            is_plain = true;
        }
    }
    if !is_plain {
        headers.push(("content-type".to_string(), "application/json; charset=utf-8".to_string()));
    }
    if need_sign {
        let ts = sign::now_ts().to_string();
        let n = nonce();
        let target = if method == "POST" { req_body.clone() } else { String::new() };
        for (k, v) in sign_headers(&ts, &n, &target, &secret) {
            headers.push((k, v));
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    let result = if method == "POST" {
        let mut r = client.post(&url).body(req_body.clone());
        for (k, v) in &headers {
            r = r.header(k, v);
        }
        r.send().await
    } else {
        let mut r = client.get(&url);
        for (k, v) in &headers {
            r = r.header(k, v);
        }
        r.send().await
    };

    match result {
        Ok(resp) => {
            let http_code = resp.status().as_u16();
            let resp_iv = resp.headers().get("X-Response-IV").map(|v| v.to_str().unwrap_or("").to_string());
            let text = resp.text().await.unwrap_or_default();
            let mut final_value: Option<Value> = None;
            let mut encrypted_resp = false;
            if let Some(iv) = resp_iv {
                if let Some(pt) = sign::aes_decrypt(&text, &iv, &secret) {
                    if let Ok(v) = serde_json::from_str::<Value>(&pt) {
                        final_value = Some(v);
                        encrypted_resp = true;
                    }
                }
            }
            if final_value.is_none() {
                final_value = serde_json::from_str::<Value>(&text).ok();
            }
            match final_value {
                Some(mut v) => {
                    if let Some(map) = v.as_object_mut() {
                        map.insert("http_code".into(), json!(http_code));
                        if encrypted_resp {
                            map.insert("encrypted_response".into(), json!(true));
                        }
                    }
                    proxy_response(Some(v))
                }
                None => {
                    let short: String = text.chars().take(200).collect();
                    let raw: String = text.chars().take(500).collect();
                    proxy_response(Some(json!({
                        "code": 500, "msg": format!("返回非JSON: {}", short),
                        "http_code": http_code, "raw": raw,
                    })))
                }
            }
        }
        Err(e) => err(500, &format!("请求失败: {}", e)),
    }
}