# Phase 13 Playability Gap Fix

This pass is a stabilization/playability step after the first visible map launch.

## What was wrong

The runtime was opening the starter map, but the app was still only drawing the map layer. The content registry loaded a sprite sheet and map spawns, but the active renderer was not drawing the player sprite and the input path had no interaction key behavior.

The terrain transition overlay pass was also using guessed atlas columns. That made the map visibly worse because the packed terrain atlas does not yet expose named transition roles for every edge/corner case.

## Changes

- Added a sprite render path to `engine_render_gl`.
- Loaded `phase5_entities` / `player_walk.png` as the active player sprite sheet.
- Spawned the player from `content/maps/<map>/spawns.ron`.
- Added WASD / arrow movement in the active app loop.
- Added E / Space interaction logging for nearby props and trigger zones.
- Changed the terrain renderer to safe named-tile mode.
- Disabled guessed transition overlays until the tileset contract has explicit names such as `water_edge_n`, `path_corner_sw`, and `tilled_inner_ne`.

## Controls

- Move: WASD or arrow keys.
- Interact: E or Space.
- Exit: Escape or close window.

## Remaining real gaps

- Interactions currently log to `logs/runtime_latest.log`; there is no in-game dialogue bubble or UI prompt yet.
- Props and triggers are still data-driven but not yet drawn as full object sprites.
- The editor still shares the runtime camera/render path; it does not yet expose real tile painting or selection tools.
- A true auto-tiler still needs named tile-role metadata or a generated 47-tile/Wang-style transition atlas.
