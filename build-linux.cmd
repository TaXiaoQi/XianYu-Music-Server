@echo off
setlocal EnableExtensions
title XianYu Music Server - Build Linux Package

set "ROOT_DIR=%~dp0"
set "TARGET=x86_64-unknown-linux-musl"
set "DEPLOY_DIR=%ROOT_DIR%deploy-linux"
set "DEPLOY_SERVER=%DEPLOY_DIR%\server"
set "DEPLOY_ADMIN=%DEPLOY_DIR%\admin-web\dist"
set "PATH=D:\Program Files\Heavy\nodejs;%USERPROFILE%\.cargo\bin;C:\Windows\System32;C:\Windows;%PATH%"

echo ============================================
echo XianYu Music Server - Build Linux Package
echo ============================================
echo.

echo [1/6] Checking Node.js and npm
where node >nul 2>nul
if errorlevel 1 goto NO_NODE
where npm >nul 2>nul
if errorlevel 1 goto NO_NPM
node --version
call npm --version
echo.

echo [2/6] Checking Rust
where cargo >nul 2>nul
if errorlevel 1 goto NO_RUST
cargo --version
echo.

echo [3/6] Checking Linux target
rustup target list --installed | findstr "%TARGET%" >nul 2>nul
if errorlevel 1 (
    rustup target add %TARGET%
    if errorlevel 1 goto TARGET_FAIL
)
echo Target OK: %TARGET%
echo.

echo [4/6] Checking Zig and cargo-zigbuild
where zig >nul 2>nul
if errorlevel 1 goto NO_ZIG
zig version
cargo zigbuild --version >nul 2>nul
if errorlevel 1 (
    cargo install cargo-zigbuild
    if errorlevel 1 goto ZIGBUILD_FAIL
)
echo cargo-zigbuild OK
echo.

echo [5/6] Building admin web
cd /d "%ROOT_DIR%admin-web"
if errorlevel 1 goto NO_ADMIN_DIR
if not exist "node_modules" (
    call npm install
    if errorlevel 1 goto NPM_INSTALL_FAIL
)
call npm run build
if errorlevel 1 goto WEB_BUILD_FAIL
echo Admin web build OK
echo.

echo [6/6] Building Linux server
cd /d "%ROOT_DIR%server"
if errorlevel 1 goto NO_SERVER_DIR
cargo zigbuild --release --target %TARGET%
if errorlevel 1 goto RUST_BUILD_FAIL
echo Rust Linux build OK
echo.

echo Packaging deploy-linux
cd /d "%ROOT_DIR%"
if exist "%DEPLOY_DIR%" rmdir /s /q "%DEPLOY_DIR%"
mkdir "%DEPLOY_DIR%"
mkdir "%DEPLOY_SERVER%"
mkdir "%DEPLOY_ADMIN%"
mkdir "%DEPLOY_SERVER%\uploads"

copy /Y "server\target\%TARGET%\release\server" "%DEPLOY_SERVER%\server" >nul
if errorlevel 1 goto COPY_SERVER_FAIL
xcopy /E /I /Y "admin-web\dist\*" "%DEPLOY_ADMIN%\" >nul
if errorlevel 1 goto COPY_WEB_FAIL

if exist "server\config.json" (
    copy /Y "server\config.json" "%DEPLOY_SERVER%\config.json" >nul
) else (
    call :WRITE_CONFIG
)

if exist "start-server-linux.sh" copy /Y "start-server-linux.sh" "%DEPLOY_DIR%\start-server-linux.sh" >nul
if exist "xianyu-music-server.service" copy /Y "xianyu-music-server.service" "%DEPLOY_DIR%\xianyu-music-server.service" >nul

echo.
echo ============================================
echo Build finished.
echo Output: %DEPLOY_DIR%
echo.
echo Upload deploy-linux to your Linux server.
echo On Linux run:
echo   chmod +x server/server
echo   chmod +x start-server-linux.sh
echo   ./start-server-linux.sh
echo Then open:
echo   http://SERVER_IP:8081/login
echo ============================================
echo.
pause
exit /b 0

:WRITE_CONFIG
> "%DEPLOY_SERVER%\config.json" echo {
>> "%DEPLOY_SERVER%\config.json" echo   "db_host": "127.0.0.1",
>> "%DEPLOY_SERVER%\config.json" echo   "db_port": 3306,
>> "%DEPLOY_SERVER%\config.json" echo   "db_name": "chexian",
>> "%DEPLOY_SERVER%\config.json" echo   "db_user": "chexian",
>> "%DEPLOY_SERVER%\config.json" echo   "db_pass": "",
>> "%DEPLOY_SERVER%\config.json" echo   "db_charset": "utf8mb4",
>> "%DEPLOY_SERVER%\config.json" echo   "api_secret": "replace_with_random_api_secret",
>> "%DEPLOY_SERVER%\config.json" echo   "api_timestamp_tolerance": 300,
>> "%DEPLOY_SERVER%\config.json" echo   "admin_username": "admin",
>> "%DEPLOY_SERVER%\config.json" echo   "admin_password": "adminadmin",
>> "%DEPLOY_SERVER%\config.json" echo   "listen_addr": "0.0.0.0:8081",
>> "%DEPLOY_SERVER%\config.json" echo   "jwt_secret": "replace_with_random_jwt_secret",
>> "%DEPLOY_SERVER%\config.json" echo   "email_api_primary": "",
>> "%DEPLOY_SERVER%\config.json" echo   "email_api_backup": "",
>> "%DEPLOY_SERVER%\config.json" echo   "email_sender": "no-reply@example.com",
>> "%DEPLOY_SERVER%\config.json" echo   "email_password": "",
>> "%DEPLOY_SERVER%\config.json" echo   "turnstile_secret": "",
>> "%DEPLOY_SERVER%\config.json" echo   "hcaptcha_secret": "",
>> "%DEPLOY_SERVER%\config.json" echo   "captcha_secret": "",
>> "%DEPLOY_SERVER%\config.json" echo   "public_base_url": "",
>> "%DEPLOY_SERVER%\config.json" echo   "static_dir": "../admin-web/dist",
>> "%DEPLOY_SERVER%\config.json" echo   "local_debug_no_db": false
>> "%DEPLOY_SERVER%\config.json" echo }
exit /b 0

:NO_NODE
echo [ERROR] Node.js was not found.
echo Install Node.js 18 or newer: https://nodejs.org/
goto FAIL

:NO_NPM
echo [ERROR] npm was not found.
goto FAIL

:NO_RUST
echo [ERROR] Rust Cargo was not found.
echo Install Rust: https://rustup.rs/
goto FAIL

:TARGET_FAIL
echo [ERROR] Failed to add Rust Linux target.
goto FAIL

:NO_ZIG
echo [ERROR] Zig was not found.
echo Install Zig first:
echo   winget install zig.zig
goto FAIL

:ZIGBUILD_FAIL
echo [ERROR] Failed to install cargo-zigbuild.
echo Try manually:
echo   cargo install cargo-zigbuild
goto FAIL

:NO_ADMIN_DIR
echo [ERROR] Cannot enter admin-web directory.
goto FAIL

:NPM_INSTALL_FAIL
echo [ERROR] npm install failed.
goto FAIL

:WEB_BUILD_FAIL
echo [ERROR] Admin web build failed.
goto FAIL

:NO_SERVER_DIR
echo [ERROR] Cannot enter server directory.
goto FAIL

:RUST_BUILD_FAIL
echo [ERROR] Rust Linux build failed.
echo If this fails, use WSL to build the package.
goto FAIL

:COPY_SERVER_FAIL
echo [ERROR] Failed to copy Linux server binary.
goto FAIL

:COPY_WEB_FAIL
echo [ERROR] Failed to copy admin web files.
goto FAIL

:FAIL
echo.
echo Build stopped. Please check the error above.
echo.
pause
exit /b 1
