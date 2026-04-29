# VOX Assets

Place MagicaVoxel `.vox` model files here.

Phase 51f scans this folder from the egui editor under Assets -> VOX Models.

Current support:
- parse MagicaVoxel `VOX ` files
- read SIZE, XYZI, and RGBA chunks
- list model dimensions, voxel counts, and palette size
- safe editor reload support

Next support target:
- orthographic bake to 2D prop/tile sprites
- generated preview thumbnails
- collision footprint extraction
- placement metadata generation
