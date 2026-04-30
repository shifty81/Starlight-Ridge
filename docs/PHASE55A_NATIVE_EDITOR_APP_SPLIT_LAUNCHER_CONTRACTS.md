# Phase 55a — Native Editor App Split Manifest + Launcher Contracts

## Goal

Create the safe bridge between the current monolithic egui editor and the planned focused native editor applications.

This phase does **not** extract tool internals yet. It gives each planned editor its own binary entry point and manifest route so the project can be split without duplicating unstable code or risking the nested-egui-shell regression.

## Implemented

- Added `NativeEditorRoute` to the app runtime.
- Added `app::run_native_editor_app(route_id)`.
- Added focused binary entry points:
  - `character_editor.exe`
  - `voxel_panel_editor.exe`
  - `world_editor.exe`
  - `pixel_atlas_editor.exe`
  - `animation_rig_editor.exe`
  - `asset_import_editor.exe`
- Kept `editor.exe` as the hub/default route.
- Added manifest-driven native launcher UI under:
  - `Project -> Native Launchers`
- Updated `content/editor_tools/native_editor_apps.ron` with route, executable, status, surface, and launch notes.
- Added root debug launch scripts for each focused editor binary.

## Current behavior

Each focused editor opens the same stable egui shell but starts on the intended workspace:

| Binary | Initial route |
| --- | --- |
| `editor.exe` | `Project / Native Launchers` |
| `character_editor.exe` | `Character / Preview` |
| `voxel_panel_editor.exe` | `Assets / Voxel Panels` |
| `world_editor.exe` | `World / 3D Preview` |
| `pixel_atlas_editor.exe` | `Assets / Pixel Editor` |
| `animation_rig_editor.exe` | `Animation / Sockets` |
| `asset_import_editor.exe` | `Assets / VOX Models` |

## Why this phase exists

The editor has a lot of working but tangled egui code. Jumping straight into extraction would risk moving broken placeholders into standalone apps. This phase creates launchable shells first, then later phases can move actual code behind those routes one tool at a time.

## Nested shell safety rule

Do not call the full top-level editor shell from inside a focused editor workspace. Focused binaries should set their initial route and then render through the same single root egui shell.

## Next phase

Phase 55b should extract the Character Editor first:

- move character preview/VOX assembly state behind a dedicated module
- add modular VOX character part loading
- add character part registry validation
- keep `character_editor.exe` as the focused entry point
