@echo off
chcp 65001 >nul 2>&1
title Cross-Compile for Android (aarch64)

echo ============================================
echo   Cross-Compile for Android aarch64
echo ============================================
echo.

set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

:: ====== 1. 检查 Rust target ======
echo [1/5] Checking Rust aarch64-linux-android target...
rustup target list --installed | findstr "aarch64-linux-android" >nul 2>&1
if errorlevel 1 (
    echo       Adding target...
    rustup target add aarch64-linux-android
) else (
    echo       Target already installed.
)

:: ====== 2. 检查 Android NDK ======
echo.
echo [2/5] Checking Android NDK...

:: 尝试常见 NDK 路径
set "NDK_PATH="
if defined ANDROID_NDK_HOME set "NDK_PATH=%ANDROID_NDK_HOME%"
if defined ANDROID_NDK_ROOT set "NDK_PATH=%ANDROID_NDK_ROOT%"

if not defined NDK_PATH (
    :: 检查常见安装位置
    if exist "%LOCALAPPDATA%\Android\Sdk\ndk" (
        for /d %%d in ("%LOCALAPPDATA%\Android\Sdk\ndk\*") do set "NDK_PATH=%%d"
    )
    if exist "%USERPROFILE%\AppData\Local\Android\Sdk\ndk" (
        for /d %%d in ("%USERPROFILE%\AppData\Local\Android\Sdk\ndk\*") do set "NDK_PATH=%%d"
    )
)

if not defined NDK_PATH (
    echo.
    echo [ERROR] Android NDK not found!
    echo.
    echo Please install Android NDK:
    echo   Option A: Install Android Studio + NDK via SDK Manager
    echo   Option B: Download from https://developer.android.com/ndk/downloads
    echo.
    echo Then set environment variable:
    echo   set ANDROID_NDK_HOME=C:\path\to\ndk
    echo.
    pause
    exit /b 1
)

echo       NDK found: %NDK_PATH%

:: 确定 NDK clang 路径
set "NDK_TOOLCHAIN=%NDK_PATH%\toolchains\llvm\prebuilt\windows-x86_64\bin"
if not exist "%NDK_TOOLCHAIN%\aarch64-linux-android24-clang.cmd" (
    echo [ERROR] NDK clang not found at: %NDK_TOOLCHAIN%
    pause
    exit /b 1
)
echo       Toolchain: %NDK_TOOLCHAIN%

:: ====== 3. 写入 cargo 配置 ======
echo.
echo [3/5] Writing .cargo/config.toml...
if not exist ".cargo" mkdir ".cargo"

(
echo [target.aarch64-linux-android]
echo linker = "%NDK_TOOLCHAIN%\aarch64-linux-android24-clang.cmd"
echo.
echo [env]
echo CC_aarch64_linux_android = "%NDK_TOOLCHAIN%\aarch64-linux-android24-clang.cmd"
echo CXX_aarch64_linux_android = "%NDK_TOOLCHAIN%\aarch64-linux-android24-clang++.cmd"
echo AR_aarch64_linux_android = "%NDK_TOOLCHAIN%\llvm-ar.exe"
) > ".cargo\config.toml"

echo       Done.

:: ====== 4. 编译 ======
echo.
echo [4/5] Compiling for aarch64-linux-android (release)...
echo       This may take 5-15 minutes...
echo.
cd /d "%~dp0server"
cargo build --release --target aarch64-linux-android
if errorlevel 1 (
    echo.
    echo [ERROR] Compilation failed!
    pause
    exit /b 1
)

:: ====== 5. 输出结果 ======
echo.
echo [5/5] Build successful!
echo.
echo Binary location:
echo   server\target\aarch64-linux-android\release\server
echo.
echo Next steps:
echo   1. Build frontend:  cd admin-web ^&^& npm run build
echo   2. Copy to phone:
echo      - server binary  -> ~/xianyu/server
echo      - admin-web/dist -> ~/xianyu/dist
echo      - server/config.json -> ~/xianyu/config.json (edit DB settings)
echo   3. Run deploy-termux.sh on phone
echo.
echo ============================================
pause
