pub mod collider;
pub mod rigid_body;
pub mod health;
pub mod auto_movement;
pub mod controlled_movement;
pub mod controlled_range_attack;
pub mod auto_range_attack;
pub mod auto_melee_attack;
pub mod controlled_melee_attack;
pub mod blocking;
pub mod gravity;
pub mod state_machine;
pub mod damageable;
pub mod team;
pub mod moving_platform;
pub mod plasma;
pub mod orientation;

// Re-exports for convenience
pub use collider::{Collider, ColliderShape};
pub use rigid_body::RigidBody;
pub use health::Health;
pub use auto_movement::AutoMovement;
pub use controlled_movement::ControlledMovement;
pub use blocking::Blocking;
pub use gravity::Gravity;
pub use state_machine::{StateMachine, EntityState};
pub use orientation::{Orientation, FacingDirection};
// GameEntity is defined in runtime_components; re-export it here so callers
// that expect `crate::game::components::GameEntity` continue to work.
pub use crate::game::runtime_components::game_entity::GameEntity;
pub use crate::game::runtime_components::spawned_level_entity::SpawnedLevelEntity;
pub use damageable::Damageable;
pub use team::Team;
pub use moving_platform::MovingPlatform;
pub use plasma::PlasmaBeam;
