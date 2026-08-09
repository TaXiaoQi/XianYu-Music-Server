# 内置邮箱机使用说明

内置邮箱机是服务端统一邮件发送模块，用于注册验证码、找回密码验证码和后台测试邮件。它保留外部 HTTP API 和标准 SMTP 两种通道作为备选，但推荐默认使用 `builtin` 模式。

## 发送模式

| 模式 | 配置值 | 说明 |
|------|--------|------|
| 内置邮箱机 | `builtin` | 服务端接收邮件请求、记录发送日志，并按配置选择投递通道 |
| 外部 HTTP API | `http_api` | 直接调用外部邮箱机 API |
| 标准 SMTP | `smtp` | 直接通过 SMTP 协议发送 |

## 内置模式流程

```text
注册/找回密码/测试邮件
↓
服务端内置邮箱机
↓
写入 email_send_log
↓
优先使用 SMTP 出口发送
↓
SMTP 未配置或失败时，回退外部 HTTP API
↓
更新发送结果和失败原因
```

## 后台配置

进入后台：

```text
系统管理 -> 邮箱机设置
```

推荐配置方式：

1. 发送方式选择「内置邮箱机」。
2. 填写「通用配置」里的发件邮箱和授权码。
3. 填写「内置邮箱机 SMTP 出口」。
4. 如需备用通道，再填写「外部 API 备选」。
5. 保存后在底部发送测试邮件。

## SMTP 出口示例

```text
SMTP 服务器地址：smtp.example.com
SMTP 端口：465
SMTP 用户名：no-reply@example.com
SMTP 密码 / 授权码：邮箱授权码
```

端口 `465` 使用隐式 SSL/TLS；端口 `587` 或 `25` 使用 STARTTLS。多数邮箱服务商要求使用授权码，不建议直接填写登录密码。

## 外部 API 备选

外部 API 备选只在以下情况使用：

- 发送方式选择「外部 HTTP API」
- 发送方式选择「内置邮箱机」，但 SMTP 出口未配置或投递失败

通用参数格式：

```text
email=发件邮箱
password=调用密码或授权码
title=邮件标题
context=邮件正文
recipient=收件邮箱
```

外部 API 地址不要写死到前端或公开文档中，生产环境建议在后台或服务端配置中维护。

## 配置项

| 配置项 | 说明 |
|--------|------|
| `email_provider` | 邮件发送方式，推荐 `builtin` |
| `email_sender` | 发件邮箱地址 |
| `email_password` | 通用密码或授权码 |
| `smtp_host` | SMTP 服务器地址 |
| `smtp_port` | SMTP 端口 |
| `smtp_username` | SMTP 用户名 |
| `smtp_password` | SMTP 密码或授权码，留空时回退 `email_password` |
| `email_api_primary` | 外部 API 主地址 |
| `email_api_backup` | 外部 API 备用地址 |

## 注意事项

- 内置邮箱机不是完整互联网邮件服务器，仍需要 SMTP 出口或外部 API 作为实际投递通道。
- 测试邮件发送成功后，注册验证码和找回密码验证码会使用同一套发送入口。
- 发送失败时可查看 `email_send_log` 中的状态和错误原因。
