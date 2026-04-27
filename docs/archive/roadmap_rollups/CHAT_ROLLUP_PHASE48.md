# Chat Rollup — Phase 48

## Decisions

- Starlight Ridge should use scene-based world generation, not a primary infinite-world model.
- Generated maps should become editable draft scenes.
- The final game can add many scenes over time by generating a base and then hand-editing it.
- World generation must produce semantic terrain IDs first.
- Autotiling remains a separate pass that converts semantic terrain into visual atlas tiles.
- Runtime/editor launch must tolerate editor-only metadata files inside `content/metadata`.

## Implemented

- Added `crates/game_worldgen`.
- Added starter farm/coastal plot generator scaffold.
- Added semantic terrain ID contracts.
- Added scene registry and template RON files.
- Added generated-to-editable workflow contract.
- Added protected-layer rules.
- Fixed `SpriteSheetDef` metadata parsing crash by skipping non-sprite metadata files.

## Next

Phase 49 — WorldGen editor preview + bake controls.
