use bevy::prelude::*;

use crate::game::runtime_components::AnimationConfig;

/// Ticks every [`AnimationConfig`] timer and, when the current frame changes,
/// updates the entity's [`Sprite`] to show the new frame handle.
///
/// Single-frame animations never trigger the sprite update (the timer still
/// ticks but `AnimationConfig::tick` only returns `true` for multi-frame
/// sequences).  The first frame is set on spawn and on every state transition
/// by `state_machine_update_system`.
pub fn animation_tick_system(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut Sprite)>,
) {
    for (mut anim, mut sprite) in &mut query {
        if anim.frames.is_empty() {
            // No frames → keep current sprite (red fallback or last image).
            continue;
        }
        if anim.tick(time.delta()) {
            // Frame index advanced — update the sprite image.
            if let Some(handle) = anim.current_frame_handle() {
                sprite.image = handle;
            }
        }
    }
}
