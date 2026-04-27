# Phase 51f — egui World Layer Editor Real Paint/Save

Patch artifact: `Starlight_Ridge_phase51f_egui_world_layer_editor_real_paint_save.zip`

## Purpose

Phase 51f turns the egui World tab from a static/clickable preview into a real mutable map-layer editor. The editor now owns an `EditorMapState` for the active map, loads `content/maps/<map_id>/layers.ron` directly, edits the selected layer rows in memory, tracks dirty state, and saves the edited map layers back to disk with an automatic backup.

This remains egui-only. It does not move the project back toward the old GL overlay editor path and does not implement the upcoming real texture-backed atlas picker.

## Implemented

- Added mutable `EditorMapState` in `crates/app/src/egui_editor.rs`.
- Loads editable `content/maps/<map_id>/layers.ron` for the active map.
- Tracks selected map, selected layer, selected tile, selected layer symbol, dirty state, and last drag-painted cell.
- Adds real map-layer editing tools:
  - Brush writes the selected symbol into the selected layer rows.
  - Erase writes `.` into the selected layer rows.
  - Fill flood-fills a contiguous region on the selected layer.
  - Pick reads the selected layer cell and updates selected symbol/tile.
  - Drag painting works for brush and erase.
- Adds selected layer controls in the World tab.
- Adds selected symbol/tile controls from the selected layer legend.
- Adds map brush-size control.
- Adds Save Map Layers button and Ctrl+S save routing for World/map dirty state.
- Adds `game_data::loader::save_map_layers_with_backup`.
- Creates backups named like `layers.phase51f.<unix_timestamp>.bak.ron` before overwriting `layers.ron`.
- Rebuilds registry/preview after save/reload.
- Updates the bottom validation panel to show real layer issues for the active editable map.
- Keeps the static bottom status bar and adds layer dirty/clean state to it.

## Files changed

- `crates/app/src/egui_editor.rs`
- `crates/game_data/src/loader.rs`
- `docs/PHASE51F_EGUI_WORLD_LAYER_EDITOR.md`
- `content/maps/starter_farm/layers.ron`

## Editor behavior

### Map selection

Use the left Maps panel to choose an active map. Switching maps reloads that map's editable `layers.ron` and rebuilds the preview.

### Layer selection

Use the World tab's layer combo to choose which layer is being edited. The layer visibility checkbox controls whether that layer contributes to the preview and marks the map dirty when changed.

### Tile and symbol selection

The current selected tile still comes from the existing tile list/inspector flow. The World tab also exposes the selected layer legend. Selecting a symbol in the layer legend updates the current tile. If the current selected tile is not already present in the selected layer's legend, brush/fill can allocate an unused symbol and add a legend entry automatically.

### Painting

Keyboard/tool mapping remains aligned with the existing toolbar:

- `B` — Brush
- `E` — Erase
- `G` — Fill
- `I` — Pick
- `Ctrl+S` — Save map layers

Brush and erase support drag painting on the map canvas. Fill and pick execute on click.

### Save/reload

Saving writes the current `EditorMapState.layers` to `content/maps/<map_id>/layers.ron`, backs up the previous file first, reloads content, and rebuilds the preview. Reload discards unsaved in-memory map edits by re-reading the files from disk.

## Validation panel

The bottom Validation tab now checks the active editable map for:

- missing editable `layers.ron`
- `map_id` mismatch
- invalid tile dimensions
- empty layer list
- row count mismatch against map metadata
- row width mismatch against map metadata
- invalid or duplicate legend symbols
- legend tile IDs missing from the active tileset and terrain catalog
- unmapped non-empty symbols used in layer rows

## Expected result

The World tab should now allow the editor user to:

1. choose a map,
2. choose a layer,
3. choose a tile/symbol,
4. paint directly on the map,
5. erase cells,
6. fill regions,
7. pick existing tiles from the active layer,
8. save the edited map,
9. reload and see changes persist.

## Content note

`content/maps/starter_farm/layers.ron` now has the `ground` layer visible by default, so the editable preview and runtime path show the base terrain under object layers immediately.

## Known follow-up

Phase 51g should replace the current color-block preview/tile list handoff with the real atlas texture picker:

`Starlight_Ridge_phase51g_egui_real_atlas_tile_picker.zip`

Target focus:

- real atlas texture preview in egui
- clickable tile selection from the atlas
- zoom/pan atlas view
- tile metadata display
- tile-to-map paint handoff
