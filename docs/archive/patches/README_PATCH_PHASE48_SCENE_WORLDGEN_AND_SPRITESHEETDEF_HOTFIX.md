# Patch Phase 48 — Scene WorldGen + SpriteSheetDef Launch Hotfix

## Files changed

- `Cargo.toml`
- `crates/game_data/src/lib.rs`
- `crates/game_worldgen/Cargo.toml`
- `crates/game_worldgen/src/lib.rs`
- `content/worldgen/scene_registry_phase48.ron`
- `content/worldgen/scene_templates_phase48.ron`
- `content/worldgen/semantic_terrain_ids_phase48.ron`
- `content/editor_worldgen/generated_scene_workflow_phase48.ron`
- `docs/worldgen/PHASE48_SCENE_BASED_WORLDGEN_EDITABLE_SCENES.md`
- `docs/roadmap/CHAT_ROLLUP_PHASE48.md`

## What it fixes

Both `app.exe` and `editor.exe` were crashing because the content registry parsed every `.ron` in `content/metadata` as `SpriteSheetDef`. Character metadata such as `character_bases_phase27.ron` is not a sprite sheet definition and does not have the required `id` field at the top level.

The loader now skips metadata files that do not look like sprite sheet metadata.

## What it adds

First source-level scene-based world generation scaffold for editable generated scenes.

## Verify

Run:

```bat
BUILD_MENU.bat
```

Then:

```text
2) cargo check
4) cargo build --release game exe
15) Build editor release exe
13) Run game release exe
14) Run editor release exe
```
