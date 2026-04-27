# Starlight Ridge Phase 33 — Web World Preview + Tile Editing

This patch intentionally shifts editor iteration back to the browser workflow before forcing parity into the native Rust editor.

## Adds

- Browser **World Preview** tab.
- Map loading from `content/maps/*/layers.ron`.
- Canvas-rendered world viewport using `base_tileset.ron` and the active terrain atlas.
- Left-click tile selection.
- Right-click tile context menu.
- Replace selected map cell with a chosen named tile.
- Open a clicked world tile directly in Asset Lab for source editing.
- Use clicked world tile as the paint source.
- Save edited map layers with automatic backup.
- Update `artifacts/editor_live_preview.ron` after save for live reload workflows.

## Workflow

1. Run `RUN_ASSET_LAB.bat`.
2. Open the **World Preview** tab.
3. Load `starter_farm`.
4. Right-click a tile in the browser viewport.
5. Inspect, replace, or open that source tile in Asset Lab.
6. Save the map and hot reload the running game/editor.

## Current boundary

This is the web workflow prototype. It does not replace the native editor yet. The native editor should later mimic this browser UX once it is proven.

The clone/edit action currently opens the source tile in Asset Lab as the clone starting point. The next phase should add a true "Save As New Tile Variant" action that appends a new atlas cell/tile ID safely.
