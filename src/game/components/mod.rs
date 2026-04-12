pub mod collider;
pub mod rigid_body;
pub mod health;
pub mod auto_movement;
pub mod controlled_movement;
pub mod controlled_range_attack;
pub mod auto_range_attack;
pub mod auto_melee_attack;
pub mod controlled_melee_attack;
pub mod player_input;
pub mod blocking;
pub mod gravity;
pub mod state_machine;
pub mod game_entity;
pub mod damageable;

// Re-exports for convenience
pub use collider::{Collider, ColliderShape};
pub use rigid_body::RigidBody;
pub use health::Health;
pub use auto_movement::AutoMovement;
pub use controlled_movement::ControlledMovement;
pub use controlled_range_attack::ControlledRangeAttack;
pub use auto_range_attack::AutoRangeAttack;
pub use auto_melee_attack::AutoMeleeAttack;
pub use controlled_melee_attack::ControlledMeleeAttack;
pub use player_input::PlayerInput;
pub use blocking::Blocking;
pub use gravity::Gravity;
pub use state_machine::{StateMachine, EntityState};
pub use game_entity::GameEntity;
// Tag components were moved to `crate::game::tags`.
