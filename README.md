# صقر (Saqr)

بوت تداول لمنصة **Binance** — مشروع مستقل عن GoldenShield AI.

## الوضع الافتراضي

- **`SAQR_MODE=demo`**: تشغيل بدون مفاتيح API (آمن للتجربة).
- **`testnet`**: عند `SAQR_MODE=testnet` مع `BINANCE_TESTNET=true` يستخدم Testnet Spot.
- **`live`**: تداول حقيقي — استخدمه فقط بعد ضبط المخاطر والمفاتيح.

## التثبيت

```bash
cd saqr
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
cp .env.example .env
```

## التشغيل

```bash
python main.py --once
python main.py --cycles 3
python main.py
```

## GitHub

المستودع المستهدف: حساب **[hassanayar93](https://github.com/hassanayar93)** (ليس `goldenshield-ai`).

## أمان

- لا ترفع `.env` أو مفاتيح API.
- Demo/Testnet افتراضياً حتى تفعيل Live صراحة.
