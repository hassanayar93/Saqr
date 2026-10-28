# حل مشاكل التشغيل على Windows

## هل أحتاج تثبيت مكتبات (Python / Node)؟

**لا.** الحزمة تحتوي **`bin\saqr.exe`** جاهزاً. لا pip ولا npm.

**اختياري فقط** إذا **`saqr.exe --version`** لا يعمل أبداً:

- [Microsoft Visual C++ Redistributable x64](https://aka.ms/v14/vc/redist/x64/)

---

## هيكل المجلد (بعد فك ZIP)

```
C:\Saqr\Saqr-Windows-Home-v6.0.3\
  START_SAQR.bat
  DIAGNOSE.bat
  CREATE_ENV.bat
  env.template
  .env          ← تنشئه أنت
  bin\saqr.exe
  web\control_terminal\index.html
```

إذا **`bin\saqr.exe`** غير موجود → ZIP ناقص أو مجلد خاطئ.

---

## خطوات بالترتيب

1. حمّل **آخر Release** (v6.0.3 أو أحدث).
2. **`DIAGNOSE.bat`** — يجب `[OK]` على exe و web و `.env`.
3. **`CREATE_ENV.bat`** → عدّل `.env`:
   - للتشغيل الأول بدون Telegram: `SAQR_REQUIRE_INTEGRATED_DEMO=false`
   - لاحقاً فعّل Telegram وضع `true`.
4. **`bin\saqr.exe --check`** — يجب `validation passed`.
5. **`START_SAQR.bat`** — لا Enter أثناء التشغيل؛ **Ctrl+C** للإيقاف.

---

## أخطاء `.env` الشائعة

| الخطأ | الحل |
|--------|------|
| Telegram partial | عبّئ TOKEN و CHAT_ID معاً أو اتركهما فارغين + `SAQR_REQUIRE_INTEGRATED_DEMO=false` |
| Token corrupted | انسخ من BotFather بدون مسافات |
| chat not found | /start للبوت + Chat ID صحيح |

---

## Enter يغلق النافذة

طبيعي **بعد** `pause` في نهاية BAT. شغّل **`DIAGNOSE.bat`** أولاً وليس Enter عشوائياً.
