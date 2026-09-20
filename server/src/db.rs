use anyhow::Result;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::Executor;

pub async fn connect(cfg: &crate::config::Config) -> Result<MySqlPool> {
    let dsn = format!(
        "mysql://{}:{}@{}:{}/{}?charset={}",
        cfg.db_user, cfg.db_pass, cfg.db_host, cfg.db_port, cfg.db_name, cfg.db_charset
    );
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .after_connect(|conn, _meta| Box::pin(async move {
            conn.execute("SET time_zone = '+00:00'").await?;
            Ok(())
        }))
        .connect_lazy_with(dsn.parse()?);
    Ok(pool)
}

pub async fn ping(_cfg: &crate::config::Config, pool: &MySqlPool) {
    if let Err(e) = sqlx::query("SELECT 1").execute(pool).await {
        tracing::warn!("initial db ping failed: {}", e);
    }
}
