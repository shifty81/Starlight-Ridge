# Chat Rollup — Phase 50

The web editor keybinds and brush changes were not carrying over from the native egui editor work. Phase 50 updates the LAN/tablet companion so it shares the same tool vocabulary and workflow direction.

Implemented:

- Web editor keybinds.
- Brush drawer in the web editor.
- Brush size and dither toggle.
- Fill tool.
- Map-layer internal clipboard.
- Copy current tile / visible viewport / entire layer.
- Paste transforms: mirror H, mirror V, mirror both, rotate 90/180/270.
- Paste preview rectangle.
- Tablet mode controls still use the same tool model.

Next recommended phase:

`Phase 51 — Shared editor action registry`

Focus:

- Define editor actions once in shared content/contracts.
- Have egui and web companion read or mirror that action registry.
- Add command palette/search.
- Add customizable keybinds later.

Also aligned the native egui shortcut handler so the same core keys mean the same thing across desktop and web companion.
