Starlight Ridge Phase 31 — Native Editor Clickable Asset Studio

Apply over the project root and overwrite files.

Then run:
  cargo check
  cargo build --release --all-targets
  RUN_EDITOR_DIAGNOSTIC.bat

This patch replaces:
  crates/app/src/lib.rs
  crates/engine_render_gl/src/lib.rs

It adds docs under docs/ and docs/editor/.

Expected result:
  The phase 29 dead-code warnings should be gone.
  Toolbar, tabs, asset cards, inspector actions, bottom tabs, and viewport tile picking should respond to clicks.
  Role/collision edits should save to content/tiles/base_tileset_roles.ron.
