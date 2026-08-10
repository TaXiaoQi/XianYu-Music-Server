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

项目现在提供多种构建方式。你的服务器是 Linux 时，优先看“Windows 构建 Linux 包”或“WSL 构建 Linux 包”。

### 方式一：Windows 构建 Linux 包

适合在 Windows 开发机上直接生成 Linux 服务器可用的部署包。运行：

```bat
build-linux.cmd
```

也可以运行中文入口：

```bat
交叉编译Linux.bat
```

`交叉编译Linux.bat` 只是调用 `build-linux.cmd`，推荐优先运行 `build-linux.cmd`，因为英文文件名更不容易受编码影响。

第一次运行前需要准备：

| 工具 | 用途 | 安装方式 |
|---|---|---|
| Node.js | 构建后台前端 | 安装 Node.js 18+ |
| Rust | 编译服务端 | 安装 Rust |
| Zig | Windows 交叉编译 Linux 的链接器 | `winget install zig.zig` |
| cargo-zigbuild | Rust 交叉编译工具 | 脚本会自动安装，也可手动执行 `cargo install cargo-zigbuild` |
| Linux target | Rust Linux 编译目标 | 脚本会自动执行 `rustup target add x86_64-unknown-linux-musl` |

如果脚本提示：

```text
[ERROR] Zig was not found.
```

先在 PowerShell 执行：

```powershell
winget install zig.zig
```

安装完成后重新打开命令窗口，再运行：

```bat
build-linux.cmd
```

构建完成后会生成：

```text
deploy-linux/
├── server/
│   ├── server
│   ├── config.json
│   └── uploads/
├── admin-web/dist/
├── start-server-linux.sh
└── xianyu-music-server.service
```

把整个 `deploy-linux/` 上传到 Linux 服务器即可。

上传后在 Linux 服务器执行：

```bash
chmod +x server/server
chmod +x start-server-linux.sh
./start-server-linux.sh
```

如果要用 systemd 后台运行，修改并复制 `xianyu-music-server.service` 到 `/etc/systemd/system/`。

### 方式二：WSL 构建 Linux 包

这是最稳的 Windows 构建方式，因为 WSL 本身就是 Linux 环境，生成的二进制天然适配 Linux 服务器。

先在 Windows PowerShell 管理员窗口安装 WSL：

```powershell
wsl --install -d Ubuntu
```

重启后打开 Ubuntu，安装环境：

```bash
sudo apt update
sudo apt install -y curl build-essential pkg-config

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 安装 Node.js 18 (Ubuntu/Debian)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

进入项目目录。Windows 的 D 盘在 WSL 里是 `/mnt/d/`：

```bash
cd "/mnt/d/Program Files/MC/开发端/开发组/弦予音乐/XianYu-Music-Server"
```

运行 Linux 构建脚本：

```bash
chmod +x 构建部署包.sh
./构建部署包.sh
```

构建完成后会生成 `deploy/`，把整个 `deploy/` 上传到 Linux 服务器。

### 方式三：Linux 服务器直接构建

适合服务器性能足够、能安装编译环境的情况。把源码上传到 Linux 服务器后安装环境：

```bash
sudo apt update
sudo apt install -y curl build-essential pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

然后运行：

```bash
chmod +x 构建部署包.sh
./构建部署包.sh
```

输出目录是 `deploy/`。

### 方式四：手动构建

如果不使用脚本，也可以手动构建。

构建后台前端：

```bash
cd admin-web
npm install
npm run build
```

构建当前系统对应的服务端：

```bash
cd server
cargo build --release
```

构建产物路径：

| 平台 | 路径 |
|---|---|
| Linux | `server/target/release/server` |
| Windows | `server/target/release/server.exe` |

注意：在 Windows 直接执行 `cargo build --release` 得到的是 `server.exe`，只能给 Windows 服务器用，不能上传到 Linux 服务器运行。Linux 服务器要用上面的 `build-linux.cmd`、WSL 或 Linux 直接构建。

### 方式五：Windows 服务器构建

在 Windows 上运行 `构建部署包.bat`，自动构建前端和 Windows 服务端，打包到 `deploy/` 目录。

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

如果用「方式一」在服务器上构建，跳过上传步骤，直接从 `deploy/` 目录复制文件。

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

### 无数据库启动

服务端支持在没有数据库的情况下启动，自动进入本地缓存模式。所有数据暂存在 `server/data/debug/state.json`，后续可在后台「配置文件管理」页面配置数据库连接，然后点击「迁移本地缓存」将数据导入数据库。

### 测试启动

在正式配置 systemd 之前，先手动测试启动：

```bash
cd /www/wwwroot/xymusic.example.com/server
./server
```

看到服务端正常监听后，按 `Ctrl+C` 停止，继续配置 Nginx 和 systemd。

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

项目根目录已提供 `xianyu-music-server.service` 模板文件，复制到服务器后修改路径即可：

```bash
# 复制模板（Windows 构建 Linux 包时在 deploy-linux/ 目录中，Linux 构建时在 deploy/ 目录中）
sudo cp xianyu-music-server.service /etc/systemd/system/

# 修改 WorkingDirectory 和 ExecStart 路径
sudo vim /etc/systemd/system/xianyu-music-server.service
```

模板内容：

```ini
[Unit]
Description=XianYu Music Server
After=network.target mysql.service

[Service]
Type=simple
WorkingDirectory=/www/wwwroot/xymusic.example.com/server
ExecStart=/www/wwwroot/xymusic.example.com/server/server
Restart=always
RestartSec=3
User=www
Group=www
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

根据实际情况修改 `WorkingDirectory`、`ExecStart`、`User`、`Group`。

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

1. 重新构建 `admin-web/dist/` 和 `server/target/release/server`（运行 `构建部署包.sh` 或 `交叉编译Linux.bat`）。
2. 上传新的 `admin-web/dist/` 覆盖旧静态文件。
3. 停止服务端：`systemctl stop xianyu-music-server`。
4. 替换 `/www/wwwroot/xymusic.example.com/server/server`。
5. 确认 `config.json` 不被覆盖。
6. 启动服务端：`systemctl start xianyu-music-server`。
7. 查看日志：`journalctl -u xianyu-music-server -f`。

## 构建脚本说明

| 脚本 | 平台 | 说明 |
|---|---|---|
| `build-linux.cmd` | Windows | 在 Windows 上交叉编译 Linux 部署包，输出到 `deploy-linux/` |
| `交叉编译Linux.bat` | Windows | 中文入口，实际调用 `build-linux.cmd` |
| `start-server-linux.sh` | Linux | Linux 部署包里的启动脚本 |
| `构建部署包.sh` | Linux | 在 Linux 上构建前端 + Rust 服务端，打包到 `deploy/` |
| `一键启动服务端.sh` | Linux | 在 Linux 上启动服务端，兼容开发和部署目录 |
| `构建部署包.bat` | Windows | 构建 Windows 版前端 + 服务端，打包到 `deploy/` |
| `一键启动服务端.bat` | Windows | 在 Windows 上启动服务端 |
| `start.bat` | Windows | 本地开发模式，启动前端 dev server + 后端 |
| `clean.bat` | Windows | 清理构建缓存 |
| `cross-compile-android.bat` | Windows | 交叉编译 Android aarch64 版本 |

## 安全说明

- `config.json` 包含数据库密码、签名密钥和 JWT 密钥，不要提交到公开仓库。
- `api_secret` 和 `jwt_secret` 生产环境必须改为随机长字符串。
- 后台初始密码部署后应立即修改。
- 邮箱授权码、人机验证 Secret 只保存在服务端或后台配置中，不要写入客户端。
