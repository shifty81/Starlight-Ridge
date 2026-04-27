Starlight Ridge Phase 28 - Content Registry Sidecar Skip Hotfix

Fixes the native editor/app immediate-close caused by editor sidecar RON files being loaded as runtime content schemas.

Problem seen in RUN_EDITOR_DIAGNOSTIC.bat:
  failed to parse RON content content/tiles/base_tileset_roles.ron: Unexpected missing field named `id` in `TilesetDef`

Root cause:
  The runtime registry was loading every .ron file in content/tiles as a TilesetDef and every .ron file in content/metadata as a SpriteSheetDef.
  Phase 25-27 added editor-only sidecars, including base_tileset_roles.ron and character metadata files, which are valid editor files but not runtime TilesetDef/SpriteSheetDef files.

Changed file:
  crates/game_data/src/lib.rs

Behavior after patch:
  - content/tiles/*.ron is loaded only when it looks like a tileset schema.
  - content/metadata/*.ron is loaded only when it looks like a sprite-sheet schema.
  - editor-only sidecars remain in place and are logged as skipped instead of crashing startup.
  - actual referenced runtime content still validates through the existing registry validation step.

Apply over the project root, overwrite, then run:
  cargo check
  BUILD_RELEASE_ALL.bat or your normal build option
  RUN_EDITOR_DIAGNOSTIC.bat
