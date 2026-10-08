use serde_json::Value;
use sqlx::MySqlPool;
use tracing::warn;

/// 结构化站点配置（关于页、公告、兜底模块、桌面端版本列表）的统一入库存储。
/// 表：site_configs(`key`, `value` json, updated_at)。
/// 读取：DB 未命中时回退读旧 JSON 文件并回填 DB（懒迁移，旧文件仅读不删）；
/// 写入：只写 DB，DB 为唯一事实源。

pub async fn get_json(
    pool: &MySqlPool,
    key: &str,
    legacy_path: Option<&std::path::Path>,
) -> Option<Value> {
    // CAST 为 CHAR：MySQL JSON 列的元数据类型 sqlx 无法直接解码成 String
    if let Ok(raw) = sqlx::query_scalar::<_, String>(
        "SELECT CAST(`value` AS CHAR) FROM site_configs WHERE `key` = ?",
    )
    .bind(key)
    .fetch_one(pool)
    .await
    {
        return serde_json::from_str(&raw).ok();
    }
    let path = legacy_path?;
    let content = tokio::fs::read_to_string(path).await.ok()?;
    let value: Value = serde_json::from_str(&content).ok()?;
    // 回填 DB（旧文件内容已校验为合法 JSON）；失败不影响本次读取
    if let Err(e) = sqlx::query("INSERT IGNORE INTO site_configs (`key`, `value`) VALUES (?, ?)")
        .bind(key)
        .bind(&content)
        .execute(pool)
        .await
    {
        warn!("site_configs backfill failed: key={} -> {}", key, e);
    }
    Some(value)
}

pub async fn set_json(pool: &MySqlPool, key: &str, value: &Value) -> Result<(), sqlx::Error> {
    let content = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
    sqlx::query(
        "INSERT INTO site_configs (`key`, `value`) VALUES (?, ?)
         ON DUPLICATE KEY UPDATE `value` = VALUES(`value`)",
    )
    .bind(key)
    .bind(&content)
    .execute(pool)
    .await
    .map(|_| ())
}
