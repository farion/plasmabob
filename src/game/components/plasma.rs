use bevy::prelude::*;

/// Speed at which the plasma beam expands, in world-space pixels per second.
pub(crate) const PLASMA_EXPAND_SPEED: f32 = 2400.0;
/// Z-index for plasma beams (above floor/NPCs, below player).
pub(crate) const PLASMA_Z: f32 = 15.0;
/// Vertical start point of the beam measured from the entity's bottom edge (0.0..=1.0).
pub(crate) const PLASMA_ORIGIN_HEIGHT_RATIO_FROM_BOTTOM: f32 = 0.7;
/// Number of particles used to visualize a single plasma beam.
pub(crate) const PLASMA_BEAM_PARTICLE_COUNT: usize = 56;
/// Half-height of the particle beam cloud around the center line.
pub(crate) const PLASMA_BEAM_VISUAL_HALF_HEIGHT: f32 = 10.0;
/// Sine-wave speed for beam particle jitter.
pub(crate) const PLASMA_BEAM_PARTICLE_WIGGLE_SPEED: f32 = 18.0;
/// Maximum lateral offset added by beam particle jitter.
pub(crate) const PLASMA_BEAM_PARTICLE_WIGGLE_AMPLITUDE: f32 = 2.8;
/// Number of particles spawned for a hit explosion.
pub(crate) const PLASMA_IMPACT_PARTICLE_COUNT: usize = 40;
/// Lifetime of impact particles in seconds.
pub(crate) const PLASMA_IMPACT_LIFETIME_SECS: f32 = 0.32;
/// Minimum speed of impact particles.
pub(crate) const PLASMA_IMPACT_MIN_SPEED: f32 = 140.0;
/// Maximum speed of impact particles.
pub(crate) const PLASMA_IMPACT_MAX_SPEED: f32 = 420.0;
/// Seconds the beam lingers visible after it stops expanding.
pub(crate) const PLASMA_LINGER_SECS: f32 = 0.2;

/// A growing plasma beam fired by the player.
#[derive(Component, Debug, Clone)]
pub(crate) struct PlasmaBeam {
    /// +1.0 = facing right, -1.0 = facing left.
    pub(crate) direction: f32,
    /// The player entity this beam originates from; used to track its position while expanding.
    pub(crate) player_entity: Entity,
    /// Current visual length in pixels.
    pub(crate) current_length: f32,
    /// Maximum length: distance to first Collision hit, capped at attack_range.
    pub(crate) max_length: f32,
    /// Entity to damage when the beam reaches its maximum length (hostile NPCs only).
    pub(crate) target_entity: Option<Entity>,
    /// Damage to apply on hit.
    pub(crate) damage: i32,
    /// Ensures damage is applied only once.
    pub(crate) damage_applied: bool,
    /// Counts down after the beam stops before it is despawned.
    pub(crate) linger_timer: Timer,
    /// True once the beam has finished expanding.
    pub(crate) stopped: bool,
    /// Ensures the impact explosion is spawned once when the beam reaches a target.
    pub(crate) impact_spawned: bool,
}

impl PlasmaBeam {
    pub(crate) fn new(
        player_entity: Entity,
        direction: f32,
        max_length: f32,
        target_entity: Option<Entity>,
        damage: i32,
    ) -> Self {
        Self {
            direction,
            player_entity,
            current_length: 1.0, // avoid zero-size sprite on first frame
            max_length: max_length.max(1.0),
            target_entity,
            damage,
            damage_applied: false,
            linger_timer: Timer::from_seconds(PLASMA_LINGER_SECS, TimerMode::Once),
            stopped: false,
            impact_spawned: false,
        }
    }
}

