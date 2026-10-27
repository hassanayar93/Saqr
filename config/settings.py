"""إعدادات صقر — قيم افتراضية آمنة."""
from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path

from dotenv import load_dotenv

_ROOT = Path(__file__).resolve().parents[1]
_ENV_PATH = _ROOT / ".env"


@dataclass(frozen=True)
class Settings:
    mode: str
    api_key: str
    api_secret: str
    testnet: bool
    symbol: str
    quote_asset: str
    max_order_usdt: float
    loop_interval_sec: int
    log_level: str

    @property
    def live_enabled(self) -> bool:
        return self.mode.lower() == "live"


def load_settings() -> Settings:
    if _ENV_PATH.is_file():
        load_dotenv(_ENV_PATH)
    else:
        load_dotenv()

    mode = os.getenv("SAQR_MODE", "demo").strip().lower()
    testnet_raw = os.getenv("BINANCE_TESTNET", "true").strip().lower()
    testnet = testnet_raw in ("1", "true", "yes", "on")

    return Settings(
        mode=mode,
        api_key=os.getenv("BINANCE_API_KEY", "").strip(),
        api_secret=os.getenv("BINANCE_API_SECRET", "").strip(),
        testnet=testnet,
        symbol=os.getenv("SAQR_SYMBOL", "BTCUSDT").strip().upper(),
        quote_asset=os.getenv("SAQR_QUOTE_ASSET", "USDT").strip().upper(),
        max_order_usdt=float(os.getenv("SAQR_MAX_ORDER_USDT", "50")),
        loop_interval_sec=int(os.getenv("SAQR_LOOP_INTERVAL_SEC", "30")),
        log_level=os.getenv("LOG_LEVEL", "INFO").strip().upper(),
    )
