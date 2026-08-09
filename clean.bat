@echo off
chcp 65001 >nul 2>&1
title XianYu Music Server - Cache Clean

echo ============================================
echo   XianYu Music Server - Cache Clean
echo ============================================
echo.

:: Set PATH (Cargo)
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

:: [1/4] Clean Rust backend build cache (server\target)
if exist "%~dp0server\target" (
    echo [1/4] Cleaning Rust build cache...
    cd /d "%~dp0server"
    cargo clean
    cd /d "%~dp0"
) else (
    echo [1/4] No cargo target, skip
)
echo.

:: [2/4] Clean local debug temporary storage
if exist "%~dp0server\data\debug" (
    echo [2/4] Removing local debug temporary storage...
    rmdir /s /q "%~dp0server\data\debug"
) else (
    echo [2/4] No local debug storage, skip
)
echo.

:: [3/4] Clean Vite cache + old frontend build (admin-web\dist, .vite)
if exist "%~dp0admin-web\dist" (
    echo [3/4] Removing old frontend dist...
    rmdir /s /q "%~dp0admin-web\dist"
) else (
    echo [3/4] No dist, skip
)
if exist "%~dp0admin-web\node_modules\.vite" (
    echo [3/4] Removing Vite dev cache (.vite)...
    rmdir /s /q "%~dp0admin-web\node_modules\.vite"
)
echo.

:: [4/4] Clean npm cache
echo [4/4] Cleaning npm cache...
call npm cache clean --force >nul 2>&1
echo       Done
echo.

echo ============================================
echo   Cache cleaned! Run start.bat to rebuild.
echo ============================================
echo.
pause
