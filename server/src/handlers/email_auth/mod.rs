mod captcha;
mod flows;
mod mailer;
mod token;

pub(crate) use axum::response::Response;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use serde_json::{json, Value};
pub(crate) use sqlx::MySqlPool;
pub(crate) use sqlx::Row;

pub(crate) use crate::handlers::helpers::{is_valid_email, parse_body, random_int, str_of};
pub(crate) use crate::response::ReqCtx;

pub use captcha::{get_captcha_config, get_turnstile_config, load_captcha_config, verify_captcha_token};
pub use flows::{get_profile, login, register, reset_password, send_code};
pub use mailer::{call_email_api, call_email_api_html, load_email_config, SmtpAccount};
pub use token::verify_email_token;
pub(crate) use token::sign_email_token;

pub(crate) async fn log_action(pool: &MySqlPool, user_id: i64, email: &str, action: &str, ip: &str) {
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

// ============================================================
//  单元测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

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
            static_dir: "../admin-web/dist".into(),
            public_base_url: "".into(),
            share_base_url: "".into(),
            require_user_token: false,
            trust_proxy: true,
            cors_origins: Vec::new(),
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
