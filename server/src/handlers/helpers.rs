use rand::Rng;
use serde_json::Value;
use sqlx::MySqlPool;
use sqlx::Row;

/// 解析请求体 JSON
pub fn parse_body(body: &str) -> Value {
    serde_json::from_str(body).unwrap_or(Value::Null)
}

pub fn str_of(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

pub fn int_of(v: &Value, key: &str) -> i64 {
    v.get(key)
        .and_then(|x| {
            x.as_i64()
                .or_else(|| x.as_str().and_then(|s| s.parse().ok()))
        })
        .unwrap_or(0)
}

#[allow(dead_code)]
pub fn bool_of(v: &Value, key: &str) -> bool {
    match v.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        Some(Value::String(s)) => s == "1" || s.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

/// 生成弦予号（顺序+1逻辑，从 1000 开始找第一个未使用的号）
pub async fn generate_ciyuanxi_id(pool: &MySqlPool) -> String {
    let mut used: std::collections::HashSet<i64> = std::collections::HashSet::new();

    if let Ok(rows) = sqlx::query("SELECT ciyuanxi_id FROM app_users WHERE ciyuanxi_id REGEXP '^[0-9]+$'")
        .fetch_all(pool)
        .await
    {
        for row in rows {
            if let Ok(v) = row.try_get::<String, _>("ciyuanxi_id") {
                if let Ok(n) = v.parse::<i64>() {
                    used.insert(n);
                }
            }
        }
    }
    if let Ok(rows) = sqlx::query("SELECT ciyuanxi_id FROM ciyuanxi_pretty_ids WHERE ciyuanxi_id REGEXP '^[0-9]+$'")
        .fetch_all(pool)
        .await
    {
        for row in rows {
            if let Ok(v) = row.try_get::<String, _>("ciyuanxi_id") {
                if let Ok(n) = v.parse::<i64>() {
                    used.insert(n);
                }
            }
        }
    }
    let mut id = 1000i64;
    while used.contains(&id) {
        id += 1;
    }
    id.to_string()
}

/// 解析版本号字符串为 [major, minor, patch]
pub fn parse_version_code(v: &str) -> Option<(i32, i32, i32)> {
    let re = regex::Regex::new(r"(\d+)\.(\d+)\.(\d+)").ok()?;
    let caps = re.captures(v)?;
    Some((
        caps.get(1)?.as_str().parse().unwrap_or(0),
        caps.get(2)?.as_str().parse().unwrap_or(0),
        caps.get(3)?.as_str().parse().unwrap_or(0),
    ))
}

/// 比较两个版本号：1 if a>b, -1 if a<b, 0 if equal/unparseable
pub fn compare_version_code(a: &str, b: &str) -> i32 {
    let pa = parse_version_code(a);
    let pb = parse_version_code(b);
    match (pa, pb) {
        (None, None) => 0,
        (None, Some(_)) => -1,
        (Some(_), None) => 1,
        (Some(pa), Some(pb)) => {
            if pa.0 != pb.0 {
                return if pa.0 > pb.0 { 1 } else { -1 };
            }
            if pa.1 != pb.1 {
                return if pa.1 > pb.1 { 1 } else { -1 };
            }
            if pa.2 != pb.2 {
                return if pa.2 > pb.2 { 1 } else { -1 };
            }
            0
        }
    }
}

pub fn random_hex(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| format!("{:02x}", rng.gen::<u8>())).collect()
}

pub fn random_int(min: i64, max: i64) -> i64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

/// 根据邮箱判定角色 member/admin/super_admin
pub async fn resolve_role_by_email(pool: &MySqlPool, email: &str) -> String {
    if email.is_empty() {
        return "member".to_string();
    }
    let row = sqlx::query("SELECT role FROM admin_users WHERE email = ? AND status = 1 LIMIT 1")
        .bind(email)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if let Some(r) = row {
        if let Ok(role) = r.try_get::<String, _>("role") {
            if role == "admin" || role == "super_admin" {
                return role;
            }
        }
    }
    "member".to_string()
}
