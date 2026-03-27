use std::collections::HashSet;

use avian2d::prelude::SpatialQuery;
use avian2d::prelude::SpatialQueryFilter;
use bevy::audio::{AudioPlayer, PlaybackSettings};
use bevy::math::Dir2;
use bevy::prelude::*;

use crate::game::components::animation::{AnimationState, EntityState, HIT_STATE_SECONDS, HitStateTimer, can_set_state};
use crate::game::components::collision::Collision;
use crate::game::components::health::{Damage, Health, InvincibilityTimer};
use crate::game::components::hostile::Hostile;
use crate::game::components::npc::Npc;
use crate::game::components::player::{Player, PlasmaAttack};
use crate::game::components::plasma::{PlasmaBeam, PLASMA_BEAM_HEIGHT, PLASMA_EXPAND_SPEED, PLASMA_Z};
use crate::game::components::SpawnedLevelEntity;
use crate::AppState;

use super::{GameViewEntity, LevelQuotes, PLAYER_INVINCIBILITY_SECONDS};

#[derive(Component)]
pub(super) struct DeathQuotePlayed;

pub(super) fn tick_invincibility_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut InvincibilityTimer)>,
) {
    for (entity, mut timer) in &mut timers {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.entity(entity).remove::<InvincibilityTimer>();
        }
    }
}

pub(super) fn apply_hostile_contact_damage(
    mut commands: Commands,
    hostile_query: Query<(&Damage, Option<&Health>), (With<Hostile>, Without<Player>)>,
    mut hostile_states: Query<
        (&mut AnimationState, Option<&HitStateTimer>),
        (With<Hostile>, Without<Player>),
    >,
    mut player_query: Query<
        (
            Entity,
            &avian2d::prelude::CollidingEntities,
            &mut Health,
            &mut AnimationState,
        ),
        (With<Player>, Without<InvincibilityTimer>, Without<Hostile>),
    >,
) {
    for (player_entity, colliding_entities, mut health, mut player_state) in &mut player_query {
        if health.is_dead() {
            continue;
        }

        for &colliding_entity in colliding_entities.0.iter() {
            if let Ok((damage, hostile_health)) = hostile_query.get(colliding_entity) {
                if hostile_health.is_some_and(|value| value.is_dead()) {
                    continue;
                }

                health.take_damage(damage.0);
                commands
                    .entity(player_entity)
                    .insert(InvincibilityTimer::new(PLAYER_INVINCIBILITY_SECONDS));

                if !health.is_dead() {
                    player_state.set(EntityState::Hit);
                    commands
                        .entity(player_entity)
                        .insert(HitStateTimer::new(HIT_STATE_SECONDS, player_state.version));
                }

                if let Ok((mut hostile_state, hostile_hit_timer)) = hostile_states.get_mut(colliding_entity)
                {
                    if can_set_state(&hostile_state, hostile_hit_timer, EntityState::Fight) {
                        hostile_state.set(EntityState::Fight);
                    }
                }

                info!(
                    "Player took {} damage from hostile - HP: {}/{}",
                    damage.0, health.current, health.max
                );
                break;
            }
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
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub(super) fn play_hostile_death_quotes(
    mut commands: Commands,
    quotes: Option<Res<LevelQuotes>>,
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

    for (entity, health) in &dead_hostiles {
        if !health.is_dead() {
            continue;
        }

        let random_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as usize)
            .unwrap_or(0)
            .wrapping_add(entity.index() as usize);
        let index = random_seed % quotes.clips.len();
        let quote_handle = quotes.clips[index].clone();

        commands.spawn((
            AudioPlayer::new(quote_handle),
            PlaybackSettings::DESPAWN,
            GameViewEntity,
        ));
        commands.entity(entity).insert(DeathQuotePlayed);
    }
}

/// Fires a plasma beam in the player's facing direction when Space is pressed.
pub(super) fn shoot_plasma(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut players: Query<
        (Entity, &Transform, &Sprite, &Health, &mut PlasmaAttack, &mut AnimationState, Option<&HitStateTimer>),
        With<Player>,
    >,
    collision_query: Query<Entity, With<Collision>>,
    hostile_query: Query<Entity, (With<Hostile>, With<Health>)>,
) {
    for (player_entity, transform, sprite, health, mut plasma_attack, mut state, hit_timer) in &mut players {
        if health.is_dead() {
            continue;
        }

        plasma_attack.cooldown.tick(time.delta());

        if !keys.just_pressed(KeyCode::Space) || !plasma_attack.cooldown.finished() {
            continue;
        }

        plasma_attack.cooldown.reset();

        if can_set_state(&state, hit_timer, EntityState::Fight) {
            state.set(EntityState::Fight);
        }

        let direction = if sprite.flip_x { -1.0f32 } else { 1.0 };
        let origin = transform.translation.truncate();
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

        commands.spawn((
            Name::new(format!("PlasmaBeam:{}", player_entity.index())),
            Sprite {
                color: Color::srgb(0.0, 0.85, 1.0),
                custom_size: Some(Vec2::new(1.0, PLASMA_BEAM_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(origin.x, origin.y, PLASMA_Z),
            PlasmaBeam::new(
                player_entity,
                direction,
                max_length,
                target_entity,
                plasma_attack.damage,
            ),
            GameViewEntity,
        ));

        info!(
            "Plasma beam fired: dir={} max_length={:.0} target={:?}",
            direction, max_length, target_entity
        );
    }
}

/// Expands active plasma beams, applies damage on contact, and despawns them after lingering.
pub(super) fn update_plasma_beams(
    mut commands: Commands,
    time: Res<Time>,
    mut beams: Query<(Entity, &mut PlasmaBeam, &mut Transform, &mut Sprite), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<PlasmaBeam>)>,
    mut health_query: Query<(Entity, &mut Health, &mut AnimationState), With<Hostile>>,
) {
    for (entity, mut beam, mut transform, mut sprite) in &mut beams {
        if !beam.stopped {
            // Follow the player's current world position so the beam origin moves with them.
            let current_origin = match player_query.get(beam.player_entity) {
                Ok(player_transform) => player_transform.translation.truncate(),
                Err(_) => {
                    // Player was despawned - remove the orphaned beam.
                    commands.entity(entity).despawn();
                    continue;
                }
            };

            beam.current_length =
                (beam.current_length + PLASMA_EXPAND_SPEED * time.delta_secs()).min(beam.max_length);

            transform.translation.x = current_origin.x + beam.direction * (beam.current_length * 0.5);
            transform.translation.y = current_origin.y;

            sprite.custom_size = Some(Vec2::new(beam.current_length, PLASMA_BEAM_HEIGHT));

            if beam.current_length >= beam.max_length {
                beam.stopped = true;

                if !beam.damage_applied {
                    if let Some(target) = beam.target_entity {
                        if let Ok((target_entity, mut health, mut state)) = health_query.get_mut(target) {
                            if health.is_dead() {
                                beam.damage_applied = true;
                                continue;
                            }

                            health.take_damage(beam.damage);
                            beam.damage_applied = true;

                            if !health.is_dead() {
                                state.set(EntityState::Hit);
                                commands.entity(target_entity).insert(HitStateTimer::new(
                                    HIT_STATE_SECONDS,
                                    state.version,
                                ));
                            }

                            info!(
                                "Plasma hit hostile for {} damage - HP: {}/{}",
                                beam.damage, health.current, health.max
                            );
                        }
                    }
                }
            }
        } else {
            // Beam has stopped - fade alpha during linger, then despawn.
            let remaining = 1.0 - beam.linger_timer.fraction();
            sprite.color = Color::srgba(0.0, 0.85, 1.0, remaining);

            beam.linger_timer.tick(time.delta());
            if beam.linger_timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
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

        if can_set_state(&state, hit_timer, EntityState::Fight) {
            state.set(EntityState::Fight);
        }
    }
}

pub(super) fn return_to_main_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
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
}



