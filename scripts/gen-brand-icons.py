#!/usr/bin/env python3
"""GlanceGuard brand icons.

Outputs:
- src-tauri/icons/glanceguard-source.png (1024x1024 master, used by `tauri icon`)
- src-tauri/icons/tray-Template.png (22x22 macOS template image)
- src-tauri/icons/tray-Template@2x.png (44x44 macOS template image)

Re-run:
  python3 scripts/gen-brand-icons.py
  npm run tauri icon src-tauri/icons/glanceguard-source.png
"""

from __future__ import annotations

import math
from pathlib import Path

try:
    from PIL import Image, ImageDraw
except ImportError as exc:
    raise SystemExit("Install Pillow: pip install Pillow") from exc

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "src-tauri" / "icons"

# Calibrated for both light and dark macOS desktops at small (Dock/menu bar) and
# large (Finder/Launchpad) sizes. Mark is a custom 'G' monogram — solid off-white
# on a graphite squircle tile, no decorations.
TILE_TOP = (15, 17, 24)
TILE_BOTTOM = (28, 32, 46)
RIM_LIGHT = (255, 255, 255, 22)

MARK_COLOR = (244, 246, 250, 255)

SOFT_GLOW = (255, 255, 255)


def superellipse_points(cx: float, cy: float, rx: float, ry: float, n: float = 5.2, samples: int = 900) -> list[tuple[float, float]]:
    """Apple-like squircle outline via superellipse parametrization."""
    pts: list[tuple[float, float]] = []
    for i in range(samples):
        t = 2 * math.pi * i / samples
        c, s = math.cos(t), math.sin(t)
        x = cx + math.copysign(abs(c) ** (2.0 / n), c) * rx
        y = cy + math.copysign(abs(s) ** (2.0 / n), s) * ry
        pts.append((x, y))
    return pts


def quad(p0: tuple[float, float], p1: tuple[float, float], p2: tuple[float, float], samples: int = 160) -> list[tuple[float, float]]:
    return [
        (
            (1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * p1[0] + t * t * p2[0],
            (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * p1[1] + t * t * p2[1],
        )
        for t in (i / samples for i in range(samples + 1))
    ]


def vertical_gradient(w: int, h: int, top: tuple[int, int, int], bottom: tuple[int, int, int]) -> Image.Image:
    grad = Image.new("RGBA", (w, h), 0)
    draw = ImageDraw.Draw(grad)
    for y in range(h):
        t = y / max(1, h - 1)
        r = round(top[0] * (1 - t) + bottom[0] * t)
        g = round(top[1] * (1 - t) + bottom[1] * t)
        b = round(top[2] * (1 - t) + bottom[2] * t)
        draw.line([(0, y), (w, y)], fill=(r, g, b, 255))
    return grad


def radial_alpha(size: int, alpha_max: int, r_inner: float = 0.0, r_outer: float = 1.0, falloff: float = 2.0) -> Image.Image:
    """Square L-mode image with radial alpha falloff. Drawn small for speed, then resized."""
    s = 384
    img = Image.new("L", (s, s), 0)
    px = img.load()
    cx = cy = s / 2
    max_r = s / 2
    for y in range(s):
        for x in range(s):
            d = math.hypot(x - cx, y - cy) / max_r
            if d >= r_outer:
                a = 0
            elif d <= r_inner:
                a = alpha_max
            else:
                t = (d - r_inner) / (r_outer - r_inner)
                a = int(alpha_max * (1 - t) ** falloff)
            px[x, y] = max(0, min(255, a))
    return img.resize((size, size), Image.Resampling.LANCZOS)


def build_tile(size: int) -> tuple[Image.Image, Image.Image]:
    mask = Image.new("L", (size, size), 0)
    pts = superellipse_points(size / 2, size / 2, size / 2 - 1, size / 2 - 1)
    ImageDraw.Draw(mask).polygon(pts, fill=255)

    tile = Image.new("RGBA", (size, size), 0)
    tile.paste(vertical_gradient(size, size, TILE_TOP, TILE_BOTTOM), (0, 0), mask)

    glow_size = int(size * 1.05)
    glow_alpha = radial_alpha(glow_size, 38, r_inner=0.0, r_outer=0.95, falloff=2.4)
    glow_color = Image.new("RGBA", (glow_size, glow_size), SOFT_GLOW + (0,))
    glow_color.putalpha(glow_alpha)
    glow_layer = Image.new("RGBA", (size, size), 0)
    glow_layer.paste(glow_color, (-int(size * 0.32), -int(size * 0.36)), glow_color)
    masked_glow = Image.new("RGBA", (size, size), 0)
    masked_glow.paste(glow_layer, (0, 0), mask)
    tile = Image.alpha_composite(tile, masked_glow)

    rim = Image.new("RGBA", (size, size), 0)
    inset = max(2, int(size * 0.0045))
    inner_pts = superellipse_points(size / 2, size / 2, size / 2 - inset, size / 2 - inset)
    ImageDraw.Draw(rim).line(
        inner_pts + [inner_pts[0]],
        fill=RIM_LIGHT,
        width=max(1, int(size * 0.0035)),
        joint="curve",
    )
    tile = Image.alpha_composite(tile, rim)

    return tile, mask


def _draw_single_g(
    canvas: Image.Image,
    cx: float,
    cy: float,
    r_outer: float,
    stroke: float,
    color: tuple[int, int, int, int],
    mirror: bool,
) -> None:
    draw = ImageDraw.Draw(canvas, "RGBA")
    r_mid = r_outer - stroke / 2
    box = [cx - r_mid, cy - r_mid, cx + r_mid, cy + r_mid]

    if not mirror:
        # Bowl: 300° arc, 60° gap centered on the right.
        bowl_start = 30
        bowl_end = 330
        gap_angles = (330, 30)
        spur_x_start = cx
        spur_x_end = cx + r_outer * 0.86
    else:
        # Mirrored: 300° arc, 60° gap centered on the left.
        bowl_start = 210
        bowl_end = 150
        gap_angles = (150, 210)
        spur_x_start = cx - r_outer * 0.86
        spur_x_end = cx

    draw.arc(box, start=bowl_start, end=bowl_end, fill=color, width=int(round(stroke)))

    cap_r = stroke / 2
    for ang in gap_angles:
        ex = cx + r_mid * math.cos(math.radians(ang))
        ey = cy + r_mid * math.sin(math.radians(ang))
        draw.ellipse([ex - cap_r, ey - cap_r, ex + cap_r, ey + cap_r], fill=color)

    draw.rounded_rectangle(
        [spur_x_start, cy - stroke / 2, spur_x_end, cy + stroke / 2],
        radius=stroke / 2,
        fill=color,
    )


def draw_monogram(canvas: Image.Image, size: int, color: tuple[int, int, int, int]) -> Image.Image:
    """Interlocking 'GG' monogram: two facing Gs, mirrored across vertical center."""
    cx = cy = size / 2

    r_outer = size * 0.205
    stroke = size * 0.069
    spacing = size * 0.225  # Distance from tile center to each G center.

    _draw_single_g(canvas, cx - spacing, cy, r_outer, stroke, color, mirror=False)
    _draw_single_g(canvas, cx + spacing, cy, r_outer, stroke, color, mirror=True)

    return canvas


def build_app_icon() -> None:
    SUPERSAMPLE = 4
    SIZE = 1024
    work = SIZE * SUPERSAMPLE

    tile, mask = build_tile(work)
    canvas = draw_monogram(tile, work, MARK_COLOR)

    final = Image.new("RGBA", (work, work), 0)
    final.paste(canvas, (0, 0), mask)
    out = final.resize((SIZE, SIZE), Image.Resampling.LANCZOS)

    out_path = OUT / "glanceguard-source.png"
    out.save(out_path)
    print(out_path)


def build_tray_icon(size: int, path: Path) -> None:
    """Black-on-transparent template icon. macOS recolors it for menu bar appearance."""
    SUPERSAMPLE = 8
    work = size * SUPERSAMPLE
    canvas = Image.new("RGBA", (work, work), 0)

    # Tray glyph fills more of the canvas because there's no surrounding tile.
    cx = cy = work / 2
    r_outer = work * 0.46
    r_inner = work * 0.30
    r_mid = (r_outer + r_inner) / 2
    stroke = r_outer - r_inner

    draw = ImageDraw.Draw(canvas)
    box = [cx - r_mid, cy - r_mid, cx + r_mid, cy + r_mid]
    draw.arc(box, start=30, end=330, fill=(0, 0, 0, 255), width=int(round(stroke)))
    cap_r = stroke / 2
    for ang in (30, 330):
        ex = cx + r_mid * math.cos(math.radians(ang))
        ey = cy + r_mid * math.sin(math.radians(ang))
        draw.ellipse([ex - cap_r, ey - cap_r, ex + cap_r, ey + cap_r], fill=(0, 0, 0, 255))
    spur_left = cx + r_inner * 0.05
    spur_right = cx + r_outer
    draw.rounded_rectangle(
        [spur_left, cy - stroke / 2, spur_right, cy + stroke / 2],
        radius=stroke / 2,
        fill=(0, 0, 0, 255),
    )

    canvas.resize((size, size), Image.Resampling.LANCZOS).save(path)
    print(path)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    build_app_icon()
    build_tray_icon(22, OUT / "tray-Template.png")
    build_tray_icon(44, OUT / "tray-Template@2x.png")


if __name__ == "__main__":
    main()
