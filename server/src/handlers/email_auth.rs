use axum::response::Response;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{parse_body, random_int, str_of};
use crate::response::ReqCtx;

/// JWT 载荷：邮箱注册用户身份
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailClaims {
    pub sub: i64,
    pub email: String,
    pub exp: usize,
}

/// 签发邮箱用户 JWT（7 天时效）
fn sign_email_token(config: &crate::config::Config, user_id: i64, email: &str) -> String {
    let exp = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + 7 * 86400) as usize;
    let claims = EmailClaims {
        sub: user_id,
        email: email.to_string(),
        exp,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .unwrap_or_default()
}

/// 解析邮箱用户 JWT
pub fn verify_email_token(config: &crate::config::Config, token: &str) -> Option<EmailClaims> {
    jsonwebtoken::decode::<EmailClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .ok()
    .map(|d| d.claims)
}

fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() || !email.contains('@') || email.contains(' ') {
        return false;
    }
    let mut parts = email.split('@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    let rest = parts.next();
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && rest.is_none()
}

#[derive(Debug, Deserialize)]
struct CaptchaVerifyResponse {
    success: bool,
}

/// 人机验证运行时配置（优先从数据库读取，兼容旧 Turnstile 配置，最后回退到环境变量）
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

/// 从 `server_settings` 表读取通用人机验证配置，兼容旧 `turnstile_*` 配置。
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

/// 校验通用人机验证。未启用或未配置密钥时跳过。
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

/// 公开接口：获取人机验证前端配置（仅返回 enabled、provider 和 site_key，不返回 secret）
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

/// 兼容旧前端 action 名称。
pub async fn get_turnstile_config(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    get_captcha_config(body, ctx, pool).await
}

/// SMTP 账号池条目。内置邮箱机可配置多个发件账号，发送时按顺序轮换。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpAccount {
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub remark: String,
}

fn default_smtp_port() -> u16 {
    465
}

fn default_true() -> bool {
    true
}

impl SmtpAccount {
    fn is_usable(&self) -> bool {
        // host 为空时会在发送前按发件邮箱域名自动识别，因此不强制要求 host
        self.enabled
            && !self.sender.trim().is_empty()
            && !self.username.trim().is_empty()
            && !self.password.trim().is_empty()
    }
}

/// 根据发件邮箱域名自动识别常见邮箱服务商的 SMTP 服务器地址和端口。
/// 返回 (host, port)。未命中的域名回退为 `smtp.<domain>:465`，无法解析时 host 为空。
fn resolve_smtp_endpoint(sender: &str) -> (String, u16) {
    let domain = sender
        .split('@')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_end_matches('.')
        .to_lowercase();
    let (host, port): (&str, u16) = match domain.as_str() {
        // QQ 系
        "qq.com" | "vip.qq.com" | "foxmail.com" => ("smtp.qq.com", 465),
        "exmail.qq.com" => ("smtp.exmail.qq.com", 465),
        // 网易系
        "163.com" => ("smtp.163.com", 465),
        "126.com" => ("smtp.126.com", 465),
        "yeah.net" => ("smtp.yeah.net", 465),
        "188.com" => ("smtp.188.com", 465),
        "qiye.163.com" => ("smtp.qiye.163.com", 465),
        // 其他国内邮箱
        "139.com" => ("smtp.139.com", 465),
        "21cn.com" => ("smtp.21cn.com", 465),
        "sohu.com" => ("smtp.sohu.com", 465),
        "sina.com" | "vip.sina.com" => ("smtp.sina.com", 465),
        "aliyun.com" | "aliyunmail.com" => ("smtp.aliyun.com", 465),
        "qiye.aliyun.com" => ("smtp.qiye.aliyun.com", 465),
        "189.cn" => ("smtp.189.cn", 465),
        "263.net" | "x263.net" => ("smtp.263.net", 465),
        // 海外邮箱
        "gmail.com" => ("smtp.gmail.com", 465),
        "outlook.com" | "hotmail.com" | "hotmail.co.uk" | "live.com" | "msn.com" => {
            ("smtp.office365.com", 587)
        }
        "yandex.com" | "yandex.ru" => ("smtp.yandex.ru", 465),
        "zoho.com" | "zohomail.com" => ("smtp.zoho.com", 465),
        _ => {
            if domain.is_empty() {
                return (String::new(), 465);
            }
            return (format!("smtp.{}", domain), 465);
        }
    };
    (host.to_string(), port)
}

/// 邮箱运行时配置（优先从数据库读取，回退到环境变量）
pub struct EmailRuntimeConfig {
    pub provider: String, // "builtin"、"http_api" 或 "smtp"（保留用于兼容，实际发送统一走内置通道逻辑）
    // 通道开关
    pub channel_general: bool, // 通用配置（发件邮箱 + 授权码）作为 SMTP 通道
    pub channel_api: bool,     // 外部 HTTP API 通道
    pub channel_pool: bool,    // SMTP 账号池通道
    // HTTP API 模式
    pub api_primary: String,
    pub api_backup: String,
    // 通用
    pub sender: String,
    pub password: String,
    // SMTP 模式
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_accounts: Vec<SmtpAccount>,
}

/// 从 `server_settings` 表读取邮箱配置，留空的字段回退到环境变量默认值。
pub async fn load_email_config(
    pool: &MySqlPool,
    fallback: &crate::config::Config,
) -> EmailRuntimeConfig {
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

    let provider = read_setting(pool, "email_provider")
        .await
        .unwrap_or_else(|| "builtin".to_string());

    // 通道开关：默认开启通用配置与账号池，外部 API 默认关闭
    let channel_general = read_setting(pool, "email_channel_general")
        .await
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    let channel_api = read_setting(pool, "email_channel_api")
        .await
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let channel_pool = read_setting(pool, "email_channel_pool")
        .await
        .map(|s| s == "1" || s.eq_ignore_ascii_case("true"))
        .unwrap_or(true);

    let api_primary = read_setting(pool, "email_api_primary")
        .await
        .unwrap_or_else(|| fallback.email_api_primary.clone());
    let api_backup = read_setting(pool, "email_api_backup")
        .await
        .unwrap_or_else(|| fallback.email_api_backup.clone());
    let sender = read_setting(pool, "email_sender")
        .await
        .unwrap_or_else(|| fallback.email_sender.clone());
    let password = read_setting(pool, "email_password")
        .await
        .unwrap_or_else(|| fallback.email_password.clone());

    let smtp_host = read_setting(pool, "smtp_host")
        .await
        .unwrap_or_default();
    let smtp_port = read_setting(pool, "smtp_port")
        .await
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(465);
    let smtp_username = read_setting(pool, "smtp_username")
        .await
        .unwrap_or_else(|| sender.clone());
    let smtp_password = read_setting(pool, "smtp_password")
        .await
        .unwrap_or_else(|| password.clone());
    let smtp_accounts = read_setting(pool, "smtp_accounts")
        .await
        .and_then(|s| serde_json::from_str::<Vec<SmtpAccount>>(&s).ok())
        .unwrap_or_default();

    EmailRuntimeConfig {
        provider,
        channel_general,
        channel_api,
        channel_pool,
        api_primary,
        api_backup,
        sender,
        password,
        smtp_host,
        smtp_port,
        smtp_username,
        smtp_password,
        smtp_accounts,
    }
}

/// 通用配置通道：用「发件邮箱 + 通用授权码」作为单一 SMTP 投递账号，SMTP 地址按域名自动识别。
fn general_smtp_account(cfg: &EmailRuntimeConfig) -> Option<SmtpAccount> {
    if cfg.sender.trim().is_empty() || cfg.password.trim().is_empty() {
        return None;
    }
    Some(SmtpAccount {
        sender: cfg.sender.clone(),
        host: String::new(), // 空 host 触发按域名自动识别
        port: 465,
        username: cfg.sender.clone(),
        password: cfg.password.clone(),
        enabled: true,
        remark: "通用配置".to_string(),
    })
}

/// 通过 HTTP API 发送邮件（主地址失败回退备用地址）
async fn send_via_http_api(cfg: &EmailRuntimeConfig, title: &str, plain: &str, html_opts: Option<&str>, recipient: &str) -> Result<(), String> {
    if cfg.api_primary.is_empty() && cfg.api_backup.is_empty() {
        return Err("外部邮箱机 API 地址未配置".to_string());
    }

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => return Err(format!("HTTP 客户端构建失败: {e}")),
    };

    // 有 HTML 时发送 HTML 内容，否则发送纯文本
    let context = html_opts.unwrap_or(plain);

    let params = [
        ("email", cfg.sender.as_str()),
        ("password", cfg.password.as_str()),
        ("title", title),
        ("context", context),
        ("recipient", recipient),
    ];

    let body = if !cfg.api_primary.is_empty() {
        match client.get(&cfg.api_primary).query(&params).send().await {
            Ok(r) => r.text().await.unwrap_or_default(),
            Err(e) => {
                if cfg.api_backup.is_empty() {
                    return Err(format!("邮箱 API 主地址请求失败，且备用地址未配置: {}", e));
                }
                match client.get(&cfg.api_backup).query(&params).send().await {
                    Ok(r) => r.text().await.unwrap_or_default(),
                    Err(e2) => {
                        return Err(format!(
                            "主地址和备用地址均请求失败。主: {}; 备: {}",
                            e, e2
                        ))
                    }
                }
            }
        }
    } else {
        match client.get(&cfg.api_backup).query(&params).send().await {
            Ok(r) => r.text().await.unwrap_or_default(),
            Err(e) => return Err(format!("邮箱 API 备用地址请求失败: {}", e)),
        }
    };

    // 根据返回内容判断是否成功
    let ok_signals = ["success", "ok", "true", "1", "200", "发送成功"];
    let lower = body.to_lowercase();
    if ok_signals.iter().any(|s| lower.contains(s)) || !body.is_empty() {
        Ok(())
    } else {
        Err(format!("邮箱 API 返回未包含成功标识: {}", &body[..body.len().min(200)]))
    }
}

/// 通过标准 SMTP 发送邮件
/// 通过指定 SMTP 账号发送邮件
async fn send_via_smtp_account(account: &SmtpAccount, title: &str, plain: &str, html_opts: Option<&str>, recipient: &str) -> Result<(), String> {
    use lettre::message::header::ContentType;
    use lettre::message::{Mailbox, MultiPart, SinglePart};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

    if account.sender.trim().is_empty() {
        return Err("发件邮箱地址未配置".to_string());
    }

    // host 为空时按发件邮箱域名自动识别 SMTP 地址和端口
    let (host, port) = if account.host.trim().is_empty() {
        let (h, p) = resolve_smtp_endpoint(&account.sender);
        if h.is_empty() {
            return Err("无法自动识别 SMTP 服务器地址，请检查发件邮箱".to_string());
        }
        (h, p)
    } else {
        (account.host.clone(), account.port)
    };

    let from_mailbox = Mailbox::new(
        Some("弦予音乐".to_string()),
        account
            .sender
            .parse()
            .map_err(|e| format!("发件邮箱地址格式错误: {e}"))?,
    );
    let to_mailbox = Mailbox::new(None, recipient.parse().map_err(|e| format!("收件邮箱地址格式错误: {e}"))?);

    // 构建邮件：有 HTML 时使用 multipart/alternative（纯文本 + HTML），否则纯文本
    let builder = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(title);

    let email = if let Some(html) = html_opts {
        builder
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(plain.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html.to_string()),
                    ),
            )
            .map_err(|e| format!("邮件构建失败: {e}"))?
    } else {
        builder
            .multipart(
                MultiPart::mixed().singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(plain.to_string()),
                ),
            )
            .map_err(|e| format!("邮件构建失败: {e}"))?
    };

    // 根据端口选择 TLS 模式
    // 465 → 隐式 TLS (Ssl)
    // 587/25 → STARTTLS
    let transport = if port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
            .map_err(|e| format!("SMTP 连接构建失败: {e}"))?
            .port(port)
            .credentials(Credentials::new(
                account.username.clone(),
                account.password.clone(),
            ))
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
            .map_err(|e| format!("SMTP 连接构建失败: {e}"))?
            .port(port)
            .credentials(Credentials::new(
                account.username.clone(),
                account.password.clone(),
            ))
            .build()
    };

    transport
        .send(email)
        .await
        .map(|_| ())
        .map_err(|e| format!("SMTP 发送失败: {e}"))
}

/// 内置邮箱机的投递通道：SMTP 账号或外部 HTTP API
enum MailChannel {
    Smtp(SmtpAccount),
    Api,
}

/// 根据通道开关构建投递通道列表，顺序为：通用配置 → 外部 API → SMTP 账号池。
/// 返回 (通道列表, 未启用/未配置的说明)。
fn build_mail_channels(cfg: &EmailRuntimeConfig) -> (Vec<MailChannel>, Vec<String>) {
    let mut channels = Vec::new();
    let mut notes = Vec::new();

    // 1. 通用配置（发件邮箱 + 授权码 → 单一 SMTP 通道）
    if cfg.channel_general {
        match general_smtp_account(cfg) {
            Some(acc) => channels.push(MailChannel::Smtp(acc)),
            None => notes.push("通用配置未启用（缺少发件邮箱或授权码）".to_string()),
        }
    } else {
        notes.push("通用配置通道未开启".to_string());
    }

    // 2. 外部 HTTP API
    if cfg.channel_api {
        if !cfg.api_primary.is_empty() || !cfg.api_backup.is_empty() {
            channels.push(MailChannel::Api);
        } else {
            notes.push("外部 API 通道未配置地址".to_string());
        }
    } else {
        notes.push("外部 API 通道未开启".to_string());
    }

    // 3. SMTP 账号池（每个可用账号独立成通道，参与轮换）
    if cfg.channel_pool {
        let usable: Vec<SmtpAccount> = cfg
            .smtp_accounts
            .iter()
            .filter(|a| a.is_usable())
            .cloned()
            .collect();
        if usable.is_empty() {
            notes.push("SMTP 账号池未配置可用账号".to_string());
        } else {
            for acc in usable {
                channels.push(MailChannel::Smtp(acc));
            }
        }
    } else {
        notes.push("SMTP 账号池通道未开启".to_string());
    }

    (channels, notes)
}

async fn create_builtin_mail_log(pool: &MySqlPool, recipient: &str, title: &str, detail: &str) -> Option<u64> {
    sqlx::query(
        "INSERT INTO email_send_log (email, subject, interface_id, template_id, status, error_msg, ip) VALUES (?,?,0,0,0,?,'builtin')",
    )
    .bind(recipient)
    .bind(title)
    .bind(detail)
    .execute(pool)
    .await
    .ok()
    .map(|r| r.last_insert_id())
}

async fn update_builtin_mail_log(pool: &MySqlPool, id: Option<u64>, status: i32, detail: &str) {
    if let Some(id) = id {
        let _ = sqlx::query("UPDATE email_send_log SET status = ?, error_msg = ? WHERE id = ?")
            .bind(status)
            .bind(detail)
            .bind(id)
            .execute(pool)
            .await;
    }
}

/// 服务端内置邮箱机：按通道开关构建投递通道列表，依次尝试直到成功，并按成功发送次数轮换起始通道。
async fn send_via_builtin_mailer(
    cfg: &EmailRuntimeConfig,
    pool: &MySqlPool,
    title: &str,
    plain: &str,
    html_opts: Option<&str>,
    recipient: &str,
) -> Result<(), String> {
    let log_id = create_builtin_mail_log(pool, recipient, title, "内置邮箱机已接收，等待投递").await;
    let mut errors = Vec::new();

    let (mut channels, notes) = build_mail_channels(cfg);
    if channels.is_empty() {
        let reason = if notes.is_empty() {
            "无可用投递通道".to_string()
        } else {
            notes.join("；")
        };
        update_builtin_mail_log(pool, log_id, 2, &reason).await;
        return Err(format!("内置邮箱机投递失败: {}", reason));
    }

    // 按「成功发送次数」轮换起始通道，实现发送成功即依次轮换
    let sent_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_send_log WHERE ip = 'builtin' AND status = 1")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    let idx = (sent_count.max(0) as usize) % channels.len();
    channels.rotate_left(idx);

    for channel in channels {
        let result = match channel {
            MailChannel::Smtp(account) => {
                let label = if account.remark.trim().is_empty() {
                    account.sender.clone()
                } else {
                    format!("{} ({})", account.sender, account.remark)
                };
                send_via_smtp_account(&account, title, plain, html_opts, recipient)
                    .await
                    .map_err(|e| format!("SMTP 账号 {} 失败: {}", label, e))
            }
            MailChannel::Api => send_via_http_api(cfg, title, plain, html_opts, recipient)
                .await
                .map_err(|e| format!("外部 API 失败: {}", e)),
        };

        match result {
            Ok(()) => {
                update_builtin_mail_log(pool, log_id, 1, "内置邮箱机投递成功").await;
                return Ok(());
            }
            Err(e) => errors.push(e),
        }
    }

    let mut reason = errors.join("；");
    if !notes.is_empty() {
        reason = format!("{}{}{}", reason, if reason.is_empty() { "" } else { "；" }, notes.join("；"));
    }
    update_builtin_mail_log(pool, log_id, 2, &reason).await;
    Err(format!("内置邮箱机投递失败: {}", reason))
}

/// 统一邮件发送入口：始终走内置邮箱机，按各通道开关依次尝试并轮换。
///
/// 邮箱配置优先从数据库 `server_settings` 表读取，留空字段回退到环境变量默认值。
/// 返回 `Ok(())` 表示发送成功，返回 `Err(reason)` 表示失败并附带原因。
pub async fn call_email_api(
    config: &crate::config::Config,
    pool: &MySqlPool,
    title: &str,
    context: &str,
    recipient: &str,
) -> Result<(), String> {
    let cfg = load_email_config(pool, config).await;
    send_via_builtin_mailer(&cfg, pool, title, context, None, recipient).await
}

/// 统一邮件发送入口（HTML 卡片版）：正文使用 HTML 渲染，同时附带纯文本兜底。
/// 各通道发送 HTML 内容；若通道不支持 HTML 会退化到纯文本（multipart/alternative 兜底）。
pub async fn call_email_api_html(
    config: &crate::config::Config,
    pool: &MySqlPool,
    title: &str,
    html: &str,
    plain: &str,
    recipient: &str,
) -> Result<(), String> {
    let cfg = load_email_config(pool, config).await;
    send_via_builtin_mailer(&cfg, pool, title, plain, Some(html), recipient).await
}

/// 写操作日志
async fn log_action(pool: &MySqlPool, user_id: i64, email: &str, action: &str, ip: &str) {
    let _ = sqlx::query(
        "INSERT INTO email_test_logs (user_id, email, action, detail) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(email)
    .bind(action)
    .bind(ip)
    .execute(pool)
    .await;
}

/// 发送邮箱验证码
pub async fn send_code(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let captcha_token = {
        let token = str_of(&data, "captcha_token");
        if token.trim().is_empty() {
            str_of(&data, "turnstile_token")
        } else {
            token
        }
    };

    if email.is_empty() || !is_valid_email(&email) {
        return ctx.err(400, "邮箱地址格式不正确");
    }

    let captcha_config = load_captcha_config(pool, &ctx.config).await;
    match verify_captcha_token(&captcha_config, &captcha_token, &ctx.client_ip).await {
        Ok(true) => {}
        Ok(false) => return ctx.err(400, "请先完成人机验证"),
        Err(e) => {
            eprintln!("[email_auth] 人机验证校验失败: {}", e);
            return ctx.err(500, "人机验证服务暂不可用，请稍后重试");
        }
    }

    // 频率限制：60 秒内只能发一次
    let recent = sqlx::query(
        "SELECT id FROM email_test_codes WHERE email = ? AND created_at > DATE_SUB(NOW(), INTERVAL 60 SECOND) ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    if recent.is_some() {
        return ctx.err(400, "发送过于频繁，请 60 秒后再试");
    }

    // 生成 6 位验证码
    let code = format!("{:06}", random_int(0, 999999));

    // 写入数据库
    let _ = sqlx::query(
        "INSERT INTO email_test_codes (email, code, type, expired_at) VALUES (?, ?, 'register', DATE_ADD(NOW(), INTERVAL 5 MINUTE))",
    )
    .bind(&email)
    .bind(&code)
    .execute(pool)
    .await;

    // 调用邮箱 API 发送
    let title = "【弦予】您的邮箱验证码";
    let context = format!(
        "您正在进行弦予测试系统的注册/登录操作。\n\n您的验证码是：{}\n\n验证码 5 分钟内有效，请勿泄露给他人。\n\n—— 弦予邮箱注册登录测试系统",
        code
    );

    match call_email_api(&ctx.config, pool, title, &context, &email).await {
        Ok(()) => ctx.ok("验证码已发送，请查收邮件", Value::Null),
        Err(e) => {
            eprintln!("[email_auth] send_code 发送失败: {}", e);
            ctx.err(500, "邮件发送失败，请稍后重试")
        }
    }
}

/// 用户注册
pub async fn register(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let code = str_of(&data, "code").trim().to_string();
    let password = str_of(&data, "password");
    let password2 = str_of(&data, "password2");
    let nickname = str_of(&data, "nickname").trim().to_string();

    // 表单验证
    if email.is_empty() || !is_valid_email(&email) {
        return ctx.err(400, "请输入合法的邮箱地址");
    }
    if code.is_empty() {
        return ctx.err(400, "请输入邮箱验证码");
    }
    if password.len() < 6 || password.len() > 32 {
        return ctx.err(400, "密码长度需为 6-32 位");
    }
    if password != password2 {
        return ctx.err(400, "两次输入的密码不一致");
    }

    // 校验验证码（未使用 + 未过期）
    let code_row = sqlx::query(
        "SELECT id FROM email_test_codes WHERE email = ? AND code = ? AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .bind(&code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(code_row) = code_row else {
        return ctx.err(400, "验证码不正确或已过期");
    };
    let code_id: i64 = code_row.try_get("id").unwrap_or(0);

    // 检查邮箱是否已注册
    let existing = sqlx::query("SELECT id FROM email_test_users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if existing.is_some() {
        return ctx.err(400, "该邮箱已注册，请直接登录");
    }

    // 哈希密码
    let hash = match bcrypt::hash(&password, 10) {
        Ok(h) => h,
        Err(_) => return ctx.err(500, "密码加密失败"),
    };

    // 写入用户
    let ins = sqlx::query("INSERT INTO email_test_users (email, password, nickname) VALUES (?, ?, ?)")
        .bind(&email)
        .bind(&hash)
        .bind(&nickname)
        .execute(pool)
        .await;

    match ins {
        Ok(result) => {
            let uid = result.last_insert_id() as i64;
            // 标记验证码已使用
            let _ = sqlx::query("UPDATE email_test_codes SET used = 1 WHERE id = ?")
                .bind(code_id)
                .execute(pool)
                .await;
            // 写日志
            log_action(pool, uid, &email, "register", &ctx.client_ip).await;

            ctx.ok("注册成功", Value::Null)
        }
        Err(_) => ctx.err(500, "注册失败，请稍后重试"),
    }
}

/// 用户登录
pub async fn login(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let password = str_of(&data, "password");

    if email.is_empty() || password.is_empty() {
        return ctx.err(400, "请输入邮箱和密码");
    }

    let row = sqlx::query("SELECT id, email, password, nickname, status FROM email_test_users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    let Some(row) = row else {
        return ctx.err(400, "邮箱或密码不正确");
    };

    let hash: String = row.try_get("password").unwrap_or_default();
    if !bcrypt::verify(&password, &hash).unwrap_or(false) {
        return ctx.err(400, "邮箱或密码不正确");
    }

    let status: i64 = row.try_get("status").unwrap_or(1);
    if status == 0 {
        return ctx.err(403, "账号已被禁用，请联系管理员");
    }

    // 检查设备封禁
    let login_device_id = str_of(&data, "device_id").trim().to_string();
    if !login_device_id.is_empty() {
        let banned = sqlx::query("SELECT reason FROM banned_devices WHERE device_id = ? LIMIT 1")
            .bind(&login_device_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        if let Some(row) = banned {
            let reason: String = row.try_get::<String, _>("reason").unwrap_or_default();
            let reason = reason.trim();
            if reason.is_empty() {
                return ctx.err(403, "该设备已被封禁，请联系管理员");
            }
            return ctx.err(403, &format!("该设备已被封禁，原因：{}。如有疑问请联系管理员", reason));
        }
    }

    let uid: i64 = row.try_get("id").unwrap_or(0);
    let user_email: String = row.try_get("email").unwrap_or_default();
    let nickname: String = row.try_get("nickname").unwrap_or_default();

    // 更新最后登录时间
    let _ = sqlx::query("UPDATE email_test_users SET last_login = NOW() WHERE id = ?")
        .bind(uid)
        .execute(pool)
        .await;

    // 写日志
    log_action(pool, uid, &user_email, "login", &ctx.client_ip).await;

    // 生成 JWT
    let token = sign_email_token(&ctx.config, uid, &user_email);

    ctx.ok(
        "登录成功",
        json!({
            "token": token,
            "user": {
                "id": uid,
                "email": user_email,
                "nickname": nickname,
            }
        }),
    )
}

/// 重置密码（找回密码）
pub async fn reset_password(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let email = str_of(&data, "email").trim().to_string();
    let code = str_of(&data, "code").trim().to_string();
    let password = str_of(&data, "password");
    let password2 = str_of(&data, "password2");

    if email.is_empty() || !is_valid_email(&email) {
        return ctx.err(400, "请输入合法的邮箱地址");
    }
    if code.is_empty() {
        return ctx.err(400, "请输入邮箱验证码");
    }
    if password.len() < 6 || password.len() > 32 {
        return ctx.err(400, "密码长度需为 6-32 位");
    }
    if password != password2 {
        return ctx.err(400, "两次输入的密码不一致");
    }

    // 校验验证码
    let code_row = sqlx::query(
        "SELECT id FROM email_test_codes WHERE email = ? AND code = ? AND used = 0 AND expired_at > NOW() ORDER BY id DESC LIMIT 1",
    )
    .bind(&email)
    .bind(&code)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(code_row) = code_row else {
        return ctx.err(400, "验证码不正确或已过期");
    };
    let code_id: i64 = code_row.try_get("id").unwrap_or(0);

    // 检查用户是否存在
    let user_row = sqlx::query("SELECT id FROM email_test_users WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user_row) = user_row else {
        return ctx.err(400, "该邮箱尚未注册，请先注册");
    };
    let user_id: i64 = user_row.try_get("id").unwrap_or(0);

    // 更新密码
    let hash = match bcrypt::hash(&password, 10) {
        Ok(h) => h,
        Err(_) => return ctx.err(500, "密码加密失败"),
    };
    let _ = sqlx::query("UPDATE email_test_users SET password = ? WHERE id = ?")
        .bind(&hash)
        .bind(user_id)
        .execute(pool)
        .await;

    // 标记验证码已使用
    let _ = sqlx::query("UPDATE email_test_codes SET used = 1 WHERE id = ?")
        .bind(code_id)
        .execute(pool)
        .await;

    // 写日志
    log_action(pool, user_id, &email, "reset_password", &ctx.client_ip).await;

    ctx.ok("密码已重置成功", Value::Null)
}

/// 获取用户信息（需 token）
pub async fn get_profile(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let token = str_of(&data, "token");

    let claims = match verify_email_token(&ctx.config, &token) {
        Some(c) => c,
        None => return ctx.err(401, "未登录或登录已过期"),
    };

    let row = sqlx::query(
        "SELECT id, email, nickname, status, created_at, last_login FROM email_test_users WHERE id = ?",
    )
    .bind(claims.sub)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    let Some(row) = row else {
        return ctx.err(404, "用户不存在");
    };

    let status: i64 = row.try_get("status").unwrap_or(1);
    if status == 0 {
        return ctx.err(403, "账号已被禁用");
    }

    // 获取最近操作日志
    let logs = sqlx::query(
        "SELECT action, detail, created_at FROM email_test_logs WHERE user_id = ? ORDER BY id DESC LIMIT 8",
    )
    .bind(claims.sub)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let logs_arr: Vec<Value> = logs
        .iter()
        .map(|r| {
            json!({
                "action": r.try_get::<String, _>("action").unwrap_or_default(),
                "detail": r.try_get::<String, _>("detail").unwrap_or_default(),
                "created_at": r.try_get::<String, _>("created_at").unwrap_or_default(),
            })
        })
        .collect();

    ctx.ok(
        "",
        json!({
            "id": row.try_get::<i64, _>("id").unwrap_or(0),
            "email": row.try_get::<String, _>("email").unwrap_or_default(),
            "nickname": row.try_get::<String, _>("nickname").unwrap_or_default(),
            "status": status,
            "created_at": row.try_get::<String, _>("created_at").unwrap_or_default(),
            "last_login": row.try_get::<Option<String>, _>("last_login").ok().flatten().unwrap_or_default(),
            "logs": logs_arr,
        }),
    )
}

// ============================================================
//  单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造测试用 Config
    fn test_config() -> crate::config::Config {
        crate::config::Config {
            db_host: "127.0.0.1".into(),
            db_port: 3306,
            db_name: "test".into(),
            db_user: "root".into(),
            db_pass: "".into(),
            db_charset: "utf8mb4".into(),
            api_secret: "test_secret".into(),
            api_timestamp_tolerance: 300,
            admin_username: "admin".into(),
            admin_password: "admin".into(),
            listen_addr: "0.0.0.0:0".into(),
            jwt_secret: "test_jwt_secret_key".into(),
            email_api_primary: "http://localhost/a".into(),
            email_api_backup: "http://localhost/b".into(),
            email_sender: "test@localhost".into(),
            email_password: "pass".into(),
            captcha_secret: "".into(),
            turnstile_secret: "".into(),
            hcaptcha_secret: "".into(),
            local_debug_no_db: false,
        }
    }

    // ===== 邮箱格式校验测试 =====

    #[test]
    fn test_valid_emails() {
        let valid = [
            "user@example.com",
            "test.user@domain.org",
            "a@b.co",
            "name+tag@sub.domain.com",
            "user_name@my-domain.net",
        ];
        for email in &valid {
            assert!(is_valid_email(email), "应判定为合法邮箱: {}", email);
        }
    }

    #[test]
    fn test_invalid_emails() {
        let invalid = [
            "",
            "notanemail",
            "@domain.com",
            "user@",
            "user@.com",
            "userdomain",
            "   ",
            "user @domain.com",
        ];
        for email in &invalid {
            assert!(!is_valid_email(email), "应判定为非法邮箱: {:?}", email);
        }
    }

    #[test]
    fn test_email_trim() {
        assert!(is_valid_email("  user@example.com  "), "应自动 trim 前后空格");
    }

    // ===== JWT 签发与验证测试 =====

    #[test]
    fn test_jwt_round_trip() {
        let cfg = test_config();
        let token = sign_email_token(&cfg, 42, "user@test.com");
        assert!(!token.is_empty(), "token 不应为空");

        let claims = verify_email_token(&cfg, &token);
        assert!(claims.is_some(), "有效 token 应验证成功");

        let c = claims.unwrap();
        assert_eq!(c.sub, 42, "user_id 应一致");
        assert_eq!(c.email, "user@test.com", "email 应一致");
    }

    #[test]
    fn test_jwt_wrong_secret() {
        let cfg1 = test_config();
        let cfg2 = crate::config::Config {
            jwt_secret: "different_secret".into(),
            ..test_config()
        };
        let token = sign_email_token(&cfg1, 1, "a@b.com");
        assert!(verify_email_token(&cfg2, &token).is_none(), "不同密钥应验证失败");
    }

    #[test]
    fn test_jwt_empty_token() {
        let cfg = test_config();
        assert!(verify_email_token(&cfg, "").is_none(), "空 token 应验证失败");
        assert!(verify_email_token(&cfg, "invalid.token.here").is_none(), "格式错误的 token 应验证失败");
    }

    // ===== bcrypt 密码哈希测试 =====

    #[test]
    fn test_bcrypt_hash_and_verify() {
        let password = "mySecurePass123";
        let hash = bcrypt::hash(password, 4).unwrap();
        assert!(bcrypt::verify(password, &hash).unwrap(), "正确密码应验证通过");
    }

    #[test]
    fn test_bcrypt_wrong_password() {
        let hash = bcrypt::hash("correctPassword", 4).unwrap();
        assert!(!bcrypt::verify("wrongPassword", &hash).unwrap(), "错误密码应验证失败");
    }

    #[test]
    fn test_bcrypt_different_hashes() {
        let password = "samePassword";
        let h1 = bcrypt::hash(password, 4).unwrap();
        let h2 = bcrypt::hash(password, 4).unwrap();
        assert_ne!(h1, h2, "相同密码应生成不同哈希（盐值随机）");
        assert!(bcrypt::verify(password, &h1).unwrap());
        assert!(bcrypt::verify(password, &h2).unwrap());
    }

    // ===== 请求体解析测试 =====

    #[test]
    fn test_parse_body_valid_json() {
        let body = r#"{"email":"test@x.com","code":"123456"}"#;
        let data = parse_body(body);
        assert_eq!(str_of(&data, "email"), "test@x.com");
        assert_eq!(str_of(&data, "code"), "123456");
    }

    #[test]
    fn test_parse_body_invalid_json() {
        let data = parse_body("not json");
        assert!(data.is_null(), "非法 JSON 应返回 Null");
    }

    #[test]
    fn test_str_of_missing_key() {
        let data = parse_body(r#"{"email":"a@b.com"}"#);
        assert_eq!(str_of(&data, "nonexistent"), "", "缺失 key 应返回空字符串");
    }
}
