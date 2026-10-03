# 兜底模块下发说明

桌面端把音源兜底行为（落雪歌曲搜索、专辑/歌单获取、时长补齐、逐字歌词、封面提取、插件宿主兜底、歌单导入）抽离为固定 key 的功能模块，内置默认实现即客户端现有代码。下发的代码在**桌面端 Rust QuickJS 沙箱**中执行（加载时完成验签 + 编译 + 硬校验，调用带超时与熔断）。服务端可通过后台「内容管理 → 兜底管理」下发新实现热修复线上问题，无需客户端发版。

## 整体链路

```text
后台兜底管理编辑代码 → 服务端保存并计算 sha256（代码变化时版本号 +1）
  → 客户端启动 / 每 30 分钟拉取 get_fallback_modules（带签名）
  → 校验通过写入本地缓存 → 预热进 Rust 宿主（下次调用零加载延迟）
  → 执行异常时该次调用立即回退内置实现
```

## 下发代码格式

下发的**不是完整函数，是一段「函数体」**。客户端 Rust 宿主以 QuickJS 执行这段代码，`ctx` 是唯一入参，代码最后必须 `return` 一个实现对象：

```js
// 注释、顶层 const/let 都可以写，只要最后 return

const API = 'https://example.com/api'

async function searchOne(page) {
  const res = await ctx.http.get(`${API}?page=${page}`)
  return JSON.parse(res.body)
}

return {
  version: 1,                    // 必须 ≥ 1 的数字
  search: async (args) => {      // 方法名必须与该模块约定完全一致
    // args = { source, keyword, page, limit }
    const data = await searchOne(args.page)
    return { list: [...], allPage, limit, total, source }
  },
}
```

### 四条硬校验

加载时在 Rust 宿主完成，不满足任意一条则整块代码被忽略、回退内置实现（客户端控制台有 `[FallbackModule]` 警告）：

1. 语法可执行，不抛异常
2. 返回值是对象
3. `version` 是 ≥ 1 的有限数字
4. 至少实现了该模块约定方法中的**一个函数**（缺失的方法自动走内置，无需全写）

## ctx 能力契约

下发代码不能 import 任何内部模块，只能通过 `ctx` 访问能力。该接口只增不改，保证旧客户端能跑新脚本。

```js
ctx.appVersion                              // 客户端版本号
ctx.http.get(url, opts)                     // 经 Rust 代理的 GET，绕过 CORS
ctx.http.post(url, body, opts)              // body 为对象时自动 JSON 序列化
//   opts: { headers: {...}, timeoutMs }
//   返回 { status, headers, body }，body 是字符串
ctx.cache.get(key)                          // 内存缓存读
ctx.cache.set(key, value, ttlSeconds?)      // 写（可带过期）
ctx.cache.del(key)
ctx.log.info / warn / error(msg, data?)     // 打到客户端控制台
ctx.config.get('dot.path')                  // 只读用户设置
ctx.utils.parseIntervalToSeconds('04:30')   // → 270
ctx.utils.normalizeQualityKey(raw)          // → QualityKey | null
ctx.utils.stripHtmlTags(html)
```

## 七个模块的方法签名

| 模块 | 方法 | args | 返回值 |
|---|---|---|---|
| `lx_search` | `search` | `{ source, keyword, page, limit }` | `{ list, allPage, limit, total, source }` |
| `lx_album` | `searchAlbums` | `{ keyword, page, limit }` | 原始专辑数组 |
| | `getAlbumSongs` | `{ source, albumRawData, page, limit }` | `LxSearchResultItem[]` |
| `lx_duration` | `batchTrackInterval` | `{ songIds }` | 普通对象 `{ songId: 秒 }`（客户端自动转 Map） |
| `lx_lyric` | `fetchLyric` | `{ source, songInfo }` | `{ lyric, tlyric, rlyric, lxlyric }` 或 `null` |
| `lx_cover` | `extractCoverUrl` | `{ item }` | 封面 URL 字符串 |
| `plugin_fallback` | `isQqMusicPluginSource` | `{ source, platform }` | boolean |
| | `hostSearchFallback` | `{ source, keyword, page, limit }` | 搜索结果 |
| | `hostAlbumSearchFallback` | `{ source, keyword, page, limit }` | 专辑数组 |
| | `hostAlbumSongsFallback` | `{ source, albumMid, page, limit }` | 曲目列表 |
| | `isQqTrialMediaUrl` | `{ url }` | boolean |
| | `fillSongDurations` | `{ source, platform, results }` | 补齐时长后的 results |
| `playlist_import` | `getListDetailKg` | `{ rawId }` | `PlaylistImportResult` |
| | `getListDetailWy` | `{ rawId }` | `PlaylistImportResult` |
| | `getListDetailTx` | `{ rawId }` | `PlaylistImportResult` |
| | `getListDetailKw` | `{ rawId }` | `PlaylistImportResult` |
| | `getListDetailQishui` | `{ rawId }` | `PlaylistImportResult` |

`playlist_import` 的入参 `rawId` 为原始歌单链接/ID/分享口令（与内置实现一致，方法内部自行提取 ID）。
`PlaylistImportResult` 结构：

```js
{
  source: string,                 // 'kg' | 'wy' | 'tx' | 'kw' | 'qishui'
  songs: [{                       // 单项与内置实现的 PluginSearchResult 一致
    id, title, artist, album, coverUrl, duration,
    platform, platformId, rawData,
  }],
  total: number,
  info: { name, img, desc, author, playCount },
}
```

注意：内置实现里酷狗签名/网易 weapi 加密由客户端 Rust `host_crypto` 完成，**下发模块拿不到这些 IPC**——若热修酷狗/网易逻辑，需在模块 JS 内自带对应签名实现（酷狗为 MD5 排序拼接，网易 weapi/linuxapi 均为固定密钥 AES，社区有现成 JS 实现）。

方法可以随意 async，也可写成同步函数（宿主统一按 Promise 处理）。但**返回值必须 JSON 可序列化**——结果要跨 Rust IPC 传回前端：

- 不能返回函数、Symbol、循环引用；`undefined` 序列化后会变成 `null`
- `Map`/`Set` 会被宿主自动归一化为普通对象/数组再返回（如 `Map<songId, 秒>` → `{ songId: 秒 }`）
- 大二进制/超长字符串会拖慢 IPC，`lx_duration` 这类批量结果建议控制在千项以内

`lx_search` 的 list 单项结构（`LxSearchResultItem`）：

```js
{
  name: string,        // 歌名
  singer: string,      // 歌手
  albumName: string,   // 专辑名
  albumId: string|number,
  songmid: string,
  source: 'kw'|'kg'|'tx'|'wy'|'mg',
  interval: string,    // "mm:ss"
  img: string|null,
  singerAvatars?: Record<string, string>,  // 可选
  singerIds?: Record<string, string>,      // 可选
}
```

## 实例一：封面提取（最简单的模块）

典型用途：某插件改版后封面字段换了名字，热修复无需发版。

```js
return {
  version: 1,
  extractCoverUrl: (args) => {
    const item = args.item
    if (!item || typeof item !== 'object') return ''
    // 新版接口把封面放在 cover.image.url
    const url = item?.cover?.image?.url || item.artwork || item.pic || ''
    return typeof url === 'string' ? url.replace('http://', 'https://') : ''
  },
}
```

## 实例二：搜索接口迁移（完整异步流程）

典型用途：落雪源失效，切换到自建镜像接口。

```js
const CACHE_TTL = 600

return {
  version: 1,
  search: async (args) => {
    const { source, keyword, page = 1, limit = 30 } = args
    const cacheKey = `lx_search:${source}:${keyword}:${page}`
    const hit = ctx.cache.get(cacheKey)
    if (hit) return hit

    const res = await ctx.http.post('https://mirror.example.com/lx/search', {
      source, keyword, page, limit,
    }, { timeoutMs: 12000 })
    if (res.status !== 200) throw new Error(`upstream ${res.status}`)

    const raw = JSON.parse(res.body)
    const list = (raw.songs || []).map(s => ({
      name: ctx.utils.stripHtmlTags(s.name),
      singer: s.singer,
      albumName: s.album || '',
      albumId: s.albumId ?? '',
      songmid: String(s.id),
      source,
      interval: s.interval || '00:00',
      img: s.pic || null,
    }))

    const result = { list, allPage: raw.allPage ?? 1, limit, total: raw.total ?? list.length, source }
    ctx.cache.set(cacheKey, result, CACHE_TTL)
    return result
  },
}
```

## 生效与回退语义

| 场景 | 行为 |
|---|---|
| 后台保存 | 服务端计算 sha256；代码变化版本号自动 +1，未变化则保持（客户端靠 version + digest 判变，不会重复刷新） |
| 客户端拉取 | 启动时 + 每 30 分钟轮询；服务端不可达时保留本地缓存继续生效 |
| 生效时机 | 校验通过写入缓存后，**下一次调用**用新实现；正在执行的调用持有旧引用不受影响 |
| 单次执行异常 | 该次调用立即回退内置实现 |
| 连续失败 3 次 | 本会话内禁用整个模块（重启客户端或服务端发新版本可恢复） |
| 后台禁用/删除 | 客户端下次拉取后该模块回退内置实现 |

## 后台操作

进入后台「内容管理 → 兜底管理」（移动端在「更多 → 兜底管理」）：

- 七个模块卡片常驻展示，未配置的显示「使用内置默认实现」
- 编辑弹窗有「填入模板」按钮，按模块生成骨架代码
- 弹窗底部「ctx 能力契约」可展开查看
- 方法 chip 绿色高亮表示当前代码已覆盖该方法（文本探测，仅供参考）

**调试建议**：先存「仅保存」状态验证代码能加载（客户端控制台搜 `[FallbackModule]`），确认无误再切「启用下发」。

## 相关接口

| 接口 | 说明 |
|---|---|
| `POST /api/?action=get_fallback_modules` | 客户端拉取（需签名），只返回已启用模块 |
| `list_fallback_modules` | 后台列表（含未配置占位） |
| `save_fallback_module` | 后台保存（upsert，按 moduleKey） |
| `delete_fallback_module` | 后台删除 |
| `toggle_fallback_module` | 后台启停 |

存储为服务端 `api/fallback_modules.json` 文件（与公告、版本配置同目录）。
