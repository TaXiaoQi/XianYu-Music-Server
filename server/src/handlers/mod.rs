pub mod auth;
pub mod email_auth;
pub mod helpers;
pub mod playlist;
pub mod recommend;
pub mod reporting;
pub mod settings;
pub mod share;
pub mod social;
pub mod sync;
pub mod sync_diff;
pub mod sync_merge;
pub mod sync_store;
pub mod system;
pub mod theme;
pub mod theme_editor;
pub mod token;
pub mod upload;
pub mod wallpaper;
pub mod watch;

use axum::response::Response;
use sqlx::MySqlPool;

use crate::response::ReqCtx;

/// 东八区「今天」(DATE)，纯算术换算、与会话时区无关：
/// unix 秒 + 8h 后按 86400s 划日，以 '1970-01-01' 加天数还原日期，
/// 与事件入桶的 day_index = (ended_at + 8h) / 86400 完全同构。
/// 不能用 DATE(NOW() + INTERVAL 8 HOUR)：它依赖 MySQL 会话时区——
/// DB 时区为 +08:00 时 NOW() 已是北京墙钟，再 +8h 就落到「明天」，
/// 与纯算术写入的今天行错位一天（今日时长/日榜恒 0 的事故根因）。
pub const TODAY_CN: &str =
    "DATE_ADD('1970-01-01', INTERVAL FLOOR((UNIX_TIMESTAMP() + 28800) / 86400) DAY)";

pub async fn dispatch(action: &str, body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    if let Some(resp) = token::check_dispatch_auth(action, body, &ctx, pool).await {
        return resp;
    }
    match action {
        // reporting
        "error" => reporting::error(body, ctx, pool).await,
        "report_user_behavior" => reporting::report_user_behavior(body, ctx, pool).await,
        "search" => reporting::search(body, ctx, pool).await,
        "input_stats" => reporting::input_stats(body, ctx, pool).await,
        "get_hot_search" => reporting::get_hot_search(body, ctx, pool).await,
        "get_daily_recommend" => recommend::get_daily_recommend(body, ctx, pool).await,
        "report_daily_dislike" => recommend::report_daily_dislike(body, ctx, pool).await,
        "report_daily_like" => recommend::report_daily_like(body, ctx, pool).await,
        "open" => reporting::app_open(body, ctx, pool).await,
        "check" => reporting::check(ctx, pool).await,
        "install" => reporting::install(ctx, pool).await,
        // system
        "get_source_status" => system::get_source_status(ctx, pool).await,
        "get_version_status" => system::get_version_status(body, ctx, pool).await,
        "get_latest_version" => system::get_latest_version(body, ctx, pool).await,
        "check_beta_access" => system::check_beta_access(body, ctx, pool).await,
        "get_fallback_modules" => system::get_fallback_modules(ctx, pool).await,
        "get_announcement" => system::get_announcement(body, ctx, pool).await,
        "confirm_announcement" => system::confirm_announcement(body, ctx, pool).await,
        "get_privacy_policy" => system::get_privacy_policy(body, ctx, pool).await,
        "confirm_privacy_policy" => system::confirm_privacy_policy(body, ctx, pool).await,
        "get_about_config" => system::get_about_config(body, ctx, pool).await,
        "get_site_logo" => system::get_site_logo(ctx, pool).await,
        "get_user_agreement" => system::get_user_agreement(ctx, pool).await,
        "get_deploy_doc" => system::get_deploy_doc(ctx, pool).await,
        "get_server_load" => system::get_server_load(ctx, pool).await,
        "get_leaderboard" => system::get_leaderboard(body, ctx, pool).await,
        // auth
        "register" => auth::register(body, ctx, pool).await,
        "user_login" => auth::user_login(body, ctx, pool).await,
        "get_captcha" => auth::get_captcha(body, ctx, pool).await,
        "verify_captcha" => auth::verify_captcha(body, ctx, pool).await,
        "login_by_code" => auth::login_by_code(body, ctx, pool).await,
        "send_verify_code" => auth::send_verify_code(body, ctx, pool).await,
        "reset_password" => auth::reset_password(body, ctx, pool).await,
        "delete_account" => auth::delete_account(body, ctx, pool).await,
        "preverify_delete_account" => auth::preverify_delete_account(body, ctx, pool).await,
        "generate_tv_login_code" => auth::generate_tv_login_code(body, ctx, pool).await,
        "poll_tv_login_status" => auth::poll_tv_login_status(body, ctx, pool).await,
        "scan_tv_login" => auth::scan_tv_login(body, ctx, pool).await,
        "confirm_tv_login" => auth::confirm_tv_login(body, ctx, pool).await,
        "check_ban_status" => auth::check_ban_status(body, ctx, pool).await,
        // settings
        "get_user_info" => settings::get_user_info(body, ctx, pool).await,
        "get_user_settings" => settings::get_user_settings(body, ctx, pool).await,
        "update_user_settings" => settings::update_user_settings(body, ctx, pool).await,
        "update_profile" => settings::update_profile(body, ctx, pool).await,
        "check_username" => settings::check_username(body, ctx, pool).await,
        "change_password" => settings::change_password(body, ctx, pool).await,
        "update_ciyuanxi_id" => settings::update_ciyuanxi_id(body, ctx, pool).await,
        "bind_email" => settings::bind_email(body, ctx, pool).await,
        "get_avatar_status" => settings::get_avatar_status(body, ctx, pool).await,
        "get_nickname_status" => settings::get_nickname_status(body, ctx, pool).await,
        "report_listen_stats" => settings::report_listen_stats(body, ctx, pool).await,
        "report_listen_events" => settings::report_listen_events(body, ctx, pool).await,
        "get_listen_stats" => settings::get_listen_stats(body, ctx, pool).await,
        "get_listen_stats_summary" => settings::get_listen_stats_summary(body, ctx, pool).await,
        "deduct_master_quota" => settings::deduct_master_quota(body, ctx, pool).await,
        "get_master_quota_usage" => settings::get_master_quota_usage(body, ctx, pool).await,
        // share
        "create_share" => share::create_share(body, ctx, pool).await,
        "report_share_action" => share::report_share_action(body, ctx, pool).await,
        "share_download" => system::share_download(body, ctx, pool).await,
        // social
        "submit_feedback" => social::submit_feedback(body, ctx, pool).await,
        "submit_appeal" => social::submit_appeal(body, ctx, pool).await,
        "check_ciyuanxi_id" => social::check_ciyuanxi_id(body, ctx, pool).await,
        "get_my_feedback_notifications" => social::get_my_feedback_notifications(body, ctx, pool).await,
        "confirm_feedback_notification" => social::confirm_feedback_notification(body, ctx, pool).await,
        "get_nickname_change_notices" => social::get_nickname_change_notices(body, ctx, pool).await,
        "confirm_nickname_change_notice" => social::confirm_nickname_change_notice(body, ctx, pool).await,
        "list_my_feedback" => social::list_my_feedback(body, ctx, pool).await,
        // wallpaper
        "list_wallpapers" => wallpaper::list_wallpapers(body, ctx, pool).await,
        "my_wallpapers" => wallpaper::my_wallpapers(body, ctx, pool).await,
        "upload_wallpaper" => wallpaper::upload_wallpaper(body, ctx, pool).await,
        // theme（主题中心）
        "list_themes" => theme::list_themes(body, ctx, pool).await,
        "my_themes" => theme::my_themes(body, ctx, pool).await,
        "upload_theme" => theme::upload_theme(body, ctx, pool).await,
        // playlist
        "delete_playlist" => playlist::delete_playlist(body, ctx, pool).await,
        // file sync
        "file_sync_upload_start" => sync::file_sync_upload_start(body, ctx, pool).await,
        "file_sync_upload_chunk" => sync::file_sync_upload_chunk(body, ctx, pool).await,
        "file_sync_upload_finish" => sync::file_sync_upload_finish(body, ctx, pool).await,
        "file_sync_download" => sync::file_sync_download(body, ctx, pool).await,
        "file_sync_delete_playlist" => sync::file_sync_delete_playlist(body, ctx, pool).await,
        "file_sync_v2_upload_start" => sync::file_sync_v2_upload_start(body, ctx, pool).await,
        "file_sync_v2_upload_chunk" => sync::file_sync_v2_upload_chunk(body, ctx, pool).await,
        "file_sync_v2_upload_finish" => sync::file_sync_v2_upload_finish(body, ctx, pool).await,
        "file_sync_v2_download_ops" => sync::file_sync_v2_download_ops(body, ctx, pool).await,
        "plugin_sync_upload_one" => sync::plugin_sync_upload_one(body, ctx, pool).await,
        "plugin_sync_download" => sync::plugin_sync_download(body, ctx, pool).await,
        "plugin_sync_delete" => sync::plugin_sync_delete(body, ctx, pool).await,
        "settings_sync_upload" => sync::settings_sync_upload(body, ctx, pool).await,
        "settings_sync_download" => sync::settings_sync_download(body, ctx, pool).await,
        "favorites_sync_upload" => sync::favorites_sync_upload(body, ctx, pool).await,
        "favorites_sync_download" => sync::favorites_sync_download(body, ctx, pool).await,
        // upload
        "upload_avatar" => upload::upload_avatar(body, ctx, pool).await,
        "upload_cover" => upload::upload_cover(body, ctx, pool).await,
        // email auth (邮箱注册登录测试)
        "email_send_code" => email_auth::send_code(body, ctx, pool).await,
        "email_get_captcha_config" => email_auth::get_captcha_config(body, ctx, pool).await,
        "email_get_turnstile_config" => email_auth::get_turnstile_config(body, ctx, pool).await,
        "email_register" => email_auth::register(body, ctx, pool).await,
        "email_login" => email_auth::login(body, ctx, pool).await,
        "email_reset_password" => email_auth::reset_password(body, ctx, pool).await,
        "email_get_profile" => email_auth::get_profile(body, ctx, pool).await,
        // watch 联动（手表↔手机 命令中继 + 在线状态）
        "watch_submit_command" => watch::watch_submit_command(body, ctx, pool).await,
        "watch_poll_command" => watch::watch_poll_command(body, ctx, pool).await,
        "watch_phone_ping" => watch::watch_phone_ping(body, ctx, pool).await,
        "watch_phone_query" => watch::watch_phone_query(body, ctx, pool).await,
        _ => {
            let msg = format!("未知操作: {}", action);
            ctx.err(404, &msg)
        }
    }
}
