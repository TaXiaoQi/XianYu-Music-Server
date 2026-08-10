use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{log_operation, ok, AdminCtx};
use crate::handlers::helpers::{parse_body, str_of};

/// 读取 server_settings 单个 key（空字符串视为未设置）
async fn read_setting(pool: &MySqlPool, key: &str) -> Option<String> {
    sqlx::query("SELECT setting_value FROM server_settings WHERE setting_key = ? LIMIT 1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get::<Option<String>, _>(0).ok().flatten())
        .filter(|s| !s.trim().is_empty())
}

/// 写入或更新 server_settings 值
async fn upsert_setting(pool: &MySqlPool, key: &str, value: &str, desc: &str) {
    let _ = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), description = VALUES(description)",
    )
    .bind(key)
    .bind(value)
    .bind(desc)
    .execute(pool)
    .await;
}

fn normalize_provider(provider: &str) -> String {
    match provider.trim().to_lowercase().as_str() {
        "turnstile" => "turnstile".to_string(),
        "hcaptcha" => "hcaptcha".to_string(),
        "off" | "none" | "disabled" | "" => "off".to_string(),
        _ => "off".to_string(),
    }
}

fn fallback_secret(provider: &str, ctx: &AdminCtx) -> String {
    if !ctx.config.captcha_secret.trim().is_empty() {
        return ctx.config.captcha_secret.clone();
    }
    match provider {
        "turnstile" => ctx.config.turnstile_secret.clone(),
        "hcaptcha" => ctx.config.hcaptcha_secret.clone(),
        _ => String::new(),
    }
}

/// 获取通用人机验证配置（secret 脱敏）
pub async fn get_captcha_config(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let old_turnstile_enabled = read_setting(pool, "turnstile_enabled").await;
    let old_turnstile_site_key = read_setting(pool, "turnstile_site_key").await;
    let old_turnstile_secret = read_setting(pool, "turnstile_secret").await;
    let new_enabled = read_setting(pool, "captcha_enabled").await;
    let new_site_key = read_setting(pool, "captcha_site_key").await;
    let new_secret = read_setting(pool, "captcha_secret").await;

    let provider = match read_setting(pool, "captcha_provider").await {
        Some(s) => normalize_provider(&s),
        None => old_turnstile_enabled
            .as_deref()
            .filter(|s| *s == "1" || s.eq_ignore_ascii_case("true"))
            .map(|_| "turnstile".to_string())
            .unwrap_or_else(|| "turnstile".to_string()),
    };
    let should_use_old_turnstile = provider == "turnstile"
        && new_site_key.is_none()
        && new_secret.is_none()
        && old_turnstile_site_key.is_some()
        && old_turnstile_secret.is_some();

    let selected_enabled = if should_use_old_turnstile {
        old_turnstile_enabled.clone()
    } else {
        new_enabled.clone()
    };
    let enabled_str = selected_enabled.unwrap_or_else(|| "0".to_string());
    let enabled = enabled_str == "1" || enabled_str.eq_ignore_ascii_case("true");

    let site_key = new_site_key
        .or_else(|| if provider == "turnstile" { old_turnstile_site_key.clone() } else { None })
        .unwrap_or_default();

    let secret = new_secret
        .or_else(|| if provider == "turnstile" { old_turnstile_secret.clone() } else { None })
        .unwrap_or_else(|| fallback_secret(&provider, ctx));
    let has_secret = !secret.trim().is_empty();

    ok("ok", json!({
        "enabled": enabled,
        "provider": provider,
        "site_key": site_key,
        "has_secret": has_secret,
        "secret": if has_secret { "********".to_string() } else { String::new() },
    }))
}

/// 保存通用人机验证配置（secret 为空或为占位符时保留原值）
pub async fn save_captcha_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);

    let enabled_raw = str_of(&data, "enabled").trim().to_string();
    let enabled = enabled_raw == "1" || enabled_raw.eq_ignore_ascii_case("true");
    let enabled_val = if enabled { "1" } else { "0" };

    let provider_raw = str_of(&data, "provider");
    let old_provider = read_setting(pool, "captcha_provider")
        .await
        .map(|s| normalize_provider(&s))
        .unwrap_or_else(|| "turnstile".to_string());
    let provider = if provider_raw.trim().is_empty() {
        "turnstile".to_string()
    } else {
        normalize_provider(&provider_raw)
    };
    let site_key = str_of(&data, "site_key").trim().to_string();
    let secret = str_of(&data, "secret").trim().to_string();

    upsert_setting(pool, "captcha_enabled", enabled_val, "是否启用人机验证：1=启用，0=关闭").await;
    upsert_setting(pool, "captcha_provider", &provider, "人机验证服务商：turnstile、hcaptcha、off").await;
    upsert_setting(pool, "captcha_site_key", &site_key, "人机验证 Site Key（前端展示用）").await;

    // secret 为空或为占位符时保留原值
    if !secret.is_empty() && secret != "********" {
        upsert_setting(pool, "captcha_secret", &secret, "人机验证 Secret Key（后端校验用，留空则回退环境变量）").await;
    } else if old_provider != provider {
        upsert_setting(pool, "captcha_secret", "", "人机验证 Secret Key（后端校验用，留空则回退环境变量）").await;
    }

    log_operation(
        pool,
        ctx,
        "save_captcha_config",
        "captcha",
        &format!("启用:{} 服务商:{} Site Key:{}", enabled_val, provider, if site_key.is_empty() { "未设置" } else { "已设置" }),
    )
    .await;

    ok("人机验证配置已保存", Value::Null)
}

/// 兼容旧后台 action 名称。
pub async fn get_turnstile_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    get_captcha_config(body, ctx, pool).await
}

/// 兼容旧后台 action 名称。
pub async fn save_turnstile_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    save_captcha_config(body, ctx, pool).await
}
