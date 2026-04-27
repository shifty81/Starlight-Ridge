Starlight Ridge Phase 25 - Editor Workflow Upgrade

Drop this patch over the project root after Phase 24.

Replaces:
- tools/asset_lab.html
- tools/asset_lab_server.py

Adds:
- assets/textures/characters/base_male_underwear.png
- assets/textures/characters/base_female_underwear.png
- content/metadata/character_mannequins_phase25.ron
- docs/editor/EDITOR_LABS_PHASE25.md
- docs/PATCH_README_PHASE25_EDITOR_WORKFLOW_UPGRADE.md

Key changes:
- Removes the fake 3x3 world-style default preview from the main workflow.
- Adds Selected Tile Preview, Atlas Output Preview, and Seam Diagnostic panels.
- Prevents editor/display grid overlays from being committed into the atlas.
- Adds marquee select, copy/cut/paste, Ctrl shortcuts, undo/redo, color picker, and selection move with Ctrl-drag.
- Moves animation into its own Animation Lab page.
- Adds per-frame duration metadata and strip preview.
- Adds Character Lab v1 with male/female mannequin bases and non-destructive layer stack.

Run:
- RUN_ASSET_LAB.bat
or
- python tools/asset_lab_server.py

Then open:
http://127.0.0.1:8724/tools/asset_lab.html
