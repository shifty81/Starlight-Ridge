# Phase 53f_1 — eframe App Trait Entry Hotfix

## Purpose

The Phase 53f source introduced the Voxel Pixel Panel Designer foundation, but the local build resolved `eframe`/`egui` 0.34.1 and reported that `eframe::App` still requires `ui(&mut self, &mut egui::Ui, &mut eframe::Frame)`.

## Fix

`crates/app/src/egui_editor.rs` now implements:

```rust
impl eframe::App for StarlightRidgeEguiEditor {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // single-root editor shell render
    }
}
```

The full editor shell remains rooted in the trait entrypoint and still uses `shell_render_depth` to prevent nested-editor regressions. Workspace tabs must not call full-shell rendering functions.

## Scope

Included:

- Restores the correct `eframe::App::ui` trait implementation for this dependency set.
- Keeps the Voxel Panel Designer work from Phase 53f intact.
- Keeps the single-root shell guard intact.

Not included:

- No gameplay changes.
- No 3D voxel viewport changes.
- No deprecated `Panel::show` cleanup beyond this compile blocker.

## Expected build result

The previous `E0046` missing `ui` trait item should be resolved. The remaining `Panel::show` warnings may still appear and can be cleaned in a later warning-only pass.
