use axum::response::Response;
use serde_json::{json, Map, Value};
use sqlx::{MySqlPool, Row};

use crate::audit_policy::{self, AuditDecision};
use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::response::ReqCtx;

const DEFAULT_THEME_UPLOAD_LIMIT: i64 = 20;
const MAX_RESOURCE_BYTES_PER_FILE: usize = 2 * 1024 * 1024;
const MAX_RESOURCE_BYTES_PER_THEME: usize = 20 * 1024 * 1024;
/// v3 页面壁纸：单张原图上限 / 单包壁纸总量上限（解码后字节数，落盘前压缩到 1080 宽）
const MAX_WALLPAPER_BYTES_PER_FILE: usize = 8 * 1024 * 1024;
const MAX_WALLPAPER_BYTES_PER_THEME: usize = 12 * 1024 * 1024;
const MAX_PAYLOAD_JSON_BYTES: usize = 8 * 1024 * 1024;
const MAX_PREVIEW_BYTES: usize = 8 * 1024 * 1024;

pub(crate) fn themes_dir() -> std::path::PathBuf {
    std::path::Path::new("uploads").join("themes")
}

fn normalize_platform(raw: &str) -> Result<String, String> {
    match raw.trim() {
        "mobile" => Ok("mobile".to_string()),
        "desktop" => Ok("desktop".to_string()),
        _ => Err("平台仅支持 mobile / desktop".to_string()),
    }
}

async fn read_theme_upload_limit(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT setting_value FROM server_settings WHERE setting_key = 'theme_upload_limit' LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .and_then(|v| v.trim().parse::<i64>().ok())
    .filter(|v| *v >= 0)
    .unwrap_or(DEFAULT_THEME_UPLOAD_LIMIT)
}

pub(crate) fn public_url(ctx: &ReqCtx, url: String) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
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

fn data_url_to_bytes(data_url: &str) -> Option<Vec<u8>> {
    let raw = data_url.split_once(',').map(|(_, v)| v).unwrap_or(data_url);
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(raw).ok()
}

fn compress_and_save_image(bytes: &[u8], target: &std::path::Path, max_w: u32, quality: u32) -> bool {
    use image::GenericImageView;
    let img = match image::load_from_memory(bytes) {
        Ok(i) => i,
        Err(_) => return false,
    };
    let (w, h) = img.dimensions();
    let (nw, nh) = if w > max_w {
        (max_w, (h * max_w / w).max(1))
    } else {
        (w, h)
    };
    let resized = img.resize_exact(nw, nh, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgb8();
    let file = match std::fs::File::create(target) {
        Ok(f) => f,
        Err(_) => return false,
    };
    image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), quality as u8)
        .encode(&rgb, nw, nh, image::ExtendedColorType::Rgb8)
        .is_ok()
}

fn sanitize_slot(slot: &str) -> String {
    slot.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

/// SVG 脚本黑名单校验（非完整 sanitizer，配合响应 attachment 头兜底）：
/// 拒绝 script/foreignObject/事件处理器/javascript: 协议/内嵌 data: 引用
fn svg_has_script(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return true;
    };
    let lower = text.to_ascii_lowercase();
    if lower.contains("<script")
        || lower.contains("<foreignobject")
        || lower.contains("javascript:")
        || lower.contains("href=\"data:")
        || lower.contains("src=\"data:")
    {
        return true;
    }
    // 事件处理器：on*=
    lower
        .match_indices(" on")
        .any(|(i, _)| {
            let rest = &lower[i + 3..];
            let name_len = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            name_len > 0
                && rest[name_len..].trim_start().starts_with('=')
        })
}

fn ext_for_data_url(data_url: &str, bytes: &[u8]) -> Option<&'static str> {
    let lower = data_url.split(';').next().unwrap_or("").to_ascii_lowercase();
    if lower.starts_with("data:image/svg") {
        return Some("svg");
    }
    if lower.starts_with("data:image/gif") {
        return Some("gif");
    }
    match image::guess_format(bytes).ok()? {
        image::ImageFormat::Png => Some("png"),
        image::ImageFormat::Jpeg => Some("jpg"),
        image::ImageFormat::WebP => Some("webp"),
        image::ImageFormat::Gif => Some("gif"),
        _ => None,
    }
}

/// 把 payload.icons / payload.stickers / payload.wallpapers 中 data URL 形式的资源
/// 校验并落盘到 uploads/themes/theme_{id}/ 并改写为服务端托管 URL；
/// 非 data URL 的值原样保留。icons/stickers 条目是字符串，wallpapers 条目是
/// `{ ref, ...调整参数 }` 对象（仅改写 ref，参数原样保留，统一压缩为 1080 宽 JPEG）。
/// id 为 None 时只做校验不落盘（dry-run）；返回完整主题包 JSON 文本。
fn save_theme_resources(id: Option<i64>, payload: &Map<String, Value>) -> Result<String, String> {
    let theme_id = id.unwrap_or(0);
    let dir = themes_dir().join(format!("theme_{}", theme_id));
    if id.is_some() && std::fs::create_dir_all(&dir).is_err() {
        return Err("无法创建主题资源目录".to_string());
    }
    let mut rewritten = payload.clone();
    let mut total_bytes: usize = 0;
    let mut wallpaper_bytes: usize = 0;
    for (group, prefix) in [("icons", "icon"), ("stickers", "sticker"), ("wallpapers", "wallpaper")] {
        let is_wallpaper = group == "wallpapers";
        let Some(entries) = payload.get(group).and_then(|v| v.as_object()).cloned() else {
            continue;
        };
        let mut out = Map::new();
        for (slot, value) in entries {
            let (entry_obj, url) = if is_wallpaper {
                match value.as_object() {
                    Some(obj) => (Some(obj.clone()), obj.get("ref").and_then(|v| v.as_str())),
                    None => (None, None),
                }
            } else {
                (None, value.as_str())
            };
            let Some(url) = url else {
                out.insert(slot, value.clone());
                continue;
            };
            if !url.starts_with("data:") {
                out.insert(slot, value.clone());
                continue;
            }
            let Some(bytes) = data_url_to_bytes(url) else {
                return Err(format!("槽位 {} 的资源数据无效", slot));
            };
            if is_wallpaper {
                if bytes.len() > MAX_WALLPAPER_BYTES_PER_FILE {
                    return Err(format!("页面 {} 的壁纸过大，单张请控制在 8MB 以内", slot));
                }
                wallpaper_bytes += bytes.len();
                if wallpaper_bytes > MAX_WALLPAPER_BYTES_PER_THEME {
                    return Err("页面壁纸总量过大，请控制在 12MB 以内".to_string());
                }
                match image::guess_format(&bytes) {
                    Ok(f) if matches!(f, image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP) => {}
                    _ => return Err(format!("页面 {} 的壁纸仅支持 JPG / PNG / WEBP", slot)),
                }
            } else {
                if bytes.len() > MAX_RESOURCE_BYTES_PER_FILE {
                    return Err(format!("槽位 {} 的资源过大，单个资源请控制在 2MB 以内", slot));
                }
                total_bytes += bytes.len();
                if total_bytes > MAX_RESOURCE_BYTES_PER_THEME {
                    return Err("主题资源总量过大，请控制在 20MB 以内".to_string());
                }
                let Some(ext) = ext_for_data_url(url, &bytes) else {
                    return Err(format!("槽位 {} 的资源格式不支持（仅 PNG/JPG/WEBP/GIF/SVG）", slot));
                };
                if ext == "svg" && svg_has_script(&bytes) {
                    return Err(format!("槽位 {} 的 SVG 包含脚本内容，已拒绝", slot));
                }
                let filename = format!("{}_{}.{}", prefix, sanitize_slot(&slot), ext);
                if id.is_some() && std::fs::write(dir.join(&filename), &bytes).is_err() {
                    let _ = std::fs::remove_dir_all(&dir);
                    return Err("主题资源保存失败，请检查目录权限".to_string());
                }
                out.insert(slot, Value::String(format!("/uploads/themes/theme_{}/{}", theme_id, filename)));
                continue;
            }
            // 壁纸：压缩为 1080 宽 JPEG 落盘，改写 ref，保留其余调整参数
            let filename = format!("{}_{}.jpg", prefix, sanitize_slot(&slot));
            if id.is_some() && !compress_and_save_image(&bytes, &dir.join(&filename), 1080, 80) {
                let _ = std::fs::remove_dir_all(&dir);
                return Err("页面壁纸保存失败，请检查目录权限".to_string());
            }
            let mut entry = entry_obj.unwrap_or_default();
            entry.insert(
                "ref".to_string(),
                Value::String(format!("/uploads/themes/theme_{}/{}", theme_id, filename)),
            );
            out.insert(slot, Value::Object(entry));
        }
        rewritten.insert(group.to_string(), Value::Object(out));
    }
    if id.is_none() {
        return Ok(String::new());
    }
    let package = json!({
        "version": 2,
        "platform": payload.get("platform").and_then(|v| v.as_str()).unwrap_or(""),
        "name": payload.get("name").and_then(|v| v.as_str()).unwrap_or(""),
        "author": payload.get("author").and_then(|v| v.as_str()).unwrap_or(""),
        "preview": payload.get("preview").and_then(|v| v.as_str()).unwrap_or(""),
        "payload": rewritten,
    });
    let text = serde_json::to_string(&package).map_err(|_| "主题数据序列化失败".to_string())?;
    if text.len() > MAX_PAYLOAD_JSON_BYTES {
        let _ = std::fs::remove_dir_all(&dir);
        return Err("主题数据过大，请精简资源后重试".to_string());
    }
    Ok(text)
}

/// 把 payload.wallpapers 中服务端相对路径的 ref（/uploads/...）改写为绝对 URL，
/// 与 previewUrl/thumbnailUrl 同一规则；data URL / 绝对 URL 原样保留。
fn absolutize_wallpaper_refs(ctx: &ReqCtx, package: &mut Value) {
    let Some(wps) = package
        .get_mut("payload")
        .and_then(|p| p.get_mut("wallpapers"))
        .and_then(|w| w.as_object_mut())
    else {
        return;
    };
    for (_page, entry) in wps.iter_mut() {
        let Some(obj) = entry.as_object_mut() else { continue };
        let Some(ref_url) = obj.get("ref").and_then(|r| r.as_str()) else {
            continue;
        };
        if ref_url.starts_with('/') {
            let abs = public_url(ctx, ref_url.to_string());
            obj.insert("ref".to_string(), Value::String(abs));
        }
    }
}

fn row_to_theme(ctx: &ReqCtx, row: &sqlx::mysql::MySqlRow) -> Value {
    let id: i64 = row.try_get::<i64, _>("id").unwrap_or_else(|_| {
        row.try_get::<i32, _>("id").map(|v| v as i64).unwrap_or_default()
    });
    let payload_raw: String = row.try_get::<String, _>("payload").unwrap_or_default();
    let mut payload_json: Value = serde_json::from_str(&payload_raw).unwrap_or(Value::Null);
    absolutize_wallpaper_refs(ctx, &mut payload_json);
    let preview_url = public_url(ctx, row.try_get::<String, _>("preview_url").unwrap_or_default());
    let thumbnail_url = public_url(ctx, row.try_get::<String, _>("thumbnail_url").unwrap_or_default());
    json!({
        "id": id,
        "name": row.try_get::<String, _>("name").unwrap_or_default(),
        "description": row.try_get::<String, _>("description").unwrap_or_default(),
        "platform": row.try_get::<String, _>("platform").unwrap_or_else(|_| "desktop".to_string()),
        "theme": payload_json,
        "previewUrl": preview_url,
        "thumbnailUrl": thumbnail_url,
        "uploaderId": row.try_get::<String, _>("uploaded_by").unwrap_or_default(),
        "uploaderNickname": row.try_get::<String, _>("uploaded_by_nickname").unwrap_or_default(),
        "status": row.try_get::<String, _>("status").unwrap_or_default(),
        "reviewedAt": row.try_get::<String, _>("reviewed_at").ok(),
        "reviewedBy": row.try_get::<String, _>("reviewed_by").unwrap_or_default(),
        "createdAt": row.try_get::<String, _>("created_at").ok(),
    })
}

pub async fn list_themes(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let platform = match normalize_platform(str_of(&data, "platform").as_str()) {
        Ok(p) => p,
        Err(msg) => return ctx.err(400, &msg),
    };
    let rows = sqlx::query(
        "SELECT * FROM themes WHERE status = 'normal' AND platform = ? ORDER BY sort_order DESC, id DESC",
    )
    .bind(&platform)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(|r| row_to_theme(&ctx, r)).collect();
            ctx.ok("ok", list)
        }
        Err(_) => ctx.err(500, "数据库错误"),
    }
}

pub async fn my_themes(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let platform = match normalize_platform(str_of(&data, "platform").as_str()) {
        Ok(p) => p,
        Err(msg) => return ctx.err(400, &msg),
    };
    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "弦予号不能为空");
    }
    let rows = sqlx::query(
        "SELECT * FROM themes WHERE uploaded_by = ? AND platform = ? ORDER BY id DESC",
    )
    .bind(&ciyuanxi_id)
    .bind(&platform)
    .fetch_all(pool)
    .await;
    match rows {
        Ok(rows) => {
            let list: Vec<Value> = rows.iter().map(|r| row_to_theme(&ctx, r)).collect();
            ctx.ok("ok", list)
        }
        Err(_) => ctx.err(500, "数据库错误"),
    }
}

/// 收集 payload.wallpapers 中仍为 data URL 的壁纸 ref（供机审）。
fn wallpaper_data_urls(payload: &Map<String, Value>) -> Vec<String> {
    payload
        .get("wallpapers")
        .and_then(|v| v.as_object())
        .map(|m| {
            m.values()
                .filter_map(|e| e.get("ref").and_then(|r| r.as_str()))
                .filter(|u| u.starts_with("data:"))
                .map(|u| u.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// 客户端签名通道（upload_theme）与网页端点（POST /theme-editor/upload）共用的落库流程。
/// 返回 (id, status, preview_url)；失败返回 (http_code, msg)。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn persist_new_theme(
    ctx: &ReqCtx,
    pool: &MySqlPool,
    ciyuanxi_id: &str,
    nickname: &str,
    name: &str,
    description: &str,
    platform: &str,
    payload: &Map<String, Value>,
    preview_data: &str,
    wallpaper_id: i64,
) -> Result<(i64, String, String), (i32, String)> {
    let payload_for_resource: Map<String, Value> = {
        let mut m = payload.clone();
        m.insert("platform".to_string(), Value::String(platform.to_string()));
        m.insert("name".to_string(), Value::String(name.to_string()));
        m.insert("author".to_string(), Value::String(nickname.to_string()));
        m
    };
    // 先 dry-run 校验资源合法性（大小/格式/总量），通过后再落库
    save_theme_resources(None, &payload_for_resource).map_err(|e| (400, e))?;
    let preview_bytes = data_url_to_bytes(preview_data)
        .filter(|b| !b.is_empty())
        .ok_or((400, "无效的预览图数据".to_string()))?;
    if preview_bytes.len() > MAX_PREVIEW_BYTES {
        return Err((400, "预览图过大，请控制在 8MB 以内".to_string()));
    }
    let preview_valid = image::guess_format(&preview_bytes)
        .map(|f| matches!(f, image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP | image::ImageFormat::Gif))
        .unwrap_or(false);
    if !preview_valid {
        return Err((400, "预览图只支持 JPG / PNG / WEBP / GIF 格式".to_string()));
    }

    let meta = json!({ "ciyuanxi_id": ciyuanxi_id, "kind": "theme", "platform": platform });
    let mut audit = audit_policy::audit_text(pool, "wallpaper", name, meta.clone()).await;
    if matches!(audit.decision, AuditDecision::Pass) {
        audit = audit_policy::audit_image(pool, "wallpaper", preview_data, meta.clone()).await;
    }
    // v3 页面壁纸逐张机审：任一拒绝→整包拒绝；无拒绝但有转人工→待审
    if matches!(audit.decision, AuditDecision::Pass) {
        for url in wallpaper_data_urls(payload) {
            let wa = audit_policy::audit_image(pool, "wallpaper", &url, meta.clone()).await;
            match wa.decision {
                AuditDecision::Reject => {
                    audit = wa;
                    break;
                }
                AuditDecision::Manual => audit = wa,
                AuditDecision::Pass => {}
            }
        }
    }
    let initial_status = match audit.decision {
        AuditDecision::Pass => "normal",
        AuditDecision::Reject => "rejected",
        AuditDecision::Manual => "pending",
    };
    let reviewed_by = if initial_status == "pending" {
        String::new()
    } else {
        format!("external:{}", audit.provider)
    };

    let ins = sqlx::query(
        "INSERT INTO themes (name, description, payload, platform, preview_url, thumbnail_url, wallpaper_id, author_name, status, uploaded_by, uploaded_by_nickname, reviewed_at, reviewed_by) VALUES (?, ?, ?, ?, '', '', ?, ?, ?, ?, ?, IF(? = 'pending', NULL, NOW()), ?)",
    )
    .bind(name)
    .bind(description)
    .bind("{}")
    .bind(platform)
    .bind(wallpaper_id)
    .bind(nickname)
    .bind(initial_status)
    .bind(ciyuanxi_id)
    .bind(nickname)
    .bind(initial_status)
    .bind(&reviewed_by)
    .execute(pool)
    .await;
    let theme_id = match ins {
        Ok(r) => r.last_insert_id() as i64,
        Err(_) => return Err((500, "数据库错误".to_string())),
    };

    // payload 内的相对资源 URL 依赖主题 id，正式落盘一次拿到真实路径
    let final_text = match save_theme_resources(Some(theme_id), &payload_for_resource) {
        Ok(t) => t,
        Err(msg) => {
            let _ = sqlx::query("DELETE FROM themes WHERE id = ?").bind(theme_id).execute(pool).await;
            return Err((500, msg));
        }
    };

    let dir = themes_dir();
    let preview_path = dir.join(format!("preview_{}.jpg", theme_id));
    let thumb_path = dir.join(format!("thumb_{}.jpg", theme_id));
    if !compress_and_save_image(&preview_bytes, &preview_path, 1920, 82) {
        let _ = sqlx::query("DELETE FROM themes WHERE id = ?").bind(theme_id).execute(pool).await;
        let _ = std::fs::remove_dir_all(dir.join(format!("theme_{}", theme_id)));
        return Err((500, "预览图保存失败，请检查目录权限".to_string()));
    }
    if !compress_and_save_image(&preview_bytes, &thumb_path, 480, 72) {
        let _ = std::fs::copy(&preview_path, &thumb_path);
    }
    let preview_url = format!("/uploads/themes/preview_{}.jpg", theme_id);
    let thumb_url = format!("/uploads/themes/thumb_{}.jpg", theme_id);
    let mut final_package: Value = serde_json::from_str(&final_text).unwrap_or(Value::Null);
    if let Some(obj) = final_package.as_object_mut() {
        obj.insert("preview".to_string(), Value::String(preview_url.clone()));
        if let Some(inner) = obj.get_mut("payload").and_then(|v| v.as_object_mut()) {
            inner.insert("preview".to_string(), Value::String(preview_url.clone()));
        }
    }
    let final_text = serde_json::to_string(&final_package).unwrap_or(final_text);
    let upd = sqlx::query("UPDATE themes SET payload = ?, preview_url = ?, thumbnail_url = ? WHERE id = ?")
        .bind(&final_text)
        .bind(&preview_url)
        .bind(&thumb_url)
        .bind(theme_id)
        .execute(pool)
        .await;
    if upd.is_err() {
        return Err((500, "数据库错误".to_string()));
    }

    if initial_status == "pending" {
        crate::admin::email::notify_external_emails_for_module(
            pool,
            &ctx.config,
            &ctx.client_ip,
            "wallpaper",
            "【弦予后台】新主题待审核",
            &format!("用户 {} 上传了主题「{}」，请及时审核。", ciyuanxi_id, name),
            &public_url(ctx, preview_url.clone()),
            &ctx.base_url,
        )
        .await;
    }
    Ok((theme_id, initial_status.to_string(), preview_url))
}

pub async fn upload_theme(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").trim().to_string();
    let nickname = str_of(&data, "nickname").trim().to_string();
    let name = str_of(&data, "name").trim().to_string();
    let description = str_of(&data, "description").trim().to_string();
    let platform = match normalize_platform(str_of(&data, "platform").as_str()) {
        Ok(p) => p,
        Err(msg) => return ctx.err(400, &msg),
    };
    let preview_data = str_of(&data, "preview");
    let wallpaper_id = data
        .get("wallpaperRef")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_i64())
        .or_else(|| data.get("wallpaper_id").and_then(|v| v.as_i64()))
        .unwrap_or_else(|| int_of(&data, "wallpaper_id"));
    let payload = match data.get("payload").and_then(|v| v.as_object()) {
        Some(obj) => obj.clone(),
        None => return ctx.err(400, "主题数据不能为空"),
    };

    if ciyuanxi_id.is_empty() {
        return ctx.err(400, "请先登录");
    }
    if name.is_empty() {
        return ctx.err(400, "请填写主题名称");
    }
    if name.chars().count() > 64 {
        return ctx.err(400, "主题名称过长（最多 64 字）");
    }
    if description.chars().count() > 200 {
        return ctx.err(400, "主题简介过长（最多 200 字）");
    }
    if payload.get("accentColor").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
        return ctx.err(400, "主题缺少强调色配置");
    }

    let user_exists = sqlx::query("SELECT id FROM app_users WHERE ciyuanxi_id = ? AND status = 1 LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if !user_exists {
        return ctx.err(404, "用户不存在");
    }

    let upload_limit = read_theme_upload_limit(pool).await;
    if upload_limit > 0 {
        let current_count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM themes WHERE uploaded_by = ?")
            .bind(&ciyuanxi_id)
            .fetch_one(pool)
            .await
        {
            Ok(v) => v,
            Err(_) => return ctx.err(500, "数据库错误"),
        };
        if current_count >= upload_limit {
            return ctx.err(400, &format!("每个用户最多只能上传 {} 个主题", upload_limit));
        }
    }

    match persist_new_theme(
        &ctx, pool, &ciyuanxi_id, &nickname, &name, &description, &platform, &payload, &preview_data, wallpaper_id,
    )
    .await
    {
        Ok((id, status, preview_url)) => ctx.ok(
            match status.as_str() {
                "normal" => "上传成功，已通过机审",
                "rejected" => "上传成功，但未通过机审",
                _ => "上传成功，等待管理员审核",
            },
            json!({
                "id": id,
                "status": status,
                "previewUrl": public_url(&ctx, preview_url),
            }),
        ),
        Err((code, msg)) => ctx.err(code, &msg),
    }
}
