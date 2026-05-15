# Roadmap

This document is the single canonical roadmap for Starlight Ridge. It supersedes phase-by-phase chat rollups and historical patch notes in `docs/archive/`.

The guiding principle is **playable first**: get a real game loop running before adding worldgen, voxels, or simulation layers.
Current execution emphasis is **editor/world authoring reliability first** so the playable-first milestones can be built on stable tools.

For the detailed editor-only completion checklist, see [EDITOR_COMPLETION_ROADMAP.md](EDITOR_COMPLETION_ROADMAP.md).

---

## Current State (as of May 2026)

✅ **Done**
- Custom GL engine (window, tilemap renderer, sprite pipeline)
- RON-driven content pipeline and registry
- Semantic autotile resolver
- Native egui editor shell (tab structure, tile inspector, atlas tools, content reload)
- Web LAN editor (paint, erase, fill, clipboard, paste transforms, brush settings)
- 4 maps (`starter_farm`, `town`, `autotile_test_coast`, `autotile_test_pond`)
- Content schemas for items, crops, NPCs, dialogue, quests, shops, terrain, biomes, seasons, weather, liquids, voxels

⚠️ **Partial / stubs**
- Player visible on screen (sprite pipeline exists, not wired to spawn)
- Editor tile painting (UI exists, save path incomplete)
- Autotile resolver (conservative cardinal rules, no blob/47-tile)
- Audio (bootstrap stub, no playback)

❌ **Not started**
- Any gameplay loop (farming, combat, NPC AI, dialogue, economy, inventory, save)
- Tests of any kind

---

## Milestone 1 — Player on Screen *(next)*

Goal: a character that appears, moves, and collides.

- [ ] Spawn player from `spawns.ron` at startup
- [ ] Pass player `SpriteInstance` into `render_frame`
- [ ] Camera follows player
- [ ] Fixed-timestep accumulator in `engine_time`
- [ ] `InputMap` / `Action` enum in `engine_input` (replaces raw bool snapshot)
- [ ] 8-direction normalised movement with facing state
- [ ] Terrain collision from `TerrainFlags`
- [ ] Interaction probe (E/Space scans nearby props and triggers)
- [ ] Debug overlay: player tile position, facing, collision grid

---

## Milestone 2 — Minimal UI

Goal: the game can show text on screen.

- [ ] `game_ui`: dialogue box (speaker name + text lines + choice list)
- [ ] HUD: hotbar strip, health bar placeholder
- [ ] Simple fade-in/out for map transitions
- [ ] Interaction prompt sprite (visible when near interactable)

---

## Milestone 3 — Farming Loop

Goal: plant → water → harvest one crop.

- [ ] `game_items` + `game_inventory`: item stack, hotbar slot
- [ ] `game_farming`: till tile, plant seed, water, grow tick per in-game day, harvest
- [ ] Tool use (hoe, watering can) tied to interaction key
- [ ] Tilled / watered tile state updates `SemanticTerrainGrid` and re-renders

---

## Milestone 4 — NPC + Dialogue

Goal: one NPC walks a schedule and has a conversation.

- [ ] `game_npc`: NPC entity with position, facing, schedule-driven waypoints
- [ ] `game_dialogue`: parse dialogue tree, display via `game_ui` dialogue box, handle choices
- [ ] NPC sprite rendered + Y-sorted with player
- [ ] Basic schedule tick (move to next waypoint at given in-game time)

---

## Milestone 5 — Economy + Quest

Goal: buy seeds, complete a simple fetch quest.

- [ ] `game_economy`: shop inventory, buy/sell transactions, currency item
- [ ] `game_quests`: quest state (not started / active / complete), objective tracking
- [ ] Quest journal UI panel (simple list)
- [ ] Reward delivery on quest complete

---

## Milestone 6 — Save / Load

Goal: progress persists between sessions.

- [ ] `game_save`: serialise player position, inventory, farm state, quest state, in-game day
- [ ] Load on startup (newest save), save on sleep / quit
- [ ] Save slot selector on title screen

---

## Milestone 7 — Audio

Goal: the game makes sounds.

- [ ] `engine_audio`: sound asset cache via `rodio` (already in workspace dependencies)
- [ ] Footstep material lookup from terrain type
- [ ] Tool SFX hooks (hoe, watering can)
- [ ] Background music per map (`music` field in `map.ron` already defined)
- [ ] Day/night ambient light curve (3-hour day clock)

---

## Milestone 8 — Editor Tile Painting

Goal: the native editor can paint and save maps.

- [ ] Tile painting save path (write `layers.ron` from egui editor)
- [ ] Layer add / remove / reorder / visibility controls
- [ ] `editor_undo` real implementation (command stack, undo/redo)
- [ ] Data editors for items, crops, NPCs, dialogue, quests, shops

---

## Milestone 9 — Combat (vertical slice)

Goal: one enemy type, basic melee combat.

- [ ] `game_combat`: health component, melee hit, knockback, death
- [ ] Enemy entity with simple patrol AI
- [ ] Hit animation / flash
- [ ] Player health bar active

---

## Milestone 10 — Vertical Slice Release

Goal: a shareable build covering town + farm + forest/cave, one crop, one NPC, one merchant, one enemy, one quest, save/load.

- [ ] All Milestone 1–9 items complete
- [ ] Smoke-test suite (content validation, tile count assertions)
- [ ] `cargo test` passes with at least basic unit coverage
- [ ] Itch.io / GitHub release build

---

## Future: Worldgen + Simulation (Phase 52+)

These phases are planned but deliberately deferred until the vertical slice is playable.
If old phase docs conflict with this roadmap ordering, follow this document.

### Phase 52a — Core Contracts *(schemas done, loading/validation pending)*
- RON loading for biome, material, liquid, weather, season, vox contracts
- Registry validation + editor validation panel

### Phase 52b — Layered World Model
- 10 visible editor layers + 4 derived/runtime layers
- Cell stack inspector, layer visibility/lock/opacity

### Phase 52c — VOX Asset Pipeline
- `.vox` registry, browser, bake profile, 4-facing bake scaffold

### Phase 52d — Biome / Material Atlas System
- 12 biome definitions, 5 grass / 5 sand / 5 water material families, lava, crude oil
- Material thumbnails and validation

### Phase 52e — WorldGen Core
- Elevation / moisture / temperature noise fields
- Biome assignment, hydrology routing, material selection
- Draft → bake workflow

### Phase 52f — Liquids + Weather Simulation
- Shallow liquid depth/flow, rain wetness, puddles
- Snow accumulation, melt/freeze/slush cycle

### Phase 52g — Orthographic Orbit Camera
- 4-step camera rotation, directional sprite selection
- Rotated selection / footprint preview

### Phase 52h — Editor Wiki / Help
- Markdown help panel, hover tooltips, `?` context buttons
- Validation messages with help links

### Phase 52i — Mobile LAN Editor
- Touch-first pixel editor and layer tools
- PWA installable metadata

---

## Technical Debt / Known Gaps

| Item | Notes |
|---|---|
| Blob/47-tile autotile resolver | Current resolver is conservative, no concave corners |
| Atlas transition roles not fully data-driven | Named roles exist but resolver still has some hardcoded fallbacks |
| No tests | Zero tests in workspace; visual regressions caught manually |
| `render_frame` called with empty sprite arrays | Player sprite never populated at runtime |
| No fixed timestep | `engine_time` is delta-only; no update/render separation |
| `InputMap` missing | No rebindable keys, no `just_pressed`/`held` distinction |
| Audio is a stub | `rodio` is in dependencies; needs wiring |
