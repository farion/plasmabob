use bevy::prelude::*;

/// Team marker with a name. Use a string so teams can be defined/serialized by name.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Team {
	pub name: String,
}

impl Team {
	/// Convenience constructor
	pub fn new<S: Into<String>>(name: S) -> Self {
		Self { name: name.into() }
	}
}

impl Default for Team {
	fn default() -> Self {
		// neutral default team name; change as appropriate
		Team { name: "Neutral".to_string() }
	}
}