# Phase 38 — egui Rollup Source Fix

## Purpose

This patch consolidates the latest Starlight Ridge editor direction into a cleaner source snapshot.

## Included fixes

- Fixes the `eframe::App` implementation for the newer egui/eframe API by implementing `ui(...)` instead of relying on the deprecated `update(...)` entrypoint.
- Restores the `editor_core::init_with_registry(...)` helper and its `game_data` dependency so the egui editor startup can compile.
- Keeps the Phase 37 runtime content loader fix that skips editor-only tileset sidecars such as `base_tileset_roles.ron`.
- Moves the web Asset Lab Color / RGBA and Display Options controls from the right preview panel into a left-side options rail between the tool rail and pixel canvas.
- Keeps the templates section removed and reserves that area for atlas compare/import queue work.
- Sorts historical patch documentation into `docs/patches/`.
- Adds an updated chat/project rollup at `docs/roadmap/CHAT_ROLLUP_PHASE38.md`.

## Notes

Cargo is unavailable in the packaging container, so this source was source-checked only. Run the project build menu locally and upload `logs/latest.log` if any new compile error appears.

## Expected result

- `cargo check` should move past the missing `init_with_registry` and missing `eframe::App::ui` errors.
- The browser Asset Lab should show color sliders and display toggles in the left-side blue options rail instead of the right preview/properties rail.
- Root `docs/` should be less cluttered, with patch notes under `docs/patches/`.
