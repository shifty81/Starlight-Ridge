# Phase 53k.2 — Voxel Panel Workbench Layout Fix

## Purpose

The Phase 53k viewport polish made the Voxel Panels asset tab too dense: kit controls, 2D slice editing, composition canvas, 3D preview, material legend, and diagnostics were all visible at once. On normal editor resolutions this looked like panels stacked on top of panels and made the tool hard to read.

## Change

This patch keeps the existing Phase 53k systems, but changes the Voxel Pixel Panel Designer into a focused workbench layout with internal modes:

- **Panel Editor** — kit controls, panel metadata, sockets, and the 2D voxel-pixel slice editor.
- **Composition** — composition canvas and side controls/validation.
- **3D Preview** — read-only Phase 53i/53k preview RON viewport, controls, grid, axes, sockets, connections, hover labels, and material legend.
- **Diagnostics** — validation and preview diagnostics without crowding the authoring canvas.

## Safety rules preserved

- No gameplay changes.
- No renderer crate changes.
- No top-level egui shell changes.
- No nested editor shell calls.
- The 3D preview remains read-only.
- Existing Phase 53k preview diagnostics remain intact.

## Test checklist

1. Run `cargo check --workspace`.
2. Run the editor debug build.
3. Open `Assets -> Voxel Panels`.
4. Confirm the Voxel Panels page shows a **Workbench** selector.
5. Switch between `Panel Editor`, `Composition`, `3D Preview`, and `Diagnostics`.
6. Confirm the whole editor is not nested inside itself.
7. Confirm the old crowded three-column layout is gone.
8. Confirm `3D Preview` can still load/export preview RON.
