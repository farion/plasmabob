#!/usr/bin/env python3
"""
Transform level JSON files:
 - remove "entity_types_path"
 - replace "terrain" with top-level "background"
 - ensure "music" is an array
 - add "layer" to each entity based on z_index:
     >125 -> "foreground", <75 -> "background", otherwise "gameplay"

Usage:
  python .\tools\transform_levels.py --dir .\assets\worlds --in-place --backup-dir .\tools\level_backups
  python .\tools\transform_levels.py --file .\assets\worlds\auralis\aqueon_level1.json --dry-run
"""

import argparse
import json
from pathlib import Path
from typing import Any, Dict


def transform_level(data: Dict[str, Any]) -> Dict[str, Any]:
    # Remove entity_types_path if present
    data.pop("entity_types_path", None)

    # Replace terrain with top-level background
    terrain = data.pop("terrain", None)
    if terrain is not None:
        if isinstance(terrain, dict) and "background" in terrain:
            data["background"] = terrain["background"]
        elif isinstance(terrain, str):
            # If terrain is a string, assume it's the background path
            data["background"] = terrain
        else:
            # Nonstandard terrain: keep as-is under "terrain_raw" for safety
            data["terrain_raw"] = terrain

    # Ensure music is an array
    music = data.get("music")
    if music is None:
        data["music"] = []
    elif isinstance(music, list):
        # leave as is
        pass
    else:
        # wrap single string into list
        data["music"] = [music]

    # Add layer to each entity based on z_index
    entities = data.get("entities")
    if isinstance(entities, list):
        for ent in entities:
            # determine z_index
            z = ent.get("z_index")
            try:
                z_val = int(z) if z is not None else None
            except Exception:
                z_val = None

            if z_val is None:
                # default to gameplay if missing or invalid
                ent["layer"] = "gameplay"
            else:
                if z_val > 125:
                    ent["layer"] = "foreground"
                elif z_val < 75:
                    ent["layer"] = "background"
                else:
                    ent["layer"] = "gameplay"
    # else: no entities or unexpected structure — do nothing

    return data


def process_file(path: Path, in_place: bool, dry_run: bool, backup_dir: Path = None, indent: int = 4):
    original = json.loads(path.read_text(encoding="utf-8"))
    transformed = transform_level(original)

    if dry_run:
        print(f"=== DRY RUN: {path} ===")
        print(json.dumps(transformed, ensure_ascii=False, indent=2))
        return

    # Ensure backup if requested
    if in_place:
        if backup_dir is not None:
            backup_dir.mkdir(parents=True, exist_ok=True)
            backup_path = backup_dir / f"{path.name}.bak"
            backup_path.write_text(json.dumps(original, ensure_ascii=False, indent=indent), encoding="utf-8")
            print(f"Backup written to {backup_path}")
        else:
            # default backup next to file with .bak extension
            bak = path.with_suffix(path.suffix + ".bak")
            bak.write_text(json.dumps(original, ensure_ascii=False, indent=indent), encoding="utf-8")
            print(f"Backup written to {bak}")

        path.write_text(json.dumps(transformed, ensure_ascii=False, indent=indent), encoding="utf-8")
        print(f"Updated {path}")

    else:
        # Not in place -> write to stdout
        print(f"--- Transformed content for {path} ---")
        print(json.dumps(transformed, ensure_ascii=False, indent=indent))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dir", "-d", type=str, help="Directory with level JSONs (will process all .json files recursively)")
    ap.add_argument("--file", "-f", type=str, help="Single level JSON file to process")
    ap.add_argument("--in-place", action="store_true", help="Write changes back to files")
    ap.add_argument("--backup-dir", type=str, default=None, help="Directory to store backups (if --in-place). If omitted, a .bak next to each file is created.")
    ap.add_argument("--dry-run", action="store_true", help="Print transformed JSON and do not write files")
    ap.add_argument("--ext", type=str, default=".json", help="File extension to process")
    args = ap.parse_args()

    if not args.dir and not args.file:
        ap.error("Either --dir or --file must be provided")

    backup_dir = Path(args.backup_dir) if args.backup_dir else None

    paths = []
    if args.file:
        p = Path(args.file)
        if not p.exists():
            raise SystemExit(f"File not found: {p}")
        paths.append(p)
    if args.dir:
        dd = Path(args.dir)
        if not dd.exists():
            raise SystemExit(f"Directory not found: {dd}")
        paths.extend([p for p in dd.rglob(f"*{args.ext}") if p.is_file()])

    for p in sorted(paths):
        process_file(p, in_place=args.in_place, dry_run=args.dry_run, backup_dir=backup_dir)


if __name__ == "__main__":
    main()

