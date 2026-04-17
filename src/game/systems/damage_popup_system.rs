use bevy::prelude::*;
use rand::Rng;

use crate::game::runtime_components::{DamagePopup, DamagePopupSettings};
use crate::helper::fonts::BoldText;

/// Spawn a floating damage/heal number in world space.
pub fn spawn_damage_popup(
    commands: &mut Commands,
    position: Vec3,
    amount: i32,
    is_heal: bool,
    is_controlled: bool,
    settings: &DamagePopupSettings,
) {
    let mut rng = rand::thread_rng();

    let angle_deg: f32 = rng.gen_range(-30.0..=30.0);
    let angle_rad = angle_deg.to_radians();
    let angular_velocity = rng.gen_range(-1.5..=1.5);
    let vx = rng.gen_range(-settings.horizontal_spread..=settings.horizontal_spread);
    let vy = settings.upward_speed;

    let mut font_size = settings.base_font_size;
    if is_controlled {
        font_size *= settings.controlled_scale;
    }

    let color = if is_heal {
        Color::srgba(0.2, 0.9, 0.2, 1.0)
    } else {
        Color::srgba(0.9, 0.1, 0.1, 1.0)
    };

    // Spawn the text entity. Attach `BoldText` so the FontsPlugin picks the
    // SpaceMono Bold variant which makes the numbers visually thicker.
    commands.spawn((
        Text2d::new(amount.to_string()),
        TextFont {
            font_size,
            ..default()
        },
        BoldText,
        TextColor(color),
        Transform {
            translation: position,
            rotation: Quat::from_rotation_z(angle_rad),
            ..default()
        },
        DamagePopup {
            velocity: Vec3::new(vx, vy, 0.0),
            angular_velocity,
            life: Timer::from_seconds(settings.lifetime_secs, TimerMode::Once),
            is_heal,
        },
    ));
}

pub fn damage_popup_animate_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DamagePopup, &mut Transform, &mut TextColor)>,
) {
    let dt = time.delta_secs();

    for (entity, mut popup, mut transform, mut text_color) in &mut query {
        popup.life.tick(time.delta());

        // Update translation based on velocity (simple linear movement).
        transform.translation += popup.velocity * dt;

        // Slightly slow down the upward movement for a pleasing arc.
        popup.velocity.y -= 200.0 * dt * dt; // small gravity-like easing

        // Update rotation by angular velocity.
        transform.rotate_local(Quat::from_rotation_z(popup.angular_velocity * dt));

        // Fade out over lifetime using fraction()
        let fraction = popup.life.fraction();
        let alpha = (1.0 - fraction).clamp(0.0, 1.0);
        text_color.0.set_alpha(alpha);

        if popup.life.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}






