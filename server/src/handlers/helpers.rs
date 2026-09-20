use rand::Rng;
use serde_json::Value;
use sqlx::MySqlPool;

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

pub fn bool_of(v: &Value, key: &str) -> bool {
    match v.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        Some(Value::String(s)) => s == "1" || s.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

pub fn is_valid_email(email: &str) -> bool {
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

pub fn extract_id(data: &Value) -> String {
    for key in ["ciyuanxi_id", "user_id", "id", "uid"] {
        let v = str_of(data, key);
        if !v.trim().is_empty() {
            return v.trim().to_string();
        }
        if let Some(n) = data.get(key).and_then(|x| x.as_i64()) {
            return n.to_string();
        }
    }
    String::new()
}

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

fn is_cjk_char(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
        || ('\u{3400}'..='\u{4dbf}').contains(&c)
        || ('\u{f900}'..='\u{faff}').contains(&c)
}

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

pub fn default_nickname(ciyuanxi_id: &str) -> String {
    format!("弦予{}", ciyuanxi_id)
}

pub fn parse_version_code(v: &str) -> Option<(i32, i32, i32)> {
    let re = regex::Regex::new(r"(\d+)\.(\d+)\.(\d+)").ok()?;
    let caps = re.captures(v)?;
    Some((
        caps.get(1)?.as_str().parse().unwrap_or(0),
        caps.get(2)?.as_str().parse().unwrap_or(0),
        caps.get(3)?.as_str().parse().unwrap_or(0),
    ))
}

fn has_prerelease(v: &str) -> bool {
    v.contains('-')
}

fn prerelease_number(v: &str) -> Option<i64> {
    let idx = v.find('-')?;
    let tail = &v[idx + 1..];
    tail.chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

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
            let a_pre = has_prerelease(a);
            let b_pre = has_prerelease(b);
            if a_pre != b_pre {
                return if a_pre { -1 } else { 1 };
            }
            if a_pre && b_pre {
                let an = prerelease_number(a).unwrap_or(0);
                let bn = prerelease_number(b).unwrap_or(0);
                if an != bn {
                    return if an > bn { 1 } else { -1 };
                }
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

pub async fn resolve_role_by_email(_pool: &MySqlPool, _email: &str) -> String {
    "member".to_string()
}

#[cfg(test)]
mod tests {
    use super::compare_version_code as cmp;

    #[test]
    fn prerelease_dot_separated_numbers_are_ordered() {
        assert_eq!(cmp("2.0.1-beta.6", "2.0.1-beta.5"), 1);
        assert_eq!(cmp("2.0.1-beta.5", "2.0.1-beta.6"), -1);
    }

    #[test]
    fn prerelease_number_separated_variants_are_ordered() {
        assert_eq!(cmp("2.0.1-beta6", "2.0.1-beta5"), 1);
        assert_eq!(cmp("2.0.1-beta-2", "2.0.1-beta-1"), 1);
    }

    #[test]
    fn stable_beats_prerelease_and_newer_patch_wins() {
        assert_eq!(cmp("2.0.1", "2.0.1-beta.9"), 1);
        assert_eq!(cmp("2.0.2", "2.0.1"), 1);
        assert_eq!(cmp("2.0.1-beta.5", "2.0.1-beta.5"), 0);
    }
}
