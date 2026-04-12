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

// Re-exports for convenience
pub use collider::{Collider, ColliderShape};
pub use rigid_body::RigidBody;
pub use health::Health;
pub use auto_movement::AutoMovement;
pub use controlled_movement::ControlledMovement;
pub use blocking::Blocking;
pub use gravity::Gravity;
pub use state_machine::StateMachine;
pub use crate::game::runtime_components::game_entity::GameEntity;
pub use damageable::Damageable;
pub use team::Team;
