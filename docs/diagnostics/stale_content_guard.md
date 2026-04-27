# Stale Content Guard / Runtime Startup Fix

## Problem found

The game/editor can compile successfully and still close immediately if an old support RON file remains in `content/tiles` after extracting a new source zip over an older folder.

The runtime log showed this exact failure:

```text
RUNTIME FAILURE: failed to parse RON content ...content\tiles\base_autotile_rules.ron: Unexpected missing field named `texture_path` in `TilesetDef`
```

`base_autotile_rules.ron` is not a tileset. The previous loader treated every `.ron` file in `content/tiles` as a `TilesetDef`, so stale rule files could crash startup before the window stayed open.

## Fix

`game_data::loader` now classifies files in `content/tiles` before parsing:

- real tileset files are loaded normally;
- stale rule/config files are skipped with a warning;
- files whose name clearly claims to be a tileset still fail loudly if malformed.

The build menu sanitize command also removes the known stale file:

```text
content/tiles/base_autotile_rules.ron
```

## Recommended use

After extracting this source zip over an older folder, run:

```text
10) Sanitize repo root
2) cargo check
5) Run game debug
6) Run editor debug
```

A completely fresh extraction also works and does not require the sanitize step.
