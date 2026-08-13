use axum::response::Response;
use serde_json::json;
use sqlx::MySqlPool;
use sqlx::Row;

use super::{err, log_operation, ok, AdminCtx};

fn str_of<'a>(v: &'a serde_json::Value, key: &str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("")
}

fn int_of(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key)
        .and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0)
}

/// 获取当前管理员账户信息
pub async fn get_account_info(_body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let row = sqlx::query("SELECT id, username, email, avatar_url, role, status, created_at, updated_at FROM admin_users WHERE id = ?")
        .bind(ctx.id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(admin) = row else {
        return err(404, "管理员不存在");
    };
    let username: String = admin.get("username");
    let email: String = admin.get("email");
    let avatar_url: String = admin.get("avatar_url");
    let role: String = admin.get("role");
    let status: i32 = admin.get("status");
    let created_at: Option<chrono::NaiveDateTime> = admin.get("created_at");
    let updated_at: Option<chrono::NaiveDateTime> = admin.get("updated_at");
    let created_at_str = created_at
        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default();
    let updated_at_str = updated_at
        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default();

    let operation_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_operation_log WHERE admin_id = ?")
        .bind(ctx.id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let last_login: Option<(String, String)> = sqlx::query("SELECT ip, created_at FROM admin_login_log WHERE admin_id = ? ORDER BY id DESC LIMIT 1")
        .bind(ctx.id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|r| {
            let ip: String = r.get("ip");
            let t: Option<chrono::NaiveDateTime> = r.get("created_at");
            let t_str = t
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();
            (ip, t_str)
        });

    ok("ok", json!({
        "id": ctx.id,
        "username": username,
        "email": email,
        "avatar_url": avatar_url,
        "role": role,
        "status": status,
        "created_at": created_at_str,
        "updated_at": updated_at_str,
        "operation_count": operation_count,
        "last_login_ip": last_login.as_ref().map(|(ip, _)| ip.as_str()).unwrap_or("未知"),
        "last_login_time": last_login.as_ref().map(|(_, t)| t.as_str()).unwrap_or("未知"),
    }))
}

/// 管理员头像上传（JSON base64 模式）
/// 权限：普通管理员只能上传自己的头像；超级管理员可为任意管理员上传。
/// 入参：admin_id（目标管理员ID，默认自己）, image（data:image/xxx;base64,...）
pub async fn upload_admin_avatar(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let mut target_id = int_of(&data, "admin_id");
    if target_id <= 0 {
        target_id = ctx.id;
    }
    // 权限：非超管只能操作自己
    if ctx.role != "super_admin" && target_id != ctx.id {
        return err(403, "普通管理员只能上传自己的头像");
    }
    let image_b64 = str_of(&data, "image").to_string();
    if image_b64.is_empty() {
        return err(400, "请选择头像图片");
    }
    // 兼容 data URL 前缀
    let b64 = if let Some(idx) = image_b64.find(',') {
        if image_b64.starts_with("data:") {
            &image_b64[idx + 1..]
        } else {
            &image_b64
        }
    } else {
        &image_b64
    };
    if b64.len() > 4 * 1024 * 1024 {
        return err(400, "图片数据过大");
    }
    use base64::Engine;
    let bytes = match base64::engine::general_purpose::STANDARD.decode(b64.trim()) {
        Ok(b) => b,
        Err(_) => return err(400, "无效的图片数据"),
    };
    let valid_ext = image::guess_format(&bytes)
        .map(|f| {
            matches!(
                f,
                image::ImageFormat::Jpeg
                    | image::ImageFormat::Png
                    | image::ImageFormat::WebP
                    | image::ImageFormat::Gif
            )
        })
        .unwrap_or(false);
    if !valid_ext {
        return err(400, "只允许上传 JPG/PNG/WEBP/GIF 图片");
    }
    // 目标管理员必须存在
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM admin_users WHERE id = ?")
        .bind(target_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    if exists.is_none() {
        return err(404, "管理员不存在");
    }
    let dir = std::path::Path::new("uploads").join("admin_avatars");
    if std::fs::create_dir_all(&dir).is_err() {
        return err(500, "无法创建上传目录");
    }
    let target = dir.join(format!("{}.png", target_id));
    if !save_admin_avatar_png(&bytes, &target) {
        return err(500, "图片保存失败，请检查目录权限");
    }
    let relative = format!("/uploads/admin_avatars/{}.png", target_id);
    let full = if !ctx.base_url.is_empty() {
        format!("{}{}", ctx.base_url.trim_end_matches('/'), &relative)
    } else if !ctx.config.public_base_url.is_empty() {
        format!("{}{}", ctx.config.public_base_url.trim_end_matches('/'), &relative)
    } else {
        relative.clone()
    };
    let _ = sqlx::query("UPDATE admin_users SET avatar_url = ? WHERE id = ?")
        .bind(&full)
        .bind(target_id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "上传管理员头像", &format!("管理员ID:{}", target_id), &full).await;
    ok("头像已更新", json!({ "avatar_url": full }))
}

/// 保存管理员头像为 PNG（透明通道，最大边长 256）
fn save_admin_avatar_png(bytes: &[u8], target: &std::path::Path) -> bool {
    use image::GenericImageView;
    let img = match image::load_from_memory(bytes) {
        Ok(i) => i,
        Err(_) => return false,
    };
    let (w, h) = img.dimensions();
    let max = 256u32;
    let (nw, nh) = if w > max || h > max {
        let scale = (max as f64 / w.max(h) as f64).min(1.0);
        (((w as f64 * scale) as u32).max(1), ((h as f64 * scale) as u32).max(1))
    } else {
        (w, h)
    };
    let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    let file = match std::fs::File::create(target) {
        Ok(f) => f,
        Err(_) => return false,
    };
    resized
        .write_to(&mut std::io::BufWriter::new(file), image::ImageFormat::Png)
        .is_ok()
}

/// 修改用户名
/// 支持分级：普通管理员只能修改自己的用户名，超级管理员可修改任意管理员用户名。
/// 入参：new_username（必填）, admin_id（目标管理员ID，默认自己）
pub async fn change_username(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let new_username = str_of(&data, "new_username").trim().to_string();
    let admin_id = int_of(&data, "admin_id");
    if new_username.is_empty() {
        return err(400, "用户名不能为空");
    }
    // 目标管理员：默认自己
    let target_id = if admin_id > 0 { admin_id } else { ctx.id };
    // 权限分级：修改他人用户名仅超管可操作
    if target_id != ctx.id && ctx.role != "super_admin" {
        return err(403, "仅超级管理员可以修改其他管理员用户名");
    }
    let exists = sqlx::query("SELECT id FROM admin_users WHERE username = ? AND id != ?")
        .bind(&new_username)
        .bind(target_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if exists {
        return err(400, "用户名已存在");
    }
    let _ = sqlx::query("UPDATE admin_users SET username = ? WHERE id = ?")
        .bind(&new_username)
        .bind(target_id)
        .execute(pool)
        .await;
    let target_username: String = sqlx::query_scalar("SELECT username FROM admin_users WHERE id = ?")
        .bind(target_id)
        .fetch_one(pool)
        .await
        .unwrap_or_default();
    log_operation(pool, ctx, "修改用户名", &target_username, &format!("id={}", target_id)).await;
    ok("用户名修改成功", serde_json::Value::Null)
}

/// 修改用户邮箱
pub async fn change_user_email(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let user_id = int_of(&data, "user_id");
    let new_email = str_of(&data, "new_email").trim().to_string();
    if user_id <= 0 {
        return err(400, "用户参数错误");
    }
    if !new_email.is_empty() && !crate::admin::is_valid_email(&new_email) {
        return err(400, "邮箱格式不正确");
    }
    let user = sqlx::query("SELECT email, username FROM app_users WHERE id = ? LIMIT 1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return err(404, "用户不存在");
    };
    let old_email: String = user.get("email");
    let _ = sqlx::query("UPDATE app_users SET email = ? WHERE id = ?")
        .bind(&new_email)
        .bind(user_id)
        .execute(pool)
        .await;
    let detail = format!("user_id={} {} -> {}", user_id, old_email, new_email);
    log_operation(pool, ctx, "修改用户邮箱", &detail, "").await;
    ok("修改成功", serde_json::json!({ "role": "普通成员" }))
}

/// 重置用户听歌时长与新歌数
pub async fn reset_listen_duration(body: &str, ctx: &AdminCtx, pool: &MySqlPool) -> Response {
    let data: serde_json::Value = serde_json::from_str(body).unwrap_or(serde_json::Value::Null);
    let mut ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        ciyuanxi_id = str_of(&data, "user_id").trim().to_string();
    }
    if ciyuanxi_id.is_empty() {
        return err(400, "用户参数错误");
    }
    let _ = sqlx::query("UPDATE app_users SET listen_duration = 0, unique_songs_count = 0, listen_stats_reset_at = NOW() WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    // 清理每日统计表，确保日榜/周榜也从零开始
    let _ = sqlx::query("DELETE FROM listen_daily_stats WHERE ciyuanxi_id = ?")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await;
    log_operation(pool, ctx, "重置听歌时长", &format!("ciyuanxi_id={}", ciyuanxi_id), "").await;
    ok("重置成功", serde_json::Value::Null)
}
