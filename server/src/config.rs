use anyhow::Result;
use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_pass: String,
    pub db_charset: String,
    pub api_secret: String,
    pub api_timestamp_tolerance: i64,
    // 过渡期旧密钥（轮换后仍接受其签名/密文，待客户端全量更新后删除该配置）。
    // 2026-10-28 轮换：acca7562… 因客户端源码泄露（渗透报告 V-01）转入此槽位。
    #[serde(default)]
    pub api_secret_previous: Option<String>,
    pub admin_username: String,
    pub admin_password: String,
    pub listen_addr: String,
    pub jwt_secret: String,
    pub email_api_primary: String,
    pub email_api_backup: String,
    pub email_sender: String,
    pub email_password: String,
    #[serde(default)]
    pub captcha_secret: String,
    #[serde(default)]
    pub turnstile_secret: String,
    #[serde(default)]
    pub hcaptcha_secret: String,
    // V-05 加固：PoW 工作量证明。email_* 流程始终强制（从未随客户端发布，无兼容
    // 影响）；username 流程（register/user_login/send_verify_code/reset_password）
    // 在本开关开启后才强制——旧版客户端不带 PoW 会被拒，待新版全量后再置 true。
    #[serde(default)]
    pub pow_required: bool,
    // PoW 难度（sha256 前导零 bit 数）：20 ≈ 平均 2^20 次哈希，移动端约 0.1-0.5s，
    // 服务端验证毫秒级。合法范围 12-28，越界自动收敛。
    #[serde(default = "default_pow_difficulty")]
    pub pow_difficulty: u32,
    #[serde(default)]
    pub local_debug_no_db: bool,
    #[serde(default = "default_static_dir")]
    pub static_dir: String,
    #[serde(default)]
    pub public_base_url: String,
    #[serde(default)]
    pub share_base_url: String,
    // 鸿蒙 QQ 开放平台分享签名密钥（QQ 互联 AppKey，严禁进客户端）。
    // 为空时 qq_share_sign 接口返回错误，即关闭鸿蒙端 QQ 分享签名能力。
    #[serde(default)]
    pub qq_app_key: String,
    #[serde(default = "default_true")]
    pub require_user_token: bool,
    // 仅当服务端处于受信反向代理（nginx）之后时才置 true：直连部署下
    // X-Forwarded-For / X-Real-IP 可被客户端任意伪造，会绕过限流与登录锁定
    #[serde(default = "default_true")]
    pub trust_proxy: bool,
    // 允许跨域访问 API 的来源（逗号分隔，含协议）。默认空 = 不下发 CORS 头：
    // 已知消费端（客户端原生 HTTP、admin-web 生产/开发代理、官网静态页）均为同源
    #[serde(default)]
    pub cors_origins: Vec<String>,
    // 兜底/内测签名档位：默认关闭 = A 档本机密钥（FALLBACK_SIGN_PRIVATE_KEY 或
    // api/fallback_sign_key.txt，无 keyId）；开启 = B 档托管（同二进制 --signer
    // 子进程持有私钥，主服务仅经 localhost 回环调用签发，自身不落私钥）。
    #[serde(default)]
    pub fallback: FallbackSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct FallbackSection {
    pub signer: SignerSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct SignerSection {
    pub enabled: bool,
    /// 签发服务地址（仅回环地址，如 http://127.0.0.1:8791）
    pub url: String,
    /// 与 --signer 进程一致的共享令牌（env FALLBACK_SIGNER_TOKEN 优先）
    pub token: String,
}

impl SignerSection {
    pub fn effective_url(&self) -> String {
        let url = std::env::var("FALLBACK_SIGNER_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.url.trim().to_string());
        if url.is_empty() {
            "http://127.0.0.1:8791".to_string()
        } else {
            url
        }
    }

    pub fn effective_token(&self) -> String {
        std::env::var("FALLBACK_SIGNER_TOKEN")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.token.trim().to_string())
    }
}

fn default_static_dir() -> String {
    "../admin-web/dist".into()
}

fn default_true() -> bool {
    true
}

fn default_pow_difficulty() -> u32 {
    20
}

const KNOWN_DEFAULT_SECRET: &str = "bf027fedb4d1b4f969c10495f12f17042bf0de02de128200";

impl Config {
    pub fn load() -> Result<Config> {
        let mut cfg = if let Ok(content) = fs::read_to_string("config.json") {
            serde_json::from_str::<Config>(&content)?
        } else {
            Config::from_env()
        };
        cfg.listen_addr = env::var("LISTEN_ADDR").unwrap_or(cfg.listen_addr);
        cfg.api_secret_previous = match env::var("API_SECRET_PREVIOUS") {
            Ok(v) => {
                let v = v.trim().to_string();
                if v.is_empty() { None } else { Some(v) }
            }
            Err(_) => cfg.api_secret_previous.clone(),
        };
        cfg.captcha_secret = env::var("CAPTCHA_SECRET").unwrap_or(cfg.captcha_secret);
        cfg.turnstile_secret = env::var("TURNSTILE_SECRET").unwrap_or(cfg.turnstile_secret);
        cfg.hcaptcha_secret = env::var("HCAPTCHA_SECRET").unwrap_or(cfg.hcaptcha_secret);
        cfg.static_dir = env::var("STATIC_DIR").unwrap_or(cfg.static_dir);
        cfg.public_base_url = env::var("PUBLIC_BASE_URL").unwrap_or(cfg.public_base_url);
        cfg.share_base_url = env::var("SHARE_BASE_URL").unwrap_or(cfg.share_base_url);
        cfg.qq_app_key = env::var("QQ_APP_KEY").unwrap_or(cfg.qq_app_key);
        cfg.require_user_token = env::var("REQUIRE_USER_TOKEN")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(cfg.require_user_token);
        cfg.local_debug_no_db = env::var("LOCAL_DEBUG_NO_DB")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(cfg.local_debug_no_db);
        cfg.trust_proxy = env::var("TRUST_PROXY")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(cfg.trust_proxy);
        if let Ok(v) = env::var("CORS_ORIGINS") {
            cfg.cors_origins = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        Ok(cfg)
    }

    pub fn validate_security(&self) -> Result<(), String> {
        if self.local_debug_no_db {
            return Ok(());
        }
        for (name, value) in [("jwt_secret", &self.jwt_secret), ("api_secret", &self.api_secret)] {
            if value.trim().is_empty() {
                return Err(format!("配置项 {name} 为空：请在 config.json 或环境变量（JWT_SECRET / API_SECRET）中设置强随机密钥"));
            }
            if value == KNOWN_DEFAULT_SECRET {
                if name == "jwt_secret" {
                    return Err(format!("配置项 jwt_secret 仍为代码内置默认值：该值随源码公开，任何人可伪造管理员令牌接管后台。请改为强随机值后重启（如 openssl rand -hex 32 生成）"));
                }
                tracing::warn!(
                    "安全警告：api_secret 仍为代码内置默认值（随源码公开，客户端签名可被伪造）。\
                     请在客户端完成密钥迁移后尽快更换（更换后需同步更新所有客户端配置）"
                );
                continue;
            }
            if value.len() < 32 {
                return Err(format!("配置项 {name} 长度不足 32 位：请使用强随机密钥（如 openssl rand -hex 32 生成）"));
            }
        }
        if self.jwt_secret == self.api_secret {
            return Err("jwt_secret 与 api_secret 不能相同：管理员令牌签名密钥与客户端 API 密钥必须各自独立，避免一处泄露全线失守".to_string());
        }
        if let Some(prev) = &self.api_secret_previous {
            if prev.trim().is_empty() {
                return Err("api_secret_previous 为空字符串：要么删除该配置，要么填入过渡期旧密钥".to_string());
            }
            if prev == &self.api_secret {
                return Err("api_secret_previous 与 api_secret 相同：过渡密钥必须与当前密钥不同".to_string());
            }
            tracing::info!(
                "api_secret 双钥过渡生效：同时接受当前密钥与旧密钥（api_secret_previous），\
                 客户端全量更新后请删除 api_secret_previous 以彻底作废旧密钥"
            );
        }
        if self.admin_username == "admin" && self.admin_password == "adminadmin" {
            tracing::warn!(
                "安全警告：后台账号仍为默认凭证 admin/adminadmin（随源码公开，任何人可登录后台）。\
                 请立即在后台修改密码，或在 config.json / 环境变量中设置 ADMIN_USERNAME / ADMIN_PASSWORD"
            );
        }
        if self.fallback.signer.enabled {
            if self.fallback.signer.effective_token().is_empty() {
                return Err("fallback.signer.enabled 已开启但未配置共享令牌：请在 config.json 的 \
                    fallback.signer.token 或环境变量 FALLBACK_SIGNER_TOKEN 设置（与 --signer 进程一致，如 openssl rand -hex 32 生成）"
                    .to_string());
            }
            let url = self.fallback.signer.effective_url();
            let host = url
                .split("://")
                .nth(1)
                .and_then(|rest| rest.split(['/', ':']).next())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !host.is_empty() && host != "127.0.0.1" && host != "localhost" && host != "[::1]" {
                tracing::warn!(
                    "安全警告：fallback.signer.url 非回环地址（{host}）——签发服务仅应监听 \
                     localhost/Unix socket，经公网转发会把签发能力暴露给外部"
                );
            }
        }
        Ok(())
    }

    fn from_env() -> Config {
        Config {
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            db_port: env::var("DB_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(3306),
            db_name: env::var("DB_NAME").unwrap_or_else(|_| "chexian".into()),
            db_user: env::var("DB_USER").unwrap_or_else(|_| "chexian".into()),
            db_pass: env::var("DB_PASS").unwrap_or_default(),
            db_charset: env::var("DB_CHARSET").unwrap_or_else(|_| "utf8mb4".into()),
            api_secret: env::var("API_SECRET").unwrap_or_else(|_| "bf027fedb4d1b4f969c10495f12f17042bf0de02de128200".into()),
            api_secret_previous: env::var("API_SECRET_PREVIOUS")
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            api_timestamp_tolerance: env::var("API_TIMESTAMP_TOLERANCE").ok().and_then(|s| s.parse().ok()).unwrap_or(900),
            admin_username: env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into()),
            admin_password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "adminadmin".into()),
            listen_addr: env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "bf027fedb4d1b4f969c10495f12f17042bf0de02de128200".into()),
            email_api_primary: env::var("EMAIL_API_PRIMARY").unwrap_or_default(),
            email_api_backup: env::var("EMAIL_API_BACKUP").unwrap_or_default(),
            email_sender: env::var("EMAIL_SENDER").unwrap_or_else(|_| "no-reply@example.com".into()),
            email_password: env::var("EMAIL_PASSWORD").unwrap_or_default(),
            captcha_secret: env::var("CAPTCHA_SECRET").unwrap_or_default(),
            turnstile_secret: env::var("TURNSTILE_SECRET").unwrap_or_default(),
            pow_required: env::var("POW_REQUIRED").ok().map(|s| s == "1" || s.eq_ignore_ascii_case("true")).unwrap_or(false),
            pow_difficulty: env::var("POW_DIFFICULTY").ok().and_then(|s| s.parse().ok()).unwrap_or_else(default_pow_difficulty),
            hcaptcha_secret: env::var("HCAPTCHA_SECRET").unwrap_or_default(),
            static_dir: env::var("STATIC_DIR").unwrap_or_else(|_| default_static_dir()),
            local_debug_no_db: env::var("LOCAL_DEBUG_NO_DB")
                .ok()
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
                .unwrap_or(false),
            public_base_url: env::var("PUBLIC_BASE_URL").unwrap_or_default(),
            share_base_url: env::var("SHARE_BASE_URL").unwrap_or_default(),
            qq_app_key: env::var("QQ_APP_KEY").unwrap_or_default(),
            require_user_token: env::var("REQUIRE_USER_TOKEN")
                .ok()
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
                .unwrap_or(true),
            trust_proxy: env::var("TRUST_PROXY")
                .ok()
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
                .unwrap_or(true),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            fallback: FallbackSection::default(),
        }
    }
}
