# Starlight Ridge Phase 21 — Animation Editor Timeline Events

## Patch name

`Starlight_Ridge_phase21_animation_editor_timeline_events.zip`

## Adds

```text
content/editor_animation/phase21_animation_editor_timeline_events.ron
crates/editor_core/src/animation_pipeline.rs
docs/data_contracts/phase21_animation_editor_timeline_events.md
```

## Updates

```text
crates/game_data/src/defs.rs
crates/game_data/src/loader.rs
crates/game_data/src/lib.rs
crates/game_data/src/registry.rs
crates/game_data/src/validate.rs
crates/editor_core/src/lib.rs
tools/web_editor_atlas_pipeline/index.html
tools/web_editor_atlas_pipeline/app.js
tools/web_editor_atlas_pipeline/styles.css
```

## Focus

```text
animation timeline schema
frame event metadata
directional animation groups
tool socket metadata
hitbox / interaction box metadata
water animation preview contract
seasonal animation variants
animation validation report
web editor animation panel scaffold
```

## Expected result

Animations stop being simple frame lists. The repo now has a typed animation editor pipeline contract that can validate frame references, gameplay events, directional sets, sockets, hitboxes, water previews, and seasonal clip fallbacks.

## Install

Extract over the project root and overwrite existing files. Then run the root build script.

## Compile note

This package was source-checked in the packaging container, but Cargo/rustc were unavailable there. Run your local build script and upload the latest log if a compile error appears.
