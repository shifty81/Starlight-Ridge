# Editor Labs Phase 25

## Asset Lab

Use Asset Lab for texture atlas editing and seam cleanup.

Recommended seam-cleanup flow:

1. Select the atlas PNG.
2. Select the tile.
3. Disable the atlas grid if needed.
4. Edit edge pixels in the Pixel Editor.
5. Watch the Selected Tile Preview and Seam Diagnostic.
6. Commit the tile to the atlas.
7. Save the atlas PNG.
8. Keep the game/editor open to confirm hot reload.

The editor grid and atlas highlight are display-only and are not written into the PNG.

## Animation Lab

Animation Lab creates simple static idle animation strips for world tiles and other small texture loops.

Useful examples:

- water shimmer
- shoreline foam
- flowers swaying
- torch/fire flicker
- crop idle movement

Saved metadata includes frame size, frame count, frame durations, loop mode, and random phase flag.

## Character Lab

Character Lab v1 starts the mannequin/clothing-overlay workflow.

The base body layer should stay separate from clothing/equipment layers so future clothing, hair, armor, seasonal outfits, and tool overlays can be edited without destroying the mannequin.

The initial sheets are 4 directional frames:

- front
- side
- back
- side

Future versions should expand this to idle, walk, tool-use, sword swing, carry, and equipped-clothing animation frames.
