# VOX Pipeline

`.vox` files are source assets. The game uses baked pixel sprites and metadata.

Minimum output for a directional object:

- north sprite
- east sprite
- south sprite
- west sprite
- shadow sprites or fallback
- footprint mask
- collision mask if blocking
- thumbnail
- bake metadata

The editor should warn when a `.vox` source has stale or missing baked outputs.
