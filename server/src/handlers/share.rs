use axum::response::Response;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use rand::Rng;
use serde_json::{json, Value};
use sha1::Sha1;
use sqlx::{MySqlPool, Row};

use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::response::ReqCtx;

const SHARE_MIN_MINUTES: i64 = 5;
const SHARE_MAX_MINUTES: i64 = 24 * 60;
const SHARE_DEFAULT_MINUTES: i64 = 120;
const SHARE_ID_LEN: usize = 8;

fn url_encode_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for byte in input.as_bytes() {
        let c = *byte;
        if c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c == b'.' || c == b'~' {
            out.push(c as char);
        } else {
            out.push('%');
            out.push_str(&format!("{:02X}", c));
        }
    }
    out
}

fn cover_thumb_url(cover: &str) -> String {
    if cover.is_empty() || !cover.contains("/uploads/covers/") {
        return cover.to_string();
    }
    let sep = if cover.contains('?') { '&' } else { '?' };
    format!("{cover}{sep}w=150")
}

fn build_song_deep_link(
    song_id: &str,
    hash: &str,
    name: &str,
    artist: &str,
    duration_ms: i64,
    source: &str,
    cover: &str,
) -> String {
    let dur = (duration_ms / 1000).max(1);
    let thumb = cover_thumb_url(cover);
    format!(
        "xianyu://song?id={}&hash={}&name={}&artist={}&duration={}&source={}&cover={}",
        url_encode_component(song_id),
        url_encode_component(hash),
        url_encode_component(name),
        url_encode_component(artist),
        dur,
        url_encode_component(source),
        url_encode_component(&thumb)
    )
}

fn html_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn json_script_safe(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            _ => out.push(ch),
        }
    }
    out
}

fn next_share_id() -> String {
    let mut rng = rand::thread_rng();
    (0..SHARE_ID_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx as u8) as char
            } else {
                (b'a' + (idx - 10) as u8) as char
            }
        })
        .collect()
}

async fn gen_unique_share_id(pool: &MySqlPool) -> String {
    loop {
        let id = next_share_id();
        let count = sqlx::query("SELECT COUNT(*) FROM share_log WHERE share_id = ?")
            .bind(&id)
            .fetch_one(pool)
            .await;
        match count {
            Ok(row) => {
                let n: i64 = row.get(0);
                if n == 0 {
                    return id;
                }
            }
            Err(_) => return id,
        }
    }
}

pub async fn report_share_action(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").to_string();
    match sqlx::query("INSERT INTO share_actions (ciyuanxi_id) VALUES (?)")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await
    {
        Ok(_) => ctx.ok("ok", json!({})),
        Err(e) => { tracing::error!("上报失败: {e}"); ctx.err(500, "上报失败") },
    }
}

fn sanitize_cover_url(cover: &str) -> String {
    let trimmed = cover.trim().to_string();
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return String::new();
    }
    let lower = trimmed.to_lowercase();
    for bad in ["asset.localhost", "localhost", "127.0.0.1", "[::1]"] {
        if lower.contains(bad) {
            return String::new();
        }
    }
    trimmed
}

pub async fn create_share(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    // 落库字段按合理上限截断，防止异常大 payload 直接入库（请求体本身另有 16MB 分级限制）
    let take = |s: &str, n: usize| s.chars().take(n).collect::<String>();
    let song_name = take(&str_of(&data, "song_name"), 200);
    if song_name.is_empty() {
        return ctx.err(400, "歌曲名不能为空");
    }
    let singer = take(&str_of(&data, "singer"), 200);
    let audio_url = take(&str_of(&data, "audio_url"), 2048);
    let lyrics = take(&str_of(&data, "lyrics"), 100_000);
    let cover = sanitize_cover_url(&str_of(&data, "cover_url"));
    let song_id = str_of(&data, "song_id");
    let hash = str_of(&data, "hash");
    let duration_ms = int_of(&data, "duration_ms");
    let source = str_of(&data, "source");
    let raw_ttl = int_of(&data, "expire_minutes");
    let ttl_min = if (SHARE_MIN_MINUTES..=SHARE_MAX_MINUTES).contains(&raw_ttl) {
        raw_ttl
    } else {
        SHARE_DEFAULT_MINUTES
    };
    let deep_link =
        build_song_deep_link(&song_id, &hash, &song_name, &singer, duration_ms, &source, &cover);

    let share_id = gen_unique_share_id(pool).await;
    let share_base = ctx.config.share_base_url.trim().trim_end_matches('/');
    let share_url = if share_base.is_empty() {
        format!("{}/s/{}", ctx.base_url.trim_end_matches('/'), share_id)
    } else {
        format!("{}/s/{}", share_base, share_id)
    };

    let params_truncated: String = body.chars().take(50_000).collect();
    let insert = sqlx::query(
        "INSERT INTO share_log \
         (share_id, song_name, singer, audio_url, lyrics, cover_path, creator_ip, expired_at, view_count, request_params) \
         VALUES (?, ?, ?, ?, ?, ?, ?, DATE_ADD(NOW(), INTERVAL ? MINUTE), 0, ?)",
    )
    .bind(&share_id)
    .bind(&song_name)
    .bind(&singer)
    .bind(&audio_url)
    .bind(&lyrics)
    .bind(&cover)
    .bind(&ctx.client_ip)
    .bind(ttl_min)
    .bind(&params_truncated)
    .execute(pool)
    .await;

    match insert {
        Ok(_) => ctx.ok(
            "ok",
            json!({
                "share_id": share_id,
                "share_url": share_url,
                "deep_link": deep_link,
            }),
        ),
        Err(e) => { tracing::error!("创建分享失败: {e}"); ctx.err(500, "创建分享失败") },
    }
}

fn row_to_share_data(row: &Value, body_params: &Value, download_api: &str, cover_abs: &str) -> Value {
    let song_name = str_of(row, "song_name");
    let singer = str_of(row, "singer");
    let song_id = str_of(body_params, "song_id");
    let hash = str_of(body_params, "hash");
    let duration_ms = int_of(body_params, "duration_ms");
    let source = str_of(body_params, "source");
    json!({
        "title": song_name,
        "artist": singer,
        "cover": cover_abs,
        "duration_ms": duration_ms,
        "deep_link": build_song_deep_link(&song_id, &hash, &song_name, &singer, duration_ms, &source, cover_abs),
        "android_package": str_of(body_params, "android_package"),
        "download_api": download_api,
    })
}

pub fn render_landing_page(row: &Value, body_params: &Value, download_api: &str) -> String {
    let base = download_api
        .split("/api?")
        .next()
        .unwrap_or("")
        .trim_end_matches('/')
        .replacen("http://", "https://", 1)
        .to_string();

    let song = str_of(row, "song_name");
    let singer = str_of(row, "singer");
    let og_title = if singer.is_empty() {
        song.clone()
    } else {
        format!("{} - {}", song, singer)
    };
    let og_desc = "这首歌曲来自弦予音乐，点击卡片即可在 App 内收听全曲";

    let cover = str_of(row, "cover_path");
    let cover_lower = cover.to_lowercase();
    let is_local_cover = cover_lower.contains("asset.localhost")
        || cover_lower.contains("localhost")
        || cover_lower.contains("127.0.0.1")
        || cover_lower.contains("[::1]");
    let cover_abs = if is_local_cover {
        format!("{}/logo.png", base)
    } else if let Some(rest) = cover.strip_prefix("http://") {
        format!("https://{}", rest)
    } else if cover.starts_with("https://") {
        cover
    } else if !cover.is_empty() {
        format!("{}/{}", base, cover.trim_start_matches('/'))
    } else {
        format!("{}/logo.png", base)
    };
    let og_url = format!("{}/s/{}", base, str_of(row, "share_id"));
    let og_image = if cover_abs.contains("/uploads/covers/") {
        let sep = if cover_abs.contains('?') { '&' } else { '?' };
        format!("{cover_abs}{sep}w=300")
    } else {
        cover_abs.clone()
    };
    let page_title = if og_title.is_empty() {
        "弦予音乐 · 分享".to_string()
    } else {
        format!("{} · 弦予音乐", og_title)
    };

    let data = row_to_share_data(row, body_params, download_api, &cover_abs);
    let json_str = json_script_safe(&serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string()));

    HTML
        .replace("__SHARE_JSON__", &json_str)
        .replace("__TITLE__", &html_escape(&page_title))
        .replace("__FAVICON__", &html_escape(&format!("{}/logo.png", base)))
        .replace("__OG_TITLE__", &html_escape(&og_title))
        .replace("__OG_DESC__", &html_escape(og_desc))
        .replace("__OG_IMAGE__", &html_escape(&og_image))
        .replace("__OG_URL__", &html_escape(&og_url))
}

const HTML: &str = include_str!("share_landing.html");

type HmacSha1 = Hmac<Sha1>;

/// 鸿蒙 QQ 开放平台分享签名：服务端用 QQ 互联 AppKey（严禁进客户端）对
/// shareJson+timestamp+nonce 拼接串做 HMAC-SHA1，原始摘要字节转 base64，
/// 客户端拿四个字段透传给鸿蒙 QQ SDK（IQQOpenApi.share）。
pub async fn qq_share_sign(body: &str, ctx: ReqCtx, _pool: &MySqlPool) -> Response {
    let app_key = ctx.config.qq_app_key.trim().to_string();
    if app_key.is_empty() {
        return ctx.err(500, "未配置 QQ_APP_KEY");
    }
    let data = parse_body(body);
    // 字段按 QQ 分享协议合理上限截断，防止异常大 payload
    let take = |s: &str, n: usize| s.chars().take(n).collect::<String>();
    let scene = str_of(&data, "scene").to_string();
    let title = take(&str_of(&data, "title"), 200);
    let summary = take(&str_of(&data, "summary"), 200);
    let url = take(&str_of(&data, "url"), 2048);
    let picture_url = sanitize_cover_url(&str_of(&data, "picture_url"));
    if url.is_empty() {
        return ctx.err(400, "分享链接不能为空");
    }

    // QQ 好友(type 2) ark 图文 / QZone(type 3009) 图文，字段名按官方协议
    let share_json = if scene == "qzone" {
        let mut obj = json!({
            "title": title,
            "summary": summary,
            "targetUrl": url,
        });
        if !picture_url.is_empty() {
            obj["imageUrls"] = json!([picture_url]);
        }
        obj
    } else {
        json!({
            "msg_style": 0,
            "title": title,
            "summary": summary,
            "brief": "弦予音乐",
            "url": url,
            "picture_url": picture_url,
        })
    };
    let share_json_str = serde_json::to_string(&share_json).unwrap_or_default();

    let timestamp = chrono::Utc::now().timestamp();
    let nonce: i64 = rand::thread_rng().gen_range(0..i32::MAX as i64);
    let sign_input = format!("{}{}{}", share_json_str, timestamp, nonce);

    let mut mac = match HmacSha1::new_from_slice(app_key.as_bytes()) {
        Ok(m) => m,
        Err(_) => return ctx.err(500, "QQ_APP_KEY 无效"),
    };
    mac.update(sign_input.as_bytes());
    let sign = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    ctx.ok(
        "ok",
        json!({
            "share_json": share_json_str,
            "timestamp": timestamp,
            "nonce": nonce,
            "share_json_sign": sign,
        }),
    )
}