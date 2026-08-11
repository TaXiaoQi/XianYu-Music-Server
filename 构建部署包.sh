#!/bin/bash
# ============================================
#   弦予音乐服务端 - Linux 构建部署包
# ============================================
set -e

cd "$(dirname "$0")"
ROOT_DIR="$(pwd)"

echo "============================================"
echo "  弦予音乐服务端 - Linux 构建部署包"
echo "============================================"
echo ""

# ===== 检查环境 =====
echo "[1/5] 检查编译环境..."
echo ""

# 检查 Node.js
if ! command -v node &>/dev/null; then
    echo "  [X] 未检测到 Node.js"
    echo "      请安装 Node.js 18+:"
    echo "      curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -"
    echo "      sudo apt-get install -y nodejs"
    exit 1
fi
echo "  Node.js: $(node --version)"

# 检查 npm
if ! command -v npm &>/dev/null; then
    echo "  [X] 未检测到 npm"
    exit 1
fi
echo "  npm: $(npm --version)"

# 检查 Rust/Cargo
CARGO_CMD=""
if command -v cargo &>/dev/null; then
    CARGO_CMD="cargo"
elif command -v rustup &>/dev/null; then
    CARGO_CMD="rustup run stable cargo"
else
    echo "  [X] 未检测到 Rust/Cargo"
    echo "      请安装 Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
echo "  Rust: $($CARGO_CMD --version)"
echo "  环境检查通过"
echo ""

# ===== 安装前端依赖 =====
echo "[2/5] 检查前端依赖..."
cd "$ROOT_DIR/admin-web"
if [ ! -d "node_modules" ]; then
    echo "      正在安装 npm 依赖..."
    npm install
    if [ $? -ne 0 ]; then
        echo "  [X] npm install 失败"
        exit 1
    fi
else
    echo "      前端依赖已存在，跳过安装"
fi
echo ""

# ===== 构建前端 =====
echo "[3/5] 构建后台管理前端..."
npm run build
if [ $? -ne 0 ]; then
    echo "  [X] 前端构建失败"
    exit 1
fi
echo "  前端构建完成 -> admin-web/dist/"
echo ""

# ===== 构建 Rust 服务端 =====
echo "[4/5] 编译 Rust 服务端 (release 模式，首次编译较慢)..."
cd "$ROOT_DIR/server"
$CARGO_CMD build --release
if [ $? -ne 0 ]; then
    echo "  [X] Rust 编译失败"
    exit 1
fi
echo "  Rust 编译完成 -> server/target/release/server"
echo ""

# ===== 打包部署目录 =====
echo "[5/5] 打包部署文件..."
cd "$ROOT_DIR"

DEPLOY_DIR="$ROOT_DIR/deploy"
DEPLOY_SERVER="$DEPLOY_DIR/server"
DEPLOY_ADMIN="$DEPLOY_DIR/admin-web/dist"

# 清理旧的部署目录
rm -rf "$DEPLOY_DIR"
mkdir -p "$DEPLOY_SERVER"
mkdir -p "$DEPLOY_ADMIN"

# 复制服务端二进制
cp -f "server/target/release/server" "$DEPLOY_SERVER/server"
chmod +x "$DEPLOY_SERVER/server"

# 复制前端构建产物
cp -rf admin-web/dist/* "$DEPLOY_ADMIN/"

# 复制配置文件
if [ -f "server/config.json" ]; then
    cp -f "server/config.json" "$DEPLOY_SERVER/config.json"
else
    cat > "$DEPLOY_SERVER/config.json" << 'JSONEOF'
{
  "db_host": "127.0.0.1",
  "db_port": 3306,
  "db_name": "chexian",
  "db_user": "chexian",
  "db_pass": "",
  "db_charset": "utf8mb4",
  "api_secret": "bf027fedb4d1b4f969c10495f12f17042bf0de02de128200",
  "api_timestamp_tolerance": 300,
  "admin_username": "admin",
  "admin_password": "adminadmin",
  "listen_addr": "0.0.0.0:8081",
  "jwt_secret": "bf027fedb4d1b4f969c10495f12f17042bf0de02de128200",
  "email_api_primary": "",
  "email_api_backup": "",
  "email_sender": "no-reply@example.com",
  "email_password": "",
  "turnstile_secret": "",
  "hcaptcha_secret": "",
  "captcha_secret": "",
  "static_dir": "../admin-web/dist",
  "local_debug_no_db": false
}
JSONEOF
fi

# 复制一键启动脚本
cp -f "一键启动服务端.sh" "$DEPLOY_DIR/一键启动服务端.sh"
chmod +x "$DEPLOY_DIR/一键启动服务端.sh"

# 创建上传目录
mkdir -p "$DEPLOY_SERVER/uploads"

echo ""
echo "============================================"
echo "  构建完成！"
echo ""
echo "  部署包目录: $DEPLOY_DIR"
echo ""
echo "  目录结构:"
echo "  deploy/"
echo "    ├── server/"
echo "    │   ├── server            (Rust 服务端)"
echo "    │   ├── config.json       (配置文件)"
echo "    │   └── uploads/          (上传目录)"
echo "    ├── admin-web/dist/       (后台管理前端)"
echo "    └── 一键启动服务端.sh      (启动脚本)"
echo ""
echo "  部署方式:"
echo "  1. 把整个 deploy 文件夹上传到服务器"
echo "  2. 编辑 server/config.json 配置数据库等"
echo "  3. 运行 ./一键启动服务端.sh 启动"
echo "  4. 打开 http://服务器IP:8081/login 登录后台"
echo ""
echo "  提示:"
echo "  - 没有数据库也能启动，自动进入本地缓存模式"
echo "  - 登录后台后可在「配置文件管理」页面配置数据库"
echo "  - 配置数据库后点击「迁移本地缓存」导入数据"
echo "  - 修改配置后需要重启服务端"
echo "============================================"
