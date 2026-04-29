# Web Asset Lab Polish — Phase 34

Phase 34 makes the browser editor a cleaner UX target for the future native editor port.

## Layout contract

The Asset Lab page should use this structure:

- **Left panel:** Texture Map, atlas selection, atlas thumbnail, templates.
- **Center panel:** Pixel canvas only, plus a left-side icon tool rail and bottom action strip.
- **Right panel:** Selected preview, color controls, display toggles, atlas output preview, seam diagnostic.

## Control placement rule

- Core tools go left.
- Color/display properties go right.
- Canvas remains central.
- Secondary cleanup/export actions go below the canvas.

## Native editor implication

When this workflow is later ported into the Rust editor, the native Asset Studio should mimic this browser layout instead of using the current viewport-first native shell as the Asset Lab workspace.
