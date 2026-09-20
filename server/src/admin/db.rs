use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;
use std::collections::HashMap;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

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

fn extract_table_name(stmt: &str) -> String {
    let first_line = stmt.lines().next().unwrap_or("");
    let after_prefix = first_line
        .trim()
        .trim_start_matches("CREATE TABLE IF NOT EXISTS")
        .trim();
    if let Some(start) = after_prefix.find('`') {
        if let Some(end) = after_prefix[start + 1..].find('`') {
            return after_prefix[start + 1..start + 1 + end].to_string();
        }
    }
    after_prefix
        .trim_matches('`')
        .split(|c: char| c.is_whitespace() || c == '(')
        .next()
        .unwrap_or("")
        .to_string()
}

pub async fn list_tables(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let schema_tables = crate::schema::table_statements();
    let mut result: Vec<Value> = Vec::new();
    let db_name = &ctx.config.db_name;
    for stmt in schema_tables {
        let name = extract_table_name(stmt);
        if name.is_empty() || name.starts_with("INSERT") || name.starts_with("VALUES") {
            continue;
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
        Err(e) => { tracing::error!("备份失败: {e}"); err(500, "备份失败") },
    }
}

pub struct BackupOutcome {
    pub filename: String,
    pub filepath: std::path::PathBuf,
    pub size: String,
    pub table_count: usize,
    pub skipped: bool,
}

fn write_sql(file: &mut std::fs::File, s: &str) -> Result<(), String> {
    use std::io::Write;
    file.write_all(s.as_bytes())
        .map_err(|e| format!("写入备份文件失败: {}", e))
}

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

    let mut file = std::fs::File::create(&filepath).map_err(|e| format!("创建备份文件失败: {}", e))?;
    write_sql(&mut file, &format!(
        "-- XiaYu Database Backup\n-- Generated: {}\n-- Mode: {}\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        mode
    ))?;
    write_sql(&mut file, "SET NAMES utf8mb4;\nSET FOREIGN_KEY_CHECKS=0;\n\n")?;

    const BATCH: usize = 1000;
    let mut table_count = 0;
    for table in &tables {
        if !backup_tables.contains(table) {
            continue;
        }
        table_count += 1;
        write_sql(&mut file, &format!("DROP TABLE IF EXISTS `{}`;\n", table))?;
        if let Ok(create) = sqlx::query_scalar::<_, String>(&format!("SHOW CREATE TABLE `{}`", table))
            .fetch_one(pool)
            .await
        {
            write_sql(&mut file, &create)?;
            write_sql(&mut file, ";\n")?;
        }
        let mut offset: usize = 0;
        loop {
            let rows = sqlx::query(&format!(
                "SELECT * FROM `{}` ORDER BY 1 LIMIT {} OFFSET {}",
                table, BATCH, offset
            ))
            .fetch_all(pool)
            .await
            .map_err(|e| format!("读取表 `{}` 失败: {}", table, e))?;
            let got = rows.len();
            for row in &rows {
                let obj = crate::admin::row_to_value(row);
                if let Value::Object(map) = &obj {
                    let cols: Vec<String> = map.keys().map(|k| format!("`{}`", k)).collect();
                    let vals: Vec<String> = map.values().map(sql_to_literal).collect();
                    write_sql(
                        &mut file,
                        &format!(
                            "INSERT INTO `{}` ({}) VALUES ({});\n",
                            table,
                            cols.join(","),
                            vals.join(",")
                        ),
                    )?;
                }
            }
            if got < BATCH {
                break;
            }
            offset += BATCH;
        }
        write_sql(&mut file, "\n")?;
    }
    write_sql(&mut file, "SET FOREIGN_KEY_CHECKS=1;\n")?;
    drop(file);

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

pub async fn get_auto_backup_config(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let enabled = read_setting(pool, KEY_ENABLED).await == "1";
    let interval = read_setting(pool, KEY_INTERVAL).await;
    let interval: i64 = interval.parse().unwrap_or(24 * 60);
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
    names.sort();
    while names.len() > max_count.max(1) as usize {
        let oldest = names.remove(0);
        let _ = std::fs::remove_file(dir.join(&oldest));
        tracing::info!("auto_backup: 已清理过期备份 {}", oldest);
    }
}

pub async fn auto_backup_loop(pool: &MySqlPool) {
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
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

pub async fn restore_backup(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    if ctx.role != "super_admin" {
        return err(403, "仅超级管理员可执行恢复备份");
    }
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
    let mut skipped = 0usize;
    let _ = sqlx::query("SET FOREIGN_KEY_CHECKS=0").execute(pool).await;
    for stmt in content.split(';') {
        let s = stmt.trim();
        if s.is_empty() {
            continue;
        }
        if let Some(reason) = unsafe_sql_reason(s) {
            tracing::warn!("恢复备份跳过危险语句（{}）：{}", reason, &s.chars().take(100).collect::<String>());
            skipped += 1;
            continue;
        }
        let _ = sqlx::query(s).execute(pool).await;
    }
    let _ = sqlx::query("SET FOREIGN_KEY_CHECKS=1").execute(pool).await;
    log_operation(pool, ctx, "数据库恢复", &filename, &format!("跳过危险语句 {}", skipped)).await;
    if skipped > 0 {
        ok("恢复成功（部分危险语句已跳过）", json!({ "skipped": skipped }))
    } else {
        ok("恢复成功", Value::Null)
    }
}

pub async fn import_db(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    if ctx.role != "super_admin" {
        return err(403, "仅超级管理员可执行导入数据库");
    }
    let data = parse_body(body);
    let content = str_of(&data, "content");
    if content.trim().is_empty() {
        return err(400, "导入内容为空");
    }
    if content.len() > 256 * 1024 * 1024 {
        return err(400, "导入文件过大");
    }
    let dir = backup_dir();
    let _ = std::fs::create_dir_all(&dir);
    let now = chrono::Local::now();
    let filename = format!("import_{}.sql", now.format("%Y%m%d_%H%M%S"));
    let _ = std::fs::write(dir.join(&filename), &content);

    let mut ok_count = 0;
    let mut err_count = 0;
    let mut skipped = 0usize;
    let _ = sqlx::query("SET FOREIGN_KEY_CHECKS=0").execute(pool).await;
    for stmt in content.split(';') {
        let s = stmt.trim();
        if s.is_empty() {
            continue;
        }
        if let Some(reason) = unsafe_sql_reason(s) {
            tracing::warn!("导入数据库跳过危险语句（{}）：{}", reason, &s.chars().take(100).collect::<String>());
            skipped += 1;
            continue;
        }
        match sqlx::query(s).execute(pool).await {
            Ok(_) => ok_count += 1,
            Err(_) => err_count += 1,
        }
    }
    let _ = sqlx::query("SET FOREIGN_KEY_CHECKS=1").execute(pool).await;
    log_operation(pool, ctx, "数据库导入", &filename, &format!("成功 {} 失败 {} 跳过 {}", ok_count, err_count, skipped)).await;
    if err_count > 0 || skipped > 0 {
        ok("导入完成（部分语句失败或被跳过）", json!({ "filename": filename, "ok": ok_count, "errors": err_count, "skipped": skipped }))
    } else {
        ok("导入成功", json!({ "filename": filename, "ok": ok_count, "errors": 0, "skipped": 0 }))
    }
}

fn sql_statement_head(stmt: &str) -> &str {
    let mut s = stmt.trim_start();
    loop {
        if let Some(rest) = s.strip_prefix("/*") {
            let Some(end) = rest.find("*/") else { return "" };
            s = rest[end + 2..].trim_start();
        } else if s.starts_with("--") || s.starts_with('#') {
            let Some(nl) = s.find('\n') else { return "" };
            s = s[nl + 1..].trim_start();
        } else {
            return s;
        }
    }
}

fn unsafe_sql_reason(stmt: &str) -> Option<&'static str> {
    let head = sql_statement_head(stmt);
    if head.is_empty() {
        return None;
    }
    let first = head.split_whitespace().next().unwrap_or("").to_ascii_uppercase();
    const ALLOWED: &[&str] = &[
        "INSERT", "REPLACE", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "TRUNCATE",
        "LOCK", "UNLOCK", "SET", "USE", "ANALYZE", "OPTIMIZE", "RENAME",
    ];
    if !ALLOWED.contains(&first.as_str()) {
        return Some("非白名单语句");
    }
    let lower = head.to_ascii_lowercase();
    const DANGEROUS: &[&str] = &[
        "into outfile", "into dumpfile", "load_file(", "load data infile",
        "create user", "drop user", "grant ", "revoke ", "general_log",
        "performance_schema", "information_schema", "mysql.user",
    ];
    if DANGEROUS.iter().any(|p| lower.contains(p)) {
        return Some("含文件系统/权限系统操作");
    }
    None
}

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