# Starlight Ridge — Current Roadblocks to Intended Behavior

This project now builds far enough that the remaining risks are mostly runtime, content-contract, and visual-resolution issues rather than basic Cargo build issues.

## Addressed in this source update

- Fixed `same_transition_group_at` so the transition resolver can actually compile and compare neighboring tile groups correctly.
- Hardened project-root discovery so launching from `target/debug`, `target/release`, the repo root, or a nested tool folder can still find `/content`, `/assets`, and `Cargo.toml`.
- Added an explicit missing-atlas failure path before renderer bootstrap so a missing terrain PNG creates a useful runtime error instead of a vague OpenGL/texture failure.
- Added a redraw keepalive through `ApplicationHandler::about_to_wait` so the app keeps requesting frames after the renderer is active.

## Still important gaps

### 1. Atlas transition contract is still assumed, not fully data-driven

The current resolver assumes terrain rows and transition columns in the packed atlas. That is enough for a prototype, but the long-term fix is explicit role metadata, for example:

- `grass_base`
- `grass_edge_n`
- `grass_edge_e`
- `grass_corner_ne`
- `water_edge_s`
- `tilled_watered_inner_ne`

Without explicit named roles, a future atlas edit can visually break the resolver without causing a compile error.

### 2. The current transition resolver is conservative, not a full 47-tile Wang autotiler

The resolver uses cardinal neighbor checks and a small set of safe transition columns. This should stop the worst random-tile look, but it will not perfectly solve all concave corners, isolated islands, and diagonal blends.

### 3. Game and editor still share almost the same runtime path

The editor binary currently opens the same window/render loop with edit mode enabled. It does not yet have a real editor UI, layer picker, tile inspector, or tile painting mode.

### 4. No automated visual regression test exists yet

There is no screenshot comparison or tile-count assertion test. That means visual regressions still need to be spotted manually by launching the game.

### 5. Runtime logs are present, but not yet mirrored into an in-game diagnostics overlay

The logs should now help with immediate-close issues, but the app itself still does not show a user-facing error panel if content or renderer startup fails.

### 6. Props/spawns/triggers are loaded as content but not rendered as finished gameplay objects

The map has props, spawns, and triggers, but the current render path is still primarily terrain/direct tile layers. Farmhouse, shipping bin, well, player placement, and map transition behavior need their own visible/gameplay path.

## Recommended next implementation order

1. Run `cargo check` and fix any compile errors introduced by transition wiring.
2. Run game debug and inspect `logs/runtime_latest.log`.
3. If it opens, verify `transition_overlays` is greater than zero in the log.
4. Add an explicit atlas-role metadata file so resolver columns are no longer hardcoded.
5. Add a starter-farm visual smoke test that verifies terrain layer draw counts and missing refs.
6. Convert editor mode into a visible debug/editor overlay with map/layer/tile information.
