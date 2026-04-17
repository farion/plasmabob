// Exports for per-component config types used during level/entity-type parsing.
pub mod health_config;
pub use health_config::HealthConfig;

pub mod controlled_movement_config;
pub use controlled_movement_config::ControlledMovementConfig;

pub mod auto_movement_config;
pub use auto_movement_config::AutoMovementConfig;

pub mod moving_platform_config;
pub use moving_platform_config::MovingPlatformConfig;

pub mod rigid_body_config;
pub use rigid_body_config::RigidBodyConfig;

pub mod gravity_config;
pub use gravity_config::GravityConfig;

pub mod blocking_config;
pub use blocking_config::BlockingConfig;

pub mod controlled_range_attack_config;
pub use controlled_range_attack_config::ControlledRangeAttackConfig;

pub mod auto_range_attack_config;
pub use auto_range_attack_config::AutoRangeAttackConfig;

pub mod auto_melee_attack_config;
pub use auto_melee_attack_config::AutoMeleeAttackConfig;

pub mod controlled_melee_attack_config;
pub use controlled_melee_attack_config::ControlledMeleeAttackConfig;

pub mod damageable_config;
pub use damageable_config::DamageableConfig;

pub mod team_config;
pub use team_config::TeamConfig;

pub mod orientation_config;
pub use orientation_config::OrientationConfig;

pub mod collider_config;
pub use collider_config::ColliderConfig;

