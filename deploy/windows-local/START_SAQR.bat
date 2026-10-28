@echo off
setlocal EnableDelayedExpansion
chcp 65001 >nul
title SAQR - صقر v6.0
cd /d "%~dp0"
set "SAQR_ROOT=%CD%"

if not exist "logs" mkdir logs

if not exist ".env" (
  if exist "env.template" (
    copy /Y "env.template" ".env" >nul
    echo [معلومة] تم إنشاء .env من env.template
    echo عدّل الملف ثم شغّل START_SAQR.bat مرة أخرى.
    notepad ".env"
    pause
    exit /b 1
  )
  echo [خطأ] لا يوجد .env — انسخ env.template الى .env
  pause
  exit /b 1
)

set "RUST_LOG=info"
set "RUST_BACKTRACE=1"

set "SAQR_EXE="
if exist "bin\saqr.exe" set "SAQR_EXE=bin\saqr.exe"
if not defined SAQR_EXE if exist "saqr.exe" set "SAQR_EXE=saqr.exe"
if not defined SAQR_EXE if exist "target\release\saqr.exe" set "SAQR_EXE=target\release\saqr.exe"

if not defined SAQR_EXE (
  echo [خطأ] saqr.exe غير موجود. حمّل Release من GitHub أو شغّل BUILD_FROM_SOURCE.bat
  pause
  exit /b 1
)

echo.
echo ========================================
echo   SAQR_ROOT: %SAQR_ROOT%
echo   EXE:       %SAQR_EXE%
echo   Dashboard: http://127.0.0.1:8787
echo   Log:       logs\saqr-last-run.log
echo ========================================
echo.

"%SAQR_ROOT%\%SAQR_EXE%" > "logs\saqr-last-run.log" 2>&1
set "EC=!ERRORLEVEL!"
echo.
echo [انتهى التشغيل] exit code=!EC!
echo راجع logs\saqr-last-run.log اذا ظهر خطأ.
type "logs\saqr-last-run.log" | more
pause
exit /b !EC!
