use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, proxy_response, AdminCtx};
use crate::handlers::helpers::{parse_body, str_of};
use crate::sign;

fn local_api_base(ctx: &AdminCtx) -> String {
    let addr = ctx.config.listen_addr.replace("0.0.0.0", "127.0.0.1");
    format!("http://{}", addr)
}

fn nonce() -> String {
    uuid::Uuid::new_v4().simple().to_string().chars().take(16).collect()
}

fn sign_headers(timestamp: &str, nonce: &str, body: &str, secret: &str) -> Vec<(String, String)> {
    let sig = sign::hmac_sha256_hex(format!("{}{}{}", timestamp, nonce, body).as_bytes(), secret);
    vec![
        ("X-Timestamp".to_string(), timestamp.to_string()),
        ("X-Nonce".to_string(), nonce.to_string()),
        ("X-Sign".to_string(), sig),
    ]
}

pub async fn proxy_api_test(body: &str, ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let api_action = str_of(&data, "api_action").trim().to_string();
    let method = str_of(&data, "method").to_uppercase();
    let req_body = str_of(&data, "body").to_string();
    let need_sign = str_of(&data, "need_sign") == "1";
    if api_action.is_empty() {
        return err(400, "缺少接口名称");
    }
    let url = format!("{}/api?action={}", local_api_base(ctx), api_action);
    let secret = &ctx.config.api_secret;
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
            for (k, v) in sign_headers(&ts, &n, &req_body, secret) {
                req = req.header(k, v);
            }
        }
        req.send().await
    } else {
        let mut req = client.get(&url);
        if need_sign {
            let ts = sign::now_ts().to_string();
            let n = nonce();
            for (k, v) in sign_headers(&ts, &n, "", secret) {
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
        Err(e) => { tracing::error!("请求失败: {e}"); err(500, "请求失败") },
    }
}

pub async fn proxy_app_api_test(body: &str, ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    proxy_api_test(body, ctx, _pool).await
}
