# Patch README — Phase 44 Character Studio + Paperdoll Scaffold

Extract this patch over the project root.

## Changes

- Adds Character workspace subtabs:
  - Mannequin
  - Paperdoll Layers
  - Equipment / Overlays
  - Animation Preview
  - Scale Validator
  - Export / Metadata
- Adds left-side Character Studio controls.
- Reuses the shared pixel editor brush drawer in Character Studio.
- Adds a block-relative mannequin preview.
- Adds placeholder paperdoll layer stack and equipment slots.
- Adds an editor-only Character Studio contract.

## Build

Run:

```text
BUILD_MENU.bat
```

Then choose:

```text
2) cargo check
```

Cargo was unavailable in the container, so this patch still needs local verification.
