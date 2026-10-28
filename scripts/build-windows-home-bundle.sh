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
cp "$LAUNCH"/FIX_MY_ENV.bat "$LAUNCH"/FIX_MY_ENV.ps1 "$OUT/"
cp "$LAUNCH"/env.template "$OUT/env.template"
cp "$LAUNCH"/دليل_التشغيل_من_الألف_للياء.md "$OUT/دليل_التشغيل.md"
cat > "$OUT/ابدأ_هنا.txt" <<'EOF'
صقر SAQR — حزمة التشغيل الكاملة
================================

1) فك الضغط في مجلد بسيط (مثلاً C:\Saqr)
2) دبل كليك: SAQR_RUN.bat
3) أول مرة: يُنشأ .env — عبّئ مفاتiح Binance Testnet و Telegram (بدون + قبل التوكن)
4) احفظ .env واضغط Enter في نافذة المشغّل
5) إن فشل التحقق: FIX_MY_ENV.bat ثم SAQR_RUN.bat مرة أخرى
6) تشخيص: SAQR_DIAGNOSE.bat

اللوحة: http://127.0.0.1:8787
إيقاف البوت: Ctrl+C

التفاصيل: دليل_التشغيل.md
EOF

python3 "$ROOT/scripts/fix-bat-crlf.py" "$OUT"

cd "$ROOT/dist"
rm -f "$ZIP"
zip -r "$(basename "$ZIP")" "$(basename "$OUT")"
echo "Created: $ZIP"
ls -lh "$ZIP"
