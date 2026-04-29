# Starlight Ridge Phase 21 Web Editor LAN Bridge

This patch adds the first LAN-usable web editor surface for tablets and other devices on the same local network.

## What was added

- `crates/web_editor_server/`
  - Small Rust HTTP server with no new third-party dependencies.
  - Binds to `0.0.0.0` by default for LAN access.
  - Prints the tablet URL on startup.
  - Serves `/tools/web_editor` as the browser app.
  - Serves project `/assets` and `/content` paths for preview.
  - Exposes read APIs for maps and tilesets.
  - Supports optional writes to `content/maps/<map>/layers.ron`.

- `tools/web_editor/`
  - Tablet-friendly HTML/CSS/JS frontend.
  - Map selector.
  - Tile atlas rendering from current RON metadata.
  - Layer visibility toggles.
  - Touch/click tile inspector.
  - Touch/click painting scaffold.
  - Palette based on the selected layer legend.
  - Grid toggle.
  - Export edited `layers.ron`.
  - Optional save back into the repo when write mode is enabled.

- Root launchers:
  - `RUN_WEB_EDITOR_LAN.bat`
  - `RUN_WEB_EDITOR_LAN_WRITE.bat`

- Build menu additions:
  - Bash `build.sh` options for read-only and write-enabled web editor launch.
  - PowerShell `tools/build_menu.ps1` options for read-only and write-enabled web editor launch.

## How to use from a tablet

1. Run `RUN_WEB_EDITOR_LAN.bat` from the project root.
2. Wait for the server to print a URL similar to:

   ```text
   http://192.168.1.25:8787/
   ```

3. Open that URL on the tablet browser.
4. Keep the server window open while using the tablet.

Both devices must be on the same local network.

## Save mode

The default launcher is read-only. It can inspect, paint in browser memory, and export a new `layers.ron`, but it does not write to the repo.

To allow browser saves directly into the repo, run:

```text
RUN_WEB_EDITOR_LAN_WRITE.bat
```

or set:

```text
STARLIGHT_WEB_ALLOW_WRITE=1
```

before launching `cargo run -p web_editor_server`.

When save mode writes `layers.ron`, it first creates:

```text
content/maps/<map>/layers.ron.web_backup
```

## Firewall note

If the tablet cannot connect, Windows Firewall probably blocked the local server. Allow private network access for the console app, or open TCP port `8787` for the private network profile.

## Current limitations

This is not the final full editor. It is the first practical LAN bridge.

Current limits:

- Paints raw layer symbols, not semantic terrain IDs yet.
- Saves only `layers.ron`.
- Does not yet edit props, spawns, collisions, triggers, terrain rules, or animation data.
- Does not yet include user accounts/authentication, so write mode should only be used on a trusted home network.
- RON parsing in the browser is intentionally lightweight and expects the current project formatting.

## Next editor upgrades after this

- Semantic terrain paint mode.
- Autotile preview after paint.
- External tileset clipboard import.
- Mirror-aware paste.
- Prop placement and metadata editing.
- Validation panel in browser.
- Export pack builder.
