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

impl Team {
	/// Apply overrides from `components.team` JSON object.
	/// Only reads the `name` field from the provided object. Does not
	/// perform any fallback to `category_tag` — callers must handle that.
	pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
		if let Some(serde_json::Value::Object(map)) = comp_obj {
			if let Some(name_v) = map.get("name").and_then(|n| n.as_str()) {
				self.name = name_v.to_string();
			}
		}
		self
	}
}
