#!/usr/bin/env python3
"""نقطة دخول بوت صقر (Binance)."""
from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path

_ROOT = Path(__file__).resolve().parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from config.settings import load_settings
from saqr.bot.runner import SaqrRunner


def _setup_logging(level: str) -> None:
    logging.basicConfig(
        level=getattr(logging, level, logging.INFO),
        format="%(asctime)s | %(levelname)s | %(name)s | %(message)s",
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="صقر — بوت Binance")
    parser.add_argument(
        "--once",
        action="store_true",
        help="تشغيل دورة واحدة ثم الخروج",
    )
    parser.add_argument(
        "--cycles",
        type=int,
        default=None,
        help="عدد الدورات في الحلقة (افتراضي: لا نهائي)",
    )
    args = parser.parse_args()

    settings = load_settings()
    _setup_logging(settings.log_level)

    runner = SaqrRunner(settings)
    if args.once:
        runner.run_once()
    else:
        runner.run_loop(max_cycles=args.cycles)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
