@echo off
chcp 65001 >nul
setlocal

:: 尝试进入 server 子目录（兼容开发目录和部署目录两种结构）
if exist "%~dp0server\server.exe" (
  cd /d "%~dp0server"
  goto start
)
if exist "%~dp0server.exe" (
  cd /d "%~dp0"
  goto start
)
if exist "%~dp0server\target\release\server.exe" (
  cd /d "%~dp0server"
  goto start
)

echo 未找到 server.exe，尝试以源码方式启动...
if exist "%~dp0server\Cargo.toml" (
  cd /d "%~dp0server"
  where cargo >nul 2>nul
  if %errorlevel% equ 0 (
    echo 检测到 Rust/Cargo，正在编译并启动...
    cargo run
    goto end
  )
)
echo 未找到 server.exe，也未检测到 cargo。
echo 请先运行「构建部署包.bat」编译服务端，或安装 Rust 后再运行本脚本。
goto end

:start
echo.
echo ========================================
echo  弦予音乐服务端一键启动
echo ========================================
echo.
echo 工作目录：%CD%
echo.
echo 说明：
echo - 如果数据库暂不可用，服务端会自动进入本地缓存模式。
echo - 可登录后台的「配置文件管理」页面配置数据库。
echo - 配置数据库后，可在后台点击「迁移本地缓存」导入数据。
echo - 修改数据库连接或监听地址后，请重启本窗口。
echo.

if exist "server.exe" (
  echo 检测到 server.exe，正在启动...
  echo.
  "server.exe"
  goto end
)

where cargo >nul 2>nul
if %errorlevel% equ 0 (
  echo 检测到 Rust/Cargo，正在以源码方式启动...
  echo.
  cargo run
  goto end
)

echo 启动失败：未找到可执行文件。

:end
echo.
echo 服务端已退出，按任意键关闭窗口。
pause >nul
