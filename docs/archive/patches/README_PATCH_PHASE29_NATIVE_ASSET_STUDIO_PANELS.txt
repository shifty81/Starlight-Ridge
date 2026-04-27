Starlight Ridge Phase 29 - Native Asset Studio Panels

This patch implements the first readable native editor panels over the live game viewport.

Replaces:
  crates/engine_render_gl/src/lib.rs
  crates/editor_core/src/lib.rs
  crates/editor_core/src/native_shell.rs
  crates/game_data/src/lib.rs
  RUN_EDITOR_DIAGNOSTIC.bat

Adds:
  content/editor/native_asset_studio_panels_phase29.ron
  docs/editor/NATIVE_ASSET_STUDIO_PANELS_PHASE29.md
  docs/PATCH_README_PHASE29_NATIVE_ASSET_STUDIO_PANELS.md

Main improvements:
  - Project/Asset/Workflow panels on the left.
  - Inspector/Selected Tile/Preview/Tools panels on the right.
  - Console/Validation/Hot Reload/Runtime Log panel cards at the bottom.
  - Icon-first toolbar remains, with tool descriptors for future hover tooltips.
  - A small native bitmap text overlay so panels are readable without a full GUI dependency.
  - Smaller bottom dock so it covers less of the viewport.
  - Diagnostic launcher clears stale runtime-failure logs before launching.
  - Includes the Phase 28 sidecar RON loader hotfix.

Apply over the project root and overwrite.
Then run cargo check, rebuild, and launch RUN_EDITOR_DIAGNOSTIC.bat.
