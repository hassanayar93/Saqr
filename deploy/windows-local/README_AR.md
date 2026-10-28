# تشغيل صقر على حاسوب Windows (مؤقت — بدون VPS)

لا يمكن لوكيل Cursor Cloud **نسخ الملفات مباشرة** إلى جهازك. الحل: **تنزيل حزمة من GitHub** أو **استنساخ المستودع** وتشغيل السكربتات أدناه.

## الطريقة 1 — حزمة جاهزة (موصى بها)

1. افتح: https://github.com/hassanayar93/Saqr/releases  
2. حمّل **`Saqr-Windows-Home-v6.0.1.zip`** (أحدث).  
3. فك الضغط في `C:\Saqr`  
4. شغّل **`CREATE_ENV.bat`** أو انسخ `env.template` → `.env`  
5. عبّئ **Telegram** (مطلوب) + **Binance Testnet** (مفاتiح اختيارية للأسعار الحية في demo)  
6. **`START_SAQR.bat`** — إن فشل، افتح `logs\saqr-last-run.log`  
7. **`OPEN_DASHBOARD.bat`** → http://127.0.0.1:8787  

**Demo متكامل:** Binance Testnet حقيقي (WS + REST) + Telegram + لوحة — **بدون أوامر** (`SAQR_ENABLE_TRADING=false`).

## الطريقة 2 — بناء من المصدر على جهازك

1. ثبّت [Git](https://git-scm.com/) و [Rust](https://rustup.rs/) (Visual Studio Build Tools إن طُلب).  
2. PowerShell **كمسؤول ليس ضرورياً** — نافذة عادية:

```powershell
cd $HOME\Desktop
git clone https://github.com/hassanayar93/Saqr.git Saqr
cd Saqr
powershell -ExecutionPolicy Bypass -File .\deploy\windows-local\SETUP_HOME_PC.ps1
```

3. عدّل `C:\Saqr\.env` أو `.env` داخل مجلد المشروع.  
4. `START_SAQR.bat` من نفس المجلد.

## إعداد `.env` للتجربة الآمنة

```env
SAQR_MODE=demo
SAQR_ENABLE_TRADING=false
BINANCE_TESTNET=true
SAQR_WEB_BIND_HOST=127.0.0.1
SAQR_WEB_BIND_PORT=8787
```

للـ Testnet لاحقاً: `SAQR_MODE=testnet` + مفاتiح Testnet + `SAQR_ENABLE_TRADING=true` بعد التأكد من Ping.

## جدار الحماية Windows

عند أول تشغيل قد يطلب Windows السماح — اسمح لـ **saqr.exe** على الشبكة الخاصة.

## إيقاف البوت

أغلق نافذة الـ CMD السوداء أو `Ctrl+C` داخلها.

## عند شراء VPS لاحقاً

انسخ نفس `.env` (بدون كلمات مرور VPS القديم) واستخدم `deploy/windows/` أو Linux حسب نظام السيرفر.
