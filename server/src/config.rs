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
    #[serde(default)]
    pub local_debug_no_db: bool,
    #[serde(default = "default_static_dir")]
    pub static_dir: String,
    /// 公网访问地址，用于拼接壁纸等资源的完整 URL（如 https://xymusic.example.com）
    /// 当请求头中无法获取 Host 时，使用此配置作为兜底
    #[serde(default)]
    pub public_base_url: String,
    /// App 用户资源操作是否强制要求 user_token（true=硬模式拒绝无 token 请求；false=软模式仅校验携带 token 的请求）
    #[serde(default)]
    pub require_user_token: bool,
}

fn default_static_dir() -> String {
    "../admin-web/dist".into()
}

/// 代码内置的默认密钥（历史遗留），生产环境继续使用等于把后台拱手让人
const KNOWN_DEFAULT_SECRET: &str = "bf027fedb4d1b4f969c10495f12f17042bf0de02de128200";

impl Config {
    pub fn load() -> Result<Config> {
        let mut cfg = if let Ok(content) = fs::read_to_string("config.json") {
            serde_json::from_str::<Config>(&content)?
        } else {
            Config::from_env()
        };
        cfg.listen_addr = env::var("LISTEN_ADDR").unwrap_or(cfg.listen_addr);
        cfg.captcha_secret = env::var("CAPTCHA_SECRET").unwrap_or(cfg.captcha_secret);
        cfg.turnstile_secret = env::var("TURNSTILE_SECRET").unwrap_or(cfg.turnstile_secret);
        cfg.hcaptcha_secret = env::var("HCAPTCHA_SECRET").unwrap_or(cfg.hcaptcha_secret);
        cfg.static_dir = env::var("STATIC_DIR").unwrap_or(cfg.static_dir);
        cfg.public_base_url = env::var("PUBLIC_BASE_URL").unwrap_or(cfg.public_base_url);
        cfg.require_user_token = env::var("REQUIRE_USER_TOKEN")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(cfg.require_user_token);
        cfg.local_debug_no_db = env::var("LOCAL_DEBUG_NO_DB")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(cfg.local_debug_no_db);
        Ok(cfg)
    }

    /// 启动期安全校验：密钥为默认值/空/过短时拒绝启动，防止带着已知密钥暴露公网
    pub fn validate_security(&self) -> Result<(), String> {
        // 本地无数据库调试模式仅用于本机开发，跳过强校验
        if self.local_debug_no_db {
            return Ok(());
        }
        for (name, value) in [("jwt_secret", &self.jwt_secret), ("api_secret", &self.api_secret)] {
            if value.trim().is_empty() {
                return Err(format!("配置项 {name} 为空：请在 config.json 或环境变量（JWT_SECRET / API_SECRET）中设置强随机密钥"));
            }
            if value == KNOWN_DEFAULT_SECRET {
                // api_secret 内置于客户端默认配置，改动会导致存量客户端失联，
                // 阶段性只警告不拦截；jwt_secret 与客户端无关，必须更换
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
            api_timestamp_tolerance: env::var("API_TIMESTAMP_TOLERANCE").ok().and_then(|s| s.parse().ok()).unwrap_or(300),
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
            hcaptcha_secret: env::var("HCAPTCHA_SECRET").unwrap_or_default(),
            static_dir: env::var("STATIC_DIR").unwrap_or_else(|_| default_static_dir()),
            local_debug_no_db: env::var("LOCAL_DEBUG_NO_DB")
                .ok()
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
                .unwrap_or(false),
            public_base_url: env::var("PUBLIC_BASE_URL").unwrap_or_default(),
            require_user_token: env::var("REQUIRE_USER_TOKEN")
                .ok()
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
                .unwrap_or(false),
        }
    }
}
