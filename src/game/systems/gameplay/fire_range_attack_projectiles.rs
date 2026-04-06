use bevy::prelude::*;
use std::time::Duration;

use crate::game::components::health::Health;
use crate::game::components::player::Player;
use crate::game::components::range_attack::RangeAttack;
use crate::game::systems::gameplay::types::RangeProjectile;
use crate::game::systems::systems_api::GameViewEntity;

const RANGE_PROJECTILE_SIZE: f32 = 14.0;

pub(crate) fn fire_range_attack_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut attackers: Query<(Entity, &Transform, Option<&Health>, &mut RangeAttack), Without<Player>>,
    player_query: Query<(&Transform, &Health), With<Player>>,
) {
    let Ok((player_transform, player_health)) = player_query.single() else {
        return;
    };

    if player_health.is_dead() {
        return;
    }

    let player_position = player_transform.translation.truncate();

    for (attacker_entity, attacker_transform, attacker_health, mut range_attack) in &mut attackers {
        if attacker_health.is_some_and(Health::is_dead) {
            continue;
        }

        let cadence_secs = (range_attack.frequency.max(1.0)) / 1000.0;
        range_attack
            .cooldown
            .set_duration(Duration::from_secs_f32(cadence_secs));
        range_attack.cooldown.tick(time.delta());

        let origin = attacker_transform.translation.truncate();
        if origin.distance(player_position) > range_attack.aggro_range {
            continue;
        }

        if !range_attack.cooldown.is_finished() {
            continue;
        }

        let direction = (player_position - origin).normalize_or_zero();
        if direction == Vec2::ZERO {
            continue;
        }

        range_attack.cooldown.reset();

        let hue = ((attacker_entity.to_bits() as f32) * 47.0) % 360.0;
        let projectile_velocity = direction * range_attack.speed;

        commands.spawn((
            Name::new(format!("RangeProjectile:{}", attacker_entity.index())),
            Sprite {
                color: Color::hsl(hue, 0.82, 0.58),
                custom_size: Some(Vec2::splat(RANGE_PROJECTILE_SIZE)),
                ..default()
            },
            Transform::from_xyz(
                origin.x,
                origin.y,
                attacker_transform.translation.z + 0.2,
            ),
            RangeProjectile::new(
                attacker_entity,
                origin,
                projectile_velocity,
                range_attack.damage,
                range_attack.max_range,
            ),
            GameViewEntity,
        ));
    }
}




