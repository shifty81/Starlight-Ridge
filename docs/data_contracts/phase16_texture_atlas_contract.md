# Phase 16 Texture Atlas Contract

This phase replaces the rudimentary 11x13 packed terrain atlas with a cleaner 16x16 terrain atlas page.

## Active atlas

- File: `assets/textures/terrain_atlas_phase16_refined.png`
- Tile size: `32x32`
- Grid: `16 columns x 16 rows`
- Content binding: `content/tiles/base_tileset.ron`
- Tileset id remains `base_tiles` so existing maps do not need to change.

## Row contract

| Row | Purpose |
| --- | --- |
| 0 | Bright grass variants |
| 1 | Dark grass variants |
| 2 | Dirt variants |
| 3 | Sand/path variants |
| 4 | Tilled dry soil variants |
| 5 | Watered soil variants |
| 6 | Stone floor variants |
| 7 | Wood floor/building floor variants |
| 8 | Beach sand variants |
| 9 | Shallow water animation frames |
| 10 | Deep water animation frames |
| 11 | Cliff/rock variants |
| 12 | Transparent decor/object sprites |
| 13 | Early building and shoreline support tiles |
| 14 | Debug/editor overlay tiles |
| 15 | Reserved |

## Rules locked in by this patch

1. Terrain base cells must be fully opaque. No clear/black gutters in terrain rows.
2. Transparent pixels are only allowed on object, decor, debug, and future overlay rows.
3. Columns `0-2` are safe filled base variants for the current renderer.
4. Columns `3-9` are reserved for future edge/transition/autotile masks.
5. Water animation frames now exist in the atlas, but runtime animation is not enabled yet.
6. The `base_tiles` id is preserved to avoid map migration.

## Next renderer upgrade

The next renderer-side step should remove terrain roles from hardcoded Rust enums and resolve them from metadata. The renderer should eventually read:

- terrain role
- transition group
- movement/collision flags
- footstep sound
- farmability/tillability/waterability
- animation frames
- seasonal overrides
- transition priority

That is the correct point to re-enable shoreline, path, soil, and farm-plot blending.
