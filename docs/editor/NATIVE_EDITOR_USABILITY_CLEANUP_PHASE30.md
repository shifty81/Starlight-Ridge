# Phase 30 Native Editor Usability Cleanup

Phase 29 proved the editor shell could render, but the UI was too noisy and did not respond to input. Phase 30 starts moving the native editor from static overlay to interactive workspace.

## Added Interaction

- Active tool state lives in the app runtime.
- Renderer receives `EditorShellRenderState` each time input changes.
- Toolbar icons can be selected by mouse click.
- Keyboard shortcuts change the active tool.
- F1/F2/F3 collapse and reopen the main docks.
- Hover text updates based on cursor area.
- Asset cards update the selected asset label.

## UI Cleanup

- The viewport is no longer covered by large tinted panel overlays.
- Bottom dock is smaller and collapsible.
- Text density is reduced.
- Toolbar is icon-first with a compact active-tool badge.
- Side panels are simplified into clear Assets and Inspector columns.

## Known Remaining Work

The native editor still needs the real editing operations:

- atlas-cell picking from the rendered atlas
- native pixel grid editing
- role/collision sidecar editing
- save/write behavior
- proper panel scrolling
- proper text/font renderer
- full docking/resizing
- native Character Lab workflow
