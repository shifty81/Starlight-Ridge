# Phase 51h — Mobile LAN Web Editor

This phase keeps the web editor inside the main Starlight Ridge repo and makes the LAN address the mobile-friendly entry point.

## Launch

Run the existing web editor server through the build menu:

- `18) Run web editor on LAN (read-only)`
- `19) Run web editor on LAN (save enabled)`

Then open the printed LAN address from the phone or tablet browser:

```text
http://<PC-LAN-IP>:8787/
```

The root address now opens the editor directly. It automatically chooses PC, tablet, or mobile layout from the viewport width.

Manual view URLs are still supported:

```text
http://<PC-LAN-IP>:8787/?mode=mobile
http://<PC-LAN-IP>:8787/?mode=tablet
http://<PC-LAN-IP>:8787/?mode=pc
http://<PC-LAN-IP>:8787/?launcher=1
```

## Mobile behavior

Mobile mode is map-first:

- Desktop side panels are hidden.
- A bottom command bar exposes Paint, Erase, Fill, Pick, Tools, and Save.
- The Tools drawer exposes map selection, layer selection, zoom, grid, quick palette, and layer visibility.
- One finger paints/inspects depending on the active tool.
- Pinch zooms the map.
- Two fingers pan the map.
- Grid, layer, and palette controls are touch-sized.

## Save behavior

Saving still depends on the LAN server mode.

Read-only launch:

```text
STARLIGHT_WEB_ALLOW_WRITE not set
```

Save-enabled launch:

```text
STARLIGHT_WEB_ALLOW_WRITE=1
```

The mobile Save button is disabled unless write mode is enabled by the server.

## Current scope

This phase is a mobile-friendly web shell for the tile/map editor. It does not split the editor into a separate APK project. That keeps one shared content path, one LAN server, and one save pipeline.
