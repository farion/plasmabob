// Tag components grouped under `game::tags` for marker components used across the game.
pub mod doodad_tag;
pub mod enemy_tag;
pub mod environment_tag;
pub mod player_tag;
pub mod collectible_tag;

// Re-exports for convenient access: `crate::game::tags::PlayerTag`, etc.
pub use doodad_tag::DoodadTag;
pub use enemy_tag::EnemyTag;
pub use environment_tag::EnvironmentTag;
pub use player_tag::PlayerTag;
pub use collectible_tag::CollectibleTag;
