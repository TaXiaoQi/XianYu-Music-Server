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
    let valid_ext = name.ends_with(".sql") || name.ends_with(".sql.gz");
    name.len() > 8
        && name.len() <= MAX_FILENAME
        && name.starts_with("backup_")
        && valid_ext
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

// ===== 数据大类：业务视角分组与一键清空（不暴露真实表名，表名全部在服务端白名单内） =====

/// 统计若干白名单表的行数总和
async fn sum_counts(pool: &MySqlPool, tables: &[&str]) -> i64 {
    let mut total = 0i64;
    for t in tables {
        total += sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM `{}`", t))
            .fetch_one(pool)
            .await
            .unwrap_or(0);
    }
    total
}

/// 删除白名单表全部数据，返回受影响行数
async fn del_table(pool: &MySqlPool, table: &str) -> u64 {
    sqlx::query(&format!("DELETE FROM `{}`", table))
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
        .unwrap_or(0)
}

/// 清空目录下所有文件（不递归子目录），返回删除数
fn remove_dir_files(dir: std::path::PathBuf) -> usize {
    let mut n = 0;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && std::fs::remove_file(&p).is_ok() {
                n += 1;
            }
        }
    }
    n
}

/// 统计某表按条件的行数
async fn count_where(pool: &MySqlPool, table: &str, cond: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM `{}` WHERE {}", table, cond))
        .fetch_one(pool)
        .await
        .unwrap_or(0)
}

pub async fn list_data_groups(_body: &str, _ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let mut groups: Vec<Value> = Vec::new();
    let mut push = |category: &str, key: &str, name: &str, desc: &str, count: i64| {
        groups.push(json!({ "category": category, "key": key, "name": name, "desc": desc, "count": count }));
    };
    // ---- 用户数据 ----
    // 歌单快照已迁入 user_sync_files（playlists.json），按 JSON 展开统计歌单总数；
    // JSON 统计失败（内容异常等）时退回快照份数
    let pl_count = match sqlx::query_scalar::<_, i64>(
        "SELECT CAST(COALESCE(SUM(JSON_LENGTH(CAST(content AS JSON), '$.playlists')), 0) AS SIGNED) FROM user_sync_files WHERE file_name = 'playlists.json'",
    )
    .fetch_one(pool)
    .await
    {
        Ok(v) => v,
        Err(_) => count_where(pool, "user_sync_files", "file_name = 'playlists.json'").await,
    };
    push("用户数据", "playlists", "用户歌单", "云端保存的歌单与歌曲", pl_count);
    push(
        "用户数据",
        "plugins",
        "插件快照",
        "客户端上传的插件脚本备份",
        count_where(pool, "user_sync_files", "file_name = 'plugins.json'").await,
    );
    push(
        "用户数据",
        "favorites",
        "收藏快照",
        "客户端同步的收藏歌曲备份",
        count_where(pool, "user_sync_files", "file_name = 'favorites.json'").await,
    );
    push(
        "用户数据",
        "sync_snapshots",
        "同步快照备份",
        "歌单/插件/收藏/设置等全部同步备份与分块数据",
        sum_counts(pool, &["user_sync_files", "user_sync_chunks"]).await,
    );
    let avatar_users: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM app_users WHERE avatar_url IS NOT NULL AND avatar_url != ''")
            .fetch_one(pool)
            .await
            .unwrap_or(0);
    let avatar_pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_avatar_pending")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    push("用户数据", "avatars", "用户头像", "用户上传的头像文件与待审记录", avatar_users + avatar_pending);
    push(
        "用户数据",
        "tokens",
        "登录令牌",
        "客户端登录态，清空后所有用户需重新登录",
        sum_counts(pool, &["user_tokens"]).await,
    );
    // ---- 内容与审核 ----
    push(
        "内容与审核",
        "nicknames",
        "昵称审核",
        "待审核的改名申请与改名通知记录",
        sum_counts(pool, &["user_nickname_pending", "nickname_change_notices"]).await,
    );
    push(
        "内容与审核",
        "feedback",
        "用户反馈",
        "意见反馈、协作请求与随附截图",
        sum_counts(pool, &["user_feedback", "feedback_collab_requests", "feedback_admin_notifications"]).await,
    );
    push("内容与审核", "beta_testers", "内测申请", "内测资格申请记录", sum_counts(pool, &["beta_testers"]).await);
    push("内容与审核", "shares", "分享记录", "歌曲分享链接与浏览统计", sum_counts(pool, &["share_log", "share_views", "share_actions"]).await);
    push(
        "内容与审核",
        "announce_confirms",
        "公告已读记录",
        "用户对公告的已读确认，清空后公告会重新弹出",
        sum_counts(pool, &["user_announcement_confirmations"]).await,
    );
    // ---- 统计与日志 ----
    let stat_users: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM app_users WHERE listen_duration != 0 OR unique_songs_count != 0 OR listen_duration_offset != 0 OR unique_songs_offset != 0",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    push(
        "统计与日志",
        "listen_stats",
        "听歌统计",
        "累计听歌时长与每日听歌统计，清空后归零重新累计",
        stat_users + sum_counts(pool, &["listen_daily_stats"]).await,
    );
    push(
        "统计与日志",
        "play_history",
        "播放历史",
        "播放记录与每日推荐喜欢/不喜欢反馈",
        sum_counts(pool, &["play_history", "daily_like", "daily_dislike"]).await,
    );
    push("统计与日志", "errors", "客户端错误日志", "客户端上报的运行错误记录", sum_counts(pool, &["error_log"]).await);
    push(
        "统计与日志",
        "login_logs",
        "登录日志",
        "客户端与后台的登录流水",
        sum_counts(pool, &["login_log", "admin_login_log", "admin_app_login_log"]).await,
    );
    push("统计与日志", "admin_ops", "后台操作日志", "后台管理的操作审计记录", sum_counts(pool, &["admin_operation_log"]).await);
    push("统计与日志", "call_stats", "曲源调用统计", "各音乐来源的调用流水记录", sum_counts(pool, &["source_call_log"]).await);
    push(
        "统计与日志",
        "access_logs",
        "访问与打开日志",
        "App 打开记录与主页访问流水",
        sum_counts(pool, &["app_open_log", "view_access_log"]).await,
    );
    push("统计与日志", "quota_logs", "配额使用记录", "大师配额扣减流水", sum_counts(pool, &["master_quota_usage_log"]).await);
    push(
        "统计与日志",
        "email_logs",
        "邮件与验证码",
        "邮件发送记录与邮箱验证码",
        sum_counts(pool, &["email_send_log", "email_verify_codes", "email_test_logs", "email_test_codes"]).await,
    );
    // ---- 运行状态 ----
    push(
        "运行状态",
        "device_status",
        "设备状态与指令",
        "设备在线状态与远程控制指令队列",
        sum_counts(pool, &["device_presence", "watch_commands"]).await,
    );
    push(
        "运行状态",
        "temp_data",
        "验证与限流临时数据",
        "人机验证、限流计数与 TV 授权码",
        sum_counts(pool, &["human_captcha_challenges", "auth_rate_limits", "api_rate_events", "api_temp_blocks", "tv_login_codes"]).await,
    );
    ok("ok", json!({ "groups": groups }))
}

/// 数据类别二级页的子表白名单（展示名, 真实表名），与 clear_data_group 的清空范围一致。
/// 歌单/插件/收藏三类存于 user_sync_files 的 JSON 快照，走 sync_group_detail 特殊通道。
fn group_parts(key: &str) -> Option<Vec<(&'static str, &'static str)>> {
    match key {
        "sync_snapshots" => Some(vec![("同步快照文件", "user_sync_files"), ("上传分块", "user_sync_chunks")]),
        "avatars" => Some(vec![("头像待审记录", "user_avatar_pending")]),
        "nicknames" => Some(vec![("改名待审", "user_nickname_pending"), ("改名通知", "nickname_change_notices")]),
        "listen_stats" => Some(vec![("每日听歌统计", "listen_daily_stats")]),
        "play_history" => Some(vec![("播放记录", "play_history"), ("每日喜欢", "daily_like"), ("每日不喜欢", "daily_dislike")]),
        "feedback" => Some(vec![("意见反馈", "user_feedback"), ("协作请求", "feedback_collab_requests"), ("管理员通知", "feedback_admin_notifications")]),
        "beta_testers" => Some(vec![("内测申请", "beta_testers")]),
        "tokens" => Some(vec![("登录令牌", "user_tokens")]),
        "announce_confirms" => Some(vec![("公告已读记录", "user_announcement_confirmations")]),
        "errors" => Some(vec![("客户端错误日志", "error_log")]),
        "login_logs" => Some(vec![("客户端登录", "login_log"), ("后台登录", "admin_login_log"), ("App 后台登录", "admin_app_login_log")]),
        "admin_ops" => Some(vec![("后台操作审计", "admin_operation_log")]),
        "call_stats" => Some(vec![("曲源调用流水", "source_call_log")]),
        "access_logs" => Some(vec![("App 打开记录", "app_open_log"), ("主页访问流水", "view_access_log")]),
        "quota_logs" => Some(vec![("大师配额扣减", "master_quota_usage_log")]),
        "shares" => Some(vec![("分享链接", "share_log"), ("分享浏览", "share_views"), ("分享操作", "share_actions")]),
        "email_logs" => Some(vec![("邮件发送", "email_send_log"), ("邮箱验证码", "email_verify_codes"), ("测试发送", "email_test_logs"), ("测试验证码", "email_test_codes")]),
        "device_status" => Some(vec![("设备在线状态", "device_presence"), ("远程指令队列", "watch_commands")]),
        "temp_data" => Some(vec![("人机验证", "human_captcha_challenges"), ("限流计数", "auth_rate_limits"), ("API 限流事件", "api_rate_events"), ("临时封禁", "api_temp_blocks"), ("TV 授权码", "tv_login_codes")]),
        _ => None,
    }
}

/// 快照类别的分页明细：歌单展开为「弦予号 × 歌单」行；插件/收藏每行一个用户快照
async fn sync_group_detail(pool: &MySqlPool, key: &str, page_num: i64, page_size: i64) -> Response {
    let offset = (page_num - 1) * page_size;
    if key == "playlists" {
        let total: i64 = match sqlx::query_scalar(
            "SELECT CAST(COALESCE(SUM(JSON_LENGTH(CAST(content AS JSON), '$.playlists')), 0) AS SIGNED) FROM user_sync_files WHERE file_name = 'playlists.json'",
        )
        .fetch_one(pool)
        .await
        {
            Ok(v) => v,
            Err(_) => count_where(pool, "user_sync_files", "file_name = 'playlists.json'").await,
        };
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT ciyuanxi_id, content FROM user_sync_files WHERE file_name = 'playlists.json' ORDER BY ciyuanxi_id LIMIT ? OFFSET ?",
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        let mut rows_v: Vec<Value> = Vec::new();
        for (uid, content) in rows {
            let v: Value = serde_json::from_str(&content).unwrap_or(Value::Null);
            if let Some(list) = v.get("playlists").and_then(|x| x.as_array()) {
                for p in list {
                    rows_v.push(json!({
                        "弦予号": uid,
                        "歌单名": p.get("name").and_then(|x| x.as_str()).unwrap_or("(未命名)"),
                        "类型": p.get("type").and_then(|x| x.as_str()).unwrap_or("-"),
                        "歌曲数": p.get("songs").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0),
                        "创建时间": p.get("createdAt").cloned().unwrap_or(Value::Null),
                    }));
                }
            }
        }
        return ok("", json!({
            "mode": "playlists",
            "columns": ["弦予号", "歌单名", "类型", "歌曲数", "创建时间"],
            "rows": rows_v,
            "total": total,
            "page": page_num,
            "pageSize": page_size,
        }));
    }
    // 插件/收藏：每行一个用户的快照文件
    let file = if key == "plugins" { "plugins.json" } else { "favorites.json" };
    let total = count_where(pool, "user_sync_files", &format!("file_name = '{file}'")).await;
    let rows = sqlx::query(&format!(
        "SELECT ciyuanxi_id, file_name, content_size, updated_at FROM user_sync_files WHERE file_name = '{file}' ORDER BY ciyuanxi_id LIMIT {page_size} OFFSET {offset}"
    ))
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    let rows: Vec<Value> = rows.iter().map(crate::admin::row_to_value).collect();
    ok("", json!({
        "mode": "sync",
        "columns": ["弦予号", "文件", "大小(字节)", "更新时间"],
        "rows": rows,
        "total": total,
        "page": page_num,
        "pageSize": page_size,
    }))
}

/// 数据类别二级页：分页查看某类别下的具体数据（表名全部来自服务端白名单，不接收前端表名）
pub async fn data_group_detail(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let key = str_of(&data, "key").trim().to_string();
    let part = int_of(&data, "part");
    let page = int_of(&data, "page");
    let page_num = if page < 1 { 1 } else { page };
    let page_size: i64 = 50;
    if key == "playlists" || key == "plugins" || key == "favorites" {
        return sync_group_detail(pool, &key, page_num, page_size).await;
    }
    let parts = match group_parts(&key) {
        Some(p) => p,
        None => return err(400, "未知的数据类别"),
    };
    let idx = if part < 0 || part as usize >= parts.len() { 0usize } else { part as usize };
    let (part_name, table) = parts[idx];
    let cols: Vec<String> = sqlx::query_scalar(&format!("SHOW COLUMNS FROM `{table}`"))
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    let total: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM `{table}`"))
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    let offset = (page_num - 1) * page_size;
    let rows = sqlx::query(&format!("SELECT * FROM `{table}` ORDER BY 1 LIMIT {page_size} OFFSET {offset}"))
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    // 截断超长文本（如快照 content 字段），避免明细页传输与渲染过大
    let mut rows: Vec<Value> = rows.iter().map(crate::admin::row_to_value).collect();
    for r in rows.iter_mut() {
        if let Some(obj) = r.as_object_mut() {
            for (_, v) in obj.iter_mut() {
                if let Some(s) = v.as_str() {
                    if s.len() > 500 {
                        let t: String = s.chars().take(500).collect();
                        *v = json!(format!("{t}…"));
                    }
                }
            }
        }
    }
    log_operation(pool, ctx, "查看数据类别", &format!("类别:{} 子项:{}", key, part_name), "").await;
    let part_names: Vec<&str> = parts.iter().map(|(n, _)| *n).collect();
    ok("", json!({
        "mode": "table",
        "columns": cols,
        "rows": rows,
        "total": total,
        "page": page_num,
        "pageSize": page_size,
        "partName": part_name,
        "parts": part_names,
    }))
}

pub async fn clear_data_group(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let key = str_of(&data, "key").trim().to_string();
    let mut cleared: u64 = 0;
    let mut files_removed: usize = 0;
    match key.as_str() {
        "playlists" => {
            cleared = del_table(pool, "user_playlists").await + del_table(pool, "user_playlist_songs").await;
        }
        "plugins" => {
            cleared = sqlx::query("DELETE FROM user_sync_files WHERE file_name = 'plugins.json'")
                .execute(pool)
                .await
                .map(|r| r.rows_affected())
                .unwrap_or(0);
        }
        "favorites" => {
            cleared = sqlx::query("DELETE FROM user_sync_files WHERE file_name = 'favorites.json'")
                .execute(pool)
                .await
                .map(|r| r.rows_affected())
                .unwrap_or(0);
        }
        "sync_snapshots" => {
            cleared = del_table(pool, "user_sync_files").await + del_table(pool, "user_sync_chunks").await;
        }
        "avatars" => {
            cleared = sqlx::query("UPDATE app_users SET avatar_url = '' WHERE avatar_url IS NOT NULL AND avatar_url != ''")
                .execute(pool)
                .await
                .map(|r| r.rows_affected())
                .unwrap_or(0);
            cleared += del_table(pool, "user_avatar_pending").await;
            files_removed = remove_dir_files(std::path::Path::new("uploads").join("avatars"));
        }
        "nicknames" => {
            cleared = del_table(pool, "user_nickname_pending").await + del_table(pool, "nickname_change_notices").await;
        }
        "listen_stats" => {
            cleared = sqlx::query(
                "UPDATE app_users SET listen_duration = 0, unique_songs_count = 0, listen_duration_offset = 0, unique_songs_offset = 0 WHERE listen_duration != 0 OR unique_songs_count != 0 OR listen_duration_offset != 0 OR unique_songs_offset != 0",
            )
            .execute(pool)
            .await
            .map(|r| r.rows_affected())
            .unwrap_or(0);
            cleared += del_table(pool, "listen_daily_stats").await;
        }
        "play_history" => {
            cleared = del_table(pool, "play_history").await + del_table(pool, "daily_like").await + del_table(pool, "daily_dislike").await;
        }
        "feedback" => {
            cleared = del_table(pool, "user_feedback").await
                + del_table(pool, "feedback_collab_requests").await
                + del_table(pool, "feedback_admin_notifications").await;
            files_removed = remove_dir_files(std::path::Path::new("uploads").join("feedback"));
        }
        "beta_testers" => cleared = del_table(pool, "beta_testers").await,
        "tokens" => cleared = del_table(pool, "user_tokens").await,
        "announce_confirms" => cleared = del_table(pool, "user_announcement_confirmations").await,
        "errors" => cleared = del_table(pool, "error_log").await,
        "login_logs" => {
            cleared = del_table(pool, "login_log").await + del_table(pool, "admin_login_log").await + del_table(pool, "admin_app_login_log").await;
        }
        "admin_ops" => cleared = del_table(pool, "admin_operation_log").await,
        "call_stats" => cleared = del_table(pool, "source_call_log").await,
        "access_logs" => {
            cleared = del_table(pool, "app_open_log").await + del_table(pool, "view_access_log").await;
        }
        "quota_logs" => cleared = del_table(pool, "master_quota_usage_log").await,
        "shares" => {
            cleared = del_table(pool, "share_log").await + del_table(pool, "share_views").await + del_table(pool, "share_actions").await;
        }
        "email_logs" => {
            for t in ["email_send_log", "email_verify_codes", "email_test_logs", "email_test_codes"] {
                cleared += del_table(pool, t).await;
            }
        }
        "device_status" => {
            cleared = del_table(pool, "device_presence").await + del_table(pool, "watch_commands").await;
        }
        "temp_data" => {
            for t in ["human_captcha_challenges", "auth_rate_limits", "api_rate_events", "api_temp_blocks", "tv_login_codes"] {
                cleared += del_table(pool, t).await;
            }
        }
        _ => return err(400, "未知的数据类别"),
    }
    let detail = if files_removed > 0 {
        format!("清除 {} 条记录、{} 个文件", cleared, files_removed)
    } else {
        format!("清除 {} 条记录", cleared)
    };
    log_operation(pool, ctx, "清空数据", &format!("类别:{}", key), &detail).await;
    ok("清空完成", json!({ "cleared": cleared, "files_removed": files_removed }))
}

pub async fn list_backups(_body: &str, _ctx: &AdminCtx, _pool: &MySqlPool) -> Response {
    let dir = backup_dir();
    let mut backups: Vec<Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("backup_") && (name.ends_with(".sql") || name.ends_with(".sql.gz")) {
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

fn write_sql<W: std::io::Write>(file: &mut W, s: &str) -> Result<(), String> {
    file.write_all(s.as_bytes())
        .map_err(|e| format!("写入备份文件失败: {}", e))
}

pub async fn perform_backup(pool: &MySqlPool, mode: &str) -> Result<BackupOutcome, String> {
    let dir = backup_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("无法创建备份目录: {}", e))?;
    let now = chrono::Local::now();
    // 备份文件 gzip 压缩落盘，节省磁盘体积；旧的 .sql 文件仍可读取/恢复
    let filename = format!("backup_{}.sql.gz", now.format("%Y%m%d_%H%M%S"));
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

    let raw = std::fs::File::create(&filepath).map_err(|e| format!("创建备份文件失败: {}", e))?;
    let mut file = flate2::write::GzEncoder::new(raw, flate2::Compression::default());
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
    // 显式收尾 gzip 流，确保压缩数据完整落盘
    let raw = file.finish().map_err(|e| format!("压缩备份文件失败: {}", e))?;
    drop(raw);

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

/// 日志自动保留：每日清理过期流水日志，控制磁盘与表体积（业务数据不清理）
pub async fn log_retention_loop(pool: &MySqlPool) {
    // (表名, 时间列, 保留天数)
    const RULES: &[(&str, &str, i64)] = &[
        ("error_log", "error_time", 7),
        ("source_call_log", "call_time", 30),
        ("app_open_log", "created_at", 30),
        ("input_stats_log", "created_at", 30),
        ("view_access_log", "created_at", 30),
        ("login_log", "login_time", 90),
        ("admin_login_log", "created_at", 90),
        ("admin_app_login_log", "created_at", 90),
        ("email_send_log", "created_at", 90),
    ];
    loop {
        let mut total: u64 = 0;
        for (table, col, days) in RULES {
            let sql = format!(
                "DELETE FROM `{}` WHERE `{}` < NOW() - INTERVAL {} DAY",
                table, col, days
            );
            match sqlx::query(&sql).execute(pool).await {
                Ok(r) => total += r.rows_affected(),
                Err(e) => tracing::warn!("log_retention: 清理 {} 失败: {}", table, e),
            }
        }
        if total > 0 {
            tracing::info!("log_retention: 已清理过期日志 {} 行", total);
        }
        tokio::time::sleep(std::time::Duration::from_secs(24 * 3600)).await;
    }
}

/// 一次性把历史 .sql 备份压缩为 .sql.gz（服务启动时调用）
pub fn migrate_backups_to_gzip() {
    let dir = backup_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    let mut count = 0usize;
    let mut saved_total: u64 = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("backup_") || !name.ends_with(".sql") {
            continue;
        }
        let Ok(raw) = std::fs::read(&path) else { continue };
        let gz_name = format!("{}.gz", name);
        let gz_path = dir.join(&gz_name);
        if gz_path.exists() {
            continue;
        }
        let Ok(out) = std::fs::File::create(&gz_path) else { continue };
        let mut enc = flate2::write::GzEncoder::new(out, flate2::Compression::default());
        let write_ok = std::io::Write::write_all(&mut enc, &raw).is_ok();
        let finish_ok = enc.finish().is_ok();
        let gz_size = std::fs::metadata(&gz_path).map(|m| m.len()).unwrap_or(0);
        if !(write_ok && finish_ok) || gz_size == 0 {
            let _ = std::fs::remove_file(&gz_path);
            continue;
        }
        if std::fs::remove_file(&path).is_ok() {
            count += 1;
            saved_total += raw.len() as u64 - gz_size;
        }
    }
    if count > 0 {
        tracing::info!("backup_gzip_migration: 已压缩 {} 个历史备份，节省 {} 字节", count, saved_total);
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

/// 读取备份文件文本内容，.sql.gz 自动解压
fn read_backup_text(filepath: &std::path::Path) -> Result<String, String> {
    let raw = std::fs::read(filepath).map_err(|_| "备份文件不存在".to_string())?;
    if filepath.extension().map(|e| e == "gz").unwrap_or(false) {
        use std::io::Read;
        let mut s = String::new();
        flate2::read::GzDecoder::new(&raw[..])
            .read_to_string(&mut s)
            .map_err(|e| format!("解压备份文件失败: {}", e))?;
        Ok(s)
    } else {
        String::from_utf8(raw).map_err(|_| "备份文件编码异常".to_string())
    }
}

pub async fn view_backup(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let filename = str_of(&data, "filename").trim().to_string();
    if !sanitize_filename(&filename) {
        return err(400, "无效的文件名");
    }
    let filepath = backup_dir().join(&filename);
    match read_backup_text(&filepath) {
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
    let content = match read_backup_text(&filepath) {
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
    let mut resp = (axum::http::StatusCode::OK, axum::body::Body::from(content)).into_response();
    if let Ok(hv) = axum::http::HeaderValue::from_str(&cd) {
        resp.headers_mut().insert(axum::http::header::CONTENT_DISPOSITION, hv);
    }
    resp
}