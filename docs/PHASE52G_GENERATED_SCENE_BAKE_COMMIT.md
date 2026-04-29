# Phase 52g — Generated Scene Bake Commit + Editor Handoff

## Goal

Phase 52g turns the generated scene dry-run path into a real editor workflow:

1. Generate a semantic scene preview from the loaded worldgen draft.
2. Validate the generated semantic terrain and object markers.
3. Preview changed/unchanged/skipped counts inside the egui World → Terrain Rules tab.
4. Back up `content/maps/<map_id>/layers.ron`.
5. Commit generated terrain into editable map layers.
6. Reload the content registry and editor preview after commit.

## Editor workflow

Open the egui editor and go to:

`World` → `Terrain Rules`

Use:

- `Generate / Refresh Preview` to build a preview report.
- `Backup + Commit to Editable Layers` to write the generated scene into `layers.ron`.
- `Reload Content` to manually reload after external edits.

The panel shows:

- active draft id
- active bake contract id
- target terrain layer
- target object layer
- editable target dimensions
- changed cell count
- unchanged cell count
- skipped protected cell count
- terrain family counts
- warnings
- backup path after commit

## Protection behavior

Phase 52g only writes the selected/generated terrain layer and the detected natural-object layer. Other layers remain untouched and are counted as protected/skipped for reporting. This preserves hand-authored objects, interactions, triggers, lighting, audio zones, logic bindings, decals, spawns, and exits.

## Dimension behavior

The current starter farm draft contract is larger than the existing editable starter farm map. Phase 52g uses the current editable layer stack dimensions for the generated preview/commit, then reports the contract-size mismatch as a warning. This keeps the commit safe for the existing map while leaving the larger future scene contract intact.

## Files changed

- `crates/app/Cargo.toml`
- `crates/app/src/egui_editor.rs`
- `crates/game_data/src/loader.rs`
- `docs/PHASE52G_GENERATED_SCENE_BAKE_COMMIT.md`

## Next recommended phase

Phase 52h — WorldGen Brush + Rule Override Editor

Focus:

- paint worldgen override masks
- lock cells or areas against future generation
- brush biome/terrain family hints
- save override masks beside map layers
- preview exact override impact before bake
