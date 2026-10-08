use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{bool_of, parse_body, str_of};
use crate::site_config_store;

fn about_config_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("about_config.json")
}

fn platform_about_config_path(platform: &str) -> Option<std::path::PathBuf> {
    match platform {
        "desktop" => Some(std::path::Path::new("api").join("about_config_desktop.json")),
        "mobile" => Some(std::path::Path::new("api").join("about_config_mobile.json")),
        "watch" => Some(std::path::Path::new("api").join("about_config_watch.json")),
        _ => None,
    }
}

fn platform_about_key(platform: &str) -> Option<&'static str> {
    match platform {
        "desktop" => Some("about_config_desktop"),
        "mobile" => Some("about_config_mobile"),
        "watch" => Some("about_config_watch"),
        _ => None,
    }
}

fn default_about_config() -> Value {
    json!({
        "officialSiteUrl": "https://xianyumusic.cn",
        "updateEnabled": true,
        "projectUrl": "https://github.com/TaXiaoQi/XianYu-Music-Desktop",
        "joinGroupUrl": "https://qm.qq.com/q/kvteWSD8yY",
        "referenceProjects": [
            { "name": "Lycia Player", "url": "https://github.com/Billy636/LyciaMusic" },
            { "name": "BakaMusic", "url": "https://github.com/Zencok/BakaMusic" }
        ],
        "acknowledgements": [
            { "name": "@Billy636", "url": "https://github.com/Billy636" },
            { "name": "@Zencok", "url": "https://github.com/Zencok" },
            { "name": "@kiomosu", "url": "https://github.com/kiomosu" }
        ]
    })
}

async fn read_about_config(pool: &MySqlPool) -> Value {
    let defaults = default_about_config();
    let Some(Value::Object(saved)) = site_config_store::get_json(pool, "about_config", Some(&about_config_path())).await else {
        return defaults;
    };
    let mut merged = defaults.as_object().cloned().unwrap_or_default();
    for (key, value) in saved {
        merged.insert(key, value);
    }
    Value::Object(merged)
}

fn default_about_config_for(platform: &str) -> Value {
    let mut config = default_about_config();
    match platform {
        "mobile" => crate::handlers::system::apply_mobile_about_overrides(&mut config),
        "watch" => crate::handlers::system::apply_watch_about_overrides(&mut config),
        _ => {}
    }
    config
}

async fn read_platform_about_config(pool: &MySqlPool, platform: &str) -> Value {
    let mut config = read_about_config(pool).await;
    match platform {
        "mobile" => crate::handlers::system::apply_mobile_about_overrides(&mut config),
        "watch" => crate::handlers::system::apply_watch_about_overrides(&mut config),
        _ => {}
    }
    let Some(key) = platform_about_key(platform) else {
        return config;
    };
    let Some(Value::Object(saved)) = site_config_store::get_json(pool, key, platform_about_config_path(platform).as_deref()).await else {
        return config;
    };
    let mut merged = default_about_config_for(platform).as_object().cloned().unwrap_or_default();
    for (key, value) in saved {
        merged.insert(key, value);
    }
    Value::Object(merged)
}

async fn write_about_config(pool: &MySqlPool, config: &Value, platform: &str) -> Result<(), sqlx::Error> {
    let key = platform_about_key(platform).unwrap_or("about_config");
    site_config_store::set_json(pool, key, config).await
}

pub async fn get(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let platform = str_of(&parse_body(body), "platform").trim().to_string();
    ok("ok", read_platform_about_config(pool, &platform).await)
}

fn name_url_list_of(data: &Value, key: &str, fallback: Vec<Value>) -> Vec<Value> {
    match data.get(key) {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|item| {
                let name = item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim())
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() {
                    return None;
                }
                let url = item
                    .get("url")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim())
                    .unwrap_or("")
                    .to_string();
                Some(json!({ "name": name, "url": url }))
            })
            .collect(),
        _ => fallback,
    }
}

pub async fn save(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let platform = str_of(&data, "platform").trim().to_string();
    let official_site_url = str_of(&data, "officialSiteUrl").trim().to_string();
    let project_url = str_of(&data, "projectUrl").trim().to_string();
    let join_group_url = str_of(&data, "joinGroupUrl").trim().to_string();

    let current = read_platform_about_config(pool, &platform).await;
    let fallback_list = |key: &str| -> Vec<Value> {
        current
            .get(key)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    };
    let reference_projects = name_url_list_of(&data, "referenceProjects", fallback_list("referenceProjects"));
    let acknowledgements = name_url_list_of(&data, "acknowledgements", fallback_list("acknowledgements"));

    let config = json!({
        "officialSiteUrl": official_site_url,
        "updateEnabled": bool_of(&data, "updateEnabled"),
        "projectUrl": project_url,
        "joinGroupUrl": join_group_url,
        "referenceProjects": reference_projects,
        "acknowledgements": acknowledgements,
    });

    if write_about_config(pool, &config, &platform).await.is_err() {
        return err(500, "数据库写入失败");
    }

    let platform_label = match platform.as_str() {
        "desktop" => "桌面端",
        "mobile" => "移动端",
        _ => "默认",
    };
    log_operation(pool, ctx, "保存关于页配置", "about_config", &format!("更新{platform_label}官网、更新检查、项目地址等入口")).await;
    ok("保存成功", config)
}
