# Editor Layers

Starlight Ridge maps are not flat tile arrays. They are stacks of semantic layers.

Visible layers:

1. Base Terrain
2. Terrain Overlay / Transition
3. Liquid Region
4. Ground State
5. Ground Cover
6. Props / Vegetation
7. Structures
8. Collision / Interaction
9. Spawns / Regions
10. Lighting / Ambience Markers

Derived layers:

11. Fluid Depth/Flow
12. Snow / Ice Deposition
13. Wetness / Puddle Accumulation
14. Autotile Resolution / Render Composite

Most direct painting happens on visible layers. Derived layers are normally rebuilt by simulation or resolver systems.
