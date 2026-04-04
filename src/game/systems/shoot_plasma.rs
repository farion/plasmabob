use avian2d::prelude::SpatialQueryFilter;
use bevy::math::Dir2;
use bevy::prelude::*;

use crate::game::components::collision::Collision;
use crate::game::components::hostile::Hostile;
use crate::game::components::player::Player;
use crate::game::components::plasma::{PlasmaBeam, PLASMA_BEAM_PARTICLE_COUNT, PLASMA_Z};
use crate::helper::audio_settings::AudioSettings;
use crate::key_bindings::KeyBindings;
use crate::LevelStats;

use crate::game::view_api::CombatSoundEffects;
use crate::game::view_api::GameViewEntity;

use crate::game::systems::combat_helpers::{
    hash_to_unit,
    plasma_origin_from_player,
    ensure_plasma_particle_image,
};

pub(crate) fn shoot_plasma(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    time: Res<Time>,
    audio_settings: Res<AudioSettings>,
    combat_sfx: Option<Res<CombatSoundEffects>>,
    mut images: ResMut<Assets<Image>>,
    mut plasma_particle_image: Local<Option<Handle<Image>>>,
    spatial_query: avian2d::prelude::SpatialQuery,
    mut players: Query<(
        Entity, &Transform, &Sprite, &crate::game::components::health::Health, &mut crate::game::components::player::PlasmaAttack, &mut crate::game::components::animation::AnimationState, Option<&crate::game::components::animation::HitStateTimer>),
        With<Player>,
    >,
    collision_query: Query<Entity, With<Collision>>,
    hostile_query: Query<Entity, (With<Hostile>, With<crate::game::components::health::Health>)>,
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

        if crate::game::components::animation::can_set_state(&state, hit_timer, None, crate::game::components::animation::EntityState::Fight) {
            state.set(crate::game::components::animation::EntityState::Fight);
        }

        let direction = if sprite.flip_x { -1.0f32 } else { 1.0 };
        let origin = plasma_origin_from_player(transform, sprite);
        let dir2 = if sprite.flip_x { Dir2::NEG_X } else { Dir2::X };

        let filter = SpatialQueryFilter {
            excluded_entities: [player_entity].into_iter().collect(),
            ..default()
        };

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
                    crate::game::systems::combat_types::PlasmaBeamParticle {
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
                    crate::game::systems::combat_types::PlasmaBeamParticle {
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

