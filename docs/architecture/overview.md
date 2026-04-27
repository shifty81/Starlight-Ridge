# Architecture Overview

See [../ARCHITECTURE.md](../ARCHITECTURE.md) for the full crate reference, data-flow diagram, and launch mode guide.

**Summary:**

- `engine_*` — platform and rendering primitives (window, GL, input, audio, math, time, assets)
- `game_*` — runtime simulation and content use (data, world, worldgen, core, and gameplay stubs)
- `editor_*` — live authoring overlay (editor_core has real implementation; others are stubs)
- `shared_types` — stable low-level contracts (StableId, GridPos, ProjectError)
- `app` — main game + editor binaries; owns the run loop
- `web_editor_server` — LAN HTTP server for the browser editor
