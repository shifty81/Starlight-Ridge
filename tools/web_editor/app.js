const state = {
  manifest: null,
  currentMapId: null,
  bundle: null,
  mapInfo: null,
  tileset: null,
  layers: [],
  visibleLayers: new Set(),
  selectedLayerId: null,
  selectedSymbol: null,
  mode: 'paint',
  dirty: false,
  tilesetImage: null,
  zoom: 2,
  presentationMode: localStorage.getItem('starlight.web.presentationMode') || 'auto',
  workspace: 'world',
  mobileToolsOpen: false,
  activePointers: new Map(),
  touchGesture: null,
  lastPaintTileKey: null,
};

const $ = (id) => document.getElementById(id);
const canvas = $('mapCanvas');
const ctx = canvas.getContext('2d');
ctx.imageSmoothingEnabled = false;

function setStatus(text, good = true) {
  const el = $('serverStatus');
  el.textContent = text;
  el.className = good ? 'statusGood' : 'statusBad';
}

function resolvedPresentationMode(mode) {
  if (mode === 'pc' || mode === 'tablet' || mode === 'mobile') return mode;
  if (window.innerWidth <= 720) return 'mobile';
  if (window.innerWidth <= 1100) return 'tablet';
  return 'pc';
}

function preferredPresentationMode() {
  const query = new URLSearchParams(location.search);
  const explicit = query.get('mode');
  if (explicit === 'pc' || explicit === 'tablet' || explicit === 'mobile') return explicit;
  return resolvedPresentationMode(state.presentationMode);
}

function applyPresentationMode(mode, persist = true) {
  const resolved = resolvedPresentationMode(mode);
  state.presentationMode = resolved;
  if (persist) localStorage.setItem('starlight.web.presentationMode', resolved);
  document.body.classList.toggle('presentation-tablet', resolved === 'tablet');
  document.body.classList.toggle('presentation-pc', resolved === 'pc');
  document.body.classList.toggle('presentation-mobile', resolved === 'mobile');
  $('pcModeButton')?.classList.toggle('active', resolved === 'pc');
  $('tabletModeButton')?.classList.toggle('active', resolved === 'tablet');
  $('mobileModeButton')?.classList.toggle('active', resolved === 'mobile');
  if (resolved !== 'mobile') setMobileToolsOpen(false);
  updateMobileControls();
  renderMap();
}

function shouldShowLauncher() {
  const query = new URLSearchParams(location.search);
  // Phase 51h: browsing to the LAN address opens the editor immediately.
  // The launcher remains available from the View button or ?launcher=1.
  if (query.get('launcher') === '1') return true;
  return false;
}

function showLauncher() {
  const launcher = $('entranceLauncher');
  if (!launcher) return;
  launcher.hidden = false;
  document.body.classList.add('launcherActive');
}

function hideLauncher() {
  const launcher = $('entranceLauncher');
  if (!launcher) return;
  launcher.hidden = true;
  document.body.classList.remove('launcherActive');
}

function launchWithMode(mode, remember = true) {
  applyPresentationMode(mode, remember);
  if (remember) localStorage.setItem('starlight.web.launcherSeen', '1');
  hideLauncher();
}

function setWorkspace(workspace) {
  state.workspace = workspace;
  for (const button of document.querySelectorAll('.workspaceTabs button')) {
    button.classList.toggle('active', button.dataset.workspace === workspace);
  }
  if (workspace !== 'world') {
    setStatus(`${workspace} workspace scaffold selected — full editing tools are native egui-first in this phase.`, true);
  } else if (state.manifest) {
    setStatus(`${location.origin} — ${state.manifest.write_enabled ? 'repo save enabled' : 'read-only LAN mode'}`);
  }
}

function setToolMode(mode, paintEnabled = null) {
  state.mode = mode;
  if (paintEnabled !== null && $('paintToggle')) $('paintToggle').checked = paintEnabled;
  buildPalette();

  const labels = {
    inspect: 'Inspect/select mode',
    paint: 'Brush paint mode',
    erase: 'Erase mode',
    fill: 'Fill mode',
    eyedrop: 'Eyedropper mode',
  };
  for (const button of document.querySelectorAll('[data-tool]')) {
    button.classList.toggle('active', button.dataset.tool === mode);
  }
  updateMobileControls();
  setStatus(`${labels[mode] ?? mode}. Shortcuts: V inspect, B brush, E erase, I eyedrop, G grid, Ctrl+S save.`, true);
}

async function apiJson(path, options = {}) {
  const response = await fetch(path, { cache: 'no-store', ...options });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`${response.status} ${response.statusText}: ${text}`);
  }
  return await response.json();
}

async function loadManifest() {
  state.manifest = await apiJson('/api/manifest');
  const select = $('mapSelect');
  const mobileSelect = $('mobileMapSelect');
  select.innerHTML = '';
  if (mobileSelect) mobileSelect.innerHTML = '';
  for (const mapId of state.manifest.maps) {
    const option = document.createElement('option');
    option.value = mapId;
    option.textContent = mapId;
    select.appendChild(option);

    if (mobileSelect) {
      const mobileOption = document.createElement('option');
      mobileOption.value = mapId;
      mobileOption.textContent = mapId;
      mobileSelect.appendChild(mobileOption);
    }
  }
  const preferred = state.manifest.maps.includes('starter_farm') ? 'starter_farm' : state.manifest.maps[0];
  select.value = preferred;
  if (mobileSelect) mobileSelect.value = preferred;
  $('saveButton').disabled = !state.manifest.write_enabled;
  if ($('mobileSaveButton')) $('mobileSaveButton').disabled = !state.manifest.write_enabled;
  setStatus(`${location.origin} — ${state.manifest.write_enabled ? 'repo save enabled' : 'read-only LAN mode'}`);
  const hint = $('lanHint');
  if (hint) hint.textContent = `Open ${location.origin}/ from your phone/tablet. The editor automatically picks mobile, tablet, or PC layout. Use ?launcher=1 to pick manually.`;
  await loadMap(preferred);
}

async function loadMap(mapId) {
  state.currentMapId = mapId;
  state.bundle = await apiJson(`/api/map/${encodeURIComponent(mapId)}`);
  state.mapInfo = parseMapRon(state.bundle.map_ron);
  state.layers = parseLayersRon(state.bundle.layers_ron);
  state.tileset = parseTilesetRon(state.bundle.tileset_ron);
  state.visibleLayers = new Set(state.layers.filter(layer => layer.visible).map(layer => layer.id));
  state.selectedLayerId = state.layers[0]?.id ?? null;
  state.selectedSymbol = firstPaintableSymbol();
  state.dirty = false;
  await loadTilesetImage();
  buildLayerControls();
  buildPaintLayerSelect();
  buildPalette();
  buildMobileControls();
  buildFileTabs();
  renderRaw('layers.ron');
  renderMap();
}

function parseMapRon(text) {
  return {
    id: matchString(text, /id:\s*"([^"]+)"/) ?? 'unknown',
    displayName: matchString(text, /display_name:\s*"([^"]+)"/) ?? 'Untitled Map',
    width: matchNumber(text, /width:\s*(\d+)/) ?? 0,
    height: matchNumber(text, /height:\s*(\d+)/) ?? 0,
    tileset: matchString(text, /tileset:\s*"([^"]+)"/) ?? 'base_tiles',
  };
}

function parseTilesetRon(text) {
  const tileWidth = matchNumber(text, /tile_width:\s*(\d+)/) ?? 32;
  const tileHeight = matchNumber(text, /tile_height:\s*(\d+)/) ?? 32;
  const columns = matchNumber(text, /columns:\s*(\d+)/) ?? 1;
  const rows = matchNumber(text, /rows:\s*(\d+)/) ?? 1;
  const texturePath = matchString(text, /texture_path:\s*"([^"]+)"/) ?? '';
  const namedTiles = new Map();
  const tileRegex = /\(id:\s*"([^"]+)",\s*x:\s*(\d+),\s*y:\s*(\d+)\)/g;
  let match;
  while ((match = tileRegex.exec(text))) {
    namedTiles.set(match[1], { id: match[1], x: Number(match[2]), y: Number(match[3]) });
  }
  return { tileWidth, tileHeight, columns, rows, texturePath, namedTiles };
}

function parseLayersRon(text) {
  const layers = [];
  const layerRegex = /id:\s*"([^"]+)",\s*visible:\s*(true|false),\s*legend:\s*\[([\s\S]*?)\],\s*rows:\s*\[([\s\S]*?)\],/g;
  let match;
  while ((match = layerRegex.exec(text))) {
    const legend = [];
    const legendBySymbol = new Map();
    const legendRegex = /\(symbol:\s*"([^"]+)",\s*tile_id:\s*"([^"]+)"\)/g;
    let legendMatch;
    while ((legendMatch = legendRegex.exec(match[3]))) {
      const entry = { symbol: legendMatch[1], tileId: legendMatch[2] };
      legend.push(entry);
      legendBySymbol.set(entry.symbol, entry.tileId);
    }
    const rows = [];
    const rowRegex = /"([^"]*)"/g;
    let rowMatch;
    while ((rowMatch = rowRegex.exec(match[4]))) rows.push(rowMatch[1]);
    layers.push({ id: match[1], visible: match[2] === 'true', legend, legendBySymbol, rows });
  }
  return layers;
}

function matchString(text, regex) {
  const match = regex.exec(text);
  return match ? match[1] : null;
}

function matchNumber(text, regex) {
  const value = matchString(text, regex);
  return value == null ? null : Number(value);
}

async function loadTilesetImage() {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => { state.tilesetImage = image; resolve(); };
    image.onerror = () => reject(new Error(`Failed to load tileset image: ${state.tileset.texturePath}`));
    image.src = `/${state.tileset.texturePath}`;
  });
}

function buildLayerControls() {
  const list = $('layerList');
  list.innerHTML = '';
  for (const layer of state.layers) {
    const row = document.createElement('div');
    row.className = 'layerRow';
    const label = document.createElement('label');
    const checkbox = document.createElement('input');
    checkbox.type = 'checkbox';
    checkbox.checked = state.visibleLayers.has(layer.id);
    checkbox.addEventListener('change', () => {
      if (checkbox.checked) state.visibleLayers.add(layer.id);
      else state.visibleLayers.delete(layer.id);
      renderMap();
    });
    const text = document.createElement('span');
    text.textContent = layer.id;
    label.append(checkbox, text);
    row.append(label);
    list.append(row);
  }
}

function buildPaintLayerSelect() {
  const select = $('paintLayerSelect');
  const mobileSelect = $('mobileLayerSelect');
  select.innerHTML = '';
  if (mobileSelect) mobileSelect.innerHTML = '';
  for (const layer of state.layers) {
    const option = document.createElement('option');
    option.value = layer.id;
    option.textContent = layer.id;
    select.appendChild(option);

    if (mobileSelect) {
      const mobileOption = document.createElement('option');
      mobileOption.value = layer.id;
      mobileOption.textContent = layer.id;
      mobileSelect.appendChild(mobileOption);
    }
  }
  select.value = state.selectedLayerId ?? '';
  if (mobileSelect) mobileSelect.value = state.selectedLayerId ?? '';
}

function buildPalette() {
  const layer = selectedLayer();
  const palette = $('palette');
  palette.innerHTML = '';
  if (!layer) return;
  for (const entry of layer.legend) {
    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = `${entry.symbol} ${entry.tileId}`;
    button.title = entry.tileId;
    if (state.selectedSymbol === entry.symbol) button.classList.add('active');
    button.addEventListener('click', () => {
      state.selectedSymbol = entry.symbol;
      state.mode = 'paint';
      buildPalette();
    });
    palette.appendChild(button);
  }
  buildMobilePalette();
}

function buildFileTabs() {
  const tabs = $('fileTabs');
  tabs.innerHTML = '';
  const files = ['layers.ron', 'map.ron', 'props.ron', 'spawns.ron', 'triggers.ron', state.bundle.tileset_file];
  for (const name of files) {
    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = name;
    button.addEventListener('click', () => renderRaw(name));
    tabs.appendChild(button);
  }
}

function buildMobileControls() {
  buildMobilePalette();
  buildMobileLayerStrip();
  updateMobileControls();
}

function buildMobilePalette() {
  const strip = $('mobilePaletteStrip');
  if (!strip) return;
  strip.innerHTML = '';
  const layer = selectedLayer();
  if (!layer) return;
  for (const entry of layer.legend) {
    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = `${entry.symbol} ${entry.tileId}`;
    button.title = entry.tileId;
    button.classList.toggle('active', state.selectedSymbol === entry.symbol);
    button.addEventListener('click', () => {
      state.selectedSymbol = entry.symbol;
      setToolMode('paint', true);
      buildPalette();
      updateMobileControls();
    });
    strip.appendChild(button);
  }
}

function buildMobileLayerStrip() {
  const strip = $('mobileLayerStrip');
  if (!strip) return;
  strip.innerHTML = '';
  for (const layer of state.layers) {
    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = `${state.visibleLayers.has(layer.id) ? '✓' : '○'} ${layer.id}`;
    button.classList.toggle('active', state.visibleLayers.has(layer.id));
    button.addEventListener('click', () => {
      if (state.visibleLayers.has(layer.id)) state.visibleLayers.delete(layer.id);
      else state.visibleLayers.add(layer.id);
      buildLayerControls();
      buildMobileLayerStrip();
      renderMap();
    });
    strip.appendChild(button);
  }
}

function updateMobileControls() {
  const summary = $('mobileToolSummary');
  if (summary) summary.textContent = `${state.mode} · zoom ${state.zoom}× · ${state.selectedLayerId ?? 'no layer'}`;
  if ($('mobileZoomValue')) $('mobileZoomValue').textContent = `${Number(state.zoom).toFixed(state.zoom % 1 === 0 ? 0 : 2)}×`;
  if ($('mobileMapSelect') && state.currentMapId) $('mobileMapSelect').value = state.currentMapId;
  if ($('mobileLayerSelect') && state.selectedLayerId) $('mobileLayerSelect').value = state.selectedLayerId;
  if ($('mobileSaveButton') && state.manifest) $('mobileSaveButton').disabled = !state.manifest.write_enabled;
  for (const button of document.querySelectorAll('[data-mobile-tool]')) {
    button.classList.toggle('active', button.dataset.mobileTool === state.mode);
  }
  $('mobileGridButton')?.classList.toggle('active', $('gridToggle')?.checked ?? false);
}

function setMobileToolsOpen(open) {
  state.mobileToolsOpen = !!open;
  document.body.classList.toggle('mobile-tools-open', state.mobileToolsOpen);
  const sheet = $('mobileToolSheet');
  if (sheet) sheet.hidden = !state.mobileToolsOpen;
}

function setZoom(value, centerEvent = null) {
  const next = Math.max(0.5, Math.min(6, Number(value)));
  state.zoom = Number(next.toFixed(2));
  if ($('zoomSlider')) $('zoomSlider').value = state.zoom;
  renderMap();

  if (centerEvent && $('canvasScroller')) {
    const scroller = $('canvasScroller');
    const rect = scroller.getBoundingClientRect();
    const cx = centerEvent.clientX - rect.left;
    const cy = centerEvent.clientY - rect.top;
    scroller.scrollLeft = Math.max(0, scroller.scrollLeft + cx * 0.02);
    scroller.scrollTop = Math.max(0, scroller.scrollTop + cy * 0.02);
  }
}

function nudgeZoom(delta) {
  setZoom(Number(state.zoom) + delta);
}


function renderRaw(name) {
  const map = {
    'layers.ron': serializeLayersRon(),
    'map.ron': state.bundle.map_ron,
    'props.ron': state.bundle.props_ron,
    'spawns.ron': state.bundle.spawns_ron,
    'triggers.ron': state.bundle.triggers_ron,
    [state.bundle.tileset_file]: state.bundle.tileset_ron,
  };
  $('rawText').value = map[name] ?? '';
}

function renderMap() {
  if (!state.mapInfo || !state.tileset || !state.tilesetImage) return;
  const tw = state.tileset.tileWidth;
  const th = state.tileset.tileHeight;
  canvas.width = state.mapInfo.width * tw;
  canvas.height = state.mapInfo.height * th;
  canvas.style.width = `${canvas.width * state.zoom}px`;
  canvas.style.height = `${canvas.height * state.zoom}px`;
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  for (const layer of state.layers) {
    if (!state.visibleLayers.has(layer.id)) continue;
    for (let y = 0; y < layer.rows.length; y++) {
      const row = layer.rows[y];
      for (let x = 0; x < row.length; x++) {
        const symbol = row[x];
        if (symbol === '.') continue;
        const tileId = layer.legendBySymbol.get(symbol);
        const tile = tileId ? state.tileset.namedTiles.get(tileId) : null;
        if (!tile) continue;
        ctx.drawImage(
          state.tilesetImage,
          tile.x * tw,
          tile.y * th,
          tw,
          th,
          x * tw,
          y * th,
          tw,
          th,
        );
      }
    }
  }

  if ($('gridToggle').checked) drawGrid(tw, th);
  updateMobileControls();
}

function drawGrid(tw, th) {
  ctx.save();
  ctx.strokeStyle = 'rgba(0,0,0,0.9)';
  ctx.lineWidth = 1;
  for (let x = 0; x <= canvas.width; x += tw) {
    ctx.beginPath();
    ctx.moveTo(x + 0.5, 0);
    ctx.lineTo(x + 0.5, canvas.height);
    ctx.stroke();
  }
  for (let y = 0; y <= canvas.height; y += th) {
    ctx.beginPath();
    ctx.moveTo(0, y + 0.5);
    ctx.lineTo(canvas.width, y + 0.5);
    ctx.stroke();
  }
  ctx.restore();
}

function canvasTileFromEvent(event) {
  const rect = canvas.getBoundingClientRect();
  const tw = state.tileset.tileWidth;
  const th = state.tileset.tileHeight;
  const px = (event.clientX - rect.left) * (canvas.width / rect.width);
  const py = (event.clientY - rect.top) * (canvas.height / rect.height);
  return { x: Math.floor(px / tw), y: Math.floor(py / th) };
}

function inspectTile(x, y) {
  const lines = [`map: ${state.currentMapId}`, `tile: ${x}, ${y}`];
  for (const layer of state.layers) {
    const symbol = layer.rows[y]?.[x] ?? 'out-of-range';
    const tileId = layer.legendBySymbol.get(symbol) ?? (symbol === '.' ? 'empty' : 'unknown');
    lines.push(`${layer.id}: ${symbol} -> ${tileId}`);
  }
  $('inspector').textContent = lines.join('\n');
}

function paintTile(x, y) {
  const layer = selectedLayer();
  if (!layer || x < 0 || y < 0 || y >= layer.rows.length) return;
  const row = layer.rows[y];
  if (x >= row.length) return;
  const symbol = state.mode === 'erase' ? '.' : state.selectedSymbol;
  if (!symbol) return;
  if (row[x] === symbol) {
    inspectTile(x, y);
    return;
  }
  layer.rows[y] = row.slice(0, x) + symbol + row.slice(x + 1);
  state.dirty = true;
  renderMap();
  renderRaw('layers.ron');
  inspectTile(x, y);
}

function fillTile(x, y) {
  const layer = selectedLayer();
  if (!layer || x < 0 || y < 0 || y >= layer.rows.length) return;
  const row = layer.rows[y];
  if (x >= row.length) return;
  const targetSymbol = row[x];
  const replacement = state.selectedSymbol;
  if (!replacement || targetSymbol === replacement) return;

  const width = row.length;
  const height = layer.rows.length;
  const rows = layer.rows.map(value => value.split(''));
  const stack = [{ x, y }];
  while (stack.length > 0) {
    const point = stack.pop();
    if (!point || point.x < 0 || point.y < 0 || point.y >= height || point.x >= width) continue;
    if (rows[point.y][point.x] !== targetSymbol) continue;
    rows[point.y][point.x] = replacement;
    stack.push({ x: point.x + 1, y: point.y });
    stack.push({ x: point.x - 1, y: point.y });
    stack.push({ x: point.x, y: point.y + 1 });
    stack.push({ x: point.x, y: point.y - 1 });
  }

  layer.rows = rows.map(value => value.join(''));
  state.dirty = true;
  renderMap();
  renderRaw('layers.ron');
  inspectTile(x, y);
  updateMobileControls();
}


function eyedropTile(x, y) {
  const layer = selectedLayer();
  if (!layer || x < 0 || y < 0 || y >= layer.rows.length) return;
  const symbol = layer.rows[y]?.[x];
  if (symbol && symbol !== '.' && layer.legendBySymbol.has(symbol)) {
    state.selectedSymbol = symbol;
    state.mode = 'paint';
    buildPalette();
    updateMobileControls();
  }
  inspectTile(x, y);
}

function selectedLayer() {
  return state.layers.find(layer => layer.id === state.selectedLayerId) ?? null;
}

function firstPaintableSymbol() {
  const layer = state.layers[0];
  return layer?.legend?.[0]?.symbol ?? null;
}

function serializeLayersRon() {
  const mapId = state.currentMapId ?? 'unknown';
  const tw = state.tileset?.tileWidth ?? 32;
  const th = state.tileset?.tileHeight ?? 32;
  const lines = [];
  lines.push('(');
  lines.push(`    map_id: "${mapId}",`);
  lines.push(`    tile_width: ${tw},`);
  lines.push(`    tile_height: ${th},`);
  lines.push('    layers: [');
  for (const layer of state.layers) {
    lines.push('        (');
    lines.push(`            id: "${layer.id}",`);
    lines.push(`            visible: ${layer.visible ? 'true' : 'false'},`);
    lines.push('            legend: [');
    for (const entry of layer.legend) {
      lines.push(`                (symbol: "${entry.symbol}", tile_id: "${entry.tileId}"),`);
    }
    lines.push('            ],');
    lines.push('            rows: [');
    for (const row of layer.rows) {
      lines.push(`                "${row}",`);
    }
    lines.push('            ],');
    lines.push('        ),');
  }
  lines.push('    ],');
  lines.push(')');
  return lines.join('\n') + '\n';
}

function downloadText(name, text) {
  const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

async function saveLayers() {
  const body = serializeLayersRon();
  const response = await fetch(`/api/save/map/${encodeURIComponent(state.currentMapId)}/layers.ron`, {
    method: 'POST',
    headers: { 'Content-Type': 'text/plain; charset=utf-8' },
    body,
  });
  const text = await response.text();
  if (!response.ok) throw new Error(text);
  state.dirty = false;
  setStatus(`Saved ${state.currentMapId}/layers.ron to repo.`, true);
}

$('mapSelect').addEventListener('change', event => loadMap(event.target.value).catch(showError));
$('paintLayerSelect').addEventListener('change', event => {
  state.selectedLayerId = event.target.value;
  state.selectedSymbol = selectedLayer()?.legend?.[0]?.symbol ?? null;
  buildPalette();
  buildMobileControls();
});
$('zoomSlider').addEventListener('input', event => {
  setZoom(event.target.value);
});
$('gridToggle').addEventListener('change', renderMap);
$('reloadButton').addEventListener('click', () => loadMap(state.currentMapId).catch(showError));
$('exportButton').addEventListener('click', () => downloadText(`${state.currentMapId}_layers.ron`, serializeLayersRon()));
$('saveButton').addEventListener('click', () => saveLayers().catch(showError));
for (const button of document.querySelectorAll('[data-tool]')) {
  button.addEventListener('click', () => {
    const tool = button.dataset.tool;
    if (tool === 'eyedrop' || tool === 'inspect') setToolMode(tool, false);
    else setToolMode(tool, true);
  });
}
$('tabletPaintButton')?.addEventListener('click', () => setToolMode('paint', true));
$('tabletInspectButton')?.addEventListener('click', () => setToolMode('inspect', false));
$('tabletEraseButton')?.addEventListener('click', () => setToolMode('erase', true));
$('tabletEyedropButton')?.addEventListener('click', () => setToolMode('eyedrop', false));
$('pcModeButton')?.addEventListener('click', () => applyPresentationMode('pc'));
$('tabletModeButton')?.addEventListener('click', () => applyPresentationMode('tablet'));
$('mobileModeButton')?.addEventListener('click', () => applyPresentationMode('mobile'));
$('viewLauncherButton')?.addEventListener('click', showLauncher);
$('launcherSkipButton')?.addEventListener('click', () => launchWithMode(preferredPresentationMode(), false));
for (const button of document.querySelectorAll('[data-launch-mode]')) {
  button.addEventListener('click', () => launchWithMode(button.dataset.launchMode, $('rememberLauncherChoice')?.checked ?? true));
}

for (const button of document.querySelectorAll('.workspaceTabs button')) {
  button.addEventListener('click', () => setWorkspace(button.dataset.workspace));
}
window.addEventListener('resize', () => {
  if (state.presentationMode === 'auto') applyPresentationMode(preferredPresentationMode(), false);
});



$('mobileMapSelect')?.addEventListener('change', event => {
  $('mapSelect').value = event.target.value;
  loadMap(event.target.value).catch(showError);
});
$('mobileLayerSelect')?.addEventListener('change', event => {
  $('paintLayerSelect').value = event.target.value;
  state.selectedLayerId = event.target.value;
  state.selectedSymbol = selectedLayer()?.legend?.[0]?.symbol ?? null;
  buildPalette();
  buildMobileControls();
});
$('mobilePaintButton')?.addEventListener('click', () => setToolMode('paint', true));
$('mobileEraseButton')?.addEventListener('click', () => setToolMode('erase', true));
$('mobileFillButton')?.addEventListener('click', () => setToolMode('fill', true));
$('mobilePickButton')?.addEventListener('click', () => setToolMode('eyedrop', false));
$('mobileToolsButton')?.addEventListener('click', () => setMobileToolsOpen(!state.mobileToolsOpen));
$('mobileCloseToolsButton')?.addEventListener('click', () => setMobileToolsOpen(false));
$('mobileSaveButton')?.addEventListener('click', () => saveLayers().catch(showError));
$('mobileZoomOutButton')?.addEventListener('click', () => nudgeZoom(-0.25));
$('mobileZoomInButton')?.addEventListener('click', () => nudgeZoom(0.25));
$('mobileGridButton')?.addEventListener('click', () => {
  $('gridToggle').checked = !$('gridToggle').checked;
  renderMap();
});

window.addEventListener('keydown', event => {
  const target = event.target;
  const isTyping = target && ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
  if (isTyping) return;

  const key = event.key.toLowerCase();
  if ((event.ctrlKey || event.metaKey) && key === 's') {
    event.preventDefault();
    if (!$('saveButton').disabled) saveLayers().catch(showError);
    return;
  }
  if ((event.ctrlKey || event.metaKey) && key === 'e') {
    event.preventDefault();
    downloadText(`${state.currentMapId}_layers.ron`, serializeLayersRon());
    return;
  }
  if ((event.ctrlKey || event.metaKey) && key === 'r') {
    event.preventDefault();
    loadMap(state.currentMapId).catch(showError);
    return;
  }

  if (key === '1' || key === 'p') {
    event.preventDefault();
    setToolMode('paint', true);
  } else if (key === '2' || key === 'e') {
    event.preventDefault();
    setToolMode('erase', true);
  } else if (key === '3' || key === 'f') {
    event.preventDefault();
    setToolMode('fill', true);
  } else if (key === '4' || key === 'i') {
    event.preventDefault();
    setToolMode('eyedrop', false);
  } else if (key === '5' || key === 'q' || key === 'escape') {
    event.preventDefault();
    setToolMode('inspect', false);
  } else if (key === 'b') {
    event.preventDefault();
    $('brushDrawer')?.classList.toggle('open');
  } else if (key === 'g') {
    event.preventDefault();
    $('gridToggle').checked = !$('gridToggle').checked;
    renderMap();
  } else if (key === ']' || key === '=' || key === '+') {
    event.preventDefault();
    setZoom(state.zoom + 0.25);
  } else if (key === '[' || key === '-' || key === '_') {
    event.preventDefault();
    setZoom(state.zoom - 0.25);
  } else if (key === 'r') {
    event.preventDefault();
    loadMap(state.currentMapId).catch(showError);
  }

});

function performCanvasAction(event, drag = false) {
  const { x, y } = canvasTileFromEvent(event);
  const key = `${x},${y}`;
  if (drag && state.lastPaintTileKey === key) return;
  state.lastPaintTileKey = key;

  if (state.mode === 'eyedrop') {
    if (!drag) eyedropTile(x, y);
  } else if (state.mode === 'fill') {
    if (!drag) fillTile(x, y);
  } else if ($('paintToggle').checked || state.mode === 'paint' || state.mode === 'erase') {
    paintTile(x, y);
  } else {
    inspectTile(x, y);
  }
}

function pointerDistance(a, b) {
  return Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY);
}

function pointerCenter(a, b) {
  return {
    clientX: (a.clientX + b.clientX) / 2,
    clientY: (a.clientY + b.clientY) / 2,
  };
}

function startTouchGesture() {
  if (state.activePointers.size < 2) {
    state.touchGesture = null;
    return;
  }
  const [a, b] = Array.from(state.activePointers.values());
  const scroller = $('canvasScroller');
  state.touchGesture = {
    distance: Math.max(1, pointerDistance(a, b)),
    zoom: state.zoom,
    center: pointerCenter(a, b),
    scrollLeft: scroller?.scrollLeft ?? 0,
    scrollTop: scroller?.scrollTop ?? 0,
  };
}

function updateTouchGesture() {
  if (!state.touchGesture || state.activePointers.size < 2) return;
  const [a, b] = Array.from(state.activePointers.values());
  const distance = Math.max(1, pointerDistance(a, b));
  const center = pointerCenter(a, b);
  const nextZoom = state.touchGesture.zoom * (distance / state.touchGesture.distance);
  setZoom(nextZoom);

  const scroller = $('canvasScroller');
  if (scroller) {
    scroller.scrollLeft = state.touchGesture.scrollLeft - (center.clientX - state.touchGesture.center.clientX);
    scroller.scrollTop = state.touchGesture.scrollTop - (center.clientY - state.touchGesture.center.clientY);
  }
}

canvas.addEventListener('pointerdown', event => {
  if (!state.tileset) return;
  canvas.setPointerCapture?.(event.pointerId);
  state.activePointers.set(event.pointerId, {
    clientX: event.clientX,
    clientY: event.clientY,
  });
  state.lastPaintTileKey = null;

  if (state.activePointers.size >= 2) {
    event.preventDefault();
    startTouchGesture();
    return;
  }

  event.preventDefault();
  performCanvasAction(event, false);
});

canvas.addEventListener('pointermove', event => {
  if (!state.activePointers.has(event.pointerId)) return;
  state.activePointers.set(event.pointerId, {
    clientX: event.clientX,
    clientY: event.clientY,
  });

  if (state.activePointers.size >= 2) {
    event.preventDefault();
    updateTouchGesture();
    return;
  }

  if (event.buttons !== 1 && event.pointerType === 'mouse') return;
  if (state.mode === 'eyedrop' || state.mode === 'fill' || state.mode === 'inspect') return;
  event.preventDefault();
  performCanvasAction(event, true);
});

function endCanvasPointer(event) {
  state.activePointers.delete(event.pointerId);
  state.lastPaintTileKey = null;
  if (state.activePointers.size >= 2) startTouchGesture();
  else state.touchGesture = null;
}

canvas.addEventListener('pointerup', endCanvasPointer);
canvas.addEventListener('pointercancel', endCanvasPointer);
canvas.addEventListener('pointerleave', event => {
  if (event.pointerType === 'mouse') endCanvasPointer(event);
});


function showError(error) {
  console.error(error);
  setStatus(error.message, false);
}

applyPresentationMode(preferredPresentationMode(), false);
setWorkspace('world');
setToolMode('paint', true);
if (shouldShowLauncher()) showLauncher();
loadManifest().catch(showError);
