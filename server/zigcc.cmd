@echo off
setlocal enabledelayedexpansion
set "OUT="
:loop
if "%~1"=="" goto done
set "AC=%~1"
set "A=%1"
set "T=!AC:~0,9!"
if "!T!"=="--target=" (
  set "TRIP=!AC:~9!"
  set "TRIP=!TRIP:unknown-=!"
  set "OUT=!OUT! -target !TRIP!"
) else (
  set "OUT=!OUT! !A!"
)
shift
goto loop
:done
"C:\zig-tools\zig.exe" cc!OUT!