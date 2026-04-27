# Web Asset Lab Phase 35 Notes

## Display overlay contract

The pixel grid is now rendered by `#pixelGridOverlay` instead of the 32x32 paint canvas. This keeps the source canvas clean and avoids the gray overlay artifact caused by scaled 1-pixel canvas strokes.

## Mirror overlay contract

- `#mirrorVerticalGuide` is shown when horizontal mirroring is active.
- `#mirrorHorizontalGuide` is shown when vertical mirroring is active.

These guides are visual-only overlays and never enter the tile PNG.

## Paint behavior

`mirrorPoints(point)` returns the cursor point plus any active mirrored points. Duplicate points are de-duplicated, so painting on the center split or with both mirrors active does not double-write the same pixel.

The current implementation applies mirror points to pencil, eraser, and fill.
