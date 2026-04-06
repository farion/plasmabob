use crate::game::components::player::Player;
use crate::helper::fonts::BoldText;
use bevy::prelude::*;

/// Component briefly attached to an entity to signal that it recently had a
/// health change. Systems watch for newly added instances and spawn the
/// floating text.
#[derive(Component)]
pub(crate) struct RecentHealthChange(pub(crate) i32);

#[derive(Component)]
pub(crate) struct FloatingHealthText {
    timer: Timer,
}

impl FloatingHealthText {
    fn new(lifetime: f32) -> Self {
        Self {
            timer: Timer::from_seconds(lifetime, TimerMode::Once),
        }
    }
}

/// Spawns a floating text when a `RecentHealthChange` component is added to an
/// entity.
pub(crate) fn spawn_on_health_change(
    mut commands: Commands,
    changed: Query<
        (
            Entity,
            &RecentHealthChange,
            &Transform,
            Option<&Sprite>,
            Option<&Player>,
        ),
        Added<RecentHealthChange>,
    >,
) {
    for (entity, recent, transform, sprite_opt, player_opt) in &changed {
        let base_y = transform.translation.y;
        let above = sprite_opt
            .and_then(|s| s.custom_size)
            .map(|size| base_y + size.y * 0.5 + 6.0)
            .unwrap_or(base_y + 24.0);

        let text_value = if recent.0 >= 0 {
            format!("+{}", recent.0)
        } else {
            format!("{}", recent.0)
        };

        let color = if recent.0 >= 0 {
            Color::srgba(0.35, 0.95, 0.35, 1.0)
        } else {
            Color::srgba(0.95, 0.35, 0.35, 1.0)
        };

        // Spawn a 2D text entity that floats up and fades out. Use the
        // project's Text2d helper (used by debug labels) and attach a
        // TextColor so we can animate its alpha.
        // Make all floating health text bold. If the change occured on the
        // player entity, use a larger font size so it stands out.
        let font_size = if player_opt.is_some() { 28.0 } else { 20.0 };

        commands.spawn((
            Text2d::new(text_value),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(color),
            BoldText,
            Transform::from_xyz(transform.translation.x, above, 500.0),
            FloatingHealthText::new(0.9),
            crate::game::systems::systems_api::GameViewEntity,
        ));

        // Remove the marker component so it only triggers once.
        commands.entity(entity).remove::<RecentHealthChange>();
    }
}

/// Moves floating text up and fades it out, despawning when its timer finishes.
pub(crate) fn animate_floating_texts(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut FloatingHealthText,
        &mut Transform,
        &mut Text2d,
        &mut TextColor,
    )>,
) {
    for (entity, mut ft, mut transform, mut _text2d, mut text_color) in &mut query {
        ft.timer.tick(time.delta());
        // Move up smoothly
        transform.translation.y += 40.0 * time.delta_secs();

        let remaining = 1.0 - ft.timer.fraction();
        // Fade alpha by replacing the TextColor while preserving RGB.
        let s = text_color.0.to_srgba();
        text_color.0 = Color::srgba(s.red, s.green, s.blue, remaining.clamp(0.0f32, 1.0f32));

        if ft.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
