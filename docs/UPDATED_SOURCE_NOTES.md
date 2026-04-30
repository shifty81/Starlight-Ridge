# Starlight Ridge Updated Source Notes

## Phase 15 Art Contract Reset

This full source zip updates the project around a cleaner runtime art/data contract.

### Major changes

- Added `assets/textures/terrain_atlas_phase15_contract.png`.
- Regenerated `assets/textures/player_walk.png` as a clean 4x5 sheet.
- Rewrote `content/tiles/base_tileset.ron` to expose explicit named terrain/object roles.
- Rewrote `content/metadata/entity_sprite_sheet_phase5.ron` for the new player + prompt sheet.
- Cleaned `content/maps/starter_farm/layers.ron` into semantic `ground` and `objects` layers.
- Added collision map generation from terrain, object layers, and props.
- Added collision-aware player movement.
- Added temporary visible interaction prompt sprite.
- Wired semantic terrain resolution to metadata-driven tile ids instead of guessed atlas columns.

### Test order

1. Extract fresh.
2. Double-click `BUILD_MENU.bat`.
3. Run option `2) cargo check`.
4. Run option `4) cargo build --release game exe`.
5. Run option `5) Run game debug`.
6. Run option `6) Run editor debug`.

### Expected visual result

- Terrain should look cleaner and less like a mismatched atlas test.
- The player should no longer look sliced into pieces while moving.
- The player should collide with water, cliffs, trees, fences, and main interactable props.
- A yellow `!` prompt should appear near interaction targets.

## Phase 53l — Voxel Composition 3D Selection + Transform Handles

- Added 3D hover hit testing for voxel panel instances, sockets, and selected-instance XYZ handles.
- Added click selection sync between the 3D preview, 2D composition canvas, selected source panel, and selected socket.
- Added drag-to-translate handles with configurable grid snap.
- Added composition transform undo/redo controls for the 3D preview.
- Rebuilds live preview data in memory after handle-based transforms.
- Preserves the single-root egui shell rule; no gameplay code was changed.
