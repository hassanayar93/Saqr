#!/usr/bin/env bash
# Builds Saqr-Windows-Complete-*.zip (official home launcher package)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
VERSION="${1:-6.1.0}"
LAUNCH="$ROOT/deploy/saqr-launcher"
OUT="$ROOT/dist/Saqr-Windows-Complete-v${VERSION}"
ZIP="$ROOT/dist/Saqr-Windows-Complete-v${VERSION}.zip"

python3 "$ROOT/scripts/fix-bat-crlf.py" "$LAUNCH" 2>/dev/null || python3 "$ROOT/scripts/fix-bat-crlf.py"

echo "Building Windows GNU target..."
cargo build --release -p saqr-cli --target x86_64-pc-windows-gnu

rm -rf "$OUT"
mkdir -p "$OUT/bin" "$OUT/web/control_terminal" "$OUT/logs" "$OUT/data"

cp "target/x86_64-pc-windows-gnu/release/saqr.exe" "$OUT/bin/saqr.exe"
cp -a web/control_terminal/. "$OUT/web/control_terminal/"
cp "$LAUNCH"/SAQR_RUN.bat "$LAUNCH"/SAQR_RUN.ps1 "$OUT/"
cp "$LAUNCH"/SAQR_DIAGNOSE.bat "$LAUNCH"/SAQR_DIAGNOSE.ps1 "$OUT/"
cp "$LAUNCH"/env.template "$OUT/env.template"
cp "$LAUNCH"/دليل_التشغيل_من_الألف_للياء.md "$OUT/دليل_التشغيل.md"

python3 "$ROOT/scripts/fix-bat-crlf.py" "$OUT"

cd "$ROOT/dist"
rm -f "$ZIP"
zip -r "$(basename "$ZIP")" "$(basename "$OUT")"
echo "Created: $ZIP"
ls -lh "$ZIP"
