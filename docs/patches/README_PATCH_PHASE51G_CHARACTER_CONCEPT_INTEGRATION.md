# Phase 51G — Character Concept Source Integration

## Purpose

Integrates the uploaded human character concept PNGs into the Starlight Ridge source tree as tracked art-source references.

## Source merge

- Uses the latest source patch contents from `Starlight_Ridge_source_2026-04-27_07-31-41.zip`.
- Preserves the current egui/editor direction from the latest patch source.
- Adds uploaded concept art under `assets/art_source/character_concepts/`.

## Added files

- `assets/art_source/character_concepts/uploaded_human_concept_01.png`
- `assets/art_source/character_concepts/uploaded_human_concept_02.png`
- `assets/art_source/character_concepts/uploaded_human_concept_03.png`
- `assets/art_source/character_concepts/uploaded_human_concept_04.png`
- `assets/art_source/character_concepts/README.md`
- `content/editor_character/character_concepts_phase51g.ron`
- `docs/patches/README_PATCH_PHASE51G_CHARACTER_CONCEPT_INTEGRATION.md`
- `PATCH_MANIFEST_phase51g.txt`

## Runtime note

The uploaded concepts are integrated as source/reference assets only. They are intentionally not wired as runtime sprites yet because they are portrait/concept proportions, not tile-aligned gameplay spritesheets. The next step is converting them into normalized male/female base sprites and paperdoll overlays.
