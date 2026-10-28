#!/usr/bin/env python3
"""Force CRLF on Windows batch files (cmd.exe breaks on LF-only)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
targets = [ROOT / "deploy" / "windows-local"]
if len(sys.argv) > 1:
    targets = [Path(sys.argv[1])]

for folder in targets:
    for path in folder.glob("*.bat"):
        text = path.read_text(encoding="utf-8").replace("\r\n", "\n").replace("\r", "\n")
        path.write_bytes(text.encode("utf-8").replace(b"\n", b"\r\n"))
        print(f"CRLF: {path}")
