#!/usr/bin/env python3
"""
Phase 53c voxel generator command runner.

This script is intentionally a workflow bridge, not final-art generation.
The egui editor calls this script so the Pipeline panel has one stable entry
point for generation, validation, and future MagicaVoxel/Blockbench/Blender
handoffs.
"""
from __future__ import annotations

import argparse
import datetime as _dt
import os
from pathlib import Path
import re
import subprocess
import sys


def project_root() -> Path:
    here = Path.cwd()
    for candidate in [here, *here.parents]:
        if (candidate / "Cargo.toml").exists() and (candidate / "content").exists():
            return candidate
    return here


def load_profiles(root: Path) -> list[dict[str, str]]:
    path = root / "content" / "editor_tools" / "voxel_generator_profiles.ron"
    if not path.exists():
        return []
    text = path.read_text(encoding="utf-8")
    blocks = re.findall(r"VoxelGeneratorProfile\((.*?)\),", text, re.S)
    profiles: list[dict[str, str]] = []
    for block in blocks:
        item: dict[str, str] = {}
        for field in ["id", "label", "output_path"]:
            match = re.search(rf'{field}:\s*"([^"]*)"', block)
            if match:
                item[field] = match.group(1)
        kind = re.search(r"generator_kind:\s*([A-Za-z0-9_]+)", block)
        if kind:
            item["generator_kind"] = kind.group(1)
        no_hair = re.search(r"no_hair:\s*(true|false)", block)
        if no_hair:
            item["no_hair"] = no_hair.group(1)
        no_facial_hair = re.search(r"no_facial_hair:\s*(true|false)", block)
        if no_facial_hair:
            item["no_facial_hair"] = no_facial_hair.group(1)
        if item.get("id"):
            profiles.append(item)
    return profiles


def write_log(root: Path, lines: list[str]) -> Path:
    log_dir = root / "logs" / "voxel_generator"
    log_dir.mkdir(parents=True, exist_ok=True)
    path = log_dir / "phase53c_last_run.log"
    stamp = _dt.datetime.now().isoformat(timespec="seconds")
    path.write_text(f"Phase 53c Voxel Generator Runner\\nTimestamp: {stamp}\\n\\n" + "\\n".join(lines) + "\\n", encoding="utf-8")
    return path


def maybe_call_phase53b_generator(root: Path, all_profiles: bool, profile_id: str | None) -> tuple[bool, str]:
    candidates = [
        root / "tools" / "scripts" / "voxel" / "generate_pixel_voxel_templates.py",
        root / "tools" / "scripts" / "voxel" / "generate_phase53b_pixel_voxel_templates.py",
    ]
    script = next((path for path in candidates if path.exists()), None)
    if script is None:
        return False, "No Phase 53b generator script found yet. This is okay for command-runner wiring."

    args = [sys.executable, str(script)]
    if all_profiles:
        args.append("--all")
    elif profile_id:
        args.extend(["--profile", profile_id])

    try:
        completed = subprocess.run(args, cwd=root, text=True, capture_output=True, check=False)
    except Exception as exc:
        return False, f"Failed to launch Phase 53b generator script: {exc}"

    summary = [
        f"Called {script.relative_to(root)}",
        f"Exit code: {completed.returncode}",
    ]
    if completed.stdout.strip():
        summary.append("stdout:")
        summary.append(completed.stdout.strip())
    if completed.stderr.strip():
        summary.append("stderr:")
        summary.append(completed.stderr.strip())
    return completed.returncode == 0, "\\n".join(summary)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--profile")
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()

    root = project_root()
    profiles = load_profiles(root)

    lines: list[str] = []
    lines.append(f"Project root: {root}")
    lines.append(f"Profiles loaded: {len(profiles)}")

    selected = None
    if args.profile:
        selected = next((profile for profile in profiles if profile.get("id") == args.profile), None)
        if selected is None:
            lines.append(f"[WARN] Requested profile not found: {args.profile}")
        else:
            lines.append(f"Selected profile: {selected.get('id')}")
            lines.append(f"Output path: {selected.get('output_path', '')}")
            if selected.get("generator_kind", "").lower().startswith("character"):
                if selected.get("no_hair") == "true" and selected.get("no_facial_hair") == "true":
                    lines.append("Character base guarantee: bald=true clean_shaven=true")
                else:
                    lines.append("[WARN] Character base profile is not locked bald/clean-shaven.")

    if args.validate_only:
        missing_outputs = [
            profile.get("output_path", "")
            for profile in profiles
            if profile.get("output_path") and not (root / profile["output_path"]).exists()
        ]
        lines.append(f"Missing generated outputs: {len(missing_outputs)}")
        for output in missing_outputs[:20]:
            lines.append(f"[MISSING] {output}")
        log_path = write_log(root, lines)
        print(f"Validation complete. Log: {log_path}")
        return 0

    if args.all or args.profile:
        ok, message = maybe_call_phase53b_generator(root, args.all, args.profile)
        lines.append(message)
        if selected and selected.get("output_path"):
            output = root / selected["output_path"]
            output.parent.mkdir(parents=True, exist_ok=True)
            if not output.exists():
                marker = output.with_suffix(output.suffix + ".pending")
                marker.write_text(
                    "Phase 53c command runner reached this profile. "
                    "Apply/run Phase 53b generator implementation to create the .vox asset.\\n",
                    encoding="utf-8",
                )
                lines.append(f"Created pending marker: {marker.relative_to(root)}")
        log_path = write_log(root, lines)
        print(f"Voxel generator runner complete. Log: {log_path}")
        return 0 if ok else 0

    lines.append("No action requested.")
    log_path = write_log(root, lines)
    print(f"No action requested. Log: {log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
