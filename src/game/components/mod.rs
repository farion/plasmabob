pub mod auto_melee_attack;
pub mod auto_movement;
pub mod auto_range_attack;
pub mod blocking;
pub mod collectible_effect;
pub mod collider;
pub mod controlled_melee_attack;
pub mod controlled_movement;
pub mod controlled_range_attack;
pub mod damageable;
pub mod gravity;
pub mod health;
pub mod moving_platform;
pub mod orientation;
pub mod plasma;
pub mod rigid_body;
pub mod state_machine;
pub mod team;

// Re-exports for convenience
pub use auto_movement::{AutoMovement, AutoMovementDefaultStrategy, AutoMovementState, AutoMovementAggroStrategy};
pub use blocking::Blocking;
pub use collectible_effect::CollectibleEffect;
pub use collider::{Collider, ColliderShape};
pub use controlled_movement::ControlledMovement;
pub use gravity::Gravity;
pub use health::Health;
pub use orientation::Orientation;
pub use rigid_body::RigidBody;
pub use state_machine::{EntityState, StateMachine};
// GameEntity is defined in runtime_components; re-export it here so callers
// that expect `crate::game::components::GameEntity` continue to work.
pub use crate::game::runtime_components::game_entity::GameEntity;
pub use damageable::Damageable;
pub use moving_platform::MovingPlatform;
pub use team::Team;
// DamagePopup is a runtime component and lives under `runtime_components`.
