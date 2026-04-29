# Phase 52b.3 — Worldgen Loader Function Recovery

## Purpose

This hotfix restores the missing game_data loader entry points that `load_registry` calls for the Phase 51/52 worldgen architecture contracts.

The failed `cargo check` reported missing functions in `crates/game_data/src/lib.rs` for:

- `load_world_manifest`
- `load_protected_layer_policy`
- `load_generated_scene_draft`
- `load_scene_bake_contract`
- `load_worldgen_editor_workflow`

## Changed file

- `crates/game_data/src/loader.rs`

## Fix

`loader.rs` now imports the matching shared worldgen contract aliases from `defs.rs` and exposes the five thin RON loader wrappers using the existing `load_ron_file<T>()` helper.

This keeps the worldgen loader path consistent with the rest of the content registry loader system and avoids adding one-off parsing logic.

## Notes

This source also retains the prior Phase 52b.2 egui single-root shell fix for the nested-editor UI regression.

## Test target

Run:

```bat
build.bat
```

Then select:

```text
2) cargo check
```

After that, run:

```text
6) Run editor debug
```
