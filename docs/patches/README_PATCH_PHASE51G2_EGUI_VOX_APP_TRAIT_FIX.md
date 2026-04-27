# Phase 51g2 — egui Vox Module + App Trait Fix

## Purpose

Fix the next `cargo check` errors after Phase 51g/51g1 integration.

## Build errors addressed

- `E0432`: `engine_assets::vox` was imported by the egui editor but the `vox.rs` module was not exported from `engine_assets/src/lib.rs`.
- `E0046`: `eframe::App` in `eframe 0.34.1` now requires `ui(&mut self, &mut egui::Ui, &mut eframe::Frame)` instead of relying on the older `update`-only implementation.

## Files changed

- `crates/engine_assets/src/lib.rs`
  - Exports the existing `vox` module with `pub mod vox;`.

- `crates/app/src/egui_editor.rs`
  - Replaces the old `eframe::App::update` implementation with `eframe::App::ui`.
  - Keeps the existing editor panel drawing flow intact by cloning `ui.ctx()` and passing it through the current panel functions.

## Notes

The deprecation warnings for `Panel::show`, `default_width`, `width_range`, and `exact_height` are not hard errors. They can be cleaned in a later egui API modernization pass after the editor is compiling cleanly again.
