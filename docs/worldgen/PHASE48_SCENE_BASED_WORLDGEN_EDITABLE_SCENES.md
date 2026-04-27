# Phase 48 — Scene-Based WorldGen + Editable Generated Scenes

## Purpose

This phase adds the first source-level scaffold for Starlight Ridge world generation.
The generator is intentionally scene-based instead of infinite-world based. Each generated map becomes an editable draft scene that can be opened, customized, and baked into normal project content.

## Main rule

World generation emits semantic terrain, not atlas coordinates.

```text
WorldGen -> semantic terrain IDs
Autotile -> resolved visual tile IDs
Renderer -> draw layers
Editor -> customize and bake
```

This prevents the generator from locking the project into one atlas layout and keeps the current atlas/metadata cleanup work compatible with future scenes.

## Added crate

`crates/game_worldgen`

The crate provides:

- `SceneKind`
- `SemanticTerrainId`
- `SceneGenRequest`
- `GeneratedScene`
- `SceneTemplate`
- `ProtectedLayerRules`
- `SceneGenerator`
- `StarterFarmGenerator`
- `CoastalPlotGenerator`
- `AutotileHandoff`
- scene validation report scaffolding

## Scene creation workflow

```text
Create New Scene
  -> choose scene type/template
  -> choose seed, size, and generation rules
  -> generate semantic terrain
  -> generate natural object markers/spawns
  -> run autotile resolver
  -> open generated draft in editor
  -> custom edit protected layers / gameplay objects
  -> bake to normal scene/map files
```

## Protected layers

Regeneratable layers:

- base terrain
- terrain variation
- natural objects
- water shape

Protected layers:

- buildings
- NPC spawns
- quest triggers
- doors
- scene exits
- logic bindings
- custom objects

The editor should never wipe protected layers during regeneration unless the user explicitly requests it.

## Runtime launch hotfix included

The runtime loader previously attempted to parse every file under `content/metadata` as a `SpriteSheetDef`. That caused both `app.exe` and `editor.exe` to fail on files such as `character_bases_phase27.ron`.

Phase 48 updates the metadata loader to inspect the file and only parse files that look like real sprite sheet metadata: files containing both `texture_path` and `entries`.

This allows editor-only character/mannequin metadata to stay in the project without crashing the game or editor.

## Next implementation step

Phase 49 should add editor-side preview controls:

- World > Scene Gen subtab
- seed input
- scene template selector
- semantic preview
- generate draft button
- bake generated draft button
- protected layer warning report
