@echo off
chcp 65001 >nul
cd /d "%~dp0..\.."
echo Building SAQR release (first time may take 10-20 minutes)...
where cargo >nul 2>&1
if errorlevel 1 (
  echo Rust not found. Install from https://rustup.rs then retry.
  pause
  exit /b 1
)
cargo build --release -p saqr-cli
if errorlevel 1 (
  echo Build failed.
  pause
  exit /b 1
)
echo OK: target\release\saqr.exe
pause
