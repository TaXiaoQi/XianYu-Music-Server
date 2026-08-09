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
    pub static_dir: String,
    #[serde(default)]
    pub local_debug_no_db: bool,
}

impl Config {
    pub fn load() -> Result<Config> {
        let mut cfg = if let Ok(content) = fs::read_to_string("config.json") {
            serde_json::from_str::<Config>(&content)?
        } else {
            Config::from_env()
        };
        cfg.listen_addr = env::var("LISTEN_ADDR").unwrap_or(cfg.listen_addr);
        cfg.local_debug_no_db = env::var("LOCAL_DEBUG_NO_DB")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(cfg.local_debug_no_db);
        Ok(cfg)
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
            static_dir: env::var("STATIC_DIR").unwrap_or_default(),
            local_debug_no_db: env::var("LOCAL_DEBUG_NO_DB")
                .ok()
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
                .unwrap_or(false),
        }
    }
}
