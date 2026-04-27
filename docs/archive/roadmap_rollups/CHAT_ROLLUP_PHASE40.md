# Chat Rollup — Phase 40

## User direction captured

- The web editor needs PC Mode and Tablet Mode.
- Tablet Mode should be mobile/touch friendly.
- LAN server integration should be first-class, not an afterthought.
- The native egui editor may wrap/control the web companion, but should not become a browser shell.
- Root scripts are out of control; consolidate into one master menu with self-heal behavior.
- Existing menu functionality must be retained.
- Text colors/theme clarity regressed and need restored through explicit theme tokens.
- The editor GUI needs to be reconsidered as one cohesive system across native egui and web companion.

## Architecture answer

Wrapping every editor surface inside egui as an embedded webview is not the best final structure. It would make the native editor depend on browser UI for core tools and would not solve the tablet use case.

The better structure is:

```text
editor.exe = full native egui editor
web editor = standalone LAN/tablet companion
web_editor_server = safe project bridge
editor_core = shared operations and validation
```

## Implemented scaffold in this phase

- Web editor PC/Tablet presentation switch.
- Touch-first tablet layout.
- Shared workspace tab strip scaffold in the web editor.
- LAN hint updated with `?mode=tablet` and `?mode=pc` URLs.
- Server manifest filters editor-only tileset sidecars from the tileset list.
- Server exposes `/api/server_info`.
- Build menu gains self-heal, patch doc organizer, and legacy root script consolidation actions.
- Documentation updated with the long-term egui/web split.
