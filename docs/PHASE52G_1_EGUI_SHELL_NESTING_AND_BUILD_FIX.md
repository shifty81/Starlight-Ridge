# Phase 52g.1 — egui Shell Nesting + Build Fix

## Purpose

This patch fixes a regression where the native egui editor could render a second full editor shell inside the active editor surface. It also fixes the two compile-blocking errors reported after Phase 52g.

## Root cause

The project is on `eframe = 0.34.1`. In this API line, `eframe::App` renders through `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`, and full app panels should be attached to that root UI with `show_inside(ui, ...)`.

The Phase 52g editor still used the older `update(ctx, frame)` style and top-level `Panel::show(ctx, ...)` calls. When those shell panels are routed through the newer app integration, the shell can be laid out as a child of an already-rooted UI, producing a nested copy of the top bar, side panels, center content, bottom console, and status strip.

## Fixes

- Replaced the native editor `eframe::App` implementation with the 0.34-compatible `ui(...)` entry point.
- Routed all shell panels through `show_inside(root_ui, ...)`.
- Replaced deprecated panel sizing calls:
  - `default_width(...)` -> `default_size(...)`
  - `width_range(...)` -> `size_range(...)`
  - `exact_height(...)` -> `exact_size(...)`
- Kept `CentralPanel` as the last shell panel draw call.
- Added `engine_assets::vox` with:
  - `VoxAssetInfo`
  - `scan_vox_files(...)`
  - recursive `.vox` discovery under `assets/voxels`, `assets/models`, and `content/voxels`
  - lightweight MagicaVoxel `SIZE`, `XYZI`, and `RGBA` summary parsing

## Regression guard

Future egui patches should not call full-shell draw functions from inside tab, asset, world, inspector, or workspace content functions. The editor shell should be drawn once per frame in this order:

1. top bar
2. left panel
3. right panel
4. static status bar
5. bottom console panel
6. central workspace panel

The central panel must stay last.
