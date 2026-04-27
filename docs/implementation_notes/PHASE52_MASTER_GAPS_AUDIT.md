# Phase 52 Gaps Audit

## Current major gaps to close

### Data/contracts

- No unified biome contract.
- No unified material contract.
- No liquid/physics contract.
- No weather/snow contract.
- No layered map contract that supports simulation and editor stacks.
- No tooltip/help metadata contract.

### World generation

- Worldgen needs pass-based architecture.
- Biome generation must be seed-deterministic.
- Hydrology must run before material placement.
- Authored protected regions need generator-respect rules.
- Debug overlays need to be first-class.

### Editor

- Layer stack needs semantic layers, not just clickable tabs.
- Cell inspector needs full stack view.
- Validation should link to docs.
- Help/wiki should be available inside the editor.
- Tooltips must be attached to every field.

### Assets

- `.vox` source files need registry and bake metadata.
- Directional sprite outputs need to be generated and validated.
- Footprint/collision/interaction metadata needs authoring tools.
- Atlas/material contracts need to stop drifting apart.

### Simulation

- Liquids need depth/flow model.
- Rain needs wetness/puddling model.
- Snow needs accumulation/melt/freeze model.
- Materials need puddle/snow/drainage behavior.

### Camera/rendering

- Orbit-style 2D camera requires directional rendering.
- Directional selection and collision must be solved with the camera.
- Terrain can stay mostly invariant first, but props/structures need 4-way outputs.

## Highest-risk areas

1. Trying to implement fluids before map layers/material contracts.
2. Creating more atlas art before material roles are locked.
3. Adding `.vox` assets without a bake manifest and validation.
4. Adding camera rotation before directional asset/facing rules exist.
5. Building a separate Android app before the LAN web editor is stable.

## Strongest next step

Phase 52a should implement contracts, docs plumbing, and validation before rendering/simulation changes.
