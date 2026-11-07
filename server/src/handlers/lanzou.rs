use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{ConnectInfo, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use std::net::SocketAddr;

use crate::AppState;

// 蓝奏云直链代理解析：后台卡片填蓝奏云分享链接（可选访问密码），
// 下发侧改写为 /api/dl?u=...&p=...，本模块代理解析出真实直链后 302 重定向，
// 客户端/官网无需感知网盘验证，体验与本服安装包一致。

/// 浏览器 UA：蓝奏云对 UA 敏感，带密码页与直链解析均要求普通浏览器特征
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// 成功解析缓存：直链 sign 有时效，超期会导致 302 后下载失败，TTL 取保守值
const CACHE_TTL_SECS: i64 = 480;
/// 失败负缓存：避免蓝奏云改版/断网时每次点击都打一轮三段请求
const NEGATIVE_TTL_SECS: i64 = 60;
/// 单 IP 解析频率上限（缓存命中不计）
const RESOLVE_PER_MINUTE: usize = 10;

fn host_of(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or("");
    rest.split(['/', '?', '#']).next().unwrap_or("").to_lowercase()
}

/// 蓝奏云系域名族（lanzou* / lanzn / lanosu*，随官方换域漂移）
pub fn is_lanzou_url(url: &str) -> bool {
    let host = host_of(url);
    !host.is_empty()
        && host
            .split('.')
            .any(|p| p.contains("lanzou") || p == "lanzn" || p.contains("lanosu"))
}

fn pct_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push('%');
            out.push_str(&format!("{:02X}", b));
        }
    }
    out
}

/// 下发侧改写：蓝奏云链接一律走代理解析端点（无密码链接也解析直链，跳过网盘中转页），
/// 其余链接走原有 absolutize 逻辑原样下发
pub fn rewrite_download_url(base_url: &str, public_base: &str, raw_url: &str, pwd: &str) -> String {
    let raw = raw_url.trim();
    if raw.is_empty() || !is_lanzou_url(raw) {
        return crate::handlers::upload::absolutize_media_url_with(base_url, public_base, raw);
    }
    let mut path = format!("/api/dl?u={}", pct_encode(raw));
    let pwd = pwd.trim();
    if !pwd.is_empty() {
        path.push_str(&format!("&p={}", pct_encode(pwd)));
    }
    crate::handlers::upload::absolutize_media_url_with(base_url, public_base, &path)
}

// ===================== 解析结果缓存 =====================

type CacheMap = HashMap<String, (i64, Option<String>)>;

static CACHE: Mutex<Option<CacheMap>> = Mutex::new(None);

/// 命中返回 Some(直链)；负缓存返回 Some(None)；无缓存返回 None
fn cache_lookup(key: &str) -> Option<Option<String>> {
    let now = chrono::Utc::now().timestamp();
    let mut guard = CACHE.lock().ok()?;
    let map = guard.get_or_insert_with(HashMap::new);
    map.get(key).and_then(|(at, val)| {
        let ttl = if val.is_some() { CACHE_TTL_SECS } else { NEGATIVE_TTL_SECS };
        if now - *at < ttl {
            Some(val.clone())
        } else {
            None
        }
    })
}

fn cache_store(key: &str, val: Option<String>) {
    let now = chrono::Utc::now().timestamp();
    if let Ok(mut guard) = CACHE.lock() {
        let map = guard.get_or_insert_with(HashMap::new);
        // 顺手清理过期项，防长期运行下键堆积
        map.retain(|_, (at, v)| {
            let ttl = if v.is_some() { CACHE_TTL_SECS } else { NEGATIVE_TTL_SECS };
            now - *at < ttl * 4
        });
        map.insert(key.to_string(), (now, val));
    }
}

// ===================== 单 IP 频率限制 =====================

static RL: Mutex<Option<HashMap<String, Vec<i64>>>> = Mutex::new(None);

fn rate_allow(ip: &str) -> bool {
    let now = chrono::Utc::now().timestamp();
    if let Ok(mut guard) = RL.lock() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.retain(|_, window| window.iter().any(|t| now - *t < 60));
        let window = map.entry(ip.to_string()).or_default();
        window.retain(|t| now - *t < 60);
        if window.len() >= RESOLVE_PER_MINUTE {
            return false;
        }
        window.push(now);
        true
    } else {
        true
    }
}

// ===================== 蓝奏云解析 =====================

/// 从 fn 页 JS 中抠 `var NAME = 'VALUE';`：同名变量可能重复赋值（先空串后真值），取第一个非空值
fn js_var(html: &str, name: &str) -> Option<String> {
    let marker = format!("{} = '", name);
    let mut found = None;
    let mut rest = html;
    while let Some(idx) = rest.find(&marker) {
        let after = &rest[idx + marker.len()..];
        match after.find('\'') {
            Some(end) => {
                if end > 0 && found.is_none() {
                    found = Some(after[..end].to_string());
                }
                rest = &after[end..];
            }
            None => break,
        }
    }
    found
}

/// 解析直链 JSON 响应（{zt:'1', dom, url}）→ dom + "/file/" + url
fn parse_direct_json(text: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let zt_ok = v.get("zt").map(|z| {
        z.as_i64() == Some(1) || z.as_str().map(|s| s == "1").unwrap_or(false)
    }).unwrap_or(false);
    if !zt_ok {
        return None;
    }
    let dom = v.get("dom").and_then(|d| d.as_str()).unwrap_or("");
    let file = v.get("url").and_then(|u| u.as_str()).unwrap_or("");
    if dom.is_empty() || file.is_empty() {
        return None;
    }
    Some(format!("{}/file/{}", dom.trim_end_matches('/'), file.trim_start_matches('/')))
}

/// 混合式解析，两种页面形态：
/// A 带密码页：页面 JS 直接内嵌 isngis 签名 + ajaxfile.php 地址，POST {sign, kd, p:密码}
/// B 无密码页：/fn? 中转页 → 抠 wp_sign/ajaxdata/domain1/domain2 → POST 换直链
async fn resolve_direct(url: &str, pwd: &str) -> Result<String, String> {
    let cookie_jar = Arc::new(reqwest::cookie::Jar::default());
    let client = reqwest::Client::builder()
        .user_agent(BROWSER_UA)
        .cookie_provider(cookie_jar.clone())
        .timeout(std::time::Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| e.to_string())?;

    // 1) 文件页（跟随分享域名的 302）
    let page_url = client
        .get(url)
        .header(reqwest::header::REFERER, "https://lanzou.com/")
        .send()
        .await
        .map_err(|e| format!("文件页请求失败: {e}"))?
        .url()
        .clone();
    let html = client
        .get(page_url.clone())
        .send()
        .await
        .map_err(|e| format!("文件页请求失败: {e}"))?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    // 分支 A：带密码页（JS 内嵌 isngis + ajaxfile.php）
    if let Some(sign) = js_var(&html, "isngis").filter(|s| !s.is_empty()) {
        let pwd = pwd.trim();
        if pwd.is_empty() {
            return Err("该链接需要访问密码".to_string());
        }
        let api = html
            .split(|c| c == '"' || c == '\'')
            .find(|seg| seg.contains("ajaxfile.php?file=") && seg.len() < 512)
            .ok_or("密码页缺少 ajaxfile 地址")?
            .to_string();
        let resp = client
            .post(&api)
            .header(reqwest::header::REFERER, page_url.as_str())
            .form(&[
                ("action", "downprocess"),
                ("sign", sign.as_str()),
                ("kd", "1"),
                ("p", pwd),
            ])
            .send()
            .await
            .map_err(|e| format!("密码提交失败: {e}"))?;
        let text = resp.text().await.map_err(|e| e.to_string())?;
        return parse_direct_json(&text).ok_or_else(|| "访问密码校验未通过".to_string());
    }

    // 分支 B：无密码页（/fn? 中转）
    let fn_path = html
        .split(|c| c == '"' || c == '\'')
        .find(|seg| seg.contains("/fn?") && seg.len() < 512)
        .ok_or_else(|| "未找到解析接口（可能已改版）".to_string())?
        .to_string();
    let fn_url = if fn_path.starts_with("http://") || fn_path.starts_with("https://") {
        fn_path
    } else {
        format!("{}://{}{}", page_url.scheme(), host_of(page_url.as_str()), fn_path)
    };
    let fn_html = client
        .get(&fn_url)
        .header(reqwest::header::REFERER, page_url.as_str())
        .send()
        .await
        .map_err(|e| format!("中转页请求失败: {e}"))?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    // 中转页 JS 变量：wp_sign 签名、ajaxdata webkey、ajaxfile 两个候选域
    let sign = js_var(&fn_html, "wp_sign").ok_or("中转页缺少 wp_sign")?;
    let webkey = js_var(&fn_html, "ajaxdata").ok_or("中转页缺少 ajaxdata")?;
    let domain1 = js_var(&fn_html, "domain1");
    let domain2 = js_var(&fn_html, "domain2");
    // 浏览器里 killdns/killdnsweb 脚本加载成功 → kd=1 + domain1；加载失败 → kd=0 + domain2
    let mut candidates: Vec<(String, i32)> = Vec::new();
    if let Some(d) = domain1.filter(|d| d.starts_with("http")) {
        candidates.push((d, 1));
    }
    if let Some(d) = domain2.filter(|d| d.starts_with("http")) {
        candidates.push((d, 0));
    }
    if candidates.is_empty() {
        return Err("中转页缺少 ajaxfile 域".to_string());
    }

    for (api, kd) in candidates {
        let resp = client
            .post(&api)
            .header(reqwest::header::REFERER, fn_url.as_str())
            .form(&[
                ("action", "downprocess"),
                ("websignkey", webkey.as_str()),
                ("signs", webkey.as_str()),
                ("sign", sign.as_str()),
                ("websign", ""),
                ("kd", if kd == 1 { "1" } else { "0" }),
                ("ves", "1"),
            ])
            .send()
            .await;
        let text = match resp {
            Ok(r) => r.text().await.unwrap_or_default(),
            Err(_) => continue,
        };
        if let Some(direct) = parse_direct_json(&text) {
            return Ok(direct);
        }
    }
    Err("解析接口拒绝或全部候选域失败".to_string())
}

/// GET /api/dl?u=<蓝奏云分享链接>&p=<访问密码> → 302 直链；解析失败 302 兜底回分享页
pub async fn resolve_dl(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let target = params.get("u").map(|u| u.trim().to_string()).unwrap_or_default();
    let pwd = params.get("p").map(|p| p.trim().to_string()).unwrap_or_default();
    if target.is_empty() || !is_lanzou_url(&target) || !target.starts_with("http") {
        return (StatusCode::BAD_REQUEST, "invalid link").into_response();
    }
    let ip = crate::sign::get_client_ip(
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
        headers.get("x-real-ip").and_then(|v| v.to_str().ok()),
        Some(peer.ip().to_string().as_str()),
        state.config.trust_proxy,
    );
    let cache_key = format!("{}|{}", target, pwd);
    let resolved = if rate_allow(&ip) {
        match cache_lookup(&cache_key) {
            Some(hit) => hit,
            None => {
                let result = resolve_direct(&target, &pwd).await;
                let out = match &result {
                    Ok(direct) => {
                        cache_store(&cache_key, Some(direct.clone()));
                        Some(direct.clone())
                    }
                    Err(e) => {
                        tracing::warn!("蓝奏云解析失败 {}: {e}", host_of(&target));
                        cache_store(&cache_key, None);
                        None
                    }
                };
                out
            }
        }
    } else {
        None
    };
    // 命中直链 → 302 直链；限流/解析失败 → 302 回分享页（用户手动点一次，服务不中断）
    let fallback = (StatusCode::FOUND, [("location", target.as_str())]).into_response();
    match resolved {
        Some(direct) => (StatusCode::FOUND, [("location", direct.as_str())]).into_response(),
        None => fallback,
    }
}

#[cfg(test)]
mod tests {
    /// 真实链路验证：LANZOU_TEST_URL / LANZOU_TEST_PWD 环境变量提供分享链接与密码
    #[tokio::test]
    #[ignore]
    async fn real_lanzou_resolution() {
        let url = std::env::var("LANZOU_TEST_URL").unwrap_or_default();
        if url.is_empty() {
            return;
        }
        let pwd = std::env::var("LANZOU_TEST_PWD").unwrap_or_default();
        let direct = super::resolve_direct(&url, &pwd).await.expect("resolve failed");
        println!("direct={direct}");
        assert!(direct.contains("/file/?"), "unexpected direct url: {direct}");
    }
}
