use std::collections::HashMap;

use bevy::prelude::{Camera, Color, GlobalTransform, Query, Vec2, Window, With};
use bevy::window::PrimaryWindow;

use crate::core::io::assets_dir;
use crate::core::{
    normalize_asset_reference, ComponentsDefinition, EntityDefinition, EntityTypeDefinition,
    LevelFile,
};

use crate::level::state::EditorDocument;

// Updated Z-layer presets and colors per user request:
// 150 - Foreground -> red
// 100 - Gameplay -> green
// 50  - Near Player Background -> orange
// 0   - Background -> blue
pub const Z_LAYER_PRESETS: [(&str, f32, [u8; 3]); 4] = [
    ("Foreground", 150.0, [255, 0, 0]),
    ("Gameplay", 100.0, [0, 255, 0]),
    ("Near Player Background", 50.0, [255, 165, 0]),
    ("Background", 0.0, [0, 0, 255]),
];

pub fn z_overlay_color_for_value(z: f32) -> Color {
    let mut layers: Vec<(f32, [u8; 3])> = Z_LAYER_PRESETS
        .iter()
        .map(|(_, value, rgb)| (*value, *rgb))
        .collect();
    layers.sort_by(|left, right| left.0.total_cmp(&right.0));

    if z <= layers[0].0 {
        let [r, g, b] = layers[0].1;
        return Color::srgba_u8(r, g, b, 110);
    }

    if z >= layers[layers.len() - 1].0 {
        let [r, g, b] = layers[layers.len() - 1].1;
        return Color::srgba_u8(r, g, b, 110);
    }

    for pair in layers.windows(2) {
        let (z0, [r0, g0, b0]) = pair[0];
        let (z1, [r1, g1, b1]) = pair[1];
        if z <= z1 {
            let factor = ((z - z0) / (z1 - z0)).clamp(0.0, 1.0);
            let lerp = |a: u8, b: u8| -> u8 {
                let af = a as f32;
                let bf = b as f32;
                (af + (bf - af) * factor).round() as u8
            };

            return Color::srgba_u8(lerp(r0, r1), lerp(g0, g1), lerp(b0, b1), 110);
        }
    }

    // Fallback
    Color::srgba(1.0, 1.0, 1.0, 0.43)
}

pub fn flatten_entity_components(
    components: Option<&ComponentsDefinition>,
) -> HashMap<String, serde_json::Value> {
    let mut flat = HashMap::new();
    let Some(components) = components else {
        return flat;
    };

    let Some(components) = serde_json::to_value(components)
        .ok()
        .and_then(|v| v.as_object().cloned())
    else {
        return flat;
    };

    for (component_name, component_value) in components {
        let Some(component_object) = component_value.as_object() else {
            continue;
        };
        for (attribute_name, attribute_value) in component_object {
            flat.insert(
                format!("{component_name}.{attribute_name}"),
                attribute_value.clone(),
            );
        }
    }

    flat
}

pub fn apply_flat_component_updates(
    entity: &mut EntityDefinition,
    removals: &std::collections::HashSet<String>,
    updates: HashMap<String, serde_json::Value>,
) {
    for key in removals {
        let Some((component, attribute)) = key.split_once('.') else {
            continue;
        };
        entity.remove_component_attribute(component, attribute);
    }

    for (key, value) in updates {
        let Some((component, attribute)) = key.split_once('.') else {
            continue;
        };
        entity.set_component_attribute_value(component, attribute, value);
    }
}

pub fn is_player_entity_type(entity_type: &EntityTypeDefinition) -> bool {
    entity_type
        .category_tag
        .as_deref()
        .map(|tag| tag.eq_ignore_ascii_case("player"))
        .unwrap_or(false)
        || entity_type.has_component("controlled_movement")
}

pub fn resolve_character_asset_path(
    asset_server: &bevy::asset::AssetServer,
    asset_path: &str,
    active: crate::level::run::ActiveCharacter,
) -> Result<String, std::io::Error> {
    let _ = asset_server;
    let normalized = normalize_asset_reference(asset_path);

    let fs_exact = assets_dir().join(&normalized);
    if std::fs::metadata(&fs_exact).is_ok() {
        tracing::warn!("using exact asset: {}", normalized);
        return Ok(normalized);
    }

    if let Some(pos) = normalized.rfind('.') {
        let (before_ext, ext) = normalized.split_at(pos);
        if before_ext.ends_with(".bob") || before_ext.ends_with(".betty") {
            return Ok(normalized);
        }
        let suf = match active {
            crate::level::run::ActiveCharacter::Bob => "bob",
            crate::level::run::ActiveCharacter::Betty => "betty",
        };
        let suffixed = format!("{}.{suf}{}", before_ext, ext);
        let fs_suff = assets_dir().join(&suffixed);
        if std::fs::metadata(&fs_suff).is_ok() {
            tracing::warn!("using character-suffixed asset: {}", suffixed);
            return Ok(suffixed);
        }
    }

    tracing::warn!("asset not found, returning normalized path: {}", normalized);
    Ok(normalized)
}

pub fn is_inside_level_bounds(pos: Vec2, level: &LevelFile) -> bool {
    let Some(bounds) = &level.bounds else {
        return false;
    };
    pos.x >= 0.0 && pos.x <= bounds.width && pos.y >= 0.0 && pos.y <= bounds.height
}

pub fn entity_render_center(entity_bottom_left: Vec2, size: Vec2) -> Vec2 {
    Vec2::new(
        entity_bottom_left.x + size.x * 0.5,
        entity_bottom_left.y + size.y * 0.5,
    )
}

// Distance in world units within which edges/corners will snap together.
pub const SNAP_THRESHOLD: f32 = 40.0;

pub fn apply_snapping(document: &mut EditorDocument, index: usize, snap_enabled: bool) {
    if !snap_enabled {
        return;
    }

    let Some(entity) = document.level.entities.get(index).cloned() else {
        return;
    };

    let size = document
        .entity_types
        .get(&entity.entity_type)
        .map(|et| et.size())
        .unwrap_or(Vec2::ZERO);

    let a_left = entity.x;
    let a_right = entity.x + size.x;
    let a_bottom = entity.y;
    let a_top = entity.y + size.y;

    let mut best_dx: Option<(f32, usize)> = None;
    let mut best_dy: Option<(f32, usize)> = None;

    for (j, other) in document.level.entities.iter().enumerate() {
        if j == index {
            continue;
        }
        let Some(ot) = document.entity_types.get(&other.entity_type) else {
            continue;
        };
        let os = ot.size();
        let b_left = other.x;
        let b_right = other.x + os.x;
        let b_bottom = other.y;
        let b_top = other.y + os.y;

        let hx = [
            b_right - a_left,
            b_left - a_right,
            b_left - a_left,
            b_right - a_right,
        ];
        for &dx in &hx {
            if dx.abs() <= SNAP_THRESHOLD {
                if best_dx.is_none() || dx.abs() < best_dx.unwrap().0.abs() {
                    best_dx = Some((dx, j));
                }
            }
        }

        let hy = [
            b_top - a_bottom,
            b_bottom - a_top,
            b_bottom - a_bottom,
            b_top - a_top,
        ];
        for &dy in &hy {
            if dy.abs() <= SNAP_THRESHOLD {
                if best_dy.is_none() || dy.abs() < best_dy.unwrap().0.abs() {
                    best_dy = Some((dy, j));
                }
            }
        }
    }

    if best_dx.is_none() && best_dy.is_none() {
        return;
    }

    let mut corner_dy: Option<f32> = None;
    if let Some((dx, other_idx)) = best_dx {
        if dx.abs() > f32::EPSILON {
            if let Some(e) = document.level.entities.get_mut(index) {
                e.x += dx;
            }
        }

        if let Some(other) = document.level.entities.get(other_idx) {
            if let Some(ot) = document.entity_types.get(&other.entity_type) {
                let os = ot.size();
                let _b_left = other.x;
                let _b_right = other.x + os.x;
                let b_bottom = other.y;
                let b_top = other.y + os.y;

                if let Some(e) = document.level.entities.get(index) {
                    let a_bottom = e.y;
                    let a_top = e.y + size.y;

                    let corner_candidates = [
                        b_top - a_top,
                        b_bottom - a_bottom,
                        b_top - a_bottom,
                        b_bottom - a_top,
                    ];
                    for &dy in &corner_candidates {
                        if dy.abs() <= SNAP_THRESHOLD {
                            if corner_dy.is_none() || dy.abs() < corner_dy.unwrap().abs() {
                                corner_dy = Some(dy);
                            }
                        }
                    }
                }
            }
        }
    }

    let dy_to_apply = corner_dy.or(best_dy.map(|t| t.0));
    if let Some(dy) = dy_to_apply {
        if dy.abs() > f32::EPSILON {
            if let Some(e) = document.level.entities.get_mut(index) {
                e.y += dy;
            }
        }
    }
}

pub fn topmost_entity_at_position(
    world_position: Vec2,
    level: &LevelFile,
    entity_types: &HashMap<String, EntityTypeDefinition>,
) -> Option<(usize, Vec2)> {
    let mut best_hit: Option<(usize, f32, Vec2)> = None;

    for (index, entity) in level.entities.iter().enumerate() {
        let Some(entity_type) = entity_types.get(&entity.entity_type) else {
            continue;
        };
        let size = entity_type.size();
        let contains_point = world_position.x >= entity.x
            && world_position.x <= entity.x + size.x
            && world_position.y >= entity.y
            && world_position.y <= entity.y + size.y;

        if !contains_point {
            continue;
        }

        let z = entity.z_index.unwrap_or(0.0);
        match best_hit {
            Some((_, best_z, _)) if best_z > z => {}
            _ => {
                best_hit = Some((index, z, Vec2::new(entity.x, entity.y)));
            }
        }
    }

    best_hit.map(|(index, _, position)| (index, position))
}

pub fn camera_center_world(
    camera_query: &Query<(&Camera, &GlobalTransform), With<crate::level::run::EditorCamera>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec2> {
    let window = window_query.single().ok()?;
    let (camera, camera_transform) = camera_query.single().ok()?;
    let viewport_center = Vec2::new(window.width() * 0.5, window.height() * 0.5);
    camera
        .viewport_to_world_2d(camera_transform, viewport_center)
        .ok()
}
