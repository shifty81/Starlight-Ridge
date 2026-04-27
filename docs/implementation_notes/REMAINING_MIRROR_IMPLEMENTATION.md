# Remaining Mirror Implementation Notes

This file is intentionally carried forward in Phase 52 documentation zips so mirror-related editor work is not lost.

## Pixel editor mirror requirements

- Mirror X painting.
- Mirror Y painting.
- Combined mirror X/Y painting.
- Red/visible mirror guide lines.
- Mirror-aware brush shapes.
- Mirror-aware fill preview, if fill mirror mode is enabled.
- Mirror-aware line tool.
- Mirror-aware paste preview.
- Mirror-aware selection transform.

## Atlas/tile clipboard mirror requirements

- Mirror paste horizontally.
- Mirror paste vertically.
- Rotate paste 90/180/270.
- Preview transformed pasted tiles before commit.
- Preserve tile metadata or prompt before destructive overwrite.

## VOX/directional mirror requirements

- Optional mirror bake for symmetrical `.vox` assets.
- Warn when mirrored output is used as a fallback instead of a true facing.
- Do not silently mirror assets with asymmetrical interaction points.

## Validation

The editor should warn when mirror/facing behavior is incomplete for an active asset/tool.
