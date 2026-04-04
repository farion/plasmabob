use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct BackgroundParallax;

#[derive(Component)]
pub(crate) struct ParallaxAnchor {
    pub(crate) base_x: f32,
    pub(crate) speed: f32,
}

