# Starlight Ridge Phase 21 — Web Editor LAN Bridge

Adds a repo-native web editor server so the editor can be opened from a tablet on the same network.

## Added files

- `crates/web_editor_server/Cargo.toml`
- `crates/web_editor_server/src/main.rs`
- `tools/web_editor/index.html`
- `tools/web_editor/app.css`
- `tools/web_editor/app.js`
- `RUN_WEB_EDITOR_LAN.bat`
- `RUN_WEB_EDITOR_LAN_WRITE.bat`
- `docs/editor/web_editor_lan_phase21.md`

## Modified files

- `Cargo.toml`
- `build.sh`
- `tools/build_menu.ps1`

## Usage

Read-only LAN editor:

```text
RUN_WEB_EDITOR_LAN.bat
```

LAN editor with repo save enabled:

```text
RUN_WEB_EDITOR_LAN_WRITE.bat
```

The server prints the tablet URL when it starts.

## Result

The browser app can load current maps, draw them with the terrain atlas, toggle layers, inspect tiles, paint layer symbols, export `layers.ron`, and optionally save `layers.ron` back to the project.
