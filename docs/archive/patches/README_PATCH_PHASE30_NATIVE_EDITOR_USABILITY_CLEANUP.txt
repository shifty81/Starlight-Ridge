Starlight Ridge Phase 30 - Native Editor Usability Cleanup

Purpose:
- Clean up the Phase 29 native editor overlay so it is less cluttered.
- Make the first native editor controls actually respond to input.
- Keep the center game viewport readable while the editor chrome is open.

Replaced files:
- crates/app/src/lib.rs
- crates/engine_render_gl/src/lib.rs

What changed:
- Adds an EditorShellRenderState passed from app input into the renderer.
- Toolbar icons now reflect the active tool state.
- Mouse hover updates the top hint/status area.
- Mouse click on toolbar icons changes the active tool.
- Mouse click on the left asset cards updates the selected asset text.
- F1 toggles the left Assets dock.
- F2 toggles the right Inspector dock.
- F3 toggles the bottom Console/Validation/Hot Reload dock.
- F5 still reloads live assets/content.
- Keyboard shortcuts select tools:
  V Select
  Space Pan
  B Brush
  E Eraser
  G Fill
  I Eyedropper
  T Tile Picker
  C Collision Paint
  A Asset Studio
  P Playtest

Important boundary:
This is still not the finished native pixel editor. It is the cleanup/interactivity pass that makes the native shell usable enough for the next phase. The next phase should bind real tile-pick, atlas-cell selection, pixel editing, and role/collision save behavior.

Suggested test:
1. Apply over project root and overwrite.
2. Run cargo check.
3. Build editor.
4. Run RUN_EDITOR_DIAGNOSTIC.bat.
5. Try F1/F2/F3, click toolbar icons, and click the asset cards.
