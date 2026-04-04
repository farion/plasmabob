use std::collections::HashSet;

use avian2d::prelude::{Collider, CollidingEntities, CollisionLayers, LinearVelocity, LockedAxes, RigidBody, SpatialQuery, SpatialQueryFilter};
use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::query::QueryFilter;
use bevy::math::Dir2;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::game::components::animation::{AnimationState, EntityState, FIGHT_STATE_SECONDS, FightStateTimer, HIT_STATE_SECONDS, HitStateTimer, can_set_state};
use crate::key_bindings::KeyBindings;
use crate::game::components::collision::Collision;
use crate::game::components::exit::Exit;
use crate::game::components::health::{Damage, Health, InvincibilityTimer};
use crate::game::components::hostile::Hostile;
use crate::game::components::npc::Npc;
use crate::game::components::player::{Player, PlasmaAttack};
use crate::game::components::plasma::{
    PlasmaBeam, PLASMA_BEAM_PARTICLE_COUNT, PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE,
    PLASMA_BEAM_PARTICLE_WIGGLE_SPEED, PLASMA_BEAM_VISUAL_HALF_HEIGHT, PLASMA_EXPAND_SPEED,
    PLASMA_IMPACT_LIFETIME_SECS, PLASMA_IMPACT_MAX_SPEED, PLASMA_IMPACT_MIN_SPEED,
    PLASMA_IMPACT_PARTICLE_COUNT, PLASMA_ORIGIN_HEIGHT_RATIO_FROM_BOTTOM, PLASMA_Z,
};
use crate::game::components::{LevelEntityType, SpawnedLevelEntity};
use crate::helper::audio_settings::AudioSettings;
use crate::game::level::CachedLevelDefinition;
use crate::AppState;
use crate::{PendingStoryScreen, StoryScreenRequest};
use crate::LevelStats;

use super::{CombatSoundEffects, GameViewEntity, LevelQuotes, PLAYER_INVINCIBILITY_SECONDS};

#[derive(Component)]
pub(super) struct DeathQuotePlayed;

#[derive(Component)]
pub(super) struct DeathCounted;

#[derive(Component)]
pub(super) struct PlasmaBeamParticle {
    normalized_distance: f32,
    lane: f32,
    phase: f32,
    layer_scale: f32,
}

#[derive(Component)]
pub(super) struct PlasmaImpactParticle {
    velocity: Vec2,
    lifetime: Timer,
    start_size: f32,
}

#[derive(Component)]
pub(super) struct DeadNpcCollisionDisabled;

// stomping behavior removed: players take contact damage immediately on collision.

pub(super) fn tick_invincibility_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut InvincibilityTimer)>,
) {
    for (entity, mut timer) in &mut timers {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).remove::<InvincibilityTimer>();
        }
    }
}

pub(super) fn apply_hostile_contact_damage(
    mut commands: Commands,
    _time: Res<Time>,
    mut hostiles: Query<(
        Entity,
        &Damage,
        Option<&mut Health>,
        &Transform,
        &mut AnimationState,
        Option<&HitStateTimer>,
        Option<&FightStateTimer>,
        Option<&LevelEntityType>,
    ), (With<Hostile>, Without<Player>)>,
    mut player_query: Query<(
        Entity,
        &avian2d::prelude::CollidingEntities,
        &Transform,
        &avian2d::prelude::LinearVelocity,
        &mut Health,
        &mut AnimationState,
        Option<&PlasmaAttack>,
    ), (With<Player>, Without<InvincibilityTimer>, Without<Hostile>)>,
) {
    for (
        player_entity,
        colliding_entities,
        _player_transform,
        _player_velocity,
        mut player_health,
        mut player_state,
        _plasma_attack_opt,
    ) in &mut player_query
    {
        if player_health.is_dead() {
            continue;
        }

        for &colliding_entity in colliding_entities.0.iter() {
            if let Ok((
                hostile_entity,
                damage,
                _hostile_health_opt,
                _hostile_transform,
                mut hostile_state,
                hostile_hit_timer,
                hostile_fight_timer,
                _level_entity_type,
            )) = hostiles.get_mut(colliding_entity)
            {
                // Apply contact damage immediately (stomping removed).
                player_health.take_damage(damage.0);
                // Mark entity for floating health text (negative = damage)
                commands.entity(player_entity).insert(super::health_floating::RecentHealthChange(-(damage.0)));
                commands.entity(player_entity).insert(InvincibilityTimer::new(PLAYER_INVINCIBILITY_SECONDS));

                if !player_health.is_dead() {
                    player_state.set(EntityState::Hit);
                    commands.entity(player_entity).insert(HitStateTimer::new(HIT_STATE_SECONDS, player_state.version));
                }

                if can_set_state(&hostile_state, hostile_hit_timer, hostile_fight_timer, EntityState::Fight) {
                    hostile_state.set(EntityState::Fight);
                    commands.entity(hostile_entity).insert(FightStateTimer::new(FIGHT_STATE_SECONDS));
                }

                info!(
                    "Player took {} damage from hostile - HP: {}/{}",
                    damage.0, player_health.current, player_health.max
                );
                break;
            }
        }
    }
    }

pub(super) fn set_hostile_fight_state_on_player_contact(
    mut commands: Commands,
    player_entities: Query<Entity, With<Player>>,
    mut hostiles: Query<
        (
            Entity,
            &CollidingEntities,
            &mut AnimationState,
            Option<&HitStateTimer>,
            Option<&FightStateTimer>,
            Option<&Health>,
        ),
        (With<Hostile>, Without<Player>),
    >,
) {
    let player_set: HashSet<Entity> = player_entities.iter().collect();

    for (hostile_entity, colliding_entities, mut hostile_state, hit_timer, fight_timer, health) in &mut hostiles {
        if health.is_some_and(|value| value.is_dead()) {
            continue;
        }

        let touches_player = colliding_entities
            .0
            .iter()
            .any(|entity| player_set.contains(entity));

        if !touches_player {
            continue;
        }

        if can_set_state(&hostile_state, hit_timer, fight_timer, EntityState::Fight) {
            hostile_state.set(EntityState::Fight);
            commands
                .entity(hostile_entity)
                .insert(FightStateTimer::new(FIGHT_STATE_SECONDS));
        }
    }
}

/// Despawns any non-player, non-NPC entity whose HP has reached zero.
pub(super) fn despawn_dead_entities(
    mut commands: Commands,
    dead_query: Query<(Entity, &Health), (Without<Player>, Without<Npc>, With<SpawnedLevelEntity>)>,
) {
    for (entity, health) in &dead_query {
        if health.is_dead() {
            info!("Entity {:?} died - despawning.", entity);
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn play_hostile_death_quotes(
    mut commands: Commands,
    time: Res<Time>,
    audio_settings: Res<AudioSettings>,
    quotes: Option<Res<LevelQuotes>>,
    cooldown: Option<ResMut<super::QuoteCooldown>>,
    dead_hostiles: Query<
        (Entity, &Health),
        (
            With<Hostile>,
            With<Npc>,
            With<SpawnedLevelEntity>,
            Without<DeathQuotePlayed>,
        ),
    >,
) {
    let Some(quotes) = quotes else {
        return;
    };

    if quotes.clips.is_empty() {
        return;
    }

    let Some(mut cooldown) = cooldown else {
        // If the cooldown resource isn't present, skip playing quotes to avoid panics.
        return;
    };

    cooldown.0.tick(time.delta());

    for (entity, health) in &dead_hostiles {
        if !health.is_dead() {
            continue;
        }

        // Always mark the entity so it is not processed again, but only
        // play the audio when the cooldown has elapsed.
        commands.entity(entity).insert(DeathQuotePlayed);

        if !cooldown.0.just_finished() {
            continue;
        }

        let random_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as usize)
            .unwrap_or(0)
            .wrapping_add(entity.index_u32() as usize);
        let index = random_seed % quotes.clips.len();
        let quote_handle = quotes.clips[index].clone();

        commands.spawn((
            AudioPlayer::new(quote_handle),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::Linear(audio_settings.quotes_volume),
                ..default()
            },
            GameViewEntity,
        ));

        cooldown.0.reset();
    }
}

/// Count hostile deaths once and record them in LevelStats.
pub(super) fn count_hostile_deaths(
    mut commands: Commands,
    mut stats: ResMut<LevelStats>,
    dead_hostiles: Query<(Entity, &Health), (With<Hostile>, With<SpawnedLevelEntity>, Without<DeathCounted>)>,
) {
    for (entity, health) in &dead_hostiles {
        if !health.is_dead() {
            continue;
        }

        // Increment enemy kill count exactly once per entity and mark it.
        stats.enemies_killed = stats.enemies_killed.saturating_add(1);
        commands.entity(entity).insert(DeathCounted);
    }
}

/// Fires a plasma beam in the player's facing direction when Space is pressed.
pub(super) fn shoot_plasma(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    time: Res<Time>,
    audio_settings: Res<AudioSettings>,
    combat_sfx: Option<Res<CombatSoundEffects>>,
    mut images: ResMut<Assets<Image>>,
    mut plasma_particle_image: Local<Option<Handle<Image>>>,
    spatial_query: SpatialQuery,
    mut players: Query<(
        Entity, &Transform, &Sprite, &Health, &mut PlasmaAttack, &mut AnimationState, Option<&HitStateTimer>),
        With<Player>,
    >,
    collision_query: Query<Entity, With<Collision>>,
    hostile_query: Query<Entity, (With<Hostile>, With<Health>)>,
    mut stats: ResMut<LevelStats>,
) {
    let particle_image = ensure_plasma_particle_image(&mut plasma_particle_image, &mut images);

    for (player_entity, transform, sprite, health, mut plasma_attack, mut state, hit_timer) in &mut players {
        if health.is_dead() {
            continue;
        }

        plasma_attack.cooldown.tick(time.delta());

        if !keys.just_pressed(key_bindings.shoot) || !plasma_attack.cooldown.is_finished() {
            continue;
        }

        plasma_attack.cooldown.reset();
        // record a shot
        stats.shots = stats.shots.saturating_add(1);

        if can_set_state(&state, hit_timer, None, EntityState::Fight) {
            state.set(EntityState::Fight);
        }

        let direction = if sprite.flip_x { -1.0f32 } else { 1.0 };
        let origin = plasma_origin_from_player(transform, sprite);
        let dir2 = if sprite.flip_x { Dir2::NEG_X } else { Dir2::X };

        let filter = SpatialQueryFilter {
            excluded_entities: [player_entity].into_iter().collect(),
            ..default()
        };

        // Collect all ray hits up to attack_range and find the closest Collision entity.
        let hits = spatial_query.ray_hits(origin, dir2, plasma_attack.range, 64, true, &filter);

        let (max_length, target_entity) = hits
            .iter()
            .filter(|hit| collision_query.contains(hit.entity))
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
            .map(|hit| {
                let target = if hostile_query.contains(hit.entity) {
                    Some(hit.entity)
                } else {
                    None
                };
                (hit.distance, target)
            })
            .unwrap_or((plasma_attack.range, None));

        let mut beam_entity = commands.spawn((
            Name::new(format!("PlasmaBeam:{}", player_entity.index())),
            Transform::from_xyz(origin.x, origin.y, PLASMA_Z),
            Visibility::default(),
            PlasmaBeam::new(
                player_entity,
                direction,
                max_length,
                target_entity,
                plasma_attack.damage,
            ),
            GameViewEntity,
        ));

        beam_entity.with_children(|parent| {
            for index in 0..PLASMA_BEAM_PARTICLE_COUNT {
                let seed = index as u32 + 1;
                let normalized_distance = if PLASMA_BEAM_PARTICLE_COUNT <= 1 {
                    1.0
                } else {
                    index as f32 / (PLASMA_BEAM_PARTICLE_COUNT - 1) as f32
                };
                // Cube the lane to keep most particles near the center so the cloud reads as a beam.
                let lane = ((hash_to_unit(seed.wrapping_mul(29)) * 2.0) - 1.0).powi(3);
                let phase = hash_to_unit(seed.wrapping_mul(53)) * std::f32::consts::TAU;
                let core_size = 4.0 + hash_to_unit(seed.wrapping_mul(97)) * 5.0;
                let glow_size = core_size * 2.0;
                let alpha = 0.55 + hash_to_unit(seed.wrapping_mul(11)) * 0.25;

                parent.spawn((
                    Sprite {
                        color: Color::srgba(0.2, 0.98, 1.0, alpha),
                        custom_size: Some(Vec2::splat(core_size)),
                        ..Sprite::from_image(particle_image.clone())
                    },
                    Transform::from_xyz(0.0, 0.0, hash_to_unit(seed.wrapping_mul(7)) * 0.2),
                    PlasmaBeamParticle {
                        normalized_distance,
                        lane,
                        phase,
                        layer_scale: 1.0,
                    },
                ));

                parent.spawn((
                    Sprite {
                        color: Color::srgba(0.12, 0.75, 1.0, alpha * 0.45),
                        custom_size: Some(Vec2::splat(glow_size)),
                        ..Sprite::from_image(particle_image.clone())
                    },
                    Transform::from_xyz(0.0, 0.0, -0.1 + hash_to_unit(seed.wrapping_mul(17)) * 0.15),
                    PlasmaBeamParticle {
                        normalized_distance,
                        lane,
                        phase: phase + 0.9,
                        layer_scale: 1.8,
                    },
                ));
            }
        });

        info!(
            "Plasma beam fired: dir={} max_length={:.0} target={:?}",
            direction, max_length, target_entity
        );

        if let Some(combat_sfx) = combat_sfx.as_ref() {
            commands.spawn((
                AudioPlayer::new(combat_sfx.plasma_shot.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    volume: bevy::audio::Volume::Linear(audio_settings.effects_volume),
                    ..default()
                },
                GameViewEntity,
            ));
        }
    }
}

/// Expands active plasma beams, applies damage on contact, and despawns them after lingering.
pub(super) fn update_plasma_beams(
    mut commands: Commands,
    time: Res<Time>,
    audio_settings: Res<AudioSettings>,
    combat_sfx: Option<Res<CombatSoundEffects>>,
    mut images: ResMut<Assets<Image>>,
    mut plasma_particle_image: Local<Option<Handle<Image>>>,
    mut beams: Query<(Entity, &mut PlasmaBeam, &mut Transform, &Children), Without<Player>>,
    player_query: Query<(
        &Transform, &Sprite),
        (With<Player>, Without<PlasmaBeam>, Without<PlasmaBeamParticle>),
    >,
    mut beam_particles: Query<(
        &PlasmaBeamParticle, &mut Transform, &mut Sprite),
        (Without<Player>, Without<PlasmaBeam>),
    >,
    mut health_query: Query<(Entity, &mut Health, &mut AnimationState, &LevelEntityType), With<Hostile>>,
    mut stats: ResMut<LevelStats>,
) {
    let particle_image = ensure_plasma_particle_image(&mut plasma_particle_image, &mut images);

    for (entity, mut beam, mut transform, children) in &mut beams {
        if !beam.stopped {
            // Follow the player's current world position so the beam origin moves with them.
            let current_origin = match player_query.get(beam.player_entity) {
                Ok((player_transform, player_sprite)) => {
                    plasma_origin_from_player(player_transform, player_sprite)
                }
                Err(_) => {
                    // Player was despawned - remove the orphaned beam.
                    commands.entity(entity).despawn();
                    continue;
                }
            };

            beam.current_length =
                (beam.current_length + PLASMA_EXPAND_SPEED * time.delta_secs()).min(beam.max_length);

            transform.translation.x = current_origin.x;
            transform.translation.y = current_origin.y;

            update_beam_particles(
                &time,
                &beam,
                children,
                &mut beam_particles,
                1.0,
            );

            if beam.current_length >= beam.max_length {
                beam.stopped = true;

                if beam.target_entity.is_some() && !beam.impact_spawned {
                    let impact_position = current_origin + Vec2::new(beam.direction * beam.max_length, 0.0);
                    spawn_plasma_impact_explosion(&mut commands, &particle_image, impact_position);
                    beam.impact_spawned = true;
                }

                if !beam.damage_applied {
                    if let Some(target) = beam.target_entity {
                        if let Ok((target_entity, mut health, mut state, level_entity_type)) =
                            health_query.get_mut(target)
                        {
                            if health.is_dead() {
                                beam.damage_applied = true;
                                continue;
                            }

                            let was_alive = !health.is_dead();
                            health.take_damage(beam.damage);
                            // Mark hostile for floating health text (negative = damage)
                            commands.entity(target_entity).insert(super::health_floating::RecentHealthChange(-(beam.damage)));
                            beam.damage_applied = true;

                            if !health.is_dead() {
                                state.set(EntityState::Hit);
                                commands.entity(target_entity).insert(HitStateTimer::new(
                                    HIT_STATE_SECONDS,
                                    state.version,
                                ));
                            }

                            if was_alive && level_entity_type.0 == "cockroach" {
                                if let Some(combat_sfx) = combat_sfx.as_ref() {
                                    let sound = if health.is_dead() {
                                        combat_sfx.cockroach_die.clone()
                                    } else {
                                        combat_sfx.plasma_hit.clone()
                                    };

                                    commands.spawn((
                                        AudioPlayer::new(sound),
                                        PlaybackSettings {
                                            mode: bevy::audio::PlaybackMode::Despawn,
                                            volume: bevy::audio::Volume::Linear(audio_settings.effects_volume),
                                            ..default()
                                        },
                                        GameViewEntity,
                                    ));
                                }
                            }

                            info!(
                                "Plasma hit hostile for {} damage - HP: {}/{}",
                                beam.damage, health.current, health.max
                            );
                            // record a hit
                            stats.hits = stats.hits.saturating_add(1);
                        }
                    }
                }
            }
        } else {
            // Beam has stopped - fade alpha during linger, then despawn.
            let remaining = 1.0 - beam.linger_timer.fraction();

            update_beam_particles(
                &time,
                &beam,
                children,
                &mut beam_particles,
                remaining,
            );

            beam.linger_timer.tick(time.delta());
            if beam.linger_timer.just_finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub(super) fn update_plasma_impact_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut PlasmaImpactParticle, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut particle, mut transform, mut sprite) in &mut particles {
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();
        particle.velocity *= 0.86;

        particle.lifetime.tick(time.delta());
        let remaining = 1.0 - particle.lifetime.fraction();

        sprite.color = Color::srgba(0.25, 0.95, 1.0, remaining.clamp(0.0, 1.0));
        let size = particle.start_size * (0.6 + remaining.max(0.0));
        sprite.custom_size = Some(Vec2::splat(size));

        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn update_beam_particles<F: QueryFilter>(
    time: &Time,
    beam: &PlasmaBeam,
    children: &Children,
    beam_particles: &mut Query<(&PlasmaBeamParticle, &mut Transform, &mut Sprite), F>,
    alpha_multiplier: f32,
) {
    for child in children.iter() {
        let Ok((particle, mut particle_transform, mut particle_sprite)) = beam_particles.get_mut(child)
        else {
            continue;
        };

        let wave = (time.elapsed_secs() * PLASMA_BEAM_PARTICLE_WIGGLE_SPEED + particle.phase).sin();
        let taper = 1.0 - (particle.normalized_distance * 0.45);
        let y_offset = (particle.lane * PLASMA_BEAM_VISUAL_HALF_HEIGHT)
            + (wave * PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE * taper * particle.layer_scale);

        particle_transform.translation.x = beam.direction * beam.current_length * particle.normalized_distance;
        particle_transform.translation.y = y_offset;

        let core_boost = 1.0 - particle.lane.abs() * 0.45;
        let alpha = (0.35 + core_boost * 0.65) * alpha_multiplier;
        let color = if particle.layer_scale > 1.0 {
            Color::srgba(0.1, 0.75, 1.0, (alpha * 0.5).clamp(0.0, 1.0))
        } else {
            Color::srgba(0.25, 1.0, 1.0, alpha.clamp(0.0, 1.0))
        };
        particle_sprite.color = color;
    }
}

fn spawn_plasma_impact_explosion(
    commands: &mut Commands,
    particle_image: &Handle<Image>,
    impact_position: Vec2,
) {

    for index in 0..PLASMA_IMPACT_PARTICLE_COUNT {
        let seed = index as u32 + 101;
        let angle = hash_to_unit(seed.wrapping_mul(37)) * std::f32::consts::TAU;
        let speed = PLASMA_IMPACT_MIN_SPEED
            + hash_to_unit(seed.wrapping_mul(71)) * (PLASMA_IMPACT_MAX_SPEED - PLASMA_IMPACT_MIN_SPEED);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        let size = 4.0 + hash_to_unit(seed.wrapping_mul(13)) * 8.0;

        commands.spawn((
            Name::new("PlasmaImpactParticle"),
            Sprite {
                image: particle_image.clone(),
                color: Color::srgba(0.45, 1.0, 1.0, 1.0),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(impact_position.x, impact_position.y, PLASMA_Z + 0.5),
            PlasmaImpactParticle {
                velocity,
                lifetime: Timer::from_seconds(PLASMA_IMPACT_LIFETIME_SECS, TimerMode::Once),
                start_size: size,
            },
            GameViewEntity,
        ));
    }

    commands.spawn((
        Name::new("PlasmaImpactFlash"),
        Sprite {
            image: particle_image.clone(),
            color: Color::srgba(0.65, 1.0, 1.0, 0.75),
            custom_size: Some(Vec2::splat(46.0)),
            ..default()
        },
        Transform::from_xyz(impact_position.x, impact_position.y, PLASMA_Z + 0.6),
        PlasmaImpactParticle {
            velocity: Vec2::ZERO,
            lifetime: Timer::from_seconds(PLASMA_IMPACT_LIFETIME_SECS * 0.55, TimerMode::Once),
            start_size: 46.0,
        },
        GameViewEntity,
    ));
}

fn plasma_origin_from_player(transform: &Transform, sprite: &Sprite) -> Vec2 {
    let size = sprite.custom_size.unwrap_or(Vec2::new(96.0, 128.0));
    let y_from_bottom = size.y * PLASMA_ORIGIN_HEIGHT_RATIO_FROM_BOTTOM;
    let y = transform.translation.y - (size.y * 0.5) + y_from_bottom;
    Vec2::new(transform.translation.x, y)
}

fn hash_to_unit(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    value = (value >> ((value >> 28) + 4)) ^ value;
    value = value.wrapping_mul(277_803_737);
    (((value >> 22) ^ value) as f32) / (u32::MAX as f32)
}

fn ensure_plasma_particle_image(
    local_handle: &mut Option<Handle<Image>>,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    if let Some(handle) = local_handle.as_ref() {
        return handle.clone();
    }

    let handle = ensure_plasma_particle_image_from_assets(images);
    *local_handle = Some(handle.clone());
    handle
}

fn ensure_plasma_particle_image_from_assets(images: &mut Assets<Image>) -> Handle<Image> {
    images.add(create_round_particle_image(32))
}

fn create_round_particle_image(size: u32) -> Image {
    let mut data = vec![0u8; (size * size * 4) as usize];
    let center = (size as f32 - 1.0) * 0.5;
    let radius = center.max(1.0);

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt() / radius;
            let softness = (1.0 - distance).clamp(0.0, 1.0);
            let alpha = (softness * softness * 255.0) as u8;

            let index = ((y * size + x) * 4) as usize;
            data[index] = 255;
            data[index + 1] = 255;
            data[index + 2] = 255;
            data[index + 3] = alpha;
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub(super) fn maintain_player_fight_state(
    beams: Query<&PlasmaBeam>,
    mut players: Query<(Entity, &mut AnimationState, Option<&HitStateTimer>), With<Player>>,
) {
    let active_beam_owners: HashSet<Entity> = beams.iter().map(|beam| beam.player_entity).collect();

    for (player_entity, mut state, hit_timer) in &mut players {
        if !active_beam_owners.contains(&player_entity) {
            continue;
        }

        if can_set_state(&state, hit_timer, None, EntityState::Fight) {
            state.set(EntityState::Fight);
        }
    }
}


pub(super) fn detect_player_reached_exit(
    player_query: Query<(&CollidingEntities, &Health), With<Player>>,
    exit_query: Query<(), With<Exit>>,
    cached_level_definition: Option<Res<CachedLevelDefinition>>,
    mut pending_story: Option<ResMut<PendingStoryScreen>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (colliding_entities, health) in &player_query {
        if health.is_dead() {
            continue;
        }

        if colliding_entities
            .0
            .iter()
            .any(|entity| exit_query.contains(*entity))
        {
            info!("Player reached exit - level won.");
            if let (Some(level), Some(pending_story)) = (cached_level_definition.as_ref(), pending_story.as_mut()) {
                if let Ok(level_definition) = level.level_definition() {
                    if let Some(story) = level_definition
                        .story
                        .as_ref()
                        .and_then(|story| story.win.as_ref())
                    {
                        pending_story.set(StoryScreenRequest {
                            text_asset_path: story.text.clone(),
                            background_asset_path: story.background.clone(),
                            continue_to: AppState::WinView,
                        });
                        next_state.set(AppState::StoryView);
                        return;
                    }
                }
            }

            next_state.set(AppState::WinView);
            return;
        }
    }
}

/// Detects when the player collides with a collectible entity. Applies effects (e.g. healing)
/// based on additional components present on the collectible entity and despawns it.
pub(super) fn detect_player_collectibles(
    mut commands: Commands,
    mut player_query: Query<(Entity, &CollidingEntities, &mut Health), With<Player>>,
    collectible_query: Query<(), With<crate::game::components::collectible::Collectible>>,
    effect_heal_query: Query<&crate::game::components::effect_heal::EffectHeal>,
) {
    for (_player_entity, colliding_entities, mut player_health) in &mut player_query {
        if player_health.is_dead() {
            continue;
        }

        for &colliding_entity in colliding_entities.0.iter() {
            // Check if colliding entity is a collectible
            if collectible_query.get(colliding_entity).is_ok() {
                // If it has an EffectHeal component, apply healing.
                if let Ok(effect) = effect_heal_query.get(colliding_entity) {
                    let heal_amount = effect.0;
                    player_health.current = (player_health.current + heal_amount).min(player_health.max);
                    info!("Player healed by {} - HP: {}/{}", heal_amount, player_health.current, player_health.max);
                    // Mark player for floating health text (positive = heal)
                    commands.entity(_player_entity).insert(super::health_floating::RecentHealthChange(heal_amount));
                }

                // Despawn the collectible after applying its effects.
                commands.entity(colliding_entity).despawn();
            }
        }
    }
}

pub(super) fn detect_player_defeated(
    player_query: Query<&Health, With<Player>>,
    cached_level_definition: Option<Res<CachedLevelDefinition>>,
    mut pending_story: Option<ResMut<PendingStoryScreen>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for health in &player_query {
        if health.is_dead() {
            info!("Player defeated - showing lose view.");
            if let (Some(level), Some(pending_story)) = (cached_level_definition.as_ref(), pending_story.as_mut()) {
                if let Ok(level_definition) = level.level_definition() {
                    if let Some(story) = level_definition
                        .story
                        .as_ref()
                        .and_then(|story| story.lose.as_ref())
                    {
                        pending_story.set(StoryScreenRequest {
                            text_asset_path: story.text.clone(),
                            background_asset_path: story.background.clone(),
                            continue_to: AppState::LoseView,
                        });
                        next_state.set(AppState::StoryView);
                        return;
                    }
                }
            }

            next_state.set(AppState::LoseView);
            return;
        }
    }
}

pub(super) fn disable_dead_npc_collisions(
    mut commands: Commands,
    dead_npcs: Query<(Entity, &Health), (With<Npc>, With<Collision>, Without<DeadNpcCollisionDisabled>)>,
) {
    for (entity, health) in &dead_npcs {
        if !health.is_dead() {
            continue;
        }

        commands.entity(entity).remove::<(
            Collision,
            Collider,
            CollidingEntities,
            CollisionLayers,
            RigidBody,
            LinearVelocity,
            LockedAxes,
        )>();
        commands.entity(entity).insert(DeadNpcCollisionDisabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_player_in_fight_while_owned_beam_exists() {
        let mut app = App::new();
        app.add_systems(Update, maintain_player_fight_state);

        let player = app.world_mut().spawn((Player, AnimationState::default())).id();
        app.world_mut()
            .spawn(PlasmaBeam::new(player, 1.0, 100.0, None, 10));

        app.update();

        let state = app
            .world()
            .get::<AnimationState>(player)
            .expect("player should have AnimationState");
        assert_eq!(state.current, EntityState::Fight);
    }

    #[test]
    fn leaves_player_state_when_no_beam_exists() {
        let mut app = App::new();
        app.add_systems(Update, maintain_player_fight_state);

        let player = app.world_mut().spawn((Player, AnimationState::default())).id();

        app.update();

        let state = app
            .world()
            .get::<AnimationState>(player)
            .expect("player should have AnimationState");
        assert_eq!(state.current, EntityState::Default);
    }

    #[test]
    fn sets_win_state_when_player_reaches_exit() {
        let mut app = App::new();
        app.init_resource::<NextState<AppState>>();
        app.add_systems(Update, detect_player_reached_exit);

        let exit = app.world_mut().spawn(Exit).id();
        let mut colliding = CollidingEntities::default();
        colliding.0.insert(exit);
        app.world_mut().spawn((Player, Health::new(10), colliding));

        app.update();

        let next_state = app
            .world()
            .resource::<NextState<AppState>>();
        assert!(matches!(
            *next_state,
            NextState::Pending(AppState::WinView)
        ));
    }

    #[test]
    fn sets_lose_state_when_player_health_is_zero() {
        let mut app = App::new();
        app.init_resource::<NextState<AppState>>();
        app.add_systems(Update, detect_player_defeated);

        app.world_mut().spawn((Player, Health::new(0)));

        app.update();

        let next_state = app
            .world()
            .resource::<NextState<AppState>>();
        assert!(matches!(
            *next_state,
            NextState::Pending(AppState::LoseView)
        ));
    }

    #[test]
    fn removes_collision_components_for_dead_npc() {
        let mut app = App::new();
        app.add_systems(Update, disable_dead_npc_collisions);

        let npc = app.world_mut().spawn((
            Npc,
            Collision,
            Health::new(0),
            Collider::circle(8.0),
            CollidingEntities::default(),
            CollisionLayers::default(),
            RigidBody::Dynamic,
            LinearVelocity::ZERO,
            LockedAxes::ROTATION_LOCKED,
        )).id();

        app.update();

        let entity = app.world().entity(npc);
        assert!(entity.get::<Collision>().is_none());
        assert!(entity.get::<Collider>().is_none());
        assert!(entity.get::<DeadNpcCollisionDisabled>().is_some());
    }

    #[test]
    fn keeps_collision_components_for_alive_npc() {
        let mut app = App::new();
        app.add_systems(Update, disable_dead_npc_collisions);

        let npc = app.world_mut().spawn((
            Npc,
            Collision,
            Health::new(10),
            Collider::circle(8.0),
            CollidingEntities::default(),
            CollisionLayers::default(),
            RigidBody::Dynamic,
            LinearVelocity::ZERO,
            LockedAxes::ROTATION_LOCKED,
        )).id();

        app.update();

        let entity = app.world().entity(npc);
        assert!(entity.get::<Collision>().is_some());
        assert!(entity.get::<Collider>().is_some());
        assert!(entity.get::<DeadNpcCollisionDisabled>().is_none());
    }
}






