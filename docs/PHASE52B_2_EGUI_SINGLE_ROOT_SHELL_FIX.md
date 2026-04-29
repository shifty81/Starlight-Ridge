# Phase 52b.2 — egui Single Root Shell Hotfix

## Problem

The editor could render a complete second copy of itself inside the main editor area. In screenshots this shows as a second top bar, left project panel, inspector, console, and status bar nested inside the first shell.

## Cause

eframe 0.34 routes the native app through `App::ui(&mut egui::Ui, &mut eframe::Frame)`. Calling root `Panel::show(ctx, ...)` from an `update(&Context, ...)` style path lets the complete shell be drawn into the root UI that eframe has already created, producing a nested editor.

## Fix

- Use `impl eframe::App::ui` as the only editor entry point.
- Remove the old `update(&Context, ...)` shell path.
- Draw the root shell with `Panel::show_inside(root_ui, ...)` and `CentralPanel::show_inside(root_ui, ...)`.
- Keep workspace content functions limited to `&mut egui::Ui`; only the root shell functions may create panels.
- Replace deprecated panel sizing calls:
  - `default_width` -> `default_size`
  - `width_range` -> `size_range`
  - `exact_height` -> `exact_size`

## Regression check

Before rebuilding, this file should not contain any of these in `crates/app/src/egui_editor.rs`:

```text
fn update(&mut self, ctx: &egui::Context
.show(ctx,
default_width(
width_range(
exact_height(
```

Expected editor result: one top bar, one left Project panel, one Inspector, one bottom console, and one static status bar at the bottom edge.
