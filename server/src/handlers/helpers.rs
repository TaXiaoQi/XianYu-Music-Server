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
/// 注：新流程改为用户自行填写弦予号，此函数仅作向后兼容保留
#[allow(dead_code)]
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

/// 校验弦予号（支持纯数字、纯字母、数字字母组合，支持大小写字母，6-20 个字符，不含特殊符号）
pub fn validate_ciyuanxi_id(id: &str) -> Result<(), &'static str> {
    let len = id.chars().count();
    if len < 6 || len > 20 {
        return Err("弦予号长度为 6-20 个字符");
    }
    for c in id.chars() {
        if !c.is_ascii_alphanumeric() {
            return Err("弦予号只能包含字母或数字");
        }
    }
    Ok(())
}

/// 判断字符是否为汉字（CJK 统一表意文字基本区）
fn is_cjk_char(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
        || ('\u{3400}'..='\u{4dbf}').contains(&c)
        || ('\u{f900}'..='\u{faff}').contains(&c)
}

/// 校验昵称：仅允许字母、数字、汉字三种字符，长度在 [min, max] 之间
pub fn validate_nickname(nickname: &str, min: usize, max: usize) -> Result<(), &'static str> {
    let len = nickname.chars().count();
    if len < min || len > max {
        return Err("昵称长度不符合要求");
    }
    for c in nickname.chars() {
        if !c.is_ascii_alphanumeric() && !is_cjk_char(c) {
            return Err("昵称仅支持字母、数字、汉字");
        }
    }
    Ok(())
}

/// 默认昵称：弦予 + 弦予号（如 弦予161）
pub fn default_nickname(ciyuanxi_id: &str) -> String {
    format!("弦予{}", ciyuanxi_id)
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
/// 管理员账号已去除邮箱，统一返回 member
pub async fn resolve_role_by_email(_pool: &MySqlPool, _email: &str) -> String {
    "member".to_string()
}
