# Phase 52b.5 — Release Warning Cleanup

This cleanup follows the successful release-all-app-binaries build after the Phase 52b.4 VOX module recovery.

## Build log result

The latest release-all app binaries run completed with exit code 0. The remaining compiler output was warning-only in `crates/app/src/egui_editor.rs`.

## Cleanup applied

- Removed the dead `open_web_asset_lab` helper and its `std::process::Command` import.
  - The egui editor now stays on the native Asset Studio path instead of routing back to the old web Asset Lab helper.
- Reused `EditorMapState::map_id` in the fixed bottom status bar so the loaded editable map state is surfaced directly.
- Replaced the unused `is_likely_transition(TileInstance)` helper with `is_likely_transition_tile_id(&str)` and wired it into the transition overlay visibility filter.

## Expected result

`cargo check` and release app binary builds should remain green, with the previous three egui dead-code warnings removed.

