use bevy::prelude::*;

/// Runtime state for simple enemy random patrol.
#[derive(Component, Debug, Clone, Copy)]
pub struct PatrolState {
    pub timer: f32,
    pub direction: f32,
    pub seed: u32,
}

impl PatrolState {
    pub fn from_entity(entity: Entity) -> Self {
        // `Entity::index()` returns an internal type on some Bevy versions.
        // Use `to_bits()` and take the low 32 bits to derive a stable u32 seed.
        let seed = (entity.to_bits() as u32)
            .wrapping_mul(747_796_405)
            .wrapping_add(2_891_336_453);
        Self {
            timer: 0.0,
            direction: 1.0,
            seed,
        }
    }

    pub fn next_rand(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.seed as f32) / (u32::MAX as f32)
    }
}

