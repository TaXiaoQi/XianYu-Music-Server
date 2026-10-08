use super::*;

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

fn default_about_config() -> serde_json::Value {
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

async fn read_about_config(pool: &MySqlPool) -> serde_json::Value {
    let defaults = default_about_config();
    let Some(serde_json::Value::Object(saved)) = crate::site_config_store::get_json(pool, "about_config", Some(&about_config_path())).await else {
        return defaults;
    };
    let mut merged = defaults.as_object().cloned().unwrap_or_default();
    for (key, value) in saved {
        merged.insert(key, value);
    }
    serde_json::Value::Object(merged)
}

pub fn apply_mobile_about_overrides(config: &mut Value) {
    let Some(obj) = config.as_object_mut() else { return };
    let desktop_url = obj
        .get("projectUrl")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("https://github.com/TaXiaoQi/XianYu-Music-Desktop")
        .to_string();
    obj.insert("projectUrl".into(), json!("https://github.com/TaXiaoQi/XianYu-Music-Mobile"));
    obj.insert(
        "referenceProjects".into(),
        json!([
            { "name": "弦予音乐桌面端", "url": desktop_url },
            { "name": "BakaMusic", "url": "https://github.com/Zencok/BakaMusic" },
            { "name": "BiliPai", "url": "https://github.com/jay3-yy/BiliPai/releases" },
            { "name": "RawS", "url": "https://github.com/QFDY-GZC/RawS-Music" }
        ]),
    );
    obj.insert(
        "acknowledgements".into(),
        json!([
            { "name": "@Zencok", "url": "https://github.com/Zencok" },
            { "name": "@jay3-yy", "url": "https://github.com/jay3-yy" },
            { "name": "@QFDY-GZC", "url": "https://github.com/QFDY-GZC" }
        ]),
    );
}

pub fn apply_watch_about_overrides(config: &mut Value) {
    let Some(obj) = config.as_object_mut() else { return };
    obj.insert(
        "referenceProjects".into(),
        json!([
            { "name": "弦予音乐移动端", "url": "https://github.com/TaXiaoQi/XianYu-Music-Mobile" }
        ]),
    );
}

pub fn apply_platform_about_overrides(config: &mut Value, body: &str) {
    match str_of(&parse_body(body), "platform").trim() {
        "mobile" => apply_mobile_about_overrides(config),
        "watch" => apply_watch_about_overrides(config),
        _ => {}
    }
}

async fn read_platform_about_config(pool: &MySqlPool, platform: &str) -> Option<serde_json::Value> {
    let key = platform_about_key(platform)?;
    let saved = crate::site_config_store::get_json(pool, key, platform_about_config_path(platform).as_deref()).await?;
    let obj = saved.as_object()?.clone();
    let mut base = default_about_config();
    match platform {
        "mobile" => apply_mobile_about_overrides(&mut base),
        "watch" => apply_watch_about_overrides(&mut base),
        _ => {}
    }
    let mut merged = base.as_object().cloned().unwrap_or_default();
    for (key, value) in obj {
        merged.insert(key, value);
    }
    Some(serde_json::Value::Object(merged))
}

pub async fn get_about_config(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let platform = str_of(&parse_body(body), "platform").trim().to_string();
    if let Some(config) = read_platform_about_config(pool, &platform).await {
        return ctx.json(200, "ok", Some(config));
    }
    let mut config = read_about_config(pool).await;
    apply_platform_about_overrides(&mut config, body);
    ctx.json(200, "ok", Some(config))
}
