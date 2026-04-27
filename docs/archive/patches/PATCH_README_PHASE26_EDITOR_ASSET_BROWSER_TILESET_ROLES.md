# Starlight Ridge Phase 26 — Editor Asset Browser + Tileset Roles

## Replaced files

- `tools/asset_lab.html`
- `tools/asset_lab_server.py`

## Added files

- `content/tiles/base_tileset_roles.ron`
- `docs/editor/EDITOR_LABS_PHASE26.md`
- `docs/PATCH_README_PHASE26_EDITOR_ASSET_BROWSER_TILESET_ROLES.md`
- `README_PATCH_PHASE26_EDITOR_ASSET_BROWSER_TILESET_ROLES.txt`

## What this adds

- Project-wide asset browser
- Texture/text preview from the browser
- Visual tileset role editor for `base_tileset.ron`
- Safe sidecar role metadata file: `base_tileset_roles.ron`
- Per-tile collision flags
- Metadata validation panel
- Runtime-safe padded atlas export
- Seam cleanup tools in Asset Lab

## Notes

This patch does not change Rust code. It upgrades the local editor/server workflow only.

The role editor intentionally writes a sidecar file instead of adding fields to `base_tileset.ron`. The current Rust tileset struct only expects `id`, `x`, and `y` for each named tile, so the sidecar approach avoids risking content-load failures.

The padded runtime atlas export creates the image, but the renderer still needs a later phase to consume padded atlas UVs directly.
