#!/bin/bash
# ============================================
#   弦予音乐服务端 - Linux 一键启动
# ============================================

cd "$(dirname "$0")"

# 尝试进入 server 子目录（兼容开发目录和部署目录两种结构）
if [ -f "server/server" ]; then
    cd server
    SERVER_BIN="./server"
elif [ -f "server" ]; then
    SERVER_BIN="./server"
elif [ -f "server/target/release/server" ]; then
    cd server
    SERVER_BIN="./target/release/server"
elif [ -f "Cargo.toml" ]; then
    # 源码模式，尝试用 cargo run
    if command -v cargo &>/dev/null; then
        echo "检测到 Rust/Cargo，正在编译并启动..."
        cargo run
        exit 0
    fi
    echo "未找到编译好的 server 二进制，也未检测到 cargo。"
    echo "请先运行「构建部署包.sh」编译服务端，或安装 Rust 后再运行本脚本。"
    exit 1
else
    echo "未找到 server 二进制文件。"
    echo "请先运行「构建部署包.sh」编译服务端。"
    exit 1
fi

# 确保有执行权限
chmod +x "$SERVER_BIN" 2>/dev/null

echo ""
echo "========================================"
echo "  弦予音乐服务端一键启动"
echo "========================================"
echo ""
echo "工作目录：$(pwd)"
echo ""
echo "说明："
echo "- 后台地址：http://服务器IP:8081/login"
echo "- 如果数据库暂不可用，服务端会自动进入本地缓存模式。"
echo "- 可登录后台的「配置文件管理」页面配置数据库。"
echo "- 配置数据库后，可在后台点击「迁移本地缓存」导入数据。"
echo "- 修改数据库连接或监听地址后，请重启本进程。"
echo "- 按 Ctrl+C 可停止服务端。"
echo ""

echo "正在启动服务端..."
echo ""
exec "$SERVER_BIN"
