use crate::registry::ContentRegistry;
use anyhow::ensure;
use std::collections::HashSet;

pub fn validate_registry(registry: &ContentRegistry) -> anyhow::Result<()> {
    ensure!(!registry.items.is_empty(), "registry has no item definitions loaded");
    ensure!(!registry.maps.is_empty(), "registry has no map bundles loaded");
    ensure!(!registry.tilesets.is_empty(), "registry has no tilesets loaded");
    ensure!(!registry.sprite_sheets.is_empty(), "registry has no sprite sheets loaded");

    for map in registry.maps.values() {
        ensure!(
            registry.tilesets.contains_key(&map.metadata.tileset),
            "map '{}' references missing tileset '{}'",
            map.metadata.id,
            map.metadata.tileset,
        );
        ensure!(
            registry.map_layers.contains_key(&map.metadata.id),
            "map '{}' is missing layers.ron data",
            map.metadata.id,
        );
    }

    for (map_id, layers) in &registry.map_layers {
        let map = registry
            .maps
            .get(map_id)
            .ok_or_else(|| anyhow::anyhow!("layers exist for unknown map '{}'", map_id))?;
        ensure!(
            layers.tile_width > 0 && layers.tile_height > 0,
            "map '{}' has invalid tile dimensions {}x{}",
            map_id,
            layers.tile_width,
            layers.tile_height
        );
        let tileset = registry
            .tilesets
            .get(&map.metadata.tileset)
            .ok_or_else(|| anyhow::anyhow!("map '{}' references missing tileset '{}'", map_id, map.metadata.tileset))?;
        let known_tiles = tileset
            .named_tiles
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<HashSet<_>>();

        for layer in &layers.layers {
            ensure!(
                layer.rows.len() == map.metadata.height as usize,
                "map '{}' layer '{}' has {} rows but metadata height is {}",
                map_id,
                layer.id,
                layer.rows.len(),
                map.metadata.height
            );
            for legend in &layer.legend {
                ensure!(
                    legend.symbol.chars().count() == 1,
                    "map '{}' layer '{}' legend symbol '{}' must be exactly one character",
                    map_id,
                    layer.id,
                    legend.symbol
                );
                let is_known_tile = known_tiles.contains(legend.tile_id.as_str());
                let is_known_terrain = registry.terrain_types.contains_key(&legend.tile_id);
                ensure!(
                    is_known_tile || is_known_terrain,
                    "map '{}' layer '{}' legend symbol '{}' references missing tile/terrain '{}' in tileset '{}' and terrain catalog",
                    map_id,
                    layer.id,
                    legend.symbol,
                    legend.tile_id,
                    tileset.id
                );
            }
            for (row_index, row) in layer.rows.iter().enumerate() {
                ensure!(
                    row.chars().count() == map.metadata.width as usize,
                    "map '{}' layer '{}' row {} has width {} but metadata width is {}",
                    map_id,
                    layer.id,
                    row_index,
                    row.chars().count(),
                    map.metadata.width
                );
            }
        }
    }

    for npc in registry.npcs.values() {
        ensure!(
            registry.schedules.contains_key(&npc.schedule_id),
            "npc '{}' references missing schedule '{}'",
            npc.id,
            npc.schedule_id
        );
        ensure!(
            registry.dialogues.contains_key(&npc.dialogue_id),
            "npc '{}' references missing dialogue '{}'",
            npc.id,
            npc.dialogue_id
        );
        if let Some(shop_id) = &npc.shop_id {
            ensure!(
                registry.shops.contains_key(shop_id),
                "npc '{}' references missing shop '{}'",
                npc.id,
                shop_id
            );
        }
    }

    for shop in registry.shops.values() {
        for stock in &shop.stock {
            ensure!(
                registry.items.contains_key(&stock.item_id),
                "shop '{}' references missing item '{}'",
                shop.id,
                stock.item_id
            );
        }
    }


    if registry.has_phase19_editor_pipeline() {
        validate_phase19_editor_pipeline(registry)?;
    }

    if registry.has_phase20_editor_export_pipeline() {
        validate_phase20_editor_export_pipeline(registry)?;
    }

    if registry.has_phase21_editor_animation_pipeline() {
        validate_phase21_editor_animation_pipeline(registry)?;
    }

    if registry.has_phase17_terrain_contracts() {
        ensure!(
            !registry.terrain_types.is_empty(),
            "phase17 terrain contracts are partially loaded: terrain_types.ron is missing or empty"
        );

        for terrain in registry.terrain_types.values() {
            ensure!(!terrain.id.trim().is_empty(), "terrain type has empty id");
            ensure!(
                !terrain.base_variants.trim().is_empty(),
                "terrain type '{}' has empty base variant set reference",
                terrain.id
            );
            ensure!(
                !terrain.fallback_tile_id.trim().is_empty(),
                "terrain type '{}' has empty fallback tile id",
                terrain.id
            );
        }

        for biome in registry.biome_packs.values() {
            ensure!(
                registry.terrain_rulesets.contains_key(&biome.ruleset),
                "biome pack '{}' references missing terrain ruleset '{}'",
                biome.id,
                biome.ruleset
            );
            ensure!(
                !biome.terrain_variant_sets.is_empty(),
                "biome pack '{}' has no terrain variant sets",
                biome.id
            );
            for set in &biome.terrain_variant_sets {
                ensure!(
                    registry.terrain_types.contains_key(&set.terrain_id),
                    "biome pack '{}' variant set '{}' references missing terrain type '{}'",
                    biome.id,
                    set.id,
                    set.terrain_id
                );
                ensure!(
                    set.variants.len() >= set.min_variants as usize,
                    "biome pack '{}' variant set '{}' requires at least {} variants but has {}",
                    biome.id,
                    set.id,
                    set.min_variants,
                    set.variants.len()
                );
                ensure!(
                    !set.fallback_tile_id.trim().is_empty(),
                    "biome pack '{}' variant set '{}' has empty fallback tile id",
                    biome.id,
                    set.id
                );
                for variant in &set.variants {
                    ensure!(
                        variant.weight > 0,
                        "biome pack '{}' variant set '{}' tile '{}' has zero weight",
                        biome.id,
                        set.id,
                        variant.tile_id
                    );
                    ensure!(
                        !variant.tile_id.trim().is_empty(),
                        "biome pack '{}' variant set '{}' has an empty tile id",
                        biome.id,
                        set.id
                    );
                }
            }
        }

        for transition_set in registry.transition_sets.values() {
            ensure!(
                !transition_set.transitions.is_empty(),
                "transition set '{}' has no transition rules",
                transition_set.id
            );
            for transition in &transition_set.transitions {
                ensure!(
                    registry.terrain_types.contains_key(&transition.from),
                    "transition set '{}' rule '{}' references missing source terrain '{}'",
                    transition_set.id,
                    transition.id,
                    transition.from
                );
                ensure!(
                    registry.terrain_types.contains_key(&transition.to),
                    "transition set '{}' rule '{}' references missing target terrain '{}'",
                    transition_set.id,
                    transition.id,
                    transition.to
                );
                ensure!(
                    !transition.fallback_tile_id.trim().is_empty(),
                    "transition set '{}' rule '{}' has empty fallback tile id",
                    transition_set.id,
                    transition.id
                );
                ensure!(
                    transition.render_layer > 0,
                    "transition set '{}' rule '{}' must use render_layer > 0",
                    transition_set.id,
                    transition.id
                );
                for tile in &transition.tiles {
                    ensure!(
                        !tile.tile_id.trim().is_empty(),
                        "transition set '{}' rule '{}' has an empty transition tile id for mask {}",
                        transition_set.id,
                        transition.id,
                        tile.mask
                    );
                }
            }
        }

        for ruleset in registry.terrain_rulesets.values() {
            ensure!(
                !ruleset.terrain_priority.is_empty(),
                "terrain ruleset '{}' has empty terrain_priority",
                ruleset.id
            );
            for terrain_id in &ruleset.terrain_priority {
                ensure!(
                    registry.terrain_types.contains_key(terrain_id),
                    "terrain ruleset '{}' references missing terrain type '{}'",
                    ruleset.id,
                    terrain_id
                );
            }
            for transition_set_id in &ruleset.active_transition_sets {
                ensure!(
                    registry.transition_sets.contains_key(transition_set_id),
                    "terrain ruleset '{}' references missing transition set '{}'",
                    ruleset.id,
                    transition_set_id
                );
            }
        }
    }

    Ok(())
}

fn validate_phase19_editor_pipeline(registry: &ContentRegistry) -> anyhow::Result<()> {
    let known_seasons = ["spring", "summer", "fall", "autumn", "winter"];

    for pipeline in registry.editor_atlas_pipelines.values() {
        ensure!(!pipeline.id.trim().is_empty(), "editor atlas pipeline has empty id");
        ensure!(pipeline.tile_size > 0, "editor atlas pipeline '{}' has invalid tile_size {}", pipeline.id, pipeline.tile_size);
        ensure!(!pipeline.atlases.is_empty(), "editor atlas pipeline '{}' has no atlas entries", pipeline.id);
        ensure!(!pipeline.validation_checks.is_empty(), "editor atlas pipeline '{}' has no validation checks", pipeline.id);
        ensure!(
            known_seasons.contains(&pipeline.active_season.as_str()),
            "editor atlas pipeline '{}' uses unknown active season '{}'",
            pipeline.id,
            pipeline.active_season
        );

        let editor_atlas_ids = pipeline
            .atlases
            .iter()
            .map(|atlas| atlas.id.as_str())
            .collect::<HashSet<_>>();

        for atlas in &pipeline.atlases {
            ensure!(!atlas.id.trim().is_empty(), "editor atlas pipeline '{}' has atlas with empty id", pipeline.id);
            ensure!(!atlas.asset_ref.trim().is_empty(), "editor atlas '{}' has empty asset_ref", atlas.id);
            let asset_exists = registry.tilesets.contains_key(&atlas.asset_ref)
                || registry.sprite_sheets.contains_key(&atlas.asset_ref);
            ensure!(
                asset_exists,
                "editor atlas '{}' references missing tileset/sprite sheet '{}'",
                atlas.id,
                atlas.asset_ref
            );
            ensure!(
                !atlas.allowed_categories.is_empty(),
                "editor atlas '{}' has no allowed_categories",
                atlas.id
            );
        }

        for season_set in &pipeline.season_sets {
            ensure!(!season_set.semantic_tile_id.trim().is_empty(), "season set '{}' has empty semantic_tile_id", season_set.id);
            validate_editor_tile_ref(registry, &editor_atlas_ids, &season_set.spring.atlas_id, &season_set.spring.tile_id, &season_set.id, "spring")?;
            validate_editor_tile_ref(registry, &editor_atlas_ids, &season_set.summer.atlas_id, &season_set.summer.tile_id, &season_set.id, "summer")?;
            validate_editor_tile_ref(registry, &editor_atlas_ids, &season_set.fall.atlas_id, &season_set.fall.tile_id, &season_set.id, "fall")?;
            validate_editor_tile_ref(registry, &editor_atlas_ids, &season_set.winter.atlas_id, &season_set.winter.tile_id, &season_set.id, "winter")?;
        }

        for animation in &pipeline.water_animations {
            ensure!(!animation.frames.is_empty(), "water animation '{}' has no frames", animation.id);
            ensure!(animation.frame_ms > 0, "water animation '{}' has invalid frame_ms {}", animation.id, animation.frame_ms);
            ensure!(animation.render_layer > 0, "water animation '{}' must render above base terrain", animation.id);
            for frame in &animation.frames {
                validate_editor_tile_ref(registry, &editor_atlas_ids, &frame.atlas_id, &frame.tile_id, &animation.id, "water animation frame")?;
            }
        }

        for tool in &pipeline.clipboard_tools {
            ensure!(!tool.id.trim().is_empty(), "editor atlas pipeline '{}' has clipboard tool with empty id", pipeline.id);
            ensure!(
                tool.snap_to_tile_grid || tool.mirror_horizontal || tool.mirror_vertical || tool.rotate_90 || tool.palette_remap || tool.assign_metadata_after_paste,
                "clipboard tool '{}' does not enable any pipeline behavior",
                tool.id
            );
        }

        for profile in &pipeline.game_preview_profiles {
            ensure!(
                known_seasons.contains(&profile.season.as_str()),
                "game preview profile '{}' uses unknown season '{}'",
                profile.id,
                profile.season
            );
        }
    }

    Ok(())
}

fn validate_editor_tile_ref(
    registry: &ContentRegistry,
    editor_atlas_ids: &HashSet<&str>,
    editor_atlas_id: &str,
    tile_id: &str,
    owner_id: &str,
    field: &str,
) -> anyhow::Result<()> {
    ensure!(
        editor_atlas_ids.contains(editor_atlas_id),
        "{} '{}' references unknown editor atlas '{}'",
        field,
        owner_id,
        editor_atlas_id
    );

    let asset_ref = registry
        .editor_atlas_pipelines
        .values()
        .flat_map(|pipeline| pipeline.atlases.iter())
        .find(|atlas| atlas.id == editor_atlas_id)
        .map(|atlas| atlas.asset_ref.as_str())
        .ok_or_else(|| anyhow::anyhow!("{} '{}' could not resolve editor atlas '{}'", field, owner_id, editor_atlas_id))?;

    let tile_exists = registry
        .tilesets
        .get(asset_ref)
        .map(|tileset| tileset.named_tiles.iter().any(|tile| tile.id == tile_id))
        .unwrap_or(false)
        || registry
            .sprite_sheets
            .get(asset_ref)
            .map(|sheet| sheet.entries.iter().any(|entry| entry.id == tile_id))
            .unwrap_or(false);

    ensure!(
        tile_exists,
        "{} '{}' references missing tile '{}' in editor atlas '{}' / asset '{}'",
        field,
        owner_id,
        tile_id,
        editor_atlas_id,
        asset_ref
    );

    Ok(())
}


fn validate_phase20_editor_export_pipeline(registry: &ContentRegistry) -> anyhow::Result<()> {
    let known_severities = ["error", "warning", "info"];
    let known_seasons = ["spring", "summer", "fall", "autumn", "winter", "all"];

    ensure!(
        registry.has_phase19_editor_pipeline(),
        "phase20 editor export pipeline requires the phase19 atlas pipeline to be loaded first"
    );

    for pipeline in registry.editor_export_pipelines.values() {
        ensure!(!pipeline.id.trim().is_empty(), "phase20 editor export pipeline has empty id");
        ensure!(!pipeline.export_profiles.is_empty(), "phase20 pipeline '{}' has no export profiles", pipeline.id);
        ensure!(!pipeline.validation_panels.is_empty(), "phase20 pipeline '{}' has no validation panels", pipeline.id);
        ensure!(!pipeline.autotile_rule_sets.is_empty(), "phase20 pipeline '{}' has no autotile rule sets", pipeline.id);
        ensure!(!pipeline.collision_interaction_profiles.is_empty(), "phase20 pipeline '{}' has no collision/interaction profiles", pipeline.id);
        ensure!(!pipeline.atlas_cleanup_manifests.is_empty(), "phase20 pipeline '{}' has no atlas cleanup manifests", pipeline.id);

        for profile in &pipeline.export_profiles {
            ensure!(!profile.id.trim().is_empty(), "phase20 pipeline '{}' has export profile with empty id", pipeline.id);
            ensure!(!profile.target_root.trim().is_empty(), "export profile '{}' has empty target_root", profile.id);
            ensure!(!profile.include_paths.is_empty(), "export profile '{}' has no include_paths", profile.id);
            ensure!(!profile.required_outputs.is_empty(), "export profile '{}' has no required_outputs", profile.id);
            for output in &profile.required_outputs {
                ensure!(!output.trim().is_empty(), "export profile '{}' contains an empty required output", profile.id);
            }
        }

        for panel in &pipeline.validation_panels {
            ensure!(!panel.id.trim().is_empty(), "phase20 pipeline '{}' has validation panel with empty id", pipeline.id);
            ensure!(!panel.issue_filters.is_empty(), "validation panel '{}' has no issue filters", panel.id);
            ensure!(!panel.jump_targets.is_empty(), "validation panel '{}' has no jump targets", panel.id);
            for filter in &panel.issue_filters {
                ensure!(
                    known_severities.contains(&filter.severity.as_str()),
                    "validation panel '{}' has unknown severity filter '{}'",
                    panel.id,
                    filter.severity
                );
            }
        }

        for rule_set in &pipeline.autotile_rule_sets {
            ensure!(!rule_set.id.trim().is_empty(), "phase20 pipeline '{}' has autotile rule set with empty id", pipeline.id);
            ensure!(rule_set.output_layer > 0, "autotile rule set '{}' must output above the base terrain layer", rule_set.id);
            ensure!(!rule_set.rules.is_empty(), "autotile rule set '{}' has no rules", rule_set.id);
            for rule in &rule_set.rules {
                ensure!(
                    registry.terrain_types.contains_key(&rule.from),
                    "autotile rule '{}' references missing source terrain '{}'",
                    rule.id,
                    rule.from
                );
                ensure!(
                    registry.terrain_types.contains_key(&rule.to),
                    "autotile rule '{}' references missing target terrain '{}'",
                    rule.id,
                    rule.to
                );
                ensure!(rule.neighbor_mask > 0, "autotile rule '{}' must have a non-zero neighbor_mask", rule.id);
                validate_editor_atlas_tile_id(registry, &rule.output_atlas_id, &rule.output_tile_id, &rule.id, "autotile output")?;
            }
        }

        for editor in &pipeline.transition_rule_editors {
            ensure!(!editor.id.trim().is_empty(), "phase20 pipeline '{}' has transition editor with empty id", pipeline.id);
            ensure!(
                registry.transition_sets.contains_key(&editor.rule_set_id),
                "transition rule editor '{}' references missing transition set '{}'",
                editor.id,
                editor.rule_set_id
            );
            ensure!(!editor.editable_fields.is_empty(), "transition rule editor '{}' has no editable fields", editor.id);
            ensure!(!editor.preview_pairs.is_empty(), "transition rule editor '{}' has no preview pairs", editor.id);
            for pair in &editor.preview_pairs {
                ensure!(registry.terrain_types.contains_key(&pair.from), "transition preview '{}' references missing source terrain '{}'", editor.id, pair.from);
                ensure!(registry.terrain_types.contains_key(&pair.to), "transition preview '{}' references missing target terrain '{}'", editor.id, pair.to);
            }
        }

        for profile in &pipeline.collision_interaction_profiles {
            ensure!(!profile.id.trim().is_empty(), "phase20 pipeline '{}' has collision profile with empty id", pipeline.id);
            ensure!(profile.collision_bounds.width > 0 && profile.collision_bounds.height > 0, "collision profile '{}' has invalid bounds", profile.id);
            validate_collision_target(registry, profile)?;
            if let Some(interaction) = &profile.interaction {
                ensure!(!interaction.prompt.trim().is_empty(), "collision profile '{}' has an interaction with empty prompt", profile.id);
                for season in &interaction.season_visibility {
                    ensure!(known_seasons.contains(&season.as_str()), "collision profile '{}' uses unknown season visibility '{}'", profile.id, season);
                }
            }
        }

        for manifest in &pipeline.atlas_cleanup_manifests {
            ensure!(!manifest.id.trim().is_empty(), "phase20 pipeline '{}' has atlas cleanup manifest with empty id", pipeline.id);
            ensure!(resolve_editor_atlas_asset_ref(registry, &manifest.source_atlas_id).is_some(), "cleanup manifest '{}' references missing source atlas '{}'", manifest.id, manifest.source_atlas_id);
            ensure!(resolve_editor_atlas_asset_ref(registry, &manifest.target_atlas_id).is_some(), "cleanup manifest '{}' references missing target atlas '{}'", manifest.id, manifest.target_atlas_id);
            ensure!(!manifest.actions.is_empty(), "cleanup manifest '{}' has no actions", manifest.id);
            for action in &manifest.actions {
                ensure!(!action.id.trim().is_empty(), "cleanup manifest '{}' contains action with empty id", manifest.id);
                ensure!(!action.action.trim().is_empty(), "cleanup action '{}' has empty action", action.id);
                ensure!(!action.reason.trim().is_empty(), "cleanup action '{}' has empty reason", action.id);
            }
        }
    }

    Ok(())
}

fn resolve_editor_atlas_asset_ref<'a>(registry: &'a ContentRegistry, editor_atlas_id: &str) -> Option<&'a str> {
    registry
        .editor_atlas_pipelines
        .values()
        .flat_map(|pipeline| pipeline.atlases.iter())
        .find(|atlas| atlas.id == editor_atlas_id)
        .map(|atlas| atlas.asset_ref.as_str())
}

fn validate_editor_atlas_tile_id(
    registry: &ContentRegistry,
    editor_atlas_id: &str,
    tile_id: &str,
    owner_id: &str,
    field: &str,
) -> anyhow::Result<()> {
    let asset_ref = resolve_editor_atlas_asset_ref(registry, editor_atlas_id)
        .ok_or_else(|| anyhow::anyhow!("{} '{}' references unknown editor atlas '{}'", field, owner_id, editor_atlas_id))?;

    let tile_exists = registry
        .tilesets
        .get(asset_ref)
        .map(|tileset| tileset.named_tiles.iter().any(|tile| tile.id == tile_id))
        .unwrap_or(false)
        || registry
            .sprite_sheets
            .get(asset_ref)
            .map(|sheet| sheet.entries.iter().any(|entry| entry.id == tile_id))
            .unwrap_or(false);

    ensure!(
        tile_exists,
        "{} '{}' references missing tile '{}' in editor atlas '{}' / asset '{}'",
        field,
        owner_id,
        tile_id,
        editor_atlas_id,
        asset_ref
    );

    Ok(())
}

fn validate_collision_target(
    registry: &ContentRegistry,
    profile: &crate::defs::CollisionInteractionProfileDef,
) -> anyhow::Result<()> {
    match profile.target_kind.as_str() {
        "terrain" => ensure!(
            registry.terrain_types.contains_key(&profile.target_id),
            "collision profile '{}' references missing terrain '{}'",
            profile.id,
            profile.target_id
        ),
        "sprite" => ensure!(
            registry
                .sprite_sheets
                .values()
                .any(|sheet| sheet.entries.iter().any(|entry| entry.id == profile.target_id)),
            "collision profile '{}' references missing sprite '{}'",
            profile.id,
            profile.target_id
        ),
        "tile" => ensure!(
            registry
                .tilesets
                .values()
                .any(|tileset| tileset.named_tiles.iter().any(|tile| tile.id == profile.target_id)),
            "collision profile '{}' references missing tile '{}'",
            profile.id,
            profile.target_id
        ),
        other => anyhow::bail!(
            "collision profile '{}' has unsupported target_kind '{}'",
            profile.id,
            other
        ),
    }
    Ok(())
}


fn validate_phase21_editor_animation_pipeline(registry: &ContentRegistry) -> anyhow::Result<()> {
    let known_directions = ["none", "down", "up", "left", "right", "down_left", "down_right", "up_left", "up_right"];
    let known_loop_modes = ["once", "loop", "ping_pong", "hold_last"];
    let known_seasons = ["spring", "summer", "fall", "autumn", "winter", "all"];
    let known_severities = ["error", "warning", "info"];
    let known_target_kinds = ["player", "sprite", "prop", "water", "effect"];

    for pipeline in registry.editor_animation_pipelines.values() {
        ensure!(!pipeline.id.trim().is_empty(), "phase21 animation pipeline has empty id");
        ensure!(pipeline.default_frame_ms > 0, "phase21 pipeline '{}' has invalid default_frame_ms", pipeline.id);
        ensure!(!pipeline.timeline_schemas.is_empty(), "phase21 pipeline '{}' has no timeline schemas", pipeline.id);
        ensure!(!pipeline.animation_clips.is_empty(), "phase21 pipeline '{}' has no animation clips", pipeline.id);
        ensure!(!pipeline.directional_groups.is_empty(), "phase21 pipeline '{}' has no directional animation groups", pipeline.id);
        ensure!(!pipeline.socket_profiles.is_empty(), "phase21 pipeline '{}' has no socket profiles", pipeline.id);
        ensure!(!pipeline.hitbox_profiles.is_empty(), "phase21 pipeline '{}' has no hitbox profiles", pipeline.id);
        ensure!(!pipeline.validation_reports.is_empty(), "phase21 pipeline '{}' has no validation reports", pipeline.id);

        for schema in &pipeline.timeline_schemas {
            ensure!(!schema.id.trim().is_empty(), "phase21 pipeline '{}' has timeline schema with empty id", pipeline.id);
            ensure!(!schema.supported_loop_modes.is_empty(), "timeline schema '{}' has no loop modes", schema.id);
            ensure!(!schema.required_tracks.is_empty(), "timeline schema '{}' has no required tracks", schema.id);
            ensure!(!schema.marker_kinds.is_empty(), "timeline schema '{}' has no marker kinds", schema.id);
            for mode in &schema.supported_loop_modes {
                ensure!(known_loop_modes.contains(&mode.as_str()), "timeline schema '{}' uses unknown loop mode '{}'", schema.id, mode);
            }
        }

        let clip_ids = pipeline.animation_clips.iter().map(|clip| clip.id.as_str()).collect::<HashSet<_>>();
        for clip in &pipeline.animation_clips {
            ensure!(!clip.id.trim().is_empty(), "phase21 pipeline '{}' has animation clip with empty id", pipeline.id);
            ensure!(known_target_kinds.contains(&clip.target_kind.as_str()), "animation clip '{}' has unsupported target_kind '{}'", clip.id, clip.target_kind);
            ensure!(known_directions.contains(&clip.direction.as_str()), "animation clip '{}' uses unknown direction '{}'", clip.id, clip.direction);
            ensure!(known_loop_modes.contains(&clip.loop_mode.as_str()), "animation clip '{}' uses unknown loop mode '{}'", clip.id, clip.loop_mode);
            ensure!(!clip.frames.is_empty(), "animation clip '{}' has no frames", clip.id);
            validate_animation_target(registry, clip)?;

            let frame_indices = clip.frames.iter().map(|frame| frame.index).collect::<HashSet<_>>();
            ensure!(frame_indices.len() == clip.frames.len(), "animation clip '{}' has duplicate frame indices", clip.id);
            for frame in &clip.frames {
                ensure!(frame.duration_ms > 0, "animation clip '{}' frame {} has zero duration", clip.id, frame.index);
                validate_animation_frame_ref(registry, &clip.id, frame)?;
                for event in &frame.events {
                    ensure!(!event.id.trim().is_empty(), "animation clip '{}' frame {} has event with empty id", clip.id, frame.index);
                    ensure!(event.frame_index == frame.index, "animation event '{}' in clip '{}' is stored on frame {} but declares frame_index {}", event.id, clip.id, frame.index, event.frame_index);
                    ensure!(!event.event_kind.trim().is_empty(), "animation event '{}' in clip '{}' has empty event_kind", event.id, clip.id);
                    ensure!(!event.payload.trim().is_empty(), "animation event '{}' in clip '{}' has empty payload", event.id, clip.id);
                }
            }
        }

        for group in &pipeline.directional_groups {
            ensure!(!group.id.trim().is_empty(), "phase21 pipeline '{}' has directional group with empty id", pipeline.id);
            ensure!(!group.directions.is_empty(), "directional group '{}' has no directions", group.id);
            ensure!(known_directions.contains(&group.fallback_direction.as_str()), "directional group '{}' has unknown fallback direction '{}'", group.id, group.fallback_direction);
            let mut has_fallback = false;
            let mut has_right = false;
            for direction in &group.directions {
                ensure!(known_directions.contains(&direction.direction.as_str()), "directional group '{}' uses unknown direction '{}'", group.id, direction.direction);
                ensure!(clip_ids.contains(direction.clip_id.as_str()), "directional group '{}' references missing clip '{}'", group.id, direction.clip_id);
                has_fallback |= direction.direction == group.fallback_direction;
                has_right |= direction.direction == "right";
            }
            ensure!(has_fallback, "directional group '{}' does not include fallback direction '{}'", group.id, group.fallback_direction);
            if group.mirror_left_from_right {
                ensure!(has_right, "directional group '{}' mirrors left from right but has no right direction clip", group.id);
            }
        }

        for profile in &pipeline.socket_profiles {
            ensure!(!profile.id.trim().is_empty(), "phase21 pipeline '{}' has socket profile with empty id", pipeline.id);
            ensure!(clip_ids.contains(profile.clip_id.as_str()), "socket profile '{}' references missing clip '{}'", profile.id, profile.clip_id);
            ensure!(!profile.sockets.is_empty(), "socket profile '{}' has no sockets", profile.id);
            for socket in &profile.sockets {
                ensure!(!socket.socket_id.trim().is_empty(), "socket profile '{}' has socket with empty id", profile.id);
                validate_animation_frame_index(pipeline, &profile.clip_id, socket.frame_index, &profile.id, "socket")?;
            }
        }

        for profile in &pipeline.hitbox_profiles {
            ensure!(!profile.id.trim().is_empty(), "phase21 pipeline '{}' has hitbox profile with empty id", pipeline.id);
            ensure!(clip_ids.contains(profile.clip_id.as_str()), "hitbox profile '{}' references missing clip '{}'", profile.id, profile.clip_id);
            ensure!(!profile.boxes.is_empty(), "hitbox profile '{}' has no boxes", profile.id);
            for hitbox in &profile.boxes {
                ensure!(hitbox.width > 0 && hitbox.height > 0, "hitbox profile '{}' has invalid box bounds", profile.id);
                ensure!(!hitbox.box_kind.trim().is_empty(), "hitbox profile '{}' has hitbox with empty kind", profile.id);
                ensure!(!hitbox.action.trim().is_empty(), "hitbox profile '{}' has hitbox with empty action", profile.id);
                validate_animation_frame_index(pipeline, &profile.clip_id, hitbox.frame_index, &profile.id, "hitbox")?;
            }
        }

        for preview in &pipeline.water_preview_profiles {
            ensure!(!preview.id.trim().is_empty(), "phase21 pipeline '{}' has water preview with empty id", pipeline.id);
            ensure!(preview.tilemap_width > 0 && preview.tilemap_height > 0, "water preview '{}' has invalid tilemap size", preview.id);
            ensure!(known_seasons.contains(&preview.season.as_str()), "water preview '{}' uses unknown season '{}'", preview.id, preview.season);
            let known_water_animation = registry
                .editor_atlas_pipelines
                .values()
                .flat_map(|pipeline| pipeline.water_animations.iter())
                .any(|anim| anim.id == preview.animation_id);
            let known_clip = clip_ids.contains(preview.animation_id.as_str());
            ensure!(known_water_animation || known_clip, "water preview '{}' references missing water animation/clip '{}'", preview.id, preview.animation_id);
        }

        for seasonal in &pipeline.seasonal_animation_sets {
            ensure!(!seasonal.id.trim().is_empty(), "phase21 pipeline '{}' has seasonal animation set with empty id", pipeline.id);
            for clip_id in [&seasonal.spring_clip_id, &seasonal.summer_clip_id, &seasonal.fall_clip_id, &seasonal.winter_clip_id, &seasonal.fallback_clip_id] {
                ensure!(clip_ids.contains(clip_id.as_str()), "seasonal animation set '{}' references missing clip '{}'", seasonal.id, clip_id);
            }
        }

        for report in &pipeline.validation_reports {
            ensure!(!report.id.trim().is_empty(), "phase21 pipeline '{}' has validation report with empty id", pipeline.id);
            ensure!(known_severities.contains(&report.severity.as_str()), "animation validation report '{}' uses unknown severity '{}'", report.id, report.severity);
            ensure!(!report.checks.is_empty(), "animation validation report '{}' has no checks", report.id);
            for check in &report.checks {
                ensure!(!check.trim().is_empty(), "animation validation report '{}' has an empty check", report.id);
            }
        }
    }

    Ok(())
}

fn validate_animation_target(
    registry: &ContentRegistry,
    clip: &crate::defs::AnimationClipDef,
) -> anyhow::Result<()> {
    match clip.target_kind.as_str() {
        "player" => ensure!(!clip.target_id.trim().is_empty(), "animation clip '{}' has empty player target", clip.id),
        "sprite" | "prop" | "effect" => ensure!(
            registry
                .sprite_sheets
                .values()
                .any(|sheet| sheet.entries.iter().any(|entry| entry.id == clip.target_id)),
            "animation clip '{}' references missing sprite/effect target '{}'",
            clip.id,
            clip.target_id
        ),
        "water" => ensure!(
            registry.terrain_types.contains_key(&clip.target_id),
            "animation clip '{}' references missing water terrain '{}'",
            clip.id,
            clip.target_id
        ),
        other => anyhow::bail!("animation clip '{}' has unsupported target kind '{}'", clip.id, other),
    }
    Ok(())
}

fn validate_animation_frame_ref(
    registry: &ContentRegistry,
    clip_id: &str,
    frame: &crate::defs::AnimationFrameDef,
) -> anyhow::Result<()> {
    let in_sprite_sheet = registry
        .sprite_sheets
        .get(&frame.sprite_sheet_id)
        .map(|sheet| sheet.entries.iter().any(|entry| entry.id == frame.sprite_id))
        .unwrap_or(false);
    let in_tileset = registry
        .tilesets
        .get(&frame.sprite_sheet_id)
        .map(|tileset| tileset.named_tiles.iter().any(|entry| entry.id == frame.sprite_id))
        .unwrap_or(false);
    ensure!(
        in_sprite_sheet || in_tileset,
        "animation clip '{}' frame {} references missing sprite/tile '{}' in sheet/tileset '{}'",
        clip_id,
        frame.index,
        frame.sprite_id,
        frame.sprite_sheet_id
    );
    Ok(())
}

fn validate_animation_frame_index(
    pipeline: &crate::defs::EditorAnimationPipelineDef,
    clip_id: &str,
    frame_index: u32,
    owner_id: &str,
    owner_kind: &str,
) -> anyhow::Result<()> {
    let clip = pipeline
        .animation_clips
        .iter()
        .find(|clip| clip.id == clip_id)
        .ok_or_else(|| anyhow::anyhow!("{} profile '{}' references missing clip '{}'", owner_kind, owner_id, clip_id))?;
    ensure!(
        clip.frames.iter().any(|frame| frame.index == frame_index),
        "{} profile '{}' references missing frame {} on clip '{}'",
        owner_kind,
        owner_id,
        frame_index,
        clip_id
    );
    Ok(())
}
