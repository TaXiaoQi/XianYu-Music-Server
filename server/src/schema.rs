use sqlx::MySqlPool;
use sqlx::Row;
use tracing::warn;

/// 全部建表语句（供启动初始化和后台修复共用）
pub fn table_statements() -> &'static [&'static str] {
    &TABLE_STATEMENTS
}

/// 启动时确保核心表存在
pub async fn ensure_schema(pool: &MySqlPool) {
    for stmt in table_statements() {
        if let Err(e) = sqlx::query(stmt).execute(pool).await {
            warn!("schema init failed: {} -> {}", &stmt[..stmt.len().min(80)], e);
        }
    }
    ensure_feedback_log_columns(pool).await;
}

async fn ensure_column(pool: &MySqlPool, table: &str, column: &str, definition: &str) {
    let exists: i64 = sqlx::query(
        "SELECT COUNT(*) AS cnt FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = ? AND column_name = ?",
    )
    .bind(table)
    .bind(column)
    .fetch_one(pool)
    .await
    .map(|r| r.get("cnt"))
    .unwrap_or(0);
    if exists > 0 {
        return;
    }
    let sql = format!("ALTER TABLE `{}` ADD COLUMN `{}` {}", table, column, definition);
    if let Err(e) = sqlx::query(&sql).execute(pool).await {
        warn!("schema alter failed: {} -> {}", sql, e);
    }
}

async fn ensure_feedback_log_columns(pool: &MySqlPool) {
    ensure_column(pool, "user_feedback", "error_logs", "LONGTEXT").await;
    ensure_column(pool, "user_feedback", "all_logs", "LONGTEXT").await;
    ensure_column(pool, "user_feedback", "log_meta", "TEXT").await;
}

static TABLE_STATEMENTS: &[&str] = &[
        "CREATE TABLE IF NOT EXISTS `source_call_log` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `ip` varchar(45) NOT NULL DEFAULT '',
            `device_id` varchar(128) NOT NULL DEFAULT '',
            `source_name` varchar(64) NOT NULL DEFAULT '',
            `action` varchar(32) NOT NULL DEFAULT '',
            `song_name` varchar(255) NOT NULL DEFAULT '',
            `singer` varchar(255) NOT NULL DEFAULT '',
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `result_status` varchar(32) NOT NULL DEFAULT '',
            `error_msg` varchar(512) NOT NULL DEFAULT '',
            `platform` varchar(32) NOT NULL DEFAULT '',
            `source_type` varchar(32) NOT NULL DEFAULT 'ikun',
            `duration_ms` int(11) NOT NULL DEFAULT 0,
            `call_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `request_params` text,
            PRIMARY KEY (`id`),
            KEY `idx_call_time` (`call_time`),
            KEY `idx_source_name` (`source_name`),
            KEY `idx_status` (`status`),
            KEY `idx_ip` (`ip`),
            KEY `idx_platform` (`platform`),
            KEY `idx_source_type` (`source_type`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `login_log` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `ip` varchar(45) NOT NULL DEFAULT '',
            `device_id` varchar(128) NOT NULL DEFAULT '',
            `user_id` varchar(64) NOT NULL DEFAULT '',
            `username` varchar(64) NOT NULL DEFAULT '',
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `login_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `request_params` text,
            PRIMARY KEY (`id`),
            KEY `idx_login_time` (`login_time`),
            KEY `idx_user_id` (`user_id`),
            KEY `idx_ip` (`ip`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `error_log` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `ip` varchar(45) NOT NULL DEFAULT '',
            `device_id` varchar(128) NOT NULL DEFAULT '',
            `app_version` varchar(32) NOT NULL DEFAULT '',
            `os_version` varchar(32) NOT NULL DEFAULT '',
            `device_model` varchar(64) NOT NULL DEFAULT '',
            `device_brand` varchar(64) NOT NULL DEFAULT '',
            `error_type` varchar(64) NOT NULL DEFAULT '',
            `platform` varchar(32) NOT NULL DEFAULT '',
            `error_message` text,
            `error_stack` text,
            `page` varchar(64) NOT NULL DEFAULT '',
            `error_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `request_params` text,
            PRIMARY KEY (`id`),
            KEY `idx_error_time` (`error_time`),
            KEY `idx_ip` (`ip`),
            KEY `idx_device_id` (`device_id`),
            KEY `idx_error_type` (`error_type`),
            KEY `idx_platform` (`platform`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `app_users` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `username` varchar(64) NOT NULL DEFAULT '',
            `password` varchar(255) NOT NULL DEFAULT '',
            `email` varchar(128) NOT NULL DEFAULT '',
            `email_verified` tinyint(1) NOT NULL DEFAULT 0,
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `avatar_url` LONGTEXT,
            `background_url` LONGTEXT,
            `signature` varchar(255) NOT NULL DEFAULT '',
            `listen_duration` int(11) unsigned NOT NULL DEFAULT 0,
            `unique_songs_count` int(11) unsigned NOT NULL DEFAULT 0,
            `master_quota` int(11) NOT NULL DEFAULT 0,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_username` (`username`),
            UNIQUE KEY `uk_email` (`email`),
            KEY `idx_status` (`status`),
            KEY `idx_created_at` (`created_at`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `email_verify_codes` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `email` varchar(128) NOT NULL DEFAULT '',
            `code` varchar(8) NOT NULL DEFAULT '',
            `type` varchar(32) NOT NULL DEFAULT 'register',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `used` tinyint(1) NOT NULL DEFAULT 0,
            `expired_at` datetime NOT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_email` (`email`),
            KEY `idx_code` (`code`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `human_captcha_challenges` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `captcha_id` varchar(64) NOT NULL DEFAULT '',
            `purpose` varchar(32) NOT NULL DEFAULT 'auth',
            `answer` varchar(16) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `used` tinyint(1) NOT NULL DEFAULT 0,
            `expires_at` datetime NOT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_captcha_id` (`captcha_id`),
            KEY `idx_purpose_ip` (`purpose`, `ip`),
            KEY `idx_expires_at` (`expires_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `auth_rate_limits` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `action` varchar(32) NOT NULL DEFAULT '',
            `identifier` varchar(128) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `failed_count` int(11) NOT NULL DEFAULT 0,
            `locked_until` datetime DEFAULT NULL,
            `last_failed_at` datetime DEFAULT NULL,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_action_identifier_ip` (`action`, `identifier`, `ip`),
            KEY `idx_locked_until` (`locked_until`),
            KEY `idx_updated_at` (`updated_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `admin_users` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `username` varchar(64) NOT NULL DEFAULT '',
            `password` varchar(255) NOT NULL DEFAULT '',
            `email` varchar(128) NOT NULL DEFAULT '',
            `role` varchar(32) NOT NULL DEFAULT 'admin',
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_username` (`username`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `admin_operation_log` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `admin_id` bigint(20) unsigned NOT NULL DEFAULT 0,
            `admin_username` varchar(64) NOT NULL DEFAULT '',
            `action` varchar(128) NOT NULL DEFAULT '',
            `target` varchar(255) NOT NULL DEFAULT '',
            `detail` text,
            `ip` varchar(45) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_admin_id` (`admin_id`),
            KEY `idx_action` (`action`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `admin_login_log` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `admin_id` bigint(20) unsigned NOT NULL DEFAULT 0,
            `admin_username` varchar(64) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `user_agent` varchar(255) NOT NULL DEFAULT '',
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_admin_id` (`admin_id`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `server_settings` (
            `setting_key` varchar(64) NOT NULL DEFAULT '',
            `setting_value` text,
            `description` varchar(255) NOT NULL DEFAULT '',
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`setting_key`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('feedback_daily_limit', '20', '每个用户每天可提交的问题反馈数量上限，0 表示不限制')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('wallpaper_upload_limit', '20', '每个用户最多可上传的壁纸数量，0 表示不限制')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('email_api_primary', '', '邮箱发送 API 主地址，留空则使用环境变量默认值')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('email_api_backup', '', '邮箱发送 API 备用地址，留空则使用环境变量默认值')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('email_sender', '', '发件邮箱地址，留空则使用环境变量默认值')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('email_password', '', '发件邮箱密码/授权码，留空则使用环境变量默认值')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('email_provider', 'builtin', '邮件发送方式：builtin=内置邮箱机，http_api=外部HTTP API，smtp=标准SMTP')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('smtp_host', '', 'SMTP 服务器地址，如 smtp.qq.com')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('smtp_port', '465', 'SMTP 端口，SSL通常465，STARTTLS通常587')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('smtp_username', '', 'SMTP 登录用户名，通常与发件邮箱相同')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('smtp_password', '', 'SMTP 登录密码/授权码，留空则使用 email_password 的值')",
        "CREATE TABLE IF NOT EXISTS `wallpaper_upload_limits` (
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `upload_limit` int(11) NOT NULL DEFAULT 20,
            `remark` varchar(255) NOT NULL DEFAULT '',
            `updated_by` varchar(64) NOT NULL DEFAULT '',
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`ciyuanxi_id`),
            KEY `idx_upload_limit` (`upload_limit`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `email_templates` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `name` varchar(128) NOT NULL DEFAULT '',
            `subject` varchar(255) NOT NULL DEFAULT '',
            `body` text,
            `variables` text,
            `is_default` tinyint(1) NOT NULL DEFAULT 0,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_is_default` (`is_default`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `email_send_log` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `email` varchar(128) NOT NULL DEFAULT '',
            `subject` varchar(255) NOT NULL DEFAULT '',
            `interface_id` bigint(20) unsigned NOT NULL DEFAULT 0,
            `template_id` bigint(20) unsigned NOT NULL DEFAULT 0,
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `error_msg` text,
            `ip` varchar(45) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_email` (`email`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `music_source_config` (
            `id` int(11) NOT NULL AUTO_INCREMENT,
            `source_name` varchar(64) NOT NULL DEFAULT '',
            `source_code` varchar(32) NOT NULL DEFAULT '',
            `is_enabled` tinyint(1) NOT NULL DEFAULT 1,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_source_code` (`source_code`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `app_versions` (
            `id` int(11) NOT NULL AUTO_INCREMENT,
            `app_name` varchar(128) NOT NULL DEFAULT '',
            `version_code` varchar(32) NOT NULL DEFAULT '',
            `download_url` varchar(512) NOT NULL DEFAULT '',
            `update_content` text,
            `status` varchar(32) NOT NULL DEFAULT 'normal',
            `file_size` bigint(20) unsigned NOT NULL DEFAULT 0,
            `message` varchar(512) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_version_code` (`version_code`),
            KEY `idx_status` (`status`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `user_settings` (
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `stream_cache_enabled` tinyint(1) NOT NULL DEFAULT 1,
            `startup_play_enabled` tinyint(1) NOT NULL DEFAULT 0,
            `bluetooth_lyric_enabled` tinyint(1) NOT NULL DEFAULT 0,
            `download_lyric_enabled` tinyint(1) NOT NULL DEFAULT 1,
            `download_cover_enabled` tinyint(1) NOT NULL DEFAULT 1,
            `download_artist_enabled` tinyint(1) NOT NULL DEFAULT 0,
            `search_board_enabled` tinyint(1) NOT NULL DEFAULT 1,
            `page_animation_enabled` tinyint(1) NOT NULL DEFAULT 1,
            `default_quality` varchar(32) NOT NULL DEFAULT 'standard',
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`ciyuanxi_id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `tv_login_codes` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `code` varchar(64) NOT NULL DEFAULT '',
            `device_id` varchar(128) NOT NULL DEFAULT '',
            `status` varchar(16) NOT NULL DEFAULT 'pending',
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `token` varchar(64) NOT NULL DEFAULT '',
            `ip` varchar(64) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `scanned_at` datetime DEFAULT NULL,
            `logged_in_at` datetime DEFAULT NULL,
            `expires_at` datetime NOT NULL,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_code` (`code`),
            KEY `idx_device_id` (`device_id`),
            KEY `idx_status` (`status`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `user_feedback` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `nickname` varchar(64) NOT NULL DEFAULT '',
            `title` varchar(60) NOT NULL DEFAULT '',
            `content` text,
            `error_logs` LONGTEXT,
            `all_logs` LONGTEXT,
            `log_meta` text,
            `status` varchar(16) NOT NULL DEFAULT 'pending',
            `admin_reply` text,
            `replied_at` datetime DEFAULT NULL,
            `replied_by` varchar(64) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_status` (`status`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `user_playlists` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `user_id` varchar(32) NOT NULL DEFAULT '0' COMMENT '所属用户弦予号(ciyuanxi_id)',
            `name` varchar(100) NOT NULL DEFAULT '',
            `description` varchar(500) NOT NULL DEFAULT '',
            `cover_url` varchar(512) NOT NULL DEFAULT '',
            `cover_path` varchar(255) NOT NULL DEFAULT '',
            `song_count` int(11) unsigned NOT NULL DEFAULT 0,
            `is_favorite` tinyint(1) NOT NULL DEFAULT 0,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_user_id` (`user_id`),
            KEY `idx_is_favorite` (`is_favorite`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `user_playlist_songs` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `playlist_id` bigint(20) unsigned NOT NULL DEFAULT 0,
            `user_id` varchar(32) NOT NULL DEFAULT '0',
            `song_hash` varchar(64) NOT NULL DEFAULT '',
            `song_name` varchar(200) NOT NULL DEFAULT '',
            `singer` varchar(200) NOT NULL DEFAULT '',
            `album` varchar(200) NOT NULL DEFAULT '',
            `cover_url` varchar(512) NOT NULL DEFAULT '',
            `duration` int(11) unsigned NOT NULL DEFAULT 0,
            `source` varchar(16) NOT NULL DEFAULT '',
            `song_url` varchar(512) NOT NULL DEFAULT '',
            `original_id` varchar(64) NOT NULL DEFAULT '',
            `sort_order` int(11) NOT NULL DEFAULT 0,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_playlist_song` (`playlist_id`, `song_hash`),
            KEY `idx_playlist_id` (`playlist_id`),
            KEY `idx_user_id` (`user_id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `user_avatar_pending` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `avatar_data` LONGTEXT NOT NULL,
            `status` varchar(16) NOT NULL DEFAULT 'pending',
            `reviewed_at` datetime DEFAULT NULL,
            `reviewed_by` varchar(64) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_status` (`status`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `user_nickname_pending` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `nickname` varchar(64) NOT NULL DEFAULT '',
            `status` varchar(16) NOT NULL DEFAULT 'pending',
            `reviewed_at` datetime DEFAULT NULL,
            `reviewed_by` varchar(64) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_status` (`status`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `notification_emails` (
            `id` int(11) NOT NULL AUTO_INCREMENT,
            `email` varchar(128) NOT NULL DEFAULT '',
            `remark` varchar(128) NOT NULL DEFAULT '',
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_email` (`email`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `wallpapers` (
            `id` int(11) NOT NULL AUTO_INCREMENT,
            `title` varchar(128) NOT NULL DEFAULT '',
            `description` varchar(512) NOT NULL DEFAULT '',
            `image_url` varchar(512) NOT NULL DEFAULT '',
            `thumbnail_url` varchar(512) NOT NULL DEFAULT '',
            `category` varchar(64) NOT NULL DEFAULT '默认',
            `sort_order` int(11) NOT NULL DEFAULT 0,
            `status` varchar(32) NOT NULL DEFAULT 'normal',
            `uploaded_by` varchar(32) NOT NULL DEFAULT '',
            `uploaded_by_nickname` varchar(64) NOT NULL DEFAULT '',
            `reviewed_at` datetime NULL,
            `reviewed_by` varchar(64) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_status` (`status`),
            KEY `idx_sort` (`sort_order`),
            KEY `idx_uploaded_by` (`uploaded_by`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `email_test_users` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `email` varchar(128) NOT NULL DEFAULT '',
            `nickname` varchar(64) NOT NULL DEFAULT '',
            `password` varchar(255) NOT NULL DEFAULT '',
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `last_login` datetime DEFAULT NULL,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_email` (`email`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `email_test_codes` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `email` varchar(128) NOT NULL DEFAULT '',
            `code` varchar(8) NOT NULL DEFAULT '',
            `type` varchar(32) NOT NULL DEFAULT 'register',
            `used` tinyint(1) NOT NULL DEFAULT 0,
            `expired_at` datetime NOT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_email` (`email`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `email_test_logs` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `user_id` bigint(20) unsigned NOT NULL DEFAULT 0,
            `email` varchar(128) NOT NULL DEFAULT '',
            `action` varchar(64) NOT NULL DEFAULT '',
            `detail` varchar(255) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_user_id` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `share_log` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `share_id` varchar(16) NOT NULL DEFAULT '',
            `song_name` varchar(255) NOT NULL DEFAULT '',
            `singer` varchar(255) NOT NULL DEFAULT '',
            `audio_url` varchar(1024) NOT NULL DEFAULT '',
            `lyrics` text,
            `cover_path` varchar(512) NOT NULL DEFAULT '',
            `creator_ip` varchar(45) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `expired_at` datetime NOT NULL,
            `view_count` int(11) NOT NULL DEFAULT 0,
            `request_params` text,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_share_id` (`share_id`),
            KEY `idx_created_at` (`created_at`),
            KEY `idx_expired_at` (`expired_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `ciyuanxi_pretty_ids` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `assigned_user_id` varchar(32) NOT NULL DEFAULT '0',
            `assigned_at` datetime DEFAULT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_assigned_user_id` (`assigned_user_id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `master_quota_usage_log` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `deducted_count` int(11) NOT NULL DEFAULT 1,
            `remaining_after` int(11) NOT NULL DEFAULT 0,
            `reason` varchar(64) NOT NULL DEFAULT '',
            `master_type` varchar(32) NOT NULL DEFAULT '',
            `song_name` varchar(255) NOT NULL DEFAULT '',
            `singer` varchar(255) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_created_at` (`created_at`),
            KEY `idx_reason` (`reason`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `play_history` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `user_id` bigint(20) unsigned NOT NULL DEFAULT 0,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `song_hash` varchar(64) NOT NULL DEFAULT '',
            `song_name` varchar(200) NOT NULL DEFAULT '',
            `singer` varchar(200) NOT NULL DEFAULT '',
            `cover_url` varchar(512) NOT NULL DEFAULT '',
            `source` varchar(16) NOT NULL DEFAULT '',
            `duration` int(11) unsigned NOT NULL DEFAULT 0,
            `played_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_user_id` (`user_id`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_played_at` (`played_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `admin_app_login_log` (
            `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
            `admin_id` bigint(20) unsigned NOT NULL DEFAULT 0,
            `admin_username` varchar(64) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `user_agent` varchar(255) NOT NULL DEFAULT '',
            `device_id` varchar(128) NOT NULL DEFAULT '',
            `device_model` varchar(64) NOT NULL DEFAULT '',
            `app_version` varchar(32) NOT NULL DEFAULT '',
            `os_version` varchar(32) NOT NULL DEFAULT '',
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `extra` varchar(255) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_admin_id` (`admin_id`),
            KEY `idx_created_at` (`created_at`),
            KEY `idx_ip` (`ip`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
];
