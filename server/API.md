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
| `check` | `reporting::check` | 服务连通性检查 |
| `install` | `reporting::install` | 安装初始化 |

### 系统

| action | 处理函数 | 说明 |
|--------|----------|------|
| `get_source_status` | `system::get_source_status` | 获取音源状态 |
| `get_version_status` | `system::get_version_status` | 获取版本状态 |
| `get_latest_version` | `system::get_latest_version` | 获取最新版本 |
| `get_announcement` | `system::get_announcement` | 获取公告 |
| `get_about_config` | `system::get_about_config` | 获取关于页配置 |
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
| `email_register` | `email_auth::register` | 邮箱注册 |
| `email_login` | `email_auth::login` | 邮箱登录 |
| `email_reset_password` | `email_auth::reset_password` | 邮箱重置密码 |
| `email_get_profile` | `email_auth::get_profile` | 获取邮箱用户资料 |

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
