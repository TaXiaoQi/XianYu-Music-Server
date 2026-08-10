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

在本地或构建机完成前端和后端打包，再把产物上传到生产服务器。

### 构建后台静态文件

```bash
cd admin-web
npm install
npm run build
```

构建产物在 `admin-web/dist/`，上传时需要完整保留该目录内的文件。

### 构建服务端

```bash
cd server
cargo build --release
```

Linux 生产环境使用：

```text
server/target/release/server
```

Windows 服务器使用：

```text
server/target/release/server.exe
```

## 生产部署

以下步骤以 Linux + Nginx + systemd 为例，域名示例为 `xymusic.example.com`，服务端监听 `127.0.0.1:8081` 或 `0.0.0.0:8081`。

### 准备目录

在服务器创建站点目录：

```bash
mkdir -p /www/wwwroot/xymusic.example.com/admin-web/dist
mkdir -p /www/wwwroot/xymusic.example.com/server
mkdir -p /www/wwwroot/xymusic.example.com/beifen
```

最终目录建议如下：

```text
/www/wwwroot/xymusic.example.com/
├── admin-web/dist/
├── server/
│   ├── server
│   └── config.json
├── beifen/
└── nginx.conf
```

### 上传文件

上传后台构建产物：

```text
本地 admin-web/dist/* -> 服务器 /www/wwwroot/xymusic.example.com/admin-web/dist/
```

上传服务端二进制：

```text
本地 server/target/release/server -> 服务器 /www/wwwroot/xymusic.example.com/server/server
```

给服务端二进制添加执行权限：

```bash
chmod +x /www/wwwroot/xymusic.example.com/server/server
```

### 写入服务端配置

在服务器创建：

```text
/www/wwwroot/xymusic.example.com/server/config.json
```

示例：

```json
{
  "db_host": "127.0.0.1",
  "db_port": 3306,
  "db_name": "chexian",
  "db_user": "chexian",
  "db_pass": "your_password",
  "db_charset": "utf8mb4",
  "api_secret": "replace_with_random_api_secret",
  "api_timestamp_tolerance": 300,
  "admin_username": "admin",
  "admin_password": "replace_with_admin_password",
  "listen_addr": "127.0.0.1:8081",
  "jwt_secret": "replace_with_random_jwt_secret",
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

生产环境必须修改：

- `db_pass`
- `api_secret`
- `admin_password`
- `jwt_secret`

`listen_addr` 推荐使用 `127.0.0.1:8081`，由 Nginx 代理到公网。首次启动时服务端会自动建表并写入默认配置。

### 配置 Nginx

Nginx 站点配置示例：

```nginx
server {
    listen 80;
    server_name xymusic.example.com;

    root /www/wwwroot/xymusic.example.com/admin-web/dist;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://127.0.0.1:8081;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /admin/api {
        proxy_pass http://127.0.0.1:8081;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /uploads {
        proxy_pass http://127.0.0.1:8081;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

检查并重载 Nginx：

```bash
nginx -t
systemctl reload nginx
```

如果使用 HTTPS，先完成证书配置，再将 `listen 80` 调整为对应的 HTTPS 配置。

### 配置 systemd

创建服务文件：

```bash
vim /etc/systemd/system/xianyu-music-server.service
```

写入：

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

启动并设置开机自启：

```bash
systemctl daemon-reload
systemctl enable xianyu-music-server
systemctl start xianyu-music-server
```

查看运行状态和日志：

```bash
systemctl status xianyu-music-server
journalctl -u xianyu-music-server -f
```

### 首次访问

打开：

```text
https://xymusic.example.com/login
```

使用 `config.json` 中的 `admin_username` 和 `admin_password` 登录后台。首次登录后建议立即修改后台密码。

## 宝塔面板部署

适用于使用宝塔面板（BT Panel）管理 Linux 服务器的场景。整体流程与通用部署一致，区别在于用宝塔界面操作数据库、网站和守护进程。

### 1. 创建数据库

宝塔「数据库」→「添加数据库」：

- 数据库名：`chexian`
- 用户名：`chexian`
- 密码：设一个强密码
- 字符集：`utf8mb4`

### 2. 编译并上传二进制

#### 服务器上直接编译

在宝塔 SSH 终端中安装 Rust：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

上传 `server/` 源码后编译：

```bash
cd /www/wwwroot/xianyu/server
cargo build --release
```

产物在 `target/release/server`。

#### 本地交叉编译后上传

Windows 下使用 WSL 或交叉编译工具链编译 Linux 二进制，编译完成后直接上传到服务器。

### 3. 构建并上传前端

```bash
cd admin-web
npm install
npm run build
```

将 `dist/` 内容上传到服务器 `/www/wwwroot/xianyu/dist/`。

### 4. 准备目录结构

```text
/www/wwwroot/xianyu/
├── server                  ← 编译好的二进制
├── config.json             ← 配置文件
├── dist/                   ← 前端构建产物
│   ├── index.html
│   └── assets/
├── uploads/                ← 上传目录
└── api/                    ← 公告、关于页等 JSON 配置目录
    ├── announcement.json
    └── about_config.json
```

### 5. 写入配置文件

创建 `/www/wwwroot/xianyu/config.json`：

```json
{
  "db_host": "127.0.0.1",
  "db_port": 3306,
  "db_name": "chexian",
  "db_user": "chexian",
  "db_pass": "宝塔创建数据库时设的密码",
  "db_charset": "utf8mb4",
  "api_secret": "替换为随机字符串",
  "api_timestamp_tolerance": 300,
  "admin_username": "admin",
  "admin_password": "替换为强密码",
  "listen_addr": "127.0.0.1:8081",
  "jwt_secret": "替换为随机字符串",
  "email_api_primary": "",
  "email_api_backup": "",
  "email_sender": "no-reply@example.com",
  "email_password": "",
  "static_dir": "./dist",
  "local_debug_no_db": false
}
```

`listen_addr` 设为 `127.0.0.1:8081`，由 Nginx 反向代理到公网。

### 6. 设置权限

```bash
chown -R www:www /www/wwwroot/xianyu
chmod +x /www/wwwroot/xianyu/server
chmod -R 755 /www/wwwroot/xianyu/uploads
```

### 7. 宝塔建站与反向代理

宝塔「网站」→「添加站点」：

- 域名：`api.example.com`
- 根目录：`/www/wwwroot/xianyu/dist`
- PHP 版本：纯静态

建好后点「设置」→「反向代理」→「添加反向代理」：

- 代理名称：`xianyu-api`
- 目标 URL：`http://127.0.0.1:8081`
- 发送域名：`$host`

或手动在站点 Nginx 配置中添加：

```nginx
location /api {
    proxy_pass http://127.0.0.1:8081;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}

location /admin/api {
    proxy_pass http://127.0.0.1:8081;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
}

location /uploads {
    proxy_pass http://127.0.0.1:8081;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
}
```

### 8. 配置守护进程

#### 方式 A：宝塔进程管理插件

宝塔软件商店搜索「进程管理」→ 安装 → 添加守护进程：

- 名称：`xianyu-server`
- 启动命令：`/www/wwwroot/xianyu/server`
- 运行目录：`/www/wwwroot/xianyu`
- 运行用户：`www`

#### 方式 B：Systemd（推荐）

```bash
cat > /etc/systemd/system/xianyu.service << 'EOF'
[Unit]
Description=XianYu Music Server
After=network.target mysql.service

[Service]
Type=simple
User=www
WorkingDirectory=/www/wwwroot/xianyu
ExecStart=/www/wwwroot/xianyu/server
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable xianyu
systemctl start xianyu
```

### 9. 验证

- 后台管理：`https://api.example.com/login`
- API 健康检查：`https://api.example.com/api?action=check`
- 返回 JSON 即说明服务正常

## 无数据库模式与缓存迁移

服务端支持在数据库不可用时自动以本地缓存模式启动，无需预先配置数据库即可运行。适合快速部署或数据库暂未就绪的场景。

### 自动降级机制

服务端启动时会探测数据库连接：

- 数据库可连接：正常数据库模式启动，自动建表。
- 数据库不可连接：自动进入本地缓存模式，数据缓存在 `data/debug/state.json`，不阻塞启动。

本地缓存模式下，后台使用 `config.json` 中的 `admin_username` 和 `admin_password` 登录。

### 从缓存迁移到数据库

1. 启动服务端（数据库未配置时自动进入缓存模式）。
2. 登录后台 → 系统管理 → 配置文件管理。
3. 填写数据库连接信息并保存。
4. 点击「迁移本地缓存」按钮。
5. 服务端会使用新配置连接数据库、自动建表，并将缓存中的用户、用户设置、反馈数据导入数据库。本地缓存中的明文测试密码会自动转为 bcrypt 哈希。
6. 迁移完成后重启服务端。
7. 重启后数据库可连接，自动切换到正式数据库模式。

### Windows 一键启动

项目根目录提供 `一键启动服务端.bat`，双击即可启动：

- 优先运行已编译的 `server.exe`。
- 没有 `exe` 时尝试 `cargo run` 源码启动。
- 数据库不可用时自动进入本地缓存模式。

## 部署后检查

按下面顺序检查：

1. 访问后台登录页，确认静态资源正常加载。
2. 登录后台，检查仪表盘是否能加载数据。
3. 打开 `https://xymusic.example.com/api?action=check`，确认 API 能返回服务端响应。
4. 在后台「系统管理 -> 邮箱机设置」配置邮箱机并发送测试邮件。
5. 如启用人机验证，在后台「系统管理 -> 人机验证设置」保存 Turnstile 或 hCaptcha 配置。
6. 在客户端服务器 API 地址中填写 `https://xymusic.example.com/api`，不要填写 `/admin/api`。
7. 测试客户端登录、注册、找回密码和发送邮箱验证码流程。

### 更新发布

后续更新时，按下面顺序替换产物：

1. 重新构建 `admin-web/dist/` 和 `server/target/release/server`。
2. 上传新的 `admin-web/dist/` 覆盖旧静态文件。
3. 停止服务端：`systemctl stop xianyu-music-server`。
4. 替换 `/www/wwwroot/xymusic.example.com/server/server`。
5. 确认 `config.json` 不被覆盖。
6. 启动服务端：`systemctl start xianyu-music-server`。
7. 查看日志：`journalctl -u xianyu-music-server -f`。

## 安全说明

- `config.json` 包含数据库密码、签名密钥和 JWT 密钥，不要提交到公开仓库。
- `api_secret` 和 `jwt_secret` 生产环境必须改为随机长字符串。
- 后台初始密码部署后应立即修改。
- 邮箱授权码、人机验证 Secret 只保存在服务端或后台配置中，不要写入客户端。
