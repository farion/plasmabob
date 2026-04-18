use bevy::prelude::*;
// InheritedVisibility is available via the prelude import above (bevy::prelude::*).
// No debug logging in hot input path to avoid spam in release runs.

use crate::game::components::controlled_range_attack::ControlledRangeAttack;
use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::{Collider, ColliderShape, RigidBody, StateMachine};
use crate::game::gfx::plasma_shoot::{
    ensure_plasma_particle_image, spawn_plasma_beam_particles, PlasmaParticleImage,
};
use crate::game::runtime_components::{Facing, GameEntity, Projectile};
use crate::game::tags::PlayerTag;
use crate::helper::active_character::ActiveCharacter;
use crate::helper::audio_settings::AudioSettings;
use crate::helper::key_bindings::{KeyAction, KeyBindings};
use crate::helper::sounds::spawn_combat_sfx;

const PLASMA_SHOT_SFX: &str = "audio/plasma-shot.ogg";
const PROJECTILE_HALF_EXTENT: f32 = 4.0;

pub fn player_shoot_system(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    asset_server: Res<AssetServer>,
    active_character: Res<ActiveCharacter>,
    audio_settings: Res<AudioSettings>,
    mut images: ResMut<Assets<Image>>,
    mut plasma_particle_image: Local<Option<Handle<Image>>>,
    particle_image_res: Option<Res<PlasmaParticleImage>>,
    // Per-system local request flag: stores whether the player tapped shoot
    // and awaits cooldown completion. Also track previous pressed state to
    // detect presses that sometimes aren't reported as just_pressed due to
    // keyboard ghosting when multiple keys are held.
    mut prev_shoot_pressed: Local<bool>,
    mut fire_requested: Local<bool>,
    mut stats: ResMut<crate::LevelStats>,
    mut players: Query<
        (
            Entity,
            &Transform,
            &Facing,
            &mut ControlledRangeAttack,
            Option<&StateMachine>,
        ),
        With<PlayerTag>,
    >,
) {
    // Track a fire request set when the user *just* pressed the shoot key.
    // We implement a small request queue so a tap will fire as soon as the
    // cooldown elapses instead of requiring the tap to land on the exact
    // frame the timer finishes.
    // Determine shoot press: prefer just_pressed but also treat the rising
    // edge of `pressed` (pressed && !prev) as a press. This helps with
    // some keyboard ghosting where just_pressed may be missed.
    let shoot_pressed_now = keyboard.pressed(key_bindings.get(KeyAction::Shoot));
    let shoot_just_pressed = keyboard.just_pressed(key_bindings.get(KeyAction::Shoot))
        || (shoot_pressed_now && !*prev_shoot_pressed);
    // Set the request flag when the key was just pressed.
    if shoot_just_pressed {
        *fire_requested = true;
    }

    // (global input detection already logged above when just pressed)

    for (player_entity, player_transform, facing, mut attack, sm) in &mut players {
        if sm.is_some_and(|sm| sm.is_non_interactive()) {
            *fire_requested = false;
            continue;
        }

        // Advance cooldown every frame so it progresses independently of input.
        let dt = time.delta();
        attack.cooldown.tick(dt);

        // If the player requested a shot and the cooldown is ready, fire.
        let _cooldown_fraction = attack.cooldown.fraction();
        let cooldown_ready = attack.cooldown.just_finished() || attack.cooldown.is_finished();

        if !*fire_requested || !cooldown_ready {
            continue;
        }

        let facing_dir = if facing.direction.length_squared() > f32::EPSILON {
            facing.direction.normalize()
        } else {
            Vec2::X
        };

        let origin = player_transform.translation.truncate();
        let entity_z = player_transform.translation.z;
        let projectile_entity = commands
            .spawn((
                Name::new("PlayerProjectile"),
                Transform::from_xyz(origin.x, origin.y, entity_z),
                Collider {
                    offset: Vec2::ZERO,
                    shape: ColliderShape::Rectangle {
                        half_extents: Vec2::splat(PROJECTILE_HALF_EXTENT),
                    },
                },
                RigidBody {
                    velocity: facing_dir * attack.speed,
                    ..default()
                },
                Projectile::new(
                    player_entity,
                    attack.damage,
                    attack.range,
                    attack.shoot_effect.clone(),
                    attack.impact_effect.clone(),
                ),
                GameEntity,
            ))
            .id();

        if attack
            .shoot_effect
            .as_deref()
            .unwrap_or("plasma_shoot")
            .eq_ignore_ascii_case("plasma_shoot")
        {
            // plasma beam spawned
            let particle_image = if let Some(resource) = particle_image_res.as_ref() {
                resource.0.clone()
            } else {
                ensure_plasma_particle_image(&mut plasma_particle_image, &mut images)
            };
            let mut beam_cmd = commands.spawn((
                Name::new("PlasmaBeam"),
                Transform::from_xyz(origin.x, origin.y, entity_z),
                GlobalTransform::default(),
                // Ensure parent has Visibility/ComputedVisibility so child sprites using
                // inherited visibility don't trigger Bevy warning B0004.
                Visibility::default(),
                InheritedVisibility::default(),
                PlasmaBeam::new(origin, facing_dir.x.signum(), Some(projectile_entity)),
                GameEntity,
            ));
            spawn_plasma_beam_particles(&mut beam_cmd, &particle_image);
        }

        spawn_combat_sfx(
            &mut commands,
            &asset_server,
            &audio_settings,
            *active_character,
            PLASMA_SHOT_SFX,
        );

        attack.cooldown.reset();
        attack.just_fired = true;
        stats.shots = stats.shots.saturating_add(1);
        *fire_requested = false;
    }

    // Update previous pressed state for next frame edge detection.
    *prev_shoot_pressed = keyboard.pressed(key_bindings.get(KeyAction::Shoot));
}
