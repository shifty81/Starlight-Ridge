# Phase 28 — Content Registry Sidecar Skip Hotfix

## Fix

The native editor was closing immediately because the runtime content registry treated editor sidecar files as runtime schema files.

Example failure:

```text
failed to parse RON content content/tiles/base_tileset_roles.ron: Unexpected missing field named `id` in `TilesetDef`
```

## Root Cause

`game_data::load_registry` loaded every `.ron` under:

- `content/tiles/` as `TilesetDef`
- `content/metadata/` as `SpriteSheetDef`

That worked before the editor grew sidecar files. Phase 25-27 added files such as:

- `content/tiles/base_tileset_roles.ron`
- `content/metadata/character_mannequins_phase25.ron`
- `content/metadata/character_bases_phase27.ron`

Those are valid editor metadata, but they are not runtime tilesets or sprite sheets.

## Change

`crates/game_data/src/lib.rs` now probes simple schema keys before loading runtime content:

- tilesets require `texture_path`, `columns`, `rows`, and `tiles`
- sprite sheets require `texture_path`, `columns`, `rows`, and `entries`

Non-matching files are skipped with an info log instead of crashing startup.

## Expected Result

After applying:

1. `cargo check` should still pass.
2. `RUN_EDITOR_DIAGNOSTIC.bat` should no longer fail on `base_tileset_roles.ron`.
3. Character editor sidecars should also stop crashing the metadata loader.
4. If the editor still closes, the next diagnostic log should reveal the next true runtime issue instead of this schema mismatch.
