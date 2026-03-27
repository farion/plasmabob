use bevy::prelude::*;

use crate::game::components::health::{Damage, Health};
use crate::game::components::hitbox::PolygonHitbox;
use crate::game::components::player::PlasmaAttack;
use crate::game::components::SpawnedLevelEntity;
use crate::DebugRenderSettings;

use super::{DebugOverlayRoot, DebugStatsLabel, GameViewEntity};

pub(super) fn toggle_hitbox_debug_lines(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<DebugRenderSettings>,
    label_query: Query<Entity, With<DebugStatsLabel>>,
    entity_query: Query<
        (Entity, Option<&Health>, Option<&Damage>, Option<&PlasmaAttack>),
        With<SpawnedLevelEntity>,
    >,
) {
    if !keys.just_pressed(KeyCode::KeyL) {
        return;
    }

    toggle_hitbox_lines(&mut debug_settings);

    if debug_settings.show_hitbox_lines {
        // Spawn a stats label for each level entity that has at least one stat.
        for (target, health, damage, plasma) in &entity_query {
            let text = build_stats_text(health, damage, plasma);
            if text.is_empty() {
                continue;
            }
            commands.spawn((
                Text2d::new(text),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.25)),
                Transform::default(),
                DebugStatsLabel { target },
                GameViewEntity,
            ));
        }
    } else {
        for entity in &label_query {
            commands.entity(entity).despawn();
        }
    }
}

fn toggle_hitbox_lines(debug_settings: &mut DebugRenderSettings) {
    debug_settings.show_hitbox_lines = !debug_settings.show_hitbox_lines;
}

/// Keeps stats labels positioned above their target entity and refreshes the text (e.g. current HP).
pub(super) fn update_debug_stats_labels(
    mut commands: Commands,
    debug_settings: Res<DebugRenderSettings>,
    mut labels: Query<(Entity, &DebugStatsLabel, &mut Transform, &mut Text2d)>,
    targets: Query<(
        &GlobalTransform,
        Option<&Health>,
        Option<&Damage>,
        Option<&PlasmaAttack>,
    )>,
) {
    if !debug_settings.show_hitbox_lines {
        return;
    }

    for (label_entity, label, mut transform, mut text) in &mut labels {
        match targets.get(label.target) {
            Ok((target_transform, health, damage, plasma)) => {
                let pos = target_transform.translation();
                transform.translation = Vec3::new(pos.x, pos.y + 80.0, 100.0);
                *text = Text2d::new(build_stats_text(health, damage, plasma));
            }
            Err(_) => {
                // Target entity was despawned - remove the dangling label.
                commands.entity(label_entity).despawn();
            }
        }
    }
}

/// Formats the available stats of an entity into a human-readable debug string.
fn build_stats_text(
    health: Option<&Health>,
    damage: Option<&Damage>,
    plasma: Option<&PlasmaAttack>,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(hp) = health {
        parts.push(format!("HP {}/{}", hp.current, hp.max));
    }

    // If the entity has PlasmaAttack, show its range and damage (covers the player).
    // Otherwise fall back to the contact-damage component (hostile NPCs).
    if let Some(p) = plasma {
        parts.push(format!("ATK-RNG {:.0}", p.range));
        parts.push(format!("DMG {}", p.damage));
    } else if let Some(dmg) = damage {
        parts.push(format!("DMG {}", dmg.0));
    }

    parts.join("\n")
}

pub(super) fn toggle_debug_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<DebugRenderSettings>,
    mut overlay: Query<&mut Visibility, With<DebugOverlayRoot>>,
) {
    if keys.just_pressed(KeyCode::KeyO) {
        debug_settings.show_overlay = !debug_settings.show_overlay;
        let visibility = if debug_settings.show_overlay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        for mut vis in &mut overlay {
            *vis = visibility;
        }
    }
}

pub(super) fn draw_hitbox_debug_lines(
    debug_settings: Res<DebugRenderSettings>,
    mut gizmos: Gizmos,
    hitboxes: Query<(&GlobalTransform, &PolygonHitbox, Option<&Sprite>), With<SpawnedLevelEntity>>,
) {
    if !debug_settings.show_hitbox_lines {
        return;
    }

    for (transform, polygon_hitbox, sprite) in &hitboxes {
        let effective_points = polygon_hitbox.effective_points(sprite.map(|sprite| sprite.flip_x).unwrap_or(false));

        if effective_points.len() < 2 {
            continue;
        }

        for edge_start in 0..effective_points.len() {
            let edge_end = (edge_start + 1) % effective_points.len();
            let start = transform.transform_point(effective_points[edge_start].extend(0.0));
            let end = transform.transform_point(effective_points[edge_end].extend(0.0));
            gizmos.line(start, end, Color::srgb(1.0, 0.0, 0.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_hitbox_debug_lines() {
        let mut settings = DebugRenderSettings {
            show_hitbox_lines: false,
            show_overlay: false,
        };

        toggle_hitbox_lines(&mut settings);
        assert!(settings.show_hitbox_lines);

        toggle_hitbox_lines(&mut settings);
        assert!(!settings.show_hitbox_lines);
    }
}





