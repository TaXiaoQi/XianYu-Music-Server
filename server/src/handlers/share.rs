use axum::response::Response;
use rand::Rng;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};

use crate::handlers::helpers::{int_of, parse_body, str_of};
use crate::response::ReqCtx;

/// 分享有效期下限（分钟）：5 分钟
const SHARE_MIN_MINUTES: i64 = 5;
/// 分享有效期上限（分钟）：24 小时
const SHARE_MAX_MINUTES: i64 = 24 * 60;
/// 分享默认有效期（分钟）：2 小时
const SHARE_DEFAULT_MINUTES: i64 = 120;
/// 条幅分享短码长度
const SHARE_ID_LEN: usize = 8;

/// 手写 URL 组件百分号编码（deep link 里歌名/歌手含中文与空格）
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

/// 封面 URL 转缩略图 URL（走 /uploads/covers ?w=150 实时缩放），
/// 深链传给客户端弹窗用小图，秒开且省流量。
/// 仅对本站 /uploads/covers/ 封面生效；第三方插件 CDN 封面原样返回，
/// 避免拼坏外链 query。
fn cover_thumb_url(cover: &str) -> String {
    if cover.is_empty() || !cover.contains("/uploads/covers/") {
        return cover.to_string();
    }
    let sep = if cover.contains('?') { '&' } else { '?' };
    format!("{cover}{sep}w=150")
}

/// 构造唤起 App 的深链（带歌曲元数据与封面缩略图，App 端弹分享预览窗并按音源播放）
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

/// HTML 属性/文本转义，用于安全地把动态值注入 <meta> 标签
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

/// 生成一个随机短码候选项。同步函数：内部自建 ThreadRng 并立即 drop，
/// 严禁将 !Send 的 ThreadRng 持有跨 await，否则会污染上层 future 的 Send 约束。
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

/// 生成短分享码，并保证在 share_log 中唯一
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
            Err(_) => return id, // 读库异常时直接返回，避免死循环
        }
    }
}

/// 上报一次真实「用户点分享」动作（切歌预生成 share_log 不触发本接口），
/// 供仪表台分享统计使用，避免被预加载刷虚高。
pub async fn report_share_action(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data = parse_body(body);
    let ciyuanxi_id = str_of(&data, "ciyuanxi_id").to_string();
    match sqlx::query("INSERT INTO share_actions (ciyuanxi_id) VALUES (?)")
        .bind(&ciyuanxi_id)
        .execute(pool)
        .await
    {
        Ok(_) => ctx.ok("ok", json!({})),
        Err(e) => ctx.err(500, &format!("上报失败: {}", e)),
    }
}

/// 校验封面 URL 是否可被外部访问：仅接受远程 http(s)，拒绝本地/回环/Tauri asset 地址，
/// 避免本地封面路径或混合内容进入落地页（导致「不安全」提示与封面无法加载）。
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

/// 创建歌曲分享记录（客户端在播放时预生成，落地页不做网页播放，仅拉起客户端）
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
    // 分享有效期（分钟）：客户端「分享链接有效时长」设置传入，下限 5min、上限 24h，缺省 2h
    let raw_ttl = int_of(&data, "expire_minutes");
    let ttl_min = if (SHARE_MIN_MINUTES..=SHARE_MAX_MINUTES).contains(&raw_ttl) {
        raw_ttl
    } else {
        SHARE_DEFAULT_MINUTES
    };
    let deep_link =
        build_song_deep_link(&song_id, &hash, &song_name, &singer, duration_ms, &source, &cover);

    let share_id = gen_unique_share_id(pool).await;
    // 分享链接优先拼配置的独立分享域名（share.xianyumusic.cn），
    // 未配置则回退用请求 Host（即客户端连接的 api 域名下的 /s/{id}）。
    let share_base = ctx.config.share_base_url.trim().trim_end_matches('/');
    let share_url = if share_base.is_empty() {
        format!("{}/s/{}", ctx.base_url.trim_end_matches('/'), share_id)
    } else {
        format!("{}/s/{}", share_base, share_id)
    };

    // request_params 保留原始请求体，落地页据此重建 deep_link
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
        Err(e) => ctx.err(500, &format!("创建分享失败: {}", e)),
    }
}

/// 落地页 __SHARE_DATA__ 所需歌曲信息（cover 需传入绝对 https 地址，与 og:image 一致）
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
        // 用于拼接 App 唤醒失败后的下载地址的上前缀：{base}/api?action=share_download
        "download_api": download_api,
    })
}

/// 渲染分享落地页（自包含单文件，无网页播放：仅拉客户端 + 无客户端引导去官方下载）
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
    // 描述文案
    let og_desc = "这首歌曲来自弦予音乐，点击卡片即可在 App 内收听全曲";

    // 封面统一做绝对化：既用于 og:image，也作为落地页展示封面。
    // 本地/回环/asset 地址（旧数据可能残留）回退站点 logo；
    // http(s) 直接复用并强制 https，避免 HTTPS 落地页加载 HTTP 封面触发混合内容「不安全」提示。
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
    // 落地页 <title>：带歌曲名，QQ/微信卡片在缺 og:title 时会回退读 title
    let page_title = if og_title.is_empty() {
        "弦予音乐 · 分享".to_string()
    } else {
        format!("{} · 弦予音乐", og_title)
    };

    // 落地页展示封面与 og:image 保持一致，避免本地/相对封面在页面与卡片里显示不一致
    let data = row_to_share_data(row, body_params, download_api, &cover_abs);
    let json_str = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());

    HTML
        .replace("__SHARE_JSON__", &json_str)
        .replace("__TITLE__", &html_escape(&page_title))
        .replace("__OG_TITLE__", &html_escape(&og_title))
        .replace("__OG_DESC__", &html_escape(og_desc))
        .replace("__OG_IMAGE__", &html_escape(&cover_abs))
        .replace("__OG_URL__", &html_escape(&og_url))
}

const HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
<title>__TITLE__</title>
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
</style>
</head>
<body>
<script>
window.__SHARE_DATA__ = __SHARE_JSON__;
</script>
<div class="bg-layer"><div class="bg-blur"></div><div class="bg-mask"></div></div>
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
    <p id="downloadHint">确认前往官方下载页安装最新版本吗？已安装用户在收到歌曲后即可直接打开收听。</p>
    <div class="row">
      <button class="btn btn-cancel" onclick="hideDownload()">暂不</button>
      <button class="btn btn-download" onclick="goDownload()" id="downloadPrimary">前往下载</button>
    </div>
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

function openApp(){
  var d = window.__SHARE_DATA__ || {};
  showToast('正在打开弦予音乐...');
  var launched = false;
  function markLaunched(){
    launched = true;
    window.removeEventListener('blur', markLaunched);
    document.removeEventListener('visibilitychange', onVisibility);
  }
  function onVisibility(){ if (document.hidden) markLaunched(); }
  // 直接改 location 唤起 scheme：隐藏 iframe 灌 scheme 在 Chrome 桌面端会被拦截，
  // 导致已安装用户被误判为未安装而弹下载引导；blur + visibilitychange 双重检测 App 是否真正拉起
  window.addEventListener('blur', markLaunched);
  document.addEventListener('visibilitychange', onVisibility);
  window.location.href = d.deep_link || 'xianyu://';
  setTimeout(function(){
    if (!launched) showDownload();
  }, 2500);
}
function showDownload(){ document.getElementById('downloadMask').classList.add('show'); }
function hideDownload(){ document.getElementById('downloadMask').classList.remove('show'); }

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
</script>
</body>
</html>
"#;