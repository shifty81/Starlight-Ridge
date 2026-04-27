# Phase 51h Implementation Notes — Web Mobile LAN Editor Shell

## Goal

Browsing to the LAN server root should open a mobile-friendly editor immediately, without needing a separate Android APK or standalone project.

## Files touched

- `tools/web_editor/index.html`
- `tools/web_editor/app.css`
- `tools/web_editor/app.js`
- `crates/web_editor_server/src/main.rs`

## Implemented

- Root address opens the editor directly.
- View picker remains available with `?launcher=1` and the top-bar View button.
- Mobile command bar:
  - Paint
  - Erase
  - Fill
  - Pick
  - Tools
  - Save
- Mobile tools drawer:
  - Map selector
  - Layer selector
  - Zoom buttons
  - Grid toggle
  - Quick palette strip
  - Layer visibility strip
- Touch behavior:
  - One-finger paint/select.
  - Pinch zoom.
  - Two-finger pan.
- Server console prints the direct root LAN URL first.

## Test plan

1. Run `cargo check`.
2. Launch option `18` for read-only LAN mode.
3. Open `http://<PC-LAN-IP>:8787/` from Android Chrome.
4. Confirm the map loads without needing the launcher.
5. Tap Paint / Erase / Fill / Pick from the bottom bar.
6. Open Tools and confirm map, layer, zoom, grid, palette, and layer controls work.
7. Relaunch with option `19` and confirm Save is enabled.

## Known limitations

- The web editor still edits the RON map layer representation, not PNG pixel buffers.
- The native egui pixel editor remains the deeper PNG-backed pixel-editing path.
- Mobile copy/paste, rectangle selection, and full mirror-aware paste should be implemented after the touch shell is stable.
