use bevy::prelude::*;
use avian2d::prelude::SpatialQuery;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::asset::RenderAssetUsages;
use crate::helper::audio_settings::AudioSettings;

use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::plasma::PLASMA_EXPAND_SPEED;
// Player type already imported earlier where needed
use crate::game::components::plasma::PLASMA_Z;
use crate::game::components::plasma::PLASMA_IMPACT_PARTICLE_COUNT;
use crate::game::components::plasma::PLASMA_IMPACT_MAX_SPEED;
use crate::game::components::plasma::PLASMA_IMPACT_MIN_SPEED;
use crate::game::components::plasma::PLASMA_IMPACT_LIFETIME_SECS;
use crate::game::components::plasma::PLASMA_BEAM_PARTICLE_WIGGLE_SPEED;
use crate::game::components::plasma::PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE;
use crate::game::components::plasma::PLASMA_BEAM_VISUAL_HALF_HEIGHT;
use crate::game::components::plasma::PLASMA_BEAM_PARTICLE_COUNT;

use crate::game::components::hostile::Hostile;
use crate::game::components::health::Health;
use crate::game::components::animation::{AnimationState, EntityState, HitStateTimer, HIT_STATE_SECONDS};
use crate::game::components::player::Player;
use crate::game::view_api::CombatSoundEffects;
use crate::game::view_api::GameViewEntity;

use crate::game::systems::combat_types::PlasmaBeamParticle;
use crate::game::systems::common::combat_helpers::{
    hash_to_unit,
    create_round_particle_image,
    update_beam_particles,
    spawn_plasma_impact_explosion,
    plasma_origin_from_player,
    ensure_plasma_particle_image,
};

pub(crate) fn update_plasma_beams(
    mut commands: Commands,
    time: Res<Time>,
    audio_settings: Res<AudioSettings>,
    combat_sfx: Option<Res<CombatSoundEffects>>,
    mut images: ResMut<Assets<Image>>,
    mut plasma_particle_image: Local<Option<Handle<Image>>>,
    mut beams: Query<(Entity, &mut PlasmaBeam, &mut Transform, &Children), Without<Player>>,
    player_query: Query<(&Transform, &Sprite), (With<Player>, Without<PlasmaBeam>, Without<PlasmaBeamParticle>)>,
    mut beam_particles: Query<(&PlasmaBeamParticle, &mut Transform, &mut Sprite), (Without<Player>, Without<PlasmaBeam>)>,
    mut health_query: Query<(Entity, &mut Health, &mut AnimationState, &crate::game::components::LevelEntityType), With<Hostile>>,
    mut stats: ResMut<crate::LevelStats>,
) {

    let particle_image = ensure_plasma_particle_image(&mut plasma_particle_image, &mut images);

    for (entity, mut beam, mut transform, children) in &mut beams {
        if !beam.stopped {
            let current_origin = match player_query.get(beam.player_entity) {
                Ok((player_transform, player_sprite)) => {
                    plasma_origin_from_player(player_transform, player_sprite)
                }
                Err(_) => {
                    commands.entity(entity).despawn();
                    continue;
                }
            };

            beam.current_length = (beam.current_length + PLASMA_EXPAND_SPEED * time.delta_secs()).min(beam.max_length);

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
                        if let Ok((target_entity, mut health, mut state, level_entity_type)) = health_query.get_mut(target) {
                            if health.is_dead() {
                                beam.damage_applied = true;
                                continue;
                            }

                            let was_alive = !health.is_dead();
                            health.take_damage(beam.damage);
                            commands.entity(target_entity).insert(super::health_floating::RecentHealthChange(-(beam.damage)));
                            beam.damage_applied = true;

                            if !health.is_dead() {
                                state.set(EntityState::Hit);
                                commands.entity(target_entity).insert(HitStateTimer::new(HIT_STATE_SECONDS, state.version));
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

                            info!("Plasma hit hostile for {} damage - HP: {}/{}", beam.damage, health.current, health.max);
                            stats.hits = stats.hits.saturating_add(1);
                        }
                    }
                }
            }
        } else {
            let remaining = 1.0 - beam.linger_timer.fraction();

            update_beam_particles(&time, &beam, children, &mut beam_particles, remaining);

            beam.linger_timer.tick(time.delta());
            if beam.linger_timer.just_finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}




