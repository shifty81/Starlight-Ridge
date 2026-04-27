# Starlight Ridge Phase 34 — Web Asset Lab Professional UI Polish

This patch continues the browser-first editor workflow. It does **not** change the Rust runtime or native editor. The goal is to make the web Asset Lab feel closer to a real production editor before porting the workflow back into the native Rust editor.

## Replaced files

- `tools/asset_lab.html`
- `tools/asset_lab_server.py` *(carried forward from Phase 33)*

## Main UI changes

### Pixel editor canvas

The center panel is now focused on the editable pixel canvas. The large tile canvas sits in its own framed workspace with less clutter around it.

### Left vertical tool rail

Core tools moved from text buttons under the canvas into an icon-first rail:

- Pencil
- Eraser
- Fill
- Pick color
- Marquee select
- Copy
- Cut
- Paste
- Deselect
- Undo
- Redo
- Clear

Icons use `title` and custom hover tooltips so the UI stays compact while still showing labels on mouse-over.

### Right preview/properties panel

Color and display controls moved to the right side under the selected-tile preview:

- current color swatch
- browser color picker
- R/G/B/A sliders
- live numeric outputs
- visible Pixel Grid toggle
- visible Atlas Grid toggle

The grid toggles are now prominent switch-style controls instead of small checkboxes.

### Bottom action bar

Secondary tile operations moved into a dedicated action strip below the canvas:

- Commit Tile To Atlas
- Copy Left → Right
- Copy Right → Left
- Copy Top → Bottom
- Copy Bottom → Top
- Blend L/R
- Blend T/B
- Soften Border
- Export Runtime-Safe Atlas

## Visual fixes

- Grid overlays are stronger and easier to see.
- Active tool state is more obvious.
- Tool layout is grouped by draw, edit, and history operations.
- The center canvas is less cluttered.

## Usage

Run:

```bat
RUN_ASSET_LAB.bat
```

Open the **Asset Lab** tab. The workflow remains the same, but the controls are reorganized:

1. Pick/load an atlas from the left Texture Map panel.
2. Select a tile from the atlas.
3. Use the left icon rail to draw/select/edit.
4. Use the right panel for color and grid visibility.
5. Use the bottom action bar for commit/seam/export actions.
