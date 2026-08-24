use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{bool_of, parse_body, str_of};

fn about_config_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("about_config.json")
}

/// 平台专属配置文件：desktop / mobile 分开存储，互不覆盖；
/// 不带 platform（旧后台）沿用默认 about_config.json。
fn platform_about_config_path(platform: &str) -> Option<std::path::PathBuf> {
    match platform {
        "desktop" => Some(std::path::Path::new("api").join("about_config_desktop.json")),
        "mobile" => Some(std::path::Path::new("api").join("about_config_mobile.json")),
        _ => None,
    }
}

fn default_about_config() -> Value {
    json!({
        "officialSiteUrl": "https://xymusic.cc",
        "officialSiteText": "前往官网",
        "updateEnabled": true,
        "updateText": "检查更新",
        "projectUrl": "https://github.com/TaXiaoQi/XianYu-Music-Desktop",
        "projectText": "开源地址",
        "referenceProjectUrl": "https://github.com/Billy636/XianYuMusic",
        "referenceProjectText": "参考项目",
        "joinGroupUrl": "https://qm.qq.com/q/kvteWSD8yY",
        "joinGroupText": "加入群组"
    })
}

fn read_about_config() -> Value {
    let defaults = default_about_config();
    let path = about_config_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return defaults;
    };
    let Ok(Value::Object(saved)) = serde_json::from_str::<Value>(&content) else {
        return defaults;
    };
    let mut merged = defaults.as_object().cloned().unwrap_or_default();
    for (key, value) in saved {
        merged.insert(key, value);
    }
    Value::Object(merged)
}

/// 平台感知默认值：移动端开源地址指向移动端仓库、参考项目指向桌面端仓库，
/// 与客户端 get_about_config 的下发逻辑保持一致。
fn default_about_config_for(platform: &str) -> Value {
    let mut config = default_about_config();
    if platform == "mobile" {
        crate::handlers::system::apply_mobile_about_overrides(&mut config);
    }
    config
}

/// 读取平台专属配置；无存档时回退共享配置并叠加平台默认覆盖，
/// 保证后台展示与客户端实际收到的配置一致。
fn read_platform_about_config(platform: &str) -> Value {
    let mut config = read_about_config();
    if platform == "mobile" {
        crate::handlers::system::apply_mobile_about_overrides(&mut config);
    }
    if let Some(path) = platform_about_config_path(platform) {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(Value::Object(saved)) = serde_json::from_str::<Value>(&content) {
                let mut merged = default_about_config_for(platform).as_object().cloned().unwrap_or_default();
                for (key, value) in saved {
                    merged.insert(key, value);
                }
                return Value::Object(merged);
            }
        }
    }
    config
}

fn write_about_config(config: &Value, platform: &str) -> std::io::Result<()> {
    let path = platform_about_config_path(platform).unwrap_or_else(about_config_path);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let json = serde_json::to_string_pretty(config).unwrap_or_else(|_| "{}".to_string());
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)
}

pub async fn get(body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let platform = str_of(&parse_body(body), "platform").trim().to_string();
    ok("ok", read_platform_about_config(&platform))
}

pub async fn save(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let platform = str_of(&data, "platform").trim().to_string();
    let official_site_url = str_of(&data, "officialSiteUrl").trim().to_string();
    let official_site_text = str_of(&data, "officialSiteText").trim().to_string();
    let update_text = str_of(&data, "updateText").trim().to_string();
    let project_url = str_of(&data, "projectUrl").trim().to_string();
    let project_text = str_of(&data, "projectText").trim().to_string();
    let reference_project_url = str_of(&data, "referenceProjectUrl").trim().to_string();
    let reference_project_text = str_of(&data, "referenceProjectText").trim().to_string();
    let join_group_url = str_of(&data, "joinGroupUrl").trim().to_string();
    let join_group_text = str_of(&data, "joinGroupText").trim().to_string();

    let config = json!({
        "officialSiteUrl": official_site_url,
        "officialSiteText": official_site_text,
        "updateEnabled": bool_of(&data, "updateEnabled"),
        "updateText": update_text,
        "projectUrl": project_url,
        "projectText": project_text,
        "referenceProjectUrl": reference_project_url,
        "referenceProjectText": reference_project_text,
        "joinGroupUrl": join_group_url,
        "joinGroupText": join_group_text,
    });

    if write_about_config(&config, &platform).is_err() {
        return err(500, "写入关于页配置失败，请检查 api 目录权限");
    }

    let platform_label = match platform.as_str() {
        "desktop" => "桌面端",
        "mobile" => "移动端",
        _ => "默认",
    };
    log_operation(pool, ctx, "保存关于页配置", "about_config", &format!("更新{platform_label}官网、更新检查、项目地址等入口")).await;
    ok("保存成功", config)
}
