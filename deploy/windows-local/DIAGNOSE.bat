@echo off
cd /d "%~dp0"
title SAQR Diagnose
echo ========== SAQR Folder Check ==========
echo Current folder:
echo   %CD%
echo.
if exist "bin\saqr.exe" (echo [OK] bin\saqr.exe) else (echo [FAIL] bin\saqr.exe missing)
if exist "web\control_terminal\index.html" (echo [OK] web UI) else (echo [FAIL] web\control_terminal\index.html missing)
if exist ".env" (echo [OK] .env) else (echo [FAIL] .env missing - run CREATE_ENV.bat)
if exist "env.template" (echo [OK] env.template) else (echo [WARN] env.template missing)
echo.
echo ========== Run saqr.exe --version ==========
if exist "bin\saqr.exe" (
  bin\saqr.exe --version
  echo exit code: %ERRORLEVEL%
) else (
  echo skipped - no exe
)
echo.
echo ========== Run saqr.exe --check (.env) ==========
if exist "bin\saqr.exe" if exist ".env" (
  bin\saqr.exe --check
  echo exit code: %ERRORLEVEL%
)
echo.
echo ========== Notes ==========
echo No Python or Node required. Only this folder + .env
echo If --version fails: install VC++ Redist x64 from Microsoft
echo If --check fails: fix .env (see env.template)
echo.
pause
