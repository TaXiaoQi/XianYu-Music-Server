# 兜底行为配置下发说明

客户端把各元数据链路（歌词获取、榜单、推荐歌单、插件宿主兜底、歌单导入）的内置实现固定在客户端原生代码中（Rust builtin）。服务端只下发**签名配置 JSON**（链路开关 / 平台来源参数），**不含任何可执行代码**——客户端 ed25519 验签通过后才采纳，未配置 / 验签失败 / 已过期 / 拉取失败时一律按默认行为执行。上游接口变动仍需客户端发版修复，但服务端可随时关闭异常链路或调整平台来源，无需发版。

> 旧版「热修 JS 模块」（QuickJS 沙箱执行下发代码）已废弃：服务端不再下发 `code`，客户端已移除宿主执行链路。旧存储键 `fallback_modules`（含历史 JS 代码）不再读取，可自行清理。

## 整体链路

```text
后台「兜底配置」编辑模块开关/平台来源 → 服务端重建配置文档（canonical JSON，附 expiresAt）
  → 签发服务（--signer，active 槽位私钥）整包签名 ed25519（xianyu-fallback-config-v1\x00 + config 原始字符串）
  → 客户端启动 / 每 30 分钟拉取 get_fallback_modules（响应含 keyId）
  → 双公钥槽位验签（keyId 精确匹配优先）→ 解析（未知 key/方法忽略）
  → 应用进内存并缓存（缓存包裹含 keyId）→ 过期后等同验签失败
  → 下一次调用按配置执行：被关闭的链路直接返回空，其余走内置原生实现
```

## 签名与密钥体系（双公钥轮换）

### 公钥槽位（三端一致）

客户端内嵌**双公钥槽位** `FALLBACK_VERIFY_KEYS: [(keyId, 公钥hex)]`（三端文件：Mobile/Watch `rust/src/fallback_verify.rs`、Desktop `src-tauri/src/fallback_verify.rs`，逐字一致）：

| 槽位 | keyId | 公钥 hex | 状态 |
|---|---|---|---|
| 当前 | `2026a` | `fd2f887e74adb2009079bc822536d8f09d1404656f748289608592b6a4c974c5` | 在用 |
| 预发布 | `2026b` | （注释态，轮换 T0 时填入） | 待启用 |

### 验签语义

- 服务端签发响应携带 `keyId`（签发槽位标识，形如 `"2026a"`）。
- 客户端验签**先按 keyId 精确匹配槽位**（严格绑定该槽位公钥，失败不回退遍历，防密钥降级）。
- 响应**无 keyId / keyId 为空 / keyId 不在已知槽位**时遍历全部槽位；签名对某槽位有效即采纳。
- 签名有效但 keyId 不在已知槽位 → 返回 `key_unknown = true`：客户端**照常采纳**配置，并上报 `FallbackConfigKeyUnknown` 事件（疑似新公钥上线或密钥降级，需人工确认）。
- 内测资格响应（`xianyu-beta-access-v1`，无 keyId 字段）按遍历全部槽位验签。

### 配置有效期 expiresAt

- 配置 JSON 顶层可选 `expiresAt`（unix 秒）；服务端保存时写入「签发时刻 + 30 天」，每次保存重签即顺延。
- 客户端把过期配置**等同验签失败**处理（丢弃并清缓存回默认），且在写缓存前即拒绝——即使服务端长期停摆或密钥泄露后轮换，旧配置也不会被无限期沿用。
- `expiresAt` 缺失视为永不过期（兼容旧服务端）。`keyId` / `expiresAt` 均不纳入签名消息（消息格式 `xianyu-fallback-config-v1\x00{config}` 逐字节不变）。

### 客户端缓存

缓存包裹结构 `{ config, signature, keyId? }`（Mobile/Watch SharedPreferences 键 `xianyu_fallback_config_v2`，Desktop localStore 同键）。启动/同步前 sanitize：验签不过、解析失败或已过期的缓存直接清除，回退全部默认行为。

## 签发档位（服务端）

| 档位 | 私钥位置 | keyId | 适用 |
|---|---|---|---|
| B 档托管（推荐） | `--signer` 子进程内存（启动时从密封文件加载） | 有（active/previous 双槽位） | 生产 |
| A 档本机（退化） | 主服务进程内存（env `FALLBACK_SIGN_PRIVATE_KEY` 或 `api/fallback_sign_key.txt`） | 无（响应省略 keyId） | 自部署轻量 |
| 未配置 | — | — | 签发不可用：保存配置报错、客户端拉取空串（行为与旧版一致，不 crash） |

- 两者都未配置时所有路径行为与旧版完全一致；`expiresAt`/`keyId` 全可选，老服务端 + 新客户端、新服务端 + 老客户端均兼容。
- 内测资格签名（check_beta_access）复用同一签发通道：B 档经签发服务，A 档本机签发。

### B 档托管：签发服务（`server --signer`）

- 同一二进制子命令模式，不连数据库，仅监听回环地址（默认 `127.0.0.1:8791`，env `FALLBACK_SIGNER_ADDR`），共享令牌 env `FALLBACK_SIGNER_TOKEN`（缺失拒启）。
- 主服务开关：`config.json` → `fallback.signer.enabled`（默认 false）、`fallback.signer.url`、`fallback.signer.token`（env `FALLBACK_SIGNER_URL` / `FALLBACK_SIGNER_TOKEN` 优先）。
- 私钥仅存在于 signer 进程内存；主服务经回环 HTTP 调用，**绝不返回私钥**，签发失败 fail closed（保存配置报错）。
- 密封私钥目录 `api/fallback_sign_keys/`：`<keyId>.key`（JSON `{keyId, seed, createdAt}`，Unix 0600）+ `state.json`（`{"active","previous"}`，缺 state 且恰有一把私钥时自动初始化）。
- 主服务侧调用接口：`POST /internal/sign`（`{message_hex}` → `{signature, keyId}`，签发前做消息格式校验，只签两类已知格式、防止沦为任意内容签名机：① 兜底配置 `xianyu-fallback-config-v1\x00{config_json}`（canonical schema 校验：仅 `version/modules/expiresAt` 结构、模块/字段白名单、64KB 上限）；② 内测资格 `xianyu-beta-access-v1\x00{device_id}\x00{0|1}\x00{0|1}\x00{exp}`（载荷格式校验））、`GET /internal/keys`、`POST /internal/switch`（`{activate}`，旧 active 移入 previous）、`GET /health`。
  > 2026-10-28 修复：此前 `/internal/sign` 仅接受兜底配置前缀，导致 B 档开启后 `check_beta_access` 走签发服务被 `bad_prefix` 拒签、响应空签名，客户端 fail-closed 弹「无法验证内测资格」。现签名服务同时接受内测消息前缀（仍保留格式校验与全部防护）。
- 防护：令牌常量时间比较、每 IP 限流（120 次/分钟）、审计日志 `logs/fallback_signer_audit.log`（JSONL：sign/switch/rejected）。

### keygen CLI（离线生成）

```bash
# 在离线/可信机器上执行；私钥明文不落联网机器、不打印到 stdout
server --keygen 2026b [--out <目录>]
```

输出 `keyId` + 公钥 hex（填入三端 `FALLBACK_VERIFY_KEYS` 预发布槽位与服务端测试常量）+ 密封私钥文件（拷贝到 signer 的 `api/fallback_sign_keys/` 后重启生效）。

## 公钥轮换流程（三步）

1. **T0 预发布**：离线 `--keygen 2026b` → 公钥填入三端 `FALLBACK_VERIFY_KEYS` 预发布槽位（解开注释）→ 发版覆盖全量客户端。此阶段服务端仍用 `2026a` 签发。
2. **切换**：管理页「签名密钥」卡片一键切换到 `2026b`（旧 `2026a` 移入 previous 槽位继续可用）→ 新保存的配置即以新密钥签发。此前保存的配置（`2026a` 签名）客户端仍可验签通过。
3. **收尾**：确认全量客户端已发版且 `FallbackConfigKeyUnknown` 事件无激增后，从 `api/fallback_sign_keys/` 移除 `2026a.key` 并重启 signer；同时在三端移除旧公钥槽位（随下次发版）。

**应急矩阵**：

| 场景 | 处置 |
|---|---|
| 疑似私钥泄露 | 立即切换到预发布新钥（或 keygen 应急新钥 + 紧急发版），客户端清缓存回默认；previous 槽位旧钥不可再用于签发 |
| 新钥签发后客户端大面积验签失败 | 服务端切回旧槽位（`--signer` 重启恢复 state 或管理页切回）；客户端 30min 内自愈 |
| 签发服务宕机 | 主服务保存配置报错（fail closed），客户端拉取旧配置/缓存继续生效至 expiresAt |
| 客户端时钟大幅偏差 | 过期判定偏严/偏松仅影响缓存生命周期，不影响验签正确性 |

## 客户端契约

### 拉取响应

`POST /api/?action=get_fallback_modules` 返回：

```json
{
  "code": 200,
  "msg": "ok",
  "data": {
    "config": "<配置 JSON 原始字符串>",
    "signature": "<ed25519 hex>",
    "keyId": "2026a"
  }
}
```

- 未启用 / 未配置任何模块时 `config`、`signature` 返回**空字符串**，客户端按「无配置」处理
- `keyId` 可选：仅 B 档托管签发时下发（A 档省略，客户端遍历槽位）
- 签名消息：`xianyu-fallback-config-v1\x00` + `config` 原始字符串字节（**逐字一致**，服务端存什么签什么，客户端验什么，避免 JSON 规范化差异）

### 配置 JSON 结构

```json
{
  "version": 3,
  "expiresAt": 1767225600,
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
- `expiresAt`（顶层，可选）：unix 秒，过期等同验签失败（见上）
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

移动端对应实现：`lib/src/plugin/fallback_modules/`（registry/types/sync）+ `plugin_host_fallback.dart`、`lyrics_repository.dart`、`download_provider.dart`、`host_sheet_import.dart`；Rust 验签：`rust/src/fallback_verify.rs`（Watch 同构）。

## 生效与回退语义

| 场景 | 行为 |
|---|---|
| 后台保存 | 服务端重建配置文档（canonical，键序稳定）、版本 +1、附 expiresAt、整包重签名 |
| 客户端拉取 | 启动时 + 每 30 分钟轮询；服务端不可达时保留本地缓存继续生效（至过期） |
| 验签失败 | 丢弃本次配置并清除本地缓存，回退默认行为（服务端 error_log 收 `FallbackConfigVerifyFail`） |
| 配置过期 | 等同验签失败：丢弃并清缓存回默认（缓存写入前即拒绝） |
| keyId 不在已知槽位 | 签名有效则照常采纳，并上报 `FallbackConfigKeyUnknown`（新公钥上线/密钥降级信号） |
| 总开关关闭 | 客户端拉取到空配置，全部回退默认行为（配置本体保留，重新开启即恢复） |
| 链路被关闭 | 该方法直接返回空结果，不走内置实现 |
| sources 配置 | 榜单/推荐歌单仅按配置的平台来源拉取（顺序即配置顺序） |

## 后台操作

移动端后台「更多 → 兜底配置」：

- 五个模块卡片常驻展示，未配置的显示「默认行为（全部链路开启）」
- 编辑弹窗：勾选要关闭的链路方法；榜单/推荐歌单模块可点选平台来源（点选顺序即生效顺序）
- 顶部「下发总开关」：暂停/恢复整包下发（显示当前签名 keyId）
- 「恢复默认」= 删除该模块条目
- 「签名密钥」卡片：签发档位徽标（托管签发 / 本机密钥 / 未配置）、active/previous 槽位、active 公钥指纹、可用槽位列表与一键切换（需确认，落管理员审计）

## 相关接口

| 接口 | 说明 |
|---|---|
| `POST /api/?action=get_fallback_modules` | 客户端拉取（需签名），返回 `{config, signature, keyId?}` |
| `get_fallback_config` | 后台列表（含未配置占位、方法白名单、来源平台、当前配置 keyId） |
| `save_fallback_module` | 后台保存（upsert，入参 `module_key`/`disabled_methods`/`sources`/`remark`；全空即恢复默认） |
| `delete_fallback_module` | 后台恢复默认 |
| `toggle_fallback_config` | 下发总开关（`enabled`） |
| `get_fallback_signer_status` | 后台签名密钥状态（mode/active/previous/available/activePublicKey/signerError） |
| `switch_fallback_signer_key` | 后台一键切换签发槽位（入参 `key_id`，旧 active 移入 previous） |

存储为 site_config 键 `fallback_config`（含 `keyId` 字段 = 签署当前配置的槽位；文件镜像 `api/fallback_config.json`，仅管理端元数据；客户端下发内容以 action 接口为准）。

## 自部署兼容底线

- `fallback.signer.enabled` 默认 false；不开 signer、不配 keyId/expiresAt 时全链路与旧版行为一致。
- 老服务端（无 keyId/expiresAt）+ 新客户端：遍历槽位验签、永不过期——完全兼容。
- 新服务端 + 老客户端（仅内嵌 `2026a`）：A 档（无 keyId）完全兼容；B 档在未轮换前（active=2026a）兼容，**切换到新 keyId 后老客户端验签失败回默认行为**——务必先走 T0 预发布。
- env `FALLBACK_SIGN_PRIVATE_KEY` / `api/fallback_sign_key.txt`（A 档）永久保留为退化路径，不删除。
