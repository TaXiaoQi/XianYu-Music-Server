use super::*;

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

#[derive(Clone)]
pub struct EmailRuntimeConfig {
    pub provider: String,
    pub channel_general: bool,
    pub channel_api: bool,
    pub channel_pool: bool,
    pub api_primary: String,
    pub api_backup: String,
    pub sender: String,
    pub password: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_accounts: Vec<SmtpAccount>,
}

pub async fn load_email_config(
    pool: &MySqlPool,
    fallback: &crate::config::Config,
) -> EmailRuntimeConfig {
    // 每封信读 13 个 setting 的开销不小；且 DB 忙时读取失败会被静默降级为空配置，
    // 导致"SMTP 账号池未配置可用账号"假故障（验证码发不出）。加 60s 缓存兜底。
    static CFG_CACHE: std::sync::OnceLock<std::sync::Mutex<Option<(EmailRuntimeConfig, std::time::Instant)>>> =
        std::sync::OnceLock::new();
    const CFG_TTL: std::time::Duration = std::time::Duration::from_secs(60);
    if let Some(cache) = CFG_CACHE.get_or_init(|| std::sync::Mutex::new(None)).lock().ok() {
        if let Some((cfg, at)) = cache.as_ref() {
            if at.elapsed() <= CFG_TTL {
                return cfg.clone();
            }
        }
    }
    let cfg = load_email_config_uncached(pool, fallback).await;
    if let Ok(mut cache) = CFG_CACHE.get_or_init(|| std::sync::Mutex::new(None)).lock() {
        *cache = Some((cfg.clone(), std::time::Instant::now()));
    }
    cfg
}

async fn load_email_config_uncached(
    pool: &MySqlPool,
    fallback: &crate::config::Config,
) -> EmailRuntimeConfig {
    async fn read_setting(pool: &MySqlPool, key: &str) -> Option<String> {
        match sqlx::query("SELECT setting_value FROM server_settings WHERE setting_key = ? LIMIT 1")
            .bind(key)
            .fetch_optional(pool)
            .await
        {
            Ok(row) => row
                .and_then(|row| row.try_get::<Option<String>, _>(0).ok().flatten())
                .filter(|s| !s.trim().is_empty()),
            Err(e) => {
                tracing::warn!("读取邮箱配置 {} 失败: {}（本次按未配置处理）", key, e);
                None
            }
        }
    }

    let provider = read_setting(pool, "email_provider")
        .await
        .unwrap_or_else(|| "builtin".to_string());

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

impl SmtpAccount {
    fn is_usable(&self) -> bool {
        self.enabled
            && !self.sender.trim().is_empty()
            && !self.username.trim().is_empty()
            && !self.password.trim().is_empty()
    }
}

fn general_smtp_account(cfg: &EmailRuntimeConfig) -> Option<SmtpAccount> {
    if cfg.sender.trim().is_empty() || cfg.password.trim().is_empty() {
        return None;
    }
    Some(SmtpAccount {
        sender: cfg.sender.clone(),
        host: String::new(),
        port: 465,
        username: cfg.sender.clone(),
        password: cfg.password.clone(),
        enabled: true,
        remark: "通用配置".to_string(),
    })
}

// reqwest 错误的 Display 会带完整 URL（query 里含 SMTP 授权码），只能落类别化描述
fn describe_http_error(e: &reqwest::Error) -> String {
    if let Some(status) = e.status() {
        format!("HTTP {}", status.as_u16())
    } else if e.is_timeout() {
        "请求超时".to_string()
    } else if e.is_connect() {
        "网络连接失败".to_string()
    } else {
        "请求失败".to_string()
    }
}

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
                    return Err(format!("邮箱 API 主地址请求失败，且备用地址未配置: {}", describe_http_error(&e)));
                }
                match client.get(&cfg.api_backup).query(&params).send().await {
                    Ok(r) => r.text().await.unwrap_or_default(),
                    Err(e2) => {
                        return Err(format!(
                            "主地址和备用地址均请求失败。主: {}; 备: {}",
                            describe_http_error(&e),
                            describe_http_error(&e2)
                        ))
                    }
                }
            }
        }
    } else {
        match client.get(&cfg.api_backup).query(&params).send().await {
            Ok(r) => r.text().await.unwrap_or_default(),
            Err(e) => return Err(format!("邮箱 API 备用地址请求失败: {}", describe_http_error(&e))),
        }
    };

    let ok_signals = ["success", "ok", "true", "1", "200", "发送成功"];
    let lower = body.to_lowercase();
    if ok_signals.iter().any(|s| lower.contains(s)) {
        Ok(())
    } else {
        let short: String = body.chars().take(200).collect();
        Err(format!("邮箱 API 返回未包含成功标识: {}", short))
    }
}

async fn send_via_smtp_account(account: &SmtpAccount, title: &str, plain: &str, html_opts: Option<&str>, recipient: &str) -> Result<(), String> {
    use lettre::message::header::ContentType;
    use lettre::message::{Mailbox, MultiPart, SinglePart};
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

    if account.sender.trim().is_empty() {
        return Err("发件邮箱地址未配置".to_string());
    }

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

    // SMTP 偶发挂起会让请求无限等待（日志里 status=0 的"等待投递"永不更新即此因），强制 30s 超时
    match tokio::time::timeout(std::time::Duration::from_secs(30), transport.send(email)).await {
        Ok(res) => res
            .map(|_| ())
            .map_err(|e| format!("SMTP 发送失败: {e}")),
        Err(_) => Err("SMTP 发送超时(30s)".to_string()),
    }
}

enum MailChannel {
    Smtp(SmtpAccount),
    Api,
}

fn build_mail_channels(cfg: &EmailRuntimeConfig) -> (Vec<MailChannel>, Vec<String>) {
    let mut channels = Vec::new();
    let mut notes = Vec::new();

    if cfg.channel_general {
        match general_smtp_account(cfg) {
            Some(acc) => channels.push(MailChannel::Smtp(acc)),
            None => notes.push("通用配置未启用（缺少发件邮箱或授权码）".to_string()),
        }
    } else {
        notes.push("通用配置通道未开启".to_string());
    }

    if cfg.channel_api {
        if !cfg.api_primary.is_empty() || !cfg.api_backup.is_empty() {
            channels.push(MailChannel::Api);
        } else {
            notes.push("外部 API 通道未配置地址".to_string());
        }
    } else {
        notes.push("外部 API 通道未开启".to_string());
    }

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

    let sent_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM email_send_log WHERE ip = 'builtin' AND status = 1")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    let idx = (sent_count.max(0) as usize) % channels.len();
    channels.rotate_left(idx);

    for channel in channels {
        let label = match &channel {
            MailChannel::Smtp(account) => {
                if account.remark.trim().is_empty() {
                    format!("{}（SMTP）", account.sender)
                } else {
                    format!("{}（{}）", account.sender, account.remark)
                }
            }
            MailChannel::Api => "外部API".to_string(),
        };
        let result = match channel {
            MailChannel::Smtp(account) => send_via_smtp_account(&account, title, plain, html_opts, recipient)
                .await
                .map_err(|e| format!("SMTP 账号 {} 失败: {}", label, e)),
            MailChannel::Api => send_via_http_api(cfg, title, plain, html_opts, recipient)
                .await
                .map_err(|e| format!("外部 API 失败: {}", e)),
        };

        match result {
            Ok(()) => {
                // 记录实际使用的发件账号：排查"延迟/未收到"时可定位具体发件号
                update_builtin_mail_log(pool, log_id, 1, &format!("内置邮箱机投递成功（via {}）", label)).await;
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
