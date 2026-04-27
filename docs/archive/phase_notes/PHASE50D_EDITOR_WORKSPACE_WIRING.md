# Phase 50D — Editor workspace wiring and dark shell polish

This patch moves the egui editor from a clickable static shell toward a wired workspace.

## Fixes

- Forces egui dark visuals at startup.
- Adds a fixed bottom dock height so Console / Validation / Hot Reload / Runtime no longer resize the main view when clicked.
- Adds a separate static bottom status bar with map/layer/tool/dirty state.
- Routes the center panel by top-level workspace tab instead of always showing the same World Preview.
- Wires World subtabs for Map Paint, Layers, Interactions, Spawns, and Terrain Rules.
- Wires Asset subtabs for Terrain Atlas, Atlas Compare, Pixel Editor, Props, and Seasons.
- Adds runtime/playtest launch buttons.
- Adds an editor validation report for map layer row widths, unknown tile legend refs, unknown symbols, and player spawns.
- Adds editable map-layer brush/erase/fill behavior for layer rows in memory.
- Adds Save Layers / Ctrl+S to persist edited `content/maps/<map>/layers.ron`.

## Notes

This is still not the final full editor. The remaining high-value work is real texture atlas image preview/merge, sprite animation preview, object placement editing, and runtime player/controller implementation.
