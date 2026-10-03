use super::*;

#[derive(Debug, Deserialize)]
struct CaptchaVerifyResponse {
    success: bool,
}

pub struct CaptchaConfig {
    pub enabled: bool,
    pub provider: String,
    pub site_key: String,
    pub secret: String,
}

fn normalize_captcha_provider(provider: &str) -> String {
    match provider.trim().to_lowercase().as_str() {
        "turnstile" => "turnstile".to_string(),
        "hcaptcha" => "hcaptcha".to_string(),
        "off" | "none" | "disabled" | "" => "off".to_string(),
        _ => "off".to_string(),
    }
}

fn fallback_captcha_secret(provider: &str, fallback: &crate::config::Config) -> String {
    if !fallback.captcha_secret.trim().is_empty() {
        return fallback.captcha_secret.clone();
    }
    match provider {
        "turnstile" => fallback.turnstile_secret.clone(),
        "hcaptcha" => fallback.hcaptcha_secret.clone(),
        _ => String::new(),
    }
}

pub async fn load_captcha_config(
    pool: &MySqlPool,
    fallback: &crate::config::Config,
) -> CaptchaConfig {
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

    let old_turnstile_enabled = read_setting(pool, "turnstile_enabled").await;
    let old_turnstile_site_key = read_setting(pool, "turnstile_site_key").await;
    let old_turnstile_secret = read_setting(pool, "turnstile_secret").await;
    let new_enabled = read_setting(pool, "captcha_enabled").await;
    let new_site_key = read_setting(pool, "captcha_site_key").await;
    let new_secret = read_setting(pool, "captcha_secret").await;
    let provider = match read_setting(pool, "captcha_provider").await {
        Some(s) => normalize_captcha_provider(&s),
        None => old_turnstile_enabled
            .as_deref()
            .filter(|s| *s == "1" || s.eq_ignore_ascii_case("true"))
            .map(|_| "turnstile".to_string())
            .unwrap_or_else(|| "off".to_string()),
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
    let enabled_str = selected_enabled
        .unwrap_or_else(|| if provider == "off" { "0".to_string() } else { "1".to_string() });
    let enabled = enabled_str == "1" || enabled_str.eq_ignore_ascii_case("true");

    let site_key = new_site_key
        .or_else(|| if provider == "turnstile" { old_turnstile_site_key.clone() } else { None })
        .unwrap_or_default();

    let secret = new_secret
        .or_else(|| if provider == "turnstile" { old_turnstile_secret.clone() } else { None })
        .unwrap_or_else(|| fallback_captcha_secret(&provider, fallback));

    CaptchaConfig {
        enabled: enabled && provider != "off",
        provider,
        site_key,
        secret,
    }
}

pub async fn verify_captcha_token(config: &CaptchaConfig, token: &str, remote_ip: &str) -> Result<bool, String> {
    if !config.enabled || config.secret.trim().is_empty() {
        return Ok(true);
    }
    if token.trim().is_empty() {
        return Ok(false);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| format!("人机验证客户端构建失败: {e}"))?;

    let mut params = vec![
        ("secret", config.secret.trim().to_string()),
        ("response", token.trim().to_string()),
    ];
    if !remote_ip.trim().is_empty() {
        params.push(("remoteip", remote_ip.trim().to_string()));
    }

    let verify_url = match config.provider.as_str() {
        "turnstile" => "https://challenges.cloudflare.com/turnstile/v0/siteverify",
        "hcaptcha" => "https://hcaptcha.com/siteverify",
        _ => return Ok(true),
    };

    let res = client
        .post(verify_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("人机验证请求失败: {e}"))?;

    let body = res
        .json::<CaptchaVerifyResponse>()
        .await
        .map_err(|e| format!("人机验证响应解析失败: {e}"))?;

    Ok(body.success)
}

pub async fn get_captcha_config(_body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let captcha_config = load_captcha_config(pool, &ctx.config).await;
    let enabled = captcha_config.enabled
        && !captcha_config.secret.trim().is_empty()
        && !captcha_config.site_key.trim().is_empty();
    ctx.ok("ok", json!({
        "enabled": enabled,
        "provider": captcha_config.provider,
        "site_key": captcha_config.site_key,
    }))
}

pub async fn get_turnstile_config(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    get_captcha_config(body, ctx, pool).await
}
