# 弦予音乐服务端

弦予音乐服务端包含 Rust API 服务、Vue 管理后台和配套部署配置，负责客户端账号体系、邮箱验证码、用户资料、同步、壁纸、反馈、公告、版本管理、后台管理等功能。项目已从旧版 PHP 后端迁移为 Rust + TypeScript 架构。

## 项目组成

| 目录 | 说明 |
|---|---|
| `server/` | Rust 后端服务，基于 Axum、SQLx、MySQL |
| `admin-web/` | Vue 3 + TypeScript 管理后台 |
| `docs/` | 独立功能配置文档 |
| `nginx.conf` | 生产环境 Nginx 示例配置 |

## 技术栈

| 层级 | 技术 |
|---|---|
| 后端 | Rust、Axum、SQLx |
| 管理后台 | Vue 3、TypeScript、Vite、Pinia |
| 数据库 | MySQL 8.0+，`utf8mb4` 编码 |
| 部署 | Nginx 反向代理 + systemd 或 supervisor 守护进程 |

## 运行要求

- Rust stable
- Node.js 18+
- MySQL 8.0+
- Nginx
- 可访问的 SMTP 服务或邮件 API，用于注册、找回密码等验证码邮件
- 如启用第三方人机验证，需要提前准备 Turnstile 或 hCaptcha 的 `Site Key` 和 `Secret Key`

## 配置需求

服务端启动时读取 `server/config.json`，也支持部分环境变量覆盖。生产环境至少需要配置数据库、接口密钥、后台账号、JWT 密钥和监听地址。

```json
{
  "db_host": "127.0.0.1",
  "db_port": 3306,
  "db_name": "chexian",
  "db_user": "chexian",
  "db_pass": "your_password",
  "db_charset": "utf8mb4",
  "api_secret": "your_api_secret",
  "api_timestamp_tolerance": 300,
  "admin_username": "admin",
  "admin_password": "adminadmin",
  "listen_addr": "0.0.0.0:8081",
  "jwt_secret": "your_jwt_secret",
  "email_api_primary": "",
  "email_api_backup": "",
  "email_sender": "no-reply@example.com",
  "email_password": "",
  "captcha_secret": "",
  "turnstile_secret": "",
  "hcaptcha_secret": "",
  "local_debug_no_db": false
}
```

| 字段 | 说明 |
|---|---|
| `db_*` | MySQL 连接信息 |
| `api_secret` | 客户端 API 签名密钥，客户端与服务端必须一致 |
| `api_timestamp_tolerance` | 签名时间戳容忍秒数 |
| `admin_username` / `admin_password` | 初始后台管理员账号 |
| `listen_addr` | 服务端监听地址 |
| `jwt_secret` | 用户与邮箱模块 JWT 签名密钥 |
| `email_*` | 邮件发送的环境兜底配置，推荐在后台页面配置 |
| `captcha_secret` | 通用人机验证 Secret 环境兜底 |
| `turnstile_secret` | Turnstile 专用 Secret 环境兜底 |
| `hcaptcha_secret` | hCaptcha 专用 Secret 环境兜底 |
| `local_debug_no_db` | 无数据库本地调试模式，生产环境保持 `false` |

首次启动时，服务端会自动创建所需数据库表，并插入部分默认配置项。

## 功能配置文档

详细功能配置都放在 `docs/` 下，主文档只保留入口链接。

| 配置项 | 文档 |
|---|---|
| 邮箱机、SMTP、外部邮件 API | [内置邮箱机使用说明](docs/BUILTIN_MAILER.md) |
| Turnstile / hCaptcha 人机验证 | [人机验证配置说明](docs/HUMAN_CAPTCHA.md) |
| APP API action 清单 | [服务端 API 文档](server/API.md) |

## 生产构建

构建后台静态文件：

```bash
cd admin-web
npm install
npm run build
```

产物目录：

```text
admin-web/dist/
```

构建服务端：

```bash
cd server
cargo build --release
```

产物位置：

```text
server/target/release/server
```

Windows 环境为：

```text
server/target/release/server.exe
```

## 生产部署

推荐目录结构：

```text
/www/wwwroot/xymusic.example.com/
├── admin-web/dist/
├── server/
│   ├── server
│   └── config.json
├── beifen/
└── nginx.conf
```

Nginx 配置要点：

- `root` 指向 `admin-web/dist`
- `/api` 反向代理到 `127.0.0.1:8081`
- `/admin/api` 反向代理到 `127.0.0.1:8081`
- `/uploads` 反向代理或映射到服务端上传目录
- SPA 路由使用 `try_files $uri $uri/ /index.html`

服务端建议用 systemd 或 supervisor 守护。启动进程时，工作目录保持在 `server/`，让程序能读取同目录下的 `config.json`。

systemd 示例：

```ini
[Unit]
Description=XianYu Music Server
After=network.target

[Service]
WorkingDirectory=/www/wwwroot/xymusic.example.com/server
ExecStart=/www/wwwroot/xymusic.example.com/server/server
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

## 部署后检查

部署完成后依次检查：

1. 访问后台登录页，确认静态资源正常加载。
2. 登录后台，检查仪表盘是否能加载数据。
3. 在后台配置邮箱机并发送测试邮件。
4. 如启用人机验证，在后台保存 Turnstile 或 hCaptcha 配置后，测试客户端登录、注册、找回密码弹窗。
5. 客户端服务器 API 地址填写到 `/api`，不要填写 `/admin/api`。

## 安全说明

- `config.json` 包含数据库密码、签名密钥和 JWT 密钥，不要提交到公开仓库。
- `api_secret` 和 `jwt_secret` 生产环境必须改为随机长字符串。
- 后台初始密码部署后应立即修改。
- 邮箱授权码、人机验证 Secret 只保存在服务端或后台配置中，不要写入客户端。
