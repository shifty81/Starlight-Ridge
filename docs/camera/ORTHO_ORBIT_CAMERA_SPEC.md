# Orthographic Orbit Camera Spec

## Goal

The game remains 2D/pixel-art presented, but the camera behaves like an orthographic orbit camera with discrete world rotation.

## Initial implementation

Use:

- fixed orthographic projection;
- fixed pitch;
- 90-degree yaw steps;
- four canonical directions: north/east/south/west;
- directional sprite swapping for `.vox`-baked objects;
- tilemap coordinate transform for editor/game selection.

## Why not free rotation first?

Free rotation would require many more directional sprites, more complex sorting, more camera math, and much harder editor selection. Four-way orbit gives the gameplay/editor benefit while keeping the content pipeline achievable.

## Directional rendering requirements

For every rotatable object:

- north sprite;
- east sprite;
- south sprite;
- west sprite;
- matching shadow sprites or fallback;
- footprint rotation rules;
- collision rotation rules;
- interaction point rotation rules.

## Terrain handling

Terrain tiles can remain mostly camera-invariant initially. Props, structures, cliffs, fences, trees, machines, and large terrain features require directional sprites.

## Editor behavior

The editor needs:

- camera rotation buttons;
- current facing indicator;
- sprite preview by facing;
- selection footprint rotated to current view;
- option to inspect unrotated logical cell coordinates;
- validation warning for assets missing current facing.

## Runtime behavior

Camera yaw maps to facing like:

| Camera yaw | Asset facing used |
|---|---|
| 0° | south/front-facing world view or configured default |
| 90° | west/east depending coordinate convention |
| 180° | north/back view |
| 270° | opposite side |

Exact facing mapping must be locked when the bake tool is implemented.
