# Starlight Ridge Chat Rollup — Phase 38

## Current direction

Starlight Ridge is moving toward a native egui editor interface while keeping the game runtime renderer separate. The web Asset Lab remains useful as a prototype and LAN/tablet helper, but `editor.exe` should become a real egui/eframe application instead of the old custom OpenGL overlay editor.

## Locked editor priorities

1. Fully convert editor UI to egui/eframe.
2. Keep the game executable and renderer path separate from the editor UI path.
3. Replace placeholder/small preview sections with useful atlas workflows.
4. Keep editor-only metadata sidecars from breaking the runtime content loader.
5. Keep patch documentation under `docs/patches/` and durable system docs in topic folders.

## Immediate fixes captured

- `base_tileset_roles.ron` must be treated as editor sidecar metadata, not as a `TilesetDef`.
- The egui editor must implement the current `eframe::App::ui(...)` method.
- `editor_core::init_with_registry(...)` is needed so editor startup can report and validate atlas, export, and animation pipelines.
- Web Asset Lab templates are removed; that area is reserved for atlas compare/import.
- Web Asset Lab Color / RGBA and Display Options controls are moved to the left options rail next to the pixel editor tools.

## Missing editor systems

### Atlas compare/import

Needed next: a large side-by-side atlas workflow with source sheet on the left and project atlas on the right. It should support drag/copy/overwrite/append, mirror-aware paste, conflict warnings, tile-cell snapping, and runtime-safe export.

### Pixel editing

Needed: native egui pencil, eraser, fill, picker, selection, copy/cut/paste, undo/redo, mirror H/V, red center guides, and padded export. The browser Asset Lab already prototypes much of this behavior.

### Seasonal terrain pipeline

Every world terrain tile needs spring, summer, autumn, and winter variants. The editor needs a season switcher and a parity validator so all required seasonal cells exist.

### Coast/water transition pipeline

Coasts and waterways need many more tile variants, animated water frames, and smoother overlay transitions. Base terrain, water, shoreline, paths, watered soil, and prop layers should be separate instead of packed into one semantic layer.

### Object/prop separation

Fences, buildings, trees, stones, driftwood, shrubs, bridges, doors, and interactables should live in prop/object atlases and placement layers, not in the main world terrain atlas.

### Animation lab

Needed: timeline events, sockets, hitboxes, interaction boxes, footstep events, slash trails, water animation preview, tool-use frames, direction groups, and seasonal animation variants.

### Runtime gameplay wiring

Needed: player collision, prompts, crop-soil behavior, watered soil visuals, doors, harvestables, object interactions, NPC placement, and save/load for edited maps.

## Recommended next patch

`Starlight_Ridge_phase39_egui_atlas_compare_import.zip`

Focus:

- Native egui source atlas import panel.
- Project atlas panel.
- Drag/copy/overwrite/append workflow.
- Import queue with validation.
- Atlas merge manifest.
- Mirror-aware paste path shared with pixel editor behavior.
