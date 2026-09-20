use axum::response::Response;
use base64::Engine;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, row_to_value, AdminCtx};
use crate::handlers::helpers::{compare_version_code, int_of, parse_body, str_of};

pub async fn add_version(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let app_name = str_of(&data, "app_name").trim().to_string();
    let version_code = str_of(&data, "version_code").trim().to_string();
    let update_content = str_of(&data, "update_content").trim().to_string();
    let file_data = str_of(&data, "file_data").trim().to_string();

    if app_name.is_empty() || version_code.is_empty() {
        return err(400, "软件名称和版本号不能为空");
    }
    if file_data.is_empty() {
        return err(400, "请上传安装包");
    }

    let file_bytes = match base64::engine::general_purpose::STANDARD.decode(&file_data) {
        Ok(b) => b,
        Err(_) => return err(400, "文件数据解码失败"),
    };
    if file_bytes.is_empty() {
        return err(400, "文件为空");
    }

    let upload_dir = std::path::Path::new("uploads").join("apk");
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        { tracing::error!("无法创建上传目录: {e}"); return err(500, "无法创建上传目录"); }
    }

    let insert = sqlx::query(
        "INSERT INTO app_versions (app_name, version_code, download_url, update_content, status, file_size) VALUES (?, ?, '', ?, 'normal', ?)",
    )
    .bind(&app_name)
    .bind(&version_code)
    .bind(&update_content)
    .bind(file_bytes.len() as i64)
    .execute(pool)
    .await;

    let version_id = match insert {
        Ok(r) => r.last_insert_id() as i64,
        Err(e) => return { tracing::error!("数据库错误: {e}"); err(500, "数据库错误") },
    };

    let new_filename = format!("app_v{}.apk", version_id);
    let target_path = upload_dir.join(&new_filename);
    if let Err(_) = std::fs::write(&target_path, &file_bytes) {
        let _ = sqlx::query("DELETE FROM app_versions WHERE id = ?")
            .bind(version_id)
            .execute(pool)
            .await;
        return err(500, "文件保存失败，请检查目录权限");
    }

    let download_url = format!("/uploads/apk/{}", new_filename);
    let _ = sqlx::query("UPDATE app_versions SET download_url = ?, file_size = ? WHERE id = ?")
        .bind(&download_url)
        .bind(file_bytes.len() as i64)
        .bind(version_id)
        .execute(pool)
        .await;

    log_operation(pool, ctx, "新增版本", &app_name, &format!("版本号:{}", version_code)).await;
    ok("添加成功", json!({
        "download_url": download_url,
        "file_size": file_bytes.len()
    }))
}

pub async fn list_versions(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let page = int_of(&data, "page").max(1);
    let page_size = {
        let ps = int_of(&data, "page_size");
        if ps == 0 { 15 } else { ps.clamp(1, 100) }
    };
    let offset = (page - 1) * page_size;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_versions")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let list_sql = "SELECT * FROM app_versions ORDER BY created_at DESC LIMIT ? OFFSET ?";
    match sqlx::query(list_sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_value).collect();
            let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;
            ok("ok", json!({
                "total": total,
                "page": page,
                "page_size": page_size,
                "total_pages": total_pages,
                "list": list,
            }))
        }
        Err(e) => { tracing::error!("查询失败: {e}"); err(500, "查询失败") },
    }
}

fn desktop_version_path() -> std::path::PathBuf {
    std::path::Path::new("api").join("version.json")
}

fn safe_version_part(version: &str) -> String {
    let s: String = version
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
        .collect();
    if s.is_empty() { "latest".to_string() } else { s }
}

fn safe_file_ext(file_name: &str) -> String {
    let ext = file_name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    if ext.is_empty() { "bin".to_string() } else { ext }
}

fn safe_file_stem(file_name: &str) -> String {
    let base = file_name.rsplit(|c| c == '/' || c == '\\').next().unwrap_or("");
    let stem = base.rsplit_once('.').map(|(s, _)| s).unwrap_or(base);
    let s: String = stem
        .chars()
        .take(80)
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
        .collect();
    s.trim_matches(|c| c == '.' || c == '_' || c == '-').to_string()
}

pub async fn update_version(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let app_name = str_of(&data, "app_name").trim().to_string();
    let version_code = str_of(&data, "version_code").trim().to_string();
    let update_content = str_of(&data, "update_content").trim().to_string();
    if id <= 0 || app_name.is_empty() || version_code.is_empty() {
        return err(400, "参数错误");
    }
    let upd = sqlx::query(
        "UPDATE app_versions SET app_name = ?, version_code = ?, update_content = ? WHERE id = ?",
    )
    .bind(&app_name)
    .bind(&version_code)
    .bind(&update_content)
    .bind(id)
    .execute(pool)
    .await;
    match upd {
        Ok(r) if r.rows_affected() > 0 => {
            log_operation(pool, ctx, "修改版本", &app_name, &format!("版本号:{}", version_code)).await;
            ok("修改成功", Value::Null)
        }
        Ok(_) => err(404, "版本不存在"),
        Err(e) => { tracing::error!("数据库错误: {e}"); err(500, "数据库错误") },
    }
}

pub async fn change_version_status(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    let status = str_of(&data, "status").trim().to_string();
    let valid = ["normal", "update", "force_update", "disabled", "crash", "group_update"];
    if id <= 0 || !valid.contains(&status.as_str()) {
        return err(400, "参数错误");
    }
    let labels = [
        ("normal", "正常"), ("update", "更新"), ("force_update", "强制更新"),
        ("disabled", "禁用"), ("crash", "闪退"), ("group_update", "进群更新"),
    ];
    let label = labels.iter().find(|(c, _)| *c == status.as_str()).map(|(_, n)| *n).unwrap_or("未知");
    let upd = sqlx::query("UPDATE app_versions SET status = ? WHERE id = ?")
        .bind(&status).bind(id).execute(pool).await;
    match upd {
        Ok(_) => {
            log_operation(pool, ctx, "修改版本状态", &format!("ID:{}", id), &format!("状态改为:{}", label)).await;
            ok("操作成功", Value::Null)
        }
        Err(_) => err(500, "数据库错误"),
    }
}

pub async fn delete_version(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let id = int_of(&data, "id");
    if id <= 0 {
        return err(400, "参数错误");
    }
    let row = sqlx::query("SELECT download_url FROM app_versions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(row) = row else {
        return err(404, "版本不存在");
    };
    let url = row.try_get::<String, _>("download_url").unwrap_or_default();
    let filename: String = url.rsplit('/').next().unwrap_or("").to_string();
    let filepath = std::path::Path::new("uploads").join("apk").join(&filename);
    if filepath.exists() {
        let _ = std::fs::remove_file(&filepath);
    }
    let _ = sqlx::query("DELETE FROM app_versions WHERE id = ?").bind(id).execute(pool).await;
    log_operation(pool, ctx, "删除版本", &format!("ID:{}", id), &filename).await;
    ok("删除成功", Value::Null)
}

fn read_desktop_versions() -> Vec<Value> {
    let path = desktop_version_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Value>(&content) {
            if let Some(arr) = v.as_array() {
                return arr.clone();
            }
        }
    }
    Vec::new()
}

fn normalize_platform(raw: &str) -> String {
    match raw {
        "mobile" => "mobile".to_string(),
        "watch" => "watch".to_string(),
        _ => "desktop".to_string(),
    }
}

fn item_platform(item: &Value) -> String {
    normalize_platform(item.get("platform").and_then(|v| v.as_str()).unwrap_or("desktop").trim())
}

fn write_desktop_versions(list: &[Value]) -> bool {
    let path = desktop_version_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let json_str = serde_json::to_string_pretty(list).unwrap_or_else(|_| "[]".to_string());
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, json_str).is_ok() && std::fs::rename(&tmp, &path).is_ok()
}

pub async fn get_desktop_version(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let list = read_desktop_versions();
    log_operation(pool, ctx, "读取桌面端更新配置", "", "").await;
    ok("", json!({ "list": list }))
}

fn normalize_channel(raw: &str) -> String {
    match raw {
        "beta" => "beta".to_string(),
        _ => "stable".to_string(),
    }
}

fn default_system(platform: &str) -> String {
    match platform {
        "mobile" => "android".to_string(),
        "watch" => "".to_string(),
        _ => "windows".to_string(),
    }
}

fn normalize_system(platform: &str, raw: &str) -> String {
    let allowed: &[&str] = match platform {
        "mobile" => &["android", "harmonyos", "ios"],
        _ => &["windows", "linux", "macos"],
    };
    let r = raw.trim();
    if allowed.contains(&r) {
        r.to_string()
    } else {
        default_system(platform)
    }
}

fn item_system(item: &Value) -> String {
    let raw = item.get("system").and_then(|v| v.as_str()).unwrap_or("").trim();
    let platform = item_platform(item);
    if raw.is_empty() {
        default_system(&platform)
    } else {
        normalize_system(&platform, raw)
    }
}

fn system_label(platform: &str, system: &str) -> &'static str {
    match (platform, system) {
        ("mobile", "harmonyos") => "鸿蒙 HarmonyOS",
        ("mobile", "ios") => "iOS",
        ("mobile", _) => "Android",
        (_, "linux") => "Linux",
        (_, "macos") => "macOS",
        _ => "Windows",
    }
}

pub async fn save_desktop_version(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let version = str_of(&data, "version").trim().to_string();
    let platform = normalize_platform(str_of(&data, "platform").trim());
    let system = normalize_system(&platform, str_of(&data, "system").trim());
    let channel = normalize_channel(str_of(&data, "channel").trim());
    let mut download_url = str_of(&data, "download_url").trim().to_string();
    let update_content = str_of(&data, "update_content").trim().to_string();
    let enabled = int_of(&data, "enabled") != 0;
    let file_data = str_of(&data, "file_data").trim().to_string();
    let file_name = str_of(&data, "file_name").trim().to_string();
    let store_url = str_of(&data, "store_url").trim().to_string();
    if !store_url.is_empty() && !store_url.starts_with("https://") {
        return err(400, "商店页链接必须以 https:// 开头");
    }
    if version.is_empty() {
        return err(400, "版本号不能为空");
    }
    let mut list = read_desktop_versions();
    if !file_data.is_empty() {
        let file_bytes = match base64::engine::general_purpose::STANDARD.decode(&file_data) {
            Ok(b) => b,
            Err(_) => return err(400, "安装包数据解码失败"),
        };
        if file_bytes.is_empty() {
            return err(400, "安装包文件为空");
        }
        let upload_dir = std::path::Path::new("uploads").join("packages");
        if let Err(e) = std::fs::create_dir_all(&upload_dir) {
            { tracing::error!("无法创建安装包目录: {e}"); return err(500, "无法创建安装包目录"); }
        }
        let ext = safe_file_ext(&file_name);
        let ts = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
        let existing_pkg_name = list
            .iter()
            .find(|item| {
                item_platform(item) == platform
                    && item_system(item) == system
                    && item_channel(item) == channel
                    && item.get("version").and_then(|v| v.as_str()) == Some(version.as_str())
            })
            .and_then(|item| item.get("downloadUrl").and_then(|v| v.as_str()))
            .and_then(|url| url.strip_prefix("/uploads/packages/"))
            .filter(|name| !name.is_empty() && !name.contains('/') && !name.contains('\\'))
            .map(|name| name.to_string());
        let stem = safe_file_stem(&file_name);
        let new_filename = if let Some(name) = existing_pkg_name {
            name
        } else if !stem.is_empty() {
            let candidate = format!("{}.{}", stem, ext);
            if upload_dir.join(&candidate).exists() {
                format!("{}_{}.{}", stem, ts, ext)
            } else {
                candidate
            }
        } else {
            format!("{}_v{}_{}.{}", platform, safe_version_part(&version), ts, ext)
        };
        let target_path = upload_dir.join(&new_filename);
        if std::fs::write(&target_path, &file_bytes).is_err() {
            return err(500, "安装包保存失败，请检查目录权限");
        }
        download_url = format!("/uploads/packages/{}", new_filename);
    }
    if enabled && download_url.is_empty() {
        return err(400, "启用更新时，请填写下载链接或上传安装包");
    }
    let lower_url = download_url.to_lowercase();
    if lower_url.contains("apps.microsoft.com") || lower_url.contains("ms-windows-store") {
        return err(400, "下载渠道不能填微软商店页链接：请填安装包直链，商店页链接请配置到「商店分发」");
    }
    let is_new = !list.iter().any(|item| {
        item_platform(item) == platform
            && item_system(item) == system
            && item_channel(item) == channel
            && item.get("version").and_then(|v| v.as_str()) == Some(version.as_str())
    });
    if is_new {
        let mut max_ver: Option<&str> = None;
        for item in &list {
            if item_platform(item) != platform || item_system(item) != system || item_channel(item) != channel {
                continue;
            }
            let ver = item.get("version").and_then(|v| v.as_str()).unwrap_or("");
            if ver.is_empty() {
                continue;
            }
            max_ver = match max_ver {
                Some(m) if compare_version_code(ver, m) > 0 => Some(ver),
                Some(_) => max_ver,
                None => Some(ver),
            };
        }
        if let Some(mv) = max_ver {
            if compare_version_code(&version, mv) <= 0 {
                let channel_label = if channel == "beta" { "测试版" } else { "正式版" };
                let sys_label = system_label(&platform, &system);
                return err(400, &format!("新版本 {} 必须大于{}{}已有的最高版本 {}", version, sys_label, channel_label, mv));
            }
        }
    }
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let new_item = json!({
        "platform": platform.clone(),
        "system": system.clone(),
        "channel": channel.clone(),
        "version": version.clone(),
        "downloadUrl": download_url,
        "updateContent": update_content,
        "enabled": enabled,
        "storeUrl": store_url,
        "updated_at": now,
    });
    let mut replaced = false;
    for item in list.iter_mut() {
        if item_platform(item) == platform
            && item_system(item) == system
            && item_channel(item) == channel
            && item.get("version").and_then(|v| v.as_str()) == Some(version.as_str())
        {
            *item = new_item.clone();
            replaced = true;
            break;
        }
    }
    if !replaced {
        list.push(new_item);
    }
    if !write_desktop_versions(&list) {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    let platform_label = platform_label(&platform);
    let sys_label = system_label(&platform, &system);
    let channel_label = if channel == "beta" { "测试版" } else { "正式版" };
    let action_label = if replaced {
        format!("修改{}{}{}更新配置", sys_label, platform_label, channel_label)
    } else {
        format!("新增{}{}{}更新配置", sys_label, platform_label, channel_label)
    };
    log_operation(pool, ctx, &action_label, &version, if enabled { "启用" } else { "禁用" }).await;
    ok("保存成功", json!({ "version": version, "platform": platform, "system": system, "channel": channel }))
}

fn item_channel(item: &Value) -> String {
    normalize_channel(item.get("channel").and_then(|v| v.as_str()).unwrap_or("stable").trim())
}

fn platform_label(platform: &str) -> &'static str {
    match platform {
        "mobile" => "移动端",
        "watch" => "腕上端",
        _ => "桌面端",
    }
}

pub async fn delete_desktop_version(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let version = str_of(&data, "version").trim().to_string();
    let platform = normalize_platform(str_of(&data, "platform").trim());
    let raw_system = str_of(&data, "system").trim().to_string();
    let system = if raw_system.is_empty() {
        default_system(&platform)
    } else {
        normalize_system(&platform, &raw_system)
    };
    if version.is_empty() {
        return err(400, "版本号不能为空");
    }
    let mut list = read_desktop_versions();
    let before = list.len();
    list.retain(|item| {
        !(item_platform(item) == platform
            && item_system(item) == system
            && item.get("version").and_then(|v| v.as_str()) == Some(version.as_str()))
    });
    if list.len() == before {
        return err(404, "版本不存在");
    }
    if !write_desktop_versions(&list) {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    log_operation(pool, ctx, &format!("删除{}{}更新配置", system_label(&platform, &system), platform_label(&platform)), &version, "").await;
    ok("删除成功", Value::Null)
}

pub async fn list_beta_testers(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    match sqlx::query("SELECT id, device_id, note, created_at FROM beta_testers ORDER BY id DESC")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_value).collect();
            ok("ok", json!({ "list": list }))
        }
        Err(e) => { tracing::error!("查询失败: {e}"); err(500, "查询失败") },
    }
}

pub async fn add_beta_tester(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    let note = str_of(&data, "note").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }
    if device_id.chars().count() > 128 {
        return err(400, "设备ID过长（最多128字符）");
    }
    let dup: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM beta_testers WHERE device_id = ?")
        .bind(&device_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    if dup > 0 {
        return err(400, "该设备ID已在内测名单中");
    }
    match sqlx::query("INSERT INTO beta_testers (device_id, note) VALUES (?, ?)")
        .bind(&device_id)
        .bind(&note)
        .execute(pool)
        .await
    {
        Ok(_) => {
            log_operation(pool, ctx, "添加内测设备", &device_id, &note).await;
            ok("添加成功", Value::Null)
        }
        Err(e) => { tracing::error!("数据库错误: {e}"); err(500, "数据库错误") },
    }
}

pub async fn delete_beta_tester(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }
    match sqlx::query("DELETE FROM beta_testers WHERE device_id = ?")
        .bind(&device_id)
        .execute(pool)
        .await
    {
        Ok(r) if r.rows_affected() > 0 => {
            log_operation(pool, ctx, "移除内测设备", &device_id, "").await;
            ok("删除成功", Value::Null)
        }
        Ok(_) => err(404, "设备不在内测名单中"),
        Err(_) => err(500, "数据库错误"),
    }
}

pub async fn update_beta_tester_note(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    let note = str_of(&data, "note").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }
    if note.chars().count() > 255 {
        return err(400, "备注不能超过 255 字");
    }
    match sqlx::query("UPDATE beta_testers SET note = ? WHERE device_id = ?")
        .bind(&note)
        .bind(&device_id)
        .execute(pool)
        .await
    {
        Ok(r) if r.rows_affected() > 0 => {
            log_operation(pool, ctx, "设置内测设备备注", &device_id, &note).await;
            ok("已更新设备备注", json!({ "device_id": device_id, "note": note }))
        }
        Ok(_) => err(404, "设备不在内测名单中"),
        Err(_) => err(500, "数据库错误"),
    }
}

pub async fn get_beta_tester_detail(body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let device_id = str_of(&data, "device_id").trim().to_string();
    if device_id.is_empty() {
        return err(400, "设备ID不能为空");
    }

    let tester = sqlx::query("SELECT id, device_id, note, created_at FROM beta_testers WHERE device_id = ? LIMIT 1")
        .bind(&device_id)
        .fetch_optional(pool)
        .await;
    let tester_row = match tester {
        Ok(Some(r)) => r,
        Ok(None) => return err(404, "设备不在内测名单中"),
        Err(_) => return err(500, "数据库错误"),
    };

    let fb = sqlx::query(
        "SELECT f.ciyuanxi_id, COALESCE(u.nickname, f.nickname) AS nickname, \
                f.device_brand, f.device_model, f.os_version, f.architecture, f.machine_name, f.app_version \
         FROM user_feedback f LEFT JOIN app_users u ON u.ciyuanxi_id = f.ciyuanxi_id \
         WHERE f.device_id = ? ORDER BY f.id DESC LIMIT 1",
    )
    .bind(&device_id)
    .fetch_optional(pool)
    .await;

    let open = sqlx::query(
        "SELECT os_version, device_model, device_name, app_version FROM app_open_log WHERE device_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(&device_id)
    .fetch_optional(pool)
    .await;

    let str_col = |r: &sqlx::mysql::MySqlRow, col: &str| -> String {
        r.try_get::<Option<String>, _>(col).ok().flatten().unwrap_or_default()
    };

    let device = match &fb {
        Ok(Some(r)) => json!({
            "brand": str_col(r, "device_brand"),
            "model": str_col(r, "device_model"),
            "os_version": str_col(r, "os_version"),
            "architecture": str_col(r, "architecture"),
            "machine_name": str_col(r, "machine_name"),
            "app_version": str_col(r, "app_version"),
        }),
        _ => json!({}),
    };
    let mut device = device;
    if let Ok(Some(r)) = &open {
        for (key, col) in [("os_version", "os_version"), ("model", "device_model"), ("app_version", "app_version")] {
            if device.get(key).and_then(|v| v.as_str()).unwrap_or("").is_empty() {
                let v = str_col(r, col);
                if !v.is_empty() {
                    device[key] = Value::String(v);
                }
            }
        }
    }

    let mut accounts: Vec<Value> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    if let Ok(rows) = sqlx::query(
        "SELECT o.ciyuanxi_id, COALESCE(u.nickname, '') AS nickname \
         FROM app_open_log o LEFT JOIN app_users u ON u.ciyuanxi_id = o.ciyuanxi_id \
         WHERE o.device_id = ? AND o.ciyuanxi_id != '' ORDER BY o.id DESC LIMIT 100",
    )
    .bind(&device_id)
    .fetch_all(pool)
    .await
    {
        for r in rows {
            let cid = str_col(&r, "ciyuanxi_id");
            if cid.is_empty() || seen.contains(&cid) {
                continue;
            }
            seen.push(cid.clone());
            accounts.push(json!({
                "ciyuanxi_id": cid,
                "nickname": str_col(&r, "nickname"),
                "source": "启动记录",
            }));
            if accounts.len() >= 10 {
                break;
            }
        }
    }
    if let Ok(Some(r)) = &fb {
        let cid = str_col(r, "ciyuanxi_id");
        if !cid.is_empty() && !seen.contains(&cid) {
            accounts.insert(0, json!({
                "ciyuanxi_id": cid,
                "nickname": str_col(r, "nickname"),
                "source": "反馈提交",
            }));
        }
    }

    ok("ok", json!({
        "tester": row_to_value(&tester_row),
        "device": device,
        "accounts": accounts,
    }))
}
