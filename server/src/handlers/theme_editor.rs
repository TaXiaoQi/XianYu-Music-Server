use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};

use crate::handlers::theme::{persist_new_theme, public_url};
use crate::response::ReqCtx;

/// 槽位清单 v2：客户端版本化契约，未知槽位客户端忽略。
/// 图标槽位（icons）/贴纸槽位（stickers）按平台分组；page 标注归属页面
/// （home/player/main），global 表示跨页面全局生效，客户端按此过滤渲染。
pub const SLOTS_JSON: &str = r#"{
  "version": 2,
  "platforms": {
    "mobile": {
      "pages": [
        { "id": "home", "label": "首页" },
        { "id": "mine", "label": "我的" },
        { "id": "player", "label": "播放页" },
        { "id": "recognize", "label": "听歌识曲" },
        { "id": "search", "label": "在线搜索" },
        { "id": "search_result", "label": "搜索结果" },
        { "id": "settings", "label": "设置页" },
        { "id": "ls-home", "label": "发现" },
        { "id": "ls-mine", "label": "我的" },
        { "id": "ls-player", "label": "播放页" },
        { "id": "ls-local", "label": "本地音乐" },
        { "id": "ls-fav", "label": "我的收藏" },
        { "id": "ls-recent", "label": "最近播放" },
        { "id": "ls-sheets", "label": "我的歌单" },
        { "id": "ls-settings", "label": "设置页" }
      ],
      "icons": [
        { "id": "entry.search", "label": "搜索框 · 放大镜", "page": "global" },
        { "id": "entry.wallpaper", "label": "壁纸/皮肤中心按钮（竖屏首页右上角圆钮）", "page": "home" },
        { "id": "entry.mic", "label": "搜索框 · 识曲钮", "page": "global" },
        { "id": "mine.stat_listen", "label": "我的 · 统计·累计听歌", "page": "mine", "also": ["ls-mine"] },
        { "id": "mine.stat_today", "label": "我的 · 统计·今日时长", "page": "mine", "also": ["ls-mine"] },
        { "id": "mine.stat_count", "label": "我的 · 统计·今日首数", "page": "mine", "also": ["ls-mine"] },
        { "id": "mine.settings", "label": "我的 · 顶栏设置钮", "page": "mine" },
        { "id": "entry.import", "label": "我的 · 导入歌单", "page": "mine" },
        { "id": "mine.grid_favorite", "label": "我的 · 宫格喜欢", "page": "mine", "also": ["ls-mine"] },
        { "id": "mine.grid_recent", "label": "我的 · 宫格最近", "page": "mine", "also": ["ls-mine"] },
        { "id": "mine.grid_local", "label": "我的 · 宫格本地", "page": "mine", "also": ["ls-mine"] },
        { "id": "mine.grid_download", "label": "我的 · 快捷宫格「下载」图标", "page": "mine", "also": ["ls-mine"] },
        { "id": "recognize.mic", "label": "识曲 · 主按钮麦克风", "page": "recognize" },
        { "id": "nav.home", "label": "底部导航 · 首页", "page": "global" },
        { "id": "nav.settings", "label": "底部导航 · 我的", "page": "global" },
        { "id": "player.prev", "label": "播放条 · 上一首", "page": "global", "also": ["player", "ls-player"] },
        { "id": "player.play", "label": "播放条 · 播放/暂停", "page": "global", "also": ["player", "ls-player"] },
        { "id": "player.next", "label": "播放条 · 下一首", "page": "global", "also": ["player", "ls-player"] },
        { "id": "player.queue", "label": "播放页 · 播放队列", "page": "player", "also": ["ls-player"] },
        { "id": "player.mode", "label": "播放页 · 播放模式", "page": "player", "also": ["ls-player"] },
        { "id": "action.favorite", "label": "播放页 · 收藏", "page": "player", "also": ["ls-player"] },
        { "id": "action.download", "label": "播放页 · 下载", "page": "player", "also": ["ls-player"] },
        { "id": "action.share", "label": "播放页 · 分享", "page": "player", "also": ["ls-player"] },
        { "id": "action.more", "label": "播放页 · 更多", "page": "player", "also": ["ls-player"] },
        { "id": "player.speed", "label": "播放页 · 倍速/音效", "page": "player", "also": ["ls-player"] },
        { "id": "player.comment", "label": "播放页 · 评论", "page": "player", "also": ["ls-player"] },
        { "id": "landscape.logo", "label": "横屏 · 侧栏品牌 Logo", "page": "global", "ls": true },
        { "id": "landscape.wallpaper", "label": "横屏 · 顶栏皮肤钮", "page": "global", "ls": true },
        { "id": "landscape.settings", "label": "横屏 · 顶栏设置钮", "page": "global", "ls": true },
        { "id": "lib.drag", "label": "音乐库 · 长按拖拽把手（本地/收藏/最近播放）", "page": "ls-local", "also": ["ls-fav", "ls-recent"] }
      ],
      "stickers": [
        { "id": "recognize.deco", "label": "识曲页 · 底部装饰贴纸", "page": "recognize" },
        { "id": "ls-sidebar.bottom", "label": "横屏 · 侧栏左下角贴纸", "page": "global", "ls": true }
      ],
      "surfaces": [
        { "id": "nav.bar", "label": "底部导航栏", "page": "global" },
        { "id": "mini.bar", "label": "mini 播放条", "page": "global" },
        { "id": "search.box", "label": "搜索框胶囊", "page": "global" },
        { "id": "home.stat", "label": "统计大卡", "page": "home" },
        { "id": "home.song", "label": "歌曲行卡", "page": "home" },
        { "id": "mine.user", "label": "用户卡", "page": "mine", "also": ["ls-mine"] },
        { "id": "mine.stats", "label": "统计卡", "page": "mine", "also": ["ls-mine"] },
        { "id": "mine.grid", "label": "快捷宫格", "page": "mine", "also": ["ls-mine"] },
        { "id": "mine.sheet", "label": "歌单行卡", "page": "mine" },
        { "id": "recognize.hint", "label": "提示卡", "page": "recognize" },
        { "id": "recognize.btn", "label": "识别主按钮", "page": "recognize" },
        { "id": "search.panel", "label": "历史/榜单卡", "page": "search" },
        { "id": "search.item", "label": "榜单行卡", "page": "search" },
        { "id": "sr.chips", "label": "tab/音源条", "page": "search_result" },
        { "id": "sr.pill", "label": "音源胶囊底色（应用到所有来源，文字不变）", "page": "search_result" },
        { "id": "sr.item", "label": "歌曲行卡", "page": "search_result" },
        { "id": "ls-home.daily", "label": "横屏 · 每日推荐", "page": "ls-home" },
        { "id": "ls-home.most", "label": "横屏 · 播放最多", "page": "ls-home" },
        { "id": "ls-lib.row", "label": "横屏 · 歌曲行卡（本地/收藏/最近播放）", "page": "ls-local", "also": ["ls-fav", "ls-recent"] },
        { "id": "ls-sheets.card", "label": "横屏 · 歌单卡", "page": "ls-sheets" },
        { "id": "settings.topbar", "label": "顶栏（返回 + 标题）", "page": "settings" },
        { "id": "settings.group", "label": "设置分组卡", "page": "settings" },
        { "id": "ls-settings.nav", "label": "横屏 · 左侧导航", "page": "ls-settings" },
        { "id": "ls-settings.detail", "label": "横屏 · 详情行卡", "page": "ls-settings" },
        { "id": "ls-mine.count", "label": "横屏 · 数量卡（收藏/歌单/历史）", "page": "ls-mine" }
      ]
    },
    "desktop": {
      "pages": [
        { "id": "main", "label": "主窗口 · 首页" },
        { "id": "playlist", "label": "歌单页" },
        { "id": "player", "label": "播放页" },
        { "id": "local", "label": "本地音乐" },
        { "id": "fav", "label": "我的收藏" },
        { "id": "settings", "label": "设置页" }
      ],
      "icons": [
        { "id": "desktop.logo", "label": "侧栏 · 品牌 Logo", "page": "main" },
        { "id": "nav.home", "label": "侧栏 · 首页" },
        { "id": "nav.settings", "label": "侧栏 · 设置" },
        { "id": "player.prev", "label": "播放 · 上一首" },
        { "id": "player.play", "label": "播放 · 播放/暂停" },
        { "id": "player.next", "label": "播放 · 下一首" },
        { "id": "player.queue", "label": "播放 · 播放队列" },
        { "id": "player.mode", "label": "播放 · 播放模式" },
        { "id": "player.lyric", "label": "播放 · 歌词开关" },
        { "id": "player.comment", "label": "播放 · 评论" },
        { "id": "player.volume", "label": "播放 · 音量" },
        { "id": "player.sound", "label": "播放 · 音效（均衡器）" },
        { "id": "player.mv", "label": "播放 · MV" },
        { "id": "player.visualizer", "label": "播放 · 可视化（频谱）" },
        { "id": "player.progress", "label": "播放 · 进度条开关" },
        { "id": "player.style", "label": "播放 · 页面样式" },
        { "id": "player.pin", "label": "播放 · 固定状态栏" },
        { "id": "action.search", "label": "顶栏 · 搜索" },
        { "id": "action.mic", "label": "顶栏 · 识曲" },
        { "id": "desktop.wallpaper", "label": "顶栏 · 皮肤钮" },
        { "id": "desktop.settings", "label": "顶栏 · 设置钮" },
        { "id": "page.playall", "label": "列表页 · 播放全部钮" },
        { "id": "page.sort", "label": "列表页 · 排序钮" },
        { "id": "page.more", "label": "列表页 · 更多钮" },
        { "id": "page.fav", "label": "歌单/收藏 · 收藏合集钮" },
        { "id": "action.favorite", "label": "操作 · 收藏" },
        { "id": "action.download", "label": "操作 · 下载" },
        { "id": "action.share", "label": "操作 · 分享" },
        { "id": "action.more", "label": "操作 · 更多" },
        { "id": "action.new_playlist", "label": "操作 · 新建歌单" }
      ],
      "stickers": [
        { "id": "player.corner", "label": "右下角贴纸" },
        { "id": "sidebar.bottom", "label": "侧栏底部贴纸" }
      ]
    }
  }
}"#;

pub fn qrcode_svg(data: &str) -> Option<String> {
    use qrcode::render::svg;
    use qrcode::{EcLevel, QrCode, Version};
    if data.is_empty() {
        return None;
    }
    let code = QrCode::with_version(data.as_bytes(), Version::Normal(6), EcLevel::M).ok()?;
    Some(
        code.render::<svg::Color>()
            .dark_color(svg::Color("#111111"))
            .light_color(svg::Color("#ffffff"))
            .quiet_zone(true)
            .build(),
    )
}

fn html_response(html: String) -> Response {
    (
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (axum::http::header::CACHE_CONTROL, "no-cache"),
        ],
        axum::body::Body::from(html),
    )
        .into_response()
}

pub fn render_editor_page() -> Response {
    html_response(EDITOR_HTML.replace("/*__SLOTS_JSON__*/", SLOTS_JSON))
}

/// POST /theme-editor/upload：网页编辑器上传，token 真验（user_tokens 表），
/// 落库与客户端 upload_theme 同一套 persist_new_theme。
pub async fn upload(body: &str, ctx: ReqCtx, pool: &MySqlPool) -> Response {
    let data: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let token = data
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if token.is_empty() {
        return ctx.err(401, "请先扫码登录");
    }
    let owner: Option<String> = sqlx::query_scalar(
        "SELECT ciyuanxi_id FROM user_tokens WHERE token = ? AND expires_at > NOW() LIMIT 1",
    )
    .bind(&token)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let Some(ciyuanxi_id) = owner else {
        return ctx.err(401, "登录状态已失效，请重新扫码登录");
    };
    let user = sqlx::query("SELECT nickname, status FROM app_users WHERE ciyuanxi_id = ? LIMIT 1")
        .bind(&ciyuanxi_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    let Some(user) = user else {
        return ctx.err(401, "账号不存在，请重新登录");
    };
    let status: i64 = user.get("status");
    if status == 0 {
        return ctx.err(403, "账号已被禁用");
    }
    let nickname: String = user
        .try_get::<String, _>("nickname")
        .unwrap_or_else(|_| ciyuanxi_id.clone());

    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let description = data
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let platform = data
        .get("platform")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if !matches!(platform.as_str(), "mobile" | "desktop") {
        return ctx.err(400, "平台仅支持 mobile / desktop");
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
    let payload = match data.get("payload").and_then(|v| v.as_object()) {
        Some(obj) => obj.clone(),
        None => return ctx.err(400, "主题数据不能为空"),
    };
    if payload.get("accentColor").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
        return ctx.err(400, "主题缺少强调色配置");
    }
    let preview = data
        .get("preview")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let wallpaper_id = data
        .get("wallpaperRef")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    match persist_new_theme(
        &ctx,
        pool,
        &ciyuanxi_id,
        &nickname,
        &name,
        &description,
        &platform,
        &payload,
        &preview,
        wallpaper_id,
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

const EDITOR_HTML: &str = r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>弦予 · 主题编辑器</title>
<link rel="icon" type="image/png" href="/logo.png">
<meta name="description" content="弦予音乐主题编辑器：调配强调色、深浅模式与图标贴纸槽位，导出主题包或上传到主题广场。">
<link rel="preconnect" href="https://fonts.loli.net" crossorigin>
<link href="https://fonts.loli.net/css2?family=Inter:wght@300;400;500;600;700;800&family=Noto+Serif+SC:wght@500;600;700&display=swap" rel="stylesheet" media="print" onload="this.media='all'">
<noscript><link href="https://fonts.loli.net/css2?family=Inter:wght@300;400;500;600;700;800&family=Noto+Serif+SC:wght@500;600;700&display=swap" rel="stylesheet"></noscript>
<style>
:root {
  --accent:#ec4141; --accent-dark:#d63333; --accent-soft:rgba(236,65,65,.08);
  --bg:#f9fafb; --bg-soft:#f3f4f6; --card:#ffffff;
  --text:#353A3E; --text-2:#6b7280; --text-3:#9ca3af;
  --border:#e5e7eb; --border-soft:#f0f1f3;
  --radius:12px; --radius-lg:18px; --radius-xl:24px;
  --font-display:'Noto Serif SC','Songti SC',serif;
  --font-body:'Inter',-apple-system,BlinkMacSystemFont,'PingFang SC','Microsoft YaHei',sans-serif;
  --font-mono:'SF Mono','JetBrains Mono',Consolas,monospace;
  --shadow:0 4px 24px rgba(15,23,42,.06); --shadow-lg:0 16px 48px rgba(15,23,42,.10);
  --shadow-accent:0 8px 28px rgba(236,65,65,.25);
}
.srow{display:flex;align-items:center;gap:8px;padding:8px 0;border-top:1px solid var(--border-soft)}
.srow__info{flex:1;min-width:0}
.srow__n{font-size:12px;font-weight:600}
.srow__sub{font-size:10px;color:var(--text-3);margin-top:1px}
.srow input[type=color]{width:30px;height:30px;border:1px solid var(--border);border-radius:8px;padding:2px;background:var(--card);cursor:pointer;flex:none}
.srow input[type=range]{width:64px;flex:none;accent-color:var(--accent)}
.srow__x{width:22px;height:22px;border-radius:6px;border:1px solid var(--border);background:none;color:var(--text-2);cursor:pointer;font-size:11px;line-height:1;flex:none}
.srow__x:hover{color:var(--accent);border-color:var(--accent)}
*,*::before,*::after{ box-sizing:border-box; margin:0; padding:0; }
body{ font-family:var(--font-body); background:var(--bg); color:var(--text); height:100vh; display:flex; flex-direction:column; overflow:hidden; -webkit-font-smoothing:antialiased; }
.bg-fx{ position:fixed; inset:0; z-index:-1; overflow:hidden; pointer-events:none; }
.blob{ position:absolute; border-radius:50%; filter:blur(90px); opacity:.16; }
.blob.b1{ width:560px; height:560px; background:#ec4141; top:-180px; right:-120px; }
.blob.b2{ width:480px; height:480px; background:#f5a623; bottom:-200px; left:-140px; opacity:.10; }
.grid-bg{ position:absolute; inset:0; background-image:linear-gradient(rgba(53,58,62,.035) 1px,transparent 1px),linear-gradient(90deg,rgba(53,58,62,.035) 1px,transparent 1px); background-size:44px 44px; mask-image:radial-gradient(ellipse 90% 60% at 50% 0%,#000 40%,transparent 100%); }

/* ====== 顶栏 ====== */
.topbar{ position:relative; z-index:50; flex:none; background:rgba(255,255,255,.92); backdrop-filter:blur(14px); -webkit-backdrop-filter:blur(14px); border-bottom:1px solid var(--border-soft); }
.topbar__inner{ max-width:1440px; margin:0 auto; padding:14px 28px; display:flex; align-items:center; justify-content:space-between; gap:14px; flex-wrap:wrap; }
.brand{ display:flex; align-items:center; gap:12px; }
.brand__mark{ width:38px; height:38px; border-radius:12px; background:linear-gradient(135deg,#ec4141,#ff7a45); display:flex; align-items:center; justify-content:center; box-shadow:var(--shadow-accent); flex:none; }
.brand__mark svg{ width:20px; height:20px; fill:#fff; }
.brand__name{ font-family:var(--font-display); font-size:19px; font-weight:700; letter-spacing:.5px; }
.brand__sub{ font-size:12.5px; color:var(--text-3); margin-top:2px; }
.topbar__actions{ display:flex; align-items:center; gap:10px; flex-wrap:wrap; }
.login-state{ font-size:13px; color:var(--text-2); display:inline-flex; align-items:center; gap:6px; }
.login-state.on{ color:#0a9d58; font-weight:600; }
.login-state .dot{ width:7px; height:7px; border-radius:50%; background:var(--text-3); }
.login-state.on .dot{ background:#0a9d58; }
.btn{ display:inline-flex; align-items:center; justify-content:center; gap:7px; border-radius:100px; padding:9px 18px; font-size:13.5px; font-weight:600; font-family:var(--font-body); cursor:pointer; border:1px solid var(--border); background:var(--card); color:var(--text); transition:all .25s cubic-bezier(.16,1,.3,1); white-space:nowrap; }
.btn:hover{ border-color:var(--accent); color:var(--accent); transform:translateY(-1px); box-shadow:var(--shadow); }
.btn--primary{ background:var(--accent); border-color:var(--accent); color:#fff; box-shadow:var(--shadow-accent); }
.btn--primary:hover{ background:var(--accent-dark); border-color:var(--accent-dark); color:#fff; }
.btn--ghost{ background:transparent; }
.btn:disabled{ opacity:.55; cursor:not-allowed; transform:none; }
.btn svg{ width:15px; height:15px; }

/* ====== 布局（左右独立滚动） ====== */
.layout{ flex:1; min-height:0; display:flex; gap:24px; position:relative; }
.panel{ flex:0 0 372px; height:100%; overflow-y:auto; overscroll-behavior:contain; display:flex; flex-direction:column; gap:16px; padding:22px 10px 48px 28px; }
.stage{ flex:1; min-width:340px; height:100%; overflow-y:auto; overscroll-behavior:contain; display:flex; flex-direction:column; align-items:center; gap:18px; padding:22px 28px 48px; }
.panel::-webkit-scrollbar,.stage::-webkit-scrollbar{ width:8px; }
.panel::-webkit-scrollbar-track,.stage::-webkit-scrollbar-track{ background:transparent; }
.panel::-webkit-scrollbar-thumb,.stage::-webkit-scrollbar-thumb{ background:#d9dce1; border-radius:4px; }
.panel::-webkit-scrollbar-thumb:hover,.stage::-webkit-scrollbar-thumb:hover{ background:#c3c7cd; }
.card{ background:var(--card); border:1px solid var(--border-soft); border-radius:var(--radius-lg); padding:20px; box-shadow:var(--shadow); }
.card__head{ display:flex; align-items:center; gap:10px; margin-bottom:14px; }
.card__head h3{ font-family:var(--font-display); font-size:15.5px; font-weight:700; }
.card__badge{ font-size:11px; font-weight:700; color:var(--accent); background:var(--accent-soft); border-radius:100px; padding:2px 9px; }
.field{ margin-bottom:14px; }
.field:last-child{ margin-bottom:0; }
.field>label{ display:block; font-size:12.5px; font-weight:600; color:var(--text-2); margin-bottom:7px; }
.field input[type=text],.field textarea{ width:100%; background:var(--bg); border:1px solid var(--border); color:var(--text); border-radius:10px; padding:9px 12px; font-size:13.5px; font-family:var(--font-body); outline:none; transition:border-color .2s, box-shadow .2s; }
.field input[type=text]:focus,.field textarea:focus{ border-color:var(--accent); box-shadow:0 0 0 3px var(--accent-soft); }
.field textarea{ resize:vertical; min-height:58px; line-height:1.6; }
.seg{ display:flex; gap:6px; background:var(--bg); border:1px solid var(--border); border-radius:100px; padding:4px; }
.seg button{ flex:1; border:none; background:transparent; color:var(--text-2); border-radius:100px; padding:7px 8px; font-size:13px; font-weight:600; cursor:pointer; transition:all .2s; white-space:nowrap; }
.seg button:hover{ color:var(--text); }
.seg button.on{ background:var(--accent); color:#fff; box-shadow:0 2px 10px rgba(236,65,65,.35); }
.seg--mini button{ padding:5px 8px; font-size:11.5px; }
.swatches{ display:flex; gap:9px; flex-wrap:wrap; align-items:center; }
.sw{ width:27px; height:27px; border-radius:50%; cursor:pointer; border:none; position:relative; transition:transform .15s; box-shadow:inset 0 0 0 1px rgba(0,0,0,.08); }
.sw:hover{ transform:scale(1.12); }
.sw.on::after{ content:''; position:absolute; inset:-4px; border-radius:50%; border:2px solid var(--text); }
.swatch-more{ display:flex; align-items:center; gap:9px; margin-top:11px; }
.swatch-more input[type=color]{ width:34px; height:34px; border:none; background:none; cursor:pointer; padding:0; }
.accent-hex{ font-family:var(--font-mono); font-size:12px; color:var(--text-2); background:var(--bg); border:1px solid var(--border); border-radius:8px; padding:4px 9px; }
.hint{ font-size:12px; color:var(--text-3); line-height:1.75; margin-top:10px; }
.hint b{ color:var(--text-2); }

/* ====== 槽位行 ====== */
.slot-row{ display:flex; align-items:center; gap:12px; padding:9px 10px; border-radius:12px; transition:background .15s; }
.slot-row:hover{ background:var(--bg); }
.slot-thumb{ width:38px; height:38px; border-radius:11px; background:var(--bg-soft); border:1px solid var(--border-soft); display:flex; align-items:center; justify-content:center; overflow:hidden; flex:none; color:var(--text-2); }
.slot-thumb img{ width:26px; height:26px; object-fit:contain; }
.slot-thumb .ic-svg{ width:19px; height:19px; }
.slot-meta{ flex:1; min-width:0; }
.slot-meta .n{ font-size:13px; font-weight:600; }
.slot-meta .id{ font-size:11px; color:var(--text-3); font-family:var(--font-mono); margin-top:1px; }
.slot-ops{ display:flex; gap:6px; flex:none; }
.slot-ops .btn{ padding:5px 13px; font-size:12px; border-radius:100px; }
.slot-ops .btn.danger{ color:var(--accent); border-color:transparent; background:var(--accent-soft); }
.slot-ops .btn.danger:hover{ border-color:var(--accent); }
.slot-group-title{ font-size:11px; font-weight:700; color:var(--text-3); letter-spacing:.5px; margin:10px 0 2px; padding-left:10px; }

/* ====== 预览区 ====== */
.stage-tabs{ display:flex; gap:6px; flex:none; background:var(--card); border:1px solid var(--border-soft); border-radius:100px; padding:5px; box-shadow:var(--shadow); max-width:100%; overflow-x:auto; scrollbar-width:none; }
.stage-tabs::-webkit-scrollbar{ display:none; }
.stage-tabs button{ border:none; background:transparent; color:var(--text-2); border-radius:100px; padding:7px 20px; font-size:13px; font-weight:600; font-family:var(--font-body); cursor:pointer; transition:all .2s; white-space:nowrap; }
.stage-tabs button:hover{ color:var(--text); }
.stage-tabs button.on{ background:var(--accent); color:#fff; box-shadow:0 2px 10px rgba(236,65,65,.35); }
.device{ background:#1b1d22; padding:11px; box-shadow:var(--shadow-lg), 0 0 0 1px rgba(0,0,0,.35); }
.device--phone{ border-radius:38px; }
.device--desktop{ border-radius:16px; padding:8px; }
.screen{ position:relative; overflow:hidden; border-radius:28px; font-size:12px; transition:background .3s; }
.device--desktop .screen{ border-radius:10px; }
.screen.dark{ --p-bg:#101014; --p-card:#1b1b22; --p-card-2:#23232c; --p-sub:#8b8b98; --p-line:#2a2a33; --p-text:#f2f2f6; }
.screen.light{ --p-bg:#f4f5f9; --p-card:#ffffff; --p-card-2:#f7f8fb; --p-sub:#6b7280; --p-line:#e7e8ee; --p-text:#23252b; }
.screen.fade{ animation:screenIn .35s cubic-bezier(.16,1,.3,1); }
@keyframes screenIn{ from{ opacity:0; transform:translateY(8px) scale(.985);} to{ opacity:1; transform:none;} }
.ic-svg{ width:17px; height:17px; display:block; }
.pv-img{ display:block; object-fit:contain; }

/* —— 移动预览 —— */
.device--phone{ width:320px; max-width:100%; }
/* 预览缩放条（悬浮固定在预览区右下角，不随内容滚动） */
.zoombar{ position:absolute; right:18px; bottom:18px; display:flex; align-items:center; gap:8px; background:var(--card); border:1px solid var(--border-soft); border-radius:100px; padding:6px 14px; box-shadow:var(--shadow); z-index:40; }
.zoombar button{ width:24px; height:24px; border-radius:50%; border:1px solid var(--border-soft); background:var(--card-2, transparent); color:var(--text); cursor:pointer; font-size:13px; line-height:1; display:flex; align-items:center; justify-content:center; padding:0; }
.zoombar button:hover{ border-color:var(--accent); color:var(--accent); }
.zoombar input[type=range]{ width:96px; accent-color:var(--accent); }
.zoombar .pct{ font-size:11px; font-weight:700; color:var(--text-2); min-width:38px; text-align:center; }
.device--phone .screen{ height:694px; display:flex; flex-direction:column; background:var(--p-bg); color:var(--p-text); overflow:hidden; }
.t-word{ font-size:11px; font-weight:600; color:var(--p-text); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.t-sub{ font-size:9px; color:var(--p-sub); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.m-grid{ display:grid; grid-template-columns:1fr 1fr; gap:10px; margin:4px 14px 0; flex:none; }
.m-album .cover{ width:100%; aspect-ratio:1.35; border-radius:12px; box-shadow:0 4px 14px rgba(0,0,0,.14); }
.m-album .name{ font-size:10.5px; margin-top:6px; color:var(--p-text); font-weight:600; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.m-album .sub{ font-size:9px; color:var(--p-sub); margin-top:2px; }
.m-status{ display:flex; justify-content:space-between; padding:10px 18px 2px; font-size:10px; font-weight:600; color:var(--p-sub); }
.m-appbar{ display:flex; align-items:center; justify-content:space-between; padding:12px 16px 8px; }
.m-appbar .t{ font-family:var(--font-display); font-weight:700; font-size:17px; }
.m-appbar .ic-svg{ color:var(--p-accent); }
.m-sticker-top{ position:absolute; top:52px; right:10px; z-index:5; pointer-events:none; }
.m-entries{ display:flex; justify-content:space-around; padding:12px 12px 6px; }
.m-entry{ display:flex; flex-direction:column; align-items:center; gap:6px; font-size:9.5px; color:var(--p-sub); }
.m-entry .bubble{ width:46px; height:46px; display:flex; align-items:center; justify-content:center; background:var(--p-card); border:1px solid var(--p-line); color:var(--p-accent); transition:all .25s; }
.m-entry .bubble img{ width:27px; height:27px; }
.shape-circle .bubble{ border-radius:50%; }
.shape-squircle .bubble{ border-radius:14px; }
.shape-pill .bubble{ border-radius:999px; width:52px; }
.m-list{ padding:8px 14px; display:flex; flex-direction:column; gap:8px; flex:1; min-height:0; overflow:hidden; }
.m-song{ display:flex; align-items:center; gap:11px; background:var(--p-card); border:1px solid var(--p-line); border-radius:12px; padding:9px 11px; }
.m-song .cov{ width:32px; height:32px; border-radius:8px; background:linear-gradient(135deg,var(--p-accent),transparent 130%); flex:none; opacity:.9; }
.m-song .tt{ flex:1; min-width:0; }
.l{ height:6px; border-radius:3px; background:var(--p-line); }
.l.a{ width:62%; background:var(--p-accent); opacity:.5; }
.l.b{ width:42%; margin-top:5px; }
.m-song .ic-svg{ color:var(--p-sub); width:15px; height:15px; }
.m-player{ margin:8px 14px; padding:10px 12px; border-radius:14px; background:var(--p-card); border:1px solid var(--p-line); display:flex; align-items:center; gap:12px; position:relative; overflow:visible; }
.m-player .big{ width:36px; height:36px; border-radius:50%; background:var(--p-accent); color:#fff; display:flex; align-items:center; justify-content:center; flex:none; box-shadow:0 4px 14px color-mix(in srgb,var(--p-accent) 45%, transparent); }
.m-player .big .ic-svg{ width:15px; height:15px; }
.m-player .mini .ic-svg{ color:var(--p-sub); width:15px; height:15px; }
.m-player .fav .ic-svg{ color:var(--p-accent); width:15px; height:15px; }
.m-sticker-corner{ position:absolute; left:-8px; bottom:-10px; z-index:6; pointer-events:none; }
.m-sticker-bot{ position:absolute; right:-6px; bottom:-4px; z-index:6; pointer-events:none; }
.m-navbar{ display:flex; justify-content:space-around; padding:9px 0 14px; background:var(--p-card); border-top:1px solid var(--p-line); }
.m-navbar .ni{ display:flex; flex-direction:column; align-items:center; gap:4px; font-size:9px; color:var(--p-sub); }
.m-navbar .ni.on{ color:var(--p-accent); font-weight:600; }
.m-banner{ margin:4px 14px 2px; height:84px; border-radius:14px; overflow:hidden; flex:none; }
.m-banner img{ width:100%; height:100%; object-fit:cover; display:block; }
.m-banner--ph{ background:linear-gradient(120deg, var(--p-accent), color-mix(in srgb, var(--p-accent) 45%, #1b1d22)); display:flex; align-items:center; justify-content:center; gap:8px; color:rgba(255,255,255,.88); font-size:10.5px; }

/* —— 移动播放页预览 —— */
.p-coverwrap{ position:relative; margin:8px 26px 0; display:flex; justify-content:center; flex:none; }
.p-cover{ width:184px; height:184px; border-radius:20px; background:linear-gradient(135deg, var(--p-accent), color-mix(in srgb, var(--p-accent) 35%, #1b1d22)); box-shadow:0 10px 30px color-mix(in srgb, var(--p-accent) 35%, transparent); }
.p-cover-deco{ position:absolute; top:-10px; right:-2px; z-index:5; pointer-events:none; }
.p-titles{ margin:16px 26px 0; flex:none; text-align:center; }
.p-name{ font-size:15px; font-weight:700; color:var(--p-text); font-family:var(--font-display); }
.p-artist{ font-size:10px; color:var(--p-sub); margin-top:4px; }
.p-progress{ margin:14px 26px 0; flex:none; }
.p-times{ display:flex; justify-content:space-between; font-size:8.5px; color:var(--p-sub); font-family:var(--font-mono); margin-bottom:4px; }
.p-time{ display:block; margin-top:5px; font-size:9px; color:var(--p-sub); text-align:center; font-family:var(--font-mono); }
.p-ctrl{ margin:14px 26px 16px; display:flex; align-items:center; justify-content:space-between; flex:none; }
.p-ctrl .big{ width:50px; height:50px; border-radius:50%; background:var(--p-accent); color:#fff; display:flex; align-items:center; justify-content:center; box-shadow:0 6px 18px color-mix(in srgb,var(--p-accent) 45%, transparent); }
.p-ctrl .mini{ color:var(--p-sub); display:flex; }
.p-ops{ margin:16px 32px 0; display:flex; justify-content:space-between; color:var(--p-sub); flex:none; }
.p-ops .mini{ display:flex; }
.p-corner{ position:absolute; right:10px; bottom:12px; z-index:6; pointer-events:none; }

/* —— 移动端其他页面预览 —— */
.m-search{ margin:6px 14px; height:34px; border-radius:100px; background:var(--p-card); border:1px solid var(--p-line); display:flex; align-items:center; gap:7px; padding:0 12px; color:var(--p-sub); font-size:11px; flex:none; }
.m-search .grow{ flex:1; }
.m-block-title{ margin:10px 16px 4px; font-size:11.5px; font-weight:700; color:var(--p-text); flex:none; }
.m-chips{ display:flex; gap:7px; margin:6px 14px; flex-wrap:wrap; flex:none; }
.m-chip{ padding:5px 12px; border-radius:100px; background:var(--p-card); font-size:10.5px; color:var(--p-sub); }
.m-chip.on{ background:var(--p-accent); color:#fff; }
.m-numrow{ display:flex; align-items:center; gap:10px; margin:0 16px; padding:9px 0; border-bottom:1px solid var(--p-line); flex:none; }
.m-numrow .no{ font-size:11px; font-weight:700; color:var(--p-sub); font-family:var(--font-mono); width:12px; flex:none; }
.m-numrow .grow{ flex:1; }
.mine-head{ display:flex; align-items:center; gap:12px; margin:8px 14px; padding:14px; border-radius:16px; background:linear-gradient(120deg, color-mix(in srgb, var(--p-accent) 22%, var(--p-card)), var(--p-card)); flex:none; }
.mine-head .ava{ width:46px; height:46px; border-radius:50%; background:linear-gradient(135deg, var(--p-accent), color-mix(in srgb, var(--p-accent) 40%, #333)); flex:none; }
.mine-head .grow{ flex:1; }
.mine-vip{ font-size:9px; font-weight:700; color:#7a5a00; background:linear-gradient(120deg,#F7E7B4,#E6C878); border-radius:6px; padding:3px 7px; flex:none; }
.mine-stats{ display:flex; justify-content:space-around; margin:2px 14px 6px; padding:10px 0; border-radius:14px; background:var(--p-card); flex:none; }
.mine-stats b{ display:block; width:26px; height:9px; border-radius:5px; background:var(--p-line); margin:0 auto 5px; }
.mine-stats span{ display:block; width:16px; height:5px; border-radius:3px; background:var(--p-line); opacity:.7; margin:0 auto; }
.mine-stats .num{ font-size:13px; font-weight:700; color:var(--p-text); font-family:var(--font-mono); text-align:center; }
.mine-stats .lbl{ font-size:9px; color:var(--p-sub); margin-top:3px; text-align:center; }
.rec-wrap{ display:flex; flex-direction:column; align-items:center; margin-top:24px; flex:none; }
.rec-orb{ position:relative; width:96px; height:96px; display:flex; align-items:center; justify-content:center; }
.rec-orb::before{ content:''; position:absolute; inset:0; border-radius:50%; background:color-mix(in srgb, var(--p-accent) 22%, transparent); animation:recPulse 1.8s ease-in-out infinite; }
.rec-orb::after{ content:''; position:absolute; inset:13px; border-radius:50%; background:color-mix(in srgb, var(--p-accent) 13%, transparent); animation:recPulse 1.8s ease-in-out .3s infinite; }
@keyframes recPulse{ 0%,100%{ transform:scale(.9); opacity:.5; } 50%{ transform:scale(1.1); opacity:1; } }
.rec-core{ position:relative; z-index:2; width:62px; height:62px; border-radius:50%; background:var(--p-accent); color:#fff; display:flex; align-items:center; justify-content:center; box-shadow:0 8px 24px color-mix(in srgb,var(--p-accent) 45%, transparent); }
.rec-hint{ margin-top:14px; font-size:10.5px; color:var(--p-sub); }
.set-group{ margin:6px 14px; border-radius:14px; background:var(--p-card); border:1px solid var(--p-line); overflow:hidden; flex:none; }
.set-row{ display:flex; align-items:center; gap:10px; padding:11px 14px; border-bottom:1px solid var(--p-line); color:var(--p-text); font-size:11.5px; }
.set-row:last-child{ border-bottom:none; }
.set-row .grow{ flex:1; }
.set-row .ic-svg{ color:var(--p-accent); }
.set-row .val{ font-size:10px; color:var(--p-sub); }
.set-row .arr{ color:var(--p-sub); font-size:15px; line-height:1; font-family:var(--font-body); }
.set-switch{ width:34px; height:20px; border-radius:100px; background:var(--p-accent); position:relative; flex:none; }
.set-switch::after{ content:''; position:absolute; top:2px; right:2px; width:16px; height:16px; border-radius:50%; background:#fff; }
.set-switch.off{ background:var(--p-line); }
.set-switch.off::after{ right:auto; left:2px; }

/* —— 真实排布（对齐客户端截图） —— */
.h-top{ display:flex; align-items:center; gap:9px; padding:8px 14px 4px; flex:none; }
.h-logo{ font-family:var(--font-display); font-weight:800; font-size:16.5px; color:var(--p-text); white-space:nowrap; }
.h-logo em{ font-style:normal; color:var(--p-accent); }
.h-search{ flex:1; min-width:0; height:34px; border-radius:100px; background:var(--p-card); border:1px solid var(--p-line); display:flex; align-items:center; gap:7px; padding:0 6px 0 12px; color:var(--p-sub); font-size:11px; }
.h-search .ph{ flex:1; white-space:nowrap; overflow:hidden; }
.h-search .ic-svg{ color:var(--p-text); opacity:.75; flex:none; }
.h-round{ width:32px; height:32px; border-radius:50%; background:var(--p-card); border:1px solid var(--p-line); color:var(--p-accent); display:flex; align-items:center; justify-content:center; flex:none; }
.stat-card{ margin:8px 14px 0; border-radius:18px; padding:14px 16px 10px; background:linear-gradient(135deg, color-mix(in srgb, var(--p-accent) 26%, var(--p-card)), color-mix(in srgb, var(--p-accent) 55%, var(--p-card))); color:#fff; flex:none; position:relative; }
.stat-card .head{ display:flex; align-items:center; gap:7px; font-size:12px; font-weight:700; }
.stat-card .head .ic-svg{ width:15px; height:15px; }
.stat-card .cap{ margin-top:10px; font-size:10px; opacity:.85; }
.stat-card .big{ font-size:21px; font-weight:800; margin-top:3px; font-family:var(--font-display); }
.stat-card .cols{ display:flex; gap:26px; margin-top:12px; }
.stat-card .cell{ display:flex; align-items:center; gap:6px; font-size:10px; }
.stat-card .cell .ic-svg{ width:13px; height:13px; color:#fff; }
.stat-card .cell b{ display:block; font-size:11.5px; margin-top:1px; }
.stat-card .dot{ width:22px; height:3.5px; border-radius:2px; background:rgba(255,255,255,.85); margin:12px auto 0; }
.sec-h{ display:flex; align-items:center; justify-content:space-between; margin:14px 18px 8px; flex:none; }
.sec-h .t{ font-size:15px; font-weight:800; color:var(--p-text); font-family:var(--font-display); }
.sec-h .t small{ font-size:11px; font-weight:600; color:var(--p-text); margin-left:4px; font-family:var(--font-body); }
.sec-h .act{ font-size:10.5px; color:var(--p-accent); border:1px solid color-mix(in srgb, var(--p-accent) 45%, transparent); border-radius:100px; padding:3px 10px; font-weight:600; }
.song-row{ display:flex; align-items:center; gap:10px; margin:0 14px 8px; padding:8px 10px; border-radius:14px; background:var(--p-card); flex:none; }
.song-row .cov{ width:38px; height:38px; border-radius:9px; flex:none; background:linear-gradient(135deg, var(--p-accent), color-mix(in srgb, var(--p-accent) 35%, #23262e)); }
.song-row .meta{ flex:1; min-width:0; }
.song-row .n{ font-size:12px; font-weight:700; color:var(--p-text); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.song-row .a{ font-size:9.5px; color:var(--p-sub); margin-top:2px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.song-row .cnt{ font-size:9.5px; color:var(--p-sub); flex:none; }
.song-row .play{ width:26px; height:26px; border-radius:50%; background:color-mix(in srgb, var(--p-accent) 16%, transparent); color:var(--p-accent); display:flex; align-items:center; justify-content:center; flex:none; }
.tabbar{ margin:8px 18px 12px; height:56px; border-radius:100px; background:var(--p-card); border:1px solid var(--p-line); display:flex; align-items:center; justify-content:space-around; flex:none; box-shadow:0 6px 18px rgba(0,0,0,.10); }
.tabbar .ti{ display:flex; flex-direction:column; align-items:center; gap:3px; font-size:9px; color:var(--p-sub); width:74px; }
.tabbar .fab{ width:46px; height:46px; border-radius:50%; background:var(--p-accent); color:#fff; display:flex; align-items:center; justify-content:center; margin-top:-18px; box-shadow:0 8px 20px color-mix(in srgb, var(--p-accent) 45%, transparent); }
.tabbar .fab .ic-svg{ width:20px; height:20px; }
.tabbar .ti.on{ color:var(--p-accent); font-weight:700; }
.mini-cov{ width:34px; height:34px; border-radius:50%; flex:none; background:linear-gradient(135deg, var(--p-accent), color-mix(in srgb, var(--p-accent) 30%, #23262e)); }
/* 播放页 */
.pl-ambient{ position:relative; }
.pl-ambient::before{ content:''; position:absolute; inset:-30px; background:radial-gradient(ellipse 65% 42% at 50% 30%, color-mix(in srgb, var(--p-accent) 34%, transparent), transparent 70%); pointer-events:none; }
.pl-top{ display:flex; align-items:center; justify-content:space-between; padding:10px 16px 6px; flex:none; position:relative; }
.pl-seg{ display:flex; background:color-mix(in srgb, var(--p-card) 75%, transparent); border:1px solid var(--p-line); border-radius:100px; padding:3px; }
.pl-seg span{ font-size:10.5px; padding:4px 14px; border-radius:100px; color:var(--p-sub); }
.pl-seg span.on{ background:var(--p-card); color:var(--p-text); font-weight:700; box-shadow:0 2px 8px rgba(0,0,0,.12); }
.pl-cover{ width:224px; height:224px; border-radius:18px; background:linear-gradient(150deg, color-mix(in srgb, var(--p-accent) 70%, #3a3f4a), color-mix(in srgb, var(--p-accent) 28%, #191c22)); box-shadow:0 16px 40px color-mix(in srgb, var(--p-accent) 30%, transparent); }
.pl-title-row{ display:flex; align-items:center; gap:12px; margin:18px 24px 0; flex:none; position:relative; }
.pl-title-row .grow{ flex:1; min-width:0; }
.pl-lyric{ margin:16px 24px 0; display:flex; flex-direction:column; gap:7px; flex:none; position:relative; }
.pl-lyric span{ font-size:11px; color:var(--p-sub); }
.pl-lyric span.on{ color:var(--p-text); font-weight:700; font-size:12px; }
.pl-ops{ display:flex; align-items:center; justify-content:space-between; margin:16px 30px 0; color:var(--p-sub); flex:none; position:relative; }
.pl-ops .mini{ display:flex; flex-direction:column; align-items:center; gap:3px; font-size:8.5px; }
.pl-ops .hq{ font-size:11px; font-weight:800; color:var(--p-text); font-family:var(--font-mono); }
.pl-bar{ margin:14px 22px 0; flex:none; position:relative; }
.pl-bar .track{ height:3px; border-radius:2px; background:color-mix(in srgb, var(--p-text) 16%, transparent); position:relative; }
.pl-bar .track::before{ content:''; position:absolute; left:0; top:0; bottom:0; width:8%; border-radius:2px; background:var(--p-accent); }
/* 我的页 */
.me-card{ display:flex; align-items:center; gap:11px; margin:8px 14px 0; padding:12px 13px; border-radius:16px; background:var(--p-card); flex:none; }
.me-card .ava{ width:42px; height:42px; border-radius:50%; background:linear-gradient(135deg, var(--p-accent), color-mix(in srgb, var(--p-accent) 40%, #333)); flex:none; }
.me-card .grow{ flex:1; min-width:0; }
.me-card .n{ font-size:13px; font-weight:800; color:var(--p-text); font-family:var(--font-display); }
.me-card .s{ font-size:9.5px; color:var(--p-sub); margin-top:2px; }
.me-card .ic-svg{ color:var(--p-text); opacity:.6; }
.me-stats{ display:flex; margin:8px 14px 0; padding:12px 0; border-radius:16px; background:var(--p-card); flex:none; }
.me-stats .cell{ flex:1; display:flex; flex-direction:column; align-items:center; gap:3px; }
.me-stats .ic-svg{ color:var(--p-accent); width:15px; height:15px; }
.me-stats .v{ font-size:11px; font-weight:800; color:var(--p-text); }
.me-stats .l{ font-size:8.5px; color:var(--p-sub); }
.me-grid{ display:flex; margin:8px 14px 0; padding:13px 0 10px; border-radius:16px; background:var(--p-card); flex:none; }
.me-grid .cell{ flex:1; display:flex; flex-direction:column; align-items:center; gap:5px; }
.me-grid .bub{ width:38px; height:38px; border-radius:50%; background:color-mix(in srgb, var(--p-accent) 14%, transparent); color:var(--p-accent); display:flex; align-items:center; justify-content:center; transition:border-radius .25s; }
.shape-squircle .me-grid .bub{ border-radius:11px; }
.shape-pill .me-grid .bub{ border-radius:999px; width:46px; }
.me-grid .n{ font-size:9.5px; color:var(--p-text); font-weight:600; }
.me-grid .c{ font-size:8.5px; color:var(--p-sub); }
.pl-row{ display:flex; align-items:center; gap:10px; margin:0 14px 8px; padding:9px 11px; border-radius:14px; background:var(--p-card); flex:none; }
.pl-row .cov{ width:40px; height:40px; border-radius:10px; flex:none; background:linear-gradient(135deg, var(--p-accent), color-mix(in srgb, var(--p-accent) 30%, #23262e)); }
.pl-row .grow{ flex:1; min-width:0; }
.pl-row .n{ font-size:12px; font-weight:700; color:var(--p-text); }
.pl-row .s{ font-size:9.5px; color:var(--p-sub); margin-top:2px; }
.pl-row .ic-svg{ color:var(--p-sub); }
.set-sec{ margin:10px 16px 4px; font-size:11px; font-weight:800; color:var(--p-accent); flex:none; }
.set-card{ margin:0 14px; border-radius:16px; background:var(--p-card); overflow:hidden; flex:none; }
.set-card .row{ display:flex; align-items:center; gap:11px; padding:11px 14px; border-bottom:1px solid var(--p-line); }
.set-card .row:last-child{ border-bottom:none; }
.set-card .row .ic-svg{ color:var(--p-text); opacity:.85; flex:none; }
.set-card .row .grow{ flex:1; min-width:0; }
.set-card .row .n{ font-size:12px; font-weight:700; color:var(--p-text); }
.set-card .row .s{ font-size:9.5px; color:var(--p-sub); margin-top:2px; }
/* 识曲/搜索/搜索结果（对齐截图） */
.nav-round{ width:38px; height:38px; border-radius:50%; background:var(--p-card); border:1px solid var(--p-line); display:flex; align-items:center; justify-content:center; flex:none; box-shadow:0 2px 8px rgba(0,0,0,.06); }
.nav-round .ic-svg{ color:var(--p-text); }
.pill-title{ display:flex; align-items:center; gap:7px; background:var(--p-card); border:1px solid var(--p-line); border-radius:100px; padding:9px 16px; font-size:14px; font-weight:800; color:var(--p-text); font-family:var(--font-display); }
.pill-title .ic-svg{ color:var(--p-accent); }
.rec-hero{ display:flex; flex-direction:column; align-items:center; margin-top:16px; flex:none; }
.rec-rings{ width:150px; height:150px; border-radius:50%; background:color-mix(in srgb, var(--p-accent) 9%, transparent); display:flex; align-items:center; justify-content:center; }
.rec-rings .mid{ width:104px; height:104px; border-radius:50%; background:color-mix(in srgb, var(--p-accent) 18%, transparent); display:flex; align-items:center; justify-content:center; }
.rec-rings .core{ width:62px; height:62px; border-radius:50%; background:var(--p-accent); color:#fff; display:flex; align-items:center; justify-content:center; box-shadow:0 6px 18px color-mix(in srgb, var(--p-accent) 45%, transparent); }
.rec-cta{ margin-top:16px; font-size:14.5px; font-weight:800; color:var(--p-text); font-family:var(--font-display); }
.rec-dash{ width:34px; height:4px; border-radius:2px; background:var(--p-line); margin-top:12px; }
.rec-tips{ margin:16px 16px 0; padding:3px 14px; border-radius:14px; background:color-mix(in srgb, var(--p-accent) 7%, var(--p-card)); flex:none; }
.rec-tips .row{ display:flex; align-items:center; gap:9px; padding:9px 0; font-size:10.5px; color:var(--p-text); }
.rec-tips .row + .row{ border-top:1px solid var(--p-line); }
.rec-tips .row .ic-svg{ color:var(--p-accent); flex:none; }
.hist-card{ margin:10px 14px 0; padding:12px 14px; border-radius:14px; background:color-mix(in srgb, var(--p-accent) 6%, var(--p-card)); flex:none; }
.hist-card .h{ display:flex; align-items:center; gap:7px; font-size:12.5px; font-weight:800; color:var(--p-text); }
.hist-card .h .ic-svg{ color:var(--p-text); opacity:.7; }
.hist-card .empty{ margin-top:8px; font-size:10.5px; color:var(--p-sub); }
.hot-head{ display:flex; align-items:center; gap:7px; margin:14px 18px 4px; font-size:13.5px; font-weight:800; color:var(--p-text); font-family:var(--font-display); }
.hot-head .ic-svg{ color:var(--p-accent); }
.hot-row{ display:flex; align-items:center; gap:12px; padding:9px 18px; }
.hot-row .no{ width:16px; font-size:13px; font-weight:800; font-family:var(--font-mono); color:var(--p-sub); text-align:center; flex:none; }
.hot-row .no.top{ color:var(--p-accent); }
.hot-row .kw{ flex:1; min-width:0; font-size:12.5px; color:var(--p-text); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.hot-row.top .kw{ font-weight:800; font-size:13.5px; }
.hot-row .cnt{ font-size:9.5px; color:var(--p-sub); flex:none; }
.seg-tabs{ display:flex; gap:26px; margin:10px 18px 0; border-bottom:1px solid var(--p-line); flex:none; }
.seg-tabs span{ font-size:13px; padding:6px 2px 9px; color:var(--p-sub); position:relative; }
.seg-tabs span.on{ color:var(--p-accent); font-weight:800; }
.seg-tabs span.on::after{ content:''; position:absolute; left:50%; transform:translateX(-50%); bottom:0; width:22px; height:3px; border-radius:2px; background:var(--p-accent); }
.src-chips{ display:flex; gap:8px; margin:10px 14px 0; overflow:hidden; flex:none; }
.src-chip{ flex:none; font-size:11px; padding:7px 14px; border-radius:100px; background:var(--p-card); border:1px solid var(--p-line); color:var(--p-sub); }
.src-chip.on{ color:var(--p-accent); font-weight:700; background:color-mix(in srgb, var(--p-accent) 8%, var(--p-card)); border-color:color-mix(in srgb, var(--p-accent) 35%, transparent); }
.res-row{ display:flex; align-items:center; gap:10px; margin:0 14px; padding:8px 0; }
.res-row .cov{ width:44px; height:44px; border-radius:10px; flex:none; background:linear-gradient(135deg, var(--p-accent), color-mix(in srgb, var(--p-accent) 35%, #23262e)); }
.res-row .grow{ flex:1; min-width:0; }
.res-row .n{ font-size:12.5px; font-weight:700; color:var(--p-text); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.res-row .s{ font-size:9.5px; color:var(--p-sub); margin-top:3px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.res-row .fav,.res-row .more{ color:var(--p-sub); flex:none; display:flex; }
.res-row .dur{ font-size:10px; color:var(--p-sub); font-family:var(--font-mono); flex:none; }
.sticker-img{ filter:drop-shadow(0 3px 8px rgba(0,0,0,.18)); }
/* 内容滚动区：状态栏/mini 条/底部导航固定，中间内容可滚（对齐真实 App 布局） */
.body-scroll{ flex:1 1 0; min-height:0; overflow-y:auto; scrollbar-width:none; }
.body-scroll::-webkit-scrollbar{ display:none; }

/* —— 桌面预览 —— */
.device--desktop{ width:660px; max-width:100%; }
.device--desktop .screen{ height:440px; display:flex; flex-direction:column; background:var(--p-bg); color:var(--p-text); }
/* 顶栏：返回圆钮 + 大搜索框 + 功能图标组 + Windows 窗口键（对齐桌面版截图） */
.d-topbar{ display:flex; align-items:center; gap:10px; padding:9px 12px 7px; }
.d-backbtn{ width:26px; height:26px; border-radius:50%; background:var(--p-card); border:1px solid var(--p-line); display:flex; align-items:center; justify-content:center; color:var(--p-text); flex:none; }
.d-searchbar{ flex:1; height:30px; border-radius:999px; background:var(--p-card); border:1px solid var(--p-line); display:flex; align-items:center; padding:0 12px; color:var(--p-sub); font-size:11px; gap:8px; min-width:0; }
.d-searchbar .grow{ flex:1; }
.d-tbtn{ color:var(--p-sub); display:flex; }
.d-tbtn .ic-svg{ width:15px; height:15px; }
.d-ava{ width:22px; height:22px; border-radius:50%; background:linear-gradient(135deg,var(--p-accent),color-mix(in srgb,var(--p-accent) 35%, #fff)); flex:none; }
.d-wsep{ width:1px; height:14px; background:var(--p-line); margin:0 3px; }
.d-wbtn{ width:24px; height:22px; display:flex; align-items:center; justify-content:center; color:var(--p-sub); }
.d-wbtn svg{ width:10px; height:10px; stroke:currentColor; stroke-width:1.3; fill:none; }
.d-shell{ flex:1; display:flex; min-height:0; }
/* —— 移动端横屏（复用桌面壳）—— */
.ls-cap{ font-size:9.5px; color:var(--p-sub); margin:10px 6px 3px; font-weight:700; letter-spacing:.4px; }
.ls-sec{ display:flex; align-items:baseline; justify-content:space-between; margin:12px 0 8px; }
.ls-sec .t{ font-size:15px; font-weight:800; }
.ls-sec .act{ font-size:10px; color:var(--p-sub); }
.ls-cards{ display:flex; gap:10px; overflow:hidden; }
.ls-card{ width:76px; flex:none; }
.ls-card .cv{ width:76px; height:76px; border-radius:10px; background:var(--p-card); border:1px solid var(--p-line); }
.ls-card .n{ font-size:9.5px; margin-top:4px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.ls-row{ display:flex; align-items:center; gap:10px; padding:5px 0; }
.ls-row .cv{ width:34px; height:34px; border-radius:7px; background:var(--p-card); border:1px solid var(--p-line); flex:none; }
.ls-lyric{ display:flex; flex-direction:column; align-items:center; justify-content:center; gap:14px; text-align:center; }
.ls-lyric span{ font-size:17px; font-weight:700; color:color-mix(in srgb, var(--p-text) 36%, transparent); }
.ls-lyric span.on{ font-size:23px; color:var(--p-text); }
.d-side{ width:150px; background:var(--p-card); border-right:1px solid var(--p-line); padding:8px 8px 6px; display:flex; flex-direction:column; gap:1px; position:relative; }
.d-logo{ display:flex; align-items:center; gap:8px; font-weight:700; font-size:14px; margin:2px 6px 10px; color:var(--p-text); }
.d-nav{ display:flex; align-items:center; gap:9px; padding:6.5px 9px; border-radius:8px; font-size:11.5px; color:var(--p-sub); }
.d-nav .ic-svg{ width:13px; height:13px; }
.d-nav.on{ background:color-mix(in srgb, var(--p-accent) 12%, transparent); color:var(--p-text); font-weight:600; }
.d-pl{ margin-top:auto; display:flex; align-items:center; gap:6px; font-size:10.5px; color:var(--p-sub); padding:7px 9px 4px; border-top:1px solid var(--p-line); }
.d-pl .sp{ flex:1; }
.d-pl .ic-svg{ width:12px; height:12px; }
.d-side .sticker-slot{ position:absolute; bottom:36px; left:0; right:0; display:flex; justify-content:center; pointer-events:none; }
.d-main{ flex:1; display:flex; flex-direction:column; min-width:0; position:relative; padding:10px 16px 0; overflow:hidden; }
.d-tabs{ display:flex; gap:18px; font-size:12.5px; color:var(--p-sub); }
.d-tabs .on{ color:var(--p-accent); font-weight:700; position:relative; }
.d-tabs .on::after{ content:''; position:absolute; left:12%; right:12%; bottom:-6px; height:3px; border-radius:2px; background:var(--p-accent); }
.d-stats{ display:flex; gap:26px; margin:14px 2px 2px; flex-wrap:wrap; row-gap:12px; }
.d-stat .cap{ font-size:10px; color:var(--p-sub); }
.d-stat .big{ font-size:15px; font-weight:800; margin-top:3px; }
.d-stat.hero .big{ font-size:26px; letter-spacing:-.5px; }
.d-rankh{ display:flex; align-items:center; gap:8px; margin:13px 2px 8px; }
.d-rankh .t{ font-weight:700; font-size:12.5px; }
.d-rankh .s{ font-size:9.5px; color:var(--p-sub); }
.d-rankh .sp{ flex:1; }
.d-rankh .ic-svg{ width:12px; height:12px; color:var(--p-sub); }
.d-seg{ display:flex; background:var(--p-card); border:1px solid var(--p-line); border-radius:8px; padding:2px; font-size:10px; gap:2px; }
.d-seg span{ padding:3px 8px; border-radius:6px; color:var(--p-sub); }
.d-seg .on{ background:var(--p-accent); color:#fff; font-weight:600; }
.d-rank{ display:flex; flex-direction:column; gap:7px; }
.d-row{ display:flex; align-items:center; gap:10px; border-radius:10px; padding:7px 12px; background:var(--p-card); border:1px solid var(--p-line); }
.d-row.hot{ background:color-mix(in srgb, var(--p-accent) 7%, var(--p-card)); }
.d-row.me{ border-color:color-mix(in srgb, var(--p-accent) 55%, transparent); }
.d-row .badge{ width:20px; height:20px; border-radius:7px; display:flex; align-items:center; justify-content:center; font-size:10px; font-weight:800; color:var(--p-sub); background:color-mix(in srgb, var(--p-line) 70%, transparent); flex:none; }
.d-row .badge.g1{ background:#F5B83D; color:#fff; }
.d-row .badge.g2{ background:#B8BCC6; color:#fff; }
.d-row .av{ width:24px; height:24px; border-radius:50%; background:linear-gradient(135deg,color-mix(in srgb,var(--p-accent) 45%, transparent),transparent 130%), var(--p-line); flex:none; }
.d-row .grow{ flex:1; min-width:0; }
.d-row .n{ font-size:12px; font-weight:600; display:flex; align-items:center; gap:5px; }
.d-row .you{ font-size:8.5px; color:#fff; background:var(--p-accent); border-radius:4px; padding:1px 4px; font-weight:600; }
.d-row .a{ font-size:9.5px; color:var(--p-sub); margin-top:1px; }
.d-row .val{ font-size:11.5px; font-weight:600; }
.d-sticker-corner{ position:absolute; right:14px; bottom:56px; z-index:6; pointer-events:none; }
/* 底部播放条：左 now playing / 中控制组 / 右工具组，无进度条（对齐截图） */
.d-player{ padding:8px 14px; border-top:1px solid var(--p-line); display:flex; align-items:center; gap:10px; background:var(--p-card); }
.d-now{ display:flex; align-items:center; gap:9px; width:168px; flex:none; min-width:0; }
.d-now .cov{ width:34px; height:34px; border-radius:8px; background:linear-gradient(135deg,var(--p-accent),color-mix(in srgb,var(--p-accent) 30%, #fff7ae)); flex:none; }
.d-now .n{ font-size:11.5px; font-weight:700; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.d-now .a{ font-size:9.5px; color:var(--p-sub); margin-top:1px; }
.d-mid{ flex:1; display:flex; justify-content:center; }
.d-ctl{ display:flex; gap:12px; align-items:center; }
.d-ctl .ic-svg{ color:var(--p-sub); width:14px; height:14px; }
.d-ctl .fav .ic-svg{ color:var(--p-text); width:15px; height:15px; }
.d-ctl .play{ width:30px; height:30px; border-radius:50%; background:var(--p-text); color:var(--p-card); display:flex; align-items:center; justify-content:center; }
.d-ctl .play .ic-svg{ width:12px; height:12px; color:var(--p-card); }
.d-right{ display:flex; gap:11px; align-items:center; flex:none; color:var(--p-sub); }
.d-right .ic-svg{ width:14px; height:14px; }
.d-right .sq{ font-size:9px; border:1px solid var(--p-sub); border-radius:4px; padding:0 3px; font-weight:700; line-height:13px; }
/* —— 桌面 · 列表页（歌单/本地音乐/我的收藏） —— */
.dl-wrap{ flex:1; min-height:0; overflow:hidden; padding:10px 18px 0; display:flex; flex-direction:column; }
.dl-head{ display:flex; gap:16px; margin-bottom:12px; flex:none; }
.dl-cover{ width:104px; height:104px; border-radius:12px; background:linear-gradient(135deg,color-mix(in srgb,var(--p-accent) 60%, transparent),transparent 140%), var(--p-line); flex:none; }
.dl-info{ display:flex; flex-direction:column; justify-content:center; gap:10px; min-width:0; }
.dl-title{ display:flex; align-items:center; gap:8px; font-size:19px; font-weight:800; }
.dl-title .ic-svg{ width:14px; height:14px; color:var(--p-sub); }
.dl-ops{ display:flex; gap:8px; }
.dl-pill{ display:flex; align-items:center; gap:6px; height:26px; padding:0 13px; border-radius:999px; background:var(--p-card); border:1px solid var(--p-line); font-size:11px; font-weight:600; }
.dl-pill .ic-svg{ width:12px; height:12px; color:var(--p-text); }
.dl-round{ width:26px; height:26px; border-radius:50%; background:var(--p-card); border:1px solid var(--p-line); display:flex; align-items:center; justify-content:center; color:var(--p-text); }
.dl-round .ic-svg{ width:12px; height:12px; }
.dl-pagebar{ display:flex; align-items:center; gap:8px; margin-bottom:8px; flex:none; }
.dl-pagebar .t{ font-size:16px; font-weight:800; }
.dl-pagebar .sp{ flex:1; }
.dl-tools{ display:flex; gap:6px; }
.dl-tools .dl-round{ width:24px; height:24px; color:var(--p-sub); }
.dl-tools .dl-round .ic-svg{ width:11px; height:11px; }
.dl-tabs{ display:flex; gap:16px; font-size:13px; color:var(--p-sub); }
.dl-tabs .on{ color:var(--p-text); font-weight:800; position:relative; }
.dl-tabs .on::after{ content:''; position:absolute; left:10%; right:10%; bottom:-5px; height:3px; border-radius:2px; background:var(--p-accent); }
.dl-body{ flex:1; min-height:0; overflow-y:auto; scrollbar-width:none; }
.dl-body::-webkit-scrollbar{ display:none; }
.dl-row{ display:grid; grid-template-columns:24px 36px minmax(0,1.35fr) minmax(0,.9fr) 40px 22px 44px 46px; gap:10px; align-items:center; padding:6px 8px; border-radius:9px; }
.dl-row.hl{ background:color-mix(in srgb, var(--p-text) 7%, transparent); }
.dl-row .no{ font-size:10.5px; color:var(--p-sub); font-family:var(--font-mono); }
.dl-row .eq{ display:flex; gap:2px; align-items:flex-end; height:12px; }
.dl-row .eq i{ width:2.5px; border-radius:1px; background:var(--p-accent); }
.dl-row .eq i:nth-child(1){ height:6px; } .dl-row .eq i:nth-child(2){ height:12px; } .dl-row .eq i:nth-child(3){ height:8px; }
.dl-row .drag{ font-size:11px; color:var(--p-sub); letter-spacing:-1px; }
.dl-row .cov{ width:36px; height:36px; border-radius:7px; background:linear-gradient(135deg,color-mix(in srgb,var(--p-accent) 42%, transparent),transparent 135%), var(--p-line); }
.dl-row .tt{ display:flex; align-items:baseline; gap:6px; min-width:0; }
.dl-row .n{ font-size:12px; font-weight:600; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.dl-row .dur{ font-size:10px; color:var(--p-sub); white-space:nowrap; }
.dl-row .a{ font-size:10.5px; color:var(--p-sub); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.dl-row .al{ font-size:10.5px; color:var(--p-sub); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
.dl-row .fmt{ font-size:10px; color:var(--p-sub); }
.dl-row .heart{ color:var(--p-sub); display:flex; }
.dl-row .heart .ic-svg{ width:13px; height:13px; }
.dl-row .heart.on{ color:var(--p-accent); }
.dl-row .d{ font-size:10.5px; color:var(--p-sub); font-family:var(--font-mono); }
.dl-row .tag{ font-size:9px; color:var(--p-accent); border:1px solid color-mix(in srgb,var(--p-accent) 45%, transparent); background:color-mix(in srgb,var(--p-accent) 9%, transparent); border-radius:5px; padding:1px 5px; font-weight:700; justify-self:start; }
/* —— 桌面 · 播放页（全屏沉浸） —— */
.dp{ flex:1; min-height:0; position:relative; display:flex; flex-direction:column; align-items:center; overflow:hidden; }
.dp::before{ content:''; position:absolute; inset:-30%; background:
  radial-gradient(45% 55% at 28% 38%, color-mix(in srgb,var(--p-accent) 52%, transparent), transparent 70%),
  radial-gradient(50% 60% at 75% 65%, color-mix(in srgb,var(--p-accent) 34%, transparent), transparent 72%),
  radial-gradient(40% 45% at 60% 20%, color-mix(in srgb,#8a5a3a 30%, transparent), transparent 70%); filter:blur(30px); }
.dp > *{ position:relative; }
.dp-name{ margin-top:16px; font-size:15px; font-weight:800; color:#fff; text-shadow:0 1px 8px rgba(0,0,0,.25); }
.dp-artist{ font-size:10px; color:rgba(255,255,255,.72); margin-top:4px; }
.dp-body{ flex:1; display:flex; align-items:center; gap:34px; padding:12px 44px 0; min-height:0; }
.dp-cover{ width:148px; height:148px; border-radius:10px; background:linear-gradient(135deg,rgba(255,255,255,.35),rgba(255,255,255,.08)), rgba(0,0,0,.25); box-shadow:0 14px 40px rgba(0,0,0,.35); position:relative; flex:none; }
.dp-cover::before{ content:''; position:absolute; top:0; left:0; right:0; height:14px; border-radius:10px 10px 0 0; background:linear-gradient(rgba(255,255,255,.55),rgba(255,255,255,.35)); }
.dp-lyric{ flex:1; min-width:0; display:flex; flex-direction:column; gap:12px; }
.dp-lyric .prev{ font-size:17px; font-weight:700; color:rgba(255,255,255,.30); line-height:1.35; filter:blur(1.5px); }
.dp-lyric .cur{ font-size:21px; font-weight:800; color:#fff; line-height:1.3; text-shadow:0 1px 10px rgba(0,0,0,.2); }
.dp-lyric .trans{ font-size:11.5px; color:rgba(255,255,255,.55); margin-top:-6px; }
/* 底部控制区：细进度线 + 控制条 */
.dp-prog{ width:calc(100% - 60px); height:2px; border-radius:1px; background:rgba(255,255,255,.28); margin:0 auto; position:relative; flex:none; }
.dp-prog::before{ content:''; position:absolute; left:0; top:0; bottom:0; width:30%; border-radius:1px; background:rgba(255,255,255,.85); }
.dp-ctrl{ width:100%; display:flex; align-items:center; gap:14px; padding:9px 22px 13px; flex:none; color:rgba(255,255,255,.9); }
.dp-time{ font-family:var(--font-mono); font-size:11.5px; font-weight:700; color:#fff; }
.dp-time .tot{ color:rgba(255,255,255,.55); font-weight:500; }
.dp-ctrl .ic-svg{ width:15px; height:15px; }
.dp-ctrl .ok{ width:15px; height:15px; border-radius:50%; border:1.5px solid #3dbd7d; display:flex; align-items:center; justify-content:center; color:#3dbd7d; font-size:9px; font-weight:800; }
.dp-ctrl .grow{ flex:1; }
.dp-ctrl .play{ width:38px; height:38px; border-radius:50%; background:rgba(255,255,255,.22); color:#fff; display:flex; align-items:center; justify-content:center; backdrop-filter:blur(4px); }
.dp-ctrl .play .ic-svg{ width:14px; height:14px; color:#fff; }
.dp-ctrl .word{ font-size:13px; font-weight:600; }
.dp-ctrl .expand{ width:24px; height:24px; border-radius:50%; background:var(--p-accent); color:#fff; display:flex; align-items:center; justify-content:center; }
.dp-ctrl .expand .ic-svg{ width:13px; height:13px; color:#fff; transform:rotate(180deg); }
.dp .d-sticker-corner{ right:26px; bottom:56px; }
/* —— 桌面 · 设置页（侧栏右侧导航列 + 分组卡片） —— */
.dset{ flex:1; display:flex; min-height:0; }
.ds-nav{ width:150px; flex:none; border-left:1px solid var(--p-line); background:var(--p-bg); padding:10px 10px; display:flex; flex-direction:column; gap:1px; }
.ds-search{ height:26px; border-radius:8px; background:var(--p-card); border:1px solid var(--p-line); display:flex; align-items:center; gap:6px; padding:0 9px; color:var(--p-sub); font-size:10.5px; margin-bottom:10px; }
.ds-search .ic-svg{ width:11px; height:11px; }
.ds-item{ padding:6.5px 10px; border-radius:8px; font-size:11.5px; color:var(--p-sub); position:relative; }
.ds-item.on{ background:color-mix(in srgb, var(--p-text) 8%, transparent); color:var(--p-text); font-weight:600; }
.ds-item.on::before{ content:''; position:absolute; left:0; top:22%; bottom:22%; width:3px; border-radius:2px; background:var(--p-accent); }
.ds-main{ flex:1; min-width:0; overflow-y:auto; scrollbar-width:none; padding:10px 18px 0; }
.ds-main::-webkit-scrollbar{ display:none; }
.ds-sec{ display:flex; align-items:center; gap:7px; font-size:12.5px; font-weight:800; margin:10px 0 8px; }
.ds-sec::before{ content:''; width:3px; height:12px; border-radius:2px; background:var(--p-accent); }
.ds-card{ background:var(--p-card); border:1px solid var(--p-line); border-radius:12px; padding:2px 14px; margin-bottom:14px; }
.ds-row{ display:flex; align-items:center; gap:12px; padding:10px 0; border-bottom:1px solid var(--p-line); }
.ds-row:last-child{ border-bottom:none; }
.ds-row .grow{ flex:1; min-width:0; }
.ds-row .n{ font-size:12px; font-weight:600; }
.ds-row .s{ font-size:10px; color:var(--p-sub); margin-top:2px; line-height:1.45; }
.ds-sel{ display:flex; align-items:center; gap:14px; height:26px; padding:0 10px; border-radius:8px; background:var(--p-bg); border:1px solid var(--p-line); font-size:10.5px; color:var(--p-text); flex:none; }
.ds-sel::after{ content:'⌄'; color:var(--p-sub); font-size:10px; }
.dsw{ width:30px; height:17px; border-radius:999px; background:color-mix(in srgb, var(--p-text) 22%, transparent); position:relative; flex:none; transition:.2s; }
.dsw::after{ content:''; position:absolute; top:2px; left:2px; width:13px; height:13px; border-radius:50%; background:#fff; box-shadow:0 1px 3px rgba(0,0,0,.25); }
.dsw.on{ background:var(--p-accent); }
.dsw.on::after{ left:auto; right:2px; }

/* ====== 弹层 ====== */
.modal{ position:fixed; inset:0; background:rgba(23,26,31,.45); backdrop-filter:blur(4px); display:none; align-items:center; justify-content:center; z-index:100; padding:20px; }
.modal.open{ display:flex; }
.modal .box{ background:var(--card); border:1px solid var(--border-soft); border-radius:var(--radius-xl); padding:26px; width:440px; max-width:100%; max-height:90vh; overflow:auto; box-shadow:var(--shadow-lg); animation:boxIn .3s cubic-bezier(.16,1,.3,1); }
@keyframes boxIn{ from{ opacity:0; transform:translateY(14px) scale(.97);} to{ opacity:1; transform:none;} }
.modal h2{ font-family:var(--font-display); font-size:18px; margin-bottom:16px; }
.qr-wrap{ text-align:center; padding:8px 0; }
.qr-wrap img,.qr-wrap svg{ width:212px; height:212px; background:#fff; border-radius:14px; padding:10px; box-shadow:var(--shadow); }
.qr-tip{ font-size:12.5px; color:var(--text-2); margin-top:12px; line-height:1.8; }
.qr-code-text{ font-family:var(--font-mono); font-size:12px; word-break:break-all; color:var(--text-2); }
.drop{ border:1.5px dashed var(--border); border-radius:14px; padding:22px; text-align:center; color:var(--text-2); font-size:13px; cursor:pointer; transition:all .2s; background:var(--bg); }
.drop:hover{ border-color:var(--accent); color:var(--accent); background:var(--accent-soft); }
.drop img{ max-width:100%; max-height:170px; border-radius:10px; }
.modal-foot{ display:flex; justify-content:flex-end; gap:10px; margin-top:18px; }
#toast{ position:fixed; left:50%; bottom:36px; transform:translateX(-50%); background:#2b2e34; color:#fff; border:none; padding:11px 20px; border-radius:100px; font-size:13px; display:none; z-index:200; max-width:82vw; box-shadow:var(--shadow-lg); }

/* ====== 响应式 ====== */
@media (max-width:1080px){
  body{ height:auto; min-height:100vh; overflow:auto; display:block; }
  .layout{ flex-direction:column; height:auto; }
  .panel{ flex:none; width:100%; height:auto; overflow:visible; padding:22px 28px 0; }
  .stage{ width:100%; height:auto; overflow:visible; padding:22px 28px 48px; }
}
@media (max-width:560px){
  .topbar__inner{ padding-left:16px; padding-right:16px; }
  .panel{ padding-left:16px; padding-right:6px; }
  .stage{ padding-left:16px; padding-right:16px; }
  .device--desktop{ width:100%; }
  .device--desktop .screen{ height:420px; }
  .d-side{ width:128px; }
  .d-now{ width:120px; }
  .zoombar{ display:none; }
}
</style>
</head>
<body>
<div class="bg-fx" aria-hidden="true"><div class="blob b1"></div><div class="blob b2"></div><div class="grid-bg"></div></div>

<header class="topbar">
  <div class="topbar__inner">
    <div class="brand">
      <div class="brand__mark" style="background:none;box-shadow:none;overflow:hidden"><img src="/logo.png" alt="弦予音乐" style="width:100%;height:100%;object-fit:cover"></div>
      <div>
        <div class="brand__name">弦予 · 主题编辑器</div>
        <div class="brand__sub">调配配色与图标，导出或上传广场</div>
      </div>
    </div>
    <div class="topbar__actions">
      <span class="login-state" id="loginState"><span class="dot"></span><span id="loginText">未登录（导出不需要登录）</span></span>
      <button class="btn" id="btnLogin" onclick="openLogin()"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><path d="M14 14h3v3h-3zM18 18h3v3h-3z"/></svg><span>扫码登录</span></button>
      <button class="btn" id="btnLogout" onclick="logout()" style="display:none">退出</button>
      <button class="btn" onclick="exportJson()"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v12m0 0-4-4m4 4 4-4"/><path d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"/></svg><span>导出 JSON</span></button>
      <button class="btn btn--primary" onclick="openUpload()"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 15V3m0 0L8 7m4-4 4 4"/><path d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"/></svg><span>上传广场</span></button>
    </div>
  </div>
</header>

<main class="layout">
  <aside class="panel">
    <div class="card">
      <div class="card__head"><h3>基本信息</h3></div>
      <div class="field"><label>主题名称</label><input type="text" id="themeName" maxlength="64" oninput="state.name=this.value"></div>
      <div class="field"><label>简介（可选）</label><textarea id="themeDesc" maxlength="200" oninput="state.description=this.value"></textarea></div>
      <div class="field"><label>编辑平台（两端各自独立成包）</label>
        <div class="seg" id="platSeg">
          <button data-p="mobile" class="on">移动端</button>
          <button data-p="desktop">桌面端</button>
        </div>
      </div>
      <div class="field" id="oriField" style="display:none"><label>屏幕方向（横竖屏是两套 UI）</label>
        <div class="seg" id="oriSeg">
          <button data-o="portrait" class="on">竖屏</button>
          <button data-o="landscape">横屏</button>
        </div>
      </div>
    </div>

    <div class="card">
      <div class="card__head"><h3>外观</h3></div>
      <div class="field"><label>强调色</label>
        <div class="swatches" id="swatches"></div>
        <div class="swatch-more">
          <input type="color" id="accentPicker" oninput="setAccent(this.value)">
          <span class="accent-hex" id="accentHex"></span>
        </div>
      </div>
      <div class="field"><label>深浅模式</label>
        <div class="seg" id="modeSeg">
          <button data-m="dark" class="on">深色</button>
          <button data-m="light">浅色</button>
        </div>
      </div>
      <div class="field"><label>推荐壁纸 id（可选，广场壁纸编号）</label><input type="text" id="wallpaperRef" placeholder="如 123" oninput="setWallpaperRef(this.value)"></div>
    </div>

    <div class="card">
      <div class="card__head"><h3 id="iconCardTitle">图标槽位</h3><span class="card__badge" id="slotCount">0</span></div>
      <div class="seg seg--mini" id="scopeSeg" style="margin:0 0 10px">
        <button data-scope="page" class="on">本页专属</button>
        <button data-scope="shared">公共通用</button>
      </div>
      <div id="iconSlots"></div>
    </div>

    <div class="card" id="stickerCard">
      <div class="card__head"><h3 id="stickerCardTitle">贴纸槽位</h3></div>
      <div id="stickerSlots"></div>
      <p class="hint" id="slotHint">图标建议 SVG 或高清 PNG（单文件 ≤2MB，主题资源总量 ≤5MB）。未设置的槽位在客户端回落默认图标，未知槽位将被忽略。</p>
    </div>

    <div class="card" id="surfaceCard" style="display:none">
      <div class="card__head"><h3 id="surfaceCardTitle">组件色块</h3></div>
      <div id="surfaceSlots"></div>
      <p class="hint">为组件设置底面色块与透明度（壁纸模式下的自定义材质效果）。颜色同主题色一样可选，透明度 0%~100%；未设置的组件维持默认材质。切换「本页专属 / 公共通用」同时作用于图标、贴纸与组件色块。</p>
    </div>

    <div class="card" id="shapeCard" style="display:none">
      <div class="card__head"><h3>快捷入口形状</h3></div>
      <div class="seg" id="shapeSeg">
        <button data-s="circle" class="on">圆形</button>
        <button data-s="squircle">方圆</button>
        <button data-s="pill">胶囊</button>
      </div>
      <p class="hint" style="margin-top:6px">作用于「我的」页喜欢 / 最近 / 本地 / 下载四个快捷入口的底座形状。</p>
    </div>
  </aside>

  <section class="stage" id="stage">
    <div class="stage-tabs" id="stageTabs"></div>
    <div class="device device--phone" id="mobileWrap"><div class="screen dark" id="mScreen"></div></div>
    <div class="device device--desktop" id="desktopWrap" style="display:none"><div class="screen dark" id="dScreen"></div></div>
  </section>

  <div class="zoombar" id="zoombar">
    <button id="zoomMinus" title="缩小">−</button>
    <input type="range" id="zoomRange" min="0.5" max="2" step="0.05" value="1">
    <button id="zoomPlus" title="放大">＋</button>
    <span class="pct" id="zoomPct">100%</span>
    <button id="zoomReset" title="重置为 100%">⟲</button>
  </div>
</main>

<div class="modal" id="loginModal">
  <div class="box">
    <h2>扫码登录弦予号</h2>
    <div class="qr-wrap">
      <div id="qrBox"><div class="hint" style="text-align:center">正在生成二维码…</div></div>
      <div class="qr-tip">打开弦予音乐 App（移动端）→ 扫一扫，确认登录后本页自动完成登录。<br>也可手动输入授权码：<span class="qr-code-text" id="qrCodeText"></span></div>
    </div>
    <div class="modal-foot"><button class="btn" onclick="closeLogin()">取消</button></div>
  </div>
</div>

<div class="modal" id="uploadModal">
  <div class="box">
    <h2>上传到主题广场</h2>
    <div class="field"><label>主题名称</label><input type="text" id="upName" maxlength="64"></div>
    <div class="field"><label>简介</label><textarea id="upDesc" maxlength="200"></textarea></div>
    <div class="field"><label>预览图（必选，JPG/PNG/WEBP/GIF ≤8MB）</label>
      <div class="drop" id="previewDrop" onclick="document.getElementById('previewFile').click()">点击选择预览图</div>
      <input type="file" id="previewFile" accept="image/jpeg,image/png,image/webp,image/gif" style="display:none" onchange="pickPreview(this)">
    </div>
    <p class="hint" id="upPlatformHint"></p>
    <div class="modal-foot">
      <button class="btn" onclick="closeUpload()">取消</button>
      <button class="btn btn--primary" onclick="doUpload()">确认上传</button>
    </div>
  </div>
</div>
<div id="toast"></div>

<script>
const SLOTS = /*__SLOTS_JSON__*/;
const PRESET_COLORS = ['#EC4141','#E91E63','#9C27B0','#673AB7','#3D5AFE','#00BCD4','#009688','#4CAF50','#FF9800','#FF5722','#795548','#607D8B'];

/* 与客户端默认图标对应的线性 SVG（stroke 风格，currentColor 染色） */
const ICON_PATHS = {
  'nav.home':'<path d="M3 10.5 12 3l9 7.5V20h-6v-6h-6v6H3z"/>',
  'nav.explore':'<circle cx="12" cy="12" r="9"/><path d="M15.5 8.5 13.4 13.4 8.5 15.5l2.1-4.9z"/>',
  'nav.library':'<path d="M5 4v16M10 4v16M14.5 5.5l4.5 14"/>',
  'nav.settings':'<path d="M4 6h9M17 6h3M13 4v4M4 12h3M11 12h9M7 10v4M4 18h11M19 18h1M15 16v4"/>',
  'entry.search':'<circle cx="11" cy="11" r="7"/><path d="M20.5 20.5 16 16"/>',
  'entry.mic':'<rect x="9" y="2.5" width="6" height="11" rx="3"/><path d="M5.5 11a6.5 6.5 0 0 0 13 0M12 17.5V21"/>',
  'entry.import':'<path d="M12 3v10m0 0-4-4m4 4 4-4"/><path d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"/>',
  'player.prev':'<path d="M19 20 9 12l10-8v16z"/><path d="M5 5v14"/>',
  'player.play':'<path d="M7 4.5 20 12 7 19.5z"/>',
  'player.next':'<path d="M5 4l10 8-10 8V4z"/><path d="M19 5v14"/>',
  'player.queue':'<path d="M4 6h16M4 12h16M4 18h10"/>',
  'player.mode':'<path d="M17 2.5 21 6l-4 3.5M21 6H7a4 4 0 0 0-4 4v1M7 21.5 3 18l4-3.5M3 18h14a4 4 0 0 0 4-4v-1"/>',
  'action.favorite':'<path d="M12 20.5s-7.2-4.6-9.3-8.8C1.2 8.4 3.2 5 6.7 5c2.2 0 3.9 1.2 5.3 3 1.4-1.8 3.1-3 5.3-3 3.5 0 5.5 3.4 4 6.7-2.1 4.2-9.3 8.8-9.3 8.8z"/>',
  'action.download':'<path d="M12 3v12m0 0-4.5-4.5M12 15l4.5-4.5"/><path d="M4 19h16"/>',
  'action.share':'<path d="M4 13v6a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-6M16 7l-4-4-4 4M12 3v12"/>',
  'action.more':'<circle cx="5" cy="12" r="1.9"/><circle cx="12" cy="12" r="1.9"/><circle cx="19" cy="12" r="1.9"/>',
  'action.search':'<circle cx="11" cy="11" r="7"/><path d="M20.5 20.5 16 16"/>',
  'action.mic':'<rect x="9" y="2.5" width="6" height="11" rx="3"/><path d="M5.5 11a6.5 6.5 0 0 0 13 0M12 17.5V21"/>',
  'action.new_playlist':'<circle cx="12" cy="12" r="9"/><path d="M12 8v8M8 12h8"/>',
  'desktop.logo':'<circle cx="12" cy="12" r="9"/><path d="M10.2 8.2v7.6l6-3.8z"/>',

  'nav.back':'<path d="M14.5 5.5 8 12l6.5 6.5"/>',
  'ui.moon':'<path d="M20.5 14.5A8.5 8.5 0 0 1 9.5 3.5a8.5 8.5 0 1 0 11 11z"/>',
  'ui.palette':'<circle cx="12" cy="12" r="9"/><circle cx="9" cy="9" r="1.2"/><circle cx="15" cy="9" r="1.2"/><circle cx="12" cy="15" r="1.2"/>',
  'ui.refresh':'<path d="M20 12a8 8 0 1 1-2.34-5.66M20 3v4h-4"/>',
  'ui.info':'<circle cx="12" cy="12" r="9"/><path d="M12 8h.01M12 11v5"/>',
  'ui.folder':'<path d="M3.5 6.5a2 2 0 0 1 2-2h4l2 2.5h7a2 2 0 0 1 2 2v8.5a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/>',
  'ui.history':'<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3.5 2"/>',
  'ui.chevron-down':'<path d="M6 9.5l6 6 6-6"/>',
  'ui.comment':'<path d="M12 20a8 8 0 1 0-7.1-4.3L4 20l4.3-.9A8 8 0 0 0 12 20z"/>',
  'ui.wave':'<path d="M4 10v4M8 7v10M12 4v16M16 7v10M20 10v4"/>',
  'ui.eye':'<path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="3"/>',
  'ui.pin':'<path d="M9 4h6l-1 6 3.5 3.5H6.5L10 10 9 4z"/><path d="M12 13.5V21"/>',
  'ui.mv':'<rect x="3.5" y="5" width="17" height="14" rx="2.5"/><path d="M10.5 9.2l4.5 2.8-4.5 2.8V9.2z"/>',
  'ui.sort':'<path d="M8 5v14M8 5l-3.5 3.5M8 5l3.5 3.5M16 19V5M16 19l-3.5-3.5M16 19l3.5-3.5"/>',
  'ui.gear':'<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M2 12h3M19 12h3M4.9 19.1L7 17M17 7l2.1-2.1"/>',
  'ui.headphone':'<path d="M4 14a8 8 0 0 1 16 0"/><rect x="3" y="14" width="4" height="6" rx="2"/><rect x="17" y="14" width="4" height="6" rx="2"/>',
  'ui.calendar':'<rect x="3.5" y="5" width="17" height="16" rx="2.5"/><path d="M3.5 10h17M8 2.5V6M16 2.5V6"/>',
  'ui.music-note':'<path d="M9 18V6l10-2v11"/><circle cx="6.5" cy="18" r="2.5"/><circle cx="16.5" cy="15" r="2.5"/>',
  'ui.user':'<circle cx="12" cy="8" r="4"/><path d="M4.5 20a7.5 7.5 0 0 1 15 0"/>',
  'ui.sliders':'<path d="M5 4v5M5 13v7M12 4v9M12 17v3M19 4v3M19 11v9"/><circle cx="5" cy="11" r="2"/><circle cx="12" cy="15" r="2"/><circle cx="19" cy="9" r="2"/>',
  'ui.volume':'<path d="M4 9.5v5h3.5L12 18.5v-13L7.5 9.5H4z"/><path d="M15.5 9a4.2 4.2 0 0 1 0 6M18 6.8a7.6 7.6 0 0 1 0 10.4"/>',
  'ui.lyrics':'<rect x="3" y="4" width="14" height="16" rx="2.5"/><path d="M7 9h6M7 13h4"/><path d="M20 9v7"/>',
  'ui.watch':'<rect x="7" y="6.5" width="10" height="11" rx="3"/><path d="M9.5 6.5 9 3h6l-.5 3.5M9.5 17.5 9 21h6l-.5-3.5"/>',
  'ui.wrench':'<path d="M21 6.5a5 5 0 0 1-7 4.6L7 18l-3-3 6.9-7A5 5 0 0 1 17.5 3L15 5.5 18.5 9z"/>',
  'ui.share-up':'<path d="M12 15V4m0 0L8 8m4-4 4 4"/><path d="M5 12v7a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-7"/>',
  'lib.drag':'<circle cx="8" cy="5" r="2.1"/><circle cx="16" cy="5" r="2.1"/><circle cx="8" cy="12" r="2.1"/><circle cx="16" cy="12" r="2.1"/><circle cx="8" cy="19" r="2.1"/><circle cx="16" cy="19" r="2.1"/>'
};
const FILLED_ICONS = ['player.play','action.more','lib.drag'];
// 无专属图形的槽位缩略图直接复用对应控件的同款图形（与客户端回落语义一致）
const FALLBACK_ICON = {
  'player.speed':'ui.wave', 'player.comment':'ui.comment', 'entry.wallpaper':'ui.palette',
  'recognize.mic':'entry.mic',
  'mine.grid_favorite':'action.favorite', 'mine.grid_recent':'ui.history',
  'mine.grid_local':'ui.folder', 'mine.grid_download':'action.download',
  'mine.stat_listen':'ui.headphone', 'mine.stat_today':'ui.calendar', 'mine.stat_count':'ui.music-note',
  'mine.settings':'ui.gear',
  'landscape.logo':'desktop.logo', 'landscape.wallpaper':'ui.palette', 'landscape.settings':'ui.gear',
  'desktop.wallpaper':'ui.palette', 'desktop.settings':'ui.gear',
  'player.lyric':'ui.lyrics', 'player.comment':'ui.comment', 'player.volume':'ui.volume', 'player.sound':'ui.sliders',
  'player.mv':'ui.mv', 'player.visualizer':'ui.wave', 'player.progress':'ui.eye', 'player.style':'ui.palette', 'player.pin':'ui.pin',
  'page.playall':'player.play', 'page.sort':'ui.sort', 'page.more':'action.more', 'page.fav':'action.favorite'
};
// 播放条三键在「播放页」本页卡里的标题（公共区仍用 SLOTS label 的「播放条 ·」前缀）
const PAGE_TITLE = {
  'player.prev':'播放页 · 上一首', 'player.play':'播放页 · 播放/暂停', 'player.next':'播放页 · 下一首'
};
const ICON_LABEL = {
  'nav.home':'首页','nav.settings':'我的',
  'entry.search':'搜索','entry.mic':'识曲','entry.import':'导入',
  'desktop.logo':'Logo','nav.back':'返回'
};
function iconHtml(slot, map, sizePx){
  const v = map && map[slot];
  if (v) return '<img class="pv-img" src="'+esc(v)+'" alt="" style="width:'+sizePx+'px;height:'+sizePx+'px">';
  const d = ICON_PATHS[slot] || (FALLBACK_ICON[slot] ? ICON_PATHS[FALLBACK_ICON[slot]] : null);
  if (!d) return '';
  const filled = FILLED_ICONS.indexOf(slot) >= 0;
  return '<svg class="ic-svg" viewBox="0 0 24 24" fill="'+(filled?'currentColor':'none')+'" stroke="'+(filled?'none':'currentColor')+'" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" style="width:'+sizePx+'px;height:'+sizePx+'px">'+d+'</svg>';
}

function defaultTheme() {
  return { accentColor:'#EC4141', themeMode:'dark', wallpaperRef:null, quickEntryShape:'circle', icons:{}, stickers:{}, surfaces:{} };
}
const state = {
  name: '我的主题',
  description: '',
  themes: { mobile: defaultTheme(), desktop: defaultTheme() },
};
let platform = 'mobile';
let orientation = 'portrait'; // 移动端二级模式：portrait=竖屏 | landscape=横屏（横屏为独立侧栏式 UI）
let previewPage = 'home';
let previewData = '';
let auth = null;
try { auth = JSON.parse(localStorage.getItem('themeEditorAuth') || 'null'); } catch(e) { auth = null; }
if (!localStorage.getItem('themeEditorDevice')) {
  localStorage.setItem('themeEditorDevice', 'web-' + Math.random().toString(16).slice(2, 10) + Date.now().toString(16));
}
const deviceId = localStorage.getItem('themeEditorDevice');

function $id(x){ return document.getElementById(x); }
function cur(){ return state.themes[platform]; }
function esc(s){ return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;'); }
function toast(msg, ms){
  const t = $id('toast'); t.textContent = msg; t.style.display = 'block';
  clearTimeout(t._h); t._h = setTimeout(()=>{ t.style.display='none'; }, ms || 2600);
}

/* ---------- 面板 ---------- */
function renderSwatches(){
  const c = cur().accentColor;
  $id('swatches').innerHTML = PRESET_COLORS.map(c2 =>
    '<button type="button" class="sw'+(c2.toLowerCase()===String(c).toLowerCase()?' on':'')+'" style="background:'+c2+'" onclick="setAccent(\''+c2+'\')" aria-label="'+c2+'"></button>'
  ).join('');
  $id('accentPicker').value = /^#[0-9a-fA-F]{6}$/.test(c) ? c : '#EC4141';
  $id('accentHex').textContent = c;
}
function setAccent(v){
  if(!/^#[0-9a-fA-F]{6}$/.test(v)) return;
  cur().accentColor = v.toUpperCase();
  renderSwatches(); renderPreview();
}
function pageLabel(){
  const pages = (SLOTS.platforms[platform] && SLOTS.platforms[platform].pages) || [];
  const p = pages.find(x => x.id === previewPage);
  return p ? p.label : '';
}
function pageShortLabel(){
  return pageLabel().replace(/^横屏 · /, '') || (platform === 'mobile' ? '移动端' : '桌面端');
}
let slotScope = 'page'; // page=本页专属 | shared=公共通用（跨页统一设置，不随页面重复）
function renderScopeSeg(){
  const seg = $id('scopeSeg'); if(!seg) return;
  const label = pageLabel() || (platform === 'mobile' ? '移动端' : '桌面端');
  [...seg.children].forEach(b=>{
    b.classList.toggle('on', b.dataset.scope === slotScope);
    if(b.dataset.scope === 'page') b.textContent = '本页 · ' + label;
  });
}
function renderSlotList(kind, mountId){
  const slots = (SLOTS.platforms[platform] || {})[kind] || [];
  const map = cur()[kind] || {};
  const mine = slots.filter(s => s.page === previewPage || (s.also && s.also.indexOf(previewPage) >= 0));
  const shared = slots.filter(s => (!s.page || s.page === 'global') && (!s.ls || orientation === 'landscape'));
  const list = slotScope === 'page' ? mine : shared;
  const rowHtml = s => {
    const v = map[s.id];
    const thumb = v
      ? '<img src="'+esc(v)+'" alt="">'
      : iconHtml(s.id, null, 19);
    return '<div class="slot-row">'
      + '<div class="slot-thumb">'+thumb+'</div>'
      + '<div class="slot-meta"><div class="n">'+esc((slotScope==='page' && PAGE_TITLE[s.id]) ? PAGE_TITLE[s.id] : s.label)+'</div><div class="id">'+esc(s.id)+'</div></div>'
      + '<div class="slot-ops">'
      + '<button class="btn" onclick="pickSlot(\''+kind+'\',\''+s.id+'\')">上传</button>'
      + (v ? '<button class="btn danger" onclick="clearSlot(\''+kind+'\',\''+s.id+'\')">清除</button>' : '')
      + '</div></div>';
  };
  const platName = platform === 'mobile' ? '移动端' : '桌面端';
  let html = '';
  if (list.length) {
    const sharedTip = slotScope === 'shared'
      ? '<p class="hint" style="margin:2px 0 8px">公共控件设置一次、所有页面统一生效：底部导航（发现/我的）、mini 播放条三键（上一首/播放/下一首，播放页控制行也用同一套）、搜索框公共件（放大镜/识曲钮）。</p>'
      : '';
    html = sharedTip + list.map(rowHtml).join('');
  } else if (slotScope === 'page') {
    html = '<p class="hint" style="margin:2px 0 8px">'+esc(pageShortLabel())+'暂无专属自定义项，切到「公共通用」设置全局生效的槽位。</p>';
  } else {
    html = '<p class="hint" style="margin:2px 0 8px">该平台暂无公共槽位。</p>';
  }
  $id(mountId).innerHTML = html;
  if(kind === 'icons'){
    $id('slotCount').textContent = list.length;
    $id('iconCardTitle').textContent = '图标槽位 · ' + (slotScope === 'page' ? pageShortLabel() : '公共通用');
  } else {
    $id('stickerCard').style.display = list.length ? '' : 'none';
    $id('stickerCardTitle').textContent = '贴纸槽位 · ' + (slotScope === 'page' ? pageShortLabel() : '公共通用');
  }
}
/* ---------- 组件色块（surfaces） ---------- */
/* 预览经根容器 CSS 变量 --sf-<id> 驱动：调色块只更新根变量，不重建 DOM，杜绝闪烁 */
function sfVar(id){ return '--sf-' + String(id).replace(/\./g, '-'); }
function surfaceBg(id, base){
  return 'background:var('+sfVar(id)+','+(base || 'var(--p-card)')+');';
}
function applySurfaceVars(){
  if(platform !== 'mobile') return;
  const root = $id(orientation === 'landscape' ? 'dScreen' : 'mScreen');
  if(!root) return;
  const t = cur();
  ((SLOTS.platforms.mobile || {}).surfaces || []).forEach(s => {
    const v = t.surfaces && t.surfaces[s.id];
    if(v && v.c){
      const a = Math.max(0, Math.min(1, typeof v.o === 'number' ? v.o : 0.5));
      root.style.setProperty(sfVar(s.id), 'color-mix(in srgb, '+v.c+' '+Math.round(a*100)+'%, transparent)');
    } else root.style.removeProperty(sfVar(s.id));
  });
}
function surfaceRowSub(v, c, o){
  return v ? '已设置 · '+c+' · '+o+'%' : '未设置 · 维持默认材质';
}
function updateSurfaceRow(id, v){
  const row = $id('surfaceSlots').querySelector('[data-sid="'+id+'"]');
  if(!row) return;
  const c = v && v.c ? v.c : '#EC4141';
  const o = v ? Math.round(Math.max(0, Math.min(1, typeof v.o === 'number' ? v.o : 0.5)) * 100) : 50;
  row.querySelector('.srow__sub').textContent = surfaceRowSub(v, c, o);
  const ci = row.querySelector('input[type=color]');
  const ri = row.querySelector('input[type=range]');
  if(!v){ ci.value = '#EC4141'; ri.value = 50; }
}
function setSurface(id, c, o){
  const t = cur();
  if(!t.surfaces) t.surfaces = {};
  const s = t.surfaces[id] || { c:'#EC4141', o:0.5 };
  if(c) s.c = c.toUpperCase();
  if(typeof o === 'number') s.o = Math.max(0, Math.min(1, o/100));
  t.surfaces[id] = s;
  updateSurfaceRow(id, s);
  applySurfaceVars();
}
function clearSurface(id){
  const t = cur();
  if(t.surfaces) delete t.surfaces[id];
  updateSurfaceRow(id, null);
  applySurfaceVars();
}
function renderSurfaceList(){
  const card = $id('surfaceCard');
  if(platform !== 'mobile'){ card.style.display = 'none'; return; }
  const slots = (SLOTS.platforms.mobile || {}).surfaces || [];
  const t = cur();
  const map = t.surfaces || {};
  const isLs = platform === 'mobile' && orientation === 'landscape';
  const mine = slots.filter(s => s.page === previewPage || (s.also && s.also.indexOf(previewPage) >= 0));
  const shared = slots.filter(s => s.page === 'global' && !(isLs && s.id === 'nav.bar'));
  const list = slotScope === 'page' ? mine : shared;
  const label = pageShortLabel();
  card.style.display = '';
  $id('surfaceCardTitle').textContent = '组件色块 · ' + (slotScope === 'page' ? label : '公共通用');
  const sharedTip = slotScope === 'shared'
    ? '<p class="hint" style="margin:2px 0 8px">公共色块设置一次、所有页面统一生效：mini 播放条（竖屏底部条 + 横屏悬浮胶囊）、搜索框胶囊（含横屏顶栏搜索条）'+(isLs?'':'、底部导航栏')+'。</p>'
    : '';
  $id('surfaceSlots').innerHTML = sharedTip + (list.length ? list.map(s => {
    const v = map[s.id];
    const c = v && v.c ? v.c : '#EC4141';
    const o = v ? Math.round(Math.max(0, Math.min(1, typeof v.o === 'number' ? v.o : 0.5)) * 100) : 50;
    return '<div class="srow" data-sid="'+s.id+'">'
      + '<div class="srow__info"><div class="srow__n">'+esc(s.label)+'</div>'
      + '<div class="srow__sub">'+surfaceRowSub(v, c, o)+'</div></div>'
      + '<input type="color" value="'+c+'" oninput="setSurface(\''+s.id+'\', this.value)">'
      + '<input type="range" min="0" max="100" value="'+o+'" oninput="setSurface(\''+s.id+'\', null, Number(this.value))">'
      + '<button class="srow__x" title="清除" onclick="clearSurface(\''+s.id+'\')">✕</button>'
      + '</div>';
  }).join('') : '<p class="hint" style="margin:2px 0 8px">'+esc(label)+'暂无组件色块，切到「公共通用」设置全局生效的槽位。</p>');
}
$id('scopeSeg').addEventListener('click', e => {
  const b = e.target.closest('button'); if(!b || b.dataset.scope === slotScope) return;
  slotScope = b.dataset.scope;
  renderScopeSeg();
  renderSlotList('icons','iconSlots');
  renderSlotList('stickers','stickerSlots');
  renderSurfaceList();
});
/* —— 预览缩放：右下角大小条 + 直接滚轮（0.5x~2x）—— */
let previewZoom = 1;
function applyZoom(){
  const z = Math.round(previewZoom*100)/100;
  $id('mobileWrap').style.zoom = z;
  $id('desktopWrap').style.zoom = z;
  $id('zoomRange').value = z;
  $id('zoomPct').textContent = Math.round(z*100)+'%';
}
function stepZoom(d){
  previewZoom = Math.min(2, Math.max(0.5, Math.round((previewZoom+d)*100)/100));
  applyZoom();
}
$id('zoomRange').addEventListener('input', e => { previewZoom = parseFloat(e.target.value); applyZoom(); });
$id('zoomMinus').addEventListener('click', () => stepZoom(-0.1));
$id('zoomPlus').addEventListener('click', () => stepZoom(0.1));
$id('zoomReset').addEventListener('click', () => { previewZoom = 1; applyZoom(); });
$id('stage').addEventListener('wheel', e => {
  if(e.shiftKey) return;                                  /* Shift+滚轮 = 原生滚动，放大后看底部用 */
  if(window.matchMedia('(max-width:1080px)').matches) return; /* 窄屏上下堆叠布局：滚轮滚整页 */
  e.preventDefault();
  stepZoom(e.deltaY < 0 ? 0.1 : -0.1);
}, {passive:false});
function pickSlot(kind, slot){
  const inp = document.createElement('input');
  inp.type = 'file';
  inp.accept = 'image/png,image/jpeg,image/webp,image/gif,image/svg+xml';
  inp.onchange = () => {
    const f = inp.files && inp.files[0];
    if(!f) return;
    if(f.size > 2*1024*1024){ toast('单个资源请控制在 2MB 以内'); return; }
    const r = new FileReader();
    r.onload = () => {
      cur()[kind][slot] = String(r.result);
      renderSlotList(kind, kind==='icons'?'iconSlots':'stickerSlots');
      renderPreview();
    };
    r.readAsDataURL(f);
  };
  inp.click();
}
function clearSlot(kind, slot){
  delete cur()[kind][slot];
  renderSlotList(kind, kind==='icons'?'iconSlots':'stickerSlots');
  renderPreview();
}
function setWallpaperRef(v){
  const n = parseInt(v, 10);
  cur().wallpaperRef = isNaN(n) || n<=0 ? null : { id:n };
}

/* ---------- 预览（跟随所选平台，单设备展示） ---------- */
function stickerImg(url, sizePx){
  return '<img class="pv-img sticker-img" src="'+esc(url)+'" alt="" style="width:'+sizePx+'px;height:'+sizePx+'px">';
}
function logoHtml(t, slot, sizePx){
  const v = t.icons && t.icons[slot];
  const r = Math.round(sizePx * 0.3);
  if (v) return '<img class="pv-img" src="'+esc(v)+'" alt="" style="width:'+sizePx+'px;height:'+sizePx+'px;border-radius:'+r+'px">';
  return '<div style="width:'+sizePx+'px;height:'+sizePx+'px;border-radius:'+r+'px;background:var(--p-accent);display:flex;align-items:center;justify-content:center;flex:none;box-shadow:0 2px 8px color-mix(in srgb,var(--p-accent) 40%, transparent)"><svg viewBox="0 0 24 24" fill="#fff" style="width:'+Math.round(sizePx*0.55)+'px;height:'+Math.round(sizePx*0.55)+'px"><path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm0 18a8 8 0 1 1 8-8 8 8 0 0 1-8 8zM10.6 7.5v9l7-4.5z"/></svg></div>';
}
function renderPreview(){
  if(platform === 'mobile' && orientation === 'landscape'){
    ({ 'ls-home': renderPreviewLsHome, 'ls-mine': renderPreviewLsMine, 'ls-player': renderPreviewLsPlayer, 'ls-settings': renderPreviewLsSettings,
            'ls-local': renderPreviewLsLocal, 'ls-fav': renderPreviewLsFav,
            'ls-recent': renderPreviewLsRecent, 'ls-sheets': renderPreviewLsSheets }[previewPage] || renderPreviewLsHome)();
  } else if(platform === 'mobile'){
    ({ home: renderPreviewMobile, player: renderPreviewMobilePlayer, mine: renderPreviewMobileMine,
       recognize: renderPreviewMobileRecognize, search: renderPreviewMobileSearch,
       search_result: renderPreviewMobileSearchResult, settings: renderPreviewMobileSettings }[previewPage] || renderPreviewMobile)();
  } else {
    renderPreviewDesktop();
  }
  applySurfaceVars();
  const scr = (platform==='mobile' && orientation==='landscape') || platform==='desktop' ? $id('dScreen') : $id('mScreen');
  scr.classList.remove('fade'); void scr.offsetWidth; scr.classList.add('fade');
}
function visiblePages(){
  const pages = (SLOTS.platforms[platform] && SLOTS.platforms[platform].pages) || [];
  if(platform !== 'mobile') return pages.filter(p => !p.id.startsWith('ls-'));
  return pages.filter(p => orientation==='landscape' ? p.id.startsWith('ls-') : !p.id.startsWith('ls-'));
}
function renderStageTabs(){
  const pages = visiblePages();
  if(!pages.some(p => p.id === previewPage)) previewPage = pages.length ? pages[0].id : '';
  $id('stageTabs').innerHTML = pages.map(p =>
    '<button class="'+(p.id===previewPage?'on':'')+'" onclick="switchPage(\''+p.id+'\')">'+esc(p.label)+'</button>'
  ).join('');
  const onBtn = $id('stageTabs').querySelector('button.on');
  if(onBtn) onBtn.scrollIntoView({ behavior:'smooth', inline:'center', block:'nearest' });
  updateShapeCard();
}
function updateShapeCard(){
  $id('shapeCard').style.display = (platform==='mobile' && orientation==='portrait' && previewPage==='mine') ? '' : 'none';
}
function switchPage(id){
  if(previewPage === id) return;
  previewPage = id;
  slotScope = 'page';
  renderStageTabs();
  renderScopeSeg();
  renderSlotList('icons','iconSlots');
  renderSlotList('stickers','stickerSlots');
  renderSurfaceList();
  renderPreview();
}
function renderPreviewMobile(){
  const t = state.themes.mobile;
  const el = $id('mScreen');
  el.className = 'screen ' + (t.themeMode==='light'?'light':'dark');
  el.style.setProperty('--p-accent', t.accentColor);
  const songs = [['Closer','The Chainsmokers, Halsey'],['Alone (Restrung)','Alan Walker'],['All We Know','The Chainsmokers, Phoebe Ryan']];
  el.innerHTML =
    '<div class="m-status"><span>08:02</span><span>●●●</span></div>'
    + '<div class="h-top">'
    + '<span class="h-logo">弦予<em>音乐</em></span>'
    + '<div class="h-search" style="'+surfaceBg('search.box')+'">'+iconHtml('entry.search', t.icons, 14)+'<span class="ph">搜索歌曲、歌手、专辑</span><span class="h-round" style="width:22px;height:22px;background:var(--p-accent);color:#fff;border:none">'+iconHtml('entry.mic', t.icons, 12)+'</span></div>'
    + '<div class="h-round">'+(t.icons && t.icons['entry.wallpaper'] ? '<img class="pv-img" src="'+esc(t.icons['entry.wallpaper'])+'" alt="" style="width:17px;height:17px;object-fit:contain">' : iconHtml('ui.palette', t.icons, 15))+'</div>'
    + '</div>'
    + '<div class="body-scroll">'
    + '<div class="stat-card" style="'+surfaceBg('home.stat', 'linear-gradient(135deg, color-mix(in srgb, var(--p-accent) 26%, var(--p-card)), color-mix(in srgb, var(--p-accent) 55%, var(--p-card)))')+'"><div class="head">'+iconHtml('ui.history', t.icons, 15)+'听歌数据统计</div>'
        + '<div class="cap">累计听歌总时长</div><div class="big">3 分钟</div>'
        + '<div class="cols">'
        + '<div class="cell">'+iconHtml('ui.calendar', t.icons, 13)+'<span>今天听歌时长<b>3 分钟</b></span></div>'
        + '<div class="cell">'+iconHtml('ui.music-note', t.icons, 13)+'<span>今天已听<b>5 首</b></span></div>'
        + '</div><div class="dot"></div></div>'
    + '<div class="sec-h"><span class="t">听过最多</span></div>'
    + songs.map(s=>'<div class="song-row" style="'+surfaceBg('home.song')+'"><div class="cov"></div><div class="meta"><div class="n">'+s[0]+'</div><div class="a">'+s[1]+'</div></div><span class="cnt">1 次</span><span class="play">'+iconHtml('player.play', t.icons, 12)+'</span></div>').join('')
    + '</div>'
    + miniBarHtml(t, '')
    + tabbarHtml(t, 0);
}
function renderPreviewMobilePlayer(){
  const t = state.themes.mobile;
  const el = $id('mScreen');
  el.className = 'screen ' + (t.themeMode==='light'?'light':'dark');
  el.style.setProperty('--p-accent', t.accentColor);
  el.innerHTML =
    '<div class="m-status"><span>21:36</span><span>●●●</span></div>'
    + '<div class="pl-top">'
    + '<span class="mini" style="color:var(--p-text)">'+iconHtml('ui.chevron-down', t.icons, 16)+'</span>'
    + '<div class="pl-seg"><span class="on">封面</span><span>歌词</span></div>'
    + '<span class="mini" style="color:var(--p-text)">'+iconHtml('ui.share-up', t.icons, 16)+'</span>'
    + '</div>'
    + '<div class="body-scroll">'
    + '<div class="p-coverwrap" style="margin-top:10px"><div class="pl-ambient" style="display:flex;justify-content:center"><div class="pl-cover"></div></div></div>'
    + '<div class="pl-title-row"><div class="grow"><div class="p-name">演员</div><div class="t-sub" style="margin-top:3px">薛之谦</div></div>'
    + '<span class="mini" style="color:var(--p-text)">'+iconHtml('action.favorite', t.icons, 20)+'</span></div>'
    + '<div class="pl-lyric"><span>防备后的这些那些</span><span class="on">才是考验</span><span>没意见</span></div>'
    + '</div>'
    + '<div style="flex:none;padding:0 18px 16px">'
    + '<div class="pl-ops">'
    + '<span class="mini">'+iconHtml('player.speed', t.icons, 17)+'</span>'
    + '<span class="hq">HQ</span>'
    + '<span class="mini">'+iconHtml('action.download', t.icons, 17)+'</span>'
    + '<span class="mini">'+iconHtml('player.comment', t.icons, 17)+'</span>'
    + '<span class="mini">'+iconHtml('action.more', t.icons, 17)+'</span>'
    + '</div>'
    + '<div class="pl-bar"><div class="track"></div>'
    + '<div style="display:flex;justify-content:space-between;font-size:9px;color:var(--p-sub);margin-top:5px;font-family:var(--font-mono)"><span>01:42</span><span>00:01</span></div></div>'
    + '<div class="p-ctrl">'
    + '<span class="mini">'+iconHtml('player.mode', t.icons, 17)+'</span>'
    + '<span class="mini">'+iconHtml('player.prev', t.icons, 18)+'</span>'
    + '<div class="big">'+iconHtml('player.play', t.icons, 16)+'</div>'
    + '<span class="mini">'+iconHtml('player.next', t.icons, 18)+'</span>'
    + '<span class="mini">'+iconHtml('player.queue', t.icons, 17)+'</span>'
    + '</div>'
    + '</div>';
}
function miniBarHtml(t, botSticker, song, artist){
  return '<div class="m-player" style="'+surfaceBg('mini.bar')+'">'
    + (botSticker || '')
    + '<div class="mini-cov"></div>'
    + '<div class="tt" style="flex:1;min-width:0"><div class="p-name" style="font-size:11.5px">'+(song||'Closer')+'</div><div class="t-sub">'+(artist||'The Chainsmokers, Halsey')+'</div></div>'
    + '<span class="mini" style="color:var(--p-text)">'+iconHtml('player.prev', t.icons, 14)+'</span>'
    + '<span class="mini" style="color:var(--p-accent)">'+iconHtml('player.play', t.icons, 16)+'</span>'
    + '<span class="mini" style="color:var(--p-text)">'+iconHtml('player.next', t.icons, 14)+'</span>'
    + '</div>';
}
function tabbarHtml(t, active){
  const tab = (idx, label, icon) => '<div class="ti'+(active===idx?' on':'')+'">'
    + (active===idx ? '<div class="fab">'+iconHtml(icon, t.icons, 20)+'</div>' : iconHtml(icon, t.icons, 18))
    + '<span>'+label+'</span></div>';
  return '<div class="tabbar" style="'+surfaceBg('nav.bar')+'">'+tab(0,'发现','nav.home')+tab(1,'我的','nav.settings')+'</div>';
}
function renderPreviewMobileMine(){
  const t = state.themes.mobile;
  const el = $id('mScreen');
  el.className = 'screen ' + (t.themeMode==='light'?'light':'dark') + ' shape-' + (t.quickEntryShape || 'circle');
  el.style.setProperty('--p-accent', t.accentColor);
  const stats = [['mine.stat_listen','21 小时 47 分钟','累计听歌'],['mine.stat_today','1 小时 16 分钟','今日时长'],['mine.stat_count','32 首','今日首数']];
  const grid = [['mine.grid_favorite','action.favorite','喜欢',33],['mine.grid_recent','ui.history','最近',12],['mine.grid_local','ui.folder','本地',78],['mine.grid_download','action.download','下载',1]];
  const gIcon = (slot, fb) => { const v = t.icons && t.icons[slot]; return v ? '<img src="'+esc(v)+'" style="width:17px;height:17px;object-fit:contain" alt="">' : iconHtml(fb, t.icons, 17); };
  el.innerHTML =
    '<div class="m-status"><span>21:36</span><span>●●●</span></div>'
    + '<div class="h-top" style="padding-top:10px">'
    + '<span class="h-logo" style="font-size:14px;background:var(--p-card);border:1px solid var(--p-line);border-radius:100px;padding:8px 14px;">个人中心</span>'
    + '<div class="h-search" style="height:40px;'+surfaceBg('search.box')+'">'+iconHtml('entry.search', t.icons, 14)+'<span class="ph">搜索歌曲、歌手、专辑</span>'
    + '<span class="h-round" style="width:24px;height:24px;background:var(--p-accent);color:#fff;border:none;opacity:1">'+iconHtml('entry.mic', t.icons, 12)+'</span></div>'
    + '<span class="h-round">'+iconHtml('mine.settings', t.icons, 15)+'</span>'
    + '</div>'
    + '<div class="body-scroll">'
    + '<div class="me-card" style="'+surfaceBg('mine.user')+'"><div class="ava"></div><div class="grow"><div class="n">小奇</div><div class="s">管理账号与安全</div></div></div>'
    + '<div class="me-stats" style="'+surfaceBg('mine.stats')+'">'+stats.map(s=>'<div class="cell">'+iconHtml(s[0], t.icons, 15)+'<div class="v">'+s[1]+'</div><div class="l">'+s[2]+'</div></div>').join('')+'</div>'
    + '<div class="me-grid" style="'+surfaceBg('mine.grid')+'">'+grid.map(g=>'<div class="cell"><div class="bub">'+gIcon(g[0], g[1])+'</div><div class="n">'+g[2]+'</div><div class="c">'+g[3]+'</div></div>').join('')+'</div>'
    + '<div class="sec-h" style="margin:12px 18px 8px"><span class="t">自建歌单 <small>1</small></span><span class="act">＋ 新建</span></div>'
    + '<div class="pl-row" style="'+surfaceBg('mine.sheet')+'"><div class="cov"></div><div class="grow"><div class="n">英文摇滚</div><div class="s">共 43 首歌</div></div>'+iconHtml('action.more', t.icons, 15)+'</div>'
    + '<div class="pl-row"><div class="cov" style="background:var(--p-line);color:var(--p-text);display:flex;align-items:center;justify-content:center">'+iconHtml('entry.import', t.icons, 16)+'</div><div class="grow"><div class="n">导入外部歌单</div><div class="s">备份文件 / 本地文件夹 / 云端导入</div></div><span style="color:var(--p-sub);font-size:15px;line-height:1">›</span></div>'
    + '</div>'
    + miniBarHtml(t, null, '演员', '薛之谦')
    + tabbarHtml(t, 1);
}
function renderPreviewMobileRecognize(){
  const t = state.themes.mobile;
  const el = $id('mScreen');
  el.className = 'screen ' + (t.themeMode==='light'?'light':'dark');
  el.style.setProperty('--p-accent', t.accentColor);
  const tips = [['ui.wave','请先让音乐外放，再点上面的麦克风'],['ui.lyrics','优先匹配本地曲库，本地没有的走在线音源解析'],['ui.music-note','识别成功后可直接播放、收藏或加入歌单']];
  const stDeco = t.stickers && t.stickers['recognize.deco'];
  const rIcon = (sz, fb) => { const v = t.icons && t.icons['recognize.mic']; return v ? '<img class="pv-img" src="'+esc(v)+'" alt="" style="width:'+sz+'px;height:'+sz+'px;object-fit:contain">' : iconHtml(fb, t.icons, sz); };
  el.innerHTML =
    '<div class="m-status"><span>17:24</span><span>●●●</span></div>'
    + '<div class="m-appbar" style="gap:10px;justify-content:flex-start">'
    + '<span class="nav-round">'+iconHtml('nav.back', t.icons, 17)+'</span>'
    + '<span class="pill-title">'+rIcon(14, 'entry.mic')+'听歌识曲</span>'
    + '</div>'
    + '<div class="body-scroll">'
    + '<div class="rec-hero"><div class="rec-rings"><div class="mid"><div class="core" style="'+surfaceBg('recognize.btn', 'var(--p-accent)')+'">'+rIcon(24, 'entry.mic')+'</div></div></div>'
    + '<div class="rec-cta">点击麦克风开始识别</div><div class="rec-dash"></div></div>'
    + '<div class="rec-tips" style="'+surfaceBg('recognize.hint', 'color-mix(in srgb, var(--p-accent) 7%, var(--p-card))')+'">'+tips.map(x=>'<div class="row">'+iconHtml(x[0], t.icons, 14)+'<span>'+x[1]+'</span></div>').join('')+'</div>'
    + (stDeco ? '<div style="display:flex;justify-content:center;padding:12px 0 10px"><img class="sticker-img" src="'+esc(stDeco)+'" alt="" style="max-height:190px;max-width:75%;object-fit:contain"></div>' : '')
    + '</div>'
    + miniBarHtml(t, null, 'Sail', 'AWOLNATION');
}
function renderPreviewMobileSearch(){
  const t = state.themes.mobile;
  const el = $id('mScreen');
  el.className = 'screen ' + (t.themeMode==='light'?'light':'dark');
  el.style.setProperty('--p-accent', t.accentColor);
  const hot = [['薛之谦','1721人搜'],['许嵩','1048人搜'],['鹿晗','974人搜'],['周杰伦','892人搜'],['沿花路前行','171人搜'],['梦的翅膀受了伤','130人搜'],['云端音乐铺','127人搜'],['邓紫棋','102人搜'],['妖精的尾巴','79人搜'],['蔡依林 pillow','60人搜']];
  el.innerHTML =
    '<div class="m-status"><span>17:18</span><span>●●●</span></div>'
    + '<div class="m-appbar" style="gap:10px;justify-content:flex-start">'
    + '<span class="nav-round">'+iconHtml('nav.back', t.icons, 17)+'</span>'
    + '<div class="h-search" style="height:40px;'+surfaceBg('search.box')+'">'+iconHtml('entry.search', t.icons, 15)+'<span class="ph" style="color:var(--p-accent);font-weight:700">|</span></div>'
    + '<span class="nav-round" style="box-shadow:none">'+iconHtml('entry.search', t.icons, 16)+'</span>'
    + '</div>'
    + '<div class="body-scroll">'
    + '<div class="hist-card" style="'+surfaceBg('search.panel', 'color-mix(in srgb, var(--p-accent) 6%, var(--p-card))')+'"><div class="h">'+iconHtml('ui.history', t.icons, 14)+'搜索历史</div><div class="empty">暂无搜索历史</div></div>'
    + '<div class="hot-head">'+iconHtml('ui.music-note', t.icons, 14)+'大家都在搜</div>'
    + hot.map((h,i)=>'<div class="hot-row'+(i<3?' top':'')+'" style="'+surfaceBg('search.item', 'transparent')+'"><span class="no'+(i<3?' top':'')+'">'+(i+1)+'</span><span class="kw">'+h[0]+'</span><span class="cnt">'+h[1]+'</span></div>').join('')
    + '</div>';
}
function renderPreviewMobileSearchResult(){
  const t = state.themes.mobile;
  const el = $id('mScreen');
  el.className = 'screen ' + (t.themeMode==='light'?'light':'dark');
  el.style.setProperty('--p-accent', t.accentColor);
  const tabs = ['单曲','歌手','专辑','歌单'];
  const srcs = [['bilibili','bilibili'],['kugou','酷狗音乐'],['kuwo','酷我音乐'],['migu','咪咕音乐'],['qishui','汽水音乐']];
  const srcChip = (label, on) => '<span class="src-chip'+(on?' on':'')+'" style="'+surfaceBg('sr.pill')+'">'+label+'</span>';
  const rows = [['演员','薛之谦 · 绅士 · 酷狗音乐','04:21'],['演员','薛之谦 · 初学者 · 酷狗音乐','04:21'],['演员','薛之谦 · 酷狗音乐','04:21'],['天外来物','薛之谦 · 天外来物 · 酷狗音乐','04:17'],['绅士','薛之谦 · 绅士 · 酷狗音乐','04:50'],['绅士','薛之谦 · 初学者 · 酷狗音乐','04:50'],['你还要我怎样','薛之谦 · 意外 · 酷狗音乐','05:10'],['其实','薛之谦 · 意外 · 酷狗音乐','04:02'],['我好像在哪见过你','薛之谦 · 初学者 · 酷狗音乐','04:39']];
  el.innerHTML =
    '<div class="m-status"><span>17:22</span><span>●●●</span></div>'
    + '<div class="m-appbar" style="gap:10px;justify-content:flex-start">'
    + '<span class="nav-round">'+iconHtml('nav.back', t.icons, 17)+'</span>'
    + '<div class="h-search" style="height:40px;'+surfaceBg('search.box')+'">'+iconHtml('entry.search', t.icons, 15)+'<span class="ph">薛之谦</span></div>'
    + '<span class="nav-round" style="box-shadow:none">'+iconHtml('entry.search', t.icons, 16)+'</span>'
    + '</div>'
    + '<div class="seg-tabs" style="'+surfaceBg('sr.chips', 'transparent')+'">'+tabs.map((n,i)=>'<span'+(i===0?' class="on"':'')+'>'+n+'</span>').join('')+'</div>'
    + '<div class="src-chips" style="'+surfaceBg('sr.chips', 'transparent')+'">'+srcs.map((s,i)=>srcChip(s[1], i===1)).join('')+'</div>'
    + '<div class="body-scroll" style="margin-top:4px">'+rows.map(r=>'<div class="res-row" style="'+surfaceBg('sr.item', 'transparent')+'"><div class="cov"></div><div class="grow"><div class="n">'+r[0]+'</div><div class="s">'+r[1]+'</div></div><span class="fav">'+iconHtml('action.favorite', t.icons, 15)+'</span><span class="dur">'+r[2]+'</span><span class="more">'+iconHtml('action.more', t.icons, 14)+'</span></div>').join('')+'</div>'
    + miniBarHtml(t, null, 'Sail', 'AWOLNATION');
}
function renderPreviewMobileSettings(){
  const t = state.themes.mobile;
  const el = $id('mScreen');
  el.className = 'screen ' + (t.themeMode==='light'?'light':'dark');
  el.style.setProperty('--p-accent', t.accentColor);
  const sec = (title, rows) => '<div class="set-sec">'+title+'</div><div class="set-card" style="'+surfaceBg('settings.group')+'">'
    + rows.map(r=>'<div class="row">'+iconHtml(r[0], t.icons, 16)+'<div class="grow"><div class="n">'+r[1]+'</div><div class="s">'+r[2]+'</div></div></div>').join('')+'</div>';
  el.innerHTML =
    '<div class="m-status"><span>21:58</span><span>●●●</span></div>'
    + '<div class="m-appbar" style="'+surfaceBg('settings.topbar', 'transparent')+'">'+iconHtml('nav.back', t.icons, 18)+'<span class="t" style="font-size:16px">设置</span><span style="width:18px;flex:none"></span></div>'
    + '<div class="m-search" style="'+surfaceBg('search.box')+'">'+iconHtml('entry.search', t.icons, 14)+'<span class="grow">搜索设置</span></div>'
    + '<div class="body-scroll" style="padding-bottom:12px">'
    + sec('账号', [['ui.user','账号','服务端设置、手动同步、自动同步']])
    + sec('偏好', [['ui.sliders','常规','语言、反馈、常亮、存储'],['ui.palette','外观','主题、主题色、壁纸、液态玻璃、导航栏'],['ui.lyrics','歌词','歌词显示、悬浮歌词窗']])
    + sec('腕上联动', [['ui.watch','腕上联动','手表遥控、云端兜底、传递策略']])
    + sec('播放', [['ui.wrench','工具','音频转换、剪辑、解密、重命名'],['ui.headphone','播放','音量、双击播放、播放行为、输出'],['ui.folder','插件','插件：导入、启用、更新、卸载'],['action.download','下载','音质、路径、并发、嵌入']])
    + '</div>';
}
/* —— 移动端横屏（复用桌面壳 CSS，渲染进 dScreen）—— */
function lsTop(t){
  const wp = t.icons && t.icons['landscape.wallpaper'];
  const st = t.icons && t.icons['landscape.settings'];
  return '<div class="d-topbar" style="border:none;padding:12px 20px 6px;flex:none">'
    + '<span class="d-backbtn">'+iconHtml('nav.back', t.icons, 14)+'</span>'
    + '<div class="d-searchbar" style="'+surfaceBg('search.box')+'">'+iconHtml('entry.search', t.icons, 13)+'<span class="grow">搜索歌曲、歌手、专辑</span>'+iconHtml('entry.mic', t.icons, 14)+'</div>'
    + '<span class="d-tbtn">'+(wp ? '<img class="pv-img" src="'+esc(wp)+'" alt="" style="width:15px;height:15px;object-fit:contain">' : iconHtml('ui.palette', t.icons, 15))+'</span>'
    + '<span class="d-tbtn" style="color:var(--p-accent)">'+(st ? '<img class="pv-img" src="'+esc(st)+'" alt="" style="width:15px;height:15px;object-fit:contain">' : iconHtml('ui.gear', t.icons, 15))+'</span>'
    + '</div>';
}
function lsSide(t, onNav){
  const navOn = onNav || 'home';
  const logo = t.icons && t.icons['landscape.logo']
    ? '<img class="pv-img" src="'+esc(t.icons['landscape.logo'])+'" alt="" style="width:22px;height:22px;object-fit:contain">'
    : logoHtml(t, 'landscape.logo', 22);
  const item = (ic,label,on) => '<div class="d-nav'+(on?' on':'')+'">'+iconHtml(ic, t.icons, 13)+'<span>'+label+'</span></div>';
  const sticker = t.stickers && t.stickers['ls-sidebar.bottom'];
  const deco = sticker
    ? '<img class="pv-img" src="'+esc(sticker)+'" alt="" style="width:118px;max-height:90px;object-fit:contain;margin:10px 0 2px">'
    : '<div style="flex:1;min-height:44px"></div>';
  return '<div class="d-side" style="width:150px;padding:14px 10px 12px;gap:1px">'
    + '<div class="d-logo">'+logo+'<span>弦予音乐</span></div>'
    + '<div class="ls-cap">导航</div>'
    + item('nav.home','发现',navOn==='home') + item('nav.settings','我的',navOn==='mine')
    + '<div class="ls-cap">音乐库</div>'
    + item('ui.folder','本地音乐',navOn==='local') + item('action.favorite','我的收藏',navOn==='fav') + item('ui.history','最近播放',navOn==='recent') + item('ui.music-note','我的歌单',navOn==='sheets')
    + deco
    + '</div>';
}
function lsMiniFloat(t){
  return '<div style="position:absolute;left:50%;transform:translateX(-50%);bottom:26px;width:60%;'+surfaceBg('mini.bar')+'border:1px solid var(--p-line);border-radius:100px;box-shadow:0 10px 30px rgba(0,0,0,.22);padding:8px 18px;display:flex;align-items:center;gap:12px">'
    + '<div class="mini-cov" style="width:28px;height:28px"></div>'
    + '<div style="min-width:0;flex:1"><div style="font-size:11.5px;font-weight:700;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">演员</div><div style="font-size:9px;color:var(--p-sub)">薛之谦</div></div>'
    + '<span style="color:var(--p-text)">'+iconHtml('player.prev', t.icons, 15)+'</span>'
    + '<span style="width:28px;height:28px;border-radius:50%;background:var(--p-accent);color:#fff;display:flex;align-items:center;justify-content:center;flex:none;box-shadow:0 3px 10px color-mix(in srgb, var(--p-accent) 40%, transparent)">'+iconHtml('player.play', t.icons, 13)+'</span>'
    + '<span style="color:var(--p-text)">'+iconHtml('player.next', t.icons, 15)+'</span>'
    + '</div>';
}
function renderPreviewLsHome(){
  const t = state.themes.mobile;
  const el = dScreen(t);
  const cards = ['热歌榜','飙升榜','新歌榜','视频歌曲','万物DJ榜','怀旧榜','影视金曲'];
  const row = (n,a) => '<div class="ls-row"><div class="cv"></div><div style="min-width:0;flex:1"><div style="font-size:11px;font-weight:700">'+n+'</div><div style="font-size:9px;color:var(--p-sub)">'+a+'</div></div><span style="color:var(--p-accent)">'+iconHtml('player.play', t.icons, 12)+'</span></div>';
  el.innerHTML = '<div class="d-shell">'
    + lsSide(t)
    + '<div style="flex:1;min-width:0;display:flex;flex-direction:column;position:relative">'
    + lsTop(t)
    + '<div style="flex:1;overflow:hidden;padding:4px 20px 0">'
    + '<div class="ls-sec" style="margin-top:8px"><span class="t">发现</span><span class="act">查看全部 ›</span></div>'
    + '<div class="ls-cards">'+cards.map(c=>'<div class="ls-card"><div class="cv"></div><div class="n">'+c+'</div></div>').join('')+'</div>'
    + '<div class="ls-sec"><span class="t">每日推荐</span><span class="act">查看全部 ›</span></div>'
    + '<div style="'+surfaceBg('ls-home.daily')+'border:1px solid var(--p-line);border-radius:12px;padding:2px 12px;margin-bottom:2px">'
    + row('演员','薛之谦 · 绅士') + row('刚刚好','薛之谦 · 初学者')
    + '</div>'
    + '<div class="ls-sec"><span class="t">播放最多</span><span class="act">查看全部 ›</span></div>'
    + '<div style="'+surfaceBg('ls-home.most')+'border:1px solid var(--p-line);border-radius:12px;padding:2px 12px">'
    + row('天外来物','薛之谦 · 天外来物') + row('绅士','薛之谦 · 绅士')
    + '</div>'
    + '</div>'
    + lsMiniFloat(t)
    + '</div></div>';
}
function renderPreviewLsMine(){
  const t = state.themes.mobile;
  const el = dScreen(t);
  const stat = (slot, v, l) => '<div style="flex:1;display:flex;flex-direction:column;align-items:center;gap:2px">'
    + '<span style="color:var(--p-accent)">'+iconHtml(slot, t.icons, 15)+'</span>'
    + '<div style="font-size:12px;font-weight:800">'+v+'</div><div style="font-size:9px;color:var(--p-sub)">'+l+'</div></div>';
  const big3 = (v,l) => '<div style="flex:1;display:flex;flex-direction:column;align-items:center;gap:1px"><div style="font-size:15px;font-weight:800">'+v+'</div><div style="font-size:9px;color:var(--p-sub)">'+l+'</div></div>';
  const gcard = (slot, n) => '<div style="'+surfaceBg('mine.grid')+'border:1px solid var(--p-line);border-radius:10px;padding:10px 6px;display:flex;flex-direction:column;align-items:center;gap:6px;min-width:0">'
    + '<span style="color:var(--p-accent);display:inline-flex">'+iconHtml(slot, t.icons, 20)+'</span>'
    + '<div style="font-size:10px;font-weight:700">'+n+'</div></div>';
  el.innerHTML = '<div class="d-shell">'
    + lsSide(t, 'mine')
    + '<div style="flex:1;min-width:0;display:flex;flex-direction:column;position:relative">'
    + lsTop(t)
    + '<div style="flex:1;overflow:hidden;padding:4px 20px 0;display:flex;flex-direction:column;gap:10px">'
    + '<div style="'+surfaceBg('mine.user')+'border:1px solid var(--p-line);border-radius:12px;padding:12px 16px;display:flex;align-items:center;gap:12px">'
    + '<div style="width:38px;height:38px;border-radius:50%;background:var(--p-line);flex:none"></div>'
    + '<div style="flex:1;min-width:0"><div style="font-size:13px;font-weight:800">小奇</div><div style="font-size:9px;color:var(--p-sub)">管理账号与安全</div></div></div>'
    + '<div style="'+surfaceBg('mine.stats')+'border:1px solid var(--p-line);border-radius:12px;padding:12px 8px;display:flex;align-items:stretch">'
    + stat('mine.stat_listen','22 小时 40 分钟','累计听歌') + '<div style="width:1px;background:var(--p-line)"></div>'
    + stat('mine.stat_today','2 小时 10 分钟','今日时长') + '<div style="width:1px;background:var(--p-line)"></div>'
    + stat('mine.stat_count','33 首','今日首数') + '</div>'
    + '<div style="'+surfaceBg('ls-mine.count')+'border:1px solid var(--p-line);border-radius:12px;padding:12px 8px;display:flex;align-items:stretch">'
    + big3('33','收藏') + '<div style="width:1px;background:var(--p-line)"></div>'
    + big3('1','歌单') + '<div style="width:1px;background:var(--p-line)"></div>'
    + big3('12','历史') + '</div>'
    + '<div style="display:grid;grid-template-columns:repeat(4,1fr);gap:10px;align-content:start">'
    + gcard('mine.grid_favorite','喜欢') + gcard('mine.grid_recent','最近') + gcard('mine.grid_local','本地') + gcard('mine.grid_download','下载')
    + '</div>'
    + '</div>'
    + lsMiniFloat(t)
    + '</div></div>';
}
function renderPreviewLsPlayer(){
  const t = state.themes.mobile;
  const el = dScreen(t);
  el.innerHTML = '<div style="flex:1;display:flex;flex-direction:column;min-height:0">'
    + '<div style="display:flex;align-items:center;justify-content:space-between;padding:12px 20px 0;flex:none">'
    + iconHtml('ui.chevron-down', t.icons, 16)
    + '<span style="font-size:11px;font-weight:700">演员</span>'
    + iconHtml('action.share', t.icons, 15)
    + '</div>'
    + '<div style="flex:1;display:flex;align-items:center;gap:44px;padding:0 44px 0 56px;min-height:0">'
    + '<div style="width:190px;height:190px;border-radius:16px;background:var(--p-card);border:1px solid var(--p-line);flex:none;box-shadow:0 14px 40px rgba(0,0,0,.35)"></div>'
    + '<div class="ls-lyric" style="flex:1;min-width:0"><span>你又不是一个演员</span><span>别设计那些情节</span><span class="on">没意见 我不想保留</span><span>我只想看看你怎么圆</span><span>你难过的大表面</span></div>'
    + '</div>'
    + '<div style="flex:none;padding:0 30px 14px">'
    + '<div style="height:3px;border-radius:2px;background:color-mix(in srgb, var(--p-text) 16%, transparent);position:relative"><i style="position:absolute;left:0;top:0;bottom:0;width:18%;border-radius:2px;background:var(--p-accent)"></i></div>'
    + '<div style="display:flex;align-items:center;margin-top:10px">'
    + '<span style="display:flex;align-items:center;gap:13px;flex:1;color:var(--p-text)"><span style="font-size:9px;color:var(--p-sub);font-family:var(--font-mono)">00:44 / 04:21</span>'
    + iconHtml('action.download', t.icons, 16)+iconHtml('action.favorite', t.icons, 16)+'</span>'
    + '<span style="display:flex;align-items:center;gap:16px">'+iconHtml('player.mode', t.icons, 16)+iconHtml('player.prev', t.icons, 16)
    + '<span style="width:44px;height:44px;border-radius:50%;background:var(--p-accent);color:#fff;display:flex;align-items:center;justify-content:center;box-shadow:0 5px 16px color-mix(in srgb,var(--p-accent) 45%, transparent)">'+iconHtml('player.play', t.icons, 18)+'</span>'
    + iconHtml('player.next', t.icons, 16)+'<span style="font-size:12.5px;font-weight:700">词</span></span>'
    + '<span style="display:flex;align-items:center;gap:13px;flex:1;justify-content:flex-end;color:var(--p-text)"><span style="font-size:11px;font-weight:800;font-family:var(--font-mono)">HQ</span>'+iconHtml('player.speed', t.icons, 16)+iconHtml('player.queue', t.icons, 16)+iconHtml('action.more', t.icons, 16)+'</span>'
    + '</div></div></div>';
}
function renderPreviewLsSettings(){
  const t = state.themes.mobile;
  const el = dScreen(t);
  const nav = (label, on) => '<div style="padding:9px 12px;border-radius:10px;font-size:11.5px;'+(on
    ? 'background:var(--p-accent);color:#fff;font-weight:700'
    : 'color:var(--p-main)')+'">'+label+'</div>';
  const secCap = (l) => '<div style="font-size:10px;font-weight:700;color:var(--p-accent);margin:12px 0 6px">'+l+'</div>';
  const srow = (slot, n, s, v) => '<div style="'+surfaceBg('ls-settings.detail')+'border:1px solid var(--p-line);border-radius:12px;padding:11px 14px;display:flex;align-items:center;gap:10px">'
    + '<span style="color:var(--p-accent)">'+iconHtml(slot, t.icons, 15)+'</span>'
    + '<div style="flex:1;min-width:0"><div style="font-size:11.5px;font-weight:700">'+n+'</div>'
    + (s ? '<div style="font-size:9px;color:var(--p-sub);margin-top:1px">'+s+'</div>' : '') + '</div>'
    + '<span style="font-size:10px;color:var(--p-sub)">'+v+' ›</span></div>';
  el.innerHTML = '<div style="flex:1;display:flex;min-height:0;padding:16px 24px;gap:22px">'
    + '<div style="'+surfaceBg('ls-settings.nav')+'width:170px;flex:none;display:flex;flex-direction:column;border:1px solid var(--p-line);border-radius:12px;padding:12px">'
    + '<div style="font-size:16px;font-weight:800;margin-bottom:12px">设置</div>'
    + '<div style="background:var(--p-card);border:1px solid var(--p-line);border-radius:10px;padding:8px 12px;display:flex;align-items:center;gap:6px;margin-bottom:12px">'
    + iconHtml('entry.search', t.icons, 12) + '<span style="font-size:10.5px;color:var(--p-sub)">搜索设置</span></div>'
    + nav('账号', false) + nav('常规', true) + nav('外观', false) + nav('歌词', false) + nav('腕上联动', false) + nav('播放', false)
    + '</div>'
    + '<div style="width:1px;background:var(--p-line);flex:none"></div>'
    + '<div style="flex:1;min-width:0;overflow:hidden">'
    + '<div style="font-size:14px;font-weight:800;margin-bottom:2px">常规</div>'
    + secCap('语言')
    + srow('ui.info','语言','','跟随系统')
    + secCap('反馈')
    + srow('ui.wave','触觉反馈强度','点击底部导航等操作的手感震动强度','正常')
    + secCap('检测更新')
    + srow('ui.refresh','检测更新模式','启动时自动检查 App 更新','启动检测')
    + '</div></div>';
}
/* —— 横屏 · 音乐库四页（对齐截图+embedded 源码：按钮行 + 把手行卡列表） —— */
function lsPaneHead(left, right){
  return '<div style="height:40px;flex:none;display:flex;align-items:center;gap:10px;padding:0 6px">'
    + (left||'') + '<span style="flex:1"></span>' + (right||'') + '</div>';
}
function lsTab(l, on, cnt){
  return '<span style="display:inline-flex;align-items:center;gap:5px;padding:10px 2px 9px;font-size:11.5px;'+(on
    ? 'color:var(--p-accent);font-weight:800;box-shadow:inset 0 -2px 0 var(--p-accent)'
    : 'color:var(--p-main)')+'">'+l+(cnt?'<b style="font-size:10px;font-weight:700">'+cnt+'</b>':'')+'</span>';
}
function lsSvg(d, s, c){
  return '<svg viewBox="0 0 24 24" fill="'+(c||'var(--p-main)')+'" style="width:'+s+'px;height:'+s+'px;flex:none"><path d="'+d+'"/></svg>';
}
function lsTag(name){
  return '<span style="font-size:8.5px;color:var(--p-main);background:var(--p-line);border-radius:5px;padding:3px 8px;flex:none">'+name+'</span>';
}
const LS_SVG_BAR = 'M5 9.2h3V19H5zM10.6 5h2.8v14h-2.8zM16.1 13h2.8v6h-2.8z';
const LS_SVG_SCAN = 'M4 4h5v2H6v3H4zM15 4h5v5h-2V6h-3zM4 15h2v3h3v2H4zM18 15h2v5h-5v-2h3z';
const LS_SVG_CHECK = 'M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z';
const LS_SVG_SORT = 'M3 6h18v2H3zM6 11h12v2H6zM10 16h4v2h-4z';
const LS_SVG_CLOSE = 'M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z';
const LS_SVG_EDIT = 'M3 17.25V21h3.75L17.8 9.94l-3.75-3.75L3 17.25zM20.7 7.04a1 1 0 0 0 0-1.41l-2.34-2.34a1 1 0 0 0-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z';
const LS_SVG_DEL = 'M6 19a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z';
const LS_SVG_SWEEP = 'M15 16h4v2h-4zM15 8h7v2h-7zM15 12h6v2h-6zM3 18c0 1.1.9 2 2 2h6V4H5c-1.1 0-2 .9-2 2v12z';
const LS_SVG_HEART = 'M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z';
const LS_SVG_PLUS = 'M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z';
const LS_SVG_IMPORT = 'M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z';
function lsSongRow(t, n, sub, trailing, cov){
  return '<div style="'+surfaceBg('ls-lib.row', 'transparent')+'display:flex;align-items:center;gap:12px;padding:9px 2px;border-radius:10px">'
    + '<span style="color:var(--p-sub);flex:none;display:flex">'+iconHtml('lib.drag', t.icons, 14)+'</span>'
    + '<div style="width:40px;height:40px;border-radius:8px;flex:none;overflow:hidden;background:'+(cov||'var(--p-line)')+';display:flex;align-items:center;justify-content:center;color:#fff">'+(cov?'':iconHtml('ui.music-note', t.icons, 15))+'</div>'
    + '<div style="min-width:0;flex:1"><div style="font-size:11.5px;font-weight:700">'+n+'</div>'
    + '<div style="font-size:9px;color:var(--p-sub);margin-top:2px">'+sub+'</div></div>'
    + (trailing||'') + '</div>';
}
const LS_G1 = 'linear-gradient(135deg,#e2574c,#7a3b8f)';
const LS_G2 = 'linear-gradient(135deg,#d8a13a,#4a7a4c)';
const LS_G3 = 'linear-gradient(135deg,#5a6acf,#2a2a4a)';
function renderPreviewLsLocal(){
  const t = cur();
  const el = dScreen(t);
  const head = lsPaneHead(
    lsTab('全部', true, '2') + lsTab('歌手', false, '2') + lsTab('专辑', false, '2'),
    '<span style="display:flex;align-items:center;gap:12px;color:var(--p-main)">'+lsSvg(LS_SVG_PLUS,14)+lsSvg(LS_SVG_SCAN,14)+lsSvg(LS_SVG_BAR,14)+lsSvg(LS_SVG_CHECK,14)+lsSvg(LS_SVG_SORT,14)+'</span>');
  const row = (n,a,dur,cov) => lsSongRow(t, n, a,
    '<span style="font-size:9px;color:var(--p-sub);font-family:var(--font-mono);flex:none">'+dur+'</span><span style="color:var(--p-sub);flex:none">'+iconHtml('action.more', t.icons, 13)+'</span>', cov);
  el.innerHTML = '<div class="d-shell">'
    + lsSide(t, 'local')
    + '<div style="flex:1;min-width:0;display:flex;flex-direction:column;position:relative">'
    + lsTop(t)
    + '<div style="flex:1;overflow:hidden;display:flex;flex-direction:column;padding:0 24px">'
    + head
    + '<div style="flex:1;overflow:hidden;padding:4px 0">'
    + row('驾鹤西去','戴荃浦 · 驾鹤西去','05:18',LS_G1) + row('鲜花','回春丹乐队 · 鲜花','05:41',LS_G2)
    + row('你还要我怎样','薛之谦 · 意外','04:26',LS_G3)
    + '</div></div>'
    + lsMiniFloat(t)
    + '</div></div>';
}
function renderPreviewLsFav(){
  const t = cur();
  const el = dScreen(t);
  const tabWrap = (l,on) => '<span style="flex:1;display:flex;justify-content:center">'+lsTab(l,on)+'</span>';
  const head = lsPaneHead(
    '<span style="flex:1;display:flex">'+tabWrap('单曲',true)+tabWrap('歌单')+tabWrap('专辑')+'</span>',
    '<span style="width:28px;height:28px;border-radius:8px;border:1px solid var(--p-line);display:flex;align-items:center;justify-content:center;color:var(--p-main)">'+lsSvg(LS_SVG_CHECK,13)+'</span>');
  const row = (n,a,src,cov,accent) => lsSongRow(t, n, a,
    lsTag(src)
    + '<span style="flex:none;color:'+(accent?'var(--p-accent)':'var(--p-sub)')+'">'+lsSvg(LS_SVG_HEART,13,accent?'var(--p-accent)':undefined)+'</span>'
    + '<span style="color:var(--p-sub);flex:none">'+iconHtml('action.more', t.icons, 13)+'</span>', cov);
  el.innerHTML = '<div class="d-shell">'
    + lsSide(t, 'fav')
    + '<div style="flex:1;min-width:0;display:flex;flex-direction:column;position:relative">'
    + lsTop(t)
    + '<div style="flex:1;overflow:hidden;display:flex;flex-direction:column;padding:0 24px">'
    + head
    + '<div style="flex:1;overflow:hidden;padding:4px 0">'
    + row('你还要我怎样','薛之谦','在线','var(--p-accent)',true) + row('玻璃','Gareth.T','在线',LS_G3)
    + row('Sail','AWOLNATION','本地','var(--p-accent)',true)
    + '</div></div>'
    + '<div style="position:absolute;right:20px;bottom:84px;width:36px;height:36px;border-radius:50%;background:var(--p-card);border:1px solid var(--p-line);display:flex;align-items:center;justify-content:center;color:var(--p-text);box-shadow:0 4px 14px rgba(0,0,0,.3)">'+lsSvg('M12 4a8 8 0 1 0 8 8 8 8 0 0 0-8-8zm0 14a6 6 0 1 1 6-6 6 6 0 0 1-6 6zm0-9a3 3 0 1 0 3 3 3 3 0 0 0-3-3z',16)+'</div>'
    + lsMiniFloat(t)
    + '</div></div>';
}
function renderPreviewLsRecent(){
  const t = cur();
  const el = dScreen(t);
  const head = lsPaneHead('', '<span style="color:var(--p-main)">'+lsSvg(LS_SVG_SWEEP,15)+'</span>');
  const row = (n,when,src,cov) => lsSongRow(t, n, when,
    lsTag(src)
    + '<span style="color:var(--p-sub);flex:none">'+lsSvg(LS_SVG_CLOSE,12)+'</span>', cov);
  el.innerHTML = '<div class="d-shell">'
    + lsSide(t, 'recent')
    + '<div style="flex:1;min-width:0;display:flex;flex-direction:column;position:relative">'
    + lsTop(t)
    + '<div style="flex:1;overflow:hidden;display:flex;flex-direction:column;padding:0 24px">'
    + head
    + '<div style="flex:1;overflow:hidden;padding:4px 0">'
    + row('鲜花','回春丹乐队 · 今天 19:58','酷狗音乐',LS_G2) + row('驾鹤西去','戴荃浦 · 今天 19:53','酷狗音乐',LS_G1)
    + row('你还要我怎样','薛之谦 · 今天 19:48','在线','var(--p-accent)')
    + '</div></div>'
    + lsMiniFloat(t)
    + '</div></div>';
}
function renderPreviewLsSheets(){
  const t = cur();
  const el = dScreen(t);
  const cbtn = (d,s) => '<span style="width:30px;height:30px;border-radius:50%;border:1px solid var(--p-line);display:flex;align-items:center;justify-content:center;color:var(--p-main)">'+lsSvg(d,s)+'</span>';
  const head = lsPaneHead('', '<span style="display:flex;align-items:center;gap:10px">'+cbtn(LS_SVG_PLUS,13)+cbtn(LS_SVG_IMPORT,12)+'</span>');
  const card = (n,c) => '<div style="'+surfaceBg('ls-sheets.card')+'border-radius:14px;padding:12px 14px;display:flex;align-items:center;gap:12px">'
    + '<div style="width:40px;height:40px;border-radius:10px;background:var(--p-accent);display:flex;align-items:center;justify-content:center;color:#fff;flex:none">'+iconHtml('ui.music-note', t.icons, 16)+'</div>'
    + '<div style="flex:1;min-width:0"><div style="font-size:12px;font-weight:700">'+n+'</div><div style="font-size:9.5px;color:var(--p-sub);margin-top:2px">'+c+'</div></div>'
    + '<span style="color:var(--p-sub);flex:none">'+lsSvg(LS_SVG_EDIT,13)+'</span>'
    + '<span style="color:var(--p-sub);flex:none">'+lsSvg(LS_SVG_DEL,13)+'</span></div>';
  el.innerHTML = '<div class="d-shell">'
    + lsSide(t, 'sheets')
    + '<div style="flex:1;min-width:0;display:flex;flex-direction:column;position:relative">'
    + lsTop(t)
    + '<div style="flex:1;overflow:hidden;display:flex;flex-direction:column;padding:0 24px">'
    + head
    + '<div style="flex:1;overflow:hidden;padding:6px 0;display:flex;flex-direction:column;gap:8px">'
    + card('英文摇滚','45 首歌曲') + card('粤语精选','28 首歌曲')
    + '</div></div>'
    + lsMiniFloat(t)
    + '</div></div>';
}
function renderPreviewDesktop(){
  const page = previewPage || 'main';
  if(page==='playlist') return renderDesktopPlaylist();
  if(page==='player') return renderDesktopPlayer();
  if(page==='local') return renderDesktopLocal();
  if(page==='fav') return renderDesktopFav();
  if(page==='settings') return renderDesktopSettings();
  renderDesktopMain();
}
/* —— 桌面公共件 —— */
function dScreen(t){
  const el = $id('dScreen');
  el.className = 'screen ' + (t.themeMode==='light'?'light':'dark');
  el.style.setProperty('--p-accent', t.accentColor);
  return el;
}
function dTopbar(t, gearOn){
  const wbtn = svg => '<span class="d-wbtn">'+svg+'</span>';
  return '<div class="d-topbar">'
    + '<span class="d-backbtn">'+iconHtml('nav.back', t.icons, 14)+'</span>'
    + '<div class="d-searchbar">'+iconHtml('action.search', t.icons, 13)+'<span class="grow">搜索音乐...</span>'+iconHtml('action.mic', t.icons, 14)+'</div>'
    + '<span class="d-tbtn">'+iconHtml('ui.moon', t.icons, 15)+'</span>'
    + '<span class="d-tbtn">'+iconHtml('desktop.wallpaper', t.icons, 15)+'</span>'
    + '<span class="d-tbtn"'+(gearOn?' style="color:var(--p-accent)"':'')+'>'+iconHtml('desktop.settings', t.icons, 15)+'</span>'
    + '<span class="d-ava"></span>'
    + '<span class="d-wsep"></span>'
    + wbtn('<svg viewBox="0 0 10 10"><path d="M1 5h8"/></svg>')
    + wbtn('<svg viewBox="0 0 10 10"><rect x="1.5" y="1.5" width="7" height="7" rx="1"/></svg>')
    + wbtn('<svg viewBox="0 0 10 10"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7"/></svg>')
    + '</div>';
}
function dSide(t, activeIdx, pl){
  const navs = ['nav.home','ui.user','ui.calendar','ui.folder','ui.wrench','ui.sliders','ui.music-note','ui.history','action.favorite'];
  const labels = ['首页','歌手','专辑','文件夹','插件管理','个人中心','本地音乐','最近播放','我的收藏'];
  const sbSticker = t.stickers && t.stickers['sidebar.bottom'];
  return '<div class="d-side">'
    + '<div class="d-logo">'+logoHtml(t, 'desktop.logo', 22)+'<span>弦予音乐</span></div>'
    + navs.map((s,i)=>'<div class="d-nav'+(i===activeIdx?' on':'')+'">'+iconHtml(s, t.icons, 13)+'<span>'+labels[i]+'</span></div>').join('')
    + (sbSticker ? '<div class="sticker-slot">'+stickerImg(sbSticker, 56)+'</div>' : '')
    + '<div class="d-pl"><span>我的歌单 '+(pl?pl.count:0)+'</span><span class="sp"></span><span>＋</span><span>⭳</span></div>'
    + (pl ? '<div class="d-nav'+(pl.sel?' on':'')+'" style="gap:8px"><span class="dl-cover" style="width:26px;height:26px;border-radius:6px;flex:none"></span><div style="min-width:0"><div style="font-size:11px;font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">'+pl.name+'</div><div style="font-size:9px;color:var(--p-sub)">'+pl.count+' 首</div></div></div>' : '')
    + '</div>';
}
function dPlayerBar(t, opts){
  opts = opts || {};
  const playIcon = opts.playing
    ? '<svg viewBox="0 0 24 24" fill="currentColor" style="width:12px;height:12px"><rect x="7" y="5" width="3.4" height="14" rx="1.2"/><rect x="13.6" y="5" width="3.4" height="14" rx="1.2"/></svg>'
    : iconHtml('player.play', t.icons, 12);
  return '<div class="d-player">'
    + '<div class="d-now"><span class="cov"></span><div style="flex:1;min-width:0"><div class="n">'+(opts.name||'A.I.N.Y. 爱你')+'</div><div class="a">'+(opts.artist||'G.E.M.邓紫棋')+'</div></div></div>'
    + '<span class="d-ctl"><span class="fav">'+iconHtml('action.favorite', t.icons, 15)+'</span><span style="display:inline-flex">'+iconHtml('action.download', t.icons, 15)+'</span></span>'
    + '<div class="d-mid"><div class="d-ctl">'
    + iconHtml('player.mode', t.icons, 14)
    + iconHtml('player.prev', t.icons, 14)
    + '<span class="play">'+playIcon+'</span>'
    + iconHtml('player.next', t.icons, 14)
    + iconHtml('player.lyric', t.icons, 14)
    + '</div></div>'
    + '<div class="d-right">'
    + '<span class="sq">'+(opts.hq||'SQ')+'</span>'
    + iconHtml('player.comment', t.icons, 14)
    + iconHtml('player.volume', t.icons, 14)
    + iconHtml('player.sound', t.icons, 14)
    + iconHtml('player.queue', t.icons, 14)
    + '<span style="display:inline-flex;transform:rotate(180deg)">'+iconHtml('ui.chevron-down', t.icons, 14)+'</span>'
    + '</div>'
    + '</div>';
}
function dRow(t, o){
  const c1 = o.playing ? '<span class="eq"><i></i><i></i><i></i></span>'
    : (o.drag ? '<span class="drag">☰</span>' : '<span class="no">'+o.no+'</span>');
  return '<div class="dl-row'+(o.hl?' hl':'')+'">'+c1+'<span class="cov"></span>'
    + '<div style="min-width:0"><div class="tt"><span class="n">'+o.name+'</span>'+(o.dur?'<span class="dur">（时长: '+o.dur+'）</span>':'')+'</div><div class="a">'+o.artist+'</div></div>'
    + '<span class="al">'+o.album+'</span>'
    + '<span class="fmt">FLAC</span>'
    + '<span class="heart'+(o.heart?' on':'')+'">'+iconHtml('action.favorite', t.icons, 13)+'</span>'
    + '<span class="d">'+o.time+'</span>'
    + '<span class="tag">本地</span>'
    + '</div>';
}
function renderDesktopPlaylist(){
  const t = state.themes.desktop;
  const rows = [
    { no:'01', name:'Alone (Restrun...', dur:'3:05', artist:'Alan Walker', album:'Alone', time:'00:00' },
    { no:'02', name:'Sooner Or Later', dur:'3:32', artist:'Aaron Carter', album:'LoVe (Explicit)', time:'00:00', hl:true, drag:true, heart:true },
    { no:'03', name:'Paris', dur:'3:41', artist:'The Chainsmokers', album:'Paris', time:'00:00' },
    { no:'04', name:'H.O.L.Y.', dur:'3:14', artist:'Florida Georgia Line', album:'Country Music Awards, V...', time:'00:00' },
    { no:'05', name:'Love Yourself', dur:'4:52', artist:'Justin Bieber', album:'Purpose', time:'00:00' }
  ];
  dScreen(t).innerHTML =
    dTopbar(t, false)
    + '<div class="d-shell">' + dSide(t, -1, { count:1, name:'英文摇滚', sel:true })
    + '<div class="dl-wrap">'
    + '<div class="dl-head"><span class="dl-cover"></span>'
    + '<div class="dl-info"><div class="dl-title">英文摇滚'+iconHtml('action.more', t.icons, 14)+'</div>'
    + '<div class="dl-ops"><span class="dl-pill">'+iconHtml('page.playall', t.icons, 12)+'全部播放</span><span class="dl-round">'+iconHtml('page.fav', t.icons, 12)+'</span><span class="dl-round">'+iconHtml('page.sort', t.icons, 12)+'</span><span class="dl-round">'+iconHtml('page.more', t.icons, 12)+'</span></div>'
    + '</div></div>'
    + '<div class="dl-body">'+rows.map(r=>dRow(t, r)).join('')+'</div>'
    + '</div></div>'
    + dPlayerBar(t, { playing:true, hq:'HR', name:'爱相随', artist:'周华健' });
}
function renderDesktopPlayer(){
  const t = state.themes.desktop;
  const stCorner = t.stickers && t.stickers['player.corner'];
  dScreen(t).innerHTML =
    '<div class="dp">'
    + '<div class="dp-name" style="margin-top:14px">All We Know <span class="dp-artist" style="margin-top:0;display:inline">- The Chainsmokers, Phoebe Ryan</span></div>'
    + '<div class="dp-body"><div class="dp-cover"></div>'
    + '<div class="dp-lyric">'
    + '<div class="prev">\'Cause this is all we know</div>'
    + '<div class="prev">echoes in the dark,</div>'
    + '<div class="cur">\'Cause this is all we know</div>'
    + '<div class="trans">这就是我们共同的拥有</div>'
    + '<div class="prev">Never face each other</div>'
    + '</div>'
    + '</div>'
    + (stCorner ? '<div class="d-sticker-corner">'+stickerImg(stCorner, 56)+'</div>' : '')
    + '<div class="dp-prog"></div>'
    + '<div class="dp-ctrl">'
    + '<span class="dp-time">01:23 <span class="tot">/ 03:14</span></span>'
    + iconHtml('action.favorite', t.icons, 15)
    + '<span class="ok">✓</span>'
    + '<span class="grow"></span>'
    + iconHtml('player.mode', t.icons, 15)
    + iconHtml('player.prev', t.icons, 15)
    + '<span class="play">'+iconHtml('player.play', t.icons, 14)+'</span>'
    + iconHtml('player.next', t.icons, 15)
    + (t.icons['player.lyric'] ? iconHtml('player.lyric', t.icons, 15) : '<span class="word">词</span>')
    + '<span class="grow"></span>'
    + '<span class="sq" style="border-color:rgba(255,255,255,.6);color:#fff">SQ</span>'
    + iconHtml('player.comment', t.icons, 15)
    + iconHtml('player.volume', t.icons, 15)
    + iconHtml('player.sound', t.icons, 15)
    + iconHtml('player.queue', t.icons, 15)
    + '<span class="expand">'+iconHtml('ui.chevron-down', t.icons, 13)+'</span>'
    + '</div>'
    + '</div>';
}
function renderDesktopLocal(){
  const t = state.themes.desktop;
  const rows = [
    { no:'01', name:'A.I.N.Y. 爱你', artist:'G.E.M.邓紫棋', album:'18', time:'03:46', playing:true },
    { no:'02', name:'爱得太迟 (Live)', artist:'容祖儿、古巨基', album:'PRETTY CRAZY JOEY YU...', time:'04:08' },
    { no:'03', name:'爱相随', artist:'周华健', album:'爱相随', time:'03:35' },
    { no:'04', name:'海洋之星', artist:'亚洲之星', album:'未知专辑', time:'02:00', hl:true, drag:true, heart:true },
    { no:'05', name:'默 (Live)', artist:'李荣浩、周杰伦', album:'2021中国好声音 第1期', time:'02:13' },
    { no:'06', name:'我爱你 Luv Is Luv', artist:'贺仙人', album:'未知专辑', time:'03:21' }
  ];
  dScreen(t).innerHTML =
    dTopbar(t, false)
    + '<div class="d-shell">' + dSide(t, 6, null)
    + '<div class="dl-wrap">'
    + '<div class="dl-pagebar"><span class="t">本地音乐</span><span class="sp"></span>'
    + '<div class="dl-tools"><span class="dl-round">'+iconHtml('page.playall', t.icons, 11)+'</span><span class="dl-round">'+iconHtml('page.sort', t.icons, 11)+'</span><span class="dl-round">'+iconHtml('page.more', t.icons, 11)+'</span></div></div>'
    + '<div class="dl-body">'+rows.map(r=>dRow(t, r)).join('')+'</div>'
    + '</div></div>'
    + dPlayerBar(t, { hq:'SQ' });
}
function renderDesktopFav(){
  const t = state.themes.desktop;
  const rows = [
    { no:'01', name:'鲜花', dur:'5:41', artist:'回春丹', album:'鲜花', time:'05:41' },
    { no:'02', name:'I Really Want t...', dur:'4:06', artist:'Cyberpunk', album:'Cyberpunk : Edgerunners...', time:'04:06' },
    { no:'03', name:'7 Years', dur:'3:58', artist:'Lukas Graham', album:'Lukas Graham (Blue Albu...', time:'03:58' },
    { no:'04', name:'I Still Do', dur:'3:22', artist:'Mokita', album:'4201', time:'03:22' },
    { no:'05', name:'Sail', dur:'4:19', artist:'AWOLNATION', album:'egoFM, Vol. 1', time:'04:19' },
    { no:'06', name:'One Day', dur:'3:27', artist:'MatisYahu', album:'Light', time:'03:27' }
  ].map(r=>({ ...r, heart:true }));
  dScreen(t).innerHTML =
    dTopbar(t, false)
    + '<div class="d-shell">' + dSide(t, 8, null)
    + '<div class="dl-wrap">'
    + '<div class="dl-pagebar"><div class="dl-tabs"><span class="on">单曲</span><span>歌单</span><span>专辑</span></div><span class="sp"></span>'
    + '<div class="dl-tools"><span class="dl-round">'+iconHtml('page.playall', t.icons, 11)+'</span><span class="dl-round">'+iconHtml('page.fav', t.icons, 11)+'</span><span class="dl-round">'+iconHtml('page.sort', t.icons, 11)+'</span><span class="dl-round">'+iconHtml('page.more', t.icons, 11)+'</span></div></div>'
    + '<div class="dl-body">'+rows.map(r=>dRow(t, r)).join('')+'</div>'
    + '</div></div>'
    + dPlayerBar(t, { playing:true, hq:'HR', name:'爱相随', artist:'周华健' });
}
function renderDesktopSettings(){
  const t = state.themes.desktop;
  const groups = ['账号','常规','外观','音源','播放','下载','音乐库','工具箱','桌面歌词','快捷按键','高级设置'];
  const item = (n, s, ctrl) => '<div class="ds-row"><div class="grow"><div class="n">'+n+'</div>'+(s?'<div class="s">'+s+'</div>':'')+'</div>'+ctrl+'</div>';
  dScreen(t).innerHTML =
    dTopbar(t, true)
    + '<div class="d-shell">' + dSide(t, -1, null)
    + '<div class="dset">'
    + '<div class="ds-nav"><div class="ds-search">'+iconHtml('action.search', t.icons, 11)+'搜索设置</div>'
    + groups.map((g,i)=>'<div class="ds-item'+(i===1?' on':'')+'">'+g+'</div>').join('')
    + '</div>'
    + '<div class="ds-main">'
    + '<div class="ds-sec">语言</div>'
    + '<div class="ds-card">'+item('软件语言', '选择界面显示语言，切换后立即生效。', '<span class="ds-sel">简体中文</span>')+'</div>'
    + '<div class="ds-sec">常规与启动</div>'
    + '<div class="ds-card">'
    + item('开机自动运行', '', '<span class="dsw"></span>')
    + item('启动检测更新', '', '<span class="dsw on"></span>')
    + item('GPU 加速', '', '<span class="dsw on"></span>')
    + item('性能模式', '低性能设备自动收缩毛玻璃与动态特效，改善流畅度', '<span class="ds-sel">自动 (满特效)</span>')
    + item('关闭时最小化到托盘', '', '<span class="dsw on"></span>')
    + '</div>'
    + '</div>'
    + '</div></div>'
    + dPlayerBar(t, { hq:'SQ' });
}
function renderDesktopMain(){
  const t = state.themes.desktop;
  const el = $id('dScreen');
  el.className = 'screen ' + (t.themeMode==='light'?'light':'dark');
  el.style.setProperty('--p-accent', t.accentColor);
  const navs = ['nav.home','ui.user','ui.calendar','ui.folder','ui.wrench','ui.sliders','ui.music-note','ui.history','action.favorite'];
  const navLabels = ['首页','歌手','专辑','文件夹','插件管理','个人中心','本地音乐','最近播放','我的收藏'];
  const sbSticker = t.stickers && t.stickers['sidebar.bottom'];
  const stCorner = t.stickers && t.stickers['player.corner'];
  const wbtn = (svg, cls) => '<span class="d-wbtn'+(cls||'')+'">'+svg+'</span>';
  const wbMin = '<svg viewBox="0 0 10 10"><path d="M1 5h8"/></svg>';
  const wbMax = '<svg viewBox="0 0 10 10"><rect x="1.5" y="1.5" width="7" height="7" rx="1"/></svg>';
  const wbCls = '<svg viewBox="0 0 10 10"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7"/></svg>';
  const stat = (cap, big, hero) => '<div class="d-stat'+(hero?' hero':'')+'"><div class="cap">'+cap+'</div><div class="big">'+big+'</div></div>';
  const rankRow = (badge, badgeCls, name, sub, val, rowCls, you) =>
    '<div class="d-row'+(rowCls?' '+rowCls:'')+'"><span class="badge'+(badgeCls?' '+badgeCls:'')+'">'+badge+'</span><span class="av"></span>'
    + '<div class="grow"><div class="n">'+name+(you?'<span class="you">你</span>':'')+'</div><div class="a">'+sub+'</div></div>'
    + '<span class="val">'+val+'</span></div>';
  el.innerHTML =
    '<div class="d-topbar">'
    + '<span class="d-backbtn">'+iconHtml('nav.back', t.icons, 14)+'</span>'
    + '<div class="d-searchbar">'+iconHtml('action.search', t.icons, 13)+'<span class="grow">搜索音乐...</span>'+iconHtml('action.mic', t.icons, 14)+'</div>'
    + '<span class="d-tbtn">'+iconHtml('ui.moon', t.icons, 15)+'</span>'
    + '<span class="d-tbtn">'+iconHtml('ui.palette', t.icons, 15)+'</span>'
    + '<span class="d-tbtn">'+iconHtml('ui.gear', t.icons, 15)+'</span>'
    + '<span class="d-ava"></span>'
    + '<span class="d-wsep"></span>'
    + wbtn(wbMin) + wbtn(wbMax) + wbtn(wbCls)
    + '</div>'
    + '<div class="d-shell">'
    + '<div class="d-side">'
    + '<div class="d-logo">'+logoHtml(t, 'desktop.logo', 22)+'<span>弦予音乐</span></div>'
    + navs.map((s,i)=>'<div class="d-nav'+(i===0?' on':'')+'">'+iconHtml(s, t.icons, 13)+'<span>'+navLabels[i]+'</span></div>').join('')
    + (sbSticker ? '<div class="sticker-slot">'+stickerImg(sbSticker, 56)+'</div>' : '')
    + '<div class="d-pl"><span>我的歌单 0</span><span class="sp"></span><span>＋</span><span>⭳</span></div>'
    + '</div>'
    + '<div class="d-main">'
    + '<div class="d-tabs"><span class="on">统计</span><span>每日推荐</span><span>音源榜单</span></div>'
    + '<div class="d-stats">'
    + stat('总歌曲', '103', true)
    + stat('歌曲总时长', '5小时 52分钟')
    + stat('库大小', '3.06 GB')
    + stat('无损占比', '87%')
    + stat('总听歌时长', '13小时 37分钟')
    + stat('播放次数', '251')
    + stat('常听歌曲', '纳塔 Natian')
    + '</div>'
    + '<div class="d-rankh"><span class="t">听歌排行榜</span><span class="s">单日听歌时长排行</span><span class="sp"></span>'
    + '<div class="d-seg"><span class="on">日榜</span><span>周榜</span><span>总榜</span></div>'
    + '<span class="d-tbtn">'+iconHtml('ui.refresh', t.icons, 13)+'</span></div>'
    + '<div class="d-rank">'
    + rankRow('1', 'g1', '梦梦', '@梦梦', '2小时32分', 'hot')
    + rankRow('2', 'g2', '向日葵', '@向日葵', '1小时38分', '')
    + rankRow('4', '', '小奇', '@小奇', '0分钟', 'me', true)
    + '</div>'
    + (stCorner ? '<div class="d-sticker-corner">'+stickerImg(stCorner, 56)+'</div>' : '')
    + '</div>'
    + '</div>'
    + '<div class="d-player">'
    + '<div class="d-now"><span class="cov"></span><div style="flex:1;min-width:0"><div class="n">A.I.N.Y. 爱你</div><div class="a">G.E.M.邓紫棋</div></div></div>'
    + '<span class="d-ctl"><span class="fav">'+iconHtml('action.favorite', t.icons, 15)+'</span><span style="display:inline-flex">'+iconHtml('action.download', t.icons, 15)+'</span></span>'
    + '<div class="d-mid"><div class="d-ctl">'
    + iconHtml('player.mode', t.icons, 14)
    + iconHtml('player.prev', t.icons, 14)
    + '<span class="play">'+iconHtml('player.play', t.icons, 12)+'</span>'
    + iconHtml('player.next', t.icons, 14)
    + iconHtml('player.lyric', t.icons, 14)
    + '</div></div>'
    + '<div class="d-right">'
    + '<span class="sq">SQ</span>'
    + iconHtml('player.comment', t.icons, 14)
    + iconHtml('player.volume', t.icons, 14)
    + iconHtml('player.sound', t.icons, 14)
    + iconHtml('player.queue', t.icons, 14)
    + '<span style="display:inline-flex;transform:rotate(180deg)">'+iconHtml('ui.chevron-down', t.icons, 14)+'</span>'
    + '</div>'
    + '</div>';
}

/* ---------- 平台切换 ---------- */
$id('platSeg').addEventListener('click', e => {
  const b = e.target.closest('button'); if(!b) return;
  platform = b.dataset.p;
  [...$id('platSeg').children].forEach(x=>x.classList.toggle('on', x===b));
  $id('wallpaperRef').value = cur().wallpaperRef ? cur().wallpaperRef.id : '';
  $id('shapeCard').style.display = 'none';
  $id('mobileWrap').style.display = (platform==='mobile' && orientation==='portrait') ? '' : 'none';
  $id('desktopWrap').style.display = (platform==='desktop' || (platform==='mobile' && orientation==='landscape')) ? '' : 'none';
  $id('oriField').style.display = platform==='mobile' ? '' : 'none';
  previewPage = (visiblePages()[0] || {}).id || '';
  slotScope = 'page';
  renderStageTabs();
  renderScopeSeg();
  renderSwatches();
  [...$id('modeSeg').children].forEach(x=>x.classList.toggle('on', x.dataset.m===cur().themeMode));
  [...$id('shapeSeg').children].forEach(x=>x.classList.toggle('on', x.dataset.s===cur().quickEntryShape));
  renderSlotList('icons','iconSlots');
  renderSlotList('stickers','stickerSlots');
  renderSurfaceList();
  renderPreview();
});
$id('oriSeg').addEventListener('click', e => {
  const b = e.target.closest('button'); if(!b || b.dataset.o === orientation) return;
  orientation = b.dataset.o;
  [...$id('oriSeg').children].forEach(x=>x.classList.toggle('on', x===b));
  previewPage = (visiblePages()[0] || {}).id || '';
  slotScope = 'page';
  $id('mobileWrap').style.display = orientation==='portrait' ? '' : 'none';
  $id('desktopWrap').style.display = orientation==='landscape' ? '' : 'none';
  renderStageTabs();
  renderScopeSeg();
  renderSlotList('icons','iconSlots');
  renderSlotList('stickers','stickerSlots');
  renderSurfaceList();
  renderPreview();
});
$id('modeSeg').addEventListener('click', e => {
  const b = e.target.closest('button'); if(!b) return;
  cur().themeMode = b.dataset.m;
  [...$id('modeSeg').children].forEach(x=>x.classList.toggle('on', x===b));
  renderPreview();
});
$id('shapeSeg').addEventListener('click', e => {
  const b = e.target.closest('button'); if(!b) return;
  cur().quickEntryShape = b.dataset.s;
  [...$id('shapeSeg').children].forEach(x=>x.classList.toggle('on', x===b));
  renderPreview();
});

/* ---------- 登录 ---------- */
let pollTimer = null, pollCode = '';
function openLogin(){
  if(auth){ toast('已登录：'+auth.nickname); return; }
  $id('loginModal').classList.add('open');
  startLogin();
}
function closeLogin(){
  $id('loginModal').classList.remove('open');
  if(pollTimer){ clearInterval(pollTimer); pollTimer=null; }
}
async function startLogin(){
  $id('qrBox').innerHTML = '<div class="hint" style="text-align:center">正在生成二维码…</div>';
  $id('qrCodeText').textContent = '';
  try {
    const resp = await fetch('/api?action=generate_tv_login_code', {
      method:'POST', headers:{'Content-Type':'application/json'},
      body: JSON.stringify({ device_id: deviceId, location: '主题编辑器网页' })
    });
    const j = await resp.json();
    if(j.code !== 200 || !j.data || !j.data.code) { toast(j.msg || '生成二维码失败'); closeLogin(); return; }
    pollCode = j.data.code;
    const data = 'xianyumusic://tvlogin/' + pollCode;
    $id('qrBox').innerHTML = '<img src="/theme-editor/qrcode?data=' + encodeURIComponent(data) + '" alt="二维码">';
    $id('qrCodeText').textContent = pollCode;
    const expireAt = Date.now() + (j.data.expire_seconds || 300) * 1000;
    if(pollTimer) clearInterval(pollTimer);
    pollTimer = setInterval(async () => {
      if(Date.now() > expireAt){ clearInterval(pollTimer); pollTimer=null; toast('二维码已过期，请重新打开'); $id('qrBox').innerHTML='<div class="hint" style="text-align:center">二维码已过期，关闭后重试</div>'; return; }
      try {
        const r2 = await fetch('/api?action=poll_tv_login_status', {
          method:'POST', headers:{'Content-Type':'application/json'},
          body: JSON.stringify({ code: pollCode, device_id: deviceId })
        });
        const j2 = await r2.json();
        if(j2.code === 200 && j2.data && j2.data.status === 'logged_in'){
          clearInterval(pollTimer); pollTimer = null;
          auth = { token: j2.data.token, nickname: j2.data.nickname || j2.data.username || '', ciyuanxi_id: j2.data.ciyuanxi_id || '' };
          localStorage.setItem('themeEditorAuth', JSON.stringify(auth));
          syncLoginUi();
          closeLogin();
          toast('登录成功，欢迎 ' + (auth.nickname || auth.ciyuanxi_id));
        } else if(j2.code !== 200){
          clearInterval(pollTimer); pollTimer=null;
        }
      } catch(e){}
    }, 2000);
  } catch(e){ toast('网络错误，请重试'); }
}
function syncLoginUi(){
  if(auth){
    $id('loginText').textContent = '已登录：' + (auth.nickname || auth.ciyuanxi_id);
    $id('loginState').classList.add('on');
    $id('btnLogin').style.display = 'none';
    $id('btnLogout').style.display = '';
  } else {
    $id('loginText').textContent = '未登录（导出不需要登录）';
    $id('loginState').classList.remove('on');
    $id('btnLogin').style.display = '';
    $id('btnLogout').style.display = 'none';
  }
}
function logout(){ auth = null; localStorage.removeItem('themeEditorAuth'); syncLoginUi(); toast('已退出登录'); }
function api(url, body){
  return fetch(url, { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(body || {}) })
    .then(r => r.json())
    .catch(() => ({ code: -1, msg: '网络错误' }));
}

/* ---------- 上传 ---------- */
function openUpload(){
  if(!auth){ toast('请先扫码登录后再上传'); openLogin(); return; }
  if(!state.name.trim()){ toast('请先填写主题名称'); return; }
  previewData = '';
  $id('previewDrop').innerHTML = '点击选择预览图';
  $id('upName').value = state.name;
  $id('upDesc').value = state.description;
  $id('upPlatformHint').textContent = '当前将上传「' + (platform==='mobile'?'移动端':'桌面端') + '」主题包';
  $id('uploadModal').classList.add('open');
}
function closeUpload(){ $id('uploadModal').classList.remove('open'); }
function pickPreview(inp){
  const f = inp.files && inp.files[0];
  if(!f) return;
  if(f.size > 8*1024*1024){ toast('预览图请控制在 8MB 以内'); return; }
  const r = new FileReader();
  r.onload = () => {
    previewData = String(r.result);
    $id('previewDrop').innerHTML = '<img src="'+previewData+'" alt="预览图">';
  };
  r.readAsDataURL(f);
}
async function doUpload(){
  if(!auth){ toast('登录状态丢失，请重新登录'); closeUpload(); return; }
  if(!previewData){ toast('请选择预览图'); return; }
  state.name = $id('upName').value.trim();
  state.description = $id('upDesc').value.trim();
  const payload = Object.assign({}, cur());
  const body = {
    token: auth.token,
    name: state.name,
    description: state.description,
    platform: platform,
    payload: payload,
    preview: previewData,
  };
  if(payload.wallpaperRef && payload.wallpaperRef.id) body.wallpaperRef = payload.wallpaperRef;
  const btn = $id('uploadModal').querySelector('.btn--primary');
  btn.disabled = true; btn.textContent = '上传中…';
  const j = await api('/theme-editor/upload', body);
  btn.disabled = false; btn.textContent = '确认上传';
  if(j.code === 200){
    closeUpload();
    toast('上传成功（' + (j.data && j.data.status === 'normal' ? '已通过机审' : j.data && j.data.status === 'rejected' ? '未通过机审' : '等待管理员审核') + '）', 4000);
  } else {
    toast(j.msg || '上传失败');
  }
}

/* ---------- 导出 ---------- */
function exportJson(){
  if(!state.name.trim()){ toast('请先填写主题名称'); return; }
  const pkg = {
    version: 2,
    platform: platform,
    name: state.name.trim(),
    author: auth ? (auth.nickname || auth.ciyuanxi_id) : '',
    preview: '',
    payload: cur(),
  };
  const blob = new Blob([JSON.stringify(pkg, null, 2)], { type:'application/json' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = state.name.trim().replace(/[\\/:*?"<>|]/g,'_') + '.json';
  a.click();
  setTimeout(()=>URL.revokeObjectURL(a.href), 3000);
  toast('已导出 .json 主题包，可在客户端「主题中心 → 导入」中使用');
}

/* ---------- init ---------- */
$id('themeName').value = state.name;
$id('oriField').style.display = '';
renderSwatches();
renderStageTabs();
renderScopeSeg();
renderSlotList('icons','iconSlots');
renderSlotList('stickers','stickerSlots');
renderSurfaceList();
renderPreview();
syncLoginUi();
</script>
</body>
</html>
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_json_is_valid_and_editable_by_replace_marker() {
        let v: Value = serde_json::from_str(SLOTS_JSON).expect("SLOTS_JSON must be valid JSON");
        assert!(v.get("platforms").and_then(|p| p.get("mobile")).is_some());
        assert!(v.get("platforms").and_then(|p| p.get("desktop")).is_some());
        let html = EDITOR_HTML.replace("/*__SLOTS_JSON__*/", SLOTS_JSON);
        assert!(!html.contains("/*__SLOTS_JSON__*/"));
        assert!(html.contains("const SLOTS ="));
    }

    #[test]
    fn qrcode_svg_renders_for_tv_login_payload() {
        let svg = qrcode_svg("xianyumusic://tvlogin/0123456789abcdef0123456789abcdef").expect("svg");
        assert!(svg.contains("<svg"));
        assert!(qrcode_svg("").is_none());
    }
}
