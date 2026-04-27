# Starlight Ridge Phase 29 — Native Asset Studio Panels

## What this patch does

- Replaces the unlabeled blue placeholder bars with readable native editor panels.
- Adds built-in bitmap text drawing to the OpenGL overlay.
- Keeps the game viewport centered and live.
- Reduces the bottom dock height so it blocks less of the map.
- Adds Project/Asset/Workflow panels on the left.
- Adds Inspector/Tile/Preview/Tools panels on the right.
- Adds Console/Validation/Hot Reload/Runtime Log cards on the bottom.
- Updates the native editor descriptor to Phase 29 panel and toolbar names.
- Keeps the Phase 28 content sidecar skip fix.
- Updates `RUN_EDITOR_DIAGNOSTIC.bat` so stale old runtime failure logs do not keep showing after a successful editor run.

## Files changed

- `crates/engine_render_gl/src/lib.rs`
- `crates/editor_core/src/lib.rs`
- `crates/editor_core/src/native_shell.rs`
- `crates/game_data/src/lib.rs`
- `RUN_EDITOR_DIAGNOSTIC.bat`

## Files added

- `content/editor/native_asset_studio_panels_phase29.ron`
- `docs/editor/NATIVE_ASSET_STUDIO_PANELS_PHASE29.md`
- `docs/PATCH_README_PHASE29_NATIVE_ASSET_STUDIO_PANELS.md`
- `README_PATCH_PHASE29_NATIVE_ASSET_STUDIO_PANELS.txt`

## After applying

Run:

```bat
cargo check
```

Then rebuild and run:

```bat
RUN_EDITOR_DIAGNOSTIC.bat
```

The editor should open with readable native panels around the live game viewport.

## Not included yet

- Mouse hit testing.
- Clickable panel buttons.
- Real selected-tile binding.
- Native pixel editing.
- Native character layer editing.

Those should be Phase 30+.
