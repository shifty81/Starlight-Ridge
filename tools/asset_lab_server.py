#!/usr/bin/env python3
"""Starlight Ridge Editor Labs local server.

Run from the project root:
    python tools/asset_lab_server.py
Then open http://127.0.0.1:8724/tools/asset_lab.html
"""
from __future__ import annotations

import base64
import json
import mimetypes
import re
import time
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, unquote, urlparse

PORT = 8724
ROOT = Path.cwd().resolve()
ALLOWED_READ_ROOTS = [ROOT / "assets", ROOT / "content", ROOT / "tools", ROOT / "docs"]
ALLOWED_WRITE_ROOTS = [ROOT / "assets", ROOT / "content", ROOT / "artifacts", ROOT / "docs"]
ASSET_EXTS = {".png", ".ron", ".json", ".toml", ".txt", ".md", ".vert", ".frag"}
TEXT_EXTS = {".ron", ".json", ".toml", ".txt", ".md", ".vert", ".frag", ".html", ".css", ".js"}


def safe_path(relative: str, roots: list[Path]) -> Path:
    relative = unquote(relative).replace("\\", "/").lstrip("/")
    path = (ROOT / relative).resolve()
    if not any(path == root or root in path.parents for root in roots):
        raise ValueError(f"path outside project editable roots: {relative}")
    return path


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def list_textures() -> list[str]:
    texture_root = ROOT / "assets" / "textures"
    if not texture_root.exists():
        return []
    return sorted(rel(path) for path in texture_root.rglob("*.png") if path.is_file())


def classify_asset(path: Path) -> str:
    s = rel(path).lower()
    if s.endswith(".png"):
        if "/characters/" in s:
            return "character texture"
        if "atlas" in s or "terrain" in s:
            return "texture atlas"
        return "texture"
    if s.endswith(".ron"):
        if "/maps/" in s:
            return "map data"
        if "/tiles/" in s:
            return "tileset metadata"
        if "/metadata/" in s:
            return "sprite metadata"
        return "content metadata"
    if s.endswith((".vert", ".frag")):
        return "shader"
    if s.endswith((".md", ".txt")):
        return "documentation"
    return path.suffix.lower().lstrip(".") or "file"


def collect_references() -> dict[str, list[str]]:
    refs: dict[str, list[str]] = {}
    for root in [ROOT / "content", ROOT / "tools", ROOT / "docs"]:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file() or path.suffix.lower() not in TEXT_EXTS:
                continue
            text = read_text(path)
            for match in re.findall(r"(?:assets|content|tools|docs)/[^\"'\s,)]+", text):
                refs.setdefault(match, []).append(rel(path))
    return refs


def list_assets() -> list[dict]:
    refs = collect_references()
    rows = []
    for root in [ROOT / "assets", ROOT / "content"]:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file() or path.suffix.lower() not in ASSET_EXTS:
                continue
            rp = rel(path)
            stat = path.stat()
            rows.append({
                "path": rp,
                "name": path.name,
                "type": classify_asset(path),
                "size": stat.st_size,
                "mtime": int(stat.st_mtime),
                "folder": rel(path.parent),
                "used_by": sorted(set(refs.get(rp, []))),
            })
    return sorted(rows, key=lambda r: (r["type"], r["path"]))


def parse_tileset(path: Path) -> dict:
    text = read_text(path)
    def grab(name: str, default=None):
        m = re.search(rf"\b{name}\s*:\s*([^,\n)]+)", text)
        if not m:
            return default
        value = m.group(1).strip()
        if value.startswith('"'):
            return value.strip('"')
        try:
            return int(value)
        except ValueError:
            return value

    entries = []
    for m in re.finditer(r"\(\s*id\s*:\s*\"([^\"]+)\"\s*,\s*x\s*:\s*(\d+)\s*,\s*y\s*:\s*(\d+)\s*\)", text):
        entries.append({"id": m.group(1), "x": int(m.group(2)), "y": int(m.group(3))})
    return {
        "id": grab("id", "base_tiles"),
        "display_name": grab("display_name", path.name),
        "texture_path": grab("texture_path", ""),
        "tile_width": int(grab("tile_width", 32) or 32),
        "tile_height": int(grab("tile_height", 32) or 32),
        "columns": int(grab("columns", 1) or 1),
        "rows": int(grab("rows", 1) or 1),
        "named_tiles": entries,
        "path": rel(path),
    }


def guess_role(tile_id: str) -> str:
    s = tile_id.lower()
    pairs = [
        ("water", "water"), ("shore", "shore"), ("sand", "sand"), ("path", "path"),
        ("dirt", "path"), ("tilled", "crop_soil"), ("grass", "grass"), ("cliff", "cliff"),
        ("stone", "stone"), ("wood", "wood"), ("fence", "fence"), ("gate", "door"),
        ("wall", "wall"), ("roof", "building"), ("farmhouse", "building"), ("door", "door"),
        ("tree", "blocking_prop"), ("boulder", "blocking_prop"), ("rock", "blocking_prop"),
        ("crate", "blocking_prop"), ("barrel", "blocking_prop"), ("flowers", "decor"), ("bush", "decor"),
    ]
    for key, role in pairs:
        if key in s:
            return role
    return "decor"


def default_collision(role: str) -> dict:
    blocking = role in {"water", "cliff", "wall", "building", "blocking_prop", "fence"}
    return {
        "collision": "blocked" if blocking else "walkable",
        "walkable": not blocking,
        "blocks_movement": blocking,
        "water": role in {"water", "shore"},
        "interactable": role in {"door", "crop_soil"},
        "crop_soil": role == "crop_soil",
        "door": role == "door",
    }


def roles_path_for_tileset(tileset_path: Path) -> Path:
    return tileset_path.with_name(f"{tileset_path.stem}_roles.ron")


def parse_roles(path: Path) -> dict[str, dict]:
    if not path.exists():
        return {}
    text = read_text(path)
    roles: dict[str, dict] = {}
    for block in re.findall(r"\(([^()]*tile_id\s*:\s*\"[^\"]+\"[^()]*)\)", text, flags=re.S):
        def str_field(name: str, default: str = "") -> str:
            m = re.search(rf"\b{name}\s*:\s*\"([^\"]*)\"", block)
            return m.group(1) if m else default
        def bool_field(name: str, default: bool = False) -> bool:
            m = re.search(rf"\b{name}\s*:\s*(true|false)", block)
            return (m.group(1) == "true") if m else default
        tid = str_field("tile_id")
        if not tid:
            continue
        role = str_field("role", guess_role(tid))
        base = default_collision(role)
        roles[tid] = {
            "tile_id": tid,
            "role": role,
            "collision": str_field("collision", base["collision"]),
            "walkable": bool_field("walkable", base["walkable"]),
            "blocks_movement": bool_field("blocks_movement", base["blocks_movement"]),
            "water": bool_field("water", base["water"]),
            "interactable": bool_field("interactable", base["interactable"]),
            "crop_soil": bool_field("crop_soil", base["crop_soil"]),
            "door": bool_field("door", base["door"]),
        }
    return roles


def merged_tile_roles(tileset_path: Path) -> dict:
    tileset = parse_tileset(tileset_path)
    sidecar = roles_path_for_tileset(tileset_path)
    existing = parse_roles(sidecar)
    entries = []
    for tile in tileset["named_tiles"]:
        tid = tile["id"]
        role = existing.get(tid)
        if role is None:
            r = guess_role(tid)
            role = {"tile_id": tid, "role": r, **default_collision(r)}
        entries.append({**tile, **role})
    return {"tileset": tileset, "roles_path": rel(sidecar), "entries": entries}


def roles_to_ron(tileset_id: str, source: str, entries: list[dict]) -> str:
    def b(v: object) -> str:
        return "true" if bool(v) else "false"
    lines = ["(", f'    tileset: "{tileset_id}",', f'    source: "{source}",', "    entries: ["]
    for e in entries:
        role = e.get("role") or guess_role(e.get("tile_id", ""))
        base = default_collision(role)
        lines.append(
            f'        (tile_id: "{e.get("tile_id", "")}", role: "{role}", collision: "{e.get("collision", base["collision"])}", '
            f'walkable: {b(e.get("walkable", base["walkable"]))}, blocks_movement: {b(e.get("blocks_movement", base["blocks_movement"]))}, '
            f'water: {b(e.get("water", base["water"]))}, interactable: {b(e.get("interactable", base["interactable"]))}, '
            f'crop_soil: {b(e.get("crop_soil", base["crop_soil"]))}, door: {b(e.get("door", base["door"]))}),'
        )
    lines += ["    ],", ")", ""]
    return "\n".join(lines)


def terrain_type_ids() -> set[str]:
    path = ROOT / "content" / "terrain" / "terrain_types.ron"
    if not path.exists():
        return set()
    return set(re.findall(r"\bid\s*:\s*\"([^\"]+)\"", read_text(path)))


def validate_project() -> list[dict]:
    issues: list[dict] = []
    tileset_path = ROOT / "content" / "tiles" / "base_tileset.ron"
    named_ids: set[str] = set()
    if not tileset_path.exists():
        issues.append({"level": "error", "path": "content/tiles/base_tileset.ron", "message": "base_tileset.ron is missing"})
    else:
        t = parse_tileset(tileset_path)
        texture_path = ROOT / t["texture_path"]
        if not texture_path.exists():
            issues.append({"level": "error", "path": t["path"], "message": f"tileset texture_path is missing: {t['texture_path']}"})
        seen_ids: dict[str, int] = {}
        seen_xy: dict[tuple[int, int], list[str]] = {}
        for tile in t["named_tiles"]:
            tid, x, y = tile["id"], tile["x"], tile["y"]
            named_ids.add(tid)
            seen_ids[tid] = seen_ids.get(tid, 0) + 1
            seen_xy.setdefault((x, y), []).append(tid)
            if x >= t["columns"] or y >= t["rows"]:
                issues.append({"level": "error", "path": t["path"], "message": f"tile {tid} coordinate {x},{y} is outside {t['columns']}x{t['rows']} atlas"})
        for tid, count in seen_ids.items():
            if count > 1:
                issues.append({"level": "warn", "path": t["path"], "message": f"duplicate tile id: {tid} appears {count} times"})
        for xy, ids in seen_xy.items():
            if len(ids) > 1:
                issues.append({"level": "info", "path": t["path"], "message": f"atlas cell {xy[0]},{xy[1]} is shared by: {', '.join(ids[:8])}"})
        role_data = merged_tile_roles(tileset_path)
        role_ids = {e["tile_id"] for e in role_data["entries"]}
        for role_id in sorted(role_ids - named_ids):
            issues.append({"level": "warn", "path": role_data["roles_path"], "message": f"role metadata references unknown tile id: {role_id}"})
        missing_roles = named_ids - role_ids
        if missing_roles:
            issues.append({"level": "info", "path": role_data["roles_path"], "message": f"{len(missing_roles)} named tiles have no saved role entry yet; defaults will be guessed in the editor"})
    known_layer_targets = named_ids | terrain_type_ids()
    maps_root = ROOT / "content" / "maps"
    if maps_root.exists():
        for layers_path in maps_root.glob("*/layers.ron"):
            text = read_text(layers_path)
            for tile_id in re.findall(r"tile_id\s*:\s*\"([^\"]+)\"", text):
                if tile_id not in known_layer_targets:
                    issues.append({"level": "warn", "path": rel(layers_path), "message": f"layer legend references unknown tile/terrain id: {tile_id}"})
    content_root = ROOT / "content"
    if content_root.exists():
        for path in content_root.rglob("*.ron"):
            text = read_text(path)
            for key, target in re.findall(r"\b(texture_path|texture)\s*:\s*\"([^\"]+\.png)\"", text):
                if not (ROOT / target).exists():
                    issues.append({"level": "warn", "path": rel(path), "message": f"{key} points at missing PNG: {target}"})
    if not issues:
        issues.append({"level": "ok", "path": "project", "message": "No validation issues found by the phase 26 editor checks."})
    return issues


def grab_ron_field(text: str, name: str, default=None):
    m = re.search(rf"\b{name}\s*:\s*([^,\n)]+)", text)
    if not m:
        return default
    value = m.group(1).strip()
    if value.startswith('"'):
        return value.strip('"')
    try:
        return int(value)
    except ValueError:
        return value


def list_maps() -> list[dict]:
    maps_root = ROOT / "content" / "maps"
    rows: list[dict] = []
    if not maps_root.exists():
        return rows
    for map_path in sorted(maps_root.glob("*/map.ron")):
        text = read_text(map_path)
        mid = grab_ron_field(text, "id", map_path.parent.name)
        rows.append({
            "id": mid,
            "display_name": grab_ron_field(text, "display_name", mid),
            "width": int(grab_ron_field(text, "width", 0) or 0),
            "height": int(grab_ron_field(text, "height", 0) or 0),
            "tileset": grab_ron_field(text, "tileset", "base_tiles"),
            "path": rel(map_path),
            "layers_path": rel(map_path.parent / "layers.ron"),
        })
    return rows


def parse_map_layers(map_id: str) -> dict:
    layers_path = ROOT / "content" / "maps" / map_id / "layers.ron"
    map_path = ROOT / "content" / "maps" / map_id / "map.ron"
    if not layers_path.exists():
        raise ValueError(f"map layers not found: {map_id}")
    text = read_text(layers_path)
    map_text = read_text(map_path) if map_path.exists() else ""
    width = int(grab_ron_field(map_text, "width", 0) or 0)
    height = int(grab_ron_field(map_text, "height", 0) or 0)
    tile_width = int(grab_ron_field(text, "tile_width", 32) or 32)
    tile_height = int(grab_ron_field(text, "tile_height", 32) or 32)
    parsed_map_id = grab_ron_field(text, "map_id", map_id)
    layers: list[dict] = []
    pat = re.compile(r'\(\s*id\s*:\s*"([^"]+)"\s*,\s*visible\s*:\s*(true|false)\s*,\s*legend\s*:\s*\[(.*?)\]\s*,\s*rows\s*:\s*\[(.*?)\]\s*,?\s*\)', re.S)
    for m in pat.finditer(text):
        legend_text = m.group(3)
        rows_text = m.group(4)
        legend = [
            {"symbol": lm.group(1), "tile_id": lm.group(2)}
            for lm in re.finditer(r'symbol\s*:\s*"([^"]+)"\s*,\s*tile_id\s*:\s*"([^"]+)"', legend_text)
        ]
        rows = re.findall(r'"([^"]*)"', rows_text)
        layers.append({"id": m.group(1), "visible": m.group(2) == "true", "legend": legend, "rows": rows})
        width = max(width, max((len(r) for r in rows), default=0))
        height = max(height, len(rows))
    return {
        "map_id": parsed_map_id,
        "path": rel(layers_path),
        "width": width,
        "height": height,
        "tile_width": tile_width,
        "tile_height": tile_height,
        "layers": layers,
    }


def map_layers_to_ron(data: dict) -> str:
    def q(s: object) -> str:
        return str(s).replace("\\", "/").replace('"', '\\"')
    lines = ["(", f'    map_id: "{q(data.get("map_id", "starter_farm"))}",', f'    tile_width: {int(data.get("tile_width", 32) or 32)},', f'    tile_height: {int(data.get("tile_height", 32) or 32)},', "    layers: ["]
    for layer in data.get("layers", []):
        lines += ["        (", f'            id: "{q(layer.get("id", "layer"))}",', f'            visible: {"true" if layer.get("visible", True) else "false"},', "            legend: ["]
        for entry in layer.get("legend", []):
            sym = q(entry.get("symbol", "."))[:1]
            tid = q(entry.get("tile_id", ""))
            lines.append(f'                (symbol: "{sym}", tile_id: "{tid}"),')
        lines += ["            ],", "            rows: ["]
        for row in layer.get("rows", []):
            lines.append(f'                "{q(row)}",')
        lines += ["            ],", "        ),"]
    lines += ["    ],", ")", ""]
    return "\n".join(lines)


def write_hot_reload_manifest(reason: str, map_id: str = "") -> Path:
    path = ROOT / "artifacts" / "editor_live_preview.ron"
    path.parent.mkdir(parents=True, exist_ok=True)
    safe_reason = reason.replace('"', "")
    safe_map = map_id.replace('"', "")
    path.write_text(
        "(\n"
        f'    reason: "{safe_reason}",\n'
        f'    map: "{safe_map}",\n'
        f"    timestamp: {int(time.time())},\n"
        ")\n",
        encoding="utf-8",
    )
    return path


class Handler(BaseHTTPRequestHandler):
    server_version = "StarlightRidgeEditorLabs/1.3"

    def log_message(self, fmt: str, *args) -> None:
        print("[editor-labs]", fmt % args)

    def send_bytes(self, data: bytes, content_type: str, status: int = 200) -> None:
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(data)

    def send_json(self, data: object, status: int = 200) -> None:
        self.send_bytes(json.dumps(data, indent=2).encode("utf-8"), "application/json", status)

    def read_json(self) -> dict:
        length = int(self.headers.get("Content-Length", "0"))
        return json.loads(self.rfile.read(length).decode("utf-8"))

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path == "/":
            self.send_response(302)
            self.send_header("Location", "/tools/asset_lab.html")
            self.end_headers()
            return
        try:
            if parsed.path == "/api/files":
                self.send_json({"root": ROOT.as_posix(), "textures": list_textures()})
                return
            if parsed.path == "/api/assets":
                self.send_json({"root": ROOT.as_posix(), "assets": list_assets()})
                return
            if parsed.path == "/api/validate":
                self.send_json({"issues": validate_project()})
                return
            if parsed.path == "/api/maps":
                self.send_json({"maps": list_maps()})
                return
            if parsed.path == "/api/map_layers":
                qs = parse_qs(parsed.query)
                map_id = qs.get("map", ["starter_farm"])[0]
                self.send_json(parse_map_layers(map_id))
                return
            if parsed.path == "/api/read_text":
                qs = parse_qs(parsed.query)
                path = safe_path(qs.get("path", [""])[0], ALLOWED_READ_ROOTS)
                if path.suffix.lower() not in TEXT_EXTS:
                    raise ValueError("read_text only accepts text-like project files")
                self.send_json({"path": rel(path), "text": read_text(path)})
                return
            if parsed.path == "/api/tile_roles":
                qs = parse_qs(parsed.query)
                rel_path = qs.get("path", ["content/tiles/base_tileset.ron"])[0]
                path = safe_path(rel_path, ALLOWED_READ_ROOTS)
                self.send_json(merged_tile_roles(path))
                return
            if parsed.path == "/asset":
                qs = parse_qs(parsed.query)
                path = safe_path(qs.get("path", [""])[0], ALLOWED_READ_ROOTS)
                self.send_bytes(path.read_bytes(), mimetypes.guess_type(path.name)[0] or "application/octet-stream")
                return
            rel_path = parsed.path.lstrip("/") or "tools/asset_lab.html"
            path = safe_path(rel_path, ALLOWED_READ_ROOTS)
            if path.is_dir():
                path = path / "index.html"
            self.send_bytes(path.read_bytes(), mimetypes.guess_type(path.name)[0] or "text/plain")
        except Exception as exc:
            self.send_bytes(str(exc).encode("utf-8"), "text/plain", 404)

    def do_POST(self) -> None:
        parsed = urlparse(self.path)
        try:
            body = self.read_json()
            if parsed.path == "/api/save_png":
                path = safe_path(body["path"], ALLOWED_WRITE_ROOTS)
                if path.suffix.lower() != ".png":
                    raise ValueError("save_png only accepts .png files")
                data_url = body["data_url"]
                encoded = data_url.split(",", 1)[1] if "," in data_url else data_url
                data = base64.b64decode(encoded)
                path.parent.mkdir(parents=True, exist_ok=True)
                if path.exists():
                    backup = path.with_name(f"{path.stem}.{int(time.time())}.bak{path.suffix}")
                    backup.write_bytes(path.read_bytes())
                path.write_bytes(data)
                self.send_json({"ok": True, "path": rel(path)})
                return
            if parsed.path == "/api/save_text":
                path = safe_path(body["path"], ALLOWED_WRITE_ROOTS)
                if path.suffix.lower() not in {".ron", ".json", ".toml", ".txt", ".md"}:
                    raise ValueError("save_text accepts .ron, .json, .toml, .txt, or .md")
                path.parent.mkdir(parents=True, exist_ok=True)
                if path.exists():
                    backup = path.with_name(f"{path.stem}.{int(time.time())}.bak{path.suffix}")
                    backup.write_text(read_text(path), encoding="utf-8")
                path.write_text(body["text"], encoding="utf-8")
                self.send_json({"ok": True, "path": rel(path)})
                return
            if parsed.path == "/api/save_tile_roles":
                source = body.get("source", "content/tiles/base_tileset.ron")
                source_path = safe_path(source, ALLOWED_READ_ROOTS)
                tileset = parse_tileset(source_path)
                entries = body.get("entries", [])
                sidecar = roles_path_for_tileset(source_path)
                sidecar.parent.mkdir(parents=True, exist_ok=True)
                if sidecar.exists():
                    backup = sidecar.with_name(f"{sidecar.stem}.{int(time.time())}.bak{sidecar.suffix}")
                    backup.write_text(read_text(sidecar), encoding="utf-8")
                sidecar.write_text(roles_to_ron(tileset["id"], rel(source_path), entries), encoding="utf-8")
                self.send_json({"ok": True, "path": rel(sidecar), "entries": len(entries)})
                return
            if parsed.path == "/api/save_map_layers":
                map_id = body.get("map_id", "starter_farm")
                path = safe_path(f"content/maps/{map_id}/layers.ron", ALLOWED_WRITE_ROOTS)
                path.parent.mkdir(parents=True, exist_ok=True)
                if path.exists():
                    backup = path.with_name(f"{path.stem}.{int(time.time())}.bak{path.suffix}")
                    backup.write_text(read_text(path), encoding="utf-8")
                path.write_text(map_layers_to_ron(body), encoding="utf-8")
                self.send_json({"ok": True, "path": rel(path), "layers": len(body.get("layers", []))})
                return
            if parsed.path == "/api/hot_reload_manifest":
                reason = str(body.get("reason", "web editor content update"))
                map_id = str(body.get("map", ""))
                path = write_hot_reload_manifest(reason, map_id)
                self.send_json({"ok": True, "path": rel(path)})
                return
            self.send_bytes(b"unknown endpoint", "text/plain", 404)
        except Exception as exc:
            self.send_bytes(str(exc).encode("utf-8"), "text/plain", 400)


def main() -> None:
    if not (ROOT / "Cargo.toml").exists() or not (ROOT / "assets").exists():
        print("Run this from the Starlight Ridge project root.")
        raise SystemExit(2)
    url = f"http://127.0.0.1:{PORT}/tools/asset_lab.html"
    print(f"Starlight Ridge Editor Labs: {url}")
    try:
        webbrowser.open(url)
    except Exception:
        pass
    ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()


if __name__ == "__main__":
    main()
