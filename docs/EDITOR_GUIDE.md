# Editor Guide

Starlight Ridge ships two editor surfaces that share the same content data:

- **Native egui editor** — a desktop application built with [egui](https://github.com/emilk/egui)/eframe, launched with `cargo run -p app --bin editor`
- **Web / LAN editor** — a browser application served over your local network, launched with `cargo run -p web_editor_server`

---

## Native egui Editor

### Launch

```sh
cargo run -p app --bin editor
# or
RUN_EDITOR_DEBUG.bat   (Windows)
```

### Tab structure

The editor is organised into nine top-level workspace tabs.

#### Project
- **Overview** — content registry summary (item count, crop count, map count, etc.)
- **Logs** — live `runtime_latest.log` tail
- **Validation** — content error list; click an entry to see details
- **Reload Content** — hot-reload all RON files without restarting

#### World
- **Map Paint** — tile painting on the active map
- **Layers** — layer visibility, lock, opacity
- **Interactions** — trigger zone placement and editing
- **Spawns** — spawn point placement
- **Terrain Rules** — autotile rule viewer

#### Assets
- **Terrain Atlas** — tile role list, filter by role name, edit role/collision metadata
- **Atlas Compare** — side-by-side atlas import tools
- **Pixel Editor** — per-pixel drawing tools (brush, erase, fill, eyedropper)
- **Voxels** — `.vox` source file browser (Phase 52c+)
- **Props / Objects** — prop sprite browser
- **Seasons** — seasonal tileset variant viewer

#### Animation
- **Clips** — animation clip list
- **Timeline** — frame-by-frame editor
- **Direction Sets** — 4/8-direction frame mapping
- **Events** — animation event hooks (footstep, SFX, etc.)
- **Preview** — live animated preview

#### Character
- **Base Body** — paperdoll base layers
- **Clothing** — clothing and equipment overlay layers
- **Hair / Face** — head customisation
- **8-Dir Preview** — directional walk cycle preview

#### Logic
- **Graphs** — visual logic/event graph editor
- **Event Bindings** — key/trigger → action wiring
- **Validation** — logic contract validation

#### Data
- **Items** — item list, inline editor
- **Crops** — crop list, inline editor
- **NPCs** — NPC list, inline editor
- **Dialogue** — dialogue tree viewer/editor
- **Quests** — quest list, inline editor
- **Shops** — shop stock editor
- **Schedules** — NPC schedule editor

#### Playtest
- **Player State** — live player position, facing, inventory (requires player sim to be wired)
- **Movement** — movement speed, sprint, collision debug toggle
- **Camera** — zoom, follow settings

#### Settings
- **Editor Preferences** — theme, font size
- **Project Paths** — content root, asset root
- **Web Companion** — LAN server port and write-mode toggle
- **Keybinds** — editor action keybind overrides

### Keyboard shortcuts (editor)

| Shortcut | Action |
|---|---|
| `Ctrl+R` | Reload content |
| `Ctrl+S` | Save current map layers |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `B` | Brush tool |
| `E` | Erase tool |
| `F` | Fill tool |
| `K` | Eyedropper (pick) |
| `1–9` | Select palette tile slot |

> Note: Several panels are still partial or stubbed. See [EDITOR_COMPLETION_ROADMAP.md](EDITOR_COMPLETION_ROADMAP.md) for the editor-specific finish order, and [ROADMAP.md](ROADMAP.md) for the broader game roadmap.

---

## Web / LAN Editor

### Launch

```sh
# Read-only (safe for shared use)
cargo run -p web_editor_server

# Write mode (saves back to content/*.ron on disk)
STARLIGHT_WEB_ALLOW_WRITE=1 cargo run -p web_editor_server
```

Windows shortcuts: `RUN_WEB_EDITOR_LAN.bat` (read-only) and `RUN_WEB_EDITOR_LAN_WRITE.bat`.

Open the URL printed to the terminal — e.g. `http://192.168.1.10:8787/` — on any device on the same network.

### Features

| Feature | Status |
|---|---|
| Map load and display | ✅ |
| Layer visibility toggle | ✅ |
| Palette (based on active layer legend) | ✅ |
| Paint / erase / eyedropper / inspect | ✅ |
| Save map layers to disk (write mode) | ✅ |
| Brush size + dither toggle | ✅ |
| Fill tool | ✅ |
| Layer clipboard (copy/paste with transforms) | ✅ |
| Paste transforms (mirror H/V, rotate 90/180/270) | ✅ |
| Undo / redo | ❌ planned |
| Selection / marquee | ❌ planned |
| Collision / spawn / trigger editing | ❌ planned |
| Atlas compare / import | ❌ planned |
| Scene generator preview + bake | ❌ planned |
| Visual validation report | ❌ planned |

### Web editor keyboard shortcuts

| Key | Action |
|---|---|
| `B` | Brush |
| `E` | Erase |
| `F` | Fill |
| `K` | Eyedropper |
| `Ctrl+C` | Copy selection |
| `Ctrl+V` | Paste |
| `[` / `]` | Decrease / increase brush size |
| `H` | Mirror paste horizontal |
| `V` | Mirror paste vertical |
| `R` | Rotate paste 90° |

### Environment variables

| Variable | Default | Description |
|---|---|---|
| `STARLIGHT_WEB_HOST` | `0.0.0.0` | Bind address |
| `STARLIGHT_WEB_PORT` | `8787` | HTTP port |
| `STARLIGHT_WEB_ALLOW_WRITE` | (unset = read-only) | Set to `1` / `true` / `yes` to enable saves |

---

## Asset Lab

A standalone browser tool for inspecting texture atlases:

```sh
# Python 3 required
python tools/asset_lab_server.py
# then open tools/asset_lab.html in a browser
```

Or use `RUN_ASSET_LAB.bat` / `RUN_ASSET_LAB_GIT_BASH.sh`.

---

## Checkpoint manifest

The native editor can write a checkpoint manifest summarising the current content state. Use **Project > Build > Write Checkpoint Manifest**. The file is written to `logs/editor_live_preview_manifest.ron`.
