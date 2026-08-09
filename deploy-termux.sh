#!/bin/bash
# ============================================================
#  弦予音乐服务器 - Termux 部署脚本
#  在手机 Termux 中运行此脚本完成部署
# ============================================================
set -e

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC} $1"; }
ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
err()   { echo -e "${RED}[ERROR]${NC} $1"; }

# 安装目录
INSTALL_DIR="$HOME/xianyu"
DB_NAME="chexian"
DB_USER="chexian"
DB_PASS="7Fp7YxLpCXC5k2iH"

echo ""
echo "============================================"
echo "  弦予音乐服务器 - Termux 部署"
echo "============================================"
echo ""

# ====== 1. 安装依赖 ======
info "安装系统依赖..."
pkg update -y
pkg install -y mariadb rust nodejs

ok "依赖安装完成"

# ====== 2. 初始化 MariaDB ======
info "初始化 MariaDB..."
if [ ! -d "$PREFIX/var/lib/mysql/mysql" ]; then
    mysql_install_db
fi

# 启动 MariaDB
info "启动 MariaDB..."
mysqld_safe -u root &
sleep 3

# 创建数据库和用户
info "创建数据库..."
mysql -u root <<EOF
CREATE DATABASE IF NOT EXISTS ${DB_NAME} CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER IF NOT EXISTS '${DB_USER}'@'localhost' IDENTIFIED BY '${DB_PASS}';
GRANT ALL PRIVILEGES ON ${DB_NAME}.* TO '${DB_USER}'@'localhost';
FLUSH PRIVILEGES;
EOF

ok "数据库 ${DB_NAME} 创建完成"

# ====== 3. 创建目录结构 ======
info "创建目录结构..."
mkdir -p "${INSTALL_DIR}/dist"
mkdir -p "${INSTALL_DIR}/uploads"

# ====== 4. 配置文件 ======
info "生成配置文件..."
cat > "${INSTALL_DIR}/config.json" <<EOF
{
  "db_host": "127.0.0.1",
  "db_port": 3306,
  "db_name": "${DB_NAME}",
  "db_user": "${DB_USER}",
  "db_pass": "${DB_PASS}",
  "db_charset": "utf8mb4",
  "api_secret": "bf027fedb4d1b4f969c10495f12f17042bf0de02de128200",
  "api_timestamp_tolerance": 300,
  "admin_username": "admin",
  "admin_password": "adminadmin",
  "listen_addr": "0.0.0.0:8081",
  "jwt_secret": "bf027fedb4d1b4f969c10495f12f17042bf0de02de128200",
  "email_api_primary": "http://a.bzxhkj.com/a",
  "email_api_backup": "http://a.bzxhkj.com/b",
  "email_sender": "admin@bzxhkj.com",
  "email_password": "1183265081aA"
}
EOF

ok "配置文件生成完成"

# ====== 5. 检查二进制文件 ======
if [ -f "${INSTALL_DIR}/server" ]; then
    ok "找到服务器二进制文件（交叉编译版）"
    BINARY="${INSTALL_DIR}/server"
else
    warn "未找到交叉编译二进制文件，将本地编译..."
    info "复制源码并编译（可能需要 10-20 分钟）..."

    if [ ! -d "${INSTALL_DIR}/src" ]; then
        err "请将项目源码复制到 ${INSTALL_DIR}/"
        err "或者将交叉编译的二进制文件复制到 ${INSTALL_DIR}/server"
        exit 1
    fi

    cd "${INSTALL_DIR}/src/server"
    cargo build --release
    cp target/release/server "${INSTALL_DIR}/server"
    cd "${INSTALL_DIR}"
fi

# ====== 6. 检查前端文件 ======
if [ ! -f "${INSTALL_DIR}/dist/index.html" ]; then
    warn "未找到前端文件，尝试本地构建..."

    if [ -d "${INSTALL_DIR}/src/admin-web" ]; then
        info "构建前端..."
        cd "${INSTALL_DIR}/src/admin-web"
        npm install
        npm run build
        cp -r dist/* "${INSTALL_DIR}/dist/"
        cd "${INSTALL_DIR}"
        ok "前端构建完成"
    else
        warn "请将 admin-web/dist 内容复制到 ${INSTALL_DIR}/dist/"
    fi
fi

# ====== 7. 创建启动脚本 ======
info "创建启动脚本..."
cat > "${INSTALL_DIR}/start.sh" <<'EOF'
#!/bin/bash
# 启动 MariaDB
echo "Starting MariaDB..."
mysqld_safe -u root &
sleep 2

# 启动服务器
echo "Starting XianYu Server on port 8081..."
cd ~/xianyu
./server

# 如果服务器退出，停止 MariaDB
kill %1 2>/dev/null
EOF
chmod +x "${INSTALL_DIR}/start.sh"

ok "启动脚本创建完成: ${INSTALL_DIR}/start.sh"

# ====== 8. 完成提示 ======
echo ""
echo "============================================"
echo -e "${GREEN}  部署完成！${NC}"
echo "============================================"
echo ""
echo "  安装目录:  ${INSTALL_DIR}"
echo "  数据库:    ${DB_NAME} (MariaDB)"
echo "  服务端口:  8081"
echo ""
echo "  启动服务:"
echo "    bash ~/xianyu/start.sh"
echo ""
echo "  访问地址:"
echo "    http://localhost:8081          (本机)"
echo "    http://手机IP:8081              (局域网)"
echo ""
echo "  神卓内网穿透:"
echo "    1. 在神卓客户端中添加 HTTP 隧道"
echo "    2. 本地端口填 8081"
echo "    3. 启动隧道后通过公网域名访问"
echo ""
echo "  后台登录:"
echo "    用户名: admin"
echo "    密码:   adminadmin"
echo ""
echo "============================================"
