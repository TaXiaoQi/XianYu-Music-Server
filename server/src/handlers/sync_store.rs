use chrono::Utc;
use serde_json::Value;
use sqlx::MySqlPool;
use std::path::PathBuf;

/// 用户同步快照的存储层：DB 表 user_sync_files / user_sync_chunks，
/// 以及旧版文件存储（data/sync/<弦予号>/*.json）的懒迁移与清理。
/// sync.rs 通过 re-export 保持旧引用路径（crate::handlers::sync::...）可用。

pub fn sync_root() -> PathBuf {
    PathBuf::from("data/sync")
}

pub fn sync_dir(ciyuanxi_id: &str) -> PathBuf {
    let digits: String = ciyuanxi_id.chars().filter(|c| c.is_ascii_digit()).collect();
    sync_root().join(digits)
}
// 注意：同步数据的存储 key 必须使用完整弦予号（可含字母）。
// 历史版本曾把弦予号去掉字母只留数字当 key，导致数字部分相同的
// 不同账号（如 abc123 与 123abc）歌单/插件互相串号，已废弃该逻辑，
// 并在 backfill_legacy_sync_files 末尾对存量数据做唯一映射迁移。

pub fn now_str() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn now_ts() -> i64 {
    Utc::now().timestamp()
}

/// 整包读用户同步文件：先查 DB，未命中时尝试旧文件懒迁移（导入后删除旧文件）。
pub async fn read_snapshot(pool: &MySqlPool, ciyuanxi_id: &str, name: &str) -> Result<Value, ()> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT content FROM user_sync_files WHERE ciyuanxi_id = ? AND file_name = ?")
            .bind(ciyuanxi_id)
            .bind(name)
            .fetch_optional(pool)
            .await
            .map_err(|_| ())?;
    if let Some((content,)) = row {
        return serde_json::from_str(&content).map_err(|_| ());
    }
    // 旧文件回退：命中即导入 DB 并移除旧文件
    let file = sync_dir(ciyuanxi_id).join(name);
    if !file.exists() {
        return Err(());
    }
    let content = std::fs::read_to_string(&file).map_err(|_| ())?;
    let v: Value = serde_json::from_str(&content).map_err(|_| ())?;
    let _ = sqlx::query("INSERT IGNORE INTO user_sync_files (ciyuanxi_id, file_name, content, content_size) VALUES (?, ?, ?, ?)")
        .bind(ciyuanxi_id)
        .bind(name)
        .bind(&content)
        .bind(content.len() as i64)
        .execute(pool)
        .await;
    let _ = std::fs::remove_file(&file);
    Ok(v)
}

/// 整包写用户同步文件（DB upsert），成功后清掉可能残留的旧文件。
pub async fn write_snapshot(pool: &MySqlPool, ciyuanxi_id: &str, name: &str, data: &Value) -> bool {
    let content = serde_json::to_string(data).unwrap_or_default();
    let ok = sqlx::query(
        "INSERT INTO user_sync_files (ciyuanxi_id, file_name, content, content_size) VALUES (?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE content = VALUES(content), content_size = VALUES(content_size)",
    )
    .bind(ciyuanxi_id)
    .bind(name)
    .bind(&content)
    .bind(content.len() as i64)
    .execute(pool)
    .await
    .is_ok();
    if ok {
        let _ = std::fs::remove_file(sync_dir(ciyuanxi_id).join(name));
    }
    ok
}

/// 注销/清理用户时删除其全部同步数据（DB + 遗留目录兜底）。
pub async fn delete_user_sync_data(pool: &MySqlPool, ciyuanxi_id: &str) {
    let _ = sqlx::query("DELETE FROM user_sync_files WHERE ciyuanxi_id = ?")
        .bind(ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_sync_chunks WHERE ciyuanxi_id = ?")
        .bind(ciyuanxi_id)
        .execute(pool)
        .await;
    let _ = std::fs::remove_dir_all(sync_dir(ciyuanxi_id));
}

/// 启动回填：把 data/sync 下遗留的旧文件（含中断上传的分块）导入 DB，
/// 导入成功的文件即删除；幂等，可重复执行。
pub async fn backfill_legacy_sync_files(pool: &MySqlPool) {
    let root = sync_root();
    let mut migrated = 0usize;
    let entries = match std::fs::read_dir(&root) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let uid = e.file_name().to_string_lossy().to_string();
            let files = match std::fs::read_dir(&p) {
                Ok(f) => f,
                Err(_) => continue,
            };
            for f in files.flatten() {
                let fp = f.path();
                if fp.is_dir() {
                    // chunks 目录：遗留的分块文件导入 user_sync_chunks
                    if fp.file_name().map(|n| n == "chunks").unwrap_or(false) {
                        if let Ok(cs) = std::fs::read_dir(&fp) {
                            for c in cs.flatten() {
                                let cp = c.path();
                                let cname = c.file_name().to_string_lossy().to_string();
                                if let Some(idx) = parse_chunk_index(&cname) {
                                    if let Ok(content) = std::fs::read_to_string(&cp) {
                                        let ins = sqlx::query(
                                            "INSERT IGNORE INTO user_sync_chunks (ciyuanxi_id, chunk_index, total_chunks, content) VALUES (?, ?, 0, ?)",
                                        )
                                        .bind(&uid)
                                        .bind(idx)
                                        .bind(&content)
                                        .execute(pool)
                                        .await;
                                        if ins.map(|r| r.rows_affected() > 0).unwrap_or(false) {
                                            let _ = std::fs::remove_file(&cp);
                                            migrated += 1;
                                        }
                                    }
                                }
                            }
                            let _ = std::fs::remove_dir(&fp);
                        }
                    }
                } else {
                    let name = f.file_name().to_string_lossy().to_string();
                    if !name.ends_with(".json") {
                        continue;
                    }
                    if let Ok(content) = std::fs::read_to_string(&fp) {
                        let size = content.len() as i64;
                        let ins = sqlx::query(
                            "INSERT IGNORE INTO user_sync_files (ciyuanxi_id, file_name, content, content_size) VALUES (?, ?, ?, ?)",
                        )
                        .bind(&uid)
                        .bind(&name)
                        .bind(&content)
                        .bind(size)
                        .execute(pool)
                        .await;
                        if ins.map(|r| r.rows_affected() > 0).unwrap_or(false) {
                            let _ = std::fs::remove_file(&fp);
                            migrated += 1;
                        }
                    }
                }
            }
            let _ = std::fs::remove_dir(&p);
        } else {
            // 根级文件：属于无数字字符的异常弦予号（旧逻辑落到根目录）
            let name = e.file_name().to_string_lossy().to_string();
            if !name.ends_with(".json") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&p) {
                let size = content.len() as i64;
                let ins = sqlx::query(
                    "INSERT IGNORE INTO user_sync_files (ciyuanxi_id, file_name, content, content_size) VALUES ('', ?, ?, ?)",
                )
                .bind(&name)
                .bind(&content)
                .bind(size)
                .execute(pool)
                .await;
                if ins.map(|r| r.rows_affected() > 0).unwrap_or(false) {
                    let _ = std::fs::remove_file(&p);
                    migrated += 1;
                }
            }
        }
    }
    // 清理超过 7 天的遗留分块（客户端中断上传的残渣）
    let _ = sqlx::query("DELETE FROM user_sync_chunks WHERE updated_at < NOW() - INTERVAL 7 DAY")
        .execute(pool)
        .await;
    if migrated > 0 {
        tracing::info!("[sync] backfilled {} legacy sync items into db", migrated);
    }
    migrate_digit_sync_keys(pool).await;
}

/// 历史版本曾用“弦予号去掉字母只留数字”作为同步 key，
/// 数字部分相同的不同账号（如 abc123 与 123abc）会共用同一行导致串号。
/// 启动时把能唯一映射回某个用户的数字 key 行迁移为完整弦予号；
/// 有歧义（多个用户映射到同一数字串）的保持原样，待该用户下次同步覆盖。
async fn migrate_digit_sync_keys(pool: &MySqlPool) {
    let users: Vec<(String,)> =
        sqlx::query_as("SELECT ciyuanxi_id FROM app_users WHERE ciyuanxi_id <> ''")
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    let mut groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (id,) in users {
        let id = id.trim().to_string();
        let digits: String = id.chars().filter(|c| c.is_ascii_digit()).collect();
        // 纯数字弦予号的 key 本就正确，无需迁移
        if digits.is_empty() || digits == id {
            continue;
        }
        groups.entry(digits).or_default().push(id);
    }
    let mut migrated = 0usize;
    for (digits, ids) in groups {
        if ids.len() != 1 {
            continue;
        }
        let uid = &ids[0];
        for table in ["user_sync_files", "user_sync_chunks"] {
            // 目标完整弦予号下已有数据则跳过，避免覆盖与撞唯一键
            let exists: i64 = sqlx::query_scalar(&format!(
                "SELECT COUNT(*) FROM {table} WHERE ciyuanxi_id = ?"
            ))
            .bind(uid)
            .fetch_one(pool)
            .await
            .unwrap_or(1);
            if exists > 0 {
                continue;
            }
            let res = sqlx::query(&format!(
                "UPDATE {table} SET ciyuanxi_id = ? WHERE ciyuanxi_id = ?"
            ))
            .bind(uid)
            .bind(&digits)
            .execute(pool)
            .await;
            if let Ok(r) = res {
                migrated += r.rows_affected() as usize;
            }
        }
    }
    if migrated > 0 {
        tracing::info!("[sync] remapped {} digit-key sync rows to full ciyuanxi_id", migrated);
    }
}

pub fn parse_chunk_index(name: &str) -> Option<i64> {
    let trimmed = name.strip_prefix("chunk_")?.strip_suffix(".json")?;
    trimmed.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chunk_index_extracts_number() {
        assert_eq!(parse_chunk_index("chunk_3.json"), Some(3));
        assert_eq!(parse_chunk_index("chunk_0.json"), Some(0));
        assert_eq!(parse_chunk_index("chunk_x.json"), None);
        assert_eq!(parse_chunk_index("other.json"), None);
        assert_eq!(parse_chunk_index("chunk_3"), None);
    }

    #[test]
    fn sync_dir_keeps_digits_only() {
        assert_eq!(sync_dir("abc123"), std::path::PathBuf::from("data/sync").join("123"));
    }
}
