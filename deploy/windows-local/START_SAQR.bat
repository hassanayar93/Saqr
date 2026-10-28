@echo off
chcp 65001 >nul
title SAQR - صقر v6.0
cd /d "%~dp0..\.."
set SAQR_ROOT=%CD%

if not exist ".env" (
  echo.
  echo [خطأ] لا يوجد ملف .env في:
  echo   %SAQR_ROOT%
  echo.
  echo انسخ env.template الى .env وعبّئ المفاتيح.
  echo.
  pause
  exit /b 1
)

if exist "target\release\saqr.exe" (
  set SAQR_EXE=target\release\saqr.exe
) else if exist "saqr.exe" (
  set SAQR_EXE=saqr.exe
) else if exist "bin\saqr.exe" (
  set SAQR_EXE=bin\saqr.exe
) else (
  echo [خطأ] لم يُعثر على saqr.exe — استخدم حزمة Releases أو شغّل BUILD_FROM_SOURCE.bat
  pause
  exit /b 1
)

echo SAQR root: %SAQR_ROOT%
echo Starting Control Terminal: http://127.0.0.1:8787
echo.
"%SAQR_ROOT%\%SAQR_EXE%"
pause
