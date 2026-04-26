use bevy::prelude::Component;

/// Blocking component: marks an entity as blocking movement/collisions and
/// optionally blocking line-of-sight.
#[derive(Component, Debug, Clone, Copy)]
pub struct Blocking {
	/// Whether this blocker obstructs enemy line-of-sight queries.
	pub blocks_line_of_sight: bool,
}

impl Default for Blocking {
	fn default() -> Self {
		Blocking {
			blocks_line_of_sight: false,
		}
	}
}

// Allow JSON-level overrides via the usual override_from_config macro.
crate::impl_override_from_config!(Blocking, crate::game::level::configs::BlockingConfig,
	pick_bool => [blocks_line_of_sight],
);
