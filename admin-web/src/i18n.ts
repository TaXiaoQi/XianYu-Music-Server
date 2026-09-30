/**
 * 弦予音乐 · 管理后台 i18n 内核
 * 与官网 i18n.js 同机制：简体为基底，运行时词典替换文本节点与常用属性。
 * zh-TW：简→繁字级映射 + 词语 override（台湾用语）；en：词典精确匹配。
 * 跟随系统/浏览器：auto 按 navigator.languages 解析（zh-TW/HK/MO/Hant→繁，en*→英，其余→简）。
 */
import { ref } from 'vue'
import { EN_ADMIN, DYN_ADMIN } from './i18n.en'

export type LangPref = 'auto' | 'zh-CN' | 'zh-TW' | 'en'
export type Lang = 'zh-CN' | 'zh-TW' | 'en'

const LS_KEY = 'xy_lang'
export const langPref = ref<LangPref>('auto')
export const langCurrent = ref<Lang>('zh-CN')

const srcMap = new WeakMap<Node, string>()
const attrMap = new WeakMap<HTMLElement, Record<string, string>>()
let suppress = false

function stored(): LangPref | null {
  try { return localStorage.getItem(LS_KEY) as LangPref | null } catch { return null }
}
function detectSystem(): Lang {
  const langs = (navigator.languages && navigator.languages.length) ? navigator.languages : [navigator.language || 'zh-CN']
  for (const raw of langs) {
    const l = String(raw || '').toLowerCase()
    if (l === 'zh-tw' || l === 'zh-hk' || l === 'zh-mo' || l.startsWith('zh-hant') || /^zh-(hant|tw|hk|mo)/.test(l)) return 'zh-TW'
    if (l.startsWith('en')) return 'en'
    if (l.startsWith('zh')) return 'zh-CN'
  }
  return 'zh-CN'
}
function resolve(p: LangPref): Lang {
  if (p === 'zh-CN' || p === 'zh-TW' || p === 'en') return p
  return detectSystem()
}

/* ===== 简→繁：词语 override（台湾用语，单轮不回扫） ===== */
const PHRASE_LIST: Array<[string, string]> = [
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
  ['游泳', '游泳'], ['皇后', '皇后'], ['手表', '手錶'],
  ['战斗', '戰鬥'], ['奋斗', '奮鬥'], ['斗争', '鬥爭'],
  ['联系', '聯繫'], ['关系', '關係'], ['细致', '細緻'], ['忧郁', '憂鬱'],
  ['浏览器', '瀏覽器'],
]
const PHRASE_MAP: Record<string, string> = {}
PHRASE_LIST.forEach(([k, v]) => { PHRASE_MAP[k] = v })
const PHRASE_RE = new RegExp(
  '(' + PHRASE_LIST.slice().sort((a, b) => b[0].length - a[0].length).map((p) => p[0].replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|') + ')',
  'g',
)

/* ===== 简→繁：字级映射（两字一组） ===== */
const S2T_SRC =
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
  '溅濺将將姜薑浆漿奖獎讲講酱醬胶膠浇澆骄驕娇嬌搅攪缴繳轿轎' +
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
  '显顯蚕蠶网網于於'
const S2T: Record<string, string> = {}
for (let i = 0; i < S2T_SRC.length; i += 2) S2T[S2T_SRC[i]] = S2T_SRC[i + 1]

function hasCJK(s: string): boolean {
  return /[\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff]/.test(s)
}
function toTraditional(s: string): string {
  s = s.replace(PHRASE_RE, (m) => PHRASE_MAP[m] || m)
  if (!hasCJK(s)) return s
  let out = ''
  for (const ch of s) out += S2T[ch] || ch
  return out
}

/* ===== EN 词典（key=简体原文） ===== */
const EN: Record<string, string> = {
  ...EN_ADMIN,
}

function translate(s: string): string {
  if (!s || !hasCJK(s)) return s
  const cur = langCurrent.value
  if (cur === 'zh-TW') return toTraditional(s)
  if (cur === 'en') {
    for (const [re, rep] of DYN_ADMIN) {
      const out = s.replace(re, rep)
      if (out !== s) return out
    }
    return EN[s] !== undefined ? EN[s] : s
  }
  return s
}

const ATTRS = ['placeholder', 'title', 'aria-label', 'data-tip']
const SKIP_TAGS = new Set(['SCRIPT', 'STYLE', 'CODE', 'PRE', 'TEXTAREA', 'NOSCRIPT'])

function apply(root: Node) {
  if (langCurrent.value === 'zh-CN') { restore(root); return }
  suppress = true
  try {
    const w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
      acceptNode: (n) => {
        if (!n.nodeValue || !n.nodeValue.trim()) return NodeFilter.FILTER_REJECT
        const p = (n as Text).parentElement
        if (p && SKIP_TAGS.has(p.tagName)) return NodeFilter.FILTER_REJECT
        return NodeFilter.FILTER_ACCEPT
      },
    })
    const nodes: Text[] = []
    while (w.nextNode()) nodes.push(w.currentNode as Text)
    for (const n of nodes) {
      const src = srcMap.has(n) ? srcMap.get(n) as string : n.nodeValue as string
      const out = src.replace(src.trim(), translate(src.trim()))
      if (out !== n.nodeValue) { srcMap.set(n, src); n.nodeValue = out }
    }
    if (root instanceof Element) {
      root.querySelectorAll('[' + ATTRS.join('],[') + ']').forEach((el) => applyAttrs(el as HTMLElement))
      applyAttrs(root as HTMLElement)
    }
  } finally { setTimeout(() => { suppress = false }, 0) }
}

function restore(root: Node) {
  suppress = true
  try {
    const w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, null)
    const nodes: Text[] = []
    while (w.nextNode()) nodes.push(w.currentNode as Text)
    for (const n of nodes) {
      if (srcMap.has(n)) { n.nodeValue = srcMap.get(n) as string; srcMap.delete(n) }
    }
    document.querySelectorAll<HTMLElement>('*').forEach((el) => {
      const saved = attrMap.get(el)
      if (saved) { for (const a in saved) el.setAttribute(a, saved[a]); attrMap.delete(el) }
    })
  } finally { setTimeout(() => { suppress = false }, 0) }
}

function applyAttrs(el: HTMLElement) {
  for (const a of ATTRS) {
    if (!el.hasAttribute(a)) continue
    const v = el.getAttribute(a) || ''
    if (!v.trim() || !hasCJK(v)) continue
    const tv = translate(v.trim())
    if (tv && tv !== v) {
      let saved = attrMap.get(el)
      if (!saved) { saved = {}; attrMap.set(el, saved) }
      if (!(a in saved)) saved[a] = v
      el.setAttribute(a, tv)
    }
  }
}

function applyToText(n: Node) {
  if (!n.nodeValue || !n.nodeValue.trim()) return
  const src = srcMap.has(n) ? srcMap.get(n) as string : n.nodeValue
  const out = src.replace(src.trim(), translate(src.trim()))
  if (out !== n.nodeValue) { srcMap.set(n, src); n.nodeValue = out }
}

let mo: MutationObserver | null = null
function ensureObserver() {
  if (mo || !document.body) return
  mo = new MutationObserver((muts) => {
    if (suppress) return
    for (const m of muts) {
      if (m.type === 'characterData') { applyToText(m.target); continue }
      if (m.type === 'attributes') { applyAttrs(m.target as HTMLElement); continue }
      for (const nd of m.addedNodes) {
        if (nd.nodeType === 3) applyToText(nd)
        else if (nd.nodeType === 1) apply(nd)
      }
    }
  })
  mo.observe(document.body, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ATTRS })
}

export function setLang(v: LangPref) {
  langPref.value = v
  try { localStorage.setItem(LS_KEY, v) } catch { /* ignore */ }
  langCurrent.value = resolve(v)
  document.documentElement.lang = langCurrent.value
  apply(document.body || document.documentElement)
}
export function getPref(): LangPref { return langPref.value }
export function t(s: string): string { return translate(s) }
export const langOpts: Array<{ v: LangPref; label: Record<Lang, string> }> = [
  { v: 'auto', label: { 'zh-CN': '跟随系统', 'zh-TW': '跟隨系統', 'en': 'System' } },
  { v: 'zh-CN', label: { 'zh-CN': '简体中文', 'zh-TW': '简体中文', 'en': '简体中文' } },
  { v: 'zh-TW', label: { 'zh-CN': '繁體中文', 'zh-TW': '繁體中文', 'en': '繁體中文' } },
  { v: 'en', label: { 'zh-CN': 'English', 'zh-TW': 'English', 'en': 'English' } },
]
export function optLabel(o: { label: Record<Lang, string> }): string { return o.label[langCurrent.value] || '跟随系统' }

export function initI18n() {
  langPref.value = stored() || 'auto'
  langCurrent.value = resolve(langPref.value)
  document.documentElement.lang = langCurrent.value
  const run = () => { apply(document.body); ensureObserver() }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', run)
  else run()
}
