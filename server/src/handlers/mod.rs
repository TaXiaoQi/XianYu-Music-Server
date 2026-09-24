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
pub mod system;
pub mod token;
pub mod upload;
pub mod wallpaper;
pub mod watch;

use axum::response::Response;
use sqlx::MySqlPool;

use crate::response::ReqCtx;

pub async fn dispatch(action: &str, body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    if let Some(resp) = token::check_dispatch_auth(action, body, &ctx, pool).await {
        return resp;
    }
    match action {
        // reporting
        "error" => reporting::error(body, ctx, pool).await,
        "report_user_behavior" => reporting::report_user_behavior(body, ctx, pool).await,
        "search" => reporting::search(body, ctx, pool).await,
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
        "get_fallback_modules" => system::get_fallback_modules(ctx).await,
        "get_announcement" => system::get_announcement(body, ctx, pool).await,
        "confirm_announcement" => system::confirm_announcement(body, ctx, pool).await,
        "get_privacy_policy" => system::get_privacy_policy(body, ctx, pool).await,
        "confirm_privacy_policy" => system::confirm_privacy_policy(body, ctx, pool).await,
        "get_about_config" => system::get_about_config(body, ctx).await,
        "get_site_logo" => system::get_site_logo(ctx, pool).await,
        "get_user_agreement" => system::get_user_agreement(ctx, pool).await,
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
        "get_listen_stats" => settings::get_listen_stats(body, ctx, pool).await,
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
        // playlist
        "delete_playlist" => playlist::delete_playlist(body, ctx, pool).await,
        // file sync
        "file_sync_upload_start" => sync::file_sync_upload_start(body, ctx).await,
        "file_sync_upload_chunk" => sync::file_sync_upload_chunk(body, ctx).await,
        "file_sync_upload_finish" => sync::file_sync_upload_finish(body, ctx).await,
        "file_sync_download" => sync::file_sync_download(body, ctx).await,
        "file_sync_delete_playlist" => sync::file_sync_delete_playlist(body, ctx).await,
        "plugin_sync_upload_one" => sync::plugin_sync_upload_one(body, ctx).await,
        "plugin_sync_download" => sync::plugin_sync_download(body, ctx).await,
        "plugin_sync_delete" => sync::plugin_sync_delete(body, ctx).await,
        "settings_sync_upload" => sync::settings_sync_upload(body, ctx).await,
        "settings_sync_download" => sync::settings_sync_download(body, ctx).await,
        "favorites_sync_upload" => sync::favorites_sync_upload(body, ctx).await,
        "favorites_sync_download" => sync::favorites_sync_download(body, ctx).await,
        "listen_stats_sync_upload" => sync::listen_stats_sync_upload(body, ctx).await,
        "listen_stats_sync_download" => sync::listen_stats_sync_download(body, ctx).await,
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
