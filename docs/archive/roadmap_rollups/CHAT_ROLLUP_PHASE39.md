# Chat Rollup — Phase 39

## User direction captured

- Atlas Compare / Import should auto-expand the project tileset when new imported tiles do not fit.
- Atlas Compare / Import should be its own tab or window, not a small preview panel.
- The tab system needs to be reworked now because more editor modules will be added.
- Top-level tabs should support subtabs per editor domain.
- A blueprint-style Logic editor is required for scripting gameplay behavior.
- Logic graphs should drive interactions such as interacting with blocks, swinging tools at blocks, checking tool type, and tying behavior directly to tools, tiles, props, items, and other game systems.
- The egui editor and web editor need to be rethought together so the UI model stays cohesive.

## Phase 39 implementation scope

- Add workspace-level tabs to the egui editor scaffold.
- Add subtabs for World, Assets, and Logic.
- Document the atlas compare/import auto-expansion workflow.
- Document the blueprint-style logic editor architecture and runtime implications.
- Preserve the existing world preview and asset panels while preparing the editor for a larger modular tab layout.

## Recommended Phase 40

`Starlight_Ridge_phase40_atlas_compare_import_window.zip`

Focus:

- Real egui Atlas Compare / Import tab.
- Source atlas + project atlas side-by-side views.
- Import queue.
- Append/overwrite modes.
- Automatic atlas expansion.
- PNG backup before write.
- RON metadata rewrite.
- Validation report before save.

## Recommended Phase 41

`Starlight_Ridge_phase41_logic_blueprint_contracts.zip`

Focus:

- Serializable logic graph schema.
- Event/condition/action node catalog.
- Tool behavior graph contracts.
- Tile/block behavior graph contracts.
- Prop/object behavior graph contracts.
- Runtime interpreter scaffold.
- egui Logic tab scaffold.
- Web editor matching Logic tab scaffold.
