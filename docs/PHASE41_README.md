# Starlight Ridge Phase 41 — egui Compile + Layout Stabilization

Drop this zip over the project root and overwrite existing files.

## Changes

- Fixes the broken duplicate `draw_top_bar` declaration in `crates/app/src/egui_editor.rs`.
- Updates the egui app implementation to use `eframe::App::update(...)`.
- Adds missing `editor_core` modules:
  - `crates/editor_core/src/atlas_pipeline.rs`
  - `crates/editor_core/src/export_pipeline.rs`
- Replaces deprecated egui panel aliases with `egui::Panel::*`.
- Forces a consistent dark egui visual theme.
- Makes the bottom Console / Validation / Hot Reload / Runtime area fixed-height.
- Adds a static status bar below the bottom console panel.
- Routes the center workspace by top-level tab instead of always showing the same world preview.
- Adds scaffold center views for Project, World, Assets, Animation, Character, Logic, Data, Playtest, and Settings.

## Expected result

- `cargo check` should get past the missing editor_core module errors and the broken egui app trait shape.
- The bottom console tabs should no longer resize the whole editor when clicked.
- The editor should default to a consistent dark UI.
- Top tabs should change the center workspace instead of appearing clickable but static.

## Still intentionally left for Phase 42+

- Real atlas PNG rendering in egui.
- Real map painting/saving.
- Side-by-side atlas import/compare workflow.
- Pixel editor tools.
- Runtime launch/log integration.
