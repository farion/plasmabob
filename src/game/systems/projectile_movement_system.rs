use bevy::prelude::*;

use crate::game::components::plasma::PlasmaBeam;
use crate::game::components::RigidBody;
use crate::game::gfx::plasma_impact::spawn_plasma_impact_explosion;
use crate::game::gfx::plasma_shoot::ensure_plasma_particle_image;
use crate::game::runtime_components::Projectile;
use crate::helper::audio_settings::AudioSettings;
use crate::helper::sounds::spawn_combat_sfx;

const PLASMA_HIT_SFX: &str = "audio/plasma-hit.ogg";
const BEAM_AFTERGLOW_SECS: f32 = 0.28;

pub fn projectile_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
    mut images: ResMut<Assets<Image>>,
    mut plasma_particle_image: Local<Option<Handle<Image>>>,
    mut projectiles: Query<(Entity, &mut Transform, &RigidBody, &mut Projectile)>,
    mut beams: Query<(Entity, &mut PlasmaBeam)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let mut expired_projectiles: Vec<(Entity, Vec2, bool)> = Vec::new();

    for (projectile_entity, mut transform, rigid_body, mut projectile) in &mut projectiles {
        let motion = rigid_body.velocity * dt;
        transform.translation.x += motion.x;
        transform.translation.y += motion.y;
        projectile.remaining_range = (projectile.remaining_range - motion.length()).max(0.0);

        if projectile.remaining_range <= f32::EPSILON {
            let impact_position = transform.translation.truncate();
            let should_spawn_impact = projectile
                .impact_effect
                .as_deref()
                .unwrap_or("plasma_impact")
                .eq_ignore_ascii_case("plasma_impact");
            expired_projectiles.push((projectile_entity, impact_position, should_spawn_impact));
        }
    }

    if expired_projectiles.is_empty() {
        return;
    }

    let particle_image = ensure_plasma_particle_image(&mut plasma_particle_image, &mut images);
    for (projectile_entity, impact_position, should_spawn_impact) in expired_projectiles {
        if should_spawn_impact {
            spawn_plasma_impact_explosion(&mut commands, &particle_image, impact_position);
        }
        spawn_combat_sfx(
            &mut commands,
            &asset_server,
            &audio_settings,
            PLASMA_HIT_SFX,
        );
        set_beams_to_afterglow(projectile_entity, &mut beams);
        commands.entity(projectile_entity).despawn();
    }
}

fn set_beams_to_afterglow(projectile_entity: Entity, beams: &mut Query<(Entity, &mut PlasmaBeam)>) {
    for (_beam_entity, mut beam) in beams {
        if beam.target_projectile == Some(projectile_entity) {
            beam.target_projectile = None;
            beam.lifetime = Some(Timer::from_seconds(BEAM_AFTERGLOW_SECS, TimerMode::Once));
        }
    }
}

