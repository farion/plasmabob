use bevy::prelude::*;

const PLASMA_SHOOT_COOLDOWN_SECS: f32 = 0.5;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct Player;

/// Enables the player entity to shoot plasma beams.
#[derive(Component, Debug, Clone)]
pub(crate) struct PlasmaAttack {
    /// Maximum beam range in world-space pixels.
    pub(crate) range: f32,
    /// Damage dealt to a hostile NPC on hit.
    pub(crate) damage: i32,
    /// Prevents spamming: the player must wait for this timer before firing again.
    pub(crate) cooldown: Timer,
}

impl PlasmaAttack {
    pub(crate) fn new(range: f32, damage: i32) -> Self {
        let mut cooldown = Timer::from_seconds(PLASMA_SHOOT_COOLDOWN_SECS, TimerMode::Once);
        // Start already finished so the player can fire immediately.
        cooldown.tick(std::time::Duration::from_secs_f32(PLASMA_SHOOT_COOLDOWN_SECS));
        Self { range, damage, cooldown }
    }
}

pub(crate) fn insert(entity: &mut EntityCommands) {
    entity.insert(Player);
}

