use axum::response::Response;
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::response::ReqCtx;

pub async fn delete_playlist(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    let playlist_id = int_of(&data, "playlist_id");
    if ciyuanxi_id.is_empty() || playlist_id <= 0 {
        return ctx.err(400, "参数错误");
    }

    let row = sqlx::query("SELECT id, cover_path FROM user_playlists WHERE id = ? AND user_id = ?")
        .bind(playlist_id)
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(row) = row else {
        return ctx.err(404, "歌单不存在");
    };

    let cover_path: String = row.try_get("cover_path").unwrap_or_default();
    if !cover_path.is_empty() {
        // 只取文件名段再拼接：防遗留脏数据含路径分隔符导致任意文件删除
        if let Some(name) = std::path::Path::new(&cover_path).file_name() {
            let abs = std::path::Path::new("uploads/playlists").join(name);
            let _ = std::fs::remove_file(abs);
        }
    }

    let _ = sqlx::query("DELETE FROM user_playlist_songs WHERE playlist_id = ?")
        .bind(playlist_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM user_playlists WHERE id = ? AND user_id = ?")
        .bind(playlist_id)
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    ctx.ok_empty("删除成功")
}
