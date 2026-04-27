# Phase 52A Validation Checklist

Use this checklist before moving into Phase 52B.

## Content IDs

- [ ] All biome IDs are unique.
- [ ] All material IDs are unique.
- [ ] All liquid IDs are unique.
- [ ] All map layer IDs are unique.
- [ ] Worldgen preset pass IDs are unique per preset.
- [ ] `.vox` asset IDs are unique.
- [ ] Help entry IDs are unique.

## Cross references

- [ ] Biome default grass IDs reference known materials.
- [ ] Biome default sand IDs reference known materials.
- [ ] Biome default water IDs reference known liquids.
- [ ] Biome weather profiles exist.
- [ ] Weather accumulation layers exist or use `none`.
- [ ] Worldgen passes only write known layers.
- [ ] VOX assets reference known bake profiles.
- [ ] Help entries point to existing wiki pages.

## Editor readiness

- [ ] Every editor-visible field has a tooltip target.
- [ ] Every new content folder has a README.
- [ ] Derived layers are not directly editable.
- [ ] Authored layers are marked saved-to-map where needed.
- [ ] Validation messages can link to help entries.

## Runtime readiness before Phase 52C+

- [ ] Layer schema can serialize into map files.
- [ ] Runtime can distinguish authored layers from derived layers.
- [ ] Weather simulation can write wetness/snow layers without modifying authored terrain.
- [ ] VOX bake outputs can be referenced by normal asset metadata.
