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
  return { accentColor:'#EC4141', themeMode:'dark', wallpaperRef:null, quickEntryShape:'circle', icons:{}, stickers:{}, surfaces:{}, wallpapers:{} };
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
/* ---------- v3 页面壁纸 ---------- */
/* 参数与客户端 CustomBackground 对齐：百分比整数，blur 渲染 ×0.6，maskAlpha 为黑色遮罩不透明度 */
function wpDefault(key){ return key === 'scale' || key === 'landscapeScale' ? 100 : key === 'maskAlpha' ? 40 : key === 'blur' ? 20 : key === 'opacity' ? 100 : 0; }
function renderWallpaperCard(){
  const body = $id('wpBody'); if(!body) return;
  const page = pageShortLabel();
  $id('wpCardTitle').textContent = '页面壁纸 · ' + (page || (platform === 'mobile' ? '移动端' : '桌面端'));
  const wps = cur().wallpapers || (cur().wallpapers = {});
  const wp = wps[previewPage];
  if(!wp || !wp.ref){
    body.innerHTML = '<div class="drop" style="padding:14px" onclick="document.getElementById(\'wpFile\').click()">为「'+esc(page || '本页')+'」上传壁纸</div>';
    return;
  }
  /* 缩放/位移键跟随当前预览方向：竖屏预览调竖屏参数，横屏预览调横屏参数（客户端同语义） */
  const isLs = platform === 'mobile' && orientation === 'landscape';
  const sliders = [
    [isLs ? 'landscapeScale' : 'scale','缩放',80,240],
    [isLs ? 'landscapeTranslateX' : 'translateX','水平位移',-100,100],
    [isLs ? 'landscapeTranslateY' : 'translateY','垂直位移',-100,100],
    ['blur','模糊',0,100],['opacity','不透明度',0,100],['maskAlpha','遮罩',0,100],
  ];
  body.innerHTML =
    '<div class="wp-fig"><img src="'+esc(wp.ref)+'" alt=""></div>'
    + sliders.map(s => {
        const v = wp[s[0]] == null ? wpDefault(s[0]) : wp[s[0]];
        return '<div class="wp-row"><span class="wp-label">'+s[1]+'</span>'
          + '<input type="range" min="'+s[2]+'" max="'+s[3]+'" step="1" value="'+v+'" oninput="setWp(\''+s[0]+'\', this.value)">'
          + '<span class="wp-val" id="wpv_'+s[0]+'">'+v+'</span></div>';
      }).join('')
    + '<div class="wp-ops"><button class="btn" onclick="document.getElementById(\'wpFile\').click()">更换</button>'
    + '<button class="btn danger" onclick="clearWallpaper()">清除</button></div>';
}
function setWp(key, value){
  const wps = cur().wallpapers; if(!wps || !wps[previewPage]) return;
  let v = parseInt(value, 10);
  if(isNaN(v)) v = wpDefault(key);
  wps[previewPage][key] = v;
  const el = $id('wpv_'+key); if(el) el.textContent = String(v);
  applyWallpaperLayer();
}
function pickWallpaper(inp){
  const f = inp.files && inp.files[0];
  inp.value = '';
  if(!f) return;
  if(!/^image\/(jpeg|png|webp)$/i.test(f.type)){ toast('壁纸仅支持 JPG / PNG / WEBP'); return; }
  if(f.size > 8*1024*1024){ toast('壁纸请控制在 8MB 以内'); return; }
  compressToDataUrl(f, 1080, 0.82, dataUrl => {
    const wps = cur().wallpapers || (cur().wallpapers = {});
    const wp = wps[previewPage] || {};
    wp.ref = dataUrl;
    ['blur','opacity','maskAlpha','scale','translateX','translateY',
     'landscapeScale','landscapeTranslateX','landscapeTranslateY'].forEach(k => {
      if(platform === 'desktop' && k.indexOf('landscape') === 0) return;
      if(wp[k] == null) wp[k] = wpDefault(k);
    });
    wps[previewPage] = wp;
    renderWallpaperCard();
    renderPreview();
    toast('壁纸已添加，可用滑杆调整观感');
  });
}
function clearWallpaper(){
  const wps = cur().wallpapers; if(!wps) return;
  delete wps[previewPage];
  renderWallpaperCard();
  renderPreview();
}
function compressToDataUrl(file, maxW, quality, cb){
  const img = new Image();
  const url = URL.createObjectURL(file);
  img.onload = () => {
    const k = Math.min(1, maxW / (img.width || maxW));
    const c = document.createElement('canvas');
    c.width = Math.max(1, Math.round((img.width || maxW) * k));
    c.height = Math.max(1, Math.round((img.height || maxW) * k));
    c.getContext('2d').drawImage(img, 0, 0, c.width, c.height);
    URL.revokeObjectURL(url);
    cb(c.toDataURL('image/jpeg', quality));
  };
  img.onerror = () => { URL.revokeObjectURL(url); toast('图片读取失败'); };
  img.src = url;
}
/* 预览层：渲染函数重建 innerHTML 后统一叠加，公式与客户端渲染对齐（blur×0.6） */
function applyWallpaperLayer(){
  const scr = (platform==='mobile' && orientation==='landscape') || platform==='desktop' ? $id('dScreen') : $id('mScreen');
  if(!scr) return;
  const wps = cur().wallpapers || {};
  const wp = wps[previewPage];
  let layer = scr.querySelector('.pv-wallpaper');
  let mask = scr.querySelector('.pv-wallpaper-mask');
  if(!wp || !wp.ref){
    if(layer) layer.remove();
    if(mask) mask.remove();
    return;
  }
  if(!layer){
    layer = document.createElement('div');
    layer.className = 'pv-wallpaper';
    scr.insertBefore(layer, scr.firstChild);
  }
  let inner = layer.querySelector('.pv-wallpaper-inner');
  if(!inner){
    inner = document.createElement('div');
    inner.className = 'pv-wallpaper-inner';
    layer.appendChild(inner);
  }
  inner.style.backgroundImage = 'url("'+wp.ref+'")';
  const blur = (wp.blur||0) * 0.6;
  layer.style.filter = blur > 0 ? 'blur('+blur+'px)' : 'none';
  layer.style.opacity = String((wp.opacity == null ? 100 : wp.opacity) / 100);
  if(!mask){
    mask = document.createElement('div');
    mask.className = 'pv-wallpaper-mask';
    scr.insertBefore(mask, layer.nextSibling);
  }
  const ma = wp.maskAlpha == null ? 0 : wp.maskAlpha;
  mask.style.display = ma > 0 ? '' : 'none';
  mask.style.opacity = String(ma / 100);
  layoutWallpaperInner(scr, inner, wp);
}
/* 与客户端渲染语义对齐（custom_background.dart）：
   box=按屏幕 cover 后的图片显示尺寸；scale/100 下限 1.0；
   位移按屏幕宽高百分比换算 px，并钳制在 (box*s-屏)/2 内——永不露边。 */
const _wpAspectCache = {};
function wpNaturalAspect(ref, cb){
  if(_wpAspectCache[ref] !== undefined){ cb(_wpAspectCache[ref]); return; }
  const im = new Image();
  im.onload = () => {
    const a = (im.naturalWidth > 0 && im.naturalHeight > 0) ? im.naturalWidth / im.naturalHeight : 0;
    _wpAspectCache[ref] = a; cb(a);
  };
  im.onerror = () => { _wpAspectCache[ref] = 0; cb(0); };
  im.src = ref;
}
function layoutWallpaperInner(scr, inner, wp){
  const draw = (aspect) => {
    const w = scr.offsetWidth, h = scr.offsetHeight;
    if(w <= 0 || h <= 0) return;
    const s = Math.max(0.8, (wp.scale || 100) / 100);
    let bw = w, bh = h;
    if(aspect > 0){
      if(aspect > w / h){ bw = h * aspect; } else { bh = w / aspect; }
    }
    const maxDx = Math.max(0, (bw * s - w) / 2);
    const maxDy = Math.max(0, (bh * s - h) / 2);
    const dx = Math.max(-maxDx, Math.min(maxDx, (wp.translateX || 0) / 100 * w));
    const dy = Math.max(-maxDy, Math.min(maxDy, (wp.translateY || 0) / 100 * h));
    inner.style.width = bw + 'px';
    inner.style.height = bh + 'px';
    inner.style.transform = 'translate(calc(-50% + ' + dx + 'px), calc(-50% + ' + dy + 'px)) scale(' + s + ')';
  };
  wpNaturalAspect(wp.ref, draw);
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
    const cEsc = esc(c);
    return '<div class="srow" data-sid="'+s.id+'">'
      + '<div class="srow__info"><div class="srow__n">'+esc(s.label)+'</div>'
      + '<div class="srow__sub">'+surfaceRowSub(v, cEsc, o)+'</div></div>'
      + '<input type="color" value="'+cEsc+'" oninput="setSurface(\''+s.id+'\', this.value)">'
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
  if(window.__rvEmbed) return; /* 内嵌审核：缩放由画布 transform 接管 */
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
  if(window.__rvEmbed) return;                            /* 内嵌审核画布接管缩放，避免 zoom 与 transform 双重叠加 */
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
  applyWallpaperLayer();
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
  renderWallpaperCard();
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
  renderWallpaperCard();
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
  renderWallpaperCard();
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
    version: 3,
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

/* ---------- 主题 payload 渲染（审核预览模式复用） ---------- */
function applyThemePayload(t){
  // 落库/导出的主题包为 {author,name,payload:{...}} 壳结构，渲染需解壳取内层 payload
  let pj = (t.payload_json && typeof t.payload_json === 'object') ? t.payload_json : {};
  if (pj.payload && typeof pj.payload === 'object') pj = pj.payload;
  state.name = t.name || pj.name || ('待审主题 #' + (t.id || ''));
  state.description = t.description || '';
  $id('themeName').value = state.name;
  $id('themeDesc').value = state.description;
  const p = t.platform === 'desktop' ? 'desktop' : 'mobile';
  state.themes[p] = Object.assign(defaultTheme(), pj);
  platform = p;
  [...$id('platSeg').children].forEach(x=>x.classList.toggle('on', x.dataset.p===platform));
  $id('wallpaperRef').value = cur().wallpaperRef ? cur().wallpaperRef.id : '';
  $id('shapeCard').style.display = 'none';
  $id('mobileWrap').style.display = (platform==='mobile' && orientation==='portrait') ? '' : 'none';
  $id('desktopWrap').style.display = (platform==='desktop' || (platform==='mobile' && orientation==='landscape')) ? '' : 'none';
  $id('oriField').style.display = platform==='mobile' ? '' : 'none';
  previewPage = (visiblePages()[0] || {}).id || '';
  slotScope = 'page';
  renderStageTabs(); renderScopeSeg(); renderSwatches();
  [...$id('modeSeg').children].forEach(x=>x.classList.toggle('on', x.dataset.m===cur().themeMode));
  [...$id('shapeSeg').children].forEach(x=>x.classList.toggle('on', x.dataset.s===cur().quickEntryShape));
  renderSlotList('icons','iconSlots'); renderSlotList('stickers','stickerSlots');
  renderSurfaceList(); renderWallpaperCard(); renderPreview();
  // embed：渲染后重新居中
  try{ if(document.body.classList.contains('embed-mode') && window.__rvFitCenter) setTimeout(window.__rvFitCenter, 30); }catch(e){}
  // 横竖屏条：仅移动端主题显示
  try{
    const ob = document.getElementById('rvOrientBar');
    if(ob){
      ob.style.display = platform==='mobile' ? 'flex' : 'none';
      [...ob.children].forEach(x=>x.classList.toggle('on', x.dataset.o===orientation));
    }
  }catch(e){}
  toast('已加载待审主题：' + state.name, 3500);
}
// 内嵌审核画布：滚轮缩放 + 拖拽平移
function setupReviewCanvas(){
  const stage = document.getElementById('stage');
  if(!stage) return;
  // 画布接管缩放：embed 下 applyZoom 被门禁，style.zoom 恒为 1
  window.__rvEmbed = true;
  const mobile = document.getElementById('mobileWrap');
  const desktop = document.getElementById('desktopWrap');
  if(!mobile && !desktop) return;
  let dev = mobile || desktop;
  let z = 1, tx = 0, ty = 0;
  const pick = () => {
    if(mobile && mobile.style.display !== 'none') dev = mobile;
    else if(desktop && desktop.style.display !== 'none') dev = desktop;
    else dev = mobile || desktop;
  };
  const apply = () => {
    if(!dev) return;
    const base = dev === mobile ? 'translateX(-50%) ' : '';
    dev.style.transform = base + 'translate(' + tx + 'px,' + ty + 'px) scale(' + z + ')';
    // 同步自带 zoombar 的滑条与百分比
    try{
      const zr = document.getElementById('zoomRange');
      if(zr) zr.value = Math.min(2, Math.max(0.5, z));
      const zp = document.getElementById('zoomPct');
      if(zp) zp.textContent = Math.round(z * 100) + '%';
    }catch(e){}
  };
  const reset = () => { z = 1; tx = 0; ty = 0; fitCenter(); };
  // 初始适配：在 tabs 下方可用区域内居中
  function fitCenter(){
    pick();
    if(!dev) return;
    z = 1; tx = 0; ty = 0; apply();
    const s = stage.getBoundingClientRect();
    const tabs = document.getElementById('stageTabs');
    const topInset = tabs ? Math.max(0, tabs.getBoundingClientRect().bottom - s.top) + 8 : 0;
    const availTop = s.top + topInset;
    const availH = s.height - topInset - 10;
    const d = dev.getBoundingClientRect();
    if(!s.height || !d.height || availH <= 0) return;
    ty = (availH - d.height) / 2 - (d.top - availTop);
    if(d.height > availH - 16){
      z = Math.max(0.4, Math.min(1, (availH - 16) / d.height));
      apply();
      const d2 = dev.getBoundingClientRect();
      ty += (availH - d2.height) / 2 - (d2.top - availTop);
    }
    apply();
  }
  window.__rvFitCenter = fitCenter;
  window.addEventListener('resize', fitCenter);
  setTimeout(fitCenter, 60);
  // 二次适配：壁纸/预览图异步加载后再居中一次
  setTimeout(fitCenter, 450);
  stage.addEventListener('wheel', (e) => {
    if(e.target.closest && e.target.closest('.stage-tabs,#rvOrientBar')) return; // tabs 上的滚轮交给 tabs 横滑
    e.preventDefault();
    z = Math.min(3, Math.max(0.4, z + (e.deltaY < 0 ? 0.1 : -0.1)));
    apply();
  }, { passive: false });
  let dragging = false, sx = 0, sy = 0, bx = 0, by = 0;
  // 捕获阶段监听：防预览内部 stopPropagation 吞掉拖拽起点
  stage.addEventListener('pointerdown', (e) => {
    if(e.target.closest('#rvOrientBar') || e.target.closest('.stage-tabs')) return;
    dragging = true; sx = e.clientX; sy = e.clientY; bx = tx; by = ty;
    stage.classList.add('dragging');
    try { stage.setPointerCapture(e.pointerId); } catch(err) {}
  }, true);
  stage.addEventListener('pointermove', (e) => {
    if(!dragging) return;
    tx = bx + (e.clientX - sx); ty = by + (e.clientY - sy); apply();
  });
  const stopDrag = () => { dragging = false; stage.classList.remove('dragging'); };
  stage.addEventListener('pointerup', stopDrag);
  stage.addEventListener('pointercancel', stopDrag);
  // tabs 行：拖拽/滚轮横向滑动，拖动后吞掉点击防误触
  const rvTabs = document.getElementById('stageTabs');
  if(rvTabs){
    let tDrag = false, tMoved = false, tX = 0, tL = 0, tSuppress = false;
    rvTabs.addEventListener('pointerdown', (e) => {
      tDrag = true; tMoved = false; tX = e.clientX; tL = rvTabs.scrollLeft;
    });
    rvTabs.addEventListener('pointermove', (e) => {
      if(!tDrag) return;
      const dx = e.clientX - tX;
      if(!tMoved && Math.abs(dx) > 4) tMoved = true;
      if(tMoved) rvTabs.scrollLeft = tL - dx;
    });
    const tUp = () => {
      if(tDrag && tMoved){
        tSuppress = true;
        setTimeout(() => { tSuppress = false; }, 0);
      }
      tDrag = false;
    };
    rvTabs.addEventListener('pointerup', tUp);
    rvTabs.addEventListener('pointercancel', tUp);
    rvTabs.addEventListener('click', (e) => {
      if(tSuppress){ e.stopPropagation(); e.preventDefault(); }
    }, true);
    rvTabs.addEventListener('wheel', (e) => {
      const max = rvTabs.scrollWidth - rvTabs.clientWidth;
      if(max <= 0) return;
      const d = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
      if(!d) return;
      e.preventDefault();
      rvTabs.scrollLeft += (d > 0 ? 1 : -1) * Math.max(30, Math.abs(d));
    }, { passive: false });
  }
  const obar = document.createElement('div');
  obar.id = 'rvOrientBar';
  obar.innerHTML = '<button data-o="portrait" class="on">竖屏</button><button data-o="landscape">横屏</button>';
  stage.appendChild(obar);
  obar.addEventListener('click', (e) => {
    const b = e.target.closest('button'); if(!b || b.dataset.o === orientation) return;
    orientation = b.dataset.o;
    [...obar.children].forEach(x=>x.classList.toggle('on', x===b));
    previewPage = (visiblePages()[0] || {}).id || '';
    slotScope = 'page';
    $id('mobileWrap').style.display = orientation==='portrait' ? '' : 'none';
    $id('desktopWrap').style.display = orientation==='landscape' ? '' : 'none';
    renderStageTabs();
    renderScopeSeg();
    renderSlotList('icons','iconSlots');
    renderSlotList('stickers','stickerSlots');
    renderSurfaceList();
    renderWallpaperCard();
    renderPreview();
    try{ if(window.__rvFitCenter) setTimeout(window.__rvFitCenter, 30); }catch(err){}
  });
  // 自带 zoombar 驱动画布缩放
  const setRvZoom = (nz) => { z = Math.min(3, Math.max(0.4, nz)); apply(); };
  const ezr = document.getElementById('zoomRange');
  if(ezr) ezr.addEventListener('input', (e) => setRvZoom(parseFloat(e.target.value)));
  const ezm = document.getElementById('zoomMinus');
  if(ezm) ezm.addEventListener('click', () => setRvZoom(z - 0.1));
  const ezp = document.getElementById('zoomPlus');
  if(ezp) ezp.addEventListener('click', () => setRvZoom(z + 0.1));
  const ezrs = document.getElementById('zoomReset');
  if(ezrs) ezrs.addEventListener('click', reset);
}
/* ---------- 轻量预览模式（/theme-editor?preview=1，管理后台审核弹层内嵌） ----------
 * 仅渲染父页面 postMessage 传入的主题 payload，不做任何身份校验、不调任何接口；
 * 完整对外编辑器（不带 preview 参数正常打开）仍走登录/TV 绑定流程。
 * 注意：back 域 /theme-editor 会被 nginx 301 到 topic 独立站，iframe 因此跨域，
 * 但预览纯前端渲染，跨域无影响。 */
function setupPreviewMode(){
  // 后台 iframe 内嵌预览：隐藏编辑器自有顶栏品牌与语言切换（embed-mode），
  // 并按纯查看语义隐藏编辑面板与登录/导出/上传入口（review-mode）
  document.body.classList.add('embed-mode', 'review-mode');
  setupReviewCanvas();
  window.addEventListener('message', (e) => {
    const d = e.data;
    if(!d || d.type !== 'xy_theme_preview' || !d.payload) return;
    applyThemePayload(d.payload);
  });
}
(function initPreviewMode(){
  const qs = new URLSearchParams(location.search);
  if(qs.get('preview') === '1') setupPreviewMode();
})();

/* ---------- init ---------- */
$id('themeName').value = state.name;
$id('oriField').style.display = '';
renderSwatches();
renderStageTabs();
renderScopeSeg();
renderSlotList('icons','iconSlots');
renderSlotList('stickers','stickerSlots');
renderSurfaceList();
renderWallpaperCard();
renderPreview();
syncLoginUi();

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
    // 中文优先：浏览器语言列表里 English 排前时也应显示中文（先简后繁，无中文再看英文）
    for (var i = 0; i < langs.length; i++) {
      var l = String(langs[i] || '').toLowerCase();
      if (l === 'zh-cn' || l === 'zh-sg' || l === 'zh-my' || l.indexOf('zh-hans') === 0 || l === 'zh' || l.indexOf('zh-') === -1 && l.indexOf('zh') === 0) return 'zh-CN';
    }
    for (var i = 0; i < langs.length; i++) {
      var l = String(langs[i] || '').toLowerCase();
      if (l === 'zh-tw' || l === 'zh-hk' || l === 'zh-mo' || l.indexOf('zh-hant') === 0 || /^zh-(hant|tw|hk|mo)/.test(l)) return 'zh-TW';
    }
    for (var i = 0; i < langs.length; i++) {
      var l = String(langs[i] || '').toLowerCase();
      if (l.indexOf('en') === 0) return 'en';
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
    
"＋ 新建": "+ New",
"备份文件 / 本地文件夹 / 云端导入": "Backup File / Local Folder / Cloud Import",
"本地": "Local",
"本地音乐": "Local Music",
"本页专属": "Page-Specific",
"编辑平台（两端各自独立成包）": "Edit Platform (mobile and desktop are packaged separately)",
"播放最多": "Most Played",
"查看全部 ›": "View All ›",
"常规": "General",
"常规与启动": "General & Startup",
"词": "Lyrics",
"打开弦予音乐 App（移动端）→ 扫一扫，确认登录后本页自动完成登录。": "Open the XianYu Music App (mobile) → Scan the QR code. This page will sign in automatically after you confirm.",
"单曲": "Tracks",
"单日听歌时长排行": "Daily Listening Time Ranking",
"导出 JSON": "Export JSON",
"导航": "Navigation",
"导入外部歌单": "Import External Playlist",
"点击麦克风开始识别": "Tap the microphone to start recognition",
"点击选择预览图": "Click to select a preview image",
"调配配色与图标，导出或上传广场": "Adjust colors and icons, then export or upload to the Square",
"二维码": "QR Code",
"二维码已过期，关闭后重试": "QR code expired. Close this and try again.",
"发现": "Discover",
"放大": "Zoom In",
"封面": "Cover",
"该平台暂无公共槽位。": "No common slots on this platform.",
"歌词": "Lyrics",
"歌单": "Playlist",
"历史": "History",
"收藏": "Favorites",
"今日首数": "Tracks today",
"累计听歌": "Total listening",
"今日时长": "Time today",
"个人中心": "Personal Center",
"公共控件设置一次、所有页面统一生效：底部导航（发现/我的）、mini 播放条三键（上一首/播放/下一首，播放页控制行也用同一套）、搜索框公共件（放大镜/识曲钮）。": "Set common controls once and they apply uniformly across all pages: the bottom navigation (Discover / Me), the three mini player buttons (Previous / Play / Next — the Now Playing control row uses the same set), and the shared search box parts (magnifier / song recognition button).",
"公共通用": "Common",
"管理账号与安全": "Manage account and security",
"横屏": "Landscape",
"基本信息": "Basic Info",
"简介": "Description",
"简介（可选）": "Description (optional)",
"简体中文": "Simplified Chinese",
"胶囊": "Capsule",
"今天听歌时长": "Listening Time Today",
"今天已听": "Listened Today",
"快捷入口形状": "Quick Entry Shape",
"累计听歌总时长": "Total Listening Time",
"每日推荐": "Daily Picks",
"屏幕方向（横竖屏是两套 UI）": "Screen Orientation (landscape and portrait are two separate UIs)",
"浅色": "Light",
"强调色": "Accent Color",
"清除": "Clear",
"取消": "Cancel",
"确认上传": "Confirm Upload",
"日榜": "Daily Chart",
"如 123": "e.g. 123",
"扫码登录": "Scan to Log In",
"扫码登录弦予号": "Scan to log in with XianYu ID",
"上传": "Upload",
"上传到主题广场": "Upload to Theme Square",
"上传广场": "Upload to Square",
"设置": "Settings",
"深浅模式": "Light/Dark Mode",
"深色": "Dark",
"竖屏": "Portrait",
"搜索歌曲、歌手、专辑": "Search songs, artists, albums",
"搜索设置": "Search Settings",
"搜索音乐...": "Search music...",
"缩小": "Zoom Out",
"贴纸槽位": "Sticker Slots",
"听歌排行榜": "Listening Ranking",
"听过最多": "Most Listened",
"统计": "Stats",
"图标槽位": "Icon Slots",
"图标建议 SVG 或高清 PNG（单文件 ≤2MB，主题资源总量 ≤5MB）。未设置的槽位在客户端回落默认图标，未知槽位将被忽略。": "Use SVG or high-resolution PNG for icons (max 2MB per file, 5MB total for all theme assets). Slots left unset fall back to default icons in the client; unknown slots are ignored.",
"推荐壁纸 id（可选，广场壁纸编号）": "Recommended wallpaper ID (optional, Square wallpaper number)",
"退出": "Exit",
"外观": "Appearance",
"为组件设置底面色块与透明度（壁纸模式下的自定义材质效果）。颜色同主题色一样可选，透明度 0%~100%；未设置的组件维持默认材质。切换「本页专属 / 公共通用」同时作用于图标、贴纸与组件色块。": "Set base color blocks and opacity for widgets (custom material effects in wallpaper mode). Colors can be any theme color; opacity ranges from 0% to 100%. Widgets left unset keep the default material. The Page-Specific / Common switch applies to icons, stickers and widget color blocks together.",
"未登录（导出不需要登录）": "Not signed in (export does not require login)",
"我的歌单 0": "My Playlists 0",
"弦予": "XianYu",
"弦予 · 主题编辑器": "XianYu · Theme Editor",
"弦予音乐": "XianYu Music",
"弦予音乐主题编辑器：调配强调色、深浅模式与图标贴纸槽位，导出主题包或上传到主题广场。": "XianYu Music Theme Editor: adjust the accent color, light/dark mode and icon & sticker slots, then export a theme pack or upload it to the Theme Square.",
"也可手动输入授权码：": "You can also enter the authorization code manually:",
"移动端": "Mobile",
"音乐": "Music",
"音乐库": "Music Library",
"音源榜单": "Source Charts",
"语言": "Language",
"预览图": "Preview Image",
"预览图（必选，JPG/PNG/WEBP/GIF ≤8MB）": "Preview image (required, JPG/PNG/WEBP/GIF ≤8MB)",
"圆形": "Circle",
"暂无搜索历史": "No search history yet",
"正在生成二维码…": "Generating QR code…",
"重置为 100%": "Reset to 100%",
"周榜": "Weekly Chart",
"主题名称": "Theme Name",
"专辑": "Album",
"桌面端": "Desktop",
"自动 (满特效)": "Auto (full effects)",
"自建歌单": "Created Playlists",
"总榜": "Overall Chart",
"组件色块": "Widget Color Blocks",
"作用于「我的」页喜欢 / 最近 / 本地 / 下载四个快捷入口的底座形状。": "Applies to the base shapes of the four quick entries on the Me page: Likes / Recent / Local / Download.",
"开机自动运行": "Launch at Startup",
"关闭时最小化到托盘": "Minimize to Tray on Close",
"启动检测更新": "Check for Updates on Launch",
"性能模式": "Performance Mode",
"低性能设备自动收缩毛玻璃与动态特效，改善流畅度": "Reduces blur and dynamic effects on low-end devices for smoother performance",
"GPU 加速": "GPU Acceleration",
"软件语言": "App Language",
"选择界面显示语言，切换后立即生效。": "Select the interface display language. Changes take effect immediately.",
"账号": "Account",
"腕上联动": "Watch Link",
"播放": "Play",
"我的": "Me",
"全部播放": "Play All",
"搜索历史": "Search History",
"听歌数据统计": "Listening Stats",
"大家都在搜": "Trending Searches",
"听歌识曲": "Song Recognition",
"共 43 首歌": "43 songs in total",
"1 次": "1 time",
"3 分钟": "3 min",
"首页": "Home",
"在线搜索": "Online Search",
"插件管理": "Plugin Management",
"反馈": "Feedback",
"文件夹": "Folders",
"下载": "Download",
"已退出登录": "Logged out",
"播放页": "Now Playing",
"搜索结果": "Search Results",
"设置页": "Settings",
"歌单页": "Playlist Page",
"主窗口 · 首页": "Main Window · Home",
"我的收藏": "My Favorites",
"最近播放": "Recently Played",
"我的歌单": "My Playlists",
"底部导航栏": "Bottom Nav Bar",
"mini 播放条": "Mini Player Bar",
"搜索框胶囊": "Search Pill",
"统计大卡": "Stats Hero Card",
"歌曲行卡": "Song Row Card",
"用户卡": "User Card",
"统计卡": "Stats Card",
"快捷宫格": "Quick Grid",
"提示卡": "Tip Card",
"歌单行卡": "Playlist Row Card",
"识别主按钮": "Recognition Main Button",
"历史/榜单卡": "History / Chart Card",
"榜单行卡": "Chart Row Card",
"tab/音源条": "Tab / Source Bar",
"音源胶囊底色（应用到所有来源，文字不变）": "Source pill base color (applies to all sources; text color unchanged)",
"横屏 · 每日推荐": "Landscape · Daily Picks",
"横屏 · 播放最多": "Landscape · Most Played",
"横屏 · 歌曲行卡（本地/收藏/最近播放）": "Landscape · Song Row Card (Local / Favorites / Recently Played)",
"横屏 · 歌单卡": "Landscape · Playlist Card",
"横屏 · 详情行卡": "Landscape · Detail Row Card",
"横屏 · 左侧导航": "Landscape · Left Navigation",
"横屏 · 数量卡（收藏/歌单/历史）": "Landscape · Count Cards (Favorites / Playlists / History)",
"顶栏（返回 + 标题）": "Top Bar (Back + Title)",
"设置分组卡": "Settings Group Card",
"搜索框 · 放大镜": "Search Box · Magnifier",
"搜索框 · 识曲钮": "Search Box · Recognition Button",
"壁纸/皮肤中心按钮（竖屏首页右上角圆钮）": "Wallpaper / Skin Center Button (portrait Home top-right round button)",
"我的 · 统计·累计听歌": "Me · Stats · Total Listening",
"我的 · 统计·今日时长": "Me · Stats · Time Today",
"我的 · 统计·今日首数": "Me · Stats · Tracks Today",
"我的 · 顶栏设置钮": "Me · Top Bar Settings Button",
"我的 · 导入歌单": "Me · Import Playlist",
"我的 · 宫格喜欢": "Me · Grid Likes",
"我的 · 宫格最近": "Me · Grid Recent",
"我的 · 宫格本地": "Me · Grid Local",
"我的 · 快捷宫格「下载」图标": "Me · Quick Grid \"Download\" icon",
"识曲 · 主按钮麦克风": "Recognition · Main Mic Button",
"底部导航 · 首页": "Bottom Nav · Home",
"底部导航 · 我的": "Bottom Nav · Me",
"播放条 · 上一首": "Player Bar · Previous",
"播放条 · 播放/暂停": "Player Bar · Play/Pause",
"播放条 · 下一首": "Player Bar · Next",
"播放页 · 上一首": "Now Playing · Previous",
"播放页 · 播放/暂停": "Now Playing · Play/Pause",
"播放页 · 下一首": "Now Playing · Next",
"播放页 · 播放队列": "Now Playing · Queue",
"播放页 · 播放模式": "Now Playing · Play Mode",
"播放页 · 收藏": "Now Playing · Favorite",
"播放页 · 下载": "Now Playing · Download",
"播放页 · 分享": "Now Playing · Share",
"播放页 · 更多": "Now Playing · More",
"播放页 · 评论": "Now Playing · Comments",
"播放页 · 倍速/音效": "Now Playing · Speed / Sound FX",
"横屏 · 侧栏品牌 Logo": "Landscape · Sidebar Brand Logo",
"横屏 · 顶栏皮肤钮": "Landscape · Top Bar Skin Button",
"横屏 · 顶栏设置钮": "Landscape · Top Bar Settings Button",
"音乐库 · 长按拖拽把手（本地/收藏/最近播放）": "Music Library · Long-press Drag Handle (Local / Favorites / Recently Played)",
"侧栏 · 品牌 Logo": "Sidebar · Brand Logo",
"侧栏 · 首页": "Sidebar · Home",
"侧栏 · 设置": "Sidebar · Settings",
"播放 · 上一首": "Playback · Previous",
"播放 · 播放/暂停": "Playback · Play/Pause",
"播放 · 下一首": "Playback · Next",
"播放 · 播放队列": "Playback · Queue",
"播放 · 播放模式": "Playback · Play Mode",
"播放 · 评论": "Playback · Comments",
"播放 · 音量": "Playback · Volume",
"播放 · 音效（均衡器）": "Playback · Sound FX (EQ)",
"播放 · 可视化（频谱）": "Playback · Visualizer (Spectrum)",
"播放 · MV": "Playback · MV",
"播放 · 歌词开关": "Playback · Lyrics Toggle",
"播放 · 进度条开关": "Playback · Progress Bar Toggle",
"播放 · 页面样式": "Playback · Page Style",
"播放 · 固定状态栏": "Playback · Pin Status Bar",
"顶栏 · 搜索": "Top Bar · Search",
"顶栏 · 识曲": "Top Bar · Recognition",
"顶栏 · 皮肤钮": "Top Bar · Skin Button",
"顶栏 · 设置钮": "Top Bar · Settings Button",
"列表页 · 播放全部钮": "List Page · Play All Button",
"列表页 · 排序钮": "List Page · Sort Button",
"列表页 · 更多钮": "List Page · More Button",
"歌单/收藏 · 收藏合集钮": "Playlist / Favorites · Favorite Collection Button",
"操作 · 收藏": "Actions · Favorite",
"操作 · 下载": "Actions · Download",
"操作 · 分享": "Actions · Share",
"操作 · 更多": "Actions · More",
"操作 · 新建歌单": "Actions · New Playlist",
"识曲页 · 底部装饰贴纸": "Recognition · Bottom Decor Sticker",
"横屏 · 侧栏左下角贴纸": "Landscape · Sidebar Bottom-left Sticker",
"右下角贴纸": "Bottom-right Sticker",
"侧栏底部贴纸": "Sidebar Bottom Sticker",
"搜索": "Search",
"识曲": "Recognize",
"返回": "Back",
"导入": "Import",
"在线": "Online",
"全部": "All",
"歌手": "Artists",
"喜欢": "Likes",
"最近": "Recent",
"音源": "Sources",
"工具": "Tools",
"偏好": "Preferences",
"你": "You",
"工具箱": "Toolbox",
"插件": "Plugins",
"高级设置": "Advanced",
"桌面歌词": "Desktop Lyrics",
"快捷按键": "Hotkeys",
"触觉反馈强度": "Haptic Feedback Strength",
"点击底部导航等操作的手感震动强度": "Vibration intensity for taps like the bottom navigation",
"插件：导入、启用、更新、卸载": "Plugins: import, enable, update, uninstall",
"音频转换、剪辑、解密、重命名": "Audio conversion, trimming, decryption, renaming",
"音质、路径、并发、嵌入": "Quality, path, concurrency, embedding",
"音量、双击播放、播放行为、输出": "Volume, double-click play, playback behavior, output",
"服务端设置、手动同步、自动同步": "Server settings, manual sync, auto sync",
"手表遥控、云端兜底、传递策略": "Watch remote, cloud fallback, handoff policy",
"歌词显示、悬浮歌词窗": "Lyrics display, floating lyrics window",
"主题、主题色、壁纸、液态玻璃、导航栏": "Theme, accent color, wallpaper, liquid glass, nav bar",
"语言、反馈、常亮、存储": "Language, feedback, always-on display, storage",
"检测更新": "Check for Updates",
"检测更新模式": "Update Check Mode",
"启动检测": "Check at Launch",
"启动时自动检查 App 更新": "Automatically check for App updates at launch",
"库大小": "Library Size",
"歌曲总时长": "Total Duration",
"总歌曲": "Total Tracks",
"总听歌时长": "Total Listening Time",
"播放次数": "Play Count",
"无损占比": "Lossless Ratio",
"常听歌曲": "Frequently Played",
"未知专辑": "Unknown Album",
"我的主题": "My Themes",
"热歌榜": "Hot Chart",
"飙升榜": "Rising Chart",
"新歌榜": "New Chart",
"视频歌曲": "Music Videos",
"万物DJ榜": "DJ Chart",
"怀旧榜": "Throwback Chart",
"影视金曲": "Movie & TV Hits",
"识别成功后可直接播放、收藏或加入歌单": "After recognition, play, favorite or add to a playlist directly",
"优先匹配本地曲库，本地没有的走在线音源解析": "Matches the local library first; falls back to online sources when missing locally",
"请先让音乐外放，再点上面的麦克风": "Play the music out loud first, then tap the microphone above",
"跟随系统": "System",
"方圆": "Squircle",
"正常": "Normal",
"未设置 · 维持默认材质": "Not set · Default material",
"上传中…": "Uploading…",
"上传失败": "Upload failed",
"网络错误": "Network error",
"网络错误，请重试": "Network error, please try again",
"请先扫码登录后再上传": "Sign in before uploading",
"请先填写主题名称": "Enter a theme name first",
"请选择预览图": "Select a preview image first",
"生成二维码失败": "Failed to generate QR code",
"登录状态丢失，请重新登录": "Sign-in state lost. Please sign in again.",
"二维码已过期，请重新打开": "QR code expired. Please reopen it.",
"已导出 .json 主题包，可在客户端「主题中心 → 导入」中使用": "Exported .json theme pack. Use it in the client via Theme Center → Import.",
"单个资源请控制在 2MB 以内": "Each file must be 2MB or smaller",
"预览图请控制在 8MB 以内": "Preview image must be 8MB or smaller",
"已通过机审": "Passed machine review",
"未通过机审": "Failed machine review",
"等待管理员审核": "Pending admin review",

  };

  /* 动态插值文案（运行时前缀/数字），正则模式表（replacement 支持字符串或函数） */
  function env(s) { return EN[s] !== undefined ? EN[s] : s; }
  var EN_MODE = [
    [/^(\d+) 首歌曲$/, '$1 tracks'],
    [/^(\d+) 首$/, '$1 tracks'],
    [/^我的歌单 (.+)$/, 'My Playlists $1'],
    [/^(\d+) 小时 (\d+) 分钟$/, '$1 hr $2 min'],
    [/^(\d+)\s*小时\s*(\d+)\s*分(?:钟)?$/, '$1 hr $2 min'],
    [/^(\d+)分钟$/, '$1 min'],
    [/^(\d+)人搜$/, '$1 searches'],
    [/^（时长: (.+)）$/, '(Duration: $1)'],
    [/^已登录：(.+)$/, 'Signed in: $1'],
    [/^登录成功，欢迎 (.+)$/, 'Signed in. Welcome, $1'],
    [/^当前将上传「(.+)」主题包$/, function (m, p) { return 'This will upload the "' + env(p) + '" theme pack'; }],
    [/^上传成功（(.+)）$/, function (m, p) { return 'Upload succeeded (' + env(p) + ')'; }],
    [/^图标槽位 · (.+)$/, function (m, p) { return 'Icon Slots · ' + env(p); }],
    [/^贴纸槽位 · (.+)$/, function (m, p) { return 'Sticker Slots · ' + env(p); }],
    [/^组件色块 · (.+)$/, function (m, p) { return 'Widget Color Blocks · ' + env(p); }],
    [/^本页 · (.+)$/, function (m, p) { return 'This Page · ' + env(p); }],
    [/^已设置 · (.+) · (\d+)%$/, 'Set · $1 · $2%'],
    [/^公共色块设置一次、所有页面统一生效：mini 播放条（竖屏底部条 \+ 横屏悬浮胶囊）、搜索框胶囊（含横屏顶栏搜索条）、底部导航栏。$/, 'Set once per color, applies to all pages: mini player bar (portrait bottom + landscape floating pill), search pill (incl. landscape top bar), and bottom nav bar.'],
    [/^公共色块设置一次、所有页面统一生效：mini 播放条（竖屏底部条 \+ 横屏悬浮胶囊）、搜索框胶囊（含横屏顶栏搜索条）。$/, 'Set once per color, applies to all pages: mini player bar (portrait bottom + landscape floating pill), search pill (incl. landscape top bar).'],
    [/^(.+?)暂无组件色块，切到「公共通用」设置全局生效的槽位。$/, '$1: No widget color blocks yet. Switch to Common to set slots that apply globally.'],
    [/^(.+?)暂无专属自定义项，切到「公共通用」设置全局生效的槽位。$/, '$1: No page-specific options yet. Switch to Common to set slots that apply globally.']
  ];

  function translate(s) {
    if (!s || !hasCJK(s)) return s;
    if (current === 'zh-TW') return toTraditional(s);
    if (current === 'en') {
      if (EN[s] !== undefined) return EN[s];
      for (var mi = 0; mi < EN_MODE.length; mi++) {
        if (EN_MODE[mi][0].test(s)) return s.replace(EN_MODE[mi][0], EN_MODE[mi][1]);
      }
      return s;
    }
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
      btn.innerHTML = '<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c2.7 2.6 4 5.6 4 9s-1.3 6.4-4 9c-2.7-2.6-4-5.6-4-9s1.3-6.4 4-9z"/></svg><span class="lang-switch__label"></span>';
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
    var curOpt = OPTS.filter(function (x) { return x.v === pref; })[0] || OPTS[0];
    var lab = wrap.querySelector('.lang-switch__label');
    if (lab) lab.textContent = optLabel(curOpt);
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

