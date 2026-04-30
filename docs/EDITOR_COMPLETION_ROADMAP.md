# Editor Completion Roadmap

This is the working roadmap for getting the Starlight Ridge editor from "usable shell with several real tools" to "fully production-ready content editor."

The editor has two surfaces:

- Native egui editor: the main production editor.
- Web / LAN editor: lightweight map editing companion.

The finish order below favors content safety first: real save paths, undo, validation, and predictable runtime/editor data contracts before heavier 3D preview work.

---

## Current Editor State

### Solid / mostly wired

- Native egui single-shell workspace with top tabs, side panels, bottom diagnostics, reload, and checkpoint manifest.
- Native map layer editing for `layers.ron`, including save with backup.
- Native map layers support visible, locked, and opacity metadata with editor controls.
- Native pixel editor for PNG-backed atlas editing, including brush, erase, fill, line, replace color, selection, clipboard, transforms, undo/redo, save with backup.
- Tile role/collision metadata inspector with save path.
- Content reload and registry validation display.
- Worldgen generated-scene draft preview and bake-to-editable-layer commit path.
- World voxel object placement records with inspector and save path.
- VOX asset scanning/browser.
- Voxel panel designer with RON-backed kits, 2D depth-slice editing, palettes/materials, sockets, transforms, composition canvas, validation, and save with backup.
- Web / LAN map editor with paint, erase, fill, eyedropper, brush settings, layer clipboard, paste transforms, and write-mode save.

### Wired but incomplete

- Native World -> 3D Preview now has a shared `VoxelSceneRenderData` mesh preview with yaw/pitch/zoom controls, first-pass projected object-bounds picking, a selected-object context strip, and right-inspector handoff, but it is still egui-painted rather than a renderer-owned embedded GL viewport.
- World voxel objects can be edited in the World Objects panel, inspector, and 2D viewport; trigger and voxel object footprints have explicit resize handles.
- Voxel panel compositions export preview data, but no live 3D mesh/camera viewport consumes it yet.
- Voxel rig preview has profile/metadata/validation/tool-button scaffolding, but no actual skeleton/attachment overlay viewport.
- Voxel generator has command/profile plumbing, but generated output validation, registration, and visual compare are still shallow.
- Native Data tabs exist as workspace placeholders; item/crop/NPC/dialogue/quest/shop editors are not real CRUD editors yet.
- Native Logic tabs are placeholders; graph editing and runtime event binding are not implemented.
- Native Animation tabs are placeholders; timeline, events, hitboxes/sockets, and clip save validation are not implemented.
- Native Playtest tab is mostly diagnostics staging until player simulation/runtime state is wired.
- Settings tab is basic; preferences, keybind persistence, paths, and web companion controls need real persistence.
- Web editor has no undo/redo, marquee selection, collision/spawn/trigger editing, atlas import, scene bake preview, or visual validation report.

### 3D implementation status

Current reality:

- Game runtime rendering has an OpenGL voxel mesh path with depth testing, camera matrices, and `VoxelSceneRenderData` draw support layered with the existing 2D tile/sprite render path.
- Runtime scene voxel objects are loaded from `content/scenes/<map>/voxel_objects.ron`, resolved through `content/voxel_assets/voxel_asset_registry.ron`, and converted from real `.vox` sources into the shared render contract.
- Native editor 3D preview is still egui-painted, but World -> 3D Preview now consumes the same combined `VoxelSceneRenderData` vertices/indices/bounds/object ranges that the game renderer draws, including editable map voxel placements plus scene voxel objects. Scene voxel selection now tries the shared render-contract projected-bounds picker first, then falls back to the painted preview bounds.
- Voxel panel composition export exists: compositions can bake preview mesh/voxel data to RON for a future renderer to consume.
- World voxel objects are editable and draggable in the 2D World viewport, represented as selectable 3D volumes in the editor 3D preview, and rendered through the runtime shared voxel mesh payload path. The 3D preview now has direct orbit, move, footprint resize, and height-adjust tools for selected voxel placements, with dirty-state tracking until the shared mesh payload is saved/reloaded.

Estimated implementation progress:

- Data contracts and editor-side voxel placement: about 55-65% of the foundation.
- Voxel panel authoring/export path: about 60-70% of the authoring foundation.
- Real native editor 3D viewport: about 50-60%, because the editor can inspect the real scene mesh contract, select scene voxel objects from projected 3D bounds, edit map voxel placements directly in the 3D preview, and hand those selections to the right inspector, but it is not yet an embedded renderer-owned GL viewport with triangle/object-ID picking.
- Real game 3D rendering: about 35-45%, because scene voxel meshes, depth, camera projection, and `.vox` loading exist, but the broader scene graph, lighting/material model, picking, and gameplay integration remain thin.

Shortest viable path to real 3D:

1. Finish the embedded editor GL viewport handoff so World -> 3D Preview is renderer-owned, not egui-painted.
2. Replace projected-bounds picking with renderer/object-ID picking. The shared render contract now carries object-keyed index ranges and bounds, and `engine_render_gl` now has an offscreen object-ID framebuffer/pick shader API. The remaining work is wiring that API into the embedded editor viewport once the egui surface handoff exists.
3. Feed voxel panel composition exports into the shared `VoxelSceneRenderData` path.
4. Add selection/highlight IDs and transform gizmos to the voxel render contract.
5. Expand runtime voxel rendering from scene preview objects into gameplay-aware voxel object instances while preserving the current 2D map path.

### Stubbed / placeholder systems still visible

- `crates/editor_inspector` and `crates/editor_tools` are still effectively init-only/stub crates; `crates/editor_data_bridge` now owns the first shared path/load/save/backup/dirty-state helpers but still needs broader editor and web-editor adoption.
- `content/editor_tools/dynamic_voxelizer_commands.ron` points at an offline placeholder script.
- `content/editor_tools/phase53f_voxel_rig_preview_commands.ron` points at a placeholder rig preview script.
- Dynamic voxelizer, voxel rig, spring bone, and attachment systems are contracts/specs first; implementation is still pending.
- Some voxel/scene/map content is intentionally placeholder:
  - `content/scenes/starter_farm/voxel_terrain.ron`
  - `content/maps/starter_farm/voxel_objects.ron`
  - `content/voxels/phase52/vox_asset_catalog.ron`
  - `content/maps/starter_farm/props.ron` still has `farmhouse_placeholder`
- Several broader game crates are still stubs, which affects editor playtest/runtime panels: entities, items, inventory, farming, combat, NPC, quests, dialogue, economy, save, UI.

---

## Finish Roadmap

### Phase U1 - Interface Consolidation

Goal: make the editor shell intuitive before adding more feature surface.

- [x] Replace the always-visible global tool row with workspace-local toolbars.
- [x] Add one context-aware command strip for Save, Save All Dirty, Reload, Undo, Redo, and Validate.
- [x] Remove the most duplicated save/reload/undo controls from map, pixel, placement, and voxel child panels once the command strip owns them.
- [x] Convert text-only subtab labels for Project, Animation, Character, Data, Playtest, and Settings into real enum-backed child tabs.
- [x] Add a first selection-aware right inspector for tile, layer, prop, spawn, and trigger selections.
- [x] Extend the selection-aware inspector to voxel object selections.
- [x] Extend the selection-aware inspector to pixel selection.
- [x] Extend the selection-aware inspector to voxel panel selection.
- [x] Extend the selection-aware inspector to runtime scene voxel object selections from World -> 3D Preview.
- [x] Merge props, spawns, and triggers into a World Objects parent panel with type filters.
- [x] Extend World Objects to include voxel object placement records.
- Track the full shell audit in `docs/EDITOR_UX_AUDIT.md`.

Done when: the active workspace owns its tools, the right inspector follows the selected thing, and save/reload/undo controls are not repeated across unrelated panels.

### Phase E1 - Editor Command/Undo Foundation

Goal: every editor mutation goes through a shared command model.

- [x] Implement `editor_undo` as a reusable command/snapshot stack with labels, undo, redo, limits, and unit tests.
- [x] Wire native map layer paint, erase, fill, layer legend additions, and layer visibility changes into the shared undo stack.
- [x] Add coalescing for native map drag/paint strokes so long brush drags become one undo step.
- [ ] Move pixel edits, voxel panel edits, voxel object edits, and metadata edits onto the shared command model.
- Add common save-state tracking: clean, dirty, saving, failed.
- Add consistent backup naming and recent-backup display.
- [x] Add unit tests for shared undo/redo stack behavior.

Done when: `Ctrl+Z`/`Ctrl+Y` works consistently across map, pixel, voxel panel, voxel object, and metadata editing.

### Phase E2 - Inspector/Data Bridge Realization

Goal: stop keeping editor logic trapped in the giant egui app file.

- [x] Start `editor_data_bridge` as the shared read/write layer for content assets with centralized map content paths, map/layer/placement load wrappers, backup save outcomes, dirty-state models, temp-write saves, and unit coverage.
- Extend `editor_data_bridge` adoption across the remaining direct save/load paths, including web-editor adoption.
- Implement `editor_inspector` as reusable typed inspector models for tiles, layers, props, spawns, triggers, voxel objects, and data records.
- Keep egui rendering thin: panels should call bridge/inspector APIs instead of hand-editing raw data everywhere.
- [x] Add validation messages with stable IDs and source file paths.

Done when: native and web editors can use the same bridge for map/layer/object edits.

### Phase E3 - Native World Editing Complete

Goal: the native World workspace can fully author maps.

- [x] Add native layer add, remove, duplicate, reorder, and visibility persistence through `layers.ron`.
- [x] Add explicit lock/opacity schema fields and persist them once the layer contract supports them.
- [x] Add trigger zone CRUD/save editing through the native inspector for the active map `triggers.ron`.
- [x] Add spawn point CRUD/save editing through the native inspector for the active map `spawns.ron`.
- [x] Add prop/object CRUD/save editing for the current 2D runtime path through the native asset props panel.
- [x] Add native World preview overlays and click selection for props, spawns, and triggers.
- [x] Add viewport delete, duplicate, arrow-key nudge, and grid-snapped coordinate edits for selected props, spawns, and triggers.
- [x] Add native World preview overlays, click selection, and grid-snapped drag movement for voxel objects.
- [x] Add grid-snapped drag movement for selected props, spawns, and triggers.
- [x] Add viewport marquee select for world placements.
- [x] Add explicit resize handles for trigger and voxel object footprints.
- Add collision overlay editing or collision-source inspection.
- Add map resize/crop/expand tools with safeguards.
- Add multi-select transform operations and richer point-marker handle affordances for props and spawns.
- [x] Add per-map validation for missing tile IDs, bad layer dimensions, duplicate IDs, invalid trigger target maps, placement bounds, missing voxel object sources, and scene voxel source health.

Done when: a map can be created or substantially edited from the native editor without manually touching RON.

### Phase E4 - Data Editors Complete

Goal: content records are edited in the editor, not by hand.

- Build CRUD editors for items, crops, NPCs, dialogue, quests, shops, and schedules.
- Use schema-aware defaults and validation.
- Add cross-reference pickers instead of free-text IDs where practical.
- Add safe rename/refactor support for IDs referenced by other content.
- Add import/export and duplicate actions.

Done when: the vertical-slice content set can be authored without opening `content/*.ron`.

### Phase E5 - Animation and Character Workspaces

Goal: character and animation assets become production tools.

- Implement animation clip list, timeline, direction sets, event tracks, socket/hitbox markers, and preview playback.
- Make animation event validation real: footstep, SFX, hitbox, tool attach, and interaction events.
- Upgrade Character workspace from paperdoll/profile panels to real layer stack editing, equipment overlays, palette swaps, 8-direction preview, and export validation.
- Connect character editor outputs to runtime sprite/voxel metadata.

Done when: a character/NPC animation set can be assembled, previewed, validated, and saved from the editor.

### Phase E6 - Voxel Toolchain Completion

Goal: voxel workflows move from contracts and 2D authoring to usable 3D production preview.

- Add renderer-owned `VoxelMeshRenderData` and `VoxelMeshInstance` contracts.
- Add an OpenGL voxel mesh pipeline with camera matrices, depth testing, simple lighting, bounds/debug grid, and selection IDs.
- Replace dynamic voxelizer placeholder scripts with real local command runners.
- Add generator output validation: missing outputs, bad dimensions, wrong materials, bad naming, and density limits.
- Add reference compare panels for generated templates.
- Add actual 3D viewport for voxel panels and compositions.
- Add panel composition mesh preview consumption from exported composition data.
- Add world voxel object viewport picking and drag-to-move handles.
- Add duplicate, snap-to-grid/free-move, footprint collision checks, and source existence checks.
- Add Voxel Rig Preview 3D overlay: bones, attachment anchors, pose selection, base/tool overlay, and validation.

Done when: voxel assets can be generated, inspected, composed, placed, and previewed in-editor without relying on placeholder scripts.

### Phase E7 - Logic Workspace

Goal: events and interactions are authored visually and validated.

- Implement graph node model and save format.
- Add event binding editor for triggers, map transitions, NPC interactions, quests, shops, and cutscenes.
- Add payload validation against runtime-supported actions.
- Add preview/simulate action for simple graph paths.

Done when: simple interactable objects and quest/dialogue triggers can be created through the editor and consumed by runtime.

### Phase E8 - Playtest and Runtime Bridge

Goal: the editor can launch and inspect the game state.

- Wire playtest launch/reload to the active map.
- Show live player position, facing, collision tile, inventory, current quest state, and current interaction target.
- Add camera controls and collision debug toggles.
- Add reset-to-spawn and teleport-to-selected-spawn actions.
- Add runtime log/error surfacing with clickable source content references.

Done when: editor changes can be saved, launched, and verified without leaving the editor loop.

### Phase E9 - Web Editor Parity Pass

Goal: web editor remains useful as a companion, not a fork.

- Add undo/redo.
- Add marquee selection and object/layer transforms.
- Add collision/spawn/trigger editing through the shared data bridge.
- Add visual validation report.
- Add atlas compare/import preview only if it can share native editor validation.
- Add conflict detection for native and web simultaneous edits.

Done when: web edits are safe, reversible, and validated against the same rules as native edits.

### Phase E10 - Editor Hardening and Release Gate

Goal: make it dependable enough to use daily.

- Add automated tests for loaders, savers, validators, command undo/redo, and map transforms.
- Add golden content validation fixtures.
- Add crash-safe save behavior: temp write, atomic replace where possible, backup before overwrite.
- Add dirty-file exit prompts.
- Add searchable command palette.
- Add keybind persistence.
- Add editor preferences persistence.
- Add release smoke checklist and diagnostic bundle export.

Done when: `cargo test`, content validation, native editor smoke test, and web editor smoke test are release blockers.

---

## Suggested Immediate Next Patch Order

1. Expand `editor_data_bridge` adoption across remaining direct save/load paths and add recent-backup display in the UI.
2. Move pixel editor and voxel panel editor histories onto shared `editor_undo`.
3. Replace voxel rig/dynamic voxelizer placeholder command runners with real command execution and validation.
4. Build the first real Data editor: items first, then crops, NPCs, dialogue, quests, shops.

This order gives the editor a reliable spine before adding more surface area.
