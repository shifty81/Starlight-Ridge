# Phase 24 — Asset Lab + Live Runtime Preview

This patch introduces the first real editor-side asset pipeline instead of continuing to guess terrain visuals inside the renderer.

## Added

- `tools/asset_lab.html`
- `tools/asset_lab_server.py`
- `RUN_ASSET_LAB.bat`
- `RUN_ASSET_LAB_GIT_BASH.sh`
- runtime hot reload for texture/content changes
- `RenderBootstrap::replace_tile_map(...)`
- editor live-preview manifest at `artifacts/editor_live_preview.ron`

## Asset Lab v1 capabilities

- Lists PNG files from `assets/textures`.
- Opens a texture atlas and slices it into editable cells.
- Provides an RGB + alpha color palette.
- Allows pencil, eraser, fill, clear, and commit-to-atlas workflow.
- Shows reusable template examples for grass, path, sand, water, and cliff-like tiles.
- Shows a 3x3 in-game-style tile preview.
- Builds a simple static idle animation strip from edited tile frames.
- Saves PNG files directly back into the project through the local server.
- Saves animation metadata as RON next to the generated animation strip.
- Creates timestamped `.bak.png` or `.bak.ron` backups before overwrite.

## Live preview

Keep the game or editor window open while saving from Asset Lab. The app now watches:

- `assets/textures`
- `content/maps`
- `content/tiles`
- `content/metadata`

When a PNG/RON/JSON/TOML file changes, the renderer reloads the tile map pipeline without rebuilding the Rust project.

In editor mode, press `F5` for a manual live-preview reload.

## Important boundary

This is not the final native world editor yet. It is the asset-editing and live-preview foundation needed before the map editor, scene editor, prefab editor, and animation metadata editor are built into the native editor shell.
