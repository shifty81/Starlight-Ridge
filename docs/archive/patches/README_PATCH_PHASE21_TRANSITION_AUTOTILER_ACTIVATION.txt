Starlight Ridge Phase 21 - Transition Autotiler Activation
=========================================================

Purpose
-------
This patch activates the existing transition helper path for legacy authored
terrain layers while keeping object/prop rendering separated from terrain.

Files replaced
--------------
- crates/app/src/lib.rs

What changed
------------
- Restores starter_farm as the normal launch map when present.
- Leaves autotile_test_coast available as a fallback diagnostic map.
- Keeps Phase 18 semantic terrain maps on the game_world::autotile resolver path.
- Re-enables a conservative legacy transition overlay pass for normal layers such
  as starter_farm.ground and town.ground.
- Uses neighbor checks to add transition overlays for:
  - grass/dirt into path
  - grass/dirt into tilled dry soil
  - tilled dry into tilled watered soil
  - grass into sand
  - sand into shallow water
  - shallow water into deep water
- Does not process layers whose id contains decor, prop, or object.
- Does not read or draw props.ron through the terrain pass.

Expected result
---------------
- cargo check should no longer warn that same_transition_group_at, TerrainGroup,
  transition_group, or transition_column are unused.
- Starter farm/town terrain should render extra transition overlay cells on
  shoreline/path/soil boundaries.
- Static props remain separate from terrain. This patch does not implement the
  second sprite-sheet prop spawning pass.

Notes
-----
Cargo is not available in the patch container, so this was source-checked only.
Run your build script after extracting over the project root. If a compiler error
appears, upload latest.log and the next patch can target the exact line.
