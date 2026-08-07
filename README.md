# 弦予音乐服务器 (XianYu-Music-Server)

音乐服务平台后端 + 后台管理系统，已完成从 PHP 到 Rust + TypeScript 的全栈重构。

## 技术栈

| 层 | 技术 | 说明 |
|----|------|------|
| 后端 | Rust + Axum + SQLx | 异步 Web 框架，MySQL 连接池 |
| 前端 | Vue 3 + TypeScript + Vite | SPA 单页应用，Pinia 状态管理 |
| 数据库 | MySQL 8.0 | utf8mb4 编码 |
| 部署 | Nginx 反向代理 | 静态文件 + API 代理 |

## 项目结构

```
XianYu-Music-Server/
├── server/                 # Rust 后端
│   ├── src/
│   │   ├── admin/          # 后台管理模块 (19 个文件)
│   │   ├── handlers/       # APP API 处理器 (12 个文件)
│   │   ├── config.rs       # 配置加载
│   │   ├── db.rs           # 数据库连接
│   │   ├── main.rs         # 入口，路由注册
│   │   ├── schema.rs       # 建表语句 (33 张表)
│   │   ├── sign.rs         # 签名验证 + AES 加解密
│   │   └── response.rs     # 统一响应封装
│   ├── Cargo.toml
│   ├── config.json         # 运行配置 (含密钥，勿入库)
│   └── target/             # 编译产物 (勿入库)
├── admin-web/              # TypeScript 前端
│   ├── src/
│   │   ├── api/client.ts   # API 客户端 (JWT 鉴权)
│   │   ├── api/email.ts    # 邮箱模块 API 客户端
│   │   ├── layouts/        # 布局组件
│   │   ├── router/         # 路由配置 (15 后台页面 + 4 邮箱页面)
│   │   ├── stores/         # Pinia 状态管理
│   │   └── views/          # 页面组件 (20 个)
│   ├── vite.config.ts
│   └── dist/               # 构建产物 (勿入库)
├── nginx.conf              # Nginx 配置
└── .gitignore
```

## 后端模块

### APP API (`/api`)

面向移动端 APP 的接口，共 74 个路由，需要签名验证 (部分免签)：

| 模块 | 文件 | 功能 |
|------|------|------|
| 认证 | `handlers/auth.rs` | 注册、登录、验证码、密码重置 |
| 设置 | `handlers/settings.rs` | 用户设置、个人信息 |
| 歌单 | `handlers/playlist.rs` | 歌单 CRUD、收藏 |
| 上报 | `handlers/reporting.rs` | 音源调用、登录、错误上报 |
| 社交 | `handlers/social.rs` | 分享、动态 |
| 同步 | `handlers/sync.rs` | 数据同步 |
| 系统 | `handlers/system.rs` | Banner、版本、音源状态 |
| 上传 | `handlers/upload.rs` | 头像、背景、封面上传 |
| 聊天 | `handlers/chat.rs` | 聊天消息 |
| 邮箱认证 | `handlers/email_auth.rs` | 邮箱注册、登录、验证码、找回密码、用户信息 |

### 后台 Admin (`/admin/api`)

面向管理后台的接口，共 66 个路由，JWT 鉴权：

| 模块 | 文件 | 功能 |
|------|------|------|
| 仪表盘 | `admin/dashboard.rs` | 统计概览 |
| 用户管理 | `admin/users.rs` | 用户列表、状态切换、配额 |
| 管理员 | `admin/admins.rs` | 管理员 CRUD、状态切换 |
| 账户 | `admin/account.rs` | 个人信息、邮箱绑定 |
| 认证 | `admin/auth.rs` | 登录、登出、改密 |
| 版本 | `admin/version.rs` | APP 版本 + 桌面端更新 |
| 公告 | `admin/announcement.rs` | 公告 CRUD |
| 壁纸 | `admin/wallpaper.rs` | 壁纸 CRUD |
| 审核 | `admin/audit.rs` | 头像、昵称审核 |
| 日志 | `admin/logs.rs` | 报错日志、APP 登录日志、操作日志、后台登录日志 |
| 反馈 | `admin/feedback.rs` | 反馈列表、回复、状态 |
| 数据库 | `admin/db.rs` | 表状态、修复、备份、恢复 |
| 音源 | `admin/source.rs` | 音源配置、开关 |
| 邮件 | `admin/email.rs` | 通知邮箱、邮件用户管理 |
| 分享 | `admin/share.rs` | 分享详情、清理过期 |
| 歌单 | `admin/playlist.rs` | 用户歌单管理 |
| 溯源ID | `admin/prettyid.rs` | 溯源 ID 变更 |
| 接口测试 | `admin/proxy.rs` | API 代理测试 |

## 前端页面

| 页面 | 路由 | 文件 | 状态 |
|------|------|------|------|
| 登录 | `/login` | `Login.vue` | 已完成 |
| 仪表盘 | `/dashboard` | `Dashboard.vue` | 已完成 |
| 用户管理 | `/users` | `Users.vue` | 已完成 |
| 报错日志 | `/error-log` | `ErrorLog.vue` | 已完成 |
| APP登录日志 | `/app-login-log` | `AppLoginLog.vue` | 已完成 |
| 版本管理 | `/version` | `Version.vue` | 已完成 |
| 公告管理 | `/announcements` | `Announcements.vue` | 已完成 |
| 壁纸管理 | `/wallpapers` | `Wallpapers.vue` | 已完成 |
| 头像审核 | `/avatar-audit` | `AvatarAudit.vue` | 已完成 |
| 反馈与建议 | `/feedback` | `Feedback.vue` | 已完成 |
| 管理员管理 | `/admins` | `Admins.vue` | 已完成 |
| 账户管理 | `/account` | `Account.vue` | 已完成 |
| 修改密码 | `/password` | `Password.vue` | 已完成 |
| 后台日志 | `/logs` | `Logs.vue` | 已完成 |
| 数据库管理 | `/database` | `Database.vue` | 已完成 |
| 接口测试 | `/api-test` | `ApiTest.vue` | 已完成 |

### 邮箱注册登录测试模块

独立于后台管理系统的用户侧页面，通过惜梦邮箱 API 发送验证码，支持注册、登录、找回密码：

| 页面 | 路由 | 文件 | 功能 |
|------|------|------|------|
| 邮箱登录 | `/email/login` | `email/EmailLogin.vue` | 邮箱+密码登录，JWT 鉴权 |
| 邮箱注册 | `/email/register` | `email/EmailRegister.vue` | 邮箱+验证码+密码注册 |
| 找回密码 | `/email/forgot` | `email/EmailForgot.vue` | 验证码重置密码 |
| 用户主页 | `/email/home` | `email/EmailHome.vue` | 个人资料 + 活动日志 |

## 本地开发

### 前置条件

- Rust (stable)
- Node.js 18+
- MySQL 8.0+

### 启动后端

```bash
cd server
# 编辑 config.json 配置数据库连接
cargo run --release
# 服务监听 0.0.0.0:8081
```

### 启动前端

```bash
cd admin-web
npm install
npm run dev
# 开发服务器 http://localhost:3000
# API 自动代理到 127.0.0.1:8081
```

### 生产构建

```bash
# 前端
cd admin-web && npm run build
# 产物: admin-web/dist/

# 后端
cd server && cargo build --release
# 产物: server/target/release/server (或 server.exe)
```

## 生产部署

### 目录结构

```
/www/wwwroot/xymusic.zh2026.cn/
├── admin-web/dist/        # 前端静态文件
├── server/
│   ├── server             # Rust 二进制
│   └── config.json        # 配置文件
├── beifen/                # 数据库备份目录
└── nginx.conf             # Nginx 配置
```

### Nginx 配置要点

- `root` 指向 `admin-web/dist`
- `/api`、`/admin/api`、`/uploads` 反向代理到 `127.0.0.1:8081`
- `/ws` 代理到 WebSocket 服务 `127.0.0.1:9501`
- SPA 路由回退: `try_files $uri $uri/ /index.html`

### 启动服务

```bash
# 使用 systemd 或 supervisor 守护进程
./server  # 读取同目录 config.json
```

## 配置说明

`server/config.json`:

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
  "email_api_primary": "http://a.bzxhkj.com/a",
  "email_api_backup": "http://a.bzxhkj.com/b",
  "email_sender": "admin@bzxhkj.com",
  "email_password": "your_email_password",
  "static_dir": "../admin-web/dist"
}
```

| 字段 | 说明 |
|------|------|
| `email_api_primary` | 惜梦邮箱 API 主调用地址 |
| `email_api_backup` | 惜梦邮箱 API 备用地址 |
| `email_sender` | 发件邮箱地址 |
| `email_password` | 发件邮箱调用密码 |
| `static_dir` | 前端静态文件目录，留空则默认 `../admin-web/dist`。Rust 服务端直接托管前端，无需 Nginx |

首次启动时，服务会自动创建全部 33 张数据库表 (含 `CREATE TABLE IF NOT EXISTS`)。

## 安全特性

- APP API 签名验证 (HMAC-SHA256 + 时间戳 + Nonce)
- 请求体 AES-256-CBC 加解密支持
- 后台 JWT 鉴权 (24 小时有效期)
- 邮箱模块独立 JWT 鉴权 (7 天有效期，与后台 token 隔离)
- 密码 bcrypt 哈希存储
- 邮箱验证码 60 秒频率限制 + 5 分钟有效期
- Nginx 敏感文件/目录访问控制
- 数据库备份文件名注入防护
