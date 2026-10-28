#!/usr/bin/env bash
# ينشئ Saqr-Windows-Home-*.zip للرفع على GitHub Releases (يُشغَّل من Linux/CI)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
VERSION="${1:-6.0.0}"
OUT="$ROOT/dist/Saqr-Windows-Home-v${VERSION}"
ZIP="$ROOT/dist/Saqr-Windows-Home-v${VERSION}.zip"

echo "Building Windows GNU target..."
cargo build --release -p saqr-cli --target x86_64-pc-windows-gnu

rm -rf "$OUT"
mkdir -p "$OUT/bin" "$OUT/web" "$OUT/data"

cp "target/x86_64-pc-windows-gnu/release/saqr.exe" "$OUT/bin/saqr.exe"
cp -a web/control_terminal "$OUT/web/"
cp .env.example "$OUT/env.template"
cp deploy/windows-local/README_AR.md "$OUT/اقرأني.txt"
cp deploy/windows-local/START_SAQR.bat "$OUT/START_SAQR.bat"
cp deploy/windows-local/OPEN_DASHBOARD.bat "$OUT/OPEN_DASHBOARD.bat"

# START at bundle root: saqr.exe in bin/, web adjacent
cat > "$OUT/START_SAQR.bat" << 'BAT'
@echo off
chcp 65001 >nul
title SAQR - صقر
cd /d "%~dp0"
if not exist ".env" (
  echo Copy env.template to .env and fill keys.
  pause
  exit /b 1
)
if not exist "bin\saqr.exe" (
  echo Missing bin\saqr.exe
  pause
  exit /b 1
)
set RUST_LOG=info
echo Dashboard: http://127.0.0.1:8787
bin\saqr.exe
pause
BAT

# saqr resolves web from CWD — copy layout expected by resolve_web_root
mkdir -p "$OUT/web/control_terminal"
# already copied above

cd "$ROOT/dist"
rm -f "$ZIP"
zip -r "$(basename "$ZIP")" "$(basename "$OUT")"
echo "Created: $ZIP"
ls -lh "$ZIP"
