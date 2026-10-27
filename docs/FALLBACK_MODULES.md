# 兜底行为配置下发说明

客户端把各元数据链路（歌词获取、榜单、推荐歌单、插件宿主兜底、歌单导入）的内置实现固定在客户端原生代码中（Rust builtin）。服务端只下发**签名配置 JSON**（链路开关 / 平台来源参数），**不含任何可执行代码**——客户端 ed25519 验签通过后才采纳，未配置 / 验签失败 / 拉取失败时一律按默认行为执行。上游接口变动仍需客户端发版修复，但服务端可随时关闭异常链路或调整平台来源，无需发版。

> 旧版「热修 JS 模块」（QuickJS 沙箱执行下发代码）已废弃：服务端不再下发 `code`，客户端已移除宿主执行链路。旧存储键 `fallback_modules`（含历史 JS 代码）不再读取，可自行清理。

## 整体链路

```text
后台「兜底配置」编辑模块开关/平台来源 → 服务端重建配置文档（canonical JSON）
  → 整包签名 ed25519（xianyu-fallback-config-v1\x00 + config 原始字符串）
  → 客户端启动 / 每 30 分钟拉取 get_fallback_modules
  → 验签通过 → 解析（未知 key/方法忽略）→ 应用进内存并缓存
  → 下一次调用按配置执行：被关闭的链路直接返回空，其余走内置原生实现
```

## 客户端契约

### 拉取响应

`POST /api/?action=get_fallback_modules` 返回：

```json
{
  "code": 200,
  "msg": "ok",
  "data": {
    "config": "<配置 JSON 原始字符串>",
    "signature": "<ed25519 hex>"
  }
}
```

- 未启用 / 未配置任何模块时 `config`、`signature` 返回**空字符串**，客户端按「无配置」处理
- 签名消息：`xianyu-fallback-config-v1\x00` + `config` 原始字符串字节（**逐字一致**，服务端存什么签什么，客户端验什么，避免 JSON 规范化差异）
- 公钥：客户端内置 `fd2f887e74adb2009079bc822536d8f09d1404656f748289608592b6a4c974c5`（私钥 `FALLBACK_SIGN_PRIVATE_KEY` 环境变量或 `api/fallback_sign_key.txt`，两仓同钥）

### 配置 JSON 结构

```json
{
  "version": 3,
  "modules": {
    "lx_lyric":          { "version": 1, "disabledMethods": ["fetchLyric"] },
    "lx_toplist":        { "version": 1, "disabledMethods": [], "params": { "sources": ["wy", "kg"] } },
    "lx_recommend_sheets": { "version": 1, "disabledMethods": [] },
    "plugin_fallback":   { "version": 1, "disabledMethods": ["fillSongDurations"] },
    "playlist_import":   { "version": 1, "disabledMethods": ["getListDetailMg"] }
  }
}
```

- `version`（顶层）：整包配置版本，服务端每次变更 +1；客户端仅透传记录
- 模块条目 `version`：该模块配置变更次数；客户端仅透传记录
- `disabledMethods`：被关闭的链路方法，客户端该行为**直接返回空**（内置与原热修模块打同一批上游接口，关闭即整体停用）
- `params.sources`：平台来源白名单与顺序，仅 `lx_toplist` / `lx_recommend_sheets` 生效，取值 `wy/kg/kw/tx` 子集；缺失 = 默认全部
- 未知模块 key、未知方法名、非法来源在客户端解析时**忽略**；`disabledMethods` 与 `sources` 全空的模块不进文档（等同默认行为）

### 模块与方法白名单（两端同步维护）

| 模块 | 方法 | 支持 sources |
|---|---|---|
| `lx_lyric` | `fetchLyric` | 否 |
| `lx_toplist` | `getTopLists` / `getTopListDetail` | 是 |
| `lx_recommend_sheets` | `getRecommendSheetTags` / `getRecommendSheetsByTag` | 是 |
| `plugin_fallback` | `hostSearchFallback` / `hostAlbumSearchFallback` / `hostAlbumSongsFallback` / `fillSongDurations` | 否 |
| `playlist_import` | `getListDetailKg` / `getListDetailWy` / `getListDetailTx` / `getListDetailKw` / `getListDetailMg` / `getListDetailQishui` | 否 |

移动端对应实现：`lib/src/plugin/fallback_modules/`（registry/types/sync）+ `plugin_host_fallback.dart`、`lyrics_repository.dart`、`download_provider.dart`、`host_sheet_import.dart`；Rust 验签：`rust/src/fallback_verify.rs`。

## 生效与回退语义

| 场景 | 行为 |
|---|---|
| 后台保存 | 服务端重建配置文档（canonical，键序稳定）、版本 +1、整包重签名 |
| 客户端拉取 | 启动时 + 每 30 分钟轮询；服务端不可达时保留本地缓存继续生效 |
| 验签失败 | 丢弃本次配置并清除本地缓存，回退默认行为（服务端 error_log 收 `FallbackConfigVerifyFail`） |
| 总开关关闭 | 客户端拉取到空配置，全部回退默认行为（配置本体保留，重新开启即恢复） |
| 链路被关闭 | 该方法直接返回空结果，不走内置实现 |
| sources 配置 | 榜单/推荐歌单仅按配置的平台来源拉取（顺序即配置顺序） |

## 后台操作

移动端后台「更多 → 兜底配置」：

- 五个模块卡片常驻展示，未配置的显示「默认行为（全部链路开启）」
- 编辑弹窗：勾选要关闭的链路方法；榜单/推荐歌单模块可点选平台来源（点选顺序即生效顺序）
- 顶部「下发总开关」：暂停/恢复整包下发
- 「恢复默认」= 删除该模块条目

## 相关接口

| 接口 | 说明 |
|---|---|
| `POST /api/?action=get_fallback_modules` | 客户端拉取（需签名），返回 `{config, signature}` |
| `get_fallback_config` | 后台列表（含未配置占位、方法白名单、来源平台） |
| `save_fallback_module` | 后台保存（upsert，入参 `module_key`/`disabled_methods`/`sources`/`remark`；全空即恢复默认） |
| `delete_fallback_module` | 后台恢复默认 |
| `toggle_fallback_config` | 下发总开关（`enabled`） |

存储为 site_config 键 `fallback_config`（文件镜像 `api/fallback_config.json`，仅管理端元数据；客户端下发内容以 action 接口为准）。
