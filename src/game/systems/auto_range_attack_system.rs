use bevy::prelude::*;

use crate::game::components::auto_range_attack::AutoRangeAttack;
use crate::game::components::{Collider, ColliderShape, Damageable, RigidBody, StateMachine, Team};
use crate::game::components::plasma::PlasmaBeam;
use crate::game::gfx::fire_shoot::{ensure_fire_particle_image, FireParticleImage, spawn_fire_shoot_particles};
use crate::game::gfx::plasma_shoot::{ensure_plasma_particle_image, spawn_plasma_beam_particles, PlasmaParticleImage};
use crate::game::gfx::poison::spawn_poison_particles;
use crate::game::gfx::spit::spawn_spit_particles;
use crate::game::runtime_components::{GameEntity, Projectile};
use crate::helper::audio_settings::AudioSettings;
use crate::helper::sounds::spawn_combat_sfx;

const NEUTRAL_TEAM: &str = "Neutral";
const PLASMA_SHOT_SFX: &str = "audio/plasma-shot.ogg";
const PROJECTILE_HALF_EXTENT: f32 = 4.0;

pub fn auto_range_attack_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    mut images: ResMut<Assets<Image>>,
    mut plasma_particle_image: Local<Option<Handle<Image>>>,
    mut fire_particle_image: Local<Option<Handle<Image>>>,
    particle_image_res: Option<Res<PlasmaParticleImage>>,
    fire_particle_image_res: Option<Res<FireParticleImage>>,
    mut attackers: Query<(
        Entity,
        &Transform,
        &Collider,
        &mut AutoRangeAttack,
        Option<&Team>,
        Option<&StateMachine>,
    )>,
    targets: Query<
        (
            Entity,
            &Transform,
            &Collider,
            Option<&Team>,
            Option<&StateMachine>,
        ),
        Without<AutoRangeAttack>,
    >,
    damageable_query: Query<&Damageable>,
) {
    let dt = time.delta();

    for (attacker_entity, attacker_transform, attacker_collider, mut attack, attacker_team, attacker_sm) in
        &mut attackers
    {
        if !attack.enabled {
            continue;
        }

        if let Some(sm) = attacker_sm {
            if sm.is_non_interactive() {
                continue;
            }
        }

        attack.cooldown.tick(dt);
        if !attack.cooldown.just_finished() {
            continue;
        }

        let attacker_center = attacker_transform.translation.truncate() + attacker_collider.offset;
        let attacker_team_name = attacker_team
            .map(|t| t.name.as_str())
            .unwrap_or(NEUTRAL_TEAM);

        let mut best_target: Option<(Entity, Vec2, f32)> = None;

        for (target_entity, target_transform, target_collider, target_team, target_sm) in &targets {
            if target_entity == attacker_entity {
                continue;
            }
            if !damageable_query.contains(target_entity) {
                continue;
            }
            if let Some(sm) = target_sm {
                if sm.is_non_interactive() {
                    continue;
                }
            }

            let target_team_name = target_team
                .map(|t| t.name.as_str())
                .unwrap_or(NEUTRAL_TEAM);
            if attacker_team_name == target_team_name {
                continue;
            }

            let target_center = target_transform.translation.truncate() + target_collider.offset;
            let distance = attacker_center.distance(target_center);
            if distance > attack.aggro_range.max(0.0) {
                continue;
            }

            match best_target {
                Some((_, _, best_distance)) if distance >= best_distance => {}
                _ => best_target = Some((target_entity, target_center, distance)),
            }
        }

        let Some((target_entity, target_center, _)) = best_target else {
            continue;
        };

        let direction = (target_center - attacker_center).normalize_or_zero();
        let direction = if direction.length_squared() > 0.0 {
            direction
        } else {
            Vec2::X
        };

        let projectile_velocity = direction * attack.speed;
        let origin = attacker_transform.translation.truncate();
        let entity_z = attacker_transform.translation.z;
        let shoot_effect = attack.shoot_effect.clone();
        let impact_effect = attack.impact_effect.clone();

        let projectile_entity = commands
            .spawn((
            Name::new("EnemyProjectile"),
            Transform::from_xyz(origin.x, origin.y, entity_z),
            Collider {
                offset: Vec2::ZERO,
                shape: ColliderShape::Rectangle {
                    half_extents: Vec2::splat(PROJECTILE_HALF_EXTENT),
                },
                is_trigger: false,
            },
            RigidBody {
                velocity: projectile_velocity,
                ..default()
            },
            Projectile::new(
                attacker_entity,
                attack.damage,
                attack.speed,
                attack.range,
                shoot_effect.clone(),
                impact_effect,
            ),
            GameEntity,
        ))
            .id();

        // Fire uses its own dedicated particle image; all other effects share the plasma image.
        let particle_image = if shoot_effect
            .as_deref()
            .unwrap_or("plasma_shoot")
            .eq_ignore_ascii_case("fire_shoot")
        {
            if let Some(resource) = fire_particle_image_res.as_ref() {
                resource.0.clone()
            } else {
                ensure_fire_particle_image(&mut fire_particle_image, &mut images)
            }
        } else if let Some(resource) = particle_image_res.as_ref() {
            resource.0.clone()
        } else {
            ensure_plasma_particle_image(&mut plasma_particle_image, &mut images)
        };

        spawn_shoot_effect(
            &mut commands,
            &particle_image,
            shoot_effect.as_deref(),
            origin,
            entity_z,
            entity_seed(attacker_entity),
            direction,
            projectile_velocity,
        );

        if shoot_effect
            .as_deref()
            .unwrap_or("plasma_shoot")
            .eq_ignore_ascii_case("plasma_shoot")
        {
            let mut beam_cmd = commands.spawn((
                Name::new("PlasmaBeam"),
                Transform::from_xyz(origin.x, origin.y, entity_z),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                PlasmaBeam::new(origin, direction.x.signum(), Some(projectile_entity)),
                GameEntity,
            ));
            spawn_plasma_beam_particles(&mut beam_cmd, &particle_image);
        }

        spawn_combat_sfx(
            &mut commands,
            &asset_server,
            &audio_settings,
            PLASMA_SHOT_SFX,
        );

        attack.just_fired = true;

        tracing::debug!(
            attacker = ?attacker_entity,
            target = ?target_entity,
            damage = attack.damage,
            speed = attack.speed,
            range = attack.range,
            "AutoRangeAttack fired projectile"
        );
    }
}

fn entity_seed(entity: Entity) -> u32 {
    (entity.to_bits() & 0xFFFF_FFFF) as u32
}

fn spawn_shoot_effect(
    commands: &mut Commands,
    particle_image: &Handle<Image>,
    shoot_effect: Option<&str>,
    origin: Vec2,
    z: f32,
    seed_base: u32,
    direction: Vec2,
    projectile_velocity: Vec2,
) {
    let Some(effect) = shoot_effect else {
        return;
    };

    if effect.eq_ignore_ascii_case("fire_shoot") {
        spawn_fire_shoot_particles(commands, particle_image, origin, z, seed_base, direction);
    } else if effect.eq_ignore_ascii_case("poison_shoot") {
        spawn_poison_particles(
            commands,
            particle_image,
            origin,
            z,
            seed_base,
            projectile_velocity,
        );
    } else if effect.eq_ignore_ascii_case("spit_shoot") {
        spawn_spit_particles(commands, particle_image, origin, z, seed_base);
    }
}



