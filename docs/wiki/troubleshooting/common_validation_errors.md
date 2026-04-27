# Common Validation Errors

## Missing material id

A tile references a material that is not defined in the material registry.

Fix: add the material definition or replace the tile material id.

## Missing directional sprite

A `.vox` asset or directional object is missing one or more required facing sprites.

Fix: rebake the asset or provide a fallback.

## Invalid liquid/material pair

A liquid appears on a material that does not allow it.

Fix: update the liquid `allowed_materials`, change the material, or remove the liquid.

## Missing help metadata

An editor field has no tooltip or help page.

Fix: add the field to the tooltip/help registry.
