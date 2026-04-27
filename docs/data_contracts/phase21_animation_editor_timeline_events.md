# Phase 21 — Animation Editor Timeline, Events, Sockets, and Hitboxes

## Goal

Phase 21 turns animation from simple frame lists into gameplay-aware editor data. The editor can now describe frame timing, event markers, directional groups, tool sockets, hitboxes, water animation previews, seasonal animation variants, and validation reports.

## New content folder

```text
content/editor_animation/phase21_animation_editor_timeline_events.ron
```

This folder is intentionally separate from `content/editor_pipeline` and `content/editor_export` so each editor pipeline can keep a strict schema.

## Contract areas

```text
timeline_schemas
animation_clips
directional_groups
socket_profiles
hitbox_profiles
water_preview_profiles
seasonal_animation_sets
validation_reports
```

## Why this matters

Tool use, combat, water, props, effects, and character movement need timeline events instead of hardcoded guesses. A hoe swing can now mark the impact frame, sound frame, effect frame, lock/unlock frames, socket positions, and tile-action hitbox.

## Current scaffold coverage

The first phase21 contract includes:

```text
player 4-direction walk clips
player hoe-down timeline clip
seagull idle flap prop animation
impact puff effect animation
shallow water tile-loop animation
frame events for footsteps/tool impact/effects
per-frame hand/tool-tip sockets
hoe interaction/action hitboxes
water preview profiles
seasonal animation fallback sets
validation report definitions
```

## Next implementation after this patch

The next useful step is to make the renderer consume `animation_clips` for water and props, then expose a real timeline canvas in the web editor instead of static cards.
