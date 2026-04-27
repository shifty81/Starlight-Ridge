Starlight Ridge Phase 35 - Web Asset Lab True Grid + Mirror Modes

Apply over the Starlight Ridge project root and overwrite files.

Changed files:
- tools/asset_lab.html

Adds/fixes:
- Replaces the old gray pixel-grid canvas drawing with a true display-only CSS grid overlay.
- Pixel grid now uses thin black lines aligned to the active tile pixel boundaries.
- Pixel grid no longer tints, darkens, or modifies the edited art.
- Adds Horizontal Mirror mode for left/right mirrored drawing.
- Adds Vertical Mirror mode for top/bottom mirrored drawing.
- Adds red vertical/horizontal center guide overlays for active mirror modes.
- Pencil, eraser, and fill now respect mirror mode.
- Mirror and grid controls live in the right-side Display Options panel.

Run:
RUN_ASSET_LAB.bat

No Rust files are changed, so cargo check is not required.
