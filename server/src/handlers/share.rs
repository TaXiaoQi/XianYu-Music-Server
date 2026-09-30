use axum::response::Response;
use rand::Rng;
use serde_json::{json, Value};
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
    let song_name = str_of(&data, "song_name");
    if song_name.is_empty() {
        return ctx.err(400, "歌曲名不能为空");
    }
    let singer = str_of(&data, "singer");
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
    .bind(str_of(&data, "audio_url"))
    .bind(str_of(&data, "lyrics"))
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

const HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
<title>__TITLE__</title>
<!-- 浏览器标签页 favicon：与官网一致使用站点 logo（绝对地址，避免 https 落地页混合内容） -->
<link rel="icon" type="image/png" href="__FAVICON__">
<link rel="apple-touch-icon" href="__FAVICON__">
<meta name="description" content="__OG_DESC__">
<!-- Open Graph：供微信 / 各大平台分享卡片抓取 -->
<meta property="og:type" content="music.song">
<meta property="og:site_name" content="弦予音乐">
<meta property="og:title" content="__OG_TITLE__">
<meta property="og:description" content="__OG_DESC__">
<meta property="og:image" content="__OG_IMAGE__">
<meta property="og:url" content="__OG_URL__">
<!-- QQ / 手机QQ 卡片：腾讯爬虫读取 itemprop 微数据而非 og -->
<meta itemprop="name" content="__OG_TITLE__">
<meta itemprop="image" content="__OG_IMAGE__">
<meta itemprop="description" name="description" content="__OG_DESC__">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:site" content="@xianyu_music">
<meta name="twitter:title" content="__OG_TITLE__">
<meta name="twitter:description" content="__OG_DESC__">
<meta name="twitter:image" content="__OG_IMAGE__">
<style>
:root{--accent:#ec4141;--accent-dark:#d63b3b;--accent-soft:rgba(236,65,65,.08);--text:#353A3E;--text-2:#6b7280;--border:#e5e7eb}
*{margin:0;padding:0;box-sizing:border-box;-webkit-tap-highlight-color:transparent}
html,body{height:100%;overflow:hidden}
body{font-family:-apple-system,BlinkMacSystemFont,"PingFang SC","Hiragino Sans GB","Microsoft YaHei",sans-serif;color:var(--text);-webkit-user-select:none;user-select:none;background:#f6f7f9}
.bg-layer{position:fixed;inset:0;z-index:0;overflow:hidden;background:#f3f4f6}
.bg-blur{position:absolute;inset:-40px;background-image:var(--cover);background-size:cover;background-position:center;filter:blur(48px) saturate(1.15);transform:scale(1.15)}
.bg-mask{position:absolute;inset:0;background:linear-gradient(180deg,rgba(255,255,255,.4) 0%,rgba(246,247,249,.9) 80%)}
.app{position:relative;z-index:1;height:100dvh;display:flex;flex-direction:column;max-width:520px;margin:0 auto;padding:0 24px}
.brand{display:flex;align-items:center;gap:8px;padding:22px 0 4px;font-size:20px;font-weight:800;letter-spacing:.5px;color:var(--text)}
.brand::before{content:"";width:10px;height:10px;border-radius:3px;background:var(--accent);box-shadow:0 4px 12px rgba(236,65,65,.4)}
.sub-brand{font-size:12px;color:var(--text-2);margin-bottom:8px}
.hero{flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;text-align:center;min-height:0}
.cover{position:relative;width:min(72vw,300px);height:min(72vw,300px);border-radius:24px;overflow:hidden;background:#fff;box-shadow:0 20px 60px rgba(236,65,65,.18),0 6px 18px rgba(15,23,42,.08);flex-shrink:0;animation:pop .6s cubic-bezier(.16,1,.3,1) both}
@keyframes pop{from{opacity:0;transform:translateY(26px) scale(.94)}to{opacity:1;transform:none}}
.cover img{width:100%;height:100%;object-fit:cover;display:block;transition:opacity .3s ease}
.song-name{margin-top:22px;font-size:21px;font-weight:700;line-height:1.3;word-break:break-all;animation:rise .6s .08s both}
.singer{font-size:14px;color:var(--text-2);margin-top:6px;animation:rise .6s .15s both}
@keyframes rise{from{opacity:0;transform:translateY(14px)}to{opacity:1;transform:none}}
.tip{font-size:12px;color:var(--text-2);margin-top:18px;animation:rise .6s .22s both}
.actions{padding:10px 0 30px;flex-shrink:0;display:flex;flex-direction:column;gap:12px}
  .btn{border:none;border-radius:16px;padding:16px;font-size:16px;font-weight:700;cursor:pointer;display:flex;align-items:center;justify-content:center;gap:8px;transition:transform .18s,box-shadow .18s;font-family:inherit;text-decoration:none}
  .btn:active{transform:scale(.97)}
  .btn-primary{background:linear-gradient(135deg,#ec4141,#d63b3b);color:#fff;box-shadow:0 10px 26px rgba(236,65,65,.35)}
  .btn-ghost{background:rgba(255,255,255,.8);color:#555a66;border:1px solid var(--border);backdrop-filter:blur(10px)}
/* 下载弹窗 */
.modal-mask{position:fixed;inset:0;z-index:50;background:rgba(15,23,42,.45);display:none;align-items:center;justify-content:center;padding:24px;backdrop-filter:blur(6px)}
.modal-mask.show{display:flex}
.modal{width:min(420px,100%);background:#fff;border-radius:20px;padding:26px 22px 20px;box-shadow:0 24px 60px rgba(15,23,42,.2);text-align:center;animation:pop .35s cubic-bezier(.16,1,.3,1) both}
.modal h3{font-size:18px;font-weight:700}
.modal p{font-size:13px;color:var(--text-2);margin:10px 0 20px;line-height:1.6}
.modal .row{display:flex;gap:12px}
.modal .btn{flex:1;padding:13px;font-size:15px;border-radius:14px}
.modal .btn-cancel{background:#f1f2f4;color:#6b7280}
.modal .btn-download{background:linear-gradient(135deg,#ec4141,#d63b3b);color:#fff;box-shadow:0 6px 18px rgba(236,65,65,.3)}
/* 更多下载渠道 */
.more-wrap{margin-top:14px;border-top:1px solid var(--border);padding-top:10px;text-align:left}
.more-toggle{border:none;background:none;color:var(--text-2);font-size:12px;font-weight:600;cursor:pointer;display:inline-flex;align-items:center;gap:4px;padding:0}
.more-toggle:hover{color:var(--accent)}
.more-arrow{transition:transform .2s}
.more-wrap.open .more-arrow{transform:rotate(180deg)}
.more-channels{display:none;flex-direction:column;gap:8px;margin-top:10px}
.more-wrap.open .more-channels{display:flex}
.channel-link{display:flex;align-items:center;justify-content:space-between;gap:10px;padding:10px 12px;border-radius:10px;background:#f6f7f9;color:#6b7280;text-decoration:none;font-size:13px;font-weight:700;text-align:left;transition:background .16s,color .16s,opacity .16s}
.channel-link:hover{background:#eef0f3;color:var(--accent)}
.channel-link:active{transform:scale(.98)}
.channel-link.no-url{opacity:.55;cursor:not-allowed}
.channel-link .channel-tag{font-size:10px;font-weight:700;padding:1px 6px;border-radius:999px;background:rgba(236,65,65,.1);color:var(--accent);margin-left:8px;flex-shrink:0}
.channel-link .channel-cur{font-size:10px;font-weight:800;padding:1px 7px;border-radius:999px;background:linear-gradient(135deg,#ec4141,#d63b3b);color:#fff;flex-shrink:0}
.channel-link.is-cur{background:rgba(236,65,65,.08);color:#c03434;box-shadow:inset 0 0 0 1px rgba(236,65,65,.25)}
.share-lang{position:fixed;top:14px;right:14px;z-index:999}
.share-lang .lang-switch__btn{display:inline-flex;align-items:center;justify-content:center;width:40px;height:40px;border-radius:50%;border:1px solid rgba(15,23,42,.08);background:rgba(255,255,255,.9);color:#353a3e;cursor:pointer;box-shadow:0 4px 16px rgba(15,23,42,.12);transition:background .18s,color .18s}
.share-lang .lang-switch__btn:hover{color:#ec4141;background:#fff}
.share-lang .lang-menu{position:absolute;top:calc(100% + 8px);right:0;min-width:150px;background:#fff;border:1px solid #e5e7eb;border-radius:12px;padding:6px;box-shadow:0 16px 40px rgba(15,23,42,.18);display:none;z-index:999}
.share-lang .lang-switch.open .lang-menu{display:block}
.share-lang .lang-menu__item{display:block;width:100%;text-align:left;padding:9px 12px;border-radius:8px;border:none;background:none;color:#353a3e;font-size:14px;font-weight:600;cursor:pointer;white-space:nowrap}
.share-lang .lang-menu__item:hover{background:#f3f4f6}
.share-lang .lang-menu__item.active{color:#ec4141;background:rgba(236,65,65,.08)}
</style>
</head>
<body>
<script>
window.__SHARE_DATA__ = __SHARE_JSON__;
</script>
<div class="bg-layer"><div class="bg-blur"></div><div class="bg-mask"></div></div>
<div class="share-lang" data-i18n-mount="dropdown"></div>
<div class="app">
  <div class="brand">弦予音乐</div>
  <div class="sub-brand">分享 · 打开客户端听全部</div>
  <div class="hero">
    <div class="cover"><img id="coverImg" alt="封面"></div>
    <div class="song-name" id="songName"></div>
    <div class="singer" id="singerName"></div>
    <div class="tip">网页不提供播放，点击下方按钮到 App 内收听全曲</div>
  </div>
  <div class="actions">
    <a class="btn btn-primary" href="javascript:void(0)" onclick="openApp()">在弦予音乐中打开</a>
    <a class="btn btn-ghost" href="https://qm.qq.com/q/d0Yfxu40us" target="_blank" rel="noopener">加入官方群聊</a>
  </div>
</div>

<div class="modal-mask" id="downloadMask">
  <div class="modal">
    <h3>未检测到弦予音乐</h3>
    <p id="browserHint" style="display:none;color:#ec4141;font-weight:700">你在 QQ / 微信内打开，拉起可能被拦截：点右上角「···」选「在浏览器打开」后重试；找不到菜单时，可直接复制链接粘贴到手机浏览器。</p>
    <p id="downloadHint">确认前往官方下载页安装最新版本吗？已安装用户在收到歌曲后即可直接打开收听。</p>
    <div class="row">
      <button class="btn btn-cancel" onclick="hideDownload()">暂不</button>
      <button class="btn btn-download" onclick="goDownload()" id="downloadPrimary">前往下载</button>
    </div>
    <button class="btn btn-ghost" onclick="copyLink()" style="margin-top:2px">复制链接，去浏览器打开</button>
    <div class="more-wrap" id="moreWrap">
      <button type="button" class="more-toggle" onclick="toggleChannels()">更多下载渠道 <span class="more-arrow">▾</span></button>
      <div class="more-channels" id="moreChannels"></div>
    </div>
  </div>
</div>

<script>
(function(){
  var d = window.__SHARE_DATA__ || {};
  var hdUrl = d.cover || '';
  // 仅本站 /uploads/covers/ 支持 ?w=150 实时缩略图；第三方插件 CDN 封面原样使用，
  // 避免给外链追加不支持的参数导致缩略图加载失败（封面区域长时间空白）。
  var isSiteCover = hdUrl.indexOf('/uploads/covers/') >= 0;
  var thumbUrl = (isSiteCover ? hdUrl + (hdUrl.indexOf('?') >= 0 ? '&' : '?') + 'w=150' : hdUrl) || '';
  var coverImg = document.getElementById('coverImg');
  var bg = document.querySelector('.bg-blur');

  // 优先显示缩略图（小体积快速加载），高清图预加载完成后无缝替换
  if (thumbUrl) {
    coverImg.src = thumbUrl;
    bg.style.setProperty('--cover', "url('" + thumbUrl + "')");
    // 缩略图加载失败时立即回退高清原图，避免封面区域空白
    coverImg.onerror = function(){
      if (hdUrl && coverImg.src !== hdUrl) {
        coverImg.src = hdUrl;
        bg.style.setProperty('--cover', "url('" + hdUrl + "')");
      }
    };
  }
  // 预加载高清版（仅本站封面需要替换；第三方 CDN 封面直接用原图）
  if (hdUrl && isSiteCover) {
    var img = new Image();
    img.onload = function(){
      // 淡入替换，无感知
      coverImg.style.opacity = '0';
      setTimeout(function(){
        coverImg.src = hdUrl;
        bg.style.setProperty('--cover', "url('" + hdUrl + "')");
        coverImg.style.opacity = '1';
      }, 250);
    };
    img.onerror = function(){
      // 高清加载失败也不影响，继续显示缩略图
    };
    img.src = hdUrl;
  }

  document.getElementById('songName').textContent = d.title || '';
  document.getElementById('singerName').textContent = d.artist || '';

  window.__devicePlatform = detectPlatform();
})();

/* 下载渠道元数据 */
var PLATFORM_META = [
  { key: 'mobile',  label: '移动端', sub: 'Android / iOS' },
  { key: 'desktop', label: '桌面端', sub: 'Windows / macOS / Linux' },
  { key: 'watch',   label: '腕上端', sub: '手表' }
];
function platformMeta(key){
  for (var i = 0; i < PLATFORM_META.length; i++) if (PLATFORM_META[i].key === key) return PLATFORM_META[i];
  return PLATFORM_META[1];
}
/* 根据 UA 判断当前设备的下载平台 */
function detectPlatform(){
  var ua = navigator.userAgent || '';
  var androidWatch = /android/i.test(ua) && /wear|wos|smartwatch/i.test(ua);
  if (androidWatch) return 'watch';
  if (/android|iPhone|iPad|iPod|Mobi/i.test(ua)) return 'mobile';
  return 'desktop';
}

/* Android：QQ/微信等内置 WebView 拦截裸 scheme 跳转，需用 intent:// 拉起。
   带上 package= 直达解析 + S.browser_fallback_url（Chrome 未装时自动开落地页）。
   iOS 及桌面端无此问题，维持裸 scheme。 */
function buildLaunchUrl(){
  var d = window.__SHARE_DATA__ || {};
  var dl = d.deep_link || 'xianyu://';
  if (!/android/i.test(navigator.userAgent || '')) return dl;
  var inner = dl.replace(/^xianyu:\/\/\/?/, '');
  var intent = 'intent://' + inner + '#Intent;scheme=xianyu;';
  if (d.android_package) intent += 'package=' + encodeURIComponent(d.android_package) + ';';
  intent += 'S.browser_fallback_url=' + encodeURIComponent(location.href) + ';end';
  return intent;
}

/* QQ/微信内置 WebView 对 location.href 的 scheme/intent 导航拦截最狠，
   动态 <a> 标签点击是用户手势导航通道，放行率高；location.href 兜底。 */
function navigate(url){
  try {
    var a = document.createElement('a');
    a.href = url;
    a.style.display = 'none';
    document.body.appendChild(a);
    a.click();
    setTimeout(function(){ a.remove(); }, 300);
  } catch (e) {
    window.location.href = url;
  }
}
/* QQ/微信内打开：intent/scheme 都可能被拦，失败时引导用右上角菜单转到系统浏览器 */
function inTencentWebview(){
  var ua = navigator.userAgent || '';
  return /MQQBrowser/i.test(ua) || /\bQQ\//i.test(ua) || /MicroMessenger/i.test(ua);
}

function openApp(){
  showToast('正在打开弦予音乐...');
  var launched = false;
  function markLaunched(){
    launched = true;
    window.removeEventListener('blur', markLaunched);
    document.removeEventListener('visibilitychange', onVisibility);
  }
  function onVisibility(){ if (document.hidden) markLaunched(); }
  // blur + visibilitychange 双重检测 App 是否真正拉起
  window.addEventListener('blur', markLaunched);
  document.addEventListener('visibilitychange', onVisibility);
  navigate(buildLaunchUrl());
  setTimeout(function(){
    if (!launched) showDownload();
  }, 2500);
}
function showDownload(){
  var hint = document.getElementById('browserHint');
  if (hint) hint.style.display = inTencentWebview() ? 'block' : 'none';
  document.getElementById('downloadMask').classList.add('show');
}
function hideDownload(){ document.getElementById('downloadMask').classList.remove('show'); }

/* 复制落地页链接：QQ 新版内置视图可能没有「···」菜单，复制粘贴是保底出口。
   navigator.clipboard 在老 X5 内核可能缺失，textarea + execCommand 兜底。 */
function copyLink(){
  var url = location.href;
  function ok(){ showToast('链接已复制，请粘贴到浏览器打开'); hideDownload(); }
  function legacy(){
    var ta = document.createElement('textarea');
    ta.value = url;
    ta.style.cssText = 'position:fixed;top:0;left:0;opacity:0';
    document.body.appendChild(ta);
    ta.focus(); ta.select();
    var done = false;
    try { done = document.execCommand('copy'); } catch (e) {}
    ta.remove();
    if (done) ok(); else showToast('复制失败，请手动复制地址栏链接');
  }
  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard.writeText(url).then(ok, legacy);
  } else {
    legacy();
  }
}

/* 请求某平台的服务器发布版本（成员 auth 接口，返回 download_url）*/
function requestDownload(platform, cb){
  var d = window.__SHARE_DATA__ || {};
  if (!d.download_api) { cb(null); return; }
  fetch(d.download_api, { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify({ platform: platform }) })
    .then(function(r){ return r.json(); })
    .then(function(res){ cb(res && res.data && res.data.download_url ? res.data : null); })
    .catch(function(){ cb(null); });
}

/* 主按钮：推当前设备的对应下载渠道 */
function goDownload(){
  var key = window.__devicePlatform || 'desktop';
  requestDownload(key, function(info){
    if (info && info.download_url) {
      window.location.href = info.download_url;
    } else {
      showToast('当前平台暂未发布版本，请在下方更多渠道中选择');
    }
  });
}

/* 「更多下载渠道」折叠面板：懒加载所有平台，逐个请求可用下载 */
var _moreLoaded = false;
function toggleChannels(){
  var wrap = document.getElementById('moreWrap');
  wrap.classList.toggle('open');
  if (!_moreLoaded) loadChannels();
  _moreLoaded = true;
}
function loadChannels(){
  var box = document.getElementById('moreChannels');
  box.innerHTML = '';
  PLATFORM_META.forEach(function(m){
    var item = document.createElement('a');
    item.className = 'channel-link';
    item.setAttribute('href', 'javascript:void(0)');
    if (m.key === window.__devicePlatform) item.classList.add('is-cur');
    var left = document.createElement('span');
    left.textContent = m.label;
    var tag = document.createElement('span');
    tag.className = 'channel-tag';
    tag.textContent = m.sub;
    left.appendChild(tag);
    item.appendChild(left);
    if (m.key === window.__devicePlatform) {
      var curBadge = document.createElement('span');
      curBadge.className = 'channel-cur';
      curBadge.textContent = '当前设备';
      item.appendChild(curBadge);
    }
    box.appendChild(item);
    requestDownload(m.key, function(info){
      if (info && info.download_url) {
        item.href = info.download_url;
      } else {
        item.classList.add('no-url');
        item.removeAttribute('href');
      }
    });
  });
}
var _toastTimer=null;
function showToast(msg){
  var t=document.createElement('div');
  t.style.cssText='position:fixed;bottom:110px;left:50%;transform:translateX(-50%);background:rgba(0,0,0,.75);color:#fff;padding:10px 22px;border-radius:24px;font-size:13px;z-index:100;transition:opacity .3s;';
  t.textContent=msg; document.body.appendChild(t);
  clearTimeout(_toastTimer);
  setTimeout(function(){ t.style.opacity='0'; setTimeout(function(){ t.remove(); },350); },2000);
}

/* QQ/微信内置 WebView 拉起 scheme/intent 几乎必被拦，主按钮直接换成
   「复制链接去浏览器打开」，省去一次注定失败的尝试与弹窗绕路。 */
if (inTencentWebview()) {
  var _primaryBtn = document.querySelector('.actions .btn-primary');
  if (_primaryBtn) {
    _primaryBtn.textContent = '复制链接，去浏览器打开';
    _primaryBtn.setAttribute('onclick', 'copyLink()');
  }
}
</script>
<script>
/* 弦予音乐 · 官网 i18n 内核
 * 机制：简体为基底，运行时词典替换文本节点 + 常用属性。
 * zh-TW：简→繁字级映射 + 词语级 override（台湾用语）。
 * en：词典精确匹配（key=简体原文），未命中保持简体。
 * 语言：auto(跟随系统/浏览器) | zh-CN | zh-TW | en，localStorage 持久化。
 */
(function () {
  'use strict';

  var LS_KEY = 'xy_lang';
  var current = null;        // 已解析语言
  var pref = 'auto';         // 用户偏好（含 auto）
  var srcMap = new WeakMap(); // TextNode -> 原始简体文本（切换语言前恢复用）
  var suppress = false;       // Observer 自触发抑制

  function stored() {
    try { return localStorage.getItem(LS_KEY); } catch (e) { return null; }
  }
  function detectSystem() {
    var langs = (navigator.languages && navigator.languages.length) ? navigator.languages : [navigator.language || 'zh-CN'];
    for (var i = 0; i < langs.length; i++) {
      var l = String(langs[i] || '').toLowerCase();
      if (l === 'zh-tw' || l === 'zh-hk' || l === 'zh-mo' || l.indexOf('zh-hant') === 0 || /^zh-(hant|tw|hk|mo)/.test(l)) return 'zh-TW';
      if (l.indexOf('en') === 0) return 'en';
      if (l.indexOf('zh') === 0) return 'zh-CN';
    }
    return 'zh-CN';
  }
  function resolve(p) {
    if (p === 'zh-CN' || p === 'zh-TW' || p === 'en') return p;
    return detectSystem();
  }

  /* ============ 简→繁：词语级 override（台湾用语，先替换，单轮不回扫） ============ */
  var PHRASE_LIST = [
    ['服务器', '伺服器'], ['数据库', '資料庫'], ['内存', '記憶體'], ['缓存', '快取'],
    ['软件', '軟體'], ['硬件', '硬體'], ['固件', '韌體'],
    ['视频', '影片'], ['音频', '音訊'], ['在线', '線上'], ['离线', '離線'],
    ['网络', '網路'], ['智能', '智慧'], ['文件夹', '資料夾'], ['文档', '文件'], ['文件', '檔案'],
    ['设置', '設定'], ['默认', '預設'], ['账号', '帳號'], ['账户', '帳戶'],
    ['登录', '登入'], ['登陆', '登入'], ['注销', '登出'],
    ['支持', '支援'], ['社区', '社群'], ['项目', '專案'], ['仓库', '儲存庫'],
    ['界面', '介面'], ['鼠标', '滑鼠'], ['屏幕', '螢幕'],
    ['桌面端', '桌面版'], ['移动端', '行動版'], ['腕上端', '腕上版'],
    ['歌单', '播放清單'], ['壁纸', '桌布'], ['消息', '訊息'], ['反馈', '回饋'],
    ['音乐', '音樂'], ['专辑', '專輯'], ['备份', '備份'], ['恢复', '還原'],
    ['上传', '上傳'], ['下载', '下載'], ['邮箱', '信箱'], ['邮件', '郵件'],
    ['设备', '裝置'], ['数据', '資料'], ['信息', '資訊'], ['链接', '連結'],
    ['菜单', '選單'], ['优化', '最佳化'], ['搜索', '搜尋'],
    ['导入', '匯入'], ['导出', '匯出'], ['刷新', '重新整理'],
    ['评论', '留言'], ['回复', '回覆'], ['用户', '使用者'], ['权限', '權限'],
    ['日志', '日誌'], ['筛选', '篩選'], ['当前', '目前'], ['实时', '即時'],
    ['性能', '效能'], ['字体', '字型'], ['图片', '圖片'],
    ['创建', '建立'], ['新建', '新增'], ['添加', '新增'], ['保存', '儲存'],
    ['粘贴', '貼上'], ['剪切', '剪下'], ['复制', '複製'], ['重复', '重複'],
    ['复杂', '複雜'], ['复合', '複合'], ['运行', '執行'], ['重启', '重新啟動'],
    ['终端', '終端機'], ['命令', '指令'], ['端口', '連接埠'],
    ['环境变量', '環境變數'], ['变量', '變數'], ['域名', '網域'], ['证书', '憑證'],
    ['进程', '處理程序'], ['磁盘', '磁碟'], ['验证', '驗證'], ['质量', '品質'],
    ['教程', '教學'], ['脚本', '腳本'], ['卸载', '解除安裝'],
    ['点赞', '點讚'], ['伙伴', '夥伴'], ['干净', '乾淨'], ['干燥', '乾燥'],
    ['面包', '麵包'], ['面条', '麵條'], ['制造', '製造'], ['制作', '製作'],
    ['采用', '採用'], ['采取', '採取'], ['采购', '採購'], ['采访', '採訪'],
    ['心脏', '心臟'], ['内脏', '內臟'], ['肮脏', '骯髒'], ['弄脏', '弄髒'],
    ['头发', '頭髮'], ['理发', '理髮'], ['杂志', '雜誌'], ['日历', '日曆'],
    ['游泳', '游泳'], ['皇后', '皇后'], ['手表', '手錶'], ['手表带', '手錶帶'],
    ['战斗', '戰鬥'], ['奋斗', '奮鬥'], ['斗争', '鬥爭'],
    ['联系', '聯繫'], ['关系', '關係'], ['细致', '細緻'], ['忧郁', '憂鬱'],
    ['览器', '覽器'], ['浏览器', '瀏覽器']
  ];
  var PHRASE_MAP = {};
  PHRASE_LIST.forEach(function (p) { PHRASE_MAP[p[0]] = p[1]; });
  var PHRASE_RE = new RegExp('(' + PHRASE_LIST
    .slice().sort(function (a, b) { return b[0].length - a[0].length; })
    .map(function (p) { return p[0].replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); })
    .join('|') + ')', 'g');

  /* ============ 简→繁：字级映射（两字一组：简繁） ============ */
  var S2T_SRC =
    '爱愛碍礙袄襖奥奧坝壩罢罷摆擺败敗办辦帮幫绑綁宝寶报報币幣毕畢边邊变變标標宾賓' +
    '补補参參惨慘灿燦苍蒼仓倉层層尝嘗长長偿償场場车車彻徹尘塵陈陳称稱惩懲迟遲' +
    '冲衝丑醜础礎处處触觸传傳闯闖创創纯純词詞辞辭从從丛叢凑湊窜竄错錯' +
    '达達带帶贷貸单單担擔胆膽弹彈诞誕当當挡擋党黨捣搗导導岛島祷禱灯燈' +
    '敌敵涤滌递遞点點电電调調钓釣订訂东東动動冻凍斗鬥独獨读讀赌賭' +
    '断斷锻鍛队隊对對吨噸顿頓夺奪堕墮讹訛额額恶惡儿兒尔爾饿餓发發罚罰' +
    '阀閥范範贩販饭飯访訪纺紡飞飛废廢费費纷紛坟墳奋奮粪糞丰豐风風' +
    '缝縫讽諷凤鳳妇婦复復负負该該盖蓋干幹刚剛钢鋼岗崗纲綱给給巩鞏' +
    '沟溝构構购購够夠蛊蠱顾顧刮颳关關观觀馆館惯慣贯貫广廣归歸龟龜' +
    '规規贵貴锅鍋国國过過韩韓汉漢号號阂閡鹤鶴贺賀横橫轰轟红紅后後' +
    '壶壺护護华華划劃画畫话話怀懷坏壞欢歡环環还還缓緩换換唤喚谎謊' +
    '挥揮汇匯会會讳諱贿賄秽穢获獲机機鸡雞积積极極级級击擊计計记記' +
    '际際剂劑济濟继繼价價驾駕歼殲监監艰艱拣揀简簡见見舰艦剑劍键鍵' +
    '溅濺将將姜薑浆漿奖獎奖獎讲講酱醬胶膠浇澆骄驕娇嬌搅攪缴繳轿轎' +
    '较較阶階节節洁潔结結诫誡届屆紧緊谨謹进進尽盡惊驚经經静靜竞競' +
    '净淨径徑旧舊剧劇据據惧懼卷捲觉覺绝絕军軍开開凯凱颗顆壳殼课課' +
    '垦懇恳懇夸誇块塊亏虧扩擴蜡蠟来來赖賴蓝藍栏欄拦攔烂爛览覽' +
    '劳勞捞撈乐樂类類泪淚篱籬离離里裡礼禮丽麗励勵历歷厉厲隶隸' +
    '联聯怜憐帘簾莲蓮连連炼煉练練粮糧两兩辆輛谅諒疗療辽遼猎獵' +
    '临臨邻鄰鳞鱗灵靈岭嶺领領刘劉龙龍楼樓娄婁芦蘆卢盧炉爐鲁魯' +
    '陆陸录錄虑慮乱亂论論罗羅络絡骆駱妈媽马馬玛瑪吗嗎买買卖賣' +
    '迈邁麦麥脉脈满滿蛮蠻谩謾猫貓么麼门門闷悶们們梦夢弥彌谜謎' +
    '觅覓绵綿缅緬庙廟灭滅悯憫鸣鳴谋謀亩畝纳納难難恼惱脑腦闹鬧' +
    '内內拟擬酿釀鸟鳥聂聶宁寧农農浓濃诺諾盘盤庞龐赔賠喷噴鹏鵬' +
    '骗騙飘飄频頻贫貧苹蘋凭憑评評泼潑铺鋪仆僕朴樸谱譜齐齊骑騎' +
    '岂豈启啟弃棄气氣迁遷签簽谦謙钱錢潜潛浅淺谴譴枪槍墙牆强強' +
    '抢搶桥橋侨僑窍竅亲親轻輕倾傾庆慶琼瓊穷窮区區驱驅权權劝勸' +
    '确確让讓扰擾热熱认認韧韌荣榮绒絨软軟锐銳闰閏润潤洒灑萨薩' +
    '赛賽伞傘丧喪骚騷涩澀杀殺筛篩晒曬删刪闪閃陕陝赡贍绍紹设設' +
    '绅紳审審肾腎声聲胜勝圣聖师師湿濕时時识識实實县縣线線宪憲' +
    '乡鄉详詳响響项項萧蕭销銷晓曉啸嘯协協胁脅写寫泻瀉谢謝兴興' +
    '锈鏽虚虛须須许許绪緒续續轩軒悬懸选選学學询詢训訓讯訊逊遜' +
    '压壓亚亞严嚴盐鹽颜顏阎閻艳豔厌厭验驗阳陽养養样樣谣謠药藥' +
    '爷爺页頁业業叶葉医醫仪儀义義议議亿億忆憶异異译譯阴陰银銀' +
    '隐隱应應婴嬰樱櫻鹰鷹营營蝇蠅赢贏拥擁佣傭踊踴优優忧憂邮郵' +
    '犹猶诱誘与與屿嶼语語誉譽预預驭馭渊淵园園员員圆圓缘緣远遠' +
    '愿願约約跃躍岳嶽粤粵云雲运運韵韻杂雜灾災载載赞讚赃贓' +
    '脏髒凿鑿责責择擇则則贼賊赠贈扎紮诈詐斋齋债債战戰张張涨漲' +
    '账帳胀脹赵趙蛰蟄辙轍针針侦偵诊診镇鎮阵陣挣掙证證织織职職' +
    '执執纸紙挚摯掷擲帜幟质質钟鐘种種众眾昼晝骤驟诸諸烛燭嘱囑' +
    '贮貯铸鑄筑築专專转轉赚賺庄莊装裝壮壯状狀准準谘諮资資' +
    '渍漬踪蹤综綜总總纵縱邹鄒组組钻鑽乌烏乔喬习習书書争爭' +
    '伟偉伤傷伪偽体體余餘侠俠侣侶侧側侪儕俩倆俭儉储儲兑兌' +
    '兰蘭兹茲兽獸冈岡况況凉涼减減凛凜' +
    '厂廠厕廁厘釐厦廈厨廚厮廝叙敘叠疊叹嘆' +
    '违違适適逻邏遗遺遥遙' +
    '郑鄭酝醞释釋鉴鑒钥鑰钦欽钧鈞钨鎢铃鈴铅鉛铜銅铝鋁' +
    '铠鎧镁鎂锋鋒顽頑' +
    '饮飲饰飾饱飽饲飼饼餅馈饋驶駛骂罵' +
    '鸭鴨鸽鴿鹅鵝黄黃' +
    '显顯蚕蠶网網于於';
  var S2T = {};
  for (var i = 0; i < S2T_SRC.length; i += 2) S2T[S2T_SRC.charAt(i)] = S2T_SRC.charAt(i + 1);

  function hasCJK(s) { return /[\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff]/.test(s); }

  function toTraditional(s) {
    s = s.replace(PHRASE_RE, function (m) { return PHRASE_MAP[m] || m; });
    if (!hasCJK(s)) return s;
    var out = '';
    for (var i = 0; i < s.length; i++) { var c = s.charAt(i); out += S2T[c] || c; }
    return out;
  }

  /* ============ EN 词典（key=简体原文） ============ */
  var EN = {
    
"分享 · 打开客户端听全部": "Share · Open the app to listen to the full song",
"封面": "Cover",
"复制链接，去浏览器打开": "Copy Link and open it in your browser",
"更多下载渠道": "More download options",
"加入官方群聊": "Join the official group chat",
"你在 QQ / 微信内打开，拉起可能被拦截：点右上角「···」选「在浏览器打开」后重试；找不到菜单时，可直接复制链接粘贴到手机浏览器。": "You opened this page in QQ / WeChat, where launching the app may be blocked: tap the ··· menu at the top right, choose Open in Browser, then retry. If you cannot find the menu, copy the link and paste it into your mobile browser.",
"前往下载": "Go to Download",
"确认前往官方下载页安装最新版本吗？已安装用户在收到歌曲后即可直接打开收听。": "Continue to the official download page to install the latest version? Users who have installed the app can open and listen to a song right after receiving it.",
"网页不提供播放，点击下方按钮到 App 内收听全曲": "Playback is not available on the web. Tap the button below to listen to the full song in the app.",
"未检测到弦予音乐": "XianYu Music not detected",
"弦予音乐": "XianYu Music",
"在弦予音乐中打开": "Open in XianYu Music",
"暂不": "Not Now"

  };

  function translate(s) {
    if (!s || !hasCJK(s)) return s;
    if (current === 'zh-TW') return toTraditional(s);
    if (current === 'en') return EN[s] !== undefined ? EN[s] : s;
    return s;
  }

  /* ============ DOM 应用 ============ */
  var ATTRS = ['placeholder', 'title', 'aria-label', 'data-tip'];
  var SKIP_TAGS = { SCRIPT: 1, STYLE: 1, CODE: 1, PRE: 1, TEXTAREA: 1, NOSCRIPT: 1 };

  function apply(root) {
    if (current === 'zh-CN' || !current) { restore(root); return; }
    suppress = true;
    try {
      var w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
        acceptNode: function (n) {
          if (!n.nodeValue || !n.nodeValue.trim()) return NodeFilter.FILTER_REJECT;
          var p = n.parentElement;
          if (p && SKIP_TAGS[p.tagName]) return NodeFilter.FILTER_REJECT;
          return NodeFilter.FILTER_ACCEPT;
        }
      });
      var nodes = [];
      while (w.nextNode()) nodes.push(w.currentNode);
      for (var i = 0; i < nodes.length; i++) {
        var n = nodes[i];
        var src = srcMap.has(n) ? srcMap.get(n) : n.nodeValue;
        var out = src.replace(src.trim(), translate(src.trim()));
        if (out !== n.nodeValue) { srcMap.set(n, src); n.nodeValue = out; }
      }
      var els = root.querySelectorAll ? root.querySelectorAll('[' + ATTRS.join('],[') + ']') : [];
      for (var j = 0; j < els.length; j++) {
        var el = els[j];
        for (var k = 0; k < ATTRS.length; k++) {
          var a = ATTRS[k];
          if (!el.hasAttribute(a)) continue;
          var v = el.getAttribute(a);
          if (!v || !v.trim() || !hasCJK(v)) continue;
          var tv = translate(v.trim());
          if (tv && tv !== v) {
            if (!el.__xy_src_a) el.__xy_src_a = {};
            if (!(a in el.__xy_src_a)) el.__xy_src_a[a] = v;
            el.setAttribute(a, tv);
          }
        }
      }
    } finally { setTimeout(function () { suppress = false; }, 0); }
  }

  function restore(root) {
    suppress = true;
    try {
      var w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, null);
      var nodes = [];
      while (w.nextNode()) nodes.push(w.currentNode);
      nodes.forEach(function (n) {
        if (srcMap.has(n)) { n.nodeValue = srcMap.get(n); srcMap.delete(n); }
      });
      var els = root.querySelectorAll ? root.querySelectorAll('*') : [];
      for (var j = 0; j < els.length; j++) {
        var el = els[j];
        if (el.__xy_src_a) {
          for (var a in el.__xy_src_a) el.setAttribute(a, el.__xy_src_a[a]);
          delete el.__xy_src_a;
        }
      }
    } finally { setTimeout(function () { suppress = false; }, 0); }
  }

  /* ============ Observer：捕获动态渲染内容 ============ */
  var mo = new MutationObserver(function (muts) {
    if (suppress) return;
    for (var i = 0; i < muts.length; i++) {
      var m = muts[i];
      if (m.type === 'characterData') { applyToText(m.target); continue; }
      if (m.type === 'attributes') { applyAttrs(m.target); continue; }
      for (var j = 0; j < m.addedNodes.length; j++) {
        var nd = m.addedNodes[j];
        if (nd.nodeType === 3) applyToText(nd);
        else if (nd.nodeType === 1) { apply(nd); }
      }
    }
  });

  function applyToText(n) {
    if (current === 'zh-CN' || !current || !n.nodeValue || !n.nodeValue.trim()) return;
    var src = srcMap.has(n) ? srcMap.get(n) : n.nodeValue;
    var tr = translate(src.trim());
    var out = src.replace(src.trim(), tr);
    if (out !== n.nodeValue) { srcMap.set(n, src); n.nodeValue = out; }
  }
  function applyAttrs(el) {
    if (current === 'zh-CN' || !current || el.nodeType !== 1) return;
    for (var k = 0; k < ATTRS.length; k++) {
      var a = ATTRS[k];
      if (!el.hasAttribute(a)) continue;
      var v = el.getAttribute(a);
      if (!v || !v.trim() || !hasCJK(v)) continue;
      var tv = translate(v.trim());
      if (tv && tv !== v) {
        if (!el.__xy_src_a) el.__xy_src_a = {};
        if (!(a in el.__xy_src_a)) el.__xy_src_a[a] = v;
        el.setAttribute(a, tv);
      }
    }
  }

  /* ============ 语言切换 UI ============ */
  var OPTS = [
    { v: 'auto', label: { 'zh-CN': '跟随系统', 'zh-TW': '跟隨系統', 'en': 'System' } },
    { v: 'zh-CN', label: { 'zh-CN': '简体中文', 'zh-TW': '简体中文', 'en': '简体中文' } },
    { v: 'zh-TW', label: { 'zh-CN': '繁體中文', 'zh-TW': '繁體中文', 'en': '繁體中文' } },
    { v: 'en', label: { 'zh-CN': 'English', 'zh-TW': 'English', 'en': 'English' } }
  ];
  function optLabel(o) { return o.label[current] || o.label['zh-CN']; }

  function buildMenu(mount, mode) {
    var wrap = document.createElement('div');
    wrap.className = 'lang-switch lang-switch--' + mode;
    var list = document.createElement('div');
    list.className = 'lang-menu';
    OPTS.forEach(function (o) {
      var b = document.createElement('button');
      b.type = 'button';
      b.className = 'lang-menu__item';
      b.dataset.lang = o.v;
      b.textContent = optLabel(o);
      b.addEventListener('click', function (e) {
        e.stopPropagation();
        setLang(o.v);
        if (mode === 'dropdown') wrap.classList.remove('open');
      });
      list.appendChild(b);
    });
    if (mode === 'dropdown') {
      var btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'lang-switch__btn';
      btn.setAttribute('aria-label', 'Language');
      btn.innerHTML = '<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.7 2.6 4 5.6 4 9s-1.3 6.4-4 9c-2.7-2.6-4-5.6-4-9s1.3-6.4 4-9z"/></svg>';
      btn.addEventListener('click', function (e) { e.stopPropagation(); wrap.classList.toggle('open'); });
      document.addEventListener('click', function () { wrap.classList.remove('open'); });
      wrap.appendChild(btn);
      wrap.appendChild(list);
    } else {
      wrap.classList.add('lang-switch--inline');
      while (list.firstChild) { var item = list.firstChild; list.removeChild(item); wrap.appendChild(item); }
    }
    mount.appendChild(wrap);
    refreshMenu(wrap);
    return wrap;
  }
  function refreshMenu(wrap) {
    wrap.querySelectorAll('.lang-menu__item').forEach(function (b) {
      var o = OPTS.filter(function (x) { return x.v === b.dataset.lang; })[0];
      if (o) { b.textContent = optLabel(o); b.classList.toggle('active', o.v === pref); }
    });
  }

  /* ============ API ============ */
  function setLang(v) {
    pref = v;
    try { localStorage.setItem(LS_KEY, v); } catch (e) { }
    current = resolve(v);
    document.documentElement.lang = current === 'zh-CN' ? 'zh-CN' : (current === 'zh-TW' ? 'zh-TW' : 'en');
    apply(document.body || document.documentElement);
    document.querySelectorAll('.lang-switch').forEach(refreshMenu);
    try { window.dispatchEvent(new CustomEvent('xylangchange', { detail: { lang: current, pref: pref } })); } catch (e) { }
  }
  function getLang() { return current; }
  window.XYI18N = { setLang: setLang, getLang: getLang, t: translate };

  /* ============ 启动 ============ */
  pref = stored() || 'auto';
  current = resolve(pref);
  document.documentElement.lang = current === 'zh-CN' ? 'zh-CN' : (current === 'zh-TW' ? 'zh-TW' : 'en');

  function boot() {
    if (document.body) apply(document.body);
    document.querySelectorAll('[data-i18n-mount]').forEach(function (m) {
      buildMenu(m, m.dataset.i18nMount === 'inline' ? 'inline' : 'dropdown');
    });
    mo.observe(document.body || document.documentElement, {
      childList: true, subtree: true, characterData: true,
      attributes: true, attributeFilter: ATTRS
    });
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
  else boot();
})();

</script>
</body>
</html>
"#;