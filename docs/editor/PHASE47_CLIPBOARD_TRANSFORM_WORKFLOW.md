# Phase 47 — Clipboard Transform Workflow

This phase adds the editor-side workflow scaffold for copied pixel/tile/viewport content to be pasted with transforms.

## Goal

The workflow should let the user build one corner, edge, tile group, or visible viewport section, then reuse it quickly by mirroring or rotating the paste preview.

## UX model

The central pixel/asset viewport stays fixed. Tool drawers and copy/paste controls expand inside the left editor region.

```text
[ Tool Rail ][ Tool Drawer ][ Color / Display ][ Copy / Paste ][ Static Canvas ][ Inspector ]
```

## Copy scopes

- Selection
- Current Tile
- Entire Viewport
- Atlas Region

## Paste transforms

- Normal
- Mirror Horizontal
- Mirror Vertical
- Mirror Both
- Rotate 90
- Rotate 180
- Rotate 270

## Safety rules

- Transforming a paste preview does not alter the original copied buffer.
- Paste commits should be undoable.
- Rotated and mirrored pastes must snap to the tile/pixel grid before writing.
- Atlas pastes should validate bounds before committing.
- Atlas region paste should be able to request atlas expansion in a later phase.

## Current implementation status

Phase 47 wires UI state, shortcuts, status messages, and checkpoint export. It does not yet mutate PNG pixels. The next stage should introduce a real in-memory clipboard buffer and an undoable paste commit path.

## Shortcut targets

- `Ctrl/Cmd + C`: copy selected scope
- `Ctrl/Cmd + V`: prepare paste with active transform
- `Ctrl/Cmd + R`: cycle rotate transform

## Next implementation

Phase 48 should add:

- real `EditorClipboardBuffer`
- source pixel region extraction
- transformed preview raster
- paste placement cursor
- undo record generation
- commit/cancel controls
