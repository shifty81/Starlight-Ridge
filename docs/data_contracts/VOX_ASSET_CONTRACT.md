# VOX Asset Contract — Phase 51f

Starlight Ridge now has a first-pass MagicaVoxel `.vox` asset path.

## Search locations

The editor/runtime asset scanner checks:

- `assets/voxels/`
- `assets/models/`
- `content/voxels/`

## Supported `.vox` chunks

The loader currently reads:

- `SIZE` — model dimensions
- `XYZI` — voxel coordinates and palette indices
- `RGBA` — palette colors

Files missing `SIZE` or `XYZI` are skipped with a warning instead of crashing the editor.

## Current editor integration

Open the egui editor and use:

`Assets -> VOX Models`

The panel lists discovered `.vox` files, dimensions, voxel count, palette size, and relative path.

## Next implementation target

The current patch makes `.vox` files valid project assets. The next phase should add actual production tools:

1. VOX preview thumbnail generation.
2. Orthographic bake to 2D PNG sprites.
3. Directional bake profiles for props/NPC decorations.
4. Collision footprint extraction from occupied voxels.
5. Metadata export into prop/object definitions.
6. Optional editor placement as object assets.
