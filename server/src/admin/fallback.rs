use std::collections::BTreeMap;

use axum::response::Response;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{json, Map, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, AdminCtx};
use crate::config::Config;
use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::signer;
use crate::site_config_store;

/// 兜底行为配置存储键（旧 "fallback_modules" 存的是热修 JS 模块，已废弃，
/// 不再读取；客户端只认这里的签名配置）。
const CONFIG_STORE_KEY: &str = "fallback_config";

/// 配置有效期（unix 秒，签发时刻 + TTL 写入配置顶层 expiresAt）。
/// 客户端把过期配置等同验签失败处理（清缓存回默认）：即使服务端长期停摆，
/// 缓存配置也不会被无限期沿用（如密钥泄露轮换后旧配置自然失效）。
const CONFIG_EXPIRES_TTL_SECS: i64 = 30 * 24 * 3600;

fn config_meta_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("fallback_config.json")
}

/// A 档退化模式本机私钥（env FALLBACK_SIGN_PRIVATE_KEY 或 api/fallback_sign_key.txt）。
/// B 档托管（fallback.signer.enabled）时主服务不持有私钥，签发走签发服务。
pub(crate) fn load_signing_private_key() -> Option<SigningKey> {
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

/// A 档本机签名（无 keyId 概念，客户端按遍历槽位验签）
fn local_sign(message: &[u8]) -> Option<String> {
    let key = load_signing_private_key()?;
    Some(hex::encode(key.sign(message).to_bytes()))
}

/// 统一签发入口：B 档走签发服务（fail closed，签发失败返回 None），A 档本机签名。
/// 返回 (signature_hex, keyId)；A 档 keyId 为空串（下发时省略 keyId 字段）。
pub(crate) async fn sign_message(config: &Config, message: &[u8]) -> Option<(String, String)> {
    if config.fallback.signer.enabled {
        match signer::client_sign(&config.fallback.signer, message).await {
            Ok(pair) => Some(pair),
            Err(e) => {
                tracing::error!("兜底配置签发失败（签发服务）: {e}");
                None
            }
        }
    } else {
        local_sign(message).map(|sig| (sig, String::new()))
    }
}

fn sign_unavailable_reason(config: &Config) -> &'static str {
    if config.fallback.signer.enabled {
        "签发服务不可用：请确认 --signer 进程存活，以及 fallback.signer.url/token 配置一致"
    } else {
        "服务端未配置签名密钥：B 档请开启 fallback.signer 并启动 --signer 进程；A 档请配置 FALLBACK_SIGN_PRIVATE_KEY 或 api/fallback_sign_key.txt"
    }
}

/// 可下发配置的模块：`(key, 管理端显示名, 方法白名单, 是否支持平台来源参数)`。
/// 客户端解析时同样按白名单过滤（新增 key/方法需两端同步）。
pub const VALID_MODULES: &[(&str, &str, &[&str], bool)] = &[
    ("lx_lyric", "逐字歌词解码", &["fetchLyric"], false),
    (
        "lx_toplist",
        "落雪音源榜单",
        &["getTopLists", "getTopListDetail"],
        true,
    ),
    (
        "lx_recommend_sheets",
        "推荐歌单",
        &["getRecommendSheetTags", "getRecommendSheetsByTag"],
        true,
    ),
    (
        "plugin_fallback",
        "插件宿主兜底",
        &[
            "hostSearchFallback",
            "hostAlbumSearchFallback",
            "hostAlbumSongsFallback",
            "fillSongDurations",
        ],
        false,
    ),
    (
        "playlist_import",
        "歌单导入",
        &[
            "getListDetailKg",
            "getListDetailWy",
            "getListDetailTx",
            "getListDetailKw",
            "getListDetailMg",
            "getListDetailQishui",
        ],
        false,
    ),
];

/// 支持平台来源参数的模块（params.sources，wy/kg/kw/tx 子集）
const SOURCE_PLATFORMS: &[&str] = &["wy", "kg", "kw", "tx"];

pub fn module_def(key: &str) -> Option<&'static (&'static str, &'static str, &'static [&'static str], bool)> {
    VALID_MODULES.iter().find(|(k, ..)| *k == key)
}

fn now_ymd_hms() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// ==================== 存储文档 ====================
// { enabled, configVersion, configJson, signature, keyId, meta: { key: { remark, updated_at } }, updated_at }
// configJson 是签名对象（客户端收到的原始字符串，顶层含 expiresAt），
// keyId 是签署当前配置的槽位标识（A 档本机签名为空串），meta 仅管理端使用、不下发。

#[derive(Default)]
struct StoredConfig {
    enabled: bool,
    config_version: i64,
    config_json: String,
    signature: String,
    key_id: String,
    meta: Map<String, Value>,
    updated_at: String,
}

async fn read_config(pool: &MySqlPool) -> StoredConfig {
    let mut cfg = StoredConfig {
        enabled: false,
        ..Default::default()
    };
    if let Some(Value::Object(obj)) =
        site_config_store::get_json(pool, CONFIG_STORE_KEY, Some(&config_meta_path())).await
    {
        cfg.enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
        cfg.config_version = obj.get("configVersion").and_then(|v| v.as_i64()).unwrap_or(0);
        cfg.config_json = obj
            .get("configJson")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        cfg.signature = obj
            .get("signature")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        cfg.key_id = obj
            .get("keyId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if let Some(Value::Object(meta)) = obj.get("meta") {
            cfg.meta = meta.clone();
        }
        cfg.updated_at = obj
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
    }
    cfg
}

async fn write_config(pool: &MySqlPool, cfg: &StoredConfig) -> Result<(), sqlx::Error> {
    site_config_store::set_json(
        pool,
        CONFIG_STORE_KEY,
        &json!({
            "enabled": cfg.enabled,
            "configVersion": cfg.config_version,
            "configJson": cfg.config_json,
            "signature": cfg.signature,
            "keyId": cfg.key_id,
            "meta": cfg.meta,
            "updated_at": cfg.updated_at,
        }),
    )
    .await
}

// ==================== 配置文档构建（canonical） ====================
// {"version":N,"expiresAt":<unix秒>,"modules":{"<key>":{"version":n,"disabledMethods":[...],"params":{"sources":[...]}}}}
// serde_json Map 默认 BTreeMap（键序稳定），同一份模块状态重建结果逐字节一致，
// 签名可复现。空 disabledMethods/无 params 的模块不进文档（等同默认行为）；
// expiresAt 为签发时刻 + TTL（保存即重签，每次顺延）。

fn parse_str_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// 从已存 configJson 解析模块条目：key -> (version, disabledMethods, sources)
fn parse_module_entries(config_json: &str) -> BTreeMap<String, (i64, Vec<String>, Vec<String>)> {
    let mut out = BTreeMap::new();
    let Ok(Value::Object(doc)) = serde_json::from_str::<Value>(config_json) else {
        return out;
    };
    let Some(Value::Object(modules)) = doc.get("modules") else {
        return out;
    };
    for (key, entry) in modules {
        let Some(entry_obj) = entry.as_object() else { continue };
        let version = entry_obj.get("version").and_then(|v| v.as_i64()).unwrap_or(1);
        let disabled = parse_str_list(entry_obj.get("disabledMethods"));
        let sources = entry_obj
            .get("params")
            .and_then(|p| p.get("sources"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        out.insert(key.clone(), (version, disabled, sources));
    }
    out
}

fn build_config_json(
    config_version: i64,
    entries: &BTreeMap<String, (i64, Vec<String>, Vec<String>)>,
    expires_at: Option<i64>,
) -> String {
    let mut modules = Map::new();
    for (key, (version, disabled, sources)) in entries {
        if disabled.is_empty() && sources.is_empty() {
            continue;
        }
        let mut entry = Map::new();
        entry.insert("version".into(), json!(version));
        entry.insert("disabledMethods".into(), json!(disabled));
        if !sources.is_empty() {
            entry.insert("params".into(), json!({ "sources": sources }));
        }
        modules.insert(key.clone(), Value::Object(entry));
    }
    let mut doc = Map::new();
    doc.insert("version".into(), json!(config_version));
    if let Some(exp) = expires_at {
        doc.insert("expiresAt".into(), json!(exp));
    }
    doc.insert("modules".into(), Value::Object(modules));
    serde_json::to_string(&Value::Object(doc)).unwrap_or_else(|_| "{}".to_string())
}

// ==================== 管理端动作 ====================

pub async fn list(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let cfg = read_config(pool).await;
    let entries = parse_module_entries(&cfg.config_json);
    let list: Vec<Value> = VALID_MODULES
        .iter()
        .map(|(key, label, methods, has_sources)| {
            let entry = entries.get(*key);
            let meta = cfg.meta.get(*key).and_then(|v| v.as_object());
            json!({
                "moduleKey": key,
                "label": label,
                "methods": methods,
                "sourcesSelectable": has_sources,
                "configured": entry.is_some(),
                "version": entry.map(|(v, ..)| *v).unwrap_or(0),
                "disabledMethods": entry.map(|(_, d, _)| d).cloned().unwrap_or_default(),
                "sources": entry.map(|(_, _, s)| s).cloned().unwrap_or_default(),
                "remark": meta.and_then(|m| m.get("remark")).and_then(|v| v.as_str()).unwrap_or(""),
                "updated_at": meta.and_then(|m| m.get("updated_at")).and_then(|v| v.as_str()).unwrap_or(""),
            })
        })
        .collect();
    ok(
        "ok",
        json!({
            "enabled": cfg.enabled,
            "configVersion": cfg.config_version,
            "signature": cfg.signature,
            "configKeyId": cfg.key_id,
            "updatedAt": cfg.updated_at,
            "sourcePlatforms": SOURCE_PLATFORMS,
            "list": list,
        }),
    )
}

pub async fn save(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let module_key = str_of(&data, "module_key").trim().to_string();
    let remark = str_of(&data, "remark").trim().to_string();
    let disabled_methods = parse_str_list(data.get("disabled_methods"));
    let sources = parse_str_list(data.get("sources"));

    let Some((_, _, methods, has_sources)) = module_def(&module_key).map(|d| (d.0, d.1, d.2, d.3)) else {
        return err(400, "模块标识无效");
    };
    if disabled_methods.iter().any(|m| !methods.contains(&m.as_str())) {
        return err(400, "存在不在白名单内的方法名");
    }
    if sources.iter().any(|s| !SOURCE_PLATFORMS.contains(&s.as_str())) {
        return err(400, "平台来源仅支持 wy/kg/kw/tx");
    }
    if !has_sources && !sources.is_empty() {
        return err(400, "该模块不支持平台来源参数");
    }
    if !has_sources && sources.len() > SOURCE_PLATFORMS.len() {
        return err(400, "平台来源数量超出上限");
    }

    let mut cfg = read_config(pool).await;
    let mut entries = parse_module_entries(&cfg.config_json);
    let is_new_entry = !entries.contains_key(&module_key);
    let disabled_desc = disabled_methods.join(",");
    let sources_desc = sources.join(",");

    // 全空 = 该模块恢复默认：不进文档
    let keep = !disabled_methods.is_empty() || !sources.is_empty();
    if keep {
        let old_version = entries.get(&module_key).map(|(v, ..)| *v).unwrap_or(0);
        let next_version = if is_new_entry { 1 } else { old_version + 1 };
        entries.insert(module_key.clone(), (next_version, disabled_methods, sources));
    } else {
        entries.remove(&module_key);
    }

    let next_config_version = cfg.config_version + 1;
    let expires_at = chrono::Utc::now().timestamp() + CONFIG_EXPIRES_TTL_SECS;
    let config_json = build_config_json(next_config_version, &entries, Some(expires_at));
    let Some((signature, key_id)) = sign_message(&ctx.config, &signer::config_signing_message(&config_json)).await
    else {
        return err(500, sign_unavailable_reason(&ctx.config));
    };

    cfg.config_version = next_config_version;
    cfg.config_json = config_json;
    cfg.signature = signature;
    cfg.key_id = key_id;
    cfg.updated_at = now_ymd_hms();
    if keep {
        cfg.meta.insert(
            module_key.clone(),
            json!({ "remark": remark, "updated_at": cfg.updated_at }),
        );
    } else {
        cfg.meta.remove(&module_key);
    }

    if write_config(pool, &cfg).await.is_err() {
        return err(500, "数据库写入失败");
    }

    log_operation(
        pool,
        ctx,
        if keep { "保存兜底配置" } else { "重置兜底配置" },
        &module_key,
        &format!(
            "配置v{} {}",
            next_config_version,
            if keep {
                format!("禁用方法[{}] 来源[{}]", disabled_desc, sources_desc)
            } else {
                "恢复默认".to_string()
            }
        ),
    )
    .await;
    ok(
        if keep {
            "保存成功，客户端将在下次拉取后生效"
        } else {
            "已恢复默认，客户端将在下次拉取后生效"
        },
        json!({ "moduleKey": module_key, "configVersion": next_config_version }),
    )
}

pub async fn delete(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let module_key = str_of(&data, "module_key").trim().to_string();
    if module_def(&module_key).is_none() {
        return err(400, "模块标识无效");
    }

    let mut cfg = read_config(pool).await;
    let mut entries = parse_module_entries(&cfg.config_json);
    if entries.remove(&module_key).is_none() && cfg.meta.remove(&module_key).is_none() {
        return err(404, "该模块未配置");
    }

    let next_config_version = cfg.config_version + 1;
    let expires_at = chrono::Utc::now().timestamp() + CONFIG_EXPIRES_TTL_SECS;
    let config_json = build_config_json(next_config_version, &entries, Some(expires_at));
    let Some((signature, key_id)) = sign_message(&ctx.config, &signer::config_signing_message(&config_json)).await
    else {
        return err(500, sign_unavailable_reason(&ctx.config));
    };
    cfg.config_version = next_config_version;
    cfg.config_json = config_json;
    cfg.signature = signature;
    cfg.key_id = key_id;
    cfg.updated_at = now_ymd_hms();

    if write_config(pool, &cfg).await.is_err() {
        return err(500, "数据库写入失败");
    }
    log_operation(pool, ctx, "重置兜底配置", &module_key, "客户端将回退默认行为").await;
    ok("已恢复默认，客户端将在下次拉取后生效", Value::Null)
}

/// 总开关：关闭后客户端拉取到空配置，全部回退默认行为（配置本体保留）。
pub async fn toggle(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let enabled = int_of(&data, "enabled") != 0;
    let mut cfg = read_config(pool).await;
    if cfg.config_json.is_empty() && enabled {
        return err(400, "尚未保存任何模块配置，无需开启");
    }
    cfg.enabled = enabled;
    cfg.updated_at = now_ymd_hms();
    if write_config(pool, &cfg).await.is_err() {
        return err(500, "数据库写入失败");
    }
    log_operation(
        pool,
        ctx,
        "切换兜底配置下发",
        "fallback_config",
        if enabled { "开启下发" } else { "暂停下发" },
    )
    .await;
    ok(
        if enabled {
            "已开启下发，客户端下次拉取后生效"
        } else {
            "已暂停下发，客户端拉取到空配置后将回退默认行为"
        },
        Value::Null,
    )
}

// ==================== 客户端下发 ====================

/// 客户端拉取：返回 { config, signature, keyId? }；未启用/未配置时返回空串，
/// 客户端按「无配置」处理（全部默认行为）。keyId 仅 B 档托管签名时随响应
/// 下发（客户端先按 keyId 精确匹配公钥槽位，缺失/未知则遍历）。
pub(crate) async fn client_config_payload(pool: &MySqlPool) -> (String, String, String) {
    let cfg = read_config(pool).await;
    if !cfg.enabled || cfg.config_json.is_empty() || cfg.signature.is_empty() {
        return (String::new(), String::new(), String::new());
    }
    (cfg.config_json, cfg.signature, cfg.key_id)
}

// ==================== 签名密钥管理（管理端） ====================

/// 签名密钥槽位状态（管理页「签名密钥」卡片数据源）：
/// - B 档托管：代理查询签发服务（active/previous/available/activePublicKey）
/// - A 档本机：mode=local，附验签公钥 hex（公钥可展示）
/// - 未配置：mode=none
pub async fn signer_status(_body: &str, ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    if ctx.config.fallback.signer.enabled {
        return match signer::client_keys(&ctx.config.fallback.signer).await {
            Ok(keys) => {
                let mut payload = json!({ "mode": "signer" });
                if let Value::Object(map) = keys {
                    for (k, v) in map {
                        payload[k] = v;
                    }
                }
                ok("ok", payload)
            }
            Err(e) => ok(
                "ok",
                json!({ "mode": "signer", "signerError": e }),
            ),
        };
    }
    if let Some(key) = load_signing_private_key() {
        return ok(
            "ok",
            json!({
                "mode": "local",
                "active": "",
                "previous": null,
                "available": [],
                "activePublicKey": hex::encode(key.verifying_key().to_bytes()),
            }),
        );
    }
    ok("ok", json!({ "mode": "none" }))
}

/// 一键切换签发槽位（仅 B 档托管）：旧 active 移入 previous。操作落管理员审计。
pub async fn signer_switch(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    if !ctx.config.fallback.signer.enabled {
        return err(400, "当前为 A 档本机密钥模式（或未启用签发服务），无槽位可切换；请先启用 fallback.signer 并启动 --signer 进程");
    }
    let data = parse_body(body);
    let key_id = str_of(&data, "key_id").trim().to_string();
    if key_id.is_empty() || key_id.len() > 32 {
        return err(400, "keyId 非法");
    }
    match signer::client_switch(&ctx.config.fallback.signer, &key_id).await {
        Ok(keys) => {
            let active = keys.get("active").and_then(|v| v.as_str()).unwrap_or("");
            let previous = keys.get("previous").and_then(|v| v.as_str()).unwrap_or("-");
            log_operation(
                pool,
                ctx,
                "切换兜底签名密钥",
                &key_id,
                &format!("active={active} previous={previous}"),
            )
            .await;
            ok(
                "已切换签发槽位，新配置保存时将以新密钥签发；请按轮换流程完成预发布与收尾",
                keys,
            )
        }
        Err(e) => err(502, &e),
    }
}

#[cfg(test)]
mod sign_tests {
    use super::*;
    use ed25519_dalek::{Verifier, VerifyingKey};

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
        let config_json = r#"{"version":1,"modules":{"lx_lyric":{"version":1,"disabledMethods":["fetchLyric"]}}}"#;
        let msg = signer::config_signing_message(config_json);
        let sig_str = local_sign(&msg).expect("签名");
        let sig_bytes = hex::decode(&sig_str).unwrap();
        let sig: ed25519_dalek::Signature =
            ed25519_dalek::Signature::from_bytes(sig_bytes.as_slice().try_into().unwrap());
        let vk = VerifyingKey::from_bytes(&key.verifying_key().to_bytes()).unwrap();
        assert!(vk.verify(&msg, &sig).is_ok());
    }

    #[test]
    fn config_json_build_is_canonical() {
        let mut a = BTreeMap::new();
        a.insert("lx_lyric".to_string(), (1i64, vec!["fetchLyric".to_string()], Vec::new()));
        let mut b = BTreeMap::new();
        b.insert("lx_lyric".to_string(), (1i64, vec!["fetchLyric".to_string()], Vec::new()));
        assert_eq!(build_config_json(3, &a, None), build_config_json(3, &b, None));
        let doc: Value = serde_json::from_str(&build_config_json(1, &a, Some(1767225600))).unwrap();
        assert_eq!(doc["version"], 1);
        assert_eq!(doc["expiresAt"], 1767225600);
        assert_eq!(doc["modules"]["lx_lyric"]["disabledMethods"][0], "fetchLyric");
        // 不带 expiresAt 时不下发该字段（自部署零破坏）
        let doc2: Value = serde_json::from_str(&build_config_json(1, &a, None)).unwrap();
        assert!(doc2.get("expiresAt").is_none());
    }
}
