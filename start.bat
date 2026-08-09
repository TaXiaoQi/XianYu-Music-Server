@echo off
chcp 65001 >nul 2>&1
title XianYu Music Server - Local Debug

echo ============================================
echo   XianYu Music Server - Local Debug
echo ============================================
echo.

:: Set PATH (Node.js + Cargo + Windows tools)
set "PATH=D:\Program Files\Heavy\nodejs;%USERPROFILE%\.cargo\bin;C:\Windows\System32;C:\Windows;%PATH%"

:: Check Node.js
where node >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Node.js not found
    pause
    exit /b 1
)

:: Check Cargo
set "CARGO_CMD=cargo"
where cargo >nul 2>&1
if errorlevel 1 (
    where rustup >nul 2>&1
    if errorlevel 1 (
        echo [ERROR] Cargo/Rust not found
        pause
        exit /b 1
    )
    set "CARGO_CMD=rustup run stable cargo"
)

echo [1/4] Environment check passed
for /f "delims=" %%v in ('node --version') do echo   Node: %%v
for /f "delims=" %%v in ('%CARGO_CMD% --version') do echo   Rust: %%v
echo.

:: Stop stale local debug processes before starting new ones
echo       Cleaning stale local debug ports...
call :kill_port 8081
call :kill_port 3000
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

:: Start backend in this console (no extra window)
echo [3/4] Starting Rust backend (port 8081)...
set "LOCAL_DEBUG_NO_DB=1"
set "BACKEND_LOG=%TEMP%\xianyu_backend.log"
set "FRONTEND_LOG=%TEMP%\xianyu_frontend.log"
del "%BACKEND_LOG%" >nul 2>&1
del "%FRONTEND_LOG%" >nul 2>&1
echo       Local no-database debug mode enabled
start "XianYu-Backend" /B cmd /c "cd /d "%~dp0server" && %CARGO_CMD% run >> "%BACKEND_LOG%" 2>&1"

:: Wait for backend to compile
echo       Waiting for backend to compile...
timeout /t 3 /nobreak >nul

:: Start frontend in this console (no extra window)
echo [4/4] Starting frontend dev server (port 3000)...
start "XianYu-Frontend" /B cmd /c "cd /d "%~dp0admin-web" && npm run dev >> "%FRONTEND_LOG%" 2>&1"

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
echo   Backend Log:  %BACKEND_LOG%
echo   Frontend Log: %FRONTEND_LOG%
echo.
echo   Keep this main window open while debugging.
echo   Close this main window to stop services.
echo   Or press any key here to stop services cleanly.
echo ============================================
echo.
pause >nul

:cleanup
echo.
echo Stopping services...
call :kill_port 8081
call :kill_port 3000
echo Services stopped.
echo.
pause
exit /b 0

:kill_port
for /f "tokens=5" %%p in ('netstat -ano ^| findstr /C:":%~1" ^| findstr /C:"LISTENING"') do (
    taskkill /F /PID %%p >nul 2>&1
)
exit /b 0
