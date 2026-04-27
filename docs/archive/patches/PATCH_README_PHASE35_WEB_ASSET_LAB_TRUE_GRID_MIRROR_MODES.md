# Starlight Ridge Phase 35 — Web Asset Lab True Grid + Mirror Modes

This patch fixes the Pixel Editor grid behavior and adds mirror painting controls to the browser Asset Lab.

## Replaced

- `tools/asset_lab.html`

## Main fixes

The old pixel-grid rendering drew lines directly into the 32x32 editor canvas. Because the canvas is scaled up, those lines became huge semi-transparent bands and made the tile look like it had a gray overlay.

Phase 35 changes the pixel grid into a display-only overlay above the canvas:

- thin black line grid
- aligned to true pixel boundaries
- no gray tint
- no modification of the image data
- no effect on saved atlas PNGs

## Mirror modes

The right-side Display Options panel now includes:

- `Pixel Grid`
- `Atlas Grid`
- `Mirror H`
- `Mirror V`

`Mirror H` mirrors left/right and shows a vertical red center guide.

`Mirror V` mirrors top/bottom and shows a horizontal red center guide.

For 32x32 tiles, the guide sits at the center split between the two middle pixel columns or rows.

## Mirror-aware tools

Mirror painting works with:

- Pencil
- Eraser
- Fill

Pick/eyedropper still samples only the pixel under the cursor.

## Notes

This patch only changes the browser editor UI and paint behavior. It does not touch Rust code.
