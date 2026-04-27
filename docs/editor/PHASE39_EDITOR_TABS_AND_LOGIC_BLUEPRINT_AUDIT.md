# Phase 39 — Editor Tabs, Atlas Import, and Logic Blueprint Audit

## Decision
The editor should move from one long toolbar plus ad-hoc side panels into a workspace tab model:

- Project
- World
- Assets
- Animation
- Character
- Logic
- Data
- Playtest
- Settings

Each workspace tab should own subtabs. This keeps the editor scalable as more tools are added and prevents the Asset Lab, World Preview, Animation Lab, Character Lab, and future Logic editor from competing for the same layout space.

## Atlas Compare / Import
Atlas Compare / Import should be its own Assets subtab/window, not a small section inside the preview inspector.

Required workflow:

1. Import source PNG or select an asset from project intake.
2. Detect or choose tile size.
3. Show source atlas and project atlas side by side.
4. Support drag, copy, paste, overwrite, insert, append, and mirror-aware paste.
5. Auto-expand the project atlas when appended tiles exceed current columns/rows.
6. Rewrite atlas metadata after expansion.
7. Preserve or prompt for tile roles, collision, seasonal variants, animation metadata, and object/prop classification.
8. Validate affected maps before final save.

Auto-expansion should write a safe backup first, then output the expanded PNG and matching RON metadata together. The editor must never update the PNG without updating the matching tileset data.

## Logic Blueprint System
The Logic tab should become the main gameplay behavior authoring system. It should handle game behavior through data-driven node graphs rather than hardcoded one-off Rust behavior.

Core event categories:

- OnInteract
- OnToolHit
- OnUseItem
- OnEnterTile
- OnExitTile
- OnCollision
- OnAnimationEvent
- OnDayStart
- OnSeasonChanged
- OnQuestStateChanged

Core condition categories:

- tool.type equals Axe, Hoe, Pickaxe, WateringCan, Sword, etc.
- tile.role equals CropSoil, Water, Tree, Rock, Door, etc.
- player has item
- item count greater than value
- season equals target season
- quest flag or world flag is set
- object health/damage state check

Core action categories:

- replace tile
- spawn prop
- remove prop
- damage object
- drop item
- add item
- consume item
- play animation
- play sound
- show dialogue
- start quest
- set flag
- teleport/change map

## Runtime Implication
The game needs a small runtime interpreter for compiled logic graphs. The editor should save graphs as data, validate them, and export a compact runtime-friendly version. Rust systems should expose safe actions to the interpreter instead of every item/tile/tool being hardcoded separately.

## GUI Implication
The egui native editor and web editor should share the same conceptual workspace model:

- same top-level workspaces
- same subtab names
- same content contracts
- same validation messages
- same asset/logic metadata

The web editor can remain useful for tablet/LAN editing, but it should become a companion interface using the same data contracts, not a separate editor design.
