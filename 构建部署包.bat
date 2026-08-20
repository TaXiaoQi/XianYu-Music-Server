@echo off
chcp 65001 >nul 2>&1
title 弦予音乐服务端 - 构建部署包

echo ============================================
echo   弦予音乐服务端 - 构建部署包
echo ============================================
echo.

:: 设置 PATH（兼容常见安装路径）
set "PATH=D:\Program Files\Heavy\nodejs;%USERPROFILE%\.cargo\bin;C:\Windows\System32;C:\Windows;%PATH%"

:: ===== 检查环境 =====
echo [1/5] 检查编译环境...
echo.

where node >nul 2>&1
if errorlevel 1 (
    echo   [X] 未检测到 Node.js
    echo       请安装 Node.js 18+ : https://nodejs.org/
    pause
    exit /b 1
)
for /f "delims=" %%v in ('node --version') do echo   Node.js: %%v

where cargo >nul 2>&1
if errorlevel 1 (
    where rustup >nul 2>&1
    if errorlevel 1 (
        echo   [X] 未检测到 Rust/Cargo
        echo       请安装 Rust: https://rustup.rs/
        pause
        exit /b 1
    )
    set "CARGO_CMD=rustup run stable cargo"
) else (
    set "CARGO_CMD=cargo"
)
for /f "delims=" %%v in ('%CARGO_CMD% --version') do echo   Rust: %%v
echo   环境检查通过
echo.

:: ===== 安装前端依赖 =====
echo [2/5] 检查前端依赖...
cd /d "%~dp0admin-web"
if not exist "node_modules" (
    echo       正在安装 npm 依赖...
    call npm install
    if errorlevel 1 (
        echo   [X] npm install 失败
        pause
        exit /b 1
    )
) else (
    echo       前端依赖已存在，跳过安装
)
echo.

:: ===== 构建前端 =====
echo [3/5] 构建后台管理前端...
call npm run build
if errorlevel 1 (
    echo   [X] 前端构建失败
    pause
    exit /b 1
)
echo   前端构建完成 -> admin-web\dist\
echo.

:: ===== 构建 Rust 服务端 =====
echo [4/5] 编译 Rust 服务端 (release 模式，首次编译较慢)...
cd /d "%~dp0server"
%CARGO_CMD% build --release
if errorlevel 1 (
    echo   [X] Rust 编译失败
    pause
    exit /b 1
)
echo   Rust 编译完成 -> server\target\release\server.exe
echo.

:: ===== 打包部署目录 =====
echo [5/5] 打包部署文件...
cd /d "%~dp0"

set "DEPLOY_DIR=%~dp0deploy"
set "DEPLOY_SERVER=%DEPLOY_DIR%\server"
set "DEPLOY_ADMIN=%DEPLOY_DIR%\admin-web\dist"

:: 清理旧的部署目录
if exist "%DEPLOY_DIR%" rmdir /s /q "%DEPLOY_DIR%"
mkdir "%DEPLOY_DIR%"
mkdir "%DEPLOY_SERVER%"
mkdir "%DEPLOY_ADMIN%"

:: 复制服务端 exe
copy /Y "server\target\release\server.exe" "%DEPLOY_SERVER%\server.exe" >nul

:: 复制前端构建产物
xcopy /E /I /Y "admin-web\dist\*" "%DEPLOY_ADMIN%\" >nul

:: 复制配置文件模板
if exist "server\config.json" (
    copy /Y "server\config.json" "%DEPLOY_SERVER%\config.json" >nul
) else (
    echo   {> "%DEPLOY_SERVER%\config.json"
    echo   "db_host": "127.0.0.1",>> "%DEPLOY_SERVER%\config.json"
    echo   "db_port": 3306,>> "%DEPLOY_SERVER%\config.json"
    echo   "db_name": "chexian",>> "%DEPLOY_SERVER%\config.json"
    echo   "db_user": "chexian",>> "%DEPLOY_SERVER%\config.json"
    echo   "db_pass": "",>> "%DEPLOY_SERVER%\config.json"
    echo   "db_charset": "utf8mb4",>> "%DEPLOY_SERVER%\config.json"
    echo   "api_secret": "bf027fedb4d1b4f969c10495f12f17042bf0de02de128200",>> "%DEPLOY_SERVER%\config.json"
    echo   "api_timestamp_tolerance": 300,>> "%DEPLOY_SERVER%\config.json"
    echo   "admin_username": "admin",>> "%DEPLOY_SERVER%\config.json"
    echo   "admin_password": "adminadmin",>> "%DEPLOY_SERVER%\config.json"
    echo   "listen_addr": "0.0.0.0:8081",>> "%DEPLOY_SERVER%\config.json"
    echo   "jwt_secret": "",>> "%DEPLOY_SERVER%\config.json"
    echo   "email_api_primary": "",>> "%DEPLOY_SERVER%\config.json"
    echo   "email_api_backup": "",>> "%DEPLOY_SERVER%\config.json"
    echo   "email_sender": "no-reply@example.com",>> "%DEPLOY_SERVER%\config.json"
    echo   "email_password": "",>> "%DEPLOY_SERVER%\config.json"
    echo   "turnstile_secret": "",>> "%DEPLOY_SERVER%\config.json"
    echo   "hcaptcha_secret": "",>> "%DEPLOY_SERVER%\config.json"
    echo   "captcha_secret": "",>> "%DEPLOY_SERVER%\config.json"
    echo   "static_dir": "../admin-web/dist",>> "%DEPLOY_SERVER%\config.json"
    echo   "local_debug_no_db": false>> "%DEPLOY_SERVER%\config.json"
    echo   }>> "%DEPLOY_SERVER%\config.json"
)

:: 复制一键启动脚本
copy /Y "一键启动服务端.bat" "%DEPLOY_DIR%\一键启动服务端.bat" >nul

echo.
echo ============================================
echo   构建完成！
echo.
echo   部署包目录: %DEPLOY_DIR%
echo.
echo   目录结构:
echo   deploy\
echo     ├── server\
echo     │   ├── server.exe        (Rust 服务端)
echo     │   └── config.json       (配置文件)
echo     ├── admin-web\dist\        (后台管理前端)
echo     └── 一键启动服务端.bat      (双击启动)
echo.
echo   部署方式:
echo   1. 把整个 deploy 文件夹上传到服务器
echo   2. 编辑 server\config.json 配置数据库等
echo   3. 双击「一键启动服务端.bat」启动
echo   4. 打开 http://服务器IP:8081 登录后台
echo.
echo   提示:
echo   - 没有数据库也能启动，自动进入本地缓存模式
echo   - 登录后台后可在「配置文件管理」页面配置数据库
echo   - 配置数据库后点击「迁移本地缓存」导入数据
echo   - 修改配置后需要重启服务端
echo ============================================
echo.
pause
