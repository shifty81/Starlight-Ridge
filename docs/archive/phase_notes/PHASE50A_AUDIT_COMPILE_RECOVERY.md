# Starlight Ridge Phase 50A Audit — Compile Recovery + Implementation Gap Map

## Current blocking build issues found

1. `crates/editor_core/src/lib.rs` declares:
   - `pub mod atlas_pipeline;`
   - `pub mod export_pipeline;`

   But `crates/editor_core/src/atlas_pipeline.rs` and `crates/editor_core/src/export_pipeline.rs` were missing from the uploaded source. This is a direct compile blocker.

2. `crates/app/src/egui_editor.rs` had a duplicate/incomplete function stub:

   ```rust
   fn draw_top_bar(&mut self, ctx: &egui::Context) {

   fn draw_left_panel(...)
   ```

   That leaves the impl body structurally broken and can cause parser/unclosed delimiter errors.

3. `crates/app/src/egui_editor.rs` implemented `eframe::App` with:

   ```rust
   fn ui(&mut self, ui: &mut egui::Ui, ...)
   ```

   For the current `eframe` app path, this must be `update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)`.

## Files changed in this recovery patch

- `crates/app/src/egui_editor.rs`
  - Removed duplicate incomplete `draw_top_bar` stub.
  - Replaced the invalid `eframe::App::ui` implementation with `eframe::App::update`.

- `crates/editor_core/src/atlas_pipeline.rs`
  - Added missing Phase 19 atlas pipeline report module.

- `crates/editor_core/src/export_pipeline.rs`
  - Added missing Phase 20 export/validation pipeline report module.

## What I could verify statically

- Map data parsed cleanly:
  - `starter_farm`: 40x28, 2 layers, no missing tile refs found.
  - `town`: 32x24, 1 layer, no missing tile refs found.
  - `autotile_test_coast`: 48x30, 1 layer, no missing tile refs found.
  - `autotile_test_pond`: 32x24, 1 layer, no missing tile refs found.

- Tileset data parsed cleanly:
  - `base_tileset.ron`: 191 named tiles, texture exists.
  - Seasonal tilesets exist for spring, summer, autumn, winter, each with 112 named tiles.
  - Texture paths referenced by metadata exist.

- Web editor JavaScript syntax passed a `node --check` syntax check.

## What I could not verify here

The execution container does not have `cargo` installed, so I could not run `cargo check` locally. Run:

```bash
cargo check
```

or use the root build script after applying this patch. If another Rust error appears, it is likely the next real compile blocker after the parser/module issues fixed here.

---

# Implementation state audit

## Game/runtime status

Current runtime is still mostly a renderer/bootstrap path.

Important observations:

- `engine_input` is still a direct key snapshot:
  - WASD/arrows movement booleans
  - interact pressed
  - no action enum
  - no rebindable `InputMap`
  - no sprint action
  - no just-pressed/held abstraction beyond raw fields

- `engine_time` is still frame delta only:
  - no fixed timestep accumulator
  - no interpolation alpha
  - no update/render separation

- `app` render call currently passes empty sprite arrays:
  - `renderer.render_frame(stats.frame_index, &[], &[])`
  - player sprite and prop sprite rendering are structurally supported in renderer, but not wired to gameplay state.

- There is no real player simulation loop yet:
  - no player position update
  - no normalized 8-direction movement
  - no sprint
  - no facing direction
  - no collision resolution
  - no interaction probe
  - no player state machine
  - no vault state

- Map props/spawns/triggers exist in content, but runtime does not yet consume them as actual gameplay entities.

- Audio is only a bootstrap stub:
  - no sound asset cache
  - no material footsteps
  - no vault/tool sound events

- Save/game persistence is scaffold-only:
  - no seed/player/chunk/save-state pipeline

## Renderer status

Already present:

- GL window bootstrap.
- Tile map renderer.
- Sprite pipeline structure.
- Prop sprite pipeline structure.
- Editor overlay renderer for legacy GL editor.
- Tile render data builder with semantic terrain and transition resolver.

Missing or incomplete:

- Active player sprite instances.
- Active prop sprite instances.
- Y-sort/depth ordering for top-down occlusion.
- Collision debug overlay tied to actual collision data.
- Camera follow/zoom behavior for gameplay.
- Lighting/day-night overlay.
- Animated water frame selection.
- Runtime sprite animation playback.

## World/autotile status

Already present:

- Semantic terrain grid.
- Terrain flags.
- Variant sets.
- Transition rules.
- Resolver emits base and transition render-layer tiles.
- Terrain IDs exist: grass, dirt, path, sand, shallow_water, deep_water, tilled_dry, tilled_watered.

Missing or incomplete:

- 8-way/blob/47-tile corner-aware resolver.
- Separate shoreline/path/tilled/watered overlay layer contract.
- Scene template generator.
- Seed storage.
- Generated draft scene state.
- Bake generated draft to editable map files.
- Protected layer rules.
- Editor-side autotile preview/validation controls.
- Seasonal parity enforcement between base and seasonal tilesets.
- Animated water metadata wired into render loop.

## Native egui editor status

Already present:

- Top-level workspace tabs:
  - Project
  - World
  - Assets
  - Animation
  - Character
  - Logic
  - Data
  - Playtest
  - Settings

- World subtabs:
  - Map Paint
  - Layers
  - Interactions
  - Spawns
  - Terrain Rules

- Asset subtabs:
  - Terrain Atlas
  - Atlas Compare / Import
  - Pixel Editor
  - Props / Objects
  - Seasons

- Logic subtabs:
  - Graphs
  - Event Bindings
  - Tools
  - Blocks / Tiles
  - Validation

- Working-ish pieces:
  - Content reload
  - Map switch
  - Tile list/filter
  - Tile inspector role/collision editing
  - Preview selection
  - Checkpoint manifest write
  - Basic status/log panels

Missing essentials:

- Actual map painting save path in egui.
- Real layer add/remove/reorder/visibility editing.
- Actual atlas image preview, not only color blocks.
- Atlas compare/import drag/drop.
- Clipboard tile buffer.
- Mirror-on-paste.
- Pixel editor drawing tools.
- Animation timeline UI.
- Sockets/hitboxes editor.
- Character sheet/animation preview.
- Logic graph editor.
- Data editors for items/crops/NPCs/dialogue/quests/shops.
- Playtest state panel tied to real player runtime.
- Settings for keybinds, movement tuning, lighting, audio, and editor preferences.
- Validation runner with clickable jump-to-source results.

## Web editor status

Already present:

- LAN server binds to `0.0.0.0`.
- Tablet/PC presentation modes.
- Map loading.
- Layer visibility.
- Palette based on selected layer legend.
- Paint/erase/eyedropper/inspect.
- Save endpoint for `layers.ron` when write mode is enabled.
- Raw map file viewer.

Missing essentials:

- Keyboard shortcuts.
- Brush settings parity with egui editor.
- Atlas preview/compare/import.
- Touch pan/paint mode separation strong enough for tablet workflow.
- Undo/redo.
- Flood fill.
- Selection/marquee tools.
- Collision/interactions/spawn editing.
- Autotile refresh/preview.
- Runtime movement/debug settings panel.
- Scene generator preview/bake controls.
- Visual validation report.
- Asset upload/import handling.

## Editor tabs still needed or should be promoted

Recommended actual editor tab structure from here:

### Project
- Overview
- Build
- Logs
- Validation
- Export/Package
- Diagnostics

### World
- Scene Map
- Layers
- Terrain Paint
- Autotile Rules
- Interactions
- Collision
- Spawns
- Triggers
- Scene Generator
- Bake/Export

### Assets
- Terrain Atlas
- Atlas Compare/Import
- Pixel Editor
- Props/Objects
- Seasons
- Animated Tiles
- Audio Materials

### Animation
- Clips
- Timeline
- Direction Sets
- Events
- Sockets
- Hitboxes
- Preview
- Seasonal Variants

### Character
- Base Body
- Clothing Layers
- Hair/Face
- Tools/Equipment
- 8-Direction Preview
- Export Sheet

### Logic
- Visual Graphs
- Event Bindings
- Tool Rules
- Tile Behaviors
- Interaction Scripts
- Validation

### Runtime
- Movement
- Input Bindings
- Player State
- Collision Debug
- Interaction Probe
- Vault Rules
- Camera
- Lighting
- Audio

### Data
- Items
- Crops
- NPCs
- Dialogue
- Quests
- Shops
- Schedules

### Settings
- Editor Preferences
- Theme
- Project Paths
- Web Companion
- Save/Backup
- Keybinds

---

# Recommended next patch order

## Phase 50A — Compile Recovery + Audit
This patch.

## Phase 50B — Runtime Feel Foundation
- Fixed timestep accumulator.
- `Action` enum.
- `InputMap` / `InputState`.
- WASD + arrows defaults.
- Sprint action.
- Normalized 8-direction movement helper.
- Facing direction.
- Interaction probe scaffold.
- Movement/debug egui panel.
- Matching web editor runtime/debug scaffold.

## Phase 51 — Player Runtime Wiring
- Player spawn from `spawns.ron`.
- Player sprite instance passed into `render_frame`.
- Movement updates player position.
- Camera follow.
- Basic collision from terrain flags.
- Interaction prompt/debug overlay.

## Phase 52 — Vault + Tile Interaction Metadata
- `vaultable` tile metadata.
- Fence interaction tags.
- Sprint + forward probe vault trigger.
- Landing validation.
- Vault state machine.
- Animation/audio hooks.

## Phase 53 — Scene Generator Preview + Bake
- Scene template registry.
- Seed input.
- Starter farm/coastal plot generator scaffold.
- Generate draft.
- Protected layer report.
- Bake to editable map files.

## Phase 54 — Layered Autotile Transition System
- Base layer + transition overlays.
- Shoreline/path/tilled/watered overlays.
- Corner-aware masks.
- Autotile debug overlay.
- Editor refresh/validate buttons.

## Phase 55 — Day/Night + Audio Material Pass
- 3-hour day clock.
- Ambient light curve.
- Night overlay.
- Footstep material lookup.
- Tool/vault SFX event hooks.
