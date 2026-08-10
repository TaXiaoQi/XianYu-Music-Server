# 人机验证配置说明

人机验证用于保护注册、登录、找回密码、发送邮箱验证码等账号相关接口，减少脚本刷验证码、撞库和恶意注册。服务端当前支持 Cloudflare Turnstile 和 hCaptcha，并保留旧版算术题验证码作为兜底。

## 支持的验证服务

| 服务商 | 配置值 | 说明 |
|---|---|---|
| Cloudflare Turnstile | `turnstile` | 推荐优先使用，无需用户识别图片，体验较轻 |
| hCaptcha | `hcaptcha` | 可作为 Turnstile 的替代方案 |
| 关闭 | `off` | 不启用第三方人机验证 |

## 工作方式

客户端或邮箱注册页面会先请求服务端公开配置接口：

```text
email_get_captcha_config
```

服务端只返回前端需要的信息：

```json
{
  "enabled": true,
  "provider": "turnstile",
  "site_key": "your_site_key"
}
```

前端根据 `provider` 动态加载对应组件：

| provider | 前端脚本 |
|---|---|
| `turnstile` | `https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit` |
| `hcaptcha` | `https://js.hcaptcha.com/1/api.js?render=explicit` |

验证通过后，前端提交：

```text
captcha_token
```

服务端再使用 `captcha_secret` 或环境变量中的 Secret 调用对应服务商的 `siteverify` 接口。Secret 不会下发到前端。

## 后台配置

进入后台：

```text
系统管理 -> 人机验证设置
```

可配置内容：

| 字段 | 说明 |
|---|---|
| 启用人机验证 | 开启后，账号相关操作会要求完成人机验证 |
| 验证服务商 | 选择 `Cloudflare Turnstile` 或 `hCaptcha` |
| Site Key | 前端展示组件使用，在对应服务商控制台获取 |
| Secret Key | 服务端校验 token 使用，在对应服务商控制台获取 |

保存后立即生效，不需要重启服务端。

## 配置项

新版本使用通用 `captcha_*` 配置项，存储在 `server_settings` 表中：

| 配置项 | 说明 |
|---|---|
| `captcha_enabled` | 是否启用人机验证，`1` 为启用，`0` 为关闭 |
| `captcha_provider` | 验证服务商：`turnstile`、`hcaptcha`、`off` |
| `captcha_site_key` | 前端展示组件使用 |
| `captcha_secret` | 服务端校验 token 使用 |

服务端仍兼容旧版 Turnstile 配置：

| 旧配置项 | 兼容说明 |
|---|---|
| `turnstile_enabled` | 新配置为空时可作为 Turnstile 启用状态回退 |
| `turnstile_site_key` | 新配置为空时可作为 Turnstile Site Key 回退 |
| `turnstile_secret` | 新配置为空时可作为 Turnstile Secret 回退 |

## 环境变量回退

Secret 的读取优先级：

1. 数据库 `captcha_secret`
2. 环境变量 `CAPTCHA_SECRET`
3. provider 为 `turnstile` 时，回退 `TURNSTILE_SECRET`
4. provider 为 `hcaptcha` 时，回退 `HCAPTCHA_SECRET`

如果后台已经填写 Secret，优先使用后台配置。环境变量适合在容器、服务器环境或不希望把 Secret 写入数据库时使用。

## 前端覆盖范围

当前接入新版人机验证的页面和组件：

| 位置 | 说明 |
|---|---|
| 管理后台邮箱注册页 | `/email/register` |
| 管理后台找回密码页 | `/email/forgot` |
| 桌面客户端 `HumanCaptchaModal` | 登录、注册、找回密码、发送验证码等账号流程 |
| 新手引导弹窗 | 复用桌面客户端人机验证弹窗 |

如果第三方人机验证未启用、配置不完整或配置接口读取失败，桌面客户端会回退旧的算术题验证码。

## 服务端接口覆盖范围

以下接口会通过统一逻辑校验人机验证：

| 接口 action | 说明 |
|---|---|
| `email_send_code` | 邮箱测试模块发送验证码 |
| `user_login` | 客户端登录 |
| `register` | 客户端注册 |
| `send_verify_code` | 客户端发送邮箱验证码 |
| `reset_password` | 客户端找回密码 |

这些接口优先识别：

```text
captcha_token
```

同时兼容旧字段：

```text
turnstile_token
```

未启用第三方人机验证时，客户端账号接口继续使用旧字段：

```text
captcha_id
captcha_answer
```

## Turnstile 配置

在 Cloudflare 控制台创建 Turnstile widget：

1. 打开 Cloudflare 控制台。
2. 进入 Turnstile。
3. 创建站点。
4. Widget 模式建议选择 `Managed`。
5. 域名填写生产域名；本地测试可加入 `localhost` 或测试域名。
6. 获取 `Site Key` 和 `Secret Key`。
7. 回到后台「人机验证设置」保存。

后台配置：

```text
启用人机验证：开启
验证服务商：Cloudflare Turnstile
Site Key：Cloudflare 提供的 Site Key
Secret Key：Cloudflare 提供的 Secret Key
```

## hCaptcha 配置

在 hCaptcha 控制台创建 site：

1. 打开 hCaptcha 控制台。
2. 创建一个 site。
3. 填写生产域名；本地测试可加入 `localhost` 或测试域名。
4. 获取 `Site Key` 和 `Secret Key`。
5. 回到后台「人机验证设置」保存。

后台配置：

```text
启用人机验证：开启
验证服务商：hCaptcha
Site Key：hCaptcha 提供的 Site Key
Secret Key：hCaptcha 提供的 Secret Key
```

## 本地调试

开启无数据库调试模式时：

```text
LOCAL_DEBUG_NO_DB=1
```

服务端会返回：

```json
{
  "enabled": false,
  "provider": "off",
  "site_key": "",
  "debug": true
}
```

此时不会加载 Turnstile 或 hCaptcha，客户端继续使用旧的本地 mock 流程。

## 常见问题

### 前端没有显示验证组件

检查后台是否同时满足：

- 已启用人机验证
- 已选择 provider
- 已填写 Site Key
- 已填写 Secret Key，或设置了对应环境变量

公开配置接口只有在 Secret 可用时才会返回 `enabled: true`。

### 服务端提示请先完成人机验证

常见原因：

- 前端没有拿到 token
- Site Key 和 Secret Key 不属于同一个站点
- provider 选择错误，例如 hCaptcha 的 key 配到了 Turnstile
- 当前域名没有加入服务商控制台允许列表

### 切换服务商后验证失败

从 Turnstile 切换到 hCaptcha，或从 hCaptcha 切回 Turnstile 时，需要填写新服务商对应的 Secret Key。服务端不会把旧服务商的 Secret 当作新服务商的 Secret 使用。

### 本地可以用，生产失败

检查生产域名是否加入 Turnstile 或 hCaptcha 控制台；同时确认服务器能访问对应服务商的 `siteverify` 接口。

### 客户端仍显示算术题

客户端会在以下情况回退算术题：

- 后台未启用第三方人机验证
- `email_get_captcha_config` 请求失败
- Site Key 或 Secret Key 未配置完整
- 服务端处于 `LOCAL_DEBUG_NO_DB=1` 模式

## 安全建议

- Secret Key 不要写入前端代码。
- 生产环境使用后台配置或环境变量维护 Secret。
- Site Key 可以公开，但应只绑定可信域名。
- 更换服务商或重置 Secret 后，需要同步更新后台配置。
