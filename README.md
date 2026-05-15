# Starlight Ridge

Rust farm/life-sim project with a custom renderer and a native/web editor stack for world and content authoring.

## Current Direction

- World/editor workflow is the active delivery focus.
- Tile/sprite authoring paths are prioritized for production content.
- Gameplay milestones remain tracked in the canonical roadmap, with editor reliability and content tooling treated as immediate enablers.

## Project Status (May 2026)

### Done
- Custom GL runtime foundation (windowing, tile/sprite render paths, content loading/validation)
- Native egui editor shell with focused editor routes
- Web/LAN editor companion for map-layer workflows
- RON-based content pipeline across maps, tiles, gameplay data, and world contracts

### In Progress
- Editor hardening (shared undo/data-bridge adoption, consistency, validation depth)
- Runtime/editor parity for voxel and world-object workflows
- Gameplay loop wiring (player, UI, farming/combat/economy systems)

### Next
- Continue editor completion milestones and world-authoring reliability
- Land playable-first runtime milestones (player on screen, interaction, minimal UI)
- Tighten test and release gates for daily use

## Quick Start

```bash
# baseline validation
cargo check -p app

# run game runtime
cargo run -p app --bin app

# run native editor hub
cargo run -p app --bin editor

# run web/LAN editor (read-only by default)
cargo run -p web_editor_server
```

Convenience menu/build helpers are available via:
- `./build.sh` (Linux/macOS shell)
- `build.bat` and `tools/build_menu.ps1` (Windows)

## Roadmap Hub

- Canonical product roadmap: [`docs/ROADMAP.md`](docs/ROADMAP.md)
- Canonical editor implementation roadmap: [`docs/EDITOR_COMPLETION_ROADMAP.md`](docs/EDITOR_COMPLETION_ROADMAP.md)
- Current phase/spec baseline: [`docs/PHASE53_PIXEL_VOXEL_FIRST_MASTER_SPEC.md`](docs/PHASE53_PIXEL_VOXEL_FIRST_MASTER_SPEC.md)
- Editor usage guide: [`docs/EDITOR_GUIDE.md`](docs/EDITOR_GUIDE.md)

## Notes

- Historical phase notes and legacy rollups are kept under `docs/archive/`.
- If a historical patch note conflicts with the roadmap, treat the roadmap docs above as the active source of truth.
