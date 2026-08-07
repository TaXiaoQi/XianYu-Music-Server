pub mod auth;
pub mod chat;
pub mod email_auth;
pub mod helpers;
pub mod playlist;
pub mod reporting;
pub mod settings;
pub mod social;
pub mod sync;
pub mod system;
pub mod upload;

use axum::response::Response;
use sqlx::MySqlPool;

use crate::response::ReqCtx;

/// 按 action 分发到对应 handler
pub async fn dispatch(action: &str, body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    match action {
        // reporting
        "source_call" => reporting::source_call(body, ctx, pool).await,
        "login" => reporting::login(body, ctx, pool).await,
        "error" => reporting::error(body, ctx, pool).await,
        "check" => reporting::check(ctx, pool).await,
        "install" => reporting::install(ctx, pool).await,
        // system
        "get_source_status" => system::get_source_status(ctx, pool).await,
        "get_version_status" => system::get_version_status(body, ctx, pool).await,
        "get_latest_version" => system::get_latest_version(ctx, pool).await,
        "get_server_load" => system::get_server_load(ctx, pool).await,
        "get_leaderboard" => system::get_leaderboard(body, ctx, pool).await,
        // auth
        "register" => auth::register(body, ctx, pool).await,
        "user_login" => auth::user_login(body, ctx, pool).await,
        "login_by_code" => auth::login_by_code(body, ctx, pool).await,
        "send_verify_code" => auth::send_verify_code(body, ctx, pool).await,
        "reset_password" => auth::reset_password(body, ctx, pool).await,
        "generate_tv_login_code" => auth::generate_tv_login_code(body, ctx, pool).await,
        "poll_tv_login_status" => auth::poll_tv_login_status(body, ctx, pool).await,
        "scan_tv_login" => auth::scan_tv_login(body, ctx, pool).await,
        "confirm_tv_login" => auth::confirm_tv_login(body, ctx, pool).await,
        // settings
        "get_user_info" => settings::get_user_info(body, ctx, pool).await,
        "get_user_settings" => settings::get_user_settings(body, ctx, pool).await,
        "update_user_settings" => settings::update_user_settings(body, ctx, pool).await,
        "update_profile" => settings::update_profile(body, ctx, pool).await,
        "check_username" => settings::check_username(body, ctx, pool).await,
        "change_password" => settings::change_password(body, ctx, pool).await,
        "get_avatar_status" => settings::get_avatar_status(body, ctx, pool).await,
        "get_nickname_status" => settings::get_nickname_status(body, ctx, pool).await,
        "report_listen_stats" => settings::report_listen_stats(body, ctx, pool).await,
        "deduct_master_quota" => settings::deduct_master_quota(body, ctx, pool).await,
        "get_master_quota_usage" => settings::get_master_quota_usage(body, ctx, pool).await,
        // social
        "submit_feedback" => social::submit_feedback(body, ctx, pool).await,
        "check_ciyuanxi_id" => social::check_ciyuanxi_id(body, ctx, pool).await,
        "create_ciyuanxi_id" => social::create_ciyuanxi_id(body, ctx, pool).await,
        // playlist
        "create_playlist" => playlist::create_playlist(body, ctx, pool).await,
        "get_playlists" => playlist::get_playlists(body, ctx, pool).await,
        "get_or_create_favorite_playlist" => playlist::get_or_create_favorite_playlist(body, ctx, pool).await,
        "check_song_in_playlist" => playlist::check_song_in_playlist(body, ctx, pool).await,
        "get_playlist_detail" => playlist::get_playlist_detail(body, ctx, pool).await,
        "update_playlist" => playlist::update_playlist(body, ctx, pool).await,
        "delete_playlist" => playlist::delete_playlist(body, ctx, pool).await,
        "add_song_to_playlist" => playlist::add_song_to_playlist(body, ctx, pool).await,
        "batch_add_songs_to_playlist" => playlist::batch_add_songs(body, ctx, pool).await,
        "batch_add_songs_to_playlist_large" => playlist::batch_add_songs_large(body, ctx, pool).await,
        "remove_song_from_playlist" => playlist::remove_song_from_playlist(body, ctx, pool).await,
        "upload_playlist_cover" => upload::upload_playlist_cover(body, ctx, pool).await,
        // chat
        "chat_get_messages" => chat::chat_get_messages(body, ctx, pool).await,
        "chat_send_message" => chat::chat_send_message(body, ctx, pool).await,
        "chat_get_channel_info" => chat::chat_get_channel_info(body, ctx, pool).await,
        "chat_update_channel_name" => chat::chat_update_channel_name(body, ctx, pool).await,
        "chat_get_members" => chat::chat_get_members(body, ctx, pool).await,
        "chat_get_remark" => chat::chat_get_remark(body, ctx, pool).await,
        "chat_update_remark" => chat::chat_update_remark(body, ctx, pool).await,
        "chat_get_channel_remark" => chat::chat_get_channel_remark(body, ctx, pool).await,
        "chat_update_channel_remark" => chat::chat_update_channel_remark(body, ctx, pool).await,
        "chat_recall_message" => chat::chat_recall_message(body, ctx, pool).await,
        // file sync
        "file_sync_upload_start" => sync::file_sync_upload_start(body, ctx).await,
        "file_sync_upload_chunk" => sync::file_sync_upload_chunk(body, ctx).await,
        "file_sync_upload_finish" => sync::file_sync_upload_finish(body, ctx).await,
        "file_sync_download" => sync::file_sync_download(body, ctx).await,
        "file_sync_status" => sync::file_sync_status(body, ctx).await,
        "plugin_sync_upload" => sync::plugin_sync_upload(body, ctx).await,
        "plugin_sync_upload_one" => sync::plugin_sync_upload_one(body, ctx).await,
        "plugin_sync_download" => sync::plugin_sync_download(body, ctx).await,
        "plugin_sync_status" => sync::plugin_sync_status(body, ctx).await,
        "settings_sync_upload" => sync::settings_sync_upload(body, ctx).await,
        "settings_sync_download" => sync::settings_sync_download(body, ctx).await,
        "settings_sync_status" => sync::settings_sync_status(body, ctx).await,
        // upload
        "upload_avatar" => upload::upload_avatar(body, ctx, pool).await,
        "upload_background" => upload::upload_background(body, ctx, pool).await,
        // email auth (邮箱注册登录测试)
        "email_send_code" => email_auth::send_code(body, ctx, pool).await,
        "email_register" => email_auth::register(body, ctx, pool).await,
        "email_login" => email_auth::login(body, ctx, pool).await,
        "email_reset_password" => email_auth::reset_password(body, ctx, pool).await,
        "email_get_profile" => email_auth::get_profile(body, ctx, pool).await,
        _ => {
            let msg = format!("未知操作: {}", action);
            ctx.err(404, &msg)
        }
    }
}
