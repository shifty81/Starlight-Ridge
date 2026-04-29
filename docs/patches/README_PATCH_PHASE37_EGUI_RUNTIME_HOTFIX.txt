Starlight Ridge Phase 37 — egui runtime hotfix

Purpose:
- Fix both game.exe/app.exe and editor.exe failing at startup on content/tiles/base_tileset_roles.ron.
- Keep base_tileset_roles.ron as an editor sidecar without allowing the runtime content registry to parse it as a TilesetDef.
- Remove the generated Templates section from the web Asset Lab and reserve that area for the future atlas compare/import queue.
- Keep editor.exe on the native egui path and remove top-level routing back into the web Asset Lab from the egui editor toolbar.

Files changed:
- crates/game_data/src/lib.rs
- crates/app/src/egui_editor.rs
- tools/asset_lab.html

Expected result:
- app.exe no longer fails with: Unexpected missing field named `id` in `TilesetDef` for base_tileset_roles.ron.
- editor.exe no longer fails for the same sidecar parse error.
- Asset Lab no longer shows the old placeholder Templates section.

Build:
- Run your normal build menu or cargo build --release.
- If a new compile error appears, upload the new log from logs/ or the build menu output.
