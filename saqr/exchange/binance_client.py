"""عميل Binance REST (demo / testnet / live)."""
from __future__ import annotations

import hashlib
import hmac
import logging
import time
from typing import Any
from urllib.parse import urlencode

import requests

logger = logging.getLogger(__name__)

SPOT_LIVE = "https://api.binance.com"
SPOT_TESTNET = "https://testnet.binance.vision"


class BinanceClient:
    def __init__(
        self,
        *,
        api_key: str,
        api_secret: str,
        testnet: bool,
        demo: bool,
    ) -> None:
        self._api_key = api_key
        self._api_secret = api_secret
        self._demo = demo
        self._base = SPOT_TESTNET if testnet else SPOT_LIVE
        self._session = requests.Session()
        if api_key and not demo:
            self._session.headers["X-MBX-APIKEY"] = api_key

    def ping(self) -> bool:
        if self._demo:
            return True
        try:
            r = self._session.get(f"{self._base}/api/v3/ping", timeout=15)
            return r.status_code == 200
        except requests.RequestException as exc:
            logger.warning("Binance ping failed: %s", exc)
            return False

    def ticker_price(self, symbol: str) -> float | None:
        if self._demo:
            return None
        try:
            r = self._session.get(
                f"{self._base}/api/v3/ticker/price",
                params={"symbol": symbol},
                timeout=15,
            )
            r.raise_for_status()
            data = r.json()
            return float(data["price"])
        except (requests.RequestException, KeyError, ValueError, TypeError) as exc:
            logger.warning("ticker_price failed for %s: %s", symbol, exc)
            return None

    def account_snapshot(self) -> dict[str, Any] | None:
        if self._demo or not self._api_key or not self._api_secret:
            return {"mode": "demo", "balances": []}
        try:
            return self._signed_get("/api/v3/account")
        except requests.RequestException as exc:
            logger.warning("account_snapshot failed: %s", exc)
            return None

    def _signed_get(self, path: str, params: dict[str, Any] | None = None) -> dict[str, Any]:
        p = dict(params or {})
        p["timestamp"] = int(time.time() * 1000)
        query = urlencode(p)
        sig = hmac.new(
            self._api_secret.encode("utf-8"),
            query.encode("utf-8"),
            hashlib.sha256,
        ).hexdigest()
        p["signature"] = sig
        r = self._session.get(f"{self._base}{path}", params=p, timeout=20)
        r.raise_for_status()
        return r.json()
