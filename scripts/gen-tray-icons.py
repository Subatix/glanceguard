#!/usr/bin/env python3
"""Generate macOS menu-bar template PNGs (monochrome on transparency).

Source: programmatic eye silhouette (no external SVG). Re-run after changing geometry:
  python3 scripts/gen-tray-icons.py
Outputs: src-tauri/icons/tray-Template.png and tray-Template@2x.png
"""

from __future__ import annotations

from pathlib import Path

try:
    from PIL import Image, ImageDraw
except ImportError as e:
    raise SystemExit("Install Pillow: pip install Pillow") from e

ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / "src-tauri" / "icons"


def write_icon(size: int, path: Path) -> None:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    pad = max(2, size // 7)
    width = max(1, size // 14)
    draw.ellipse(
        [pad, pad, size - pad, size - pad],
        outline=(0, 0, 0, 255),
        width=width,
    )
    cx, cy = size // 2, size // 2
    r = max(2, size // 7)
    draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(0, 0, 0, 255))
    img.save(path)
    print(path)


def main() -> None:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    write_icon(22, OUT_DIR / "tray-Template.png")
    write_icon(44, OUT_DIR / "tray-Template@2x.png")


if __name__ == "__main__":
    main()
