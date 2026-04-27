use crate::state::{AppMode, BootstrapState, InteractionMode};
use game_data::registry::ContentRegistry;

pub fn bootstrap_state(registry: &ContentRegistry) -> BootstrapState {
    let active_map_id = preferred_start_map(registry);

    BootstrapState {
        app_mode: AppMode::InGame,
        interaction_mode: InteractionMode::Play,
        active_map_id,
    }
}

fn preferred_start_map(registry: &ContentRegistry) -> String {
    for candidate in ["starter_farm", "farm", "town"] {
        if registry.maps.contains_key(candidate) {
            return candidate.to_string();
        }
    }

    let mut map_ids = registry.maps.keys().cloned().collect::<Vec<_>>();
    map_ids.sort();
    map_ids.into_iter().next().unwrap_or_else(|| "starter_farm".to_string())
}
