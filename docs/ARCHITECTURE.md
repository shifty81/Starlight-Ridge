# Architecture

## Crate Groups

The workspace is split into four logical groups. Dependency only flows downward — game crates depend on engine crates, editor crates depend on game crates, and `app` ties everything together.

```
app  (binaries: game + editor)
 ├── editor_*  (editor runtime state, tools, inspector, undo, bridge)
 ├── game_*    (simulation, content, world, entities, UI)
 └── engine_*  (window, GL renderer, input, audio, math, time, assets)
         └── shared_types  (StableId, GridPos, ProjectError)
```

---

## Crate Reference

### Engine crates

| Crate | Status | Purpose |
|---|---|---|
| `engine_window` | ✅ | glutin + winit GL window creation |
| `engine_render_gl` | ✅ | OpenGL tilemap batch renderer + sprite pipeline |
| `engine_input` | ✅ | Keyboard snapshot (WASD, arrows, interact, escape) |
| `engine_math` | ✅ | `Camera2D`, `Mat4` helpers via glam |
| `engine_time` | ✅ | Frame delta timer, `FrameStats` |
| `engine_assets` | ✅ | Asset root discovery, `.vox` file scanner |
| `engine_audio` | stub | AudioBootstrap struct — no sound playback yet |
| `engine_debug` | stub | DebugOverlayState struct — enabled flag only |

### Game crates

| Crate | Status | Purpose |
|---|---|---|
| `game_data` | ✅ | RON loaders, `ContentRegistry`, all `*Def` structs |
| `game_world` | ✅ | `MapMetadata`, `PropPlacement`, semantic autotile resolver |
| `game_worldgen` | ✅ structs | `SceneKind`, Phase 52 contract types (biome, material, liquid, weather, season) — no execution yet |
| `game_core` | ✅ | `AppMode`, `InteractionMode`, `BootstrapState` |
| `game_entities` | stub | |
| `game_items` | stub | |
| `game_inventory` | stub | |
| `game_farming` | stub | |
| `game_combat` | stub | |
| `game_npc` | stub | |
| `game_quests` | stub | |
| `game_dialogue` | stub | |
| `game_economy` | stub | |
| `game_save` | stub | |
| `game_ui` | stub | |

### Editor crates

| Crate | Status | Purpose |
|---|---|---|
| `editor_core` | ✅ | Mode, selection, atlas/export/animation pipeline log reports |
| `editor_tools` | stub | |
| `editor_inspector` | stub | |
| `editor_undo` | stub | |
| `editor_data_bridge` | stub | |

### Other

| Crate | Status | Purpose |
|---|---|---|
| `shared_types` | ✅ | `StableId`, `GridPos`, `ProjectError` |
| `web_editor_server` | ✅ | Minimal HTTP server for the browser LAN editor |
| `app` | ✅ | Game loop, egui editor shell, logging, crash reporting |

---

## Data Flow

### Content loading (startup)

```
/content/
  maps/<id>/map.ron          → MapMetadata
              layers.ron     → MapLayersDef (tile layers)
              props.ron      → Vec<PropPlacement>
              spawns.ron     → Vec<SpawnPoint>
              triggers.ron   → Vec<TriggerZone>
  tiles/base_tileset.ron     → TilesetDef (191 named roles)
  tiles/season_*.ron         → seasonal TilesetDef variants
  metadata/sprite_sheets/    → SpriteSheetDef
  terrain/                   → TerrainTypeDef, BiomePackDef, TransitionSetDef
  items/                     → ItemDef
  crops/                     → CropDef
  npc/                       → NpcDef, ScheduleDef
  dialogue/                  → DialogueDef
  quests/                    → QuestDef
  shops/                     → ShopDef
        ↓
  game_data::ContentRegistry  (all defs in HashMaps keyed by string id)
```

### Render frame (per tick)

```
ContentRegistry
  → MapLayersDef
  → TilesetDef
  → game_world::SemanticTerrainGrid
  → game_world::AutotileResolver
  → Vec<TileInstance>
  → engine_render_gl::TileMapRenderData
  → OpenGL draw call

ContentRegistry + spawns.ron (not yet wired)
  → Vec<SpriteInstance>
  → engine_render_gl::SpriteRenderData
  → OpenGL draw call
```

---

## Launch Modes

```
cargo run -p app                    # Game window
cargo run -p app --bin editor       # Native egui editor (same window, edit mode on)
cargo run -p web_editor_server      # LAN HTTP server → browser editor on :8787
```

The game and editor share the same `app::run` / `app::run_editor` entry points and the same GL render loop. The `LaunchMode` enum (`Game` vs `Editor`) controls which overlay is active.

---

## Content Format

All content files use [RON](https://github.com/ron-rs/ron) (Rusty Object Notation). The workspace `Cargo.toml` pins `ron = "0.10"`.

Tile role names follow the pattern `<terrain>_<variant>` e.g. `grass_base`, `water_edge_n`, `path_corner_sw`, `tilled_watered_inner_ne`. The autotile resolver reads these named roles — it does not assume atlas column positions.

---

## Autotile Resolver

`game_world::autotile::AutotileResolver` converts a `SemanticTerrainGrid` into renderable `TileInstance` lists.

1. For each cell, read its `TerrainFlags` and compare against cardinal + diagonal neighbours.
2. Look up the terrain type's `TerrainVariantSet` to pick a base tile.
3. Apply `TerrainTransitionRule` entries to add edge/corner overlay tiles where terrain types border each other.
4. Emit `ResolvedTileKind::Base` + zero or more `ResolvedTileKind::Transition` instances.

The current resolver is **conservative** (cardinal checks, limited corner cases). A full 47-tile Wang/blob resolver is planned for Milestone 5.
