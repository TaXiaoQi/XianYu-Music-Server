use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

pub fn aes_key(secret: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    let digest = sha256_bytes(secret.as_bytes());
    key.copy_from_slice(&digest);
    key
}

pub fn hmac_sha256_hex(data: &[u8], secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

/// 防重放：首次见到该 nonce 返回 false 并登记，窗口内重复出现返回 true。
/// 必须对每个请求只调用一次——调用方若对同一请求逐密钥各调一次 verify，
/// 第一次调用就会登记 nonce，后续密钥分支恒判重放（双钥过渡陷阱）。
pub fn nonce_replayed(nonce: &str, now: i64, window: i64) -> bool {
    static SEEN: OnceLock<Mutex<HashMap<String, i64>>> = OnceLock::new();
    let seen = SEEN.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = seen.lock().unwrap();
    map.retain(|_, ts| now - *ts <= window * 2);
    if map.contains_key(nonce) {
        return true;
    }
    map.insert(nonce.to_string(), now);
    false
}

/// 纯签名校验：空值 / 时间容差 / HMAC 比对，不登记 nonce。
/// 防重放由调用方在逐密钥比对前统一调用 nonce_replayed 一次。
pub fn verify_signature(
    timestamp: &str,
    nonce: &str,
    signature: &str,
    body: &str,
    secret: &str,
    tolerance: i64,
) -> bool {
    if timestamp.is_empty() || nonce.is_empty() || signature.is_empty() {
        return false;
    }
    let ts: i64 = timestamp.parse().unwrap_or(0);
    if (now_ts() - ts).abs() > tolerance {
        return false;
    }
    let hmac_expected = hmac_sha256_hex(format!("{}{}{}", timestamp, nonce, body).as_bytes(), secret);
    hmac_expected.eq_ignore_ascii_case(signature)
}

pub fn aes_encrypt(plaintext: &[u8], secret: &str) -> Option<(String, String)> {
    let key = aes_key(secret);
    let mut iv = [0u8; 16];
    use rand::RngCore;
    rand::thread_rng().fill_bytes(&mut iv);
    let mut buf = vec![0u8; plaintext.len() + 16];
    let ct = Aes256CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut buf, plaintext.len())
        .ok()?;
    Some((BASE64.encode(iv), BASE64.encode(ct)))
}

pub fn aes_decrypt(ciphertext_b64: &str, iv_b64: &str, secret: &str) -> Option<String> {
    let key = aes_key(secret);
    let iv_bytes = BASE64.decode(iv_b64).ok()?;
    if iv_bytes.len() != 16 {
        return None;
    }
    let mut iv = [0u8; 16];
    iv.copy_from_slice(&iv_bytes);
    let ct = BASE64.decode(ciphertext_b64).ok()?;
    let mut buf = vec![0u8; ct.len() + 16];
    let pt = Aes256CbcDec::new(&key.into(), &iv.into())
        .decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut buf)
        .ok()?;
    Some(String::from_utf8_lossy(pt).into_owned())
}

pub fn get_client_ip(
    x_forwarded_for: Option<&str>,
    x_real_ip: Option<&str>,
    remote_addr: Option<&str>,
    trust_proxy: bool,
) -> String {
    // 直连部署（trust_proxy=false）时请求头可被任意伪造，只信 TCP 对端
    if !trust_proxy {
        return remote_addr.unwrap_or("").to_string();
    }
    // 受信反代之后的信任顺序：X-Real-IP（应由反代覆盖设置，客户端伪造无效）
    // > X-Forwarded-For 最后一段（直连反代追加的那跳，前面的段均可被客户端伪造）> TCP 对端
    if let Some(ri) = x_real_ip {
        let t = ri.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    if let Some(ff) = x_forwarded_for {
        if let Some(last) = ff.rsplit(',').next() {
            let t = last.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    remote_addr.unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 回归测试（V-01 双钥过渡）：重放检查必须每请求只做一次。
    /// 若逐密钥各调一次带 nonce 登记的 verify，第一次（当前密钥）调用会
    /// 登记 nonce，第二次（旧密钥）调用恒判重放，旧客户端永远 403。
    #[test]
    fn dual_key_transition_accepts_previous_secret() {
        let now = now_ts();
        let ts = now.to_string();
        let nonce = format!("dual-key-{}", now);
        let body = r#"{"action":"get_music_list"}"#;
        let current_secret = "current-secret-0123456789abcdef";
        let previous_secret = "previous-secret-fedcba9876543210";

        // 旧密钥签出的请求：当前密钥校验失败，旧密钥必须通过
        let sig = hmac_sha256_hex(format!("{ts}{nonce}{body}").as_bytes(), previous_secret);

        // main.rs 双钥路径的等价逻辑：先统一登记 nonce，再逐密钥比对
        assert!(!nonce_replayed(&nonce, now, 900), "首次请求不得判重放");
        assert!(
            !verify_signature(&ts, &nonce, &sig, body, current_secret, 900),
            "当前密钥应校验失败"
        );
        assert!(
            verify_signature(&ts, &nonce, &sig, body, previous_secret, 900),
            "旧密钥必须通过（重放检查不得吃掉后续密钥分支）"
        );

        // 同 nonce 重放仍被拒绝
        assert!(nonce_replayed(&nonce, now, 900), "同 nonce 重放必须拒绝");
    }

    #[test]
    fn verify_signature_rejects_bad_input() {
        let now = now_ts();
        let ts = now.to_string();
        let body = "{}";
        let secret = "some-secret";
        let sig = hmac_sha256_hex(format!("{ts}n{body}").as_bytes(), secret);
        assert!(!verify_signature("", "n", &sig, body, secret, 900));
        assert!(!verify_signature(&ts, "", &sig, body, secret, 900));
        assert!(!verify_signature(&ts, "n", "", body, secret, 900));
        assert!(verify_signature(&ts, "n", &sig, body, secret, 900), "正确密钥应通过");
        assert!(!verify_signature(&ts, "n", &sig, body, "wrong-secret", 900), "错误密钥应拒绝");
        assert!(
            !verify_signature(&(now - 3600).to_string(), "n", &sig, body, secret, 900),
            "超容差时间戳应拒绝"
        );
    }
}
