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

/// 调用惜梦邮箱 API 发送邮件（主地址失败回退备用地址）
async fn call_email_api(
    config: &crate::config::Config,
    title: &str,
    context: &str,
    recipient: &str,
) -> bool {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let params = [
        ("email", config.email_sender.as_str()),
        ("password", config.email_password.as_str()),
        ("title", title),
        ("context", context),
        ("recipient", recipient),
    ];

    // 主地址
    let resp = client.get(&config.email_api_primary).query(&params).send().await;
    let body = match resp {
        Ok(r) => r.text().await.unwrap_or_default(),
        Err(_) => {
            // 备用地址
            match client.get(&config.email_api_backup).query(&params).send().await {
                Ok(r) => r.text().await.unwrap_or_default(),
                Err(_) => return false,
            }
        }
    };

    // 根据返回内容判断是否成功
    let ok_signals = ["success", "ok", "true", "1", "200", "发送成功"];
    let lower = body.to_lowercase();
    ok_signals.iter().any(|s| lower.contains(s)) || !body.is_empty()
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

    if email.is_empty() || !is_valid_email(&email) {
        return ctx.err(400, "邮箱地址格式不正确");
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

    let sent = call_email_api(&ctx.config, title, &context, &email).await;

    if sent {
        ctx.ok("验证码已发送，请查收邮件", Value::Null)
    } else {
        ctx.err(500, "邮件发送失败，请稍后重试")
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
            static_dir: "".into(),
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
