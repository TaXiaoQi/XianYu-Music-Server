use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use hmac::{Hmac, Mac};
use md5::Md5;
use sha2::{Digest, Sha256};
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

pub fn md5_hex(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn hmac_sha256_hex(data: &[u8], secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

/// 兼容新旧两种签名：HMAC-SHA256 优先，失败再兼容旧 MD5
pub fn verify(timestamp: &str, nonce: &str, signature: &str, body: &str, secret: &str, tolerance: i64) -> bool {
    if timestamp.is_empty() || nonce.is_empty() || signature.is_empty() {
        return false;
    }
    let ts: i64 = timestamp.parse().unwrap_or(0);
    if (now_ts() - ts).abs() > tolerance {
        return false;
    }
    let hmac_expected = hmac_sha256_hex(format!("{}{}{}", timestamp, nonce, body).as_bytes(), secret);
    if hmac_expected.eq_ignore_ascii_case(signature) {
        return true;
    }
    let raw = format!("{}{}{}{}", timestamp, nonce, body, secret);
    let md5_expected = md5_hex(raw.as_bytes());
    md5_expected.eq_ignore_ascii_case(signature)
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

pub fn get_client_ip(x_forwarded_for: Option<&str>, x_real_ip: Option<&str>, remote_addr: Option<&str>) -> String {
    if let Some(ff) = x_forwarded_for {
        if let Some(first) = ff.split(',').next() {
            let t = first.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    if let Some(ri) = x_real_ip {
        let t = ri.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    remote_addr.unwrap_or("").to_string()
}
