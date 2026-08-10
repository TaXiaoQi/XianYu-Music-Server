@echo off
chcp 65001 >nul
setlocal

cd /d "%~dp0server"

echo.
echo ========================================
echo  弦予音乐服务端一键启动
echo ========================================
echo.
echo 当前目录：%CD%
echo.
echo 说明：
echo - 如果数据库暂不可用，服务端会自动进入本地缓存模式。
echo - 可登录后台的「配置文件管理」页面配置数据库。
echo - 配置数据库后，可在后台点击「迁移本地缓存」导入数据。
echo - 修改数据库连接或监听地址后，请重启本窗口。
echo.

if exist "server.exe" (
  echo 检测到 server.exe，正在启动打包版服务端...
  echo.
  "server.exe"
  goto end
)

where cargo >nul 2>nul
if %errorlevel% equ 0 (
  echo 未找到 server.exe，检测到 Rust/Cargo，正在以源码方式启动...
  echo.
  cargo run
  goto end
)

echo 未找到 server.exe，也未检测到 cargo。
echo 请先放入已编译的 server.exe，或安装 Rust 后再运行本脚本。

:end
echo.
echo 服务端已退出，按任意键关闭窗口。
pause >nul
