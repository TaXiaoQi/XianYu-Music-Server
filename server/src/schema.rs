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
    ensure_column(pool, "app_users", "email_verified", "tinyint(1) NOT NULL DEFAULT 0").await;
    ensure_column(pool, "app_users", "ciyuanxi_id", "varchar(32) NOT NULL DEFAULT ''").await;
    ensure_column(pool, "app_users", "avatar_url", "LONGTEXT NULL").await;
    ensure_column(pool, "app_users", "master_quota", "int(11) NOT NULL DEFAULT 0").await;
    ensure_column(pool, "app_users", "last_device_id", "varchar(128) NOT NULL DEFAULT ''").await;
    ensure_column(pool, "app_users", "ban_reason", "varchar(255) NOT NULL DEFAULT ''").await;
    ensure_column(pool, "app_users", "listen_duration", "int(11) unsigned NOT NULL DEFAULT 0").await;
    ensure_column(pool, "app_users", "unique_songs_count", "int(11) unsigned NOT NULL DEFAULT 0").await;
    ensure_column(pool, "app_users", "background_url", "LONGTEXT NULL").await;
    ensure_column(pool, "app_users", "signature", "varchar(255) NOT NULL DEFAULT ''").await;
    ensure_column(pool, "app_users", "listen_stats_reset_at", "datetime NULL").await;
    // 听歌统计重置基准偏移量：重置后客户端首次上报的累计值作为基准，后续上报减去基准得到真实增量
    ensure_column(pool, "app_users", "listen_duration_offset", "bigint(20) NOT NULL DEFAULT 0").await;
    ensure_column(pool, "app_users", "unique_songs_offset", "int(11) NOT NULL DEFAULT 0").await;
    // 弦予号每月限改：记录最近一次修改时间
    ensure_column(pool, "app_users", "ciyuanxi_id_updated_at", "datetime NULL").await;
    ensure_column(pool, "listen_daily_stats", "unique_songs_count", "int(11) unsigned NOT NULL DEFAULT 0").await;
    // 账号系统重构：app_users.username 改为 nickname（仅改应用用户表，不动管理员/日志表）
    ensure_app_users_username_to_nickname(pool).await;
    // 管理员头像：平滑补列
    ensure_column(pool, "admin_users", "avatar_url", "varchar(512) NOT NULL DEFAULT ''").await;
    // 管理员账号邮箱：用于后台通知接收与快捷导入外部通知
    ensure_column(pool, "admin_users", "email", "varchar(128) NOT NULL DEFAULT ''").await;
    // 通知邮箱板块开关：壁纸审核 / 头像 / 昵称 / 反馈更新
    ensure_column(pool, "notification_emails", "notify_wallpaper", "tinyint(1) NOT NULL DEFAULT 1").await;
    ensure_column(pool, "notification_emails", "notify_avatar", "tinyint(1) NOT NULL DEFAULT 1").await;
    ensure_column(pool, "notification_emails", "notify_nickname", "tinyint(1) NOT NULL DEFAULT 1").await;
    ensure_column(pool, "notification_emails", "notify_feedback", "tinyint(1) NOT NULL DEFAULT 1").await;
    // 壁纸表后置补列：旧库可能缺少新增列，缺列会导致列表查询报“数据库错误”
    ensure_column(pool, "wallpapers", "category", "varchar(64) NOT NULL DEFAULT '默认'").await;
    ensure_column(pool, "wallpapers", "sort_order", "int(11) NOT NULL DEFAULT 0").await;
    ensure_column(pool, "wallpapers", "uploaded_by_nickname", "varchar(64) NOT NULL DEFAULT ''").await;
    ensure_column(pool, "wallpapers", "reviewed_at", "datetime NULL").await;
    ensure_column(pool, "wallpapers", "reviewed_by", "varchar(64) NOT NULL DEFAULT ''").await;
    // 日推画像聚合与排行榜查询加速：ciyuanxi_id + played_at 复合索引
    ensure_index(pool, "play_history", "idx_ciyuanxi_played", "ciyuanxi_id, played_at").await;
    ensure_default_admin(pool).await;
}

/// 确保至少有一个管理员账号，如果 admin_users 表为空则创建默认 admin
async fn ensure_default_admin(pool: &MySqlPool) {
    let count: i64 = sqlx::query("SELECT COUNT(*) AS cnt FROM admin_users")
        .fetch_one(pool)
        .await
        .map(|r| r.get("cnt"))
        .unwrap_or(0);
    if count > 0 {
        return;
    }
    // 使用 bcrypt 哈希默认密码 adminadmin
    let hash = match bcrypt::hash("adminadmin", 10) {
        Ok(h) => h,
        Err(e) => {
            warn!("failed to hash default admin password: {}", e);
            return;
        }
    };
    let _ = sqlx::query(
        "INSERT IGNORE INTO admin_users (username, password, avatar_url, role, status) VALUES (?, ?, '', ?, 1)",
    )
    .bind("admin")
    .bind(&hash)
    .bind("super_admin")
    .execute(pool)
    .await;
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

async fn ensure_index(pool: &MySqlPool, table: &str, index_name: &str, columns: &str) {
    let exists: i64 = sqlx::query(
        "SELECT COUNT(*) AS cnt FROM information_schema.statistics WHERE table_schema = DATABASE() AND table_name = ? AND index_name = ?",
    )
    .bind(table)
    .bind(index_name)
    .fetch_one(pool)
    .await
    .map(|r| r.get("cnt"))
    .unwrap_or(0);
    if exists > 0 {
        return;
    }
    let sql = format!("ALTER TABLE `{}` ADD INDEX `{}` ({})", table, index_name, columns);
    if let Err(e) = sqlx::query(&sql).execute(pool).await {
        warn!("schema index failed: {} -> {}", sql, e);
    }
}



async fn ensure_feedback_log_columns(pool: &MySqlPool) {
    ensure_column(pool, "user_feedback", "error_logs", "LONGTEXT").await;
    ensure_column(pool, "user_feedback", "all_logs", "LONGTEXT").await;
    ensure_column(pool, "user_feedback", "log_meta", "TEXT").await;
    // 申诉与普通反馈共用 user_feedback 表，用 category 区分：feedback / appeal
    ensure_column(pool, "user_feedback", "category", "VARCHAR(16) NOT NULL DEFAULT 'feedback'").await;
    // 认领人（管理员账号名）与完成说明、通知确认时间（反馈 todo 化）
    ensure_column(pool, "user_feedback", "assignee", "VARCHAR(64) NOT NULL DEFAULT ''").await;
    ensure_column(pool, "user_feedback", "resolve_note", "TEXT").await;
    // 拒绝理由：反馈被拒绝时必填，展示给提交用户（与 resolve_note 相对）
    ensure_column(pool, "user_feedback", "reject_reason", "TEXT").await;
    ensure_column(pool, "user_feedback", "notified_at", "DATETIME DEFAULT NULL").await;
    // 认领时间与完成时间（用于后台反馈时间线展示）
    ensure_column(pool, "user_feedback", "claimed_at", "DATETIME DEFAULT NULL").await;
    ensure_column(pool, "user_feedback", "resolved_at", "DATETIME DEFAULT NULL").await;
    // 反馈类型：problem（问题反馈）/ suggestion（功能建议），images 保存图片 URL 的 JSON 数组
    ensure_column(pool, "user_feedback", "feedback_type", "VARCHAR(16) NOT NULL DEFAULT 'problem'").await;
    ensure_column(pool, "user_feedback", "images", "TEXT").await;
    // 平台版本：desktop（桌面版）/ mobile（移动版）/ watch（腕上版，预留），后台创建时由管理员选择
    ensure_column(pool, "user_feedback", "platform", "VARCHAR(32) NOT NULL DEFAULT ''").await;
    // 客户端反馈时上报的具体应用版本号（如 1.1.4-beta1），用于后台按版本定位问题
    ensure_column(pool, "user_feedback", "app_version", "VARCHAR(32) NOT NULL DEFAULT ''").await;
    // 回收站：软删除时间与删除人，14天后自动过期
    ensure_column(pool, "user_feedback", "deleted_at", "DATETIME DEFAULT NULL").await;
    ensure_column(pool, "user_feedback", "deleted_by", "VARCHAR(64) NOT NULL DEFAULT ''").await;
    // 协同功能：collaborators 存储所有协作者列表（JSON 数组），completed_by 存储已完成者列表
    ensure_column(pool, "user_feedback", "collaborators", "TEXT").await;
    ensure_column(pool, "user_feedback", "completed_by", "TEXT").await;
    // 完成反馈时附带的图片（管理员在完成弹窗上传），存图片 URL 的 JSON 数组
    ensure_column(pool, "user_feedback", "resolve_images", "TEXT").await;
}

/// 账号系统重构迁移：将 app_users.username 列改名为 nickname。
/// 仅迁移应用用户表，不影响 admin_users / 各类日志表。
/// 若存在 username 列且尚不存在 nickname 列，则执行改名并重建唯一键。
async fn ensure_app_users_username_to_nickname(pool: &MySqlPool) {
    // 检查 username 列是否存在
    let has_username: i64 = sqlx::query(
        "SELECT COUNT(*) AS cnt FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = 'app_users' AND column_name = 'username'",
    )
    .fetch_one(pool)
    .await
    .map(|r| r.get("cnt"))
    .unwrap_or(0);

    // 检查 nickname 列是否存在
    let has_nickname: i64 = sqlx::query(
        "SELECT COUNT(*) AS cnt FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = 'app_users' AND column_name = 'nickname'",
    )
    .fetch_one(pool)
    .await
    .map(|r| r.get("cnt"))
    .unwrap_or(0);

    if has_username > 0 && has_nickname == 0 {
        if let Err(e) = sqlx::query("ALTER TABLE `app_users` CHANGE COLUMN `username` `nickname` varchar(64) NOT NULL DEFAULT ''")
            .execute(pool)
            .await
        {
            warn!("schema rename username->nickname failed: {}", e);
            return;
        }
        // 删除旧唯一键（若存在），再重建为 nickname 唯一键
        let _ = sqlx::query("ALTER TABLE `app_users` DROP INDEX `uk_username`")
            .execute(pool)
            .await;
        let _ = sqlx::query("ALTER TABLE `app_users` ADD UNIQUE KEY `uk_nickname` (`nickname`)")
            .execute(pool)
            .await;
        warn!("schema migrated: app_users.username -> nickname");
    } else if has_nickname == 0 {
        // 全新表：确保 nickname 列与唯一键存在
        ensure_column(pool, "app_users", "nickname", "varchar(64) NOT NULL DEFAULT ''").await;
    }
}

static TABLE_STATEMENTS: &[&str] = &[
        "CREATE TABLE IF NOT EXISTS `source_call_log` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `nickname` varchar(64) NOT NULL DEFAULT '',
            `password` varchar(255) NOT NULL DEFAULT '',
            `email` varchar(128) NOT NULL DEFAULT '',
            `email_verified` tinyint(1) NOT NULL DEFAULT 0,
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `ban_reason` varchar(255) NOT NULL DEFAULT '',
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `ciyuanxi_id_updated_at` datetime NULL,
            `avatar_url` LONGTEXT,
            `background_url` LONGTEXT,
            `signature` varchar(255) NOT NULL DEFAULT '',
            `listen_duration` int(11) unsigned NOT NULL DEFAULT 0,
            `unique_songs_count` int(11) unsigned NOT NULL DEFAULT 0,
            `master_quota` int(11) NOT NULL DEFAULT 0,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_nickname` (`nickname`),
            UNIQUE KEY `uk_email` (`email`),
            KEY `idx_status` (`status`),
            KEY `idx_created_at` (`created_at`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `email_verify_codes` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
        "CREATE TABLE IF NOT EXISTS `user_tokens` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `token` varchar(128) NOT NULL,
            `ciyuanxi_id` varchar(64) NOT NULL DEFAULT '',
            `device_id` varchar(128) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `last_used_at` datetime DEFAULT NULL,
            `expires_at` datetime NOT NULL,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_token` (`token`),
            KEY `idx_ciyuanxi` (`ciyuanxi_id`),
            KEY `idx_expires` (`expires_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `api_rate_events` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `level` varchar(16) NOT NULL DEFAULT '',
            `action` varchar(64) NOT NULL DEFAULT '',
            `identity_type` varchar(32) NOT NULL DEFAULT '',
            `identity` varchar(128) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `request_count` int(11) NOT NULL DEFAULT 0,
            `threshold_count` int(11) NOT NULL DEFAULT 0,
            `window_seconds` int(11) NOT NULL DEFAULT 0,
            `reason` varchar(255) NOT NULL DEFAULT '',
            `blocked_until` datetime DEFAULT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_level_created` (`level`, `created_at`),
            KEY `idx_action_created` (`action`, `created_at`),
            KEY `idx_identity_created` (`identity_type`, `identity`, `created_at`),
            KEY `idx_ip_created` (`ip`, `created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `api_temp_blocks` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `identity_type` varchar(32) NOT NULL DEFAULT '',
            `identity` varchar(128) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `reason` varchar(255) NOT NULL DEFAULT '',
            `expires_at` datetime NOT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_identity` (`identity_type`, `identity`),
            KEY `idx_expires_at` (`expires_at`),
            KEY `idx_ip` (`ip`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `admin_users` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `username` varchar(64) NOT NULL DEFAULT '',
            `password` varchar(255) NOT NULL DEFAULT '',
            `email` varchar(128) NOT NULL DEFAULT '',
            `avatar_url` varchar(512) NOT NULL DEFAULT '',
            `role` varchar(32) NOT NULL DEFAULT 'admin',
            `status` tinyint(1) NOT NULL DEFAULT 1,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_username` (`username`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `admin_operation_log` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `admin_id` bigint(20) NOT NULL DEFAULT 0,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `admin_id` bigint(20) NOT NULL DEFAULT 0,
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
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('smtp_accounts', '[]', '内置邮箱机 SMTP 账号池 JSON，支持多个邮箱轮换发送')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('captcha_enabled', '0', '是否启用人机验证：1=启用，0=关闭')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('captcha_provider', 'turnstile', '人机验证服务商：turnstile、hcaptcha、off')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('captcha_site_key', '', '人机验证 Site Key（前端展示用）')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('captcha_secret', '', '人机验证 Secret Key（后端校验用，留空则回退环境变量）')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('turnstile_enabled', '0', '是否启用 Cloudflare Turnstile 人机验证：1=启用，0=关闭')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('turnstile_site_key', '', 'Cloudflare Turnstile Site Key（前端展示用）')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('turnstile_secret', '', 'Cloudflare Turnstile Secret Key（后端校验用，留空则回退环境变量 TURNSTILE_SECRET）')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('auto_backup_enabled', '0', '自动备份开关：1=开启，0=关闭')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('auto_backup_interval', '1440', '自动备份间隔（分钟），默认 1440=每天')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('auto_backup_max_count', '20', '自动备份最大保留份数，超出自动清理最旧备份')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('auto_backup_mode', 'full', '备份模式：full=全量，incremental=增量')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('commtool_enabled', '0', '通信工具服务开关：1=开启，0=关闭')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('commtool_port', '8090', '通信工具服务监听端口')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('commtool_token', '', '通信工具连接鉴权令牌（空=不鉴权，支持 query token 或 Authorization Bearer）')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('ws_client_url', '', 'WS客户端连接地址（配置后用于自动连接/重连）')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('ws_client_auto_reconnect', '0', 'WS客户端自动重连开关：1=开启，0=关闭')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('ws_client_reconnect_interval', '10', 'WS客户端重连间隔(秒)')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('ws_client_heartbeat_interval', '30', 'WS客户端心跳间隔(秒)')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('webhook_enabled', '0', '通用Webhook开关：1=开启，0=关闭')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('webhook_url', '', '通用Webhook回调地址')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('webhook_method', 'POST', '通用Webhook请求方法')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('webhook_headers', '', '通用Webhook自定义请求头（每行 Key: Value）')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('webhook_body_template', '', '通用Webhook请求体模板（支持 {{event}} {{title}} {{detail}} {{image_url}} {{link}} {{time}} 占位符）')",
        "INSERT IGNORE INTO `server_settings` (`setting_key`, `setting_value`, `description`) VALUES ('webhook_modules', '', '通用Webhook触发板块，逗号分隔')",
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `email` varchar(128) NOT NULL DEFAULT '',
            `subject` varchar(255) NOT NULL DEFAULT '',
            `interface_id` bigint(20) NOT NULL DEFAULT 0,
            `template_id` bigint(20) NOT NULL DEFAULT 0,
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
            `file_size` bigint(20) NOT NULL DEFAULT 0,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `collaborators` text COMMENT '协同认领人列表(JSON数组)',
            `completed_by` text COMMENT '协同完成确认人列表(JSON数组)',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_status` (`status`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `user_playlists` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `playlist_id` bigint(20) NOT NULL DEFAULT 0,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `notify_wallpaper` tinyint(1) NOT NULL DEFAULT 1,
            `notify_avatar` tinyint(1) NOT NULL DEFAULT 1,
            `notify_nickname` tinyint(1) NOT NULL DEFAULT 1,
            `notify_feedback` tinyint(1) NOT NULL DEFAULT 1,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_email` (`email`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `comm_clients` (
            `id` int(11) NOT NULL AUTO_INCREMENT,
            `name` varchar(128) NOT NULL DEFAULT '',
            `type` varchar(20) NOT NULL DEFAULT 'ws',
            `url` varchar(512) NOT NULL DEFAULT '',
            `events` varchar(512) NOT NULL DEFAULT '',
            `enabled` tinyint(1) NOT NULL DEFAULT 1,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`)
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `user_id` bigint(20) NOT NULL DEFAULT 0,
            `email` varchar(128) NOT NULL DEFAULT '',
            `action` varchar(64) NOT NULL DEFAULT '',
            `detail` varchar(255) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_user_id` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `share_log` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `assigned_user_id` varchar(32) NOT NULL DEFAULT '0',
            `assigned_at` datetime DEFAULT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_assigned_user_id` (`assigned_user_id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `master_quota_usage_log` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
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
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `user_id` bigint(20) NOT NULL DEFAULT 0,
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
            KEY `idx_played_at` (`played_at`),
            KEY `idx_ciyuanxi_played` (`ciyuanxi_id`, `played_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `admin_app_login_log` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `admin_id` bigint(20) NOT NULL DEFAULT 0,
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
        "CREATE TABLE IF NOT EXISTS `app_open_log` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `device_id` varchar(128) NOT NULL DEFAULT '',
            `app_version` varchar(32) NOT NULL DEFAULT '',
            `os_version` varchar(32) NOT NULL DEFAULT '',
            `device_model` varchar(64) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_device_id` (`device_id`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `user_announcement_confirmations` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `device_id` varchar(128) NOT NULL DEFAULT '',
            `announcement_id` varchar(64) NOT NULL DEFAULT '',
            `announcement_title` varchar(255) NOT NULL DEFAULT '',
            `announcement_updated_at` varchar(32) NOT NULL DEFAULT '',
            `ip` varchar(45) NOT NULL DEFAULT '',
            `confirmed_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_user_announcement_version` (`ciyuanxi_id`, `device_id`, `announcement_id`, `announcement_updated_at`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_announcement_id` (`announcement_id`),
            KEY `idx_confirmed_at` (`confirmed_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `listen_daily_stats` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `stat_date` date NOT NULL,
            `listen_duration` int(11) unsigned NOT NULL DEFAULT 0,
            `unique_songs_count` int(11) unsigned NOT NULL DEFAULT 0,
            `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_user_date` (`ciyuanxi_id`, `stat_date`),
            KEY `idx_stat_date` (`stat_date`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `banned_devices` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `device_id` varchar(128) NOT NULL DEFAULT '',
            `reason` varchar(255) NOT NULL DEFAULT '',
            `banned_by` varchar(64) NOT NULL DEFAULT '',
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            UNIQUE KEY `uk_device_id` (`device_id`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `nickname_change_notices` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `ciyuanxi_id` varchar(32) NOT NULL DEFAULT '',
            `old_nickname` varchar(64) NOT NULL DEFAULT '',
            `new_nickname` varchar(64) NOT NULL DEFAULT '',
            `reason` varchar(255) NOT NULL DEFAULT '',
            `changed_by` varchar(64) NOT NULL DEFAULT '',
            `confirmed_at` datetime DEFAULT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_ciyuanxi_id` (`ciyuanxi_id`),
            KEY `idx_confirmed_at` (`confirmed_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `feedback_collab_requests` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `feedback_id` bigint(20) NOT NULL DEFAULT 0,
            `feedback_title` varchar(60) NOT NULL DEFAULT '',
            `requester` varchar(64) NOT NULL DEFAULT '',
            `assignee` varchar(64) NOT NULL DEFAULT '',
            `status` varchar(16) NOT NULL DEFAULT 'pending',
            `responded_at` datetime DEFAULT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_feedback_id` (`feedback_id`),
            KEY `idx_assignee` (`assignee`),
            KEY `idx_status` (`status`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
        "CREATE TABLE IF NOT EXISTS `feedback_admin_notifications` (
            `id` bigint(20) NOT NULL AUTO_INCREMENT,
            `feedback_id` bigint(20) NOT NULL DEFAULT 0,
            `to_admin` varchar(64) NOT NULL DEFAULT '',
            `from_admin` varchar(64) NOT NULL DEFAULT '',
            `type` varchar(32) NOT NULL DEFAULT '',
            `content` varchar(500) NOT NULL DEFAULT '',
            `read_at` datetime DEFAULT NULL,
            `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (`id`),
            KEY `idx_to_admin` (`to_admin`),
            KEY `idx_read_at` (`read_at`),
            KEY `idx_created_at` (`created_at`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
];
