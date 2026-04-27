# Editor Overview

The Starlight Ridge editor is a layered world editor, asset editor, pixel editor, and generation workbench.

The editor is responsible for:

- editing scene layers;
- previewing and baking world generation;
- validating content;
- editing terrain/material metadata;
- managing `.vox` source assets and baked sprites;
- editing atlas/pixel content;
- exposing help and troubleshooting information.

Core principle:

```text
The editor should explain every field it exposes.
```
