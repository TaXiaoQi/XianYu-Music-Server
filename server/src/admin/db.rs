use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;
use std::collections::HashMap;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

/// 备份文件目录（与 serve 根平级的 beifen，与原 PHP 保持一致）
fn backup_dir() -> std::path::PathBuf {
    std::path::Path::new("beifen").to_path_buf()
}

const MAX_FILENAME: usize = 64;

// ===== 自动备份配置（存于 server_settings KV 表） =====
const KEY_ENABLED: &str = "auto_backup_enabled";
const KEY_INTERVAL: &str = "auto_backup_interval";
const KEY_MAX_COUNT: &str = "auto_backup_max_count";
const KEY_MODE: &str = "auto_backup_mode";
const KEY_LAST_RUN: &str = "auto_backup_last_run";
const KEY_SNAPSHOT: &str = "auto_backup_snapshot";

/// 读取 server_settings 单个 key（空字符串视为未设置）
pub async fn read_setting(pool: &MySqlPool, key: &str) -> String {
    sqlx::query("SELECT setting_value FROM server_settings WHERE setting_key = ? LIMIT 1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get::<Option<String>, _>(0).ok().flatten())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_default()
}

/// 写入或更新 server_settings 值
pub async fn upsert_setting(pool: &MySqlPool, key: &str, value: &str, desc: &str) {
    let _ = sqlx::query(
        "INSERT INTO server_settings (setting_key, setting_value, description) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE setting_value = VALUES(setting_value), description = VALUES(description)",
    )
    .bind(key)
    .bind(value)
    .bind(desc)
    .execute(pool)
    .await;
}

fn sanitize_filename(name: &str) -> bool {
    name.len() > 8
        && name.len() <= MAX_FILENAME
        && name.starts_with("backup_")
        && name.ends_with(".sql")
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

/// 从 CREATE TABLE 语句中提取表名（如 `CREATE TABLE IF NOT EXISTS \`listen_daily_stats\` ( ...` → `listen_daily_stats`）
fn extract_table_name(stmt: &str) -> String {
    let first_line = stmt.lines().next().unwrap_or("");
    // 移除 "CREATE TABLE IF NOT EXISTS" 前缀
    let after_prefix = first_line
        .trim()
        .trim_start_matches("CREATE TABLE IF NOT EXISTS")
        .trim();
    // 提取反引号中的表名
    if let Some(start) = after_prefix.find('`') {
        if let Some(end) = after_prefix[start + 1..].find('`') {
            return after_prefix[start + 1..start + 1 + end].to_string();
        }
    }
    // 回退：移除反引号并截断到第一个空格或括号
    after_prefix
        .trim_matches('`')
        .split(|c: char| c.is_whitespace() || c == '(')
        .next()
        .unwrap_or("")
        .to_string()
}

/// 数据库表状态检查
pub async fn list_tables(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let schema_tables = crate::schema::table_statements();
    let mut result: Vec<Value> = Vec::new();
    let db_name = &ctx.config.db_name;
    for stmt in schema_tables {
        let name = extract_table_name(stmt);
        if name.is_empty() || name.starts_with("INSERT") || name.starts_with("VALUES") {
            continue; // 跳过非 CREATE TABLE 语句（如 server_settings 的 INSERT）
        }
        let exists: bool = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = ? AND table_name = ?",
        )
        .bind(db_name)
        .bind(&name)
        .fetch_one(pool)
        .await
        .unwrap_or(0)
            > 0;
        let row_count: i64 = if exists {
            sqlx::query_scalar(&format!("SELECT COUNT(*) FROM `{}`", name))
                .fetch_one(pool)
                .await
                .unwrap_or(0)
        } else {
            0
        };
        result.push(json!({
            "name": name,
            "exists": exists,
            "row_count": row_count,
        }));
    }
    ok("ok", json!({ "tables": result }))
}

/// 备份文件列表
pub async fn list_backups(_body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let dir = backup_dir();
    let mut backups: Vec<Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("backup_") && name.ends_with(".sql") {
                let meta = entry.metadata().ok();
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let size_str = if size >= 1024 * 1024 {
                    format!("{:.2} MB", size as f64 / 1048576.0)
                } else {
                    format!("{:.2} KB", size as f64 / 1024.0)
                };
                let modified = meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| {
                        let dt = chrono::DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_default();
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    })
                    .unwrap_or_default();
                backups.push(json!({
                    "name": name,
                    "size": size_str,
                    "size_bytes": size,
                    "created_at": modified,
                }));
            }
        }
    }
    backups.sort_by(|a, b| {
        b.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(a.get("name").and_then(|v| v.as_str()).unwrap_or(""))
    });
    ok("ok", json!({ "backups": backups, "total": backups.len() }))
}

/// 数据库修复：执行全部建表语句
pub async fn repair_database(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let mut created: Vec<String> = Vec::new();
    let mut errors: Vec<Value> = Vec::new();
    for stmt in crate::schema::table_statements() {
        let name = extract_table_name(stmt);
        match sqlx::query(stmt).execute(pool).await {
            Ok(_) => created.push(name.clone()),
            Err(e) => errors.push(json!({ "table": name, "msg": e.to_string() })),
        }
    }
    // 同时执行 ensure_schema 以补充缺失列和默认数据
    crate::schema::ensure_schema(pool).await;
    log_operation(pool, ctx, "修复数据库", "", &format!("created={} errors={}", created.len(), errors.len())).await;
    ok("修复完成", json!({
        "created_tables": created,
        "errors": errors,
        "summary": {
            "created_tables_count": created.len(),
            "errors_count": errors.len(),
        }
    }))
}

/// 查看表内容（分页，每页 100 行）
pub async fn view_table(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let table_name = str_of(&data, "table_name").trim().to_string();
    let page = int_of(&data, "page");
    let page_num = if page < 1 { 1 } else { page };
    let page_size = 100;
    if table_name.is_empty()
        || !table_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        || !table_name.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false)
    {
        return err(400, "无效的表名");
    }
    let db_name = &ctx.config.db_name;
    tracing::info!("view_table: db_name={}, table_name={}", db_name, table_name);
    match sqlx::query_scalar::<_, String>(
        "SELECT CONVERT(table_name USING utf8mb4) FROM information_schema.tables WHERE table_schema = ? AND table_name = ?",
    )
    .bind(db_name)
    .bind(&table_name)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            tracing::warn!("view_table: table not found in information_schema: db_name={}, table_name={}", db_name, table_name);
            return err(400, "表不存在");
        }
        Err(e) => {
            tracing::error!("view_table: information_schema query failed: {}", e);
            return err(500, "查询数据库失败，请检查数据库连接");
        }
    }
    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM `{}`", table_name))
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    let cols: Vec<String> = sqlx::query_scalar::<_, String>(&format!("SHOW COLUMNS FROM `{}`", table_name))
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    let offset = (page_num - 1) * page_size;
    let rows = sqlx::query(&format!(
        "SELECT * FROM `{}` ORDER BY 1 LIMIT {} OFFSET {}",
        table_name, page_size, offset
    ))
    .fetch_all(pool)
    .await;
    let rows: Vec<Value> = rows
        .map(|rs| rs.iter().map(crate::admin::row_to_value).collect())
        .unwrap_or_default();
    log_operation(pool, ctx, "查看表", &table_name, "").await;
    ok("", json!({
        "table": table_name, "columns": cols, "rows": rows,
        "total": total, "page": page_num, "pageSize": page_size
    }))
}

/// 将一个单元格转为 SQL 插入字面量（数值不加引号，文本转义）
#[allow(dead_code)]
fn sql_literal(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
        other => {
            let s = other.as_str().unwrap_or("").to_string();
            format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
        }
    }
}

/// 备份整个数据库，写入 beifen/backup_YYYYmmdd_HHMMSS.sql
pub async fn backup_db(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    match perform_backup(pool, "full").await {
        Ok(outcome) => {
            log_operation(
                pool,
                ctx,
                "数据库备份",
                &outcome.filename,
                &format!("大小 {} 表数 {} 模式 全量", outcome.size, outcome.table_count),
            )
            .await;
            ok("备份成功", json!({
                "filename": outcome.filename,
                "filepath": outcome.filepath.display().to_string(),
                "size": outcome.size,
                "tables": outcome.table_count,
                "mode": "full"
            }))
        }
        Err(e) => err(500, &format!("备份失败: {}", e)),
    }
}

/// 备份结果
pub struct BackupOutcome {
    pub filename: String,
    pub filepath: std::path::PathBuf,
    pub size: String,
    pub table_count: usize,
    pub skipped: bool,
}

/// 备份核心逻辑（可被手动备份与自动备份复用）。
/// mode：`full` 全量备份所有表；`incremental` 增量备份——基于上次备份时的行数快照，
/// 仅导出行数发生变化或有新增的表，并更新行数快照。若增量模式下无任何表变化，
/// 则不生成文件，返回 skipped=true。
pub async fn perform_backup(pool: &MySqlPool, mode: &str) -> Result<BackupOutcome, String> {
    let dir = backup_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建备份目录: {}", e))?;
    let now = chrono::Local::now();
    let filename = format!("backup_{}.sql", now.format("%Y%m%d_%H%M%S"));
    let filepath = dir.join(&filename);

    let tables: Vec<String> = sqlx::query("SHOW TABLES")
        .fetch_all(pool)
        .await
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    crate::admin::row_to_value(row)
                        .as_object()
                        .map(|m| m.values().next().and_then(|v| v.as_str().map(|s| s.to_string())))
                        .flatten()
                })
                .collect()
        })
        .map_err(|e| format!("读取表列表失败: {}", e))?;

    // 增量模式：读取上次行数快照，决定本次需要备份的表
    let mut backup_tables: std::collections::HashSet<String> = tables.clone().into_iter().collect();
    if mode == "incremental" {
        let snapshot = read_setting(pool, KEY_SNAPSHOT).await;
        let prev: HashMap<String, u64> = if snapshot.is_empty() {
            HashMap::new()
        } else {
            serde_json::from_str(&snapshot).unwrap_or_default()
        };
        let mut current: HashMap<String, u64> = HashMap::new();
        for table in &tables {
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM `{}`", table))
                .fetch_one(pool)
                .await
                .unwrap_or(0);
            current.insert(table.clone(), count.max(0) as u64);
        }
        // 仅保留行数变化的表（首轮快照为空时全部视为变化）
        backup_tables = tables
            .iter()
            .filter(|t| current.get(*t) != prev.get(*t))
            .cloned()
            .collect();
        if let Ok(json) = serde_json::to_string(&current) {
            upsert_setting(pool, KEY_SNAPSHOT, &json, "自动备份-各表行数快照（增量备份用）").await;
        }
        if backup_tables.is_empty() {
            return Ok(BackupOutcome {
                filename,
                filepath,
                size: "0 B".to_string(),
                table_count: 0,
                skipped: true,
            });
        }
    }

    let mut out = String::new();
    out.push_str(&format!(
        "-- XiaYu Database Backup\n-- Generated: {}\n-- Mode: {}\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        mode
    ));
    out.push_str("SET NAMES utf8mb4;\nSET FOREIGN_KEY_CHECKS=0;\n\n");

    let mut table_count = 0;
    for table in &tables {
        if !backup_tables.contains(table) {
            continue;
        }
        table_count += 1;
        out.push_str(&format!("DROP TABLE IF EXISTS `{}`;\n", table));
        if let Ok(create) = sqlx::query_scalar::<_, String>(&format!("SHOW CREATE TABLE `{}`", table))
            .fetch_one(pool)
            .await
        {
            out.push_str(&create);
            out.push_str(";\n");
        }
        if let Ok(rows) = sqlx::query(&format!("SELECT * FROM `{}`", table)).fetch_all(pool).await {
            for row in &rows {
                let obj = crate::admin::row_to_value(row);
                if let Value::Object(map) = &obj {
                    let cols: Vec<String> = map.keys().map(|k| format!("`{}`", k)).collect();
                    let vals: Vec<String> = map.values().map(sql_to_literal).collect();
                    out.push_str(&format!(
                        "INSERT INTO `{}` ({}) VALUES ({});\n",
                        table,
                        cols.join(","),
                        vals.join(",")
                    ));
                }
            }
        }
        out.push('\n');
    }
    out.push_str("SET FOREIGN_KEY_CHECKS=1;\n");

    std::fs::write(&filepath, &out).map_err(|e| format!("写入备份文件失败: {}", e))?;
    let size = std::fs::metadata(&filepath).map(|m| m.len()).unwrap_or(0);
    let size_str = if size >= 1024 * 1024 {
        format!("{:.2} MB", size as f64 / 1048576.0)
    } else {
        format!("{:.2} KB", size as f64 / 1024.0)
    };
    Ok(BackupOutcome {
        filename,
        filepath,
        size: size_str,
        table_count,
        skipped: false,
    })
}

/// 自动备份配置
pub async fn get_auto_backup_config(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let enabled = read_setting(pool, KEY_ENABLED).await == "1";
    let interval = read_setting(pool, KEY_INTERVAL).await;
    let interval: i64 = interval.parse().unwrap_or(24 * 60); // 默认每天
    let max_count = read_setting(pool, KEY_MAX_COUNT).await;
    let max_count: i64 = max_count.parse().unwrap_or(20);
    let mode = read_setting(pool, KEY_MODE).await;
    let mode = if mode == "incremental" { "incremental" } else { "full" };
    let last_run = read_setting(pool, KEY_LAST_RUN).await;
    ok("ok", json!({
        "enabled": enabled,
        "interval_minutes": interval,
        "max_count": max_count,
        "mode": mode,
        "last_run": last_run,
    }))
}

/// 保存自动备份配置
pub async fn save_auto_backup_config(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let enabled = int_of(&data, "enabled") == 1;
    let interval = int_of(&data, "interval_minutes");
    let max_count = int_of(&data, "max_count");
    let mode = str_of(&data, "mode");
    if interval < 1 || interval > 30 * 24 * 60 {
        return err(400, "备份间隔需在 1 ~ 43200 分钟之间");
    }
    if max_count < 1 || max_count > 1000 {
        return err(400, "备份最大次数需在 1 ~ 1000 之间");
    }
    let mode = if mode == "incremental" { "incremental" } else { "full" };
    upsert_setting(pool, KEY_ENABLED, if enabled { "1" } else { "0" }, "是否启用自动备份").await;
    upsert_setting(pool, KEY_INTERVAL, &interval.to_string(), "自动备份间隔（分钟）").await;
    upsert_setting(pool, KEY_MAX_COUNT, &max_count.to_string(), "自动备份最大保留次数").await;
    upsert_setting(pool, KEY_MODE, mode, "备份模式：full=全量，incremental=增量").await;
    log_operation(pool, ctx, "自动备份设置", "", &format!("enabled={} interval={} max={} mode={}", enabled, interval, max_count, mode)).await;
    ok("保存成功", json!({
        "enabled": enabled,
        "interval_minutes": interval,
        "max_count": max_count,
        "mode": mode,
    }))
}

/// 整理备份文件：仅保留最近 max_count 个 backup_*.sql（删除更旧的）
pub async fn enforce_backup_retention(max_count: i64) {
    let dir = backup_dir();
    let mut names: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if sanitize_filename(&name) {
                names.push(name);
            }
        }
    }
    names.sort(); // 文件名按时间戳字典序即时间先后
    while names.len() > max_count.max(1) as usize {
        let oldest = names.remove(0);
        let _ = std::fs::remove_file(dir.join(&oldest));
        tracing::info!("auto_backup: 已清理过期备份 {}", oldest);
    }
}

/// 后台自动备份循环：每 60s 检查一次配置，若启用且距上次备份超过间隔则执行备份。
/// 备份完成后整理保留数量，并记录上次执行时间。
pub async fn auto_backup_loop(pool: &MySqlPool) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
    // 首次立即执行一次，便于启动后很快校验
    ticker.tick().await;
    loop {
        ticker.tick().await;
        if read_setting(pool, KEY_ENABLED).await != "1" {
            continue;
        }
        let interval: i64 = read_setting(pool, KEY_INTERVAL).await.parse().unwrap_or(24 * 60);
        let max_count: i64 = read_setting(pool, KEY_MAX_COUNT).await.parse().unwrap_or(20);
        let mode = read_setting(pool, KEY_MODE).await;
        let mode = if mode == "incremental" { "incremental" } else { "full" };

        // 距上次执行是否已到间隔
        let last_run_raw = read_setting(pool, KEY_LAST_RUN).await;
        let due = if last_run_raw.is_empty() {
            true
        } else {
            match chrono::NaiveDateTime::parse_from_str(&last_run_raw, "%Y-%m-%d %H:%M:%S") {
                Ok(last) => (chrono::Local::now().naive_local() - last).num_minutes() >= interval,
                Err(_) => true,
            }
        };
        if !due {
            continue;
        }

        match perform_backup(pool, mode).await {
            Ok(outcome) => {
                let now_str = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                upsert_setting(pool, KEY_LAST_RUN, &now_str, "自动备份-上次执行时间").await;
                if outcome.skipped {
                    tracing::info!("auto_backup: 增量备份无表变化，跳过本次生成");
                } else {
                    tracing::info!(
                        "auto_backup: 已生成 {}（{}，{} 张表，模式 {}）",
                        outcome.filename,
                        outcome.size,
                        outcome.table_count,
                        mode
                    );
                }
                // 整理保留数量（仅统计实际生成的备份文件）
                enforce_backup_retention(max_count).await;
            }
            Err(e) => {
                tracing::error!("auto_backup: 备份失败: {}", e);
            }
        }
    }
}

fn sql_to_literal(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'")),
        _ => "''".to_string(),
    }
}

/// 查看备份文件内容
pub async fn view_backup(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let filename = str_of(&data, "filename").trim().to_string();
    if !sanitize_filename(&filename) {
        return err(400, "无效的文件名");
    }
    let filepath = backup_dir().join(&filename);
    match std::fs::read_to_string(&filepath) {
        Ok(content) => {
            log_operation(pool, ctx, "查看备份", &filename, "").await;
            ok("success", json!({ "content": content }))
        }
        Err(_) => err(404, "备份文件不存在"),
    }
}

/// 恢复备份
pub async fn restore_backup(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let filename = str_of(&data, "filename").trim().to_string();
    if !sanitize_filename(&filename) {
        return err(400, "无效的文件名");
    }
    let filepath = backup_dir().join(&filename);
    let content = match std::fs::read_to_string(&filepath) {
        Ok(c) => c,
        Err(_) => return err(404, "备份文件不存在"),
    };
    let _ = sqlx::query("SET FOREIGN_KEY_CHECKS=0").execute(pool).await;
    for stmt in content.split(';') {
        let s = stmt.trim();
        if !s.is_empty() {
            let _ = sqlx::query(s).execute(pool).await;
        }
    }
    let _ = sqlx::query("SET FOREIGN_KEY_CHECKS=1").execute(pool).await;
    log_operation(pool, ctx, "数据库恢复", &filename, "").await;
    ok("恢复成功", Value::Null)
}

/// 导入数据库：从前端上传的 SQL 文本执行导入（覆盖式，与恢复备份一致）
pub async fn import_db(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let content = str_of(&data, "content");
    if content.trim().is_empty() {
        return err(400, "导入内容为空");
    }
    if content.len() > 256 * 1024 * 1024 {
        return err(400, "导入文件过大");
    }
    // 记录导入文件到 beifen/import_*.sql（命名带 import_ 前缀，不进入备份列表）
    let dir = backup_dir();
    let _ = std::fs::create_dir_all(&dir);
    let now = chrono::Local::now();
    let filename = format!("import_{}.sql", now.format("%Y%m%d_%H%M%S"));
    let _ = std::fs::write(dir.join(&filename), &content);

    let mut ok_count = 0;
    let mut err_count = 0;
    let _ = sqlx::query("SET FOREIGN_KEY_CHECKS=0").execute(pool).await;
    for stmt in content.split(';') {
        let s = stmt.trim();
        if s.is_empty() {
            continue;
        }
        match sqlx::query(s).execute(pool).await {
            Ok(_) => ok_count += 1,
            Err(_) => err_count += 1,
        }
    }
    let _ = sqlx::query("SET FOREIGN_KEY_CHECKS=1").execute(pool).await;
    log_operation(pool, ctx, "数据库导入", &filename, &format!("成功 {} 失败 {}", ok_count, err_count)).await;
    if err_count > 0 {
        ok("导入完成（部分语句失败）", json!({ "filename": filename, "ok": ok_count, "errors": err_count }))
    } else {
        ok("导入成功", json!({ "filename": filename, "ok": ok_count, "errors": 0 }))
    }
}

/// 删除备份文件
pub async fn delete_backup(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let filename = str_of(&data, "filename").trim().to_string();
    if !sanitize_filename(&filename) {
        return err(400, "无效的文件名");
    }
    let filepath = backup_dir().join(&filename);
    if !filepath.exists() {
        return err(404, "备份文件不存在");
    }
    match std::fs::remove_file(&filepath) {
        Ok(_) => {
            log_operation(pool, ctx, "删除备份", &filename, "").await;
            ok("删除成功", Value::Null)
        }
        Err(_) => err(500, "删除失败"),
    }
}

/// 下载备份文件（返回 application/sql 文件流，带 Content-Disposition）
pub async fn download_backup(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let filename = str_of(&data, "filename").trim().to_string();
    if !sanitize_filename(&filename) {
        return err(400, "无效的文件名");
    }
    let filepath = backup_dir().join(&filename);
    let content = match std::fs::read(&filepath) {
        Ok(c) => c,
        Err(_) => return err(404, "备份文件不存在"),
    };
    log_operation(pool, ctx, "下载备份", &filename, "").await;
    let cd = format!("attachment; filename=\"{}\"", filename);
    (
        axum::http::StatusCode::OK,
        [
            ("content-type", "application/sql"),
            ("content-disposition", Box::leak(cd.into_boxed_str())),
        ],
        axum::body::Body::from(content),
    )
        .into_response()
}