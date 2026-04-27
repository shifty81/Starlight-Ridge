# Starlight_Ridge_phase24_asset_lab_live_preview.zip

Purpose:
Adds the first practical editor asset workflow while keeping the current Phase 23 visual baseline.

Replaced:
- crates/app/src/lib.rs
- crates/engine_render_gl/src/lib.rs
- assets/textures/terrain_atlas_phase17_generated.png
- content/maps/starter_farm/layers.ron

Added:
- tools/asset_lab.html
- tools/asset_lab_server.py
- RUN_ASSET_LAB.bat
- RUN_ASSET_LAB_GIT_BASH.sh
- docs/editor/ASSET_LAB_PHASE24.md

What this enables:
- Browser-based local Asset Lab for pixel editing PNG atlases.
- RGB + alpha palette.
- 32x32 tile slicing with atlas-cell selection.
- Template examples for grass/path/sand/water/cliff-style tiles.
- 3x3 in-game style preview.
- Static idle animation strip authoring.
- Direct project-file saves through a local Python server.
- Timestamped backups before PNG/RON overwrites.
- Runtime hot reload in the game/editor window for assets/textures and content files.
- F5 manual reload in editor mode.

How to use:
1. Extract this zip over the Starlight Ridge project root and overwrite.
2. Run cargo check.
3. Launch Starlight Ridge or Starlight Ridge Editor.
4. Run RUN_ASSET_LAB.bat from the project root.
5. Edit/save a tile from Asset Lab.
6. The open game/editor window should hot reload the atlas/map without rebuilding.

Notes:
- This is the editor asset-lab foundation, not the final native map/scene/prefab editor.
- The next editor phase should move the Asset Lab controls into the native editor shell and add map painting against layers.ron.
- Cargo is unavailable in the container, so this patch is source-checked but not compile-tested here.
