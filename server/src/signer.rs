//! 兜底/内测签名签发服务（B 档托管）+ 离线 keygen CLI。
//!
//! 同一二进制的两个子命令模式（不连数据库、不对外网监听）：
//! - `server --signer`：签发服务。私钥仅存在于本进程内存（启动时从
//!   `api/fallback_sign_keys/` 密封文件加载），主服务经 localhost 回环 +
//!   共享令牌调用；对消息做 canonical 配置 schema 校验、限流与审计后签发，
//!   绝不返回私钥。
//! - `server --keygen <keyId> [--out <目录>]`：离线机器生成 ed25519 密钥对，
//!   输出 keyId + 公钥 hex（进仓库/客户端）与密封私钥文件（导入 signer），
//!   私钥明文不落联网机器、不打印到 stdout。
//!
//! 轮换（双槽位）：signer 同时持有 active/previous 两个私钥槽位；
//! keyId 形如 "2026a"。客户端内嵌双公钥槽位，验签先按响应 keyId 精确匹配，
//! 无 keyId/未知 keyId 则遍历全部槽位。

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use serde_json::{json, Value};

/// 配置签名消息前缀（必须与客户端 fallback_verify 逐字一致）：
/// 对配置 JSON 原始字符串（未解析、未重排序）直接签名，前缀防止签名在
/// 其他消息类型间复用。
pub const FALLBACK_CONFIG_SIGN_PREFIX: &str = "xianyu-fallback-config-v1";

/// 配置签名消息：`xianyu-fallback-config-v1\x00{config_json}`（服务端/客户端逐字一致）。
pub fn config_signing_message(config_json: &str) -> Vec<u8> {
    format!("{FALLBACK_CONFIG_SIGN_PREFIX}\x00{config_json}").into_bytes()
}

/// 内测资格签名消息前缀（与 version.rs / 客户端 fallback_verify 逐字一致）。
pub const BETA_ACCESS_SIGN_PREFIX: &str = "xianyu-beta-access-v1";

/// 签发消息分类校验：签发服务只签两类已知格式，防止沦为任意内容签名机。
/// - 兜底配置：`xianyu-fallback-config-v1\x00{config_json}`（schema 校验）
/// - 内测资格：`xianyu-beta-access-v1\x00{device_id}\x00{0|1}\x00{0|1}\x00{exp}`
/// 返回 Ok(类别) 表示接受；Err(原因) 表示拒绝。
fn validate_signing_message(message: &[u8]) -> Result<&'static str, String> {
    let config_prefix = FALLBACK_CONFIG_SIGN_PREFIX.as_bytes();
    if message.starts_with(config_prefix) && message.get(config_prefix.len()) == Some(&0) {
        let config_part = std::str::from_utf8(&message[config_prefix.len() + 1..])
            .map_err(|_| "配置不是合法 UTF-8".to_string())?;
        validate_config_schema(config_part)?;
        return Ok("config");
    }
    let beta_prefix = BETA_ACCESS_SIGN_PREFIX.as_bytes();
    if message.starts_with(beta_prefix) && message.get(beta_prefix.len()) == Some(&0) {
        let payload = std::str::from_utf8(&message[beta_prefix.len() + 1..])
            .map_err(|_| "内测消息不是合法 UTF-8".to_string())?;
        validate_beta_payload(payload)?;
        return Ok("beta");
    }
    Err(format!(
        "消息前缀必须是 {FALLBACK_CONFIG_SIGN_PREFIX} 或 {BETA_ACCESS_SIGN_PREFIX}"
    ))
}

/// 内测资格消息载荷校验：`{device_id}\x00{allowed01}\x00{pending01}\x00{exp}`。
fn validate_beta_payload(payload: &str) -> Result<(), String> {
    let parts: Vec<&str> = payload.split('\x00').collect();
    if parts.len() != 4 {
        return Err("内测消息格式应为 device_id\\x00allowed\\x00pending\\x00exp".to_string());
    }
    let device_id = parts[0];
    if device_id.is_empty()
        || device_id.len() > 128
        || !device_id.chars().all(|c| c.is_ascii_graphic())
    {
        return Err("device_id 为空/超长/含非法字符".to_string());
    }
    if !matches!(parts[1], "0" | "1") || !matches!(parts[2], "0" | "1") {
        return Err("allowed/pending 必须为 0 或 1".to_string());
    }
    if parts[3].parse::<i64>().is_err() {
        return Err("exp 必须是 unix 秒".to_string());
    }
    Ok(())
}

/// 密封私钥目录（相对服务端工作目录，与其他 api/ 资产一致）。
pub fn keys_dir() -> PathBuf {
    Path::new("api").join("fallback_sign_keys")
}

// ==================== 密封私钥文件 ====================
// <dir>/<keyId>.key：JSON { keyId, seed, created_at }（seed 为 64 位 hex 私钥种子）。
// Unix 下写入后收紧为 0600；Windows 建议部署在受 ACL 保护的账户目录。
// <dir>/state.json：{ "active": "<keyId>", "previous": "<keyId>" | null }

#[derive(Clone)]
struct KeySlot {
    key_id: String,
    signing: SigningKey,
}

#[derive(Clone, serde::Serialize)]
struct KeysSummary {
    active: String,
    previous: Option<String>,
    available: Vec<String>,
    #[serde(rename = "activePublicKey")]
    active_public_key: String,
}

fn valid_key_id(key_id: &str) -> bool {
    !key_id.is_empty()
        && key_id.len() <= 32
        && key_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn load_seed_file(path: &Path) -> Option<(String, SigningKey)> {
    let raw = std::fs::read_to_string(path).ok()?;
    let obj = serde_json::from_str::<Value>(&raw).ok()?;
    let key_id = obj.get("keyId")?.as_str()?.to_string();
    let seed_hex = obj.get("seed")?.as_str()?;
    if !valid_key_id(&key_id) {
        return None;
    }
    let seed = seed_of_hex(seed_hex)?;
    Some((key_id, SigningKey::from_bytes(&seed)))
}

fn seed_of_hex(hex_str: &str) -> Option<[u8; 32]> {
    let trimmed = hex_str.trim();
    if trimmed.len() != 64 {
        return None;
    }
    let bytes: Vec<u8> = (0..trimmed.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&trimmed[i..i + 2], 16).ok())
        .collect();
    if bytes.len() != 32 {
        return None;
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes);
    Some(seed)
}

fn restrict_perms(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mut perm = meta.permissions();
            perm.set_mode(0o600);
            let _ = std::fs::set_permissions(path, perm);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

/// 写入密封私钥文件（keygen 与导入共用）。
pub fn write_sealed_key(dir: &Path, key_id: &str, seed: [u8; 32]) -> anyhow::Result<PathBuf> {
    if !valid_key_id(key_id) {
        anyhow::bail!("keyId 非法（仅允许字母数字-_，长度 1..=32）：{key_id}");
    }
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{key_id}.key"));
    if path.exists() {
        anyhow::bail!("密封私钥文件已存在，拒绝覆盖：{}", path.display());
    }
    let doc = json!({
        "keyId": key_id,
        "seed": hex::encode(seed),
        "createdAt": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    });
    std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
    restrict_perms(&path);
    Ok(path)
}

fn load_state_file(dir: &Path) -> Option<(String, Option<String>)> {
    let raw = std::fs::read_to_string(dir.join("state.json")).ok()?;
    let obj = serde_json::from_str::<Value>(&raw).ok()?;
    let active = obj.get("active")?.as_str()?.to_string();
    let previous = obj
        .get("previous")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Some((active, previous))
}

fn save_state_file(dir: &Path, active: &str, previous: Option<&str>) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let doc = json!({ "active": active, "previous": previous });
    std::fs::write(dir.join("state.json"), serde_json::to_string_pretty(&doc)?)?;
    restrict_perms(&dir.join("state.json"));
    Ok(())
}

struct SignerCore {
    dir: PathBuf,
    slots: Mutex<HashMap<String, SigningKey>>,
    state: Mutex<(String, Option<String>)>,
}

impl SignerCore {
    /// 启动加载：读取目录内全部 <keyId>.key 与 state.json；
    /// state 缺失且目录内恰有一把私钥时自动初始化（首次接入 signer 平滑过渡）。
    fn load(dir: PathBuf) -> anyhow::Result<SignerCore> {
        let mut slots = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("key") {
                    continue;
                }
                match load_seed_file(&path) {
                    Some((key_id, signing)) => {
                        tracing::info!("signer 已加载密封私钥槽位: {key_id}");
                        slots.insert(key_id, signing);
                    }
                    None => tracing::error!("signer 密封私钥文件损坏，已跳过: {}", path.display()),
                }
            }
        }
        if slots.is_empty() {
            anyhow::bail!(
                "签发服务未找到任何密封私钥（{}/*.key）。先用 `server --keygen <keyId>` 生成并放入该目录",
                dir.display()
            );
        }
        let state = match load_state_file(&dir) {
            Some((active, previous)) if slots.contains_key(&active) => (active, previous),
            _ => {
                if slots.len() == 1 {
                    let only = slots.keys().next().cloned().unwrap();
                    tracing::warn!("signer 缺少 state.json，自动初始化 active={only}");
                    save_state_file(&dir, &only, None)?;
                    (only, None)
                } else {
                    anyhow::bail!(
                        "signer 缺少有效的 state.json 且目录内有多把私钥，拒绝猜测 active 槽位"
                    );
                }
            }
        };
        Ok(SignerCore {
            dir,
            slots: Mutex::new(slots),
            state: Mutex::new(state),
        })
    }

    fn summary(&self) -> KeysSummary {
        let state = self.state.lock().unwrap();
        let slots = self.slots.lock().unwrap();
        let active_public_key = slots
            .get(&state.0)
            .map(|k| hex::encode(k.verifying_key().to_bytes()))
            .unwrap_or_default();
        KeysSummary {
            active: state.0.clone(),
            previous: state.1.clone(),
            available: {
                let mut ids: Vec<String> = slots.keys().cloned().collect();
                ids.sort();
                ids
            },
            active_public_key,
        }
    }

    fn sign(&self, message: &[u8]) -> Option<(String, String)> {
        let state = self.state.lock().unwrap();
        let slots = self.slots.lock().unwrap();
        let key = slots.get(&state.0)?;
        Some((hex::encode(key.sign(message).to_bytes()), state.0.clone()))
    }

    fn switch(&self, activate: &str) -> Result<KeysSummary, String> {
        let mut state = self.state.lock().unwrap();
        let slots = self.slots.lock().unwrap();
        if !slots.contains_key(activate) {
            return Err(format!("密封私钥目录中不存在槽位 {activate}"));
        }
        if state.0 == activate {
            return Ok(self.summary_unlocked(&slots, &state));
        }
        let old = state.0.clone();
        state.0 = activate.to_string();
        state.1 = Some(old);
        if let Err(e) = save_state_file(&self.dir, &state.0, state.1.as_deref()) {
            tracing::error!("signer state.json 持久化失败: {e}");
            return Err("切换已生效但 state.json 持久化失败，重启后将回退旧槽位".to_string());
        }
        Ok(self.summary_unlocked(&slots, &state))
    }

    fn summary_unlocked(&self, slots: &HashMap<String, SigningKey>, state: &(String, Option<String>)) -> KeysSummary {
        let active_public_key = slots
            .get(&state.0)
            .map(|k| hex::encode(k.verifying_key().to_bytes()))
            .unwrap_or_default();
        KeysSummary {
            active: state.0.clone(),
            previous: state.1.clone(),
            available: {
                let mut ids: Vec<String> = slots.keys().cloned().collect();
                ids.sort();
                ids
            },
            active_public_key,
        }
    }
}

// ==================== canonical 配置 schema 校验 ====================
// 签发前强校验：signer 只为结构合法的 canonical 配置签名，防止被诱导签发
// 任意内容（例如把签发能力转用于其他消息）。仅结构校验，不绑定模块白名单
// （业务白名单由主服务 save/delete 把关，新增模块无需升级 signer）。

const CONFIG_JSON_MAX_BYTES: usize = 64 * 1024;
const MAX_MODULES: usize = 64;
const MAX_LIST_LEN: usize = 256;

fn check_str_list(value: &Value, what: &str) -> Result<(), String> {
    let arr = value
        .as_array()
        .ok_or_else(|| format!("{what} 必须是字符串数组"))?;
    if arr.len() > MAX_LIST_LEN {
        return Err(format!("{what} 长度超出上限 {MAX_LIST_LEN}"));
    }
    for item in arr {
        if item.as_str().map(|s| s.is_empty()).unwrap_or(true) {
            return Err(format!("{what} 含空/非字符串元素"));
        }
    }
    Ok(())
}

pub fn validate_config_schema(config_json: &str) -> Result<(), String> {
    if config_json.len() > CONFIG_JSON_MAX_BYTES {
        return Err(format!("配置超过大小上限 {CONFIG_JSON_MAX_BYTES} 字节"));
    }
    let doc: Value = serde_json::from_str(config_json).map_err(|e| format!("配置非法 JSON: {e}"))?;
    let obj = doc.as_object().ok_or("配置顶层必须是对象")?;
    for key in obj.keys() {
        if !matches!(key.as_str(), "version" | "modules" | "expiresAt") {
            return Err(format!("配置顶层含未知字段: {key}"));
        }
    }
    let version = obj
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or("配置 version 必须是非负整数")?;
    if version < 1 {
        return Err("配置 version 必须 >= 1".to_string());
    }
    if let Some(expires_at) = obj.get("expiresAt") {
        if expires_at.as_u64().is_none() {
            return Err("expiresAt 必须是非负整数（unix 秒）".to_string());
        }
    }
    let modules = obj
        .get("modules")
        .and_then(|v| v.as_object())
        .ok_or("配置 modules 必须是对象")?;
    if modules.len() > MAX_MODULES {
        return Err(format!("modules 数量超出上限 {MAX_MODULES}"));
    }
    for (key, entry) in modules {
        let entry = entry.as_object().ok_or_else(|| format!("模块 {key} 必须是对象"))?;
        for field in entry.keys() {
            if !matches!(field.as_str(), "version" | "disabledMethods" | "params") {
                return Err(format!("模块 {key} 含未知字段: {field}"));
            }
        }
        if let Some(v) = entry.get("version") {
            if v.as_u64().is_none() {
                return Err(format!("模块 {key} version 必须是非负整数"));
            }
        }
        if let Some(v) = entry.get("disabledMethods") {
            check_str_list(v, &format!("模块 {key} disabledMethods"))?;
        }
        if let Some(params) = entry.get("params") {
            let params = params.as_object().ok_or_else(|| format!("模块 {key} params 必须是对象"))?;
            for field in params.keys() {
                if field != "sources" {
                    return Err(format!("模块 {key} params 含未知字段: {field}"));
                }
            }
            if let Some(sources) = params.get("sources") {
                check_str_list(sources, &format!("模块 {key} params.sources"))?;
            }
        }
    }
    Ok(())
}

// ==================== 限流 ====================

struct IpRateLimiter {
    window: Mutex<HashMap<IpAddr, (std::time::Instant, u32)>>,
    max_per_window: u32,
}

impl IpRateLimiter {
    fn new(max_per_window: u32) -> Self {
        Self {
            window: Mutex::new(HashMap::new()),
            max_per_window,
        }
    }

    fn allow(&self, ip: IpAddr) -> bool {
        const WINDOW: std::time::Duration = std::time::Duration::from_secs(60);
        let mut map = self.window.lock().unwrap();
        if map.len() > 10_000 {
            let now = std::time::Instant::now();
            map.retain(|_, (start, _)| now.duration_since(*start) < WINDOW);
        }
        let now = std::time::Instant::now();
        let entry = map.entry(ip).or_insert((now, 0));
        if now.duration_since(entry.0) >= WINDOW {
            *entry = (now, 0);
        }
        entry.1 += 1;
        entry.1 <= self.max_per_window
    }
}

// ==================== 审计日志 ====================

fn audit(action: &str, ip: &str, detail: Value) {
    let line = json!({
        "ts": chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        "ip": ip,
        "action": action,
        "detail": detail,
    });
    let dir = Path::new("logs");
    let _ = std::fs::create_dir_all(dir);
    let path = dir.join("fallback_signer_audit.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        use std::io::Write;
        let _ = writeln!(file, "{line}");
    }
}

// ==================== 签发服务（--signer） ====================

const SIGN_BODY_MAX_BYTES: usize = 256 * 1024;

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(json!({ "ok": false, "error": "unauthorized" })),
    )
        .into_response()
}

fn check_token(headers: &HeaderMap, token: &str) -> bool {
    let provided = headers
        .get("x-signer-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // 常量时间比较，避免令牌逐字节猜测
    if provided.len() != token.len() {
        return false;
    }
    provided
        .bytes()
        .zip(token.bytes())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[derive(Clone)]
struct SignerState {
    core: Arc<SignerCore>,
    token: Arc<String>,
    limiter: Arc<IpRateLimiter>,
}

async fn handle_sign(
    State(st): State<SignerState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    req: Request<Body>,
) -> Response {
    let ip = peer.ip().to_string();
    if !check_token(&headers, &st.token) {
        audit("sign_rejected", &ip, json!({ "reason": "unauthorized" }));
        return unauthorized();
    }
    if !st.limiter.allow(peer.ip()) {
        audit("sign_rejected", &ip, json!({ "reason": "rate_limited" }));
        return (
            StatusCode::TOO_MANY_REQUESTS,
            axum::Json(json!({ "ok": false, "error": "rate limited" })),
        )
            .into_response();
    }
    let body = match axum::body::to_bytes(req.into_body(), SIGN_BODY_MAX_BYTES).await {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::PAYLOAD_TOO_LARGE,
                axum::Json(json!({ "ok": false, "error": "body too large" })),
            )
                .into_response()
        }
    };
    let parsed: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "ok": false, "error": "invalid json" })),
            )
                .into_response()
        }
    };
    let Some(message_hex) = parsed.get("message_hex").and_then(|v| v.as_str()) else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({ "ok": false, "error": "missing message_hex" })),
        )
            .into_response();
    };
    let message = match hex::decode(message_hex.trim()) {
        Ok(m) => m,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "ok": false, "error": "message_hex 非法 hex" })),
            )
                .into_response()
        }
    };
    let kind = match validate_signing_message(&message) {
        Ok(kind) => kind,
        Err(reason) => {
            audit("sign_rejected", &ip, json!({ "reason": "bad_message", "error": reason, "len": message.len() }));
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "ok": false, "error": reason })),
            )
                .into_response();
        }
    };
    let Some((signature, key_id)) = st.core.sign(&message) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({ "ok": false, "error": "active 槽位不可用" })),
        )
            .into_response();
    };
    let mut audit_extra = json!({ "kind": kind, "keyId": key_id, "len": message.len() });
    if kind == "config" {
        let config_prefix_len = FALLBACK_CONFIG_SIGN_PREFIX.as_bytes().len();
        let config_version = serde_json::from_str::<Value>(
            std::str::from_utf8(&message[config_prefix_len + 1..]).unwrap_or(""),
        )
        .ok()
        .and_then(|v| v.get("version").and_then(|x| x.as_u64()))
        .unwrap_or(0);
        audit_extra["configVersion"] = json!(config_version);
    }
    audit("sign", &ip, audit_extra);
    (
        StatusCode::OK,
        axum::Json(json!({ "ok": true, "signature": signature, "keyId": key_id })),
    )
        .into_response()
}

async fn handle_keys(
    State(st): State<SignerState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
) -> Response {
    if !check_token(&headers, &st.token) {
        return unauthorized();
    }
    let _ = peer;
    (
        StatusCode::OK,
        axum::Json(serde_json::to_value(st.core.summary()).unwrap_or(Value::Null)),
    )
        .into_response()
}

async fn handle_switch(
    State(st): State<SignerState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    req: Request<Body>,
) -> Response {
    let ip = peer.ip().to_string();
    if !check_token(&headers, &st.token) {
        return unauthorized();
    }
    let body = match axum::body::to_bytes(req.into_body(), 64 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "ok": false, "error": "invalid body" })),
            )
                .into_response()
        }
    };
    let parsed: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "ok": false, "error": "invalid json" })),
            )
                .into_response()
        }
    };
    let Some(activate) = parsed.get("activate").and_then(|v| v.as_str()) else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({ "ok": false, "error": "missing activate" })),
        )
            .into_response();
    };
    match st.core.switch(activate) {
        Ok(summary) => {
            audit("switch", &ip, json!({ "activate": activate }));
            (
                StatusCode::OK,
                axum::Json(serde_json::to_value(summary).unwrap_or(Value::Null)),
            )
                .into_response()
        }
        Err(e) => {
            audit("switch_rejected", &ip, json!({ "activate": activate, "error": e }));
            (
                StatusCode::BAD_REQUEST,
                axum::Json(json!({ "ok": false, "error": e })),
            )
                .into_response()
        }
    }
}

async fn handle_health() -> Response {
    (
        StatusCode::OK,
        axum::Json(json!({ "ok": true, "service": "fallback-signer" })),
    )
        .into_response()
}

/// `server --signer`：启动签发服务（仅回环监听 + 共享令牌）。
pub async fn run_signer_service() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let token = std::env::var("FALLBACK_SIGNER_TOKEN")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(anyhow::Ok)
        .unwrap_or_else(|| {
            anyhow::bail!(
                "签发服务需要共享令牌：请设置环境变量 FALLBACK_SIGNER_TOKEN（与主服务 fallback.signer.token 一致，如 openssl rand -hex 32 生成）"
            )
        })?;
    if token.len() < 32 {
        tracing::warn!("安全警告：FALLBACK_SIGNER_TOKEN 长度不足 32，建议使用 openssl rand -hex 32 生成");
    }
    let addr: std::net::SocketAddr = std::env::var("FALLBACK_SIGNER_ADDR")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "127.0.0.1:8791".to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("FALLBACK_SIGNER_ADDR 非法: {e}"))?;
    if !addr.ip().is_loopback() {
        anyhow::bail!("签发服务仅允许监听回环地址（当前 {addr}）：私钥进程绝不可暴露到非回环网卡");
    }
    let core = Arc::new(SignerCore::load(keys_dir())?);
    let summary = core.summary();
    tracing::info!(
        "签发服务就绪: active={} previous={:?} available={:?}",
        summary.active,
        summary.previous,
        summary.available
    );
    let state = SignerState {
        core,
        token: Arc::new(token),
        limiter: Arc::new(IpRateLimiter::new(120)),
    };
    let app = Router::new()
        .route("/internal/sign", post(handle_sign))
        .route("/internal/keys", get(handle_keys))
        .route("/internal/switch", post(handle_switch))
        .route("/health", get(handle_health))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("fallback signer listening on {addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

// ==================== keygen（--keygen） ====================

/// `server --keygen <keyId> [--out <目录>]`：离线生成 ed25519 密钥对。
/// 输出 keyId + 公钥 hex（填入客户端双公钥槽位/服务端测试），密封私钥文件
/// 写入 <out>（默认 api/fallback_sign_keys/）供 signer 加载；私钥明文不打印。
pub fn run_keygen(key_id: &str, out_dir: Option<String>) -> anyhow::Result<()> {
    if !valid_key_id(key_id) {
        anyhow::bail!("keyId 非法（仅允许字母数字-_，长度 1..=32）：{key_id}");
    }
    let dir = out_dir
        .map(PathBuf::from)
        .unwrap_or_else(keys_dir);
    let mut seed = [0u8; 32];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut seed);
    let signing = SigningKey::from_bytes(&seed);
    let path = write_sealed_key(&dir, key_id, seed)?;
    let public_hex = hex::encode(signing.verifying_key().to_bytes());
    println!("已生成签名密钥槽位：");
    println!("  keyId        : {key_id}");
    println!("  公钥 hex     : {public_hex}");
    println!("                 （填入三端客户端 FALLBACK_VERIFY_KEYS 预发布槽位，并同步服务端测试常量）");
    println!("  密封私钥文件 : {}", path.display());
    println!("                 （拷贝到 signer 进程的 api/fallback_sign_keys/ 目录后重启 signer 生效）");
    println!();
    println!("注意：请在离线/可信机器上执行本命令；私钥文件权限已收紧（Unix 0600），");
    println!("切勿将密封私钥提交进仓库或上传到任何联网存储。验证公钥：");
    if let Ok(vk) = VerifyingKey::from_bytes(&signing.verifying_key().to_bytes()) {
        let probe = b"xianyu-keygen-probe";
        let sig = signing.sign(probe);
        if vk.verify(probe, &sig).is_ok() {
            println!("  自检通过（签名/验签回路正常）");
        }
    }
    Ok(())
}

// ==================== 主服务侧签发客户端 ====================
// 主服务（B 档）自身不持有私钥，仅经回环调用签发服务。请求失败一律 fail
// closed：保存配置/内测签发返回错误，绝不落无签名或错误签名的配置。

const SIGNER_CALL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

fn signer_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(SIGNER_CALL_TIMEOUT)
        .build()
        .unwrap_or_default()
}

async fn signer_call_json(
    section: &crate::config::SignerSection,
    method: reqwest::Method,
    path: &str,
    body: Option<&Value>,
) -> Result<Value, String> {
    let url = format!("{}{}", section.effective_url().trim_end_matches('/'), path);
    let mut req = signer_http_client()
        .request(method, &url)
        .header("x-signer-token", section.effective_token());
    if let Some(b) = body {
        req = req.json(b);
    }
    let resp = req.send().await.map_err(|e| format!("签发服务不可达（{url}）: {e}"))?;
    let status = resp.status();
    let value: Value = resp.json().await.map_err(|e| format!("签发服务响应非法: {e}"))?;
    if !status.is_success() {
        let err = value
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Err(format!("签发服务返回 {status}: {err}"));
    }
    Ok(value)
}

/// 经签发服务对消息签名。成功返回 (signature_hex, keyId)。
pub(crate) async fn client_sign(
    section: &crate::config::SignerSection,
    message: &[u8],
) -> Result<(String, String), String> {
    let resp = signer_call_json(
        section,
        reqwest::Method::POST,
        "/internal/sign",
        Some(&json!({ "message_hex": hex::encode(message) })),
    )
    .await?;
    let signature = resp
        .get("signature")
        .and_then(|v| v.as_str())
        .ok_or("签发服务响应缺少 signature")?
        .to_string();
    let key_id = resp
        .get("keyId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok((signature, key_id))
}

/// 查询签发服务槽位状态（active/previous/available/activePublicKey）。
pub(crate) async fn client_keys(
    section: &crate::config::SignerSection,
) -> Result<Value, String> {
    signer_call_json(section, reqwest::Method::GET, "/internal/keys", None).await
}

/// 切换 active 槽位（旧 active 移入 previous）。
pub(crate) async fn client_switch(
    section: &crate::config::SignerSection,
    activate: &str,
) -> Result<Value, String> {
    signer_call_json(
        section,
        reqwest::Method::POST,
        "/internal/switch",
        Some(&json!({ "activate": activate })),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_schema_accepts_canonical_shape() {
        assert!(validate_config_schema(r#"{"version":1,"modules":{"lx_lyric":{"version":1,"disabledMethods":["fetchLyric"]}}}"#).is_ok());
        assert!(validate_config_schema(
            r#"{"version":2,"expiresAt":1767225600,"modules":{"lx_toplist":{"version":1,"disabledMethods":["getTopLists"],"params":{"sources":["wy","kg"]}}}}"#
        )
        .is_ok());
        assert!(validate_config_schema(r#"{"version":1,"modules":{}}"#).is_ok());
    }

    #[test]
    fn config_schema_rejects_unknown_or_malformed() {
        // 非法 JSON
        assert!(validate_config_schema("{").is_err());
        // 顶层未知字段
        assert!(validate_config_schema(r#"{"version":1,"modules":{},"evil":"x"}"#).is_err());
        // version 非法
        assert!(validate_config_schema(r#"{"version":0,"modules":{}}"#).is_err());
        assert!(validate_config_schema(r#"{"version":"1","modules":{}}"#).is_err());
        // modules 缺失
        assert!(validate_config_schema(r#"{"version":1}"#).is_err());
        // 模块未知字段
        assert!(validate_config_schema(
            r#"{"version":1,"modules":{"m":{"version":1,"code":"alert(1)"}}}"#
        )
        .is_err());
        // params 未知字段
        assert!(validate_config_schema(
            r#"{"version":1,"modules":{"m":{"version":1,"params":{"code":"x"}}}}"#
        )
        .is_err());
        // disabledMethods 非数组/含空串
        assert!(validate_config_schema(
            r#"{"version":1,"modules":{"m":{"version":1,"disabledMethods":[""]}}}"#
        )
        .is_err());
        assert!(validate_config_schema(
            r#"{"version":1,"modules":{"m":{"version":1,"disabledMethods":"fetchLyric"}}}"#
        )
        .is_err());
        // expiresAt 非整数
        assert!(validate_config_schema(r#"{"version":1,"expiresAt":"soon","modules":{}}"#).is_err());
    }

    #[test]
    fn signing_message_round_trip() {
        let msg = config_signing_message(r#"{"version":1}"#);
        assert_eq!(msg, b"xianyu-fallback-config-v1\x00{\"version\":1}");
        assert!(msg.starts_with(FALLBACK_CONFIG_SIGN_PREFIX.as_bytes()));
    }

    #[test]
    fn signing_message_accepts_beta_and_config() {
        // 内测消息（与服务端 check_beta_access / 客户端 beta_access_message 同构）
        let beta = b"xianyu-beta-access-v1\x00dev-abc\x001\x000\x001799999999".to_vec();
        assert_eq!(validate_signing_message(&beta), Ok("beta"));
        // 配置消息
        let config = config_signing_message(r#"{"version":1,"modules":{}}"#);
        assert_eq!(validate_signing_message(&config), Ok("config"));
        // 未知前缀 / 缺分隔符 / 混用前缀
        assert!(validate_signing_message(b"random-bytes").is_err());
        assert!(validate_signing_message(b"xianyu-beta-access-v1noseparator").is_err());
        let crossed = {
            let mut m = b"xianyu-beta-access-v1\x00".to_vec();
            m.extend_from_slice(br#"{"version":1,"modules":{}}"#);
            m
        };
        // beta 前缀下挂 JSON 配置载荷 → 载荷格式校验必须拒绝
        assert!(validate_signing_message(&crossed).is_err());
    }

    #[test]
    fn beta_payload_validation() {
        assert!(validate_beta_payload("dev1\x001\x000\x00123").is_ok());
        assert!(validate_beta_payload("\x001\x000\x00123").is_err()); // device_id 空
        assert!(validate_beta_payload(&format!("{}\x001\x000\x00123", "d".repeat(129))).is_err());
        assert!(validate_beta_payload("dev1\x00x\x000\x00123").is_err()); // allowed 非 0/1
        assert!(validate_beta_payload("dev1\x001\x000\x00soon").is_err()); // exp 非数字
        assert!(validate_beta_payload("dev1\x001\x000").is_err()); // 段数不足
    }

    #[test]
    fn key_id_validation() {
        assert!(valid_key_id("2026a"));
        assert!(valid_key_id("prod-2027_1"));
        assert!(!valid_key_id(""));
        assert!(!valid_key_id("带/斜杠"));
        assert!(!valid_key_id(&"a".repeat(33)));
    }

    #[test]
    fn schema_rejects_oversize() {
        let big = format!(r#"{{"version":1,"modules":{{"m":{{"version":1,"disabledMethods":["{}"]}}}}}}"#, "x".repeat(70 * 1024));
        assert!(validate_config_schema(&big).is_err());
    }
}
