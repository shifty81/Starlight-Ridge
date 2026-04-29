# Starlight Ridge Phase 50B — Compile + Runtime Content Recovery

This patch fixes the `cargo check` blocker from `2026-04-26_12-26-07_cargo_check.log` and the runtime startup blocker shown in the screenshot.

## Fixes

1. Restores the `eframe::App` implementation to the local trait shape expected by this project:

```rust
fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)
```

2. Fixes the content registry/runtime startup failure by changing the water preview reference from the missing `shallow_water_idle` id to the existing `shallow_water_loop_tiles` animation clip.

3. Carries forward the Phase 50A editor-core recovery files:

```text
crates/editor_core/src/atlas_pipeline.rs
crates/editor_core/src/export_pipeline.rs
```

## Files changed

```text
crates/app/src/egui_editor.rs
crates/editor_core/src/atlas_pipeline.rs
crates/editor_core/src/export_pipeline.rs
content/editor_animation/phase21_animation_editor_timeline_events.ron
docs/PHASE50B_COMPILE_RUNTIME_CONTENT_RECOVERY.md
```

After applying, run `cargo check`, then launch `editor.exe` and `app.exe` again.
