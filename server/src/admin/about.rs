use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{bool_of, parse_body, str_of};

fn about_config_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("about_config.json")
}

fn default_about_config() -> Value {
    json!({
        "officialSiteUrl": "https://xy.zh2026.cn/ciyuanxi/",
        "officialSiteText": "前往官网",
        "updateEnabled": true,
        "updateText": "检查更新",
        "projectUrl": "https://github.com/TaXiaoQi/XianYu-Music-Desktop",
        "projectText": "开源地址",
        "referenceProjectUrl": "https://github.com/Billy636/XianYuMusic",
        "referenceProjectText": "参考项目"
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

fn write_about_config(config: &Value) -> std::io::Result<()> {
    let path = about_config_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let json = serde_json::to_string_pretty(config).unwrap_or_else(|_| "{}".to_string());
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)
}

pub async fn get(_body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    ok("ok", read_about_config())
}

pub async fn save(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let official_site_url = str_of(&data, "officialSiteUrl").trim().to_string();
    let official_site_text = str_of(&data, "officialSiteText").trim().to_string();
    let update_text = str_of(&data, "updateText").trim().to_string();
    let project_url = str_of(&data, "projectUrl").trim().to_string();
    let project_text = str_of(&data, "projectText").trim().to_string();
    let reference_project_url = str_of(&data, "referenceProjectUrl").trim().to_string();
    let reference_project_text = str_of(&data, "referenceProjectText").trim().to_string();

    let config = json!({
        "officialSiteUrl": official_site_url,
        "officialSiteText": official_site_text,
        "updateEnabled": bool_of(&data, "updateEnabled"),
        "updateText": update_text,
        "projectUrl": project_url,
        "projectText": project_text,
        "referenceProjectUrl": reference_project_url,
        "referenceProjectText": reference_project_text,
    });

    if write_about_config(&config).is_err() {
        return err(500, "写入关于页配置失败，请检查 api 目录权限");
    }

    log_operation(pool, ctx, "保存关于页配置", "about_config", "更新官网、更新检查、项目地址等入口").await;
    ok("保存成功", config)
}
