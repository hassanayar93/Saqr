#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
VERSION="${1:-6.0.1}"
OUT="$ROOT/dist/Saqr-Windows-Home-v${VERSION}"
ZIP="$ROOT/dist/Saqr-Windows-Home-v${VERSION}.zip"

echo "Building Windows GNU target..."
cargo build --release -p saqr-cli --target x86_64-pc-windows-gnu

rm -rf "$OUT"
mkdir -p "$OUT/bin" "$OUT/web/control_terminal" "$OUT/logs" "$OUT/data"

cp "target/x86_64-pc-windows-gnu/release/saqr.exe" "$OUT/bin/saqr.exe"
cp -a web/control_terminal/. "$OUT/web/control_terminal/"
cp deploy/windows-local/env.template "$OUT/env.template"
cp deploy/windows-local/README_AR.md "$OUT/اقرأني.txt"
cp deploy/windows-local/START_SAQR.bat "$OUT/START_SAQR.bat"
cp deploy/windows-local/OPEN_DASHBOARD.bat "$OUT/OPEN_DASHBOARD.bat"
cp deploy/windows-local/CREATE_ENV.bat "$OUT/CREATE_ENV.bat"

cd "$ROOT/dist"
rm -f "$ZIP"
zip -r "$(basename "$ZIP")" "$(basename "$OUT")"
echo "Created: $ZIP"
ls -lh "$ZIP"
