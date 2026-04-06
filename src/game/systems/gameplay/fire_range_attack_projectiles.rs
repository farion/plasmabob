use std::time::Duration;

use bevy::prelude::*;

use crate::game::components::animation::{AnimationPlayback, PreloadedAnimations};
use crate::game::components::health::Health;
use crate::game::components::player::Player;
use crate::game::components::range_attack::RangeAttack;
use crate::game::components::{AnimationCatalog, AnimationFrameDurations};
use crate::game::systems::gameplay::types::{
    ProjectileEmitter, ProjectileParticleKind, RangeProjectile, RangeProjectileVisual,
};
use crate::game::systems::systems_api::GameViewEntity;

const RANGE_PROJECTILE_SIZE: f32 = 16.0;
const PARTICLE_EMIT_INTERVAL_SECS: f32 = 0.06;

pub(crate) fn fire_range_attack_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut attackers: Query<(
        Entity,
        &Transform,
        Option<&Health>,
        &mut RangeAttack,
        &mut crate::game::components::animation::AnimationState,
        Option<&crate::game::components::animation::HitStateTimer>,
        Option<&crate::game::components::animation::FightStateTimer>,
        Option<&crate::game::components::animation::MeleeAttackStateTimer>,
        Option<&crate::game::components::animation::RangeAttackStateTimer>,
    ), Without<Player>>,
    player_query: Query<(&Transform, &Health), With<Player>>,
) {
    let Ok((player_transform, player_health)) = player_query.single() else {
        return;
    };

    if player_health.is_dead() {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    for (
        attacker_entity,
        attacker_transform,
        attacker_health,
        mut range_attack,
        mut animation_state,
        hit_timer,
        fight_timer,
        melee_timer,
        range_timer,
    ) in &mut attackers {
        if attacker_health.is_some_and(Health::is_dead) {
            continue;
        }

        let cadence_secs = range_attack.frequency.max(1.0) / 1000.0;
        range_attack
            .cooldown
            .set_duration(Duration::from_secs_f32(cadence_secs));
        range_attack.cooldown.tick(time.delta());

        let origin = attacker_transform.translation.truncate();
        if origin.distance(player_pos) > range_attack.aggro_range {
            continue;
        }

        if !range_attack.cooldown.is_finished() {
            continue;
        }

        let direction = (player_pos - origin).normalize_or_zero();
        if direction == Vec2::ZERO {
            continue;
        }

        range_attack.cooldown.reset();

        let velocity = direction * range_attack.speed;
        let z = attacker_transform.translation.z + 0.2;

        spawn_projectile(
            &mut commands,
            &asset_server,
            attacker_entity,
            origin,
            z,
            velocity,
            &range_attack,
        );

        // Set the attacker's animation state to RangeAttack for a short duration
        // so the entity uses its `range_attack` animation (if present).
        if crate::game::components::animation::can_set_state(
            &animation_state,
            hit_timer,
            fight_timer,
            melee_timer,
            range_timer,
            crate::game::components::animation::EntityState::RangeAttack,
        ) {
            animation_state.set(crate::game::components::animation::EntityState::RangeAttack);
            commands.entity(attacker_entity).insert(
                crate::game::components::animation::RangeAttackStateTimer::new(
                    crate::game::components::animation::RANGE_ATTACK_STATE_SECONDS,
                ),
            );
        }
    }
}

fn spawn_projectile(
    commands: &mut Commands,
    asset_server: &AssetServer,
    shooter: Entity,
    origin: Vec2,
    z: f32,
    velocity: Vec2,
    range_attack: &RangeAttack,
) {
    let transform = Transform::from_xyz(origin.x, origin.y, z);
    let projectile = RangeProjectile::new(
        shooter,
        origin,
        velocity,
        range_attack.damage,
        range_attack.max_range,
    );

    if !range_attack.animation.is_empty() {
        let paths = range_attack.animation.clone();
        let frame_secs = range_attack
            .animation_frame_ms
            .map(|ms| (ms as f32 / 1000.0).max(0.001))
            .unwrap_or(0.15);

        let mut catalog_map = std::collections::HashMap::new();
        catalog_map.insert("default".to_string(), paths.clone());

        let mut dur_map = std::collections::HashMap::new();
        dur_map.insert("default".to_string(), frame_secs);

        let preloaded = PreloadedAnimations::from_paths(asset_server, &catalog_map);

        let sprite = paths
            .first()
            .map(|p| {
                let mut s = Sprite::from_image(asset_server.load(p.clone()));
                s.custom_size = Some(Vec2::splat(RANGE_PROJECTILE_SIZE));
                s
            })
            .unwrap_or_else(|| Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                custom_size: Some(Vec2::splat(RANGE_PROJECTILE_SIZE)),
                ..default()
            });

        commands.spawn((
            Name::new(format!("RangeProjectile:{}", shooter.index())),
            sprite,
            transform,
            projectile,
            GameViewEntity,
            RangeProjectileVisual,
            AnimationCatalog(catalog_map),
            AnimationFrameDurations(dur_map),
            preloaded,
            AnimationPlayback::new(frame_secs),
        ));
    } else if let Some(effect) = &range_attack.particle_effect {
        let (base_color, kind) = match effect.as_str() {
            "fire" => (Color::srgba(1.0, 0.35, 0.05, 1.0), ProjectileParticleKind::Fire),
            "poison" => (Color::srgba(0.35, 0.95, 0.35, 1.0), ProjectileParticleKind::Poison),
            "spit" => (Color::srgba(1.0, 0.72, 0.2, 1.0), ProjectileParticleKind::Spit),
            _ => (Color::srgba(1.0, 1.0, 1.0, 1.0), ProjectileParticleKind::Fire),
        };

        commands.spawn((
            Name::new(format!("RangeProjectile:{}", shooter.index())),
            Sprite {
                // keep RGB for potential debugging but the sprite is hidden via Visibility so
                // only particles are rendered.
                color: base_color,
                custom_size: Some(Vec2::splat(RANGE_PROJECTILE_SIZE)),
                ..default()
            },
            Visibility::Hidden,
            transform,
            projectile,
            GameViewEntity,
            ProjectileEmitter {
                kind,
                timer: Timer::from_seconds(PARTICLE_EMIT_INTERVAL_SECS, TimerMode::Repeating),
            },
        ));
    } else {
        let hue = (shooter.to_bits() as f32 * 47.0) % 360.0;

        // No animation and no particle effect: spawn an invisible sprite so the projectile
        // entity still has a visible transform but does not render as a colored quad.
        commands.spawn((
            Name::new(format!("RangeProjectile:{}", shooter.index())),
            Sprite {
                // spawn visible-color data but hide the sprite so it doesn't render as a quad
                color: Color::hsl(hue, 0.82, 0.58),
                custom_size: Some(Vec2::splat(RANGE_PROJECTILE_SIZE)),
                ..default()
            },
            Visibility::Hidden,
            transform,
            projectile,
            GameViewEntity,
        ));
    }
}
