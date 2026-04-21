use bevy::prelude::Component;

/// Runtime component describing an effect applied when a collectible is picked up.
#[derive(Component, Debug, Clone)]
pub struct CollectibleEffect {
    /// Optional healing amount to apply to the picker.
    pub heal: i32,
}

impl CollectibleEffect {
    pub fn new(heal: i32) -> Self {
        CollectibleEffect { heal }
    }
}

impl Default for CollectibleEffect {
    fn default() -> Self {
        CollectibleEffect::new(0)
    }
}

crate::impl_override_from_config!(CollectibleEffect, crate::game::level::configs::collectible_effect_config::CollectibleEffectConfig,
    pick_u32 => [heal],
);
