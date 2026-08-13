use axum::response::Response;
use serde_json::{json, Value};
use sqlx::MySqlPool;
use sqlx::Row;

use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::response::ReqCtx;

const MAX_FEEDBACK_LOG_CHARS: usize = 500_000;
const DEFAULT_FEEDBACK_DAILY_LIMIT: i64 = 20;
const MAX_FEEDBACK_IMAGES: usize = 6;
const MAX_FEEDBACK_IMAGE_BYTES: usize = 8 * 1024 * 1024;

fn feedback_img_dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").join("feedback")
}

fn data_url_to_bytes(data_url: &str) -> Option<Vec<u8>> {
    let raw = data_url.split_once(',').map(|(_, v)| v).unwrap_or(data_url);
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(raw).ok()
}

/// 将图片数据压缩为 JPEG 保存到 uploads/feedback/，返回相对 URL。
fn compress_and_save_feedback_image(bytes: &[u8], name: &str, max_w: u32, quality: u32) -> Option<String> {
    use image::GenericImageView;
    let img = image::load_from_memory(bytes).ok()?;
    let (w, h) = img.dimensions();
    let (nw, nh) = if w > max_w {
        (max_w, (h * max_w / w).max(1))
    } else {
        (w, h)
    };
    let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgb8();
    let dir = feedback_img_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return None;
    }
    let path = dir.join(name);
    let file = std::fs::File::create(&path).ok()?;
    if image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), quality as u8)
        .encode(&rgb, nw, nh, image::ExtendedColorType::Rgb8)
        .is_err()
    {
        return None;
    }
    Some(format!("/uploads/feedback/{}", name))
}

fn public_feedback_img_url(ctx: &ReqCtx, url: String) -> String {
    if url.starts_with("http://") || url.starts_with("https://") || url.is_empty() {
        return url;
    }
    let base = if !ctx.base_url.is_empty() {
        &ctx.base_url
    } else if !ctx.config.public_base_url.is_empty() {
        &ctx.config.public_base_url
    } else {
        return url;
    };
    format!("{}{}", base.trim_end_matches('/'), url)
}

fn trim_feedback_log(value: String) -> String {
    if value.chars().count() <= MAX_FEEDBACK_LOG_CHARS {
        return value;
    }
    value.chars().take(MAX_FEEDBACK_LOG_CHARS).collect()
}

async fn get_feedback_daily_limit(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT setting_value FROM server_settings WHERE setting_key = 'feedback_daily_limit' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .and_then(|v| v.trim().parse::<i64>().ok())
    .filter(|v| *v >= 0)
    .unwrap_or(DEFAULT_FEEDBACK_DAILY_LIMIT)
}

pub async fn submit_feedback(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let mut nickname = str_of(&data, "nickname").trim().to_string();
    let mut title = str_of(&data, "title").trim().to_string();
    let mut feedback_type = str_of(&data, "feedback_type").trim().to_string();
    if feedback_type.is_empty() {
        feedback_type = "problem".to_string();
    }
    if feedback_type != "problem" && feedback_type != "suggestion" {
        feedback_type = "problem".to_string();
    }
    // 标题为空时按反馈类型赋予默认标题
    if title.is_empty() {
        title = if feedback_type == "suggestion" {
            "功能建议".to_string()
        } else {
            "问题反馈".to_string()
        };
    }
    let content = str_of(&data, "content").trim().to_string();
    let raw_error_logs = str_of(&data, "error_logs");
    let raw_all_logs = str_of(&data, "all_logs");
    let error_logs = trim_feedback_log(raw_error_logs.clone());
    let all_logs = trim_feedback_log(raw_all_logs.clone());
    let log_meta = json!({
        "has_error_logs": !error_logs.is_empty(),
        "has_all_logs": !all_logs.is_empty(),
        "error_logs_chars": error_logs.chars().count(),
        "all_logs_chars": all_logs.chars().count(),
        "error_logs_truncated": raw_error_logs.chars().count() > error_logs.chars().count(),
        "all_logs_truncated": raw_all_logs.chars().count() > all_logs.chars().count(),
    })
    .to_string();
    // 图片：功能建议支持上传图片，接收 base64 data URL 数组，压缩保存到 uploads/feedback/
    let raw_images = data.get("images").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if feedback_type == "problem" && !raw_images.is_empty() {
        return ctx.err(400, "问题反馈不支持上传图片");
    }
    if raw_images.len() > MAX_FEEDBACK_IMAGES {
        return ctx.err(400, &format!("最多上传 {} 张图片", MAX_FEEDBACK_IMAGES));
    }
    let mut image_urls: Vec<String> = Vec::new();
    if !raw_images.is_empty() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        for (i, img_val) in raw_images.iter().enumerate() {
            let data_url = img_val.as_str().unwrap_or("");
            if data_url.is_empty() {
                continue;
            }
            if data_url.len() > MAX_FEEDBACK_IMAGE_BYTES {
                continue;
            }
            let bytes = match data_url_to_bytes(data_url) {
                Some(b) if b.len() <= MAX_FEEDBACK_IMAGE_BYTES => b,
                _ => continue,
            };
            let name = format!("feedback_{}_{}.jpg", ts, i);
            if let Some(url) = compress_and_save_feedback_image(&bytes, &name, 1600, 82) {
                image_urls.push(public_feedback_img_url(&ctx, url));
            }
        }
    }
    let images_json = json!(image_urls).to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "请先登录");
    }
    if title.is_empty() {
        return ctx.err(400, "标题不能为空");
    }
    if content.is_empty() {
        return ctx.err(400, "内容不能为空");
    }
    if title.chars().count() > 60 {
        return ctx.err(400, "标题不能超过 60 字");
    }
    if content.chars().count() > 1000 {
        return ctx.err(400, "内容不能超过 1000 字");
    }
    let daily_limit = get_feedback_daily_limit(pool).await;
    if daily_limit > 0 {
        let submitted_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_feedback WHERE ciyuanxi_id = ? AND created_at >= CURDATE() AND created_at < CURDATE() + INTERVAL 1 DAY",
        )
        .bind(&ciyuanxi_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        if submitted_today >= daily_limit {
            return ctx.err(
                429,
                &format!("今日反馈提交次数已达上限（{} 条），请明天再试", daily_limit),
            );
        }
    }
    if nickname.is_empty() {
        let row = sqlx::query("SELECT username FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
            .bind(&ciyuanxi_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        if let Some(r) = row {
            nickname = r.get("username");
        }
    }
    let ip = ctx.client_ip.clone();
    let result = sqlx::query(
        "INSERT INTO user_feedback (ciyuanxi_id, nickname, title, content, feedback_type, images, error_logs, all_logs, log_meta, ip, category) VALUES (?,?,?,?,?,?,?,?,?,?,?)",
    )
        .bind(&ciyuanxi_id)
        .bind(&nickname)
        .bind(&title)
        .bind(&content)
        .bind(&feedback_type)
        .bind(&images_json)
        .bind(&error_logs)
        .bind(&all_logs)
        .bind(&log_meta)
        .bind(&ip)
        .bind("feedback")
        .execute(pool)
        .await;
    match result {
        Ok(r) => ctx.json(200, "提交成功", Some(json!({ "id": r.last_insert_id() }))),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

/// 账号封禁申诉：并入 user_feedback 表（category='appeal'），与普通反馈共享每日限额。
pub async fn submit_appeal(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let mut nickname = str_of(&data, "nickname").trim().to_string();
    let content = str_of(&data, "content").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "请先登录");
    }
    if content.is_empty() {
        return ctx.err(400, "申诉内容不能为空");
    }
    if content.chars().count() > 1000 {
        return ctx.err(400, "申诉内容不能超过 1000 字");
    }
    // 与反馈共用每日限额（反馈 + 申诉合计）
    let daily_limit = get_feedback_daily_limit(pool).await;
    if daily_limit > 0 {
        let submitted_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_feedback WHERE ciyuanxi_id = ? AND created_at >= CURDATE() AND created_at < CURDATE() + INTERVAL 1 DAY",
        )
        .bind(&ciyuanxi_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        if submitted_today >= daily_limit {
            return ctx.err(
                429,
                &format!("今日反馈提交次数已达上限（{} 条），请明天再试", daily_limit),
            );
        }
    }
    if nickname.is_empty() {
        let row = sqlx::query("SELECT nickname FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
            .bind(&ciyuanxi_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        if let Some(r) = row {
            nickname = r.get("nickname");
        }
    }
    let ip = ctx.client_ip.clone();
    let result = sqlx::query("INSERT INTO user_feedback (ciyuanxi_id, nickname, title, content, category, ip) VALUES (?,?,?,?,?,?)")
        .bind(&ciyuanxi_id)
        .bind(&nickname)
        .bind("账号申诉")
        .bind(&content)
        .bind("appeal")
        .bind(&ip)
        .execute(pool)
        .await;
    match result {
        Ok(r) => ctx.json(200, "申诉已提交，请耐心等待处理", Some(json!({ "id": r.last_insert_id() }))),
        Err(e) => ctx.err(500, &format!("服务器错误: {}", e)),
    }
}

pub async fn check_ciyuanxi_id(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let mut identifier = str_of(&data, "ciyuanxi_id");
    if identifier.is_empty() {
        identifier = str_of(&data, "user_id");
    }
    if identifier.is_empty() {
        return ctx.err(400, "用户标识不能为空");
    }
    let row = sqlx::query("SELECT id, ciyuanxi_id FROM app_users WHERE id = ? OR ciyuanxi_id = ? LIMIT 1")
        .bind(&identifier)
        .bind(&identifier)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    match row {
        Some(r) => {
            let user_id: i64 = r.get("id");
            let ciyuanxi_id: String = r.get("ciyuanxi_id");
            ctx.json(200, "ok", Some(json!({ "user_id": user_id, "ciyuanxi_id": ciyuanxi_id })))
        }
        None => ctx.err(404, "用户不存在"),
    }
}

/// 获取当前用户的反馈完成通知：返回该用户已解决（resolved）且尚未确认（notified_at 为空）的反馈。
/// 客户端据此调用公告弹窗展示处理管理员与完成说明。
pub async fn get_my_feedback_notifications(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "请先登录");
    }
    let rows = sqlx::query(
        "SELECT id, title, content, assignee, resolve_note, replied_at, updated_at
         FROM user_feedback
         WHERE ciyuanxi_id = ? AND status = 'resolved' AND assignee <> '' AND resolve_note IS NOT NULL AND resolve_note <> '' AND notified_at IS NULL AND deleted_at IS NULL
         ORDER BY updated_at DESC",
    )
    .bind(&ciyuanxi_id)
    .fetch_all(pool)
    .await;
    let list: Vec<Value> = match rows {
        Ok(rows) => rows.iter().map(row_to_json).collect(),
        Err(_) => Vec::new(),
    };
    ctx.ok("获取通知成功", json!({ "list": list }))
}

/// 确认反馈完成通知：将指定反馈的 notified_at 置为当前时间，标记该用户已读。
pub async fn confirm_feedback_notification(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let id = int_of(&data, "id");
    if ciyuanxi_id.is_empty() || id <= 0 {
        return ctx.err(400, "参数错误");
    }
    let upd = sqlx::query(
        "UPDATE user_feedback SET notified_at = NOW() WHERE id = ? AND ciyuanxi_id = ? AND status = 'resolved'",
    )
    .bind(id)
    .bind(&ciyuanxi_id)
    .execute(pool)
    .await;
    match upd {
        Ok(_) => ctx.ok("通知确认成功", json!({ "id": id })),
        Err(_) => ctx.err(500, "服务器错误"),
    }
}

/// 将一行反馈记录转换为 JSON（含 nullable 字段处理）
fn row_to_json(row: &sqlx::mysql::MySqlRow) -> Value {
    use sqlx::Row;
    json!({
        "id": row.get::<i64, _>("id"),
        "title": row.get::<String, _>("title"),
        "content": row.get::<Option<String>, _>("content").unwrap_or_default(),
        "assignee": row.get::<String, _>("assignee"),
        "resolve_note": row.get::<Option<String>, _>("resolve_note").unwrap_or_default(),
        "replied_at": row.try_get::<String, _>("replied_at").unwrap_or_default(),
        "updated_at": row.try_get::<String, _>("updated_at").unwrap_or_default(),
    })
}

/// 获取当前用户的反馈列表（含状态），用于客户端「我的反馈」查看。
pub async fn list_my_feedback(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    if data.is_null() {
        return ctx.err(400, "参数错误");
    }
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "请先登录");
    }
    let rows = sqlx::query(
        "SELECT id, title, content, feedback_type, images, status, category, assignee, resolve_note, error_logs, all_logs, created_at, replied_at, updated_at
         FROM user_feedback
         WHERE ciyuanxi_id = ? AND category = 'feedback' AND deleted_at IS NULL
         ORDER BY created_at DESC
         LIMIT 100",
    )
    .bind(&ciyuanxi_id)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let list: Vec<Value> = rows
                .iter()
                .map(|r| {
                    let images_raw: Option<String> = r.get("images");
                    let images: Vec<String> = images_raw
                        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
                        .unwrap_or_default();
                    json!({
                        "id": r.get::<i64, _>("id"),
                        "title": r.get::<String, _>("title"),
                        "content": r.get::<Option<String>, _>("content").unwrap_or_default(),
                        "feedbackType": r.get::<String, _>("feedback_type"),
                        "images": images,
                        "status": r.get::<String, _>("status"),
                        "category": r.get::<String, _>("category"),
                        "assignee": r.get::<String, _>("assignee"),
                        "resolveNote": r.get::<Option<String>, _>("resolve_note").unwrap_or_default(),
                        "hasErrorLogs": r.get::<Option<String>, _>("error_logs").map(|v| !v.is_empty()).unwrap_or(false),
                        "hasAllLogs": r.get::<Option<String>, _>("all_logs").map(|v| !v.is_empty()).unwrap_or(false),
                        "createdAt": r.try_get::<String, _>("created_at").unwrap_or_default(),
                        "repliedAt": r.try_get::<String, _>("replied_at").unwrap_or_default(),
                        "updatedAt": r.try_get::<String, _>("updated_at").unwrap_or_default(),
                    })
                })
                .collect();
            ctx.ok("ok", json!({ "list": list }))
        }
        Err(_) => ctx.err(500, "数据库错误"),
    }
}
