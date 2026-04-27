# Patch Phase 42 — Character Scale + Brush Workflow + egui delimiter hotfix

Extract this patch over the project root.

## Fixes

- Removes the duplicate empty `draw_top_bar` function header that caused `egui_editor.rs` to report an unclosed delimiter.

## Adds

- Character scale metadata for 1.0, 1.5, 2.0, and 2.25 tile-height character classes.
- Shared pixel editor workflow metadata for brush drawers, zoom, pan, and static canvas behavior.
- Editor design documentation for Character Studio and shared pixel editor direction.

## After applying

Run:

```bat
BUILD_MENU.bat
```

Then choose:

```text
2) cargo check
```
