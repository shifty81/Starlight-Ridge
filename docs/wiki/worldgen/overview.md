# Worldgen Overview

Worldgen is a deterministic multi-pass pipeline.

It generates draft scene layers from:

- seed;
- scene template;
- biome/material rules;
- noise fields;
- hydrology;
- feature placement;
- authored protected overlays.

The editor previews the generated draft, validates it, and then bakes it into editable scene files.
