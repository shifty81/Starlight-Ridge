# Phase 40 — Web Companion Modes, LAN Integration, Master Script, and egui Wrap Plan

## Decision

The web editor should stay a standalone LAN/tablet companion, but it should be governed by the same editor architecture as `editor.exe`.

`editor.exe` should become the complete native egui desktop editor. The web editor should be the mobile-friendly companion view served by the local LAN server. The two surfaces should share data contracts, naming, tabs, validation rules, atlas operations, logic graph contracts, and save paths.

## Why not fully absorb the web editor into egui?

Embedding the whole web editor inside egui would make the native editor depend on a browser/webview stack and would duplicate UI paths. It would also make tablet use worse because the tablet still needs the LAN web surface.

The better model is:

```text
editor.exe
  Native egui desktop editor
  Full authoring workflow
  Starts/stops or monitors the web companion server
  Displays LAN URLs and write-mode status

web_editor_server
  Local/LAN server
  Serves the browser companion
  Performs safe project file operations

web editor
  PC Mode for desktop browser use
  Tablet Mode for touch/mobile use
  Uses the same project contracts as editor.exe
```

## Replacement rule

The old custom/native editor UI should be replaced by egui. The web editor should not replace egui; it should replace ad-hoc browser-only tools and act as a companion surface.

## Web editor presentation modes

The web editor now has two presentation modes:

- **PC Mode**: three-column layout with controls, canvas, and raw file panel.
- **Tablet Mode**: touch-first layout with the canvas moved to the top, larger controls, quick action buttons, and raw text hidden by default.

URLs:

```text
http://<pc-lan-ip>:8787/?mode=pc
http://<pc-lan-ip>:8787/?mode=tablet
```

## LAN integration contract

The web editor server should expose:

```text
/api/health
/api/server_info
/api/manifest
/api/map/<map_id>
/api/save/map/<map_id>/layers.ron
```

This phase adds `/api/server_info` and mode URLs in `/api/manifest`.

Future server endpoints should be grouped by editor operation, not by raw file access:

```text
/api/editor/atlas/import
/api/editor/atlas/expand
/api/editor/atlas/save
/api/editor/map/paint
/api/editor/map/validate
/api/editor/logic/graph/save
/api/editor/logic/validate
```

## Master script direction

Root scripts are getting too numerous. The project should keep only tiny launchers in root:

```text
BUILD_MENU.bat
build.bat
BUILD_MENU_GIT_BASH.bat
build.sh
```

Everything else should move under:

```text
tools/legacy_launchers/
tools/build_menu.ps1
```

The master menu now owns:

- check tools
- cargo check
- debug/release builds
- game/editor launch
- web editor LAN launch
- web editor save-enabled LAN launch
- diagnostics bundle
- source zip
- root sanitization
- patch docs organization
- root script consolidation
- self-heal pass

## Text color/theme direction

Both egui and web editor should use a named theme token set:

```text
background
panel
panelStrong
line
lineBright
text
textStrong
muted
accent
accentSoft
good
warn
danger
```

No page should use default uncolored browser text. Headings, field labels, status text, warnings, and disabled states should use explicit tokens.

## Next implementation phases

### Phase 41 — egui Web Companion Control Panel

Add a native egui panel under Settings or Playtest:

```text
Web Companion
  server status
  PC URL
  Tablet URL
  read-only/write mode toggle
  launch server button
  copy URLs
  firewall/help notes
```

### Phase 42 — Atlas Compare / Import Window

Make Atlas Compare / Import a full Assets subtab/window with automatic atlas expansion.

### Phase 43 — Logic Blueprint Contracts

Add serializable graph definitions for tools, blocks/tiles, props/objects, interactions, day ticks, and event bindings.
