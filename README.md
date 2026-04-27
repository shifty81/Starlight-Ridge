# Starlight Ridge

A farming and adventure game built from scratch in Rust — custom OpenGL engine, native egui editor, and a browser-based LAN companion editor all in one workspace.

> **Status: Early development.** The engine, data pipeline, and editor shell are working. Core gameplay systems (movement, farming, NPCs, dialogue, combat, save) are next.

---

## Features (current)

- Custom OpenGL/glow renderer with tilemap batching and sprite pipeline
- RON-driven content pipeline — maps, tilesets, items, crops, NPCs, dialogue, quests all defined as data
- Semantic autotile resolver (terrain transitions without hardcoded atlas offsets)
- Native egui editor — multi-tab shell with tile inspector, atlas pipeline tools, and content reload
- Web LAN editor — runs on `http://<your-ip>:8787`, usable on a tablet or second screen
- 27-crate Cargo workspace with a clean engine / game / editor split

---

## Quick Start

### Prerequisites

- Rust **stable** toolchain (`rustup show` should report stable)
- A GPU with OpenGL 3.3+ support
- On Linux: `libGL`, `libasound2-dev` (audio stub, not yet active)
- On Windows: no extra dependencies

```sh
# Clone and enter the repo
git clone https://github.com/shifty81/Starlight-Ridge.git
cd Starlight-Ridge

# Verify the workspace compiles
cargo check

# Run the game (debug)
cargo run -p app

# Run the native editor
cargo run -p app --bin editor

# Run the web/LAN editor server (read-only)
cargo run -p web_editor_server
# Then open http://127.0.0.1:8787/ in a browser
```

Windows batch shortcuts are also available at the repo root (`BUILD_MENU.bat`, `RUN_GAME_DEBUG.bat`, etc.).

---

## Controls

| Key | Action |
|---|---|
| `WASD` / Arrow keys | Move (not yet wired to player sim) |
| `E` / `Space` | Interact |
| `Escape` | Close |
| `Tab` | Toggle editor overlay (editor binary) |
| `1–5` | Editor tool select (overlay mode) |

---

## Project Layout

```
Starlight-Ridge/
├── crates/           # Rust workspace — 27 crates
│   ├── app/          # Main game + editor binaries, main loop
│   ├── engine_*/     # Platform, GL renderer, input, audio, math, time, assets
│   ├── game_*/       # All gameplay systems (many are stubs — see roadmap)
│   ├── editor_*/     # Editor runtime state, tools, inspector, undo, data bridge
│   ├── shared_types/ # StableId, GridPos, ProjectError
│   └── web_editor_server/ # LAN HTTP server for the browser editor
├── content/          # RON data files (maps, tiles, items, NPCs, quests, …)
├── assets/           # Textures, shaders, audio, art source, voxels
├── tools/            # Python/HTML tooling (asset lab, web editor)
├── docs/             # Architecture, guides, roadmap, editor docs (this folder)
└── saves/            # Player save files (git-ignored)
```

See **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** for the full crate dependency map and data-flow diagram.

---

## Documentation

| Document | Description |
|---|---|
| [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md) | Prerequisites, build commands, troubleshooting |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Crate map, data flow, launch modes |
| [docs/EDITOR_GUIDE.md](docs/EDITOR_GUIDE.md) | Native egui editor and web editor usage |
| [docs/CONTENT_AUTHORING.md](docs/CONTENT_AUTHORING.md) | Adding maps, tiles, items, NPCs, crops, dialogue |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Milestones and implementation order |

---

## License

MIT — see `Cargo.toml` workspace metadata.
