Starlight Ridge Phase 26 - Editor Asset Browser + Tileset Roles

Apply:
1. Extract this zip over the Starlight Ridge project root.
2. Overwrite files.
3. Run RUN_ASSET_LAB.bat or python tools/asset_lab_server.py.

This patch does not touch Rust code, so cargo check is not required.

Main additions:
- Project Browser tab
- Tileset Roles tab
- Visual role/collision editor for content/tiles/base_tileset.ron
- Safe sidecar: content/tiles/base_tileset_roles.ron
- Metadata validation panel
- Runtime-safe padded atlas export
- Seam cleanup tools: edge copy, edge blend, border soften

Important:
The padded runtime atlas export produces a safer texture, but a later renderer patch is still needed before the game runtime uses padded atlas UVs automatically.
