// cloud_proxy：网盘业务代理总入口。
//
// 客户端把各家网盘的目录浏览/分享解析/转存/直链解析业务上移到本模块，
// 请求体携带凭据、服务端即用即弃（无持久化、无明文落库、日志不打 cred）。
// 客户端在 Server 不可达/未登录/报错时自动回退本地直连实现。
//
// 日志脱敏约定：本模块任何 tracing 输出不得包含 cred / token / cookie 内容。

use axum::response::Response;
use serde_json::Value;
use sqlx::MySqlPool;

use crate::response::ReqCtx;

pub mod aliyun;
pub mod baidu;
pub mod ecloud189;
pub mod http;
pub mod lanzou;
pub mod mobile139;
pub mod pan123;
pub mod parser;
pub mod pikpak;
pub mod quark_uc;
pub mod thunder;
pub mod types;
pub mod weiyun;

use types::{CloudOut, CloudReq};

pub async fn cloud_proxy(body: &str, ctx: ReqCtx, _pool: &MySqlPool) -> Response {
    let req = match CloudReq::parse(body) {
        Ok(r) => r,
        Err((code, msg)) => return ctx.err(code, &msg),
    };
    tracing::info!(action = "cloud_proxy", provider = %req.provider, op = %req.op);

    let outcome: Result<CloudOut, (i32, String)> = match req.provider.as_str() {
        "quark" | "uc" => quark_uc::dispatch(&req).await,
        "aliyun" => aliyun::dispatch(&req).await,
        "baidu" => baidu::dispatch(&req).await,
        "ecloud189" => ecloud189::dispatch(&req).await,
        "lanzou" => lanzou::dispatch(&req).await,
        "mobile139" => mobile139::dispatch(&req).await,
        "pan123" => pan123::dispatch(&req).await,
        "pikpak" => pikpak::dispatch(&req).await,
        "thunder" => thunder::dispatch(&req).await,
        "weiyun" => weiyun::dispatch(&req).await,
        _ => Err((404, format!("未知网盘: {}", req.provider))),
    };

    match outcome {
        Ok(CloudOut {
            result,
            rotated_cred,
        }) => {
            let mut data = serde_json::json!({ "result": result });
            if let Some(rc) = rotated_cred {
                if rc.is_object() && !rc.as_object().unwrap().is_empty() {
                    data["rotated_cred"] = rc;
                }
            }
            ctx.ok("成功", data)
        }
        Err((code, msg)) => {
            // 业务失败（分享失效/提取码错误/凭据过期等）→ 客户端将回退本地直连。
            // msg 仅中文业务文案，按脱敏约定不含 cred/token/cookie。
            tracing::warn!(
                action = "cloud_proxy",
                provider = %req.provider,
                op = %req.op,
                code,
                msg = %msg,
                "云端代理业务失败，客户端将回退本地直连"
            );
            ctx.err(code, &msg)
        }
    }
}

/// 各盘 op 前置公共校验：需凭据的 op 拒绝空 cred。
pub fn require_cred(req: &CloudReq) -> Result<(), (i32, String)> {
    if req.cred.as_object().map(|m| m.is_empty()).unwrap_or(true) {
        return Err((400, "缺少网盘凭据".to_string()));
    }
    Ok(())
}

/// parse_share op 的公共前置：识别链接归属 + 要求 provider 一致。
pub fn pre_parse_share(req: &CloudReq, expect_provider: &str) -> Result<parser::ShareLinkInfo, (i32, String)> {
    let url = req.param_str("url");
    if url.is_empty() {
        return Err((400, "缺少分享链接".to_string()));
    }
    let info = parser::parse_url_to_provider(&url)
        .ok_or_else(|| (400, "无法识别的分享链接".to_string()))?;
    if info.provider_id != expect_provider {
        return Err((400, "分享链接与网盘不匹配".to_string()));
    }
    Ok(info)
}

/// 业务错误统一映射：网盘侧错误透传为 502（客户端视为回退信号）。
pub fn biz_err(msg: impl std::fmt::Display) -> (i32, String) {
    (502, msg.to_string())
}

/// op 特定参数读取辅助：params 里任意 JSON 值。
pub fn param_value<'a>(req: &'a CloudReq, key: &str) -> Option<&'a Value> {
    req.params.get(key)
}
