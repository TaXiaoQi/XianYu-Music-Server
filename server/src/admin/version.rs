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

/// 保存桌面端版本配置。按版本号 upsert：版本号已存在则替换该条，否则新增一条，其余版本保持不变。
pub async fn save_desktop_version(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let version = str_of(&data, "version").trim().to_string();
    let mut download_url = str_of(&data, "download_url").trim().to_string();
    let update_content = str_of(&data, "update_content").trim().to_string();
    let enabled = int_of(&data, "enabled") != 0;
    let file_data = str_of(&data, "file_data").trim().to_string();
    let file_name = str_of(&data, "file_name").trim().to_string();
    if version.is_empty() {
        return err(400, "版本号不能为空");
    }
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
        let new_filename = format!("desktop_v{}_{}.{}", safe_version_part(&version), ts, ext);
        let target_path = upload_dir.join(&new_filename);
        if std::fs::write(&target_path, &file_bytes).is_err() {
            return err(500, "安装包保存失败，请检查目录权限");
        }
        download_url = format!("/uploads/packages/{}", new_filename);
    }
    if enabled && download_url.is_empty() {
        return err(400, "启用更新时，请填写下载链接或上传安装包");
    }
    let mut list = read_desktop_versions();
    // 新增版本号必须大于当前已列出的最高版本，防止版本号回退
    let is_new = !list.iter().any(|item| item.get("version").and_then(|v| v.as_str()) == Some(version.as_str()));
    if is_new {
        let mut max_ver: Option<&str> = None;
        for item in &list {
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
                return err(400, &format!("新版本 {} 必须大于已有的最高版本 {}", version, mv));
            }
        }
    }
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let new_item = json!({
        "version": version.clone(),
        "downloadUrl": download_url,
        "updateContent": update_content,
        "enabled": enabled,
        "updated_at": now,
    });
    let mut replaced = false;
    for item in list.iter_mut() {
        if item.get("version").and_then(|v| v.as_str()) == Some(version.as_str()) {
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
    log_operation(
        pool, ctx,
        if replaced { "修改桌面端更新配置" } else { "新增桌面端更新配置" },
        &version,
        if enabled { "启用" } else { "禁用" },
    ).await;
    ok("保存成功", json!({ "version": version }))
}

/// 删除桌面端版本配置
pub async fn delete_desktop_version(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let version = str_of(&data, "version").trim().to_string();
    if version.is_empty() {
        return err(400, "版本号不能为空");
    }
    let mut list = read_desktop_versions();
    let before = list.len();
    list.retain(|item| item.get("version").and_then(|v| v.as_str()) != Some(version.as_str()));
    if list.len() == before {
        return err(404, "版本不存在");
    }
    if !write_desktop_versions(&list) {
        return err(500, "写入文件失败，请检查 api 目录权限");
    }
    log_operation(pool, ctx, "删除桌面端更新配置", &version, "").await;
    ok("删除成功", Value::Null)
}
