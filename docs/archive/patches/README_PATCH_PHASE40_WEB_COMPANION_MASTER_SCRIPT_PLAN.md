# Patch Phase 40 — Web Companion Modes + Master Script Plan

## Drop-in files

- `tools/web_editor/index.html`
- `tools/web_editor/app.css`
- `tools/web_editor/app.js`
- `crates/web_editor_server/src/main.rs`
- `tools/build_menu.ps1`
- `build.sh`
- `content/editor_web/web_companion_phase40.ron`
- `docs/editor/PHASE40_WEB_COMPANION_MASTER_SCRIPT_EGUI_WRAP_PLAN.md`
- `docs/roadmap/CHAT_ROLLUP_PHASE40.md`

## Test

1. Extract over the project root.
2. Run `BUILD_MENU.bat`.
3. Choose `20) Self-heal project root + docs`.
4. Choose `2) cargo check`.
5. Choose `18) Run web editor on LAN (read-only)` or `19) Run web editor on LAN (save enabled)`.
6. Open the displayed LAN URL from a tablet, adding `?mode=tablet` if needed.

## Notes

This phase is a planning/scaffold implementation, not the final atlas importer or logic editor.
