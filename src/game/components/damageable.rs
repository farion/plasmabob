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

impl Damageable {
    /// Apply overrides from JSON for Damageable. This component currently has
    /// no configurable fields; method exists for API symmetry with other
    /// components.
    pub fn override_from_json(self, _comp_obj: Option<&serde_json::Value>) -> Self {
        self
    }
}

