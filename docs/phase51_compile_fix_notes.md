# Starlight Ridge Phase 51 Compile Fix

Fixes the cargo check failure introduced by the Phase 51 egui editor patch.

## Error fixed

```text
error[E0046]: not all trait items implemented, missing: `ui`
--> crates\app\src\egui_editor.rs
impl eframe::App for StarlightRidgeEguiEditor
```

## Change

`StarlightRidgeEguiEditor` now exposes a shared `draw_editor_frame(&egui::Context)` method and implements both:

- `update(&mut self, &egui::Context, &mut eframe::Frame)`
- `ui(&mut self, &mut egui::Ui, &mut eframe::Frame)`

This keeps the existing editor rendering path intact while satisfying the `eframe::App` trait expected by the currently compiled `eframe` version.

## Files changed

- `crates/app/src/egui_editor.rs`

