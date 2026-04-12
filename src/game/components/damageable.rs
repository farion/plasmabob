use bevy::prelude::Component;

/// Damageable component indicates that an entity can take damage and potentially be destroyed.
#[derive(Component, Debug, Clone, Copy)]
pub struct Damageable {

}

impl Damageable {
    pub fn new() -> Self {
        Damageable { }
    }
}

impl Default for Damageable {
    fn default() -> Self {
        Damageable::new()
    }
}

