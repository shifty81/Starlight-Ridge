# Phase 50 — Web / egui Tool Parity

## Goal

Bring the LAN web editor back in line with the native egui editor workflow so keybinds, brush selection, copy/paste transforms, and tablet/PC presentation modes use the same mental model.

## Added to the web companion

- Real document-level keybind handler.
- Tool rail matching the native editor vocabulary:
  - Pencil / Paint
  - Erase
  - Fill
  - Pick
  - Inspect
- Expandable brush drawer:
  - 1 tile pencil
  - square brush
  - circle brush
  - dither brush
  - line scaffold
  - stamp scaffold
- Brush size control.
- Copy / paste transform panel:
  - current tile
  - visible viewport
  - entire layer
  - mirror horizontal
  - mirror vertical
  - mirror both
  - rotate 90 / 180 / 270
- Internal map-layer clipboard with paste ghost preview.
- Tablet-friendly controls retain the same tools.

## Keybinds

| Key | Action |
|---|---|
| 1 / P | Paint |
| 2 / E | Erase |
| 3 / F | Fill |
| 4 / I | Pick |
| 5 / Q / Escape | Inspect |
| B | Toggle brush drawer |
| C / Ctrl+C | Copy selected copy scope |
| V / Ctrl+V | Paste internal clipboard |
| H | Mirror pasted content horizontally |
| J | Mirror pasted content vertically |
| R | Cycle paste rotation/transform |
| G | Toggle tile grid |
| [ / - | Zoom out |
| ] / + | Zoom in |
| Ctrl+S | Save layers when write mode is enabled |
| Ctrl+E | Export layers.ron |
| Ctrl+R | Reload map |

## Important limitation

This phase catches the web editor up to the egui workflow for **map-layer editing**. It does not yet mutate atlas PNG pixels. PNG-level editing still belongs to the native egui pixel editor until the shared image clipboard/buffer exists.


## Native egui parity update

The native egui shortcut map is also aligned with the web companion direction:

- `P` = Pencil
- `E` = Erase
- `F` = Fill
- `I` = Pick
- `Q` / `Escape` = Select / inspect-safe mode
- `B` = toggle brush drawer
- `C` / `Ctrl+C` = copy selected scope
- `V` / `Ctrl+V` = paste with transform
- `H` = paste mirror horizontal
- `J` = paste mirror vertical
- `R` = rotate paste transform
- `G` = toggle pixel grid
- `Ctrl+R` / `F5` = reload content

This keeps the web companion and native editor from drifting into different control schemes.
