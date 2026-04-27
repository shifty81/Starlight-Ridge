# Web World Editor Phase 33

## Intended workflow

The browser editor is now the fast-iteration source of truth for editor UX.

The target loop is:

```text
World Preview
  -> right-click tile
  -> inspect / replace / open source tile
  -> edit in Asset Lab
  -> save atlas or map
  -> hot reload native runtime
```

## Added API endpoints

- `GET /api/maps`
- `GET /api/map_layers?map=starter_farm`
- `POST /api/save_map_layers`
- `POST /api/hot_reload_manifest`

## Notes

Map saving rewrites `content/maps/<map>/layers.ron` in normalized RON form and creates a timestamped `.bak.ron` file first.

The native editor is intentionally not advanced in this phase. Once this browser workflow feels right, the native Asset Studio should copy this layout and behavior.
