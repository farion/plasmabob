use bevy::prelude::Component;

/// Marker component: this entity blocks movement/collisions (e.g. walls, platforms).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Blocking;

impl Blocking {
	/// Apply overrides from JSON. Blocking is a marker component and has no
	/// configurable fields, but we provide the method for API symmetry so
	/// the spawner can call `Blocking::default().override_from_json(...)`.
	pub fn override_from_json(self, _comp_obj: Option<&serde_json::Value>) -> Self {
		self
	}
}
