# Editor UX Audit

This audit covers the native egui editor as it exists now, with emphasis on missing editor capability, duplicated controls, parent-window organization, and changes that would make daily authoring feel more obvious.

## Main Diagnosis

The editor has useful working tools, but the shell is still organized like a growing debug surface. The biggest usability issue is not one broken panel; it is that the same concepts appear in too many places:

- Global tool buttons are always visible even when the active workspace cannot use them.
- Save/reload/undo controls appear in the top bar, left map panel, world paint controls, layer workspace, bottom hot reload tab, and some asset workspaces.
- World placement editing is split across World Map Paint, World Spawns, World Interactions, and Assets Props, even though props/spawns/triggers are all map objects.
- Some parent workspace tabs show real child tabs, while Animation, Character, Data, Project, Playtest, and Settings show plain text pretending to be sub-navigation.
- The right inspector is mostly tile-focused, so selected props/spawns/triggers do not feel like first-class selected objects yet.

The core rework should make the editor feel like this:

1. Choose a parent workspace.
2. Choose a mode inside that workspace.
3. See only the tools relevant to that mode.
4. Edit selection details in the inspector.
5. Save the dirty assets from one predictable place.

## Navigation Gaps

### Parent Workspaces

Current top-level workspaces are Project, World, Assets, Animation, Character, Logic, Data, Playtest, and Settings.

Problems:

- Too many parent tabs are always visible.
- Some parents are real workspaces, while others are scaffold pages.
- Top bar wraps into multiple rows and competes with the active workspace.

Recommended structure:

- Project: overview, validation, diagnostics, export.
- World: map paint, layers, objects, collision, terrain bake.
- Assets: atlas, pixel editor, voxel panels, VOX browser, seasons.
- Gameplay Data: items, crops, NPCs, dialogue, quests, shops, schedules.
- Animation: clips, timeline, events, sockets, hitboxes.
- Logic: graphs, event bindings, validation.
- Playtest: launch, runtime state, logs.
- Settings: preferences, keybinds, paths, web companion.

Move Character under Animation or Assets until it has real tools. It currently reads like another placeholder parent.

### Child Navigation

World, Assets, and Logic have actual enum-backed subtabs. Animation, Character, Data, Project, Playtest, and Settings only display text lists.

Needed:

- Add enum-backed subtabs for Project, Animation, Character, Data, Playtest, and Settings.
- Replace text-only subtab labels with real selectable controls.
- Keep each parent responsible for one child toolbar.

## Duplicated Controls

### Save / Reload

Current duplication:

- Top bar has Reload and Save checkpoint.
- Left Maps tab has Save map layers and Reload layers.
- World Map Paint has Save layers, Undo, Redo, Reload.
- World Layers repeats Save layers, Undo, Redo, layer controls.
- Bottom Hot Reload has Reload now.
- Individual props/spawns/triggers panels each have Save and Reload.

Recommended:

- Add one persistent command strip near the top or bottom:
  - Save
  - Save All Dirty
  - Reload
  - Undo
  - Redo
  - Validate
- Make the command strip context-aware:
  - World Map Paint saves dirty layers and dirty placements.
  - Assets Pixel Editor saves the edited atlas PNG.
  - Assets Voxel Panels saves the active kit.
  - Data editors save the active record file.
- Remove repeated save/reload buttons from left panels unless they are truly local to a dangerous operation.

### Tools

Current global `TOOL_NAMES` are Select, Pan, Brush, Erase, Fill, Pick, Tiles, Collision, Assets, Playtest.

Problems:

- The tool row is global, but most tools only make sense in World Map Paint.
- Some entries are modes or destinations, not tools. Tiles, Assets, and Playtest read like navigation, not canvas tools.
- Pixel editor and voxel panel designer have their own separate tool models, so the global row is misleading outside World.

Recommended:

- Replace global tools with workspace-local tool palettes.
- World Map Paint tools:
  - Select
  - Pan
  - Paint
  - Erase
  - Fill
  - Eyedropper
  - Collision
  - Object
  - Trigger
  - Spawn
- Pixel Editor tools stay inside Pixel Editor.
- Voxel Panel tools stay inside Voxel Panels.
- Playtest becomes a parent or child workspace, not a tool.

### Layer Controls

Current duplication:

- Layer selector appears in World Map Paint and World Layers.
- Brush size appears in left Maps and World Map Paint.
- Visibility appears in both selected-layer control and layer grid.

Recommended:

- Parent World workspace should have one left panel section for map/layer context.
- World Layers should be the management table only.
- World Map Paint should show only the active canvas toolbar and selected layer summary.
- Brush size belongs in the World Map Paint toolbar only.

## Inspector Gaps

Right inspector currently has Tile, Seams, Export.

Missing:

- Active object inspector for selected prop/spawn/trigger.
- Layer inspector for selected layer ID, visibility, future opacity/lock, dimensions, legend.
- Map inspector for metadata, dimensions, tileset, content file health.
- Validation inspector with clickable issue source paths.
- Dirty/save-state inspector showing which files are dirty.

Recommended right inspector tabs:

- Selection: changes based on selected tile/layer/prop/spawn/trigger/pixel/voxel.
- Map: active map metadata and content file status.
- Validation: issues for current context.
- Export: build/export/checkpoint actions.

This lets the World Objects list stay focused on selection and management while actual editing happens consistently in the inspector.

## World Editor Gaps

Already good:

- Map layer paint/erase/fill.
- Layer undo/redo.
- Layer add/remove/duplicate/reorder/visibility.
- Props/spawns/triggers CRUD/save.
- World preview overlays and click selection.
- Keyboard duplicate/delete/nudge for selected placements.
- Worldgen bake to editable layers.

Still missing:

- Multi-select edit operations.
- Collision overlay editing or collision-source inspection.
- Map resize/crop/expand.
- Per-map validation for duplicate spawn IDs, invalid trigger target maps, bad prop references, and placement bounds.
- Richer point-marker handle affordances for props/spawns.
- Dirty-state save all for layers plus placement files.

Recommended rework:

- Keep World Objects as the unified list for props, spawns, triggers, and voxel objects.
- Add richer multi-select transforms and point-marker affordances alongside the current marquee and footprint resize handles.
- World Map Paint should select and manipulate these objects on-canvas.
- Right inspector edits the selected object.

## Assets Editor Gaps

Already good:

- Pixel editor exists and is functional.
- Voxel panel designer is substantial.
- VOX browser exists.

Still missing:

- Terrain Atlas page is still note-only instead of real atlas texture browser.
- Atlas Compare / Import is note-only.
- Seasons is note-only.
- Seam inspector has placeholder actions.
- Pixel editor history is not on the shared undo stack.
- Voxel panel history is not on the shared undo stack.
- No unified asset validation report.

Recommended rework:

- Terrain Atlas should become the parent view for atlas browsing, metadata editing, and role/collision preview.
- Pixel Editor should be a child mode of Terrain Atlas or a clearly related asset tool.
- Atlas Compare / Import should graduate from notes into a two-pane import workflow before more asset categories are added.

## Data / Logic / Animation Gaps

Data:

- Needs real CRUD editors for items, crops, NPCs, dialogue, quests, shops, schedules.
- Needs schema-aware defaults and cross-reference pickers.
- Needs duplicate, rename, and safe refactor.

Logic:

- Current subpages are note-only.
- Needs a graph model, save format, node palette, connections, validation, and runtime bindings.

Animation:

- Current page is note-only.
- Needs real subtabs and data model for clips, timeline, events, sockets, hitboxes, direction sets, and preview playback.

Character:

- Currently a scaffold and probably should not be a top-level parent until it owns real authoring tools.

## Playtest / Runtime Gaps

Current playtest/runtime panels are mostly diagnostics.

Needed:

- Launch active map.
- Reset/reload runtime from editor.
- Teleport to selected spawn.
- Show live player position, facing, tile, collision, inventory, quest state, and interaction target.
- Surface runtime errors with clickable content references.

## Settings Gaps

Current settings are note-only.

Needed:

- Persisted preferences.
- Keybind editor.
- Editor paths.
- Web companion server controls.
- Autosave/backup policy.
- UI layout reset.

## Technical Architecture Gaps

The biggest maintenance issue is that `crates/app/src/egui_editor.rs` owns too much:

- UI shell.
- Content loading.
- Save/backup helpers.
- Map layer mutations.
- Placement mutations.
- Pixel editor state.
- Voxel panel designer state.
- Validation.

Stub or thin crates that should absorb this:

- `editor_data_bridge`: typed load/save/backup/dirty-state layer for editor content.
- `editor_inspector`: reusable selected-object inspector models.
- `editor_tools`: command/tool definitions, shortcuts, and tool context.
- `editor_undo`: already started, but needs adoption by pixel, voxel, placement, metadata, and data editors.

Recommended extraction order:

1. Move save/load/backup and dirty-state models into `editor_data_bridge`.
2. Move prop/spawn/trigger/layer inspector models into `editor_inspector`.
3. Move tool definitions and shortcut routing into `editor_tools`.
4. Keep `egui_editor.rs` as rendering glue.

## Interface Rework Roadmap

### UX Phase U1 - Shell Cleanup

- Replace always-visible global tool row with workspace-local toolbars.
- Add a context-aware command strip: Save, Save All Dirty, Reload, Undo, Redo, Validate.
- Convert text-only subtab labels into real enum-backed child tabs.
- Move status text out of the crowded top toolbar and keep it in the status bar.

### UX Phase U2 - Parent Window Ownership

- Put map/layer context in the World parent side panel.
- Put selected-object details in the right inspector.
- Add multi-select transform operations and richer point-marker affordances for World Objects, including voxel object placement records.
- Move Playtest out of tool row completely.
- Keep Pixel and Voxel tools inside their own asset workspaces.

### UX Phase U3 - Selection Model

- Introduce one editor selection enum:
  - Tile
  - MapCell
  - Layer
  - Prop
  - Spawn
  - Trigger
  - VoxelObject
  - PixelSelection
  - VoxelPanelCell
- Right inspector renders from this selection enum.
- Keyboard actions route through the selected object and active workspace.

### UX Phase U4 - Validation and Dirty State

- Show dirty files in a single persistent area.
- Add Save All Dirty.
- Add validation issues with source file paths and stable IDs.
- Add dirty-file prompts before map switch, reload, and exit.

### UX Phase U5 - Daily Authoring Flow

- World authoring should be possible from one screen:
  - select map
  - pick layer/object mode
  - paint/move/edit
  - inspect selection
  - save all
  - validate
  - playtest active map
- Asset authoring should be possible from one screen per asset family:
  - browse
  - edit
  - preview
  - validate
  - save

## Immediate Patch Recommendation

Initial cleanup now completed:

- Context command strip added for Save, Save All Dirty, Reload, Undo, Redo, Validate, and Checkpoint.
- Always-visible global tools replaced with a World-local tool strip; Pixel and Voxel tools remain in their native workspaces.
- Text-only subtab labels were replaced with real enum-backed child tabs for Project, Animation, Character, Data, Playtest, and Settings.
- The right inspector now follows tile, layer, prop, spawn, and trigger selection.
- The right inspector now follows pixel rectangle selection with texture, origin, size, hover, copy, and clear actions.
- The right inspector now follows voxel panel designer selection with kit, mode, panel, material, socket, composition, instance, export, and dirty-state readouts.
- The most duplicated save/reload/undo controls were removed from child panels that should defer to the command strip.
- Props, spawns, triggers, and voxel objects were merged into a World Objects panel with type filters and shared duplicate/remove actions.
- World Map Paint select mode now supports click selection and grid-snapped drag movement for props, spawns, triggers, and voxel objects.
- World Map Paint select mode now supports marquee selection for world placements and bottom-right resize handles for trigger and voxel object footprints.
- Map layers now have lock and opacity metadata with editor controls; locked layers block paint/fill/erase and opacity affects preview rendering.

Remaining near-term sequence:

1. Start `editor_data_bridge` so native and web edits stop diverging.
2. Move pixel editor and voxel panel editor histories onto shared `editor_undo`.
3. Add per-map validation for duplicate spawns, invalid triggers, bad prop references, bad dimensions, and locked-layer edit warnings.

That sequence will make the editor easier to reason about and will reduce future wiring work.
