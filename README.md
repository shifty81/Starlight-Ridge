# Starlight Forge Phase-1 Bootstrap

Rust top-down adventure life-sim starter workspace with a built-in editor architecture.

## What this phase does

This bootstrap is no longer a pure stub pack. It now provides:

- a compileable Cargo workspace
- a minimal `winit` windowed app loop
- frame timing and redraw flow
- a renderer bootstrap placeholder where the OpenGL context should be wired next
- a real `game_data` loader for the sample `.ron` content
- content validation for basic cross-references

## Current phase boundary

Included now:
- project root discovery
- content loading from `content/`
- sample registry validation
- a live app window
- redraw loop
- Escape and close-to-exit handling

Still intentionally deferred to the next milestone:
- real OpenGL context creation
- sprite/tile rendering
- camera and player movement
- gameplay simulation
- editor overlay UI

## Build

```bash
cargo check
cargo run -p app
```

## Expected result

On launch, the app should:
1. load the sample RON files
2. validate the loaded registry
3. open a window titled `Starlight Forge - Phase 1 Bootstrap`
4. enter a redraw loop until the window is closed or Escape is pressed

## Recommended next implementation order

1. wire an actual OpenGL context into `engine_render_gl`
2. add a simple clear pass and viewport handling
3. load one tileset texture
4. draw the `town` map metadata as the first rendered scene root
5. add a camera and player bootstrap entity
6. route input into gameplay movement
7. add the first editor toggle state


## Phase 5 Starter Art Assets

This scaffold now includes generated prototype art assets for the tilemap + sprite pass:

- `assets/textures/terrain_atlas_phase5.png`
- `assets/textures/entity_sprite_sheet_phase5.png`
- `content/tiles/base_tileset.ron`
- `content/metadata/entity_sprite_sheet_phase5.ron`

Suggested next implementation step:
- load the tileset atlas and sprite sheet in `engine_assets`
- add atlas-region lookup from the RON metadata
- render world tiles from a tile layer instead of generated debug colors
- render a first player sprite / marker on top of the map


## Phase 6 In-Game Tilemap + Sprite Integration

This pass wires the generated starter art into the runtime path.

### Added / updated
- `content/maps/town/layers.ron`
- `content/tiles/base_tileset.ron`
- `content/metadata/entity_sprite_sheet_phase5.ron`
- `crates/game_data/*` tile/sprite metadata loading
- `crates/engine_render_gl/src/lib.rs` texture atlas + sprite sheet rendering
- `crates/app/src/lib.rs` camera movement and phase-6 boot flow

### Expected result
Running `cargo run -p app` should now:
- open a GL window
- load the generated terrain atlas and entity sprite sheet
- render the `town` tile layer from `layers.ron`
- render player/NPC markers and prop sprites on top
- allow camera pan with `WASD` / arrow keys

### Note
This is still a prototype render path and was prepared without being compiled in this container.

## Phase 21 Web Editor LAN Bridge

A first LAN-accessible web editor surface is available for tablet testing.

- Read-only tablet mode: `RUN_WEB_EDITOR_LAN.bat`
- Tablet mode with `layers.ron` saving enabled: `RUN_WEB_EDITOR_LAN_WRITE.bat`
- Bash menu commands: `./build.sh web-editor` or `./build.sh web-editor-write`
- Windows menu options are also available through `build.bat` / `tools/build_menu.ps1`.

The server prints a local network URL such as `http://192.168.1.25:8787/`. Open that URL from a tablet on the same network. See `docs/editor/web_editor_lan_phase21.md` for details and firewall notes.
