Starlight Ridge Phase 32 - Native Asset Studio Preview Helper Hotfix

Apply over the project root after Phase 31 and overwrite files.

Fixes cargo check errors:
- cannot find function draw_mini_tile
- cannot find function draw_seam_grid
- cannot find function draw_atlas_mini

Replaces:
- crates/engine_render_gl/src/lib.rs

Then run:
- cargo check
- cargo build --release --all-targets
- RUN_EDITOR_DIAGNOSTIC.bat
