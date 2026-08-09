use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::MySqlPool;
use std::time::Duration;

const SETTING_KEY: &str = "audit_external_config";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExternalConfig {
    pub enabled: bool,
    pub provider: String,
    pub endpoint: String,
    pub api_key: String,
    pub nickname_enabled: bool,
    pub avatar_enabled: bool,
    pub wallpaper_enabled: bool,
    pub timeout_ms: u64,
    pub fail_to_manual: bool,
}

impl Default for AuditExternalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "generic".to_string(),
            endpoint: String::new(),
            api_key: String::new(),
            nickname_enabled: true,
            avatar_enabled: true,
            wallpaper_enabled: true,
            timeout_ms: 5000,
            fail_to_manual: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditDecision {
    Pass,
    Reject,
    Manual,
}

#[derive(Debug, Clone)]
pub struct AuditResult {
    pub decision: AuditDecision,
    pub reason: String,
    pub provider: String,
}

impl AuditResult {
    fn manual(reason: impl Into<String>) -> Self {
        Self {
            decision: AuditDecision::Manual,
            reason: reason.into(),
            provider: "manual".to_string(),
        }
    }

    fn pass(reason: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            decision: AuditDecision::Pass,
            reason: reason.into(),
            provider: provider.into(),
        }
    }

    fn reject(reason: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            decision: AuditDecision::Reject,
            reason: reason.into(),
            provider: provider.into(),
        }
    }
}

pub async fn ensure_audit_setting(pool: &MySqlPool) {
    let value = serde_json::to_string(&AuditExternalConfig::default()).unwrap_or_else(|_| "{}".to_string());
    let _ = sqlx::query(
        "INSERT IGNORE INTO server_settings (setting_key, setting_value, description) VALUES (?, ?, ?)",
    )
    .bind(SETTING_KEY)
    .bind(value)
    .bind("外部内容审核配置：支持昵称、头像、壁纸先机审，无法判断再人工审核")
    .execute(pool)
    .await;
}

pub async fn load_config(pool: &MySqlPool) -> AuditExternalConfig {
    ensure_audit_setting(pool).await;
    let raw = sqlx::query_scalar::<_, Option<String>>(
        "SELECT setting_value FROM server_settings WHERE setting_key = ? LIMIT 1",
    )
    .bind(SETTING_KEY)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten();

    raw.and_then(|s| serde_json::from_str::<AuditExternalConfig>(&s).ok())
        .unwrap_or_default()
}

pub async fn save_config(pool: &MySqlPool, cfg: &AuditExternalConfig) -> Result<(), sqlx::Error> {
    ensure_audit_setting(pool).await;
    let value = serde_json::to_string(cfg).unwrap_or_else(|_| "{}".to_string());
    sqlx::query(
        "UPDATE server_settings SET setting_value = ? WHERE setting_key = ?",
    )
    .bind(value)
    .bind(SETTING_KEY)
    .execute(pool)
    .await?;
    Ok(())
}

fn module_enabled(cfg: &AuditExternalConfig, scene: &str) -> bool {
    match scene {
        "nickname" => cfg.nickname_enabled,
        "avatar" => cfg.avatar_enabled,
        "wallpaper" => cfg.wallpaper_enabled,
        _ => false,
    }
}

async fn call_external(cfg: &AuditExternalConfig, payload: Value) -> AuditResult {
    if cfg.endpoint.trim().is_empty() {
        return AuditResult::manual("外部审核地址为空");
    }

    let timeout = Duration::from_millis(cfg.timeout_ms.clamp(1000, 30000));
    let client = match reqwest::Client::builder().timeout(timeout).build() {
        Ok(c) => c,
        Err(e) => return AuditResult::manual(format!("审核客户端创建失败: {}", e)),
    };

    let mut req = client
        .post(cfg.endpoint.trim())
        .header("Content-Type", "application/json")
        .header("X-Audit-Provider", cfg.provider.trim())
        .json(&payload);

    if !cfg.api_key.trim().is_empty() {
        req = req.bearer_auth(cfg.api_key.trim());
    }

    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => return AuditResult::manual(format!("外部审核请求失败: {}", e)),
    };
    if !resp.status().is_success() {
        return AuditResult::manual(format!("外部审核 HTTP {}", resp.status()));
    }

    let body: Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return AuditResult::manual(format!("外部审核响应解析失败: {}", e)),
    };

    let decision = body
        .get("decision")
        .or_else(|| body.get("suggestion"))
        .or_else(|| body.get("result"))
        .and_then(|v| v.as_str())
        .unwrap_or("review")
        .to_ascii_lowercase();
    let reason = body
        .get("reason")
        .or_else(|| body.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let provider = body
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or(cfg.provider.as_str())
        .to_string();

    match decision.as_str() {
        "pass" | "approve" | "approved" | "normal" | "ok" => AuditResult::pass(reason, provider),
        "reject" | "rejected" | "block" | "blocked" | "deny" => AuditResult::reject(reason, provider),
        _ => AuditResult::manual(if reason.is_empty() { "外部审核建议人工复核".to_string() } else { reason }),
    }
}

pub async fn audit_text(pool: &MySqlPool, scene: &str, text: &str, meta: Value) -> AuditResult {
    let cfg = load_config(pool).await;
    if !cfg.enabled || !module_enabled(&cfg, scene) {
        return AuditResult::manual("外部审核未启用");
    }
    call_external(&cfg, json!({
        "scene": scene,
        "content_type": "text",
        "text": text,
        "meta": meta,
    }))
    .await
}

pub async fn audit_image(pool: &MySqlPool, scene: &str, image_data: &str, meta: Value) -> AuditResult {
    let cfg = load_config(pool).await;
    if !cfg.enabled || !module_enabled(&cfg, scene) {
        return AuditResult::manual("外部审核未启用");
    }
    call_external(&cfg, json!({
        "scene": scene,
        "content_type": "image",
        "image": image_data,
        "meta": meta,
    }))
    .await
}
