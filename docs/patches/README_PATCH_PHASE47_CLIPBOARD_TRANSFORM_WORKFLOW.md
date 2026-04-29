# Patch Phase 47 — Clipboard Transform Workflow

## Files changed

- `crates/app/src/egui_editor.rs`
- `content/editor/pixel_clipboard_transform_phase47.ron`
- `docs/editor/PHASE47_CLIPBOARD_TRANSFORM_WORKFLOW.md`
- `docs/roadmap/CHAT_ROLLUP_PHASE47.md`
- `docs/patches/README_PATCH_PHASE47_CLIPBOARD_TRANSFORM_WORKFLOW.md`

## Summary

Adds the scaffold for editor clipboard copy/paste transforms:

- copy selection/current tile/entire viewport/atlas region
- paste normal, mirror horizontal, mirror vertical, mirror both, rotate 90/180/270
- left-side copy/paste controls
- static canvas toolbar buttons
- checkpoint export

## Notes

This phase intentionally does not write pixels yet. The next phase should add a real in-memory clipboard image buffer, transformed preview raster, and undoable paste commit.
