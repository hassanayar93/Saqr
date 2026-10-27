"""حلقة تشغيل صقر — لا ترفع استثناءات غير معالجة."""
from __future__ import annotations

import logging
import time

from config.settings import Settings
from saqr.exchange.binance_client import BinanceClient

logger = logging.getLogger(__name__)


class SaqrRunner:
    def __init__(self, settings: Settings) -> None:
        self._settings = settings
        demo = settings.mode.lower() == "demo"
        if settings.live_enabled:
            logger.warning(
                "SAQR_MODE=live — تأكد أنك تفهم المخاطر قبل التداول الحقيقي."
            )
        self._client = BinanceClient(
            api_key=settings.api_key,
            api_secret=settings.api_secret,
            testnet=settings.testnet,
            demo=demo,
        )

    def run_once(self) -> None:
        s = self._settings
        if s.mode.lower() == "demo":
            logger.info(
                "[demo] صقر جاهز | symbol=%s | max_order=%s USDT (لا اتصال API)",
                s.symbol,
                s.max_order_usdt,
            )
            return

        if not self._client.ping():
            logger.error("فشل الاتصال بـ Binance (%s)", "testnet" if s.testnet else "live")
            return

        price = self._client.ticker_price(s.symbol)
        if price is not None:
            logger.info("سعر %s ≈ %s", s.symbol, price)

        acct = self._client.account_snapshot()
        if acct is not None:
            logger.debug("account keys: %s", list(acct.keys())[:5])

        # طبقة استراتيجيات لاحقاً — حالياً مراقبة فقط
        logger.info(
            "دورة مراقبة | symbol=%s | interval=%ss",
            s.symbol,
            s.loop_interval_sec,
        )

    def run_loop(self, *, max_cycles: int | None = None) -> None:
        cycles = 0
        while max_cycles is None or cycles < max_cycles:
            try:
                self.run_once()
            except Exception:
                logger.exception("خطأ في دورة صقر — الاستمرار")
            cycles += 1
            if max_cycles is not None and cycles >= max_cycles:
                break
            time.sleep(max(1, self._settings.loop_interval_sec))
