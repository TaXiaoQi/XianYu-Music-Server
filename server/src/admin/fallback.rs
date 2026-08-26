use axum::response::Response;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

/// 兜底模块 JSON 文件路径（相对 serve 根，与公告/版本配置同目录）
fn fallback_modules_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("fallback_modules.json")
}

/// 签名私钥来源：优先环境变量，其次 api/fallback_sign_key.txt（均存 hex 32 字节种子）。
/// 刻意不放进 Config/config.json，避免经后台「配置管理」页面读取/篡改私钥。
/// 客户端内嵌验签公钥（fd2f887e74ad...）与之成对，切勿在服务端之外泄露。
fn load_signing_private_key() -> Option<SigningKey> {
    let seed_hex = std::env::var("FALLBACK_SIGN_PRIVATE_KEY")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let path = std::path::Path::new("api").join("fallback_sign_key.txt");
            std::fs::read_to_string(path)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })?;
    let bytes: Vec<u8> = (0..seed_hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&seed_hex[i..i + 2], 16).ok())
        .collect();
    if bytes.len() != 32 {
        return None;
    }
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes);
    Some(SigningKey::from_bytes(&seed))
}

/// 签名消息构造：与服务端/客户端约定逐字一致（clinent 端 fallback_verify.rs 同构）。
/// 含 moduleKey + version 防止签名在不同模块/版本间复用。
fn module_signing_message(module_key: &str, version: i64, code: &str) -> Vec<u8> {
    format!("xianyu-fallback-v1\x00{module_key}\x00{version}\x00{code}").into_bytes()
}

/// 对模块签名，返回 hex（128 字符）。私钥未配置返回 None（由调用方决定拒绝保存）。
fn sign_module(module_key: &str, version: i64, code: &str) -> Option<String> {
    let key = load_signing_private_key()?;
    let sig = key.sign(&module_signing_message(module_key, version, code));
    Some(hex::encode(sig.to_bytes()))
}

/// 客户端与后台共同约定的模块 key 白名单（与桌面端 FALLBACK_MODULE_METHODS 一一对应）
pub const VALID_MODULE_KEYS: &[(&str, &str)] = &[
    ("lx_search", "落雪歌曲搜索"),
    ("lx_album", "专辑/歌单获取"),
    ("lx_duration", "歌曲时长加载"),
    ("lx_lyric", "逐字歌词解码"),
    ("lx_cover", "歌曲封面提取"),
    ("plugin_fallback", "插件宿主兜底"),
];

pub fn module_key_label(key: &str) -> &'static str {
    VALID_MODULE_KEYS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, name)| *name)
        .unwrap_or("未知模块")
}

pub fn is_valid_module_key(key: &str) -> bool {
    VALID_MODULE_KEYS.iter().any(|(k, _)| *k == key)
}

fn read_modules() -> Vec<Value> {
    let path = fallback_modules_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Value>(&content) {
            if let Some(arr) = v.as_array() {
                return arr
                    .iter()
                    .filter(|item| {
                        item.get("moduleKey").and_then(|v| v.as_str()).map(is_valid_module_key).unwrap_or(false)
                    })
                    .cloned()
                    .collect();
            }
        }
    }
    Vec::new()
}

fn write_modules(list: &[Value]) -> std::io::Result<()> {
    let path = fallback_modules_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let json = serde_json::to_string_pretty(list).unwrap_or_else(|_| "[]".to_string());
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)
}

pub fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn now_ymd_hms() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 公开端点数据：仅返回已启用模块，字段与桌面端 sync.ts 期望一致。
/// 每个模块附带 ed25519 签名（save 时落盘；缺失则按当前代码/版本实时重签）。
/// 客户端验签通过才允许执行；私钥未配置时 signature 为空串 → 客户端将丢弃该模块回退内置实现（fail-closed）。
pub fn enabled_modules_payload() -> Vec<Value> {
    read_modules()
        .into_iter()
        .filter(|m| m.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false))
        .map(|m| {
            let module_key = m.get("moduleKey").and_then(|v| v.as_str()).unwrap_or("");
            let version = m.get("version").and_then(|v| v.as_i64()).unwrap_or(1);
            let code = m.get("code").and_then(|v| v.as_str()).unwrap_or("");
            let stored_signature = m.get("signature").and_then(|v| v.as_str()).unwrap_or("");
            let signature = if !stored_signature.is_empty() {
                stored_signature.to_string()
            } else {
                sign_module(module_key, version, code).unwrap_or_default()
            };
            json!({
                "moduleKey": m.get("moduleKey").cloned().unwrap_or(Value::Null),
                "name": m.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                "version": version,
                "digest": m.get("digest").and_then(|v| v.as_str()).unwrap_or(""),
                "code": code,
                "signature": signature,
                "updatedAt": m.get("updated_at").and_then(|v| v.as_str()).unwrap_or(""),
            })
        })
        .collect()
}

/// 获取兜底模块列表（未配置的模块也返回占位，供后台展示全部可用模块）
pub async fn list(_body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let configured = read_modules();
    let list: Vec<Value> = VALID_MODULE_KEYS
        .iter()
        .map(|(key, label)| {
            let existing = configured.iter().find(|m| m.get("moduleKey").and_then(|v| v.as_str()) == Some(*key));
            match existing {
                Some(m) => m.clone(),
                None => json!({
                    "moduleKey": key,
                    "name": label,
                    "version": 0,
                    "digest": "",
                    "code": "",
                    "remark": "",
                    "enabled": false,
                    "configured": false,
                    "updated_at": "",
                }),
            }
        })
        .collect();
    ok("ok", json!({ "list": list }))
}

/// 新增/编辑兜底模块（按 moduleKey upsert）。
/// 代码变化时版本号自动 +1（客户端按 version 变化感知更新），代码不变则保持版本号。
pub async fn save(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let module_key = str_of(&data, "module_key").trim().to_string();
    let code = str_of(&data, "code").trim_end_matches('\n').to_string();
    let mut name = str_of(&data, "name").trim().to_string();
    let remark = str_of(&data, "remark").trim().to_string();
    let enabled = int_of(&data, "enabled") != 0;

    if !is_valid_module_key(&module_key) {
        return err(400, "模块标识无效");
    }
    if code.is_empty() {
        return err(400, "模块代码不能为空");
    }
    if code.len() > 512 * 1024 {
        return err(400, "模块代码过大（上限 512KB）");
    }
    if name.is_empty() {
        name = module_key_label(&module_key).to_string();
    }

    let digest = sha256_hex(&code);
    let mut list = read_modules();
    let existing = list
        .iter()
        .position(|m| m.get("moduleKey").and_then(|v| v.as_str()) == Some(module_key.as_str()));

    let (version, is_new, code_changed) = match existing {
        None => (1i64, true, true),
        Some(idx) => {
            let old = &list[idx];
            let old_version = old.get("version").and_then(|v| v.as_i64()).unwrap_or(0);
            let old_code = old.get("code").and_then(|v| v.as_str()).unwrap_or("");
            let changed = old_code != code;
            (if changed { old_version + 1 } else { old_version }, false, changed)
        }
    };

    // 签名：私钥未配置则拒绝保存，防止产出无法被客户端验签通过的模块。
    let signature = match sign_module(&module_key, version, &code) {
        Some(sig) => sig,
        None => return err(500, "服务端未配置兜底模块签名密钥（FALLBACK_SIGN_PRIVATE_KEY 或 api/fallback_sign_key.txt），无法保存"),
    };

    let item = json!({
        "moduleKey": module_key,
        "name": name,
        "version": version,
        "digest": digest,
        "code": code,
        "signature": signature,
        "remark": remark,
        "enabled": enabled,
        "configured": true,
        "created_at": if is_new { json!(now_ymd_hms()) } else { json!(Value::Null) },
        "updated_at": now_ymd_hms(),
    });

    match existing {
        None => list.push(item),
        Some(idx) => {
            let created_at = list[idx].get("created_at").cloned().unwrap_or(json!(now_ymd_hms()));
            let mut item = item;
            item["created_at"] = created_at;
            list[idx] = item;
        }
    }

    if write_modules(&list).is_err() {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }

    log_operation(
        pool,
        ctx,
        if is_new { "新增兜底模块" } else { "编辑兜底模块" },
        &module_key,
        &format!("v{} {} {}", version, if enabled { "启用" } else { "禁用" }, if code_changed { "代码已变更" } else { "代码未变更" }),
    )
    .await;
    ok(if code_changed { "保存成功，客户端将在下次拉取后生效" } else { "保存成功（代码未变更，版本号保持不变）" }, json!({
        "moduleKey": module_key,
        "version": version,
        "digest": digest,
    }))
}

/// 删除兜底模块（客户端下次拉取后回退内置默认实现）
pub async fn delete(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let module_key = str_of(&data, "module_key").trim().to_string();
    if module_key.is_empty() {
        return err(400, "参数错误");
    }
    let list = read_modules();
    let new_list: Vec<Value> = list
        .iter()
        .filter(|m| m.get("moduleKey").and_then(|v| v.as_str()) != Some(module_key.as_str()))
        .cloned()
        .collect();
    if new_list.len() == list.len() {
        return err(404, "模块未配置");
    }
    if write_modules(&new_list).is_err() {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    log_operation(pool, ctx, "删除兜底模块", &module_key, "客户端将回退内置实现").await;
    ok("删除成功，客户端将回退内置默认实现", Value::Null)
}

/// 切换兜底模块启用状态
pub async fn toggle(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let module_key = str_of(&data, "module_key").trim().to_string();
    let enabled = int_of(&data, "enabled") != 0;
    if module_key.is_empty() {
        return err(400, "参数错误");
    }
    let mut list = read_modules();
    let mut found = false;
    for m in list.iter_mut() {
        if m.get("moduleKey").and_then(|v| v.as_str()) == Some(module_key.as_str()) {
            m["enabled"] = json!(enabled);
            m["updated_at"] = json!(now_ymd_hms());
            found = true;
            break;
        }
    }
    if !found {
        return err(404, "模块未配置");
    }
    if write_modules(&list).is_err() {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    log_operation(pool, ctx, "切换兜底模块状态", &module_key, if enabled { "启用" } else { "禁用" }).await;
    ok(if enabled { "已启用，客户端下次拉取后生效" } else { "已禁用，客户端将回退内置实现" }, Value::Null)
}

#[cfg(test)]
mod sign_tests {
    use super::*;
    use ed25519_dalek::{Verifier, VerifyingKey};

    /// 与客户端 fallback_verify.rs 内嵌公钥逐字一致；若此常量与客户端不同步，签名将无法被客户端验证。
    const CLIENT_PUBLIC_KEY_HEX: &str =
        "fd2f887e74adb2009079bc822536d8f09d1404656f748289608592b6a4c974c5";

    fn load_key() -> SigningKey {
        load_signing_private_key().expect("必须能读取 api/fallback_sign_key.txt")
    }

    #[test]
    fn private_key_matches_client_public_key() {
        let pub_bytes = hex::decode(CLIENT_PUBLIC_KEY_HEX).unwrap();
        let published: [u8; 32] = pub_bytes.as_slice().try_into().unwrap();
        assert_eq!(load_key().verifying_key().to_bytes(), published);
    }

    #[test]
    fn server_signature_verifies_with_client_public_key() {
        let key = load_key();
        let module_key = "lx_lyric";
        let version = 4i64;
        let code = "function(){ return { version: 4 } }";
        let sig_str = sign_module(module_key, version, code).expect("签名");
        let sig_bytes = hex::decode(&sig_str).unwrap();
        let sig: ed25519_dalek::Signature =
            ed25519_dalek::Signature::from_bytes(sig_bytes.as_slice().try_into().unwrap());
        let vk = VerifyingKey::from_bytes(&key.verifying_key().to_bytes()).unwrap();
        let msg = module_signing_message(module_key, version, code);
        assert!(vk.verify(&msg, &sig).is_ok());
    }
}
