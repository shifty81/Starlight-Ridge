# Phase 43 — egui Warning Cleanup + Pixel Editor Brush Drawer Scaffold

Phase 43 stabilizes the editor UI foundation before adding larger art systems.

## Goals

- Move the egui panel calls to the current `egui::Panel` / `show_inside` API shape.
- Remove the unused old web Asset Lab launcher method from the native editor path.
- Rename the generic Brush tool to Pencil in the native editor shell.
- Add a shared pixel-editor tool state for tile, atlas, prop, character, and animation-frame editing.
- Add a left-side fixed control region with primary tool rail, expandable tool drawer, RGBA controls, display toggles, and zoom controls.
- Add a static central pixel canvas preview for `Assets > Pixel Editor`.
- Add a Character Studio placeholder viewport so Phase 44 has a clear landing zone.

## Layout Rule

The pixel canvas should feel stable. Tool drawers open inside the left control region. The central asset canvas should not jump around when the Pencil drawer opens or collapses.

```text
[ Tool Rail ][ Optional Tool Drawer ][ Color / RGBA + Display ][ Static Canvas ][ Inspector ]
```

## Tools Added to the Scaffold

- Select
- Pencil
- Erase
- Fill
- Pick
- Stamp

## Pencil Drawer Options

- 1px Pencil
- Square Brush
- Circle Brush
- Line
- Dither
- Replace Color
- Size
- Opacity
- Mirror X
- Mirror Y
- Alpha Lock

## Display Options

- Pixel grid
- Checkerboard
- Tile bounds
- Center guides
- Zoom slider
- Fit / 100% / 200% / 400% quick buttons

## Character Studio Handoff

Phase 43 does not implement the full character editor yet. It adds the placeholder and scale visualization so Phase 44 can build:

- Mannequin
- Paperdoll Layers
- Equipment / Overlays
- Animation Preview
- Scale Validator
- Export / Metadata
