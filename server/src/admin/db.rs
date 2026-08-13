use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};
use sqlx::MySqlPool;

use super::{err, log_operation, ok, AdminCtx};
use crate::handlers::helpers::{int_of, parse_body, str_of};

/// 备份文件目录（与 serve 根平级的 beifen，与原 PHP 保持一致）
fn backup_dir() -> std::path::PathBuf {
    std::path::Path::new("beifen").to_path_buf()
}

const MAX_FILENAME: usize = 64;

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
    let exists: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = ? AND table_name = ?",
    )
    .bind(db_name)
    .bind(&table_name)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    if exists.is_none() {
        tracing::warn!("view_table: table not found in information_schema: db_name={}, table_name={}", db_name, table_name);
        return err(400, "表不存在");
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
    let dir = backup_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return err(500, &format!("无法创建备份目录: {}", e));
    }
    let now = chrono::Local::now();
    let filename = format!("backup_{}.sql", now.format("%Y%m%d_%H%M%S"));
    let filepath = dir.join(&filename);

    let mut out = String::new();
    out.push_str(&format!(
        "-- XiaYu Database Backup\n-- Generated: {}\n",
        now.format("%Y-%m-%d %H:%M:%S")
    ));
    out.push_str("SET NAMES utf8mb4;\nSET FOREIGN_KEY_CHECKS=0;\n\n");

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
        .unwrap_or_default();

    let mut table_count = 0;
    for table in &tables {
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

    if let Err(e) = std::fs::write(&filepath, &out) {
        return err(500, &format!("备份失败: {}", e));
    }
    let size = std::fs::metadata(&filepath).map(|m| m.len()).unwrap_or(0);
    let size_str = if size >= 1024 * 1024 {
        format!("{:.2} MB", size as f64 / 1048576.0)
    } else {
        format!("{:.2} KB", size as f64 / 1024.0)
    };
    log_operation(pool, ctx, "数据库备份", &filename, &format!("大小 {} 表数 {}", size_str, table_count)).await;
    ok("备份成功", json!({
        "filename": filename,
        "filepath": filepath.display().to_string(),
        "size": size_str,
        "tables": table_count
    }))
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