# Voxel Scene Format

Scene maps keep the existing gameplay/control layer files and gain voxel scene files beside them.

Recommended scene folder shape:

```txt
content/scenes/<scene_id>/
  voxel_scene.ron
  voxel_objects.ron
  voxel_terrain.ron
```

`voxel_scene.ron` defines grid/density rules. `voxel_objects.ron` defines placed voxel assets. `voxel_terrain.ron` defines scene terrain chunks.

Gameplay map layers remain useful for soil state, watered state, collision hints, placement masks, interactions, pathfinding, biome IDs, season rules, and validation overlays.
