use axum::response::Response;
use base64::Engine;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, row_to_value, AdminCtx};
use crate::handlers::helpers::{compare_version_code, int_of, parse_body, str_of};

/// 新增 APP 版本（接收 base64 编码的 APK 文件数据）
/// 入参（JSON）：
///   app_name: 软件名称
///   version_code: 版本号
///   update_content: 更新内容
///   file_data: base64 编码的 APK 文件内容
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

    // 解码 base64 文件数据
    let file_bytes = match base64::engine::general_purpose::STANDARD.decode(&file_data) {
        Ok(b) => b,
        Err(_) => return err(400, "文件数据解码失败"),
    };
    if file_bytes.is_empty() {
        return err(400, "文件为空");
    }

    let upload_dir = std::path::Path::new("uploads").join("apk");
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        return err(500, &format!("无法创建上传目录: {}", e));
    }

    // 先插入记录获取 ID，再保存文件
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
        Err(e) => return err(500, &format!("数据库错误: {}", e)),
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

/// 获取 APP 版本列表（分页）
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
        Err(e) => err(500, &format!("查询失败: {}", e)),
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

/// 提取上传文件名的安全主干（去路径、去扩展名），仅保留 ASCII 字母数字与 `. _ -`。
/// 全部被净化为空时返回空串，由调用方回退到生成的文件名。
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

/// 修改版本信息（不重新上传安装包）
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
        Err(e) => err(500, &format!("数据库错误: {}", e)),
    }
}

/// 修改版本状态
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

/// 删除版本（同时删除 APK 文件）
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

/// 平台白名单：desktop / mobile / watch（腕上端预留）。未携带或非法值回退 desktop。
fn normalize_platform(raw: &str) -> String {
    match raw {
        "mobile" => "mobile".to_string(),
        "watch" => "watch".to_string(),
        _ => "desktop".to_string(),
    }
}

/// 配置项的平台标签；旧数据（分平台上线前保存的）视为 desktop。
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

/// 获取桌面端版本配置（多版本列表）
pub async fn get_desktop_version(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let list = read_desktop_versions();
    log_operation(pool, ctx, "读取桌面端更新配置", "", "").await;
    ok("", json!({ "list": list }))
}

/// 渠道白名单：stable（正式版）/ beta（测试版，仅对内测名单设备下发）。非法值回退 stable。
fn normalize_channel(raw: &str) -> String {
    match raw {
        "beta" => "beta".to_string(),
        _ => "stable".to_string(),
    }
}

/// 保存版本更新配置。按「平台 + 渠道 + 版本号」upsert：同平台同渠道版本号已存在则替换该条，
/// 否则新增；各平台/渠道版本号独立比较，互不影响。
pub async fn save_desktop_version(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let version = str_of(&data, "version").trim().to_string();
    let platform = normalize_platform(str_of(&data, "platform").trim());
    let channel = normalize_channel(str_of(&data, "channel").trim());
    let mut download_url = str_of(&data, "download_url").trim().to_string();
    let update_content = str_of(&data, "update_content").trim().to_string();
    let enabled = int_of(&data, "enabled") != 0;
    let file_data = str_of(&data, "file_data").trim().to_string();
    let file_name = str_of(&data, "file_name").trim().to_string();
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
            return err(500, &format!("无法创建安装包目录: {}", e));
        }
        let ext = safe_file_ext(&file_name);
        let ts = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
        // 保留上传文件的原名（净化后），避免官网下载出现 platform_v版本_时间戳 这类不可读文件名。
        // 同版本重复保存沿用已存文件名覆盖写入，保持下载地址稳定不失效；
        // 仅当原名不可用或与其他版本的包重名时才回退到旧的时间戳命名。
        let existing_pkg_name = list
            .iter()
            .find(|item| {
                item_platform(item) == platform
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
                // 同名文件已被其他版本的安装包占用，追加时间戳保护旧包不被覆盖
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
    // 新增版本号必须大于该平台同渠道已列出的最高版本，防止版本号回退
    // （正式版与测试版互不干扰：1.2.0-beta-1 允许与 1.2.0 正式版并存）
    let is_new = !list.iter().any(|item| {
        item_platform(item) == platform
            && item_channel(item) == channel
            && item.get("version").and_then(|v| v.as_str()) == Some(version.as_str())
    });
    if is_new {
        let mut max_ver: Option<&str> = None;
        for item in &list {
            if item_platform(item) != platform || item_channel(item) != channel {
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
                return err(400, &format!("新版本 {} 必须大于该平台{}已有的最高版本 {}", version, channel_label, mv));
            }
        }
    }
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let new_item = json!({
        "platform": platform.clone(),
        "channel": channel.clone(),
        "version": version.clone(),
        "downloadUrl": download_url,
        "updateContent": update_content,
        "enabled": enabled,
        "updated_at": now,
    });
    let mut replaced = false;
    for item in list.iter_mut() {
        if item_platform(item) == platform
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
    let channel_label = if channel == "beta" { "测试版" } else { "正式版" };
    let action_label = if replaced {
        format!("修改{}{}更新配置", platform_label, channel_label)
    } else {
        format!("新增{}{}更新配置", platform_label, channel_label)
    };
    log_operation(pool, ctx, &action_label, &version, if enabled { "启用" } else { "禁用" }).await;
    ok("保存成功", json!({ "version": version, "platform": platform, "channel": channel }))
}

/// 配置项的渠道标签；旧数据（分渠道上线前保存的）视为正式版。
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

/// 删除版本更新配置（按平台 + 版本号匹配）
pub async fn delete_desktop_version(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let version = str_of(&data, "version").trim().to_string();
    let platform = normalize_platform(str_of(&data, "platform").trim());
    if version.is_empty() {
        return err(400, "版本号不能为空");
    }
    let mut list = read_desktop_versions();
    let before = list.len();
    list.retain(|item| {
        !(item_platform(item) == platform && item.get("version").and_then(|v| v.as_str()) == Some(version.as_str()))
    });
    if list.len() == before {
        return err(404, "版本不存在");
    }
    if !write_desktop_versions(&list) {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    log_operation(pool, ctx, &format!("删除{}更新配置", platform_label(&platform)), &version, "").await;
    ok("删除成功", Value::Null)
}

/// 获取内测名单（beta_testers 全量列表，按添加时间倒序）
pub async fn list_beta_testers(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    match sqlx::query("SELECT id, device_id, note, created_at FROM beta_testers ORDER BY id DESC")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(row_to_value).collect();
            ok("ok", json!({ "list": list }))
        }
        Err(e) => err(500, &format!("查询失败: {}", e)),
    }
}

/// 添加内测设备（device_id 唯一，可选备注）
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
        Err(e) => err(500, &format!("数据库错误: {}", e)),
    }
}

/// 从内测名单移除设备
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
