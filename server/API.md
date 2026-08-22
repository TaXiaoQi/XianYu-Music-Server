# 服务端 APP API 文档

本文档记录 Rust 服务端当前保留的 APP 侧接口。接口以 `/api?action=...` 作为统一入口，由 `server/src/handlers/mod.rs` 分发到各模块。

## 调用方式

- 入口地址：`/api?action={action}`
- 请求方法：`GET` 或 `POST` 均可进入统一处理器，带请求体的接口按 JSON 解析
- 请求体：未加密时为 JSON 字符串；加密时请求体为 AES 密文
- 响应格式：

```json
{
  "code": 200,
  "msg": "ok",
  "data": {}
}
```

当 `code` 为 `400`、`401`、`403`、`404`、`500` 等错误码时，HTTP 状态码会同步返回对应错误状态。

## 签名与加密

默认接口需要签名。服务端读取以下请求头：

| 请求头 | 说明 |
|--------|------|
| `x-timestamp` | Unix 秒级时间戳 |
| `x-nonce` | 客户端随机串 |
| `x-sign` | 请求签名 |
| `x-encrypted-iv` | 可选，存在时表示请求体为 AES-256-CBC 加密内容 |

签名优先使用 HMAC-SHA256：

```text
hex(hmac_sha256(x-timestamp + x-nonce + raw_body, api_secret))
```

服务端仍兼容旧 MD5 签名：

```text
md5(x-timestamp + x-nonce + raw_body + api_secret)
```

时间戳容忍范围由 `server/config.json` 的 `api_timestamp_tolerance` 控制。请求带 `x-encrypted-iv` 时，服务端会用 `api_secret` 派生 AES key 解密请求体，并对响应进行 AES 加密，响应头包含 `x-encrypted-response: 1` 和 `x-response-iv`。

## 免签 action

当前有效 action 中，下列接口免签：

| action | 说明 |
|--------|------|
| `install` | 安装初始化 |
| `check` | 连通性检查 |
| `get_source_status` | 获取音源状态 |
| `upload_avatar` | 上传头像 |
| `deduct_master_quota` | 扣减主站配额 |
| `get_master_quota_usage` | 查询主站配额用量 |
| `get_captcha` | 获取验证码 |
| `verify_captcha` | 校验验证码 |
| `email_send_code` | 发送邮箱验证码 |
| `email_register` | 邮箱注册 |
| `email_login` | 邮箱登录 |
| `email_reset_password` | 邮箱重置密码 |
| `email_get_profile` | 获取邮箱用户资料 |

## 当前有效 action

### 上报

| action | 处理函数 | 说明 |
|--------|----------|------|
| `error` | `reporting::error` | 错误上报 |
| `report_user_behavior` | `reporting::report_user_behavior` | 播放行为上报（写 `play_history`） |
| `open` | `reporting::app_open` | 客户端启动上报 |
| `check` | `reporting::check` | 服务连通性检查 |
| `install` | `reporting::install` | 安装初始化 |

### 推荐

| action | 处理函数 | 说明 |
|--------|----------|------|
| `get_daily_recommend` | `recommend::get_daily_recommend` | 每日推荐算法下发（需登录，token 属主校验） |

#### `get_daily_recommend` 说明

- 请求体：`{ "ciyuanxi_id": "..." }`（token 由客户端信封自动注入）。
- 服务端基于账号近 90 天播放历史（`play_history`）聚合画像（常听歌手/歌曲/收听规模），决策推荐策略权重并下发"算法 DSL"：策略列表（`artist_search` / `song_search` / `keyword_search` + 权重 + 查询词 + 推荐理由）、排除项（近 14 天听过的歌）、每日种子与目标数量。
- 客户端在本机调用已安装的音源插件按策略执行搜索，过滤排除项、按权重打分去重、按每日种子洗牌，整理出当日推荐歌曲板块。
- 算法缓存 10 分钟（同用户同日），同一天多次请求返回一致结果；策略与权重由服务端决策，可随时调整无需客户端发版。

### 系统

| action | 处理函数 | 说明 |
|--------|----------|------|
| `get_source_status` | `system::get_source_status` | 获取音源状态 |
| `get_version_status` | `system::get_version_status` | 获取版本状态 |
| `get_latest_version` | `system::get_latest_version` | 获取最新版本 |
| `get_announcement` | `system::get_announcement` | 获取公告 |
| `confirm_announcement` | `system::confirm_announcement` | 确认公告 |
| `get_about_config` | `system::get_about_config` | 获取关于页配置 |
| `get_user_agreement` | `system::get_user_agreement` | 获取用户协议 |
| `get_server_load` | `system::get_server_load` | 获取服务负载 |
| `get_leaderboard` | `system::get_leaderboard` | 获取排行榜 |

### 认证

| action | 处理函数 | 说明 |
|--------|----------|------|
| `register` | `auth::register` | 注册账号 |
| `user_login` | `auth::user_login` | 用户登录 |
| `get_captcha` | `auth::get_captcha` | 获取验证码 |
| `verify_captcha` | `auth::verify_captcha` | 校验验证码 |
| `login_by_code` | `auth::login_by_code` | 验证码登录 |
| `send_verify_code` | `auth::send_verify_code` | 发送验证码 |
| `reset_password` | `auth::reset_password` | 重置密码 |
| `delete_account` | `auth::delete_account` | 注销账号 |
| `preverify_delete_account` | `auth::preverify_delete_account` | 预验证注销凭据 |
| `generate_tv_login_code` | `auth::generate_tv_login_code` | 生成 TV 登录码 |
| `poll_tv_login_status` | `auth::poll_tv_login_status` | 轮询 TV 登录状态 |
| `scan_tv_login` | `auth::scan_tv_login` | 扫描 TV 登录码 |
| `confirm_tv_login` | `auth::confirm_tv_login` | 确认 TV 登录 |

### 设置

| action | 处理函数 | 说明 |
|--------|----------|------|
| `get_user_info` | `settings::get_user_info` | 获取用户信息 |
| `get_user_settings` | `settings::get_user_settings` | 获取用户设置 |
| `update_user_settings` | `settings::update_user_settings` | 更新用户设置 |
| `update_profile` | `settings::update_profile` | 更新资料 |
| `check_username` | `settings::check_username` | 检查用户名 |
| `change_password` | `settings::change_password` | 修改密码 |
| `get_avatar_status` | `settings::get_avatar_status` | 获取头像审核状态 |
| `get_nickname_status` | `settings::get_nickname_status` | 获取昵称审核状态 |
| `report_listen_stats` | `settings::report_listen_stats` | 上报听歌统计 |
| `deduct_master_quota` | `settings::deduct_master_quota` | 扣减主站配额 |
| `get_master_quota_usage` | `settings::get_master_quota_usage` | 查询主站配额用量 |

### 社交

| action | 处理函数 | 说明 |
|--------|----------|------|
| `submit_feedback` | `social::submit_feedback` | 提交反馈 |
| `check_ciyuanxi_id` | `social::check_ciyuanxi_id` | 检查溯源 ID |

### 壁纸

| action | 处理函数 | 说明 |
|--------|----------|------|
| `list_wallpapers` | `wallpaper::list_wallpapers` | 获取壁纸列表 |
| `my_wallpapers` | `wallpaper::my_wallpapers` | 获取我的壁纸 |
| `upload_wallpaper` | `wallpaper::upload_wallpaper` | 上传壁纸 |

### 歌单

| action | 处理函数 | 说明 |
|--------|----------|------|
| `delete_playlist` | `playlist::delete_playlist` | 删除歌单 |

### 同步

| action | 处理函数 | 说明 |
|--------|----------|------|
| `file_sync_upload_start` | `sync::file_sync_upload_start` | 开始歌单分块上传 |
| `file_sync_upload_chunk` | `sync::file_sync_upload_chunk` | 上传歌单分块 |
| `file_sync_upload_finish` | `sync::file_sync_upload_finish` | 完成歌单同步 |
| `file_sync_download` | `sync::file_sync_download` | 下载歌单同步数据 |
| `plugin_sync_upload_one` | `sync::plugin_sync_upload_one` | 上传单个插件快照 |
| `plugin_sync_download` | `sync::plugin_sync_download` | 下载插件快照 |
| `settings_sync_upload` | `sync::settings_sync_upload` | 上传设置快照 |
| `settings_sync_download` | `sync::settings_sync_download` | 下载设置快照 |

### 上传

| action | 处理函数 | 说明 |
|--------|----------|------|
| `upload_avatar` | `upload::upload_avatar` | 上传头像 |

### 邮箱认证

| action | 处理函数 | 说明 |
|--------|----------|------|
| `email_send_code` | `email_auth::send_code` | 发送邮箱验证码 |
| `email_get_captcha_config` | `email_auth::get_captcha_config` | 获取邮箱验证码配置 |
| `email_get_turnstile_config` | `email_auth::get_turnstile_config` | 获取 Turnstile 配置 |
| `email_register` | `email_auth::register` | 邮箱注册 |
| `email_login` | `email_auth::login` | 邮箱登录 |
| `email_reset_password` | `email_auth::reset_password` | 邮箱重置密码 |
| `email_get_profile` | `email_auth::get_profile` | 获取邮箱用户资料 |

## 排行榜接口

### `get_leaderboard`

获取听歌排行榜，支持按时间周期筛选。

**请求参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `type` | string | 否 | 排行类型：`listen`（听歌时长，默认）、`songs`（听歌数量） |
| `period` | string | 否 | 时间周期：`daily`（日榜）、`weekly`（周榜）、`total`（总榜，默认） |
| `limit` | int | 否 | 返回条目数，默认 50，范围 1~100 |
| `ciyuanxi_id` | string | 否 | 当前用户弦予号，用于标记本人排名 |

**period 说明：**

| 值 | 数据来源 | 说明 |
|------|----------|------|
| `daily` | `listen_daily_stats` 今天 | 当日听歌时长排行 |
| `weekly` | `listen_daily_stats` 本周一~今天 | 本周累计听歌时长排行 |
| `total` | `app_users` 累计值 | 历史总听歌时长排行 |

**响应示例：**

```json
{
  "code": 200,
  "msg": "ok",
  "data": {
    "leaderboard": [
      {
        "rank": 1,
        "username": "user123",
        "nickname": "user123",
        "avatar": "https://...",
        "duration": 3600,
        "is_me": false
      }
    ],
    "me": {
      "rank": 5,
      "username": "myuser",
      "nickname": "myuser",
      "avatar": "https://...",
      "duration": 1800,
      "is_me": true
    },
    "total_users": 50,
    "period": "daily"
  }
}
```

- `me` 为 `null` 时表示当前用户无排行数据。
- 日榜/周榜依赖 `listen_daily_stats` 表，该表由 `report_listen_stats` 和 `report_user_behavior` 在客户端上报播放数据时同步写入。

## 已清理的旧接口范围

本轮清理后，服务端不再暴露旧聊天接口、旧歌单 CRUD/收藏接口、分享动态接口、背景/封面上传接口，以及其他未在 `handlers::dispatch` 中登记的旧 action。客户端调用未登记 action 时会返回：

```json
{
  "code": 404,
  "msg": "未知操作: {action}",
  "data": null
}
```

接口清单以 `server/src/handlers/mod.rs` 为准。新增或删除 action 时，需要同步更新本文档和根目录 `README.md` 的 APP API 模块说明。
