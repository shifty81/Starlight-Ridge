const pipeline = {
  id: 'phase19_atlas_pipeline',
  activeSeason: 'spring',
  tileSize: 32,
  atlases: [
    { id: 'terrain_world', label: 'World Terrain', asset: '../../assets/textures/terrain_atlas_phase17_generated.png', kind: 'tileset', layer: 0, role: 'terrain_world_tiles', columns: 11, rows: 13, allowed: ['terrain', 'transition', 'soil', 'path', 'water'], forbidden: ['props', 'fences', 'buildings'] },
    { id: 'terrain_spring', label: 'Spring Terrain', asset: '../../assets/textures/terrain_spring_v1.png', kind: 'tileset', layer: 0, role: 'season_variant_spring', columns: 10, rows: 10, allowed: ['terrain', 'water'], forbidden: ['props'] },
    { id: 'terrain_summer', label: 'Summer Terrain', asset: '../../assets/textures/terrain_summer_v1.png', kind: 'tileset', layer: 0, role: 'season_variant_summer', columns: 10, rows: 10, allowed: ['terrain', 'water'], forbidden: ['props'] },
    { id: 'terrain_fall', label: 'Fall Terrain', asset: '../../assets/textures/terrain_autumn_v1.png', kind: 'tileset', layer: 0, role: 'season_variant_fall', columns: 10, rows: 10, allowed: ['terrain', 'water'], forbidden: ['props'] },
    { id: 'terrain_winter', label: 'Winter Terrain', asset: '../../assets/textures/terrain_winter_v1.png', kind: 'tileset', layer: 0, role: 'season_variant_winter', columns: 10, rows: 10, allowed: ['terrain', 'water'], forbidden: ['props'] },
    { id: 'props_static', label: 'Static Props', asset: '../../assets/textures/oceans_heart_bridge_phase17.png', kind: 'sprite_sheet', layer: 5, role: 'prop_sprite_sheet', columns: 4, rows: 4, allowed: ['static_props', 'effects', 'critters'], forbidden: ['base_terrain'] },
  ],
  seasonSets: [
    ['grass', 'grass_0', 'grass_0', 'grass_0', 'grass_0'],
    ['path', 'path_sand', 'path_sand', 'path_sand', 'path_sand'],
    ['shallow_water', 'water_shallow', 'water_shallow', 'water_shallow', 'water_shallow'],
    ['deep_water', 'water_deep', 'water_deep', 'water_deep', 'water_deep'],
    ['tilled_dry', 'tilled_dry', 'tilled_dry', 'tilled_dry', 'tilled_dry'],
    ['tilled_watered', 'tilled_watered', 'tilled_watered', 'tilled_watered', 'tilled_watered'],
  ],
  waterAnimations: [
    { id: 'shallow_water_idle', terrain: 'shallow_water', frames: 4, frameMs: 180, loop: 'loop', layer: 2 },
    { id: 'deep_water_idle', terrain: 'deep_water', frames: 4, frameMs: 220, loop: 'loop', layer: 2 },
  ],
  clipboardTools: [
    { id: 'tile_clipboard_import', label: 'Clipboard Import / Paste', flags: ['snap grid', 'mirror H', 'mirror V', 'rotate', 'palette remap', 'assign metadata'] },
    { id: 'mirror_aware_paint', label: 'Mirror-Aware Paint and Paste', flags: ['snap grid', 'mirror H', 'mirror V', 'assign metadata'] },
  ],
  validation: [
    ['missing_tile_reference', 'error', 'Every map legend and pipeline tile reference must resolve.'],
    ['terrain_prop_separation', 'warning', 'Terrain tools should not use prop/fence/building atlas roles.'],
    ['season_variant_completeness', 'error', 'Semantic terrain needs spring, summer, fall, and winter variants.'],
    ['water_animation_metadata', 'error', 'Water terrain needs animation frame metadata.'],
    ['map_game_preview_export', 'warning', 'Game preview must hide editor/debug overlays.'],
  ],
};

const phase20 = {
  exportProfiles: [
    { id: 'game_content_pack_dry_run', target: 'exports/game_content_pack', includes: ['assets/textures', 'content/tiles', 'content/terrain', 'content/editor_pipeline', 'content/editor_export', 'content/maps', 'content/metadata'], required: 7, dryRun: true },
  ],
  autotileRuleSets: [
    { id: 'coastal_base_autotile_v1', mode: 'cardinal_and_corner_masks', layer: 3, rules: 12, pairs: ['grass → sand', 'sand → shallow_water', 'shallow_water → deep_water'] },
    { id: 'farm_soil_autotile_v1', mode: 'cardinal_masks', layer: 2, rules: 12, pairs: ['grass → path', 'grass → tilled_dry', 'tilled_dry → tilled_watered'] },
  ],
  collisionProfiles: [
    { id: 'terrain_grass_walkable', target: 'terrain:grass', behavior: 'walkable' },
    { id: 'terrain_shallow_water_blocked', target: 'terrain:shallow_water', behavior: 'blocked water + fish prompt' },
    { id: 'prop_weak_tree_full_choppable', target: 'sprite:weak_tree_full', behavior: 'blocked + axe/chop interaction' },
    { id: 'prop_driftwood_log_collectable', target: 'sprite:driftwood_log', behavior: 'blocked + collect interaction' },
  ],
  cleanup: [
    { id: 'split_props_from_terrain_world_v1', source: 'terrain_world', target: 'props_static', actions: ['move trees', 'move rocks', 'extract fences', 'flag misaligned tail rows'] },
  ],
};



const phase21 = {
  timelineSchemas: [
    { id: 'gameplay_timeline_v1', tracks: ['frames', 'events', 'sockets', 'hitboxes', 'audio', 'effects'], markers: ['play_sound', 'spawn_effect', 'apply_tool_action', 'emit_footstep'] },
    { id: 'water_preview_timeline_v1', tracks: ['frames', 'season', 'random_start_offset', 'shore_overlay'], markers: ['water_loop', 'shore_foam', 'sparkle_overlay'] },
  ],
  clips: [
    { id: 'player_walk_down', target: 'player', direction: 'down', loop: 'loop', frames: 4, events: ['footstep L', 'footstep R'] },
    { id: 'player_walk_left', target: 'player', direction: 'left', loop: 'loop', frames: 4, events: ['footstep L', 'footstep R'] },
    { id: 'player_walk_right', target: 'player', direction: 'right', loop: 'loop', frames: 4, events: ['footstep L', 'footstep R'] },
    { id: 'player_walk_up', target: 'player', direction: 'up', loop: 'loop', frames: 4, events: ['footstep L', 'footstep R'] },
    { id: 'player_hoe_down_timeline', target: 'player', direction: 'down', loop: 'once', frames: 4, events: ['lock input', 'tool sound', 'apply hoe action', 'spawn dust', 'unlock input'] },
    { id: 'seagull_idle_flap', target: 'sprite:seagull_idle', direction: 'none', loop: 'loop', frames: 4, events: [] },
    { id: 'impact_puff_short', target: 'effect:impact_puff', direction: 'none', loop: 'once', frames: 2, events: ['soft impact sound'] },
    { id: 'shallow_water_loop_tiles', target: 'water:shallow_water', direction: 'none', loop: 'loop', frames: 4, events: ['water loop marker'] },
  ],
  directionalGroups: [
    { id: 'player_walk_4dir', directions: ['down', 'left', 'right', 'up'], fallback: 'down' },
    { id: 'player_hoe_down_action', directions: ['down'], fallback: 'down' },
  ],
  sockets: [
    { id: 'player_walk_hand_and_feet_sockets', clip: 'player_walk_down', count: 3, required: 0 },
    { id: 'hoe_down_tool_sockets', clip: 'player_hoe_down_timeline', count: 4, required: 4 },
  ],
  hitboxes: [
    { id: 'hoe_down_tool_hitbox', clip: 'player_hoe_down_timeline', boxes: ['tool_action', 'interaction_window'] },
    { id: 'impact_puff_effect_bounds', clip: 'impact_puff_short', boxes: ['effect_bounds'] },
  ],
  waterPreviews: [
    { id: 'shallow_water_editor_preview', animation: 'shallow_water_idle', size: '5×3', season: 'spring', shore: true },
    { id: 'shallow_water_timeline_preview', animation: 'shallow_water_loop_tiles', size: '4×4', season: 'all', shore: false },
  ],
  seasonalSets: [
    { id: 'player_walk_all_seasons', semantic: 'player_walk', fallback: 'player_walk_down' },
    { id: 'shallow_water_loop_all_seasons', semantic: 'shallow_water_loop', fallback: 'shallow_water_loop_tiles' },
  ],
  validation: [
    ['animation_frame_reference_validation', 'error', 'Frame sprite IDs resolve, durations are non-zero, duplicate frame indices are rejected.'],
    ['animation_event_validation', 'error', 'Required gameplay events have payloads and matching frame indices.'],
    ['socket_hitbox_validation', 'warning', 'Required sockets exist on impact frames and hitboxes reference valid frames.'],
    ['seasonal_animation_validation', 'warning', 'Season variants and fallback clips resolve.'],
  ],
};

let activeAtlas = pipeline.atlases[0];
const state = { mirrorH: false, mirrorV: false, paste: false };

const $ = (id) => document.getElementById(id);

function init() {
  $('pipeline-status').textContent = `${pipeline.atlases.length} atlases • ${phase20.autotileRuleSets.length} autotile rule sets • ${phase21.clips.length} animation clips • ${phase21.waterPreviews.length} water previews`;
  renderAtlasList();
  renderSeasonSelect();
  renderMetadataPanels();
  renderAnimationPanels();
  renderAtlas(activeAtlas);

  $('toggle-grid').addEventListener('change', () => renderAtlas(activeAtlas));
  $('toggle-center').addEventListener('change', () => renderAtlas(activeAtlas));
  $('toggle-game-preview').addEventListener('change', () => document.body.classList.toggle('game-preview', $('toggle-game-preview').checked));
  $('mirror-h').addEventListener('click', () => toggleTool('mirrorH', 'Mirror H'));
  $('mirror-v').addEventListener('click', () => toggleTool('mirrorV', 'Mirror V'));
  $('paste-tool').addEventListener('click', () => toggleTool('paste', 'Paste Tool'));
  $('validate-map').addEventListener('click', () => runValidationFlash());
  $('export-pack').addEventListener('click', () => runExportDryRun());
}

function renderAtlasList() {
  $('atlas-list').innerHTML = '';
  pipeline.atlases.forEach((atlas) => {
    const node = document.createElement('div');
    node.className = `nav-item ${atlas.id === activeAtlas.id ? 'active' : ''}`;
    node.innerHTML = `<strong>${atlas.label}</strong><span>${atlas.role}</span>`;
    node.addEventListener('click', () => { activeAtlas = atlas; renderAtlasList(); renderAtlas(atlas); });
    $('atlas-list').appendChild(node);
  });
}

function renderSeasonSelect() {
  const select = $('season-select');
  ['spring', 'summer', 'fall', 'winter'].forEach((season) => {
    const option = document.createElement('option');
    option.value = season;
    option.textContent = season[0].toUpperCase() + season.slice(1);
    option.selected = season === pipeline.activeSeason;
    select.appendChild(option);
  });
}

function renderAtlas(atlas) {
  $('active-atlas-title').textContent = atlas.label;
  $('active-atlas-meta').textContent = `${atlas.kind} • ${atlas.role} • render layer ${atlas.layer} • allows ${atlas.allowed.join(', ')} • forbids ${atlas.forbidden.join(', ')}`;

  const canvas = $('atlas-canvas');
  const ctx = canvas.getContext('2d');
  canvas.width = Math.max(320, atlas.columns * pipeline.tileSize * 2);
  canvas.height = Math.max(192, atlas.rows * pipeline.tileSize * 2);
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = '#0b0e13';
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  const img = new Image();
  img.onload = () => {
    ctx.save();
    if (state.mirrorH || state.mirrorV) {
      ctx.translate(state.mirrorH ? canvas.width : 0, state.mirrorV ? canvas.height : 0);
      ctx.scale(state.mirrorH ? -1 : 1, state.mirrorV ? -1 : 1);
    }
    ctx.drawImage(img, 0, 0, atlas.columns * pipeline.tileSize * 2, atlas.rows * pipeline.tileSize * 2);
    ctx.restore();
    drawOverlays(ctx, canvas, atlas);
  };
  img.onerror = () => {
    ctx.fillStyle = '#ff6666';
    ctx.fillText(`Missing preview image: ${atlas.asset}`, 24, 48);
    drawOverlays(ctx, canvas, atlas);
  };
  img.src = atlas.asset;
}

function drawOverlays(ctx, canvas, atlas) {
  const size = pipeline.tileSize * 2;
  if ($('toggle-grid').checked) {
    ctx.strokeStyle = '#000000';
    ctx.lineWidth = 1;
    for (let x = 0; x <= atlas.columns; x++) {
      ctx.beginPath(); ctx.moveTo(x * size + 0.5, 0); ctx.lineTo(x * size + 0.5, atlas.rows * size); ctx.stroke();
    }
    for (let y = 0; y <= atlas.rows; y++) {
      ctx.beginPath(); ctx.moveTo(0, y * size + 0.5); ctx.lineTo(atlas.columns * size, y * size + 0.5); ctx.stroke();
    }
  }
  if ($('toggle-center').checked) {
    ctx.strokeStyle = '#ff3333';
    ctx.lineWidth = 1;
    for (let x = 0; x < atlas.columns; x++) {
      ctx.beginPath(); ctx.moveTo(x * size + size / 2 + 0.5, 0); ctx.lineTo(x * size + size / 2 + 0.5, atlas.rows * size); ctx.stroke();
    }
    for (let y = 0; y < atlas.rows; y++) {
      ctx.beginPath(); ctx.moveTo(0, y * size + size / 2 + 0.5); ctx.lineTo(atlas.columns * size, y * size + size / 2 + 0.5); ctx.stroke();
    }
  }
}

function renderMetadataPanels() {
  $('season-table').innerHTML = pipeline.seasonSets.map(([role, spring, summer, fall, winter]) => `
    <div class="table-row"><strong>${role}</strong><span>${spring}</span><span>${summer}</span><span>${fall}</span><span>${winter}</span></div>
  `).join('');

  $('water-list').innerHTML = pipeline.waterAnimations.map((anim) => `
    <div class="card"><strong>${anim.id}</strong><span>${anim.terrain} • ${anim.frames} frames • ${anim.frameMs}ms • layer ${anim.layer}</span></div>
  `).join('');

  $('clipboard-list').innerHTML = pipeline.clipboardTools.map((tool) => `
    <div class="card"><strong>${tool.label}</strong><span>${tool.flags.map(flag => `<span class="badge">${flag}</span>`).join(' ')}</span></div>
  `).join('');

  $('validation-list').innerHTML = pipeline.validation.map(([id, severity, desc]) => `
    <div class="card"><strong class="${severity === 'error' ? 'error' : 'warn'}">${severity.toUpperCase()} — ${id}</strong><span>${desc}</span></div>
  `).join('');

  $('export-list').innerHTML = phase20.exportProfiles.map((profile) => `
    <div class="card"><strong>${profile.id}</strong><span>${profile.target} • ${profile.includes.length} include paths • ${profile.required} required outputs • ${profile.dryRun ? 'dry run default' : 'write enabled'}</span></div>
  `).join('');

  $('autotile-list').innerHTML = phase20.autotileRuleSets.map((ruleset) => `
    <div class="card"><strong>${ruleset.id}</strong><span>${ruleset.mode} • layer ${ruleset.layer} • ${ruleset.rules} rules<br>${ruleset.pairs.map(pair => `<span class="badge">${pair}</span>`).join(' ')}</span></div>
  `).join('');

  $('collision-list').innerHTML = phase20.collisionProfiles.map((profile) => `
    <div class="card"><strong>${profile.id}</strong><span>${profile.target} • ${profile.behavior}</span></div>
  `).join('');

  $('cleanup-list').innerHTML = phase20.cleanup.map((manifest) => `
    <div class="card"><strong>${manifest.id}</strong><span>${manifest.source} → ${manifest.target}<br>${manifest.actions.map(action => `<span class="badge">${action}</span>`).join(' ')}</span></div>
  `).join('');
}

function toggleTool(key, label) {
  state[key] = !state[key];
  $('pipeline-status').textContent = `${label}: ${state[key] ? 'on' : 'off'}`;
  renderAtlas(activeAtlas);
}

function runValidationFlash() {
  $('pipeline-status').textContent = 'Validation contract loaded: atlas/export/autotile/collision plus animation frames, events, sockets, hitboxes, water previews, and seasonal fallbacks ready.';
}

function runExportDryRun() {
  const profile = phase20.exportProfiles[0];
  $('pipeline-status').textContent = `Dry-run export profile: ${profile.includes.length} folders, ${profile.required} required outputs, target ${profile.target}.`;
}

init();


function renderAnimationPanels() {
  $('animation-timeline-list').innerHTML = phase21.clips.map((clip) => `
    <div class="timeline-card">
      <div class="timeline-header"><strong>${clip.id}</strong><span>${clip.target} • ${clip.direction} • ${clip.loop}</span></div>
      <div class="timeline-track">${Array.from({ length: clip.frames }).map((_, index) => `<span class="frame-block">${index}</span>`).join('')}</div>
      <div class="timeline-events">${clip.events.length ? clip.events.map(event => `<span class="badge">${event}</span>`).join(' ') : '<span class="muted">no frame events</span>'}</div>
    </div>
  `).join('');

  $('animation-event-list').innerHTML = phase21.timelineSchemas.map((schema) => `
    <div class="card"><strong>${schema.id}</strong><span>${schema.tracks.map(track => `<span class="badge">${track}</span>`).join(' ')}<br>${schema.markers.map(marker => `<span class="badge">${marker}</span>`).join(' ')}</span></div>
  `).join('');

  $('socket-hitbox-list').innerHTML = [
    ...phase21.sockets.map((profile) => `<div class="card"><strong>${profile.id}</strong><span>${profile.clip} • ${profile.count} sockets • ${profile.required} required</span></div>`),
    ...phase21.hitboxes.map((profile) => `<div class="card"><strong>${profile.id}</strong><span>${profile.clip}<br>${profile.boxes.map(box => `<span class="badge">${box}</span>`).join(' ')}</span></div>`),
  ].join('');

  $('water-preview-list').innerHTML = phase21.waterPreviews.map((preview) => `
    <div class="card"><strong>${preview.id}</strong><span>${preview.animation} • ${preview.size} tiles • season ${preview.season} • shore overlay ${preview.shore ? 'on' : 'off'}</span></div>
  `).join('');

  $('seasonal-animation-list').innerHTML = phase21.seasonalSets.map((set) => `
    <div class="card"><strong>${set.id}</strong><span>${set.semantic} • fallback ${set.fallback}</span></div>
  `).join('');

  $('animation-validation-list').innerHTML = phase21.validation.map(([id, severity, desc]) => `
    <div class="card"><strong class="${severity === 'error' ? 'error' : 'warn'}">${severity.toUpperCase()} — ${id}</strong><span>${desc}</span></div>
  `).join('');
}
