#!/usr/bin/env python3
"""Generate a polygon hitbox from non-transparent PNG pixels.

This reproduces the same approach used in the project:
- load RGBA image
- keep pixels with alpha > 10
- convert to bottom-left local coordinates
- compute convex hull (monotonic chain)
- print polygon points for JSON hitbox fields
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError as exc:  # pragma: no cover
    raise SystemExit(
        "Pillow is required. Install with: python3 -m pip install Pillow"
    ) from exc


DEFAULT_ALPHA_THRESHOLD = 10


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate a polygon hitbox from a PNG file.",
    )
    parser.add_argument("png_path", type=Path, help="Path to the PNG image")
    parser.add_argument(
        "--alpha-threshold",
        type=int,
        default=DEFAULT_ALPHA_THRESHOLD,
        help="Only pixels with alpha > threshold are used (default: 10)",
    )
    return parser.parse_args()


def load_opaque_pixels_bottom_left(png_path: Path, alpha_threshold: int) -> list[tuple[int, int]]:
    image = Image.open(png_path).convert("RGBA")
    width, height = image.size
    pixels = image.load()

    points: list[tuple[int, int]] = []
    for y in range(height):
        for x in range(width):
            if pixels[x, y][3] > alpha_threshold:
                # Convert from image top-left origin to bottom-left origin.
                points.append((x, (height - 1) - y))
    return points


def cross(o: tuple[int, int], a: tuple[int, int], b: tuple[int, int]) -> int:
    return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])


def convex_hull(points: list[tuple[int, int]]) -> list[tuple[int, int]]:
    unique_points = sorted(set(points))
    if len(unique_points) <= 1:
        return unique_points

    lower: list[tuple[int, int]] = []
    for point in unique_points:
        while len(lower) >= 2 and cross(lower[-2], lower[-1], point) <= 0:
            lower.pop()
        lower.append(point)

    upper: list[tuple[int, int]] = []
    for point in reversed(unique_points):
        while len(upper) >= 2 and cross(upper[-2], upper[-1], point) <= 0:
            upper.pop()
        upper.append(point)

    return lower[:-1] + upper[:-1]


def main() -> int:
    args = parse_args()

    if not args.png_path.exists():
        print(f"File not found: {args.png_path}", file=sys.stderr)
        return 1

    points = load_opaque_pixels_bottom_left(args.png_path, args.alpha_threshold)
    if not points:
        print("[]")
        return 0

    hull = convex_hull(points)
    polygon = [[x, y] for x, y in hull]
    print(json.dumps(polygon))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

