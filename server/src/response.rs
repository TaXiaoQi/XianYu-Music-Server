use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::{json, Value};

use crate::config::Config;

/// 请求上下文：携带共享状态与请求头（用于判断是否需要加密响应）
#[derive(Clone)]
pub struct ReqCtx {
    pub config: Config,
    pub encrypted: bool,
    pub client_ip: String,
    pub base_url: String,
}

impl ReqCtx {
    pub fn new(config: Config, headers: &HeaderMap) -> Self {
        let encrypted = headers
            .get("x-encrypted-iv")
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        let client_ip = crate::sign::get_client_ip(
            headers
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok()),
            headers.get("x-real-ip").and_then(|v| v.to_str().ok()),
            None,
        );
        let host = headers
            .get("x-forwarded-host")
            .or_else(|| headers.get(axum::http::header::HOST))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let scheme = headers
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(',').next().unwrap_or(v).trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| {
                if host.starts_with("localhost") || host.starts_with("127.") || host.starts_with("0.0.0.0") {
                    "http".to_string()
                } else {
                    "https".to_string()
                }
            });
        let base_url = if host.is_empty() {
            config.public_base_url.clone()
        } else {
            format!("{}://{}", scheme, host)
        };
        Self {
            config,
            encrypted,
            client_ip,
            base_url,
        }
    }

    /// 输出 JSON 响应；若为加密请求则 AES 加密后返回，与 PHP Sign::jsonResponse 一致
    pub fn json<T: Serialize>(&self, code: i32, msg: &str, data: Option<T>) -> Response {
        let payload = serde_json::to_string(&json!({ "code": code, "msg": msg, "data": data }))
            .unwrap_or_else(|_| r#"{"code":500,"msg":"serialize error","data":null}"#.to_string());

        if self.encrypted {
            if let Some((iv, ct)) = crate::sign::aes_encrypt(payload.as_bytes(), &self.config.api_secret) {
                let mut headers = HeaderMap::new();
                headers.insert("content-type", "text/plain; charset=utf-8".parse().unwrap());
                headers.insert("x-encrypted-response", "1".parse().unwrap());
                headers.insert("x-response-iv", iv.parse().unwrap());
                let status = if code >= 400 && code < 600 {
                    StatusCode::from_u16(code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
                } else {
                    StatusCode::OK
                };
                return (status, headers, Body::from(ct)).into_response();
            }
        }

        let status = if code >= 400 && code < 600 {
            StatusCode::from_u16(code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        } else {
            StatusCode::OK
        };
        (status, [(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")], Body::from(payload)).into_response()
    }

    pub fn ok<T: Serialize>(&self, msg: &str, data: T) -> Response {
        self.json(200, msg, Some(data))
    }

    pub fn ok_empty(&self, msg: &str) -> Response {
        self.json::<Value>(200, msg, None)
    }

    pub fn err(&self, code: i32, msg: &str) -> Response {
        self.json::<Value>(code, msg, None)
    }
}
