# Starlight Ridge Phase 32 — Native Asset Studio Preview Helper Hotfix

This hotfix repairs the Phase 31 compile error in `engine_render_gl`.

## Fix

Phase 31 still calls three small native preview drawing helpers from `draw_clean_preview`:

- `draw_mini_tile`
- `draw_seam_grid`
- `draw_atlas_mini`

Those helper definitions were accidentally removed while cleaning out older Phase 29 panel helpers. This patch restores only the active preview helpers and leaves the unused Phase 29 panel helpers removed.

## Replaced file

- `crates/engine_render_gl/src/lib.rs`

## After applying

Run:

```bash
cargo check
cargo build --release --all-targets
```

Then launch:

```bat
RUN_EDITOR_DIAGNOSTIC.bat
```
