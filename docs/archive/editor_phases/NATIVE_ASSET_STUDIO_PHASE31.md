# Native Asset Studio Phase 31

Phase 31 establishes the first usable native Asset Studio loop inside the Rust editor.

## Workflow

1. Open the editor.
2. Select the Terrain Atlas asset in the left dock.
3. Click a tile in the center viewport, or click the Tile/Atlas preview cards to cycle tiles.
4. Inspect tile name, atlas cell, role, and collision in the right dock.
5. Click `Role` to cycle tile role.
6. Click `Block` to cycle collision mode.
7. Click `Export` to write the current selection manifest.
8. Press `F5` to reload content manually if needed.

## Files touched at runtime

- `content/tiles/base_tileset_roles.ron`
- `content/tiles/base_tileset_roles.ron.phase31.bak`
- `artifacts/native_asset_studio_selection.ron`

## Next recommended phase

Phase 32 should add actual native texture/atlas editing:

- tile atlas texture preview from the real PNG
- selected tile pixel grid
- brush/eraser/fill tools backed by PNG writes
- undo/redo for native edits
- runtime-safe padded atlas export wired to renderer UVs
