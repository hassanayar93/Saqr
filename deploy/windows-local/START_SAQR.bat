@echo off
cd /d "%~dp0"
title SAQR v6.0

if not exist ".env" (
  if exist "env.template" copy /Y env.template .env
  echo Edit .env then run START_SAQR.bat again.
  notepad .env
  pause
  exit /b 1
)

if not exist "bin\saqr.exe" (
  echo ERROR: bin\saqr.exe not found in this folder.
  pause
  exit /b 1
)

if not exist "logs" mkdir logs

set RUST_LOG=info

echo.
echo SAQR folder: %CD%
echo Dashboard:   http://127.0.0.1:8787
echo.
echo Starting bot... Stop with Ctrl+C (not Enter).
echo.

bin\saqr.exe

echo.
echo Bot exited. Code: %ERRORLEVEL%
pause
