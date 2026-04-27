# Phase 52 Roadmap

## Phase 52a — Core Contracts + Docs Foundation

Implement:

- Rust structs for biome/material/liquid/weather/season/map-layer/vox/help metadata.
- RON loading.
- Registry validation.
- Help metadata loading.
- Editor validation panel integration.

Expected result:

The project can load and validate the new contracts without changing rendering yet.

## Phase 52b — Layered World Model

Implement:

- 10 visible editor layers.
- 4 derived/runtime layers.
- Cell stack inspector.
- Layer visibility/lock/opacity controls.
- Map save/load upgrade path.

Expected result:

The editor understands the full world layer model.

## Phase 52c — VOX Asset Pipeline

Implement:

- `.vox` registry.
- `.vox` browser.
- parse status and metadata display.
- bake profile contract.
- initial 4-facing bake scaffold.
- generated output manifest.

Expected result:

`.vox` becomes a real source-asset path for world objects.

## Phase 52d — Biome/Material Atlas System

Implement:

- canonical 12 biome definitions.
- 5 grass material families.
- 5 sand/soil material families.
- 5 water families.
- lava and crude oil definitions.
- material thumbnails and validation.

Expected result:

Terrain generation and editor painting use semantic materials instead of loose tile ids.

## Phase 52e — WorldGen Core

Implement:

- elevation/moisture/temperature fields.
- biome assignment.
- hydrology routing.
- material family selection.
- debug overlays.
- draft/bake workflow.

Expected result:

The editor can generate and inspect biome/material draft maps.

## Phase 52f — Liquids + Weather Simulation

Implement:

- shallow liquid depth/flow.
- rain wetness and puddles.
- snow accumulation.
- melt/freeze/slush cycle.
- simulation debug overlays.

Expected result:

Weather and liquids start behaving physically but cheaply.

## Phase 52g — Orthographic Orbit Camera + Directional Rendering

Implement:

- 4-step camera rotation.
- directional sprite selection.
- rotated selection/footprint preview.
- camera-aware editor preview.

Expected result:

The world can be viewed from four sides using directional baked assets.

## Phase 52h — Editor Wiki + Full Help Integration

Implement:

- markdown help panel.
- hover tooltip plumbing.
- `?` context help buttons.
- validation messages with help links.
- missing-doc validation.

Expected result:

The editor becomes self-documenting.

## Phase 52i — Mobile LAN Editor Expansion

Implement:

- touch-first pixel editor mode.
- touch-first layer tools.
- mobile inspector/help mode.
- installable PWA metadata later.

Expected result:

Browsing to the LAN address gives a usable mobile/tablet editor.
