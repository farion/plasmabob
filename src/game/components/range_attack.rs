use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub(crate) struct RangeAttack {
    pub(crate) damage: i32,
    pub(crate) speed: f32,
    /// Fire cadence in milliseconds from JSON data.
    pub(crate) frequency: f32,
    pub(crate) max_range: f32,
    pub(crate) aggro_range: f32,
    pub(crate) cooldown: Timer,
    /// Optional sprite frames for an animated projectile visual.
    pub(crate) animation: Vec<String>,
    /// Animation frame duration in ms (used when animation is set).
    pub(crate) animation_frame_ms: Option<u64>,
    /// Named particle effect: "fire" | "poison" | "spit".
    pub(crate) particle_effect: Option<String>,
}

impl RangeAttack {
    pub(crate) fn new(
        damage: i32,
        speed: f32,
        frequency: f32,
        max_range: f32,
        aggro_range: f32,
        animation: Vec<String>,
        animation_frame_ms: Option<u64>,
        particle_effect: Option<String>,
    ) -> Self {
        let cadence_secs = frequency.max(1.0) / 1000.0;
        let mut cooldown = Timer::from_seconds(cadence_secs, TimerMode::Once);
        // Start already finished so the entity fires immediately on first aggro.
        cooldown.tick(std::time::Duration::from_secs_f32(cadence_secs));

        Self {
            damage: damage.max(0),
            speed: speed.max(0.0),
            frequency: frequency.max(1.0),
            max_range: max_range.max(1.0),
            aggro_range: aggro_range.max(1.0),
            cooldown,
            animation,
            animation_frame_ms,
            particle_effect,
        }
    }
}
