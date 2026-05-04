#!/usr/bin/env python3
"""Compatibility wrapper for the GlanceGuard brand icon generator.

Re-run after changing geometry:
  python3 scripts/gen-tray-icons.py
Outputs: src-tauri/icons/glanceguard-source.png, tray-Template.png, and tray-Template@2x.png
"""

from __future__ import annotations

from pathlib import Path
from runpy import run_path

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    run_path(str(ROOT / "scripts" / "gen-brand-icons.py"), run_name="__main__")


if __name__ == "__main__":
    main()
