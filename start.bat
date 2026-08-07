@echo off
chcp 65001 >nul 2>&1
title XianYu Music Server - Local Debug

echo ============================================
echo   XianYu Music Server - Local Debug
echo ============================================
echo.

:: Set PATH (Node.js + Cargo)
set "PATH=D:\Program Files\Heavy\nodejs;%USERPROFILE%\.cargo\bin;%PATH%"

:: Check Node.js
where node >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Node.js not found
    pause
    exit /b 1
)

:: Check Cargo
where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Cargo/Rust not found
    pause
    exit /b 1
)

echo [1/4] Environment check passed
for /f "delims=" %%v in ('node --version') do echo   Node: %%v
for /f "delims=" %%v in ('cargo --version') do echo   Rust: %%v
echo.

:: Check frontend dependencies
if not exist "admin-web\node_modules" (
    echo [2/4] Installing frontend dependencies...
    cd /d "%~dp0admin-web"
    call npm install
    cd /d "%~dp0"
) else (
    echo [2/4] Frontend dependencies ready
)
echo.

:: Start backend
echo [3/4] Starting Rust backend (port 8081)...
start "XianYu-Backend" /D "%~dp0server" cmd /k cargo run

:: Wait for backend to compile
echo       Waiting for backend to compile...
timeout /t 3 /nobreak >nul

:: Start frontend
echo [4/4] Starting frontend dev server (port 3000)...
start "XianYu-Frontend" /D "%~dp0admin-web" cmd /k npm run dev

:: Wait for Vite to boot up
echo       Waiting for Vite dev server...
timeout /t 5 /nobreak >nul

:: Auto open browser
echo       Opening browser...
start "" "http://localhost:3000/login"

echo.
echo ============================================
echo   Started successfully!
echo.
echo   Backend API:  http://127.0.0.1:8081
echo   Frontend:     http://localhost:3000
echo   Admin Login:  http://localhost:3000/login
echo   Email Module: http://localhost:3000/email/login
echo.
echo   Close the windows to stop services.
echo ============================================
echo.
pause
