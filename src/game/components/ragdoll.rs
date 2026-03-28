use bevy::prelude::*;

/// Marker: kennzeichnet die Chest-Entity des Ragdolls.
/// Physik (Velocity, Gravity, Collision) wird von avian2d gesteuert.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RagdollChest;

/// Marker: kennzeichnet die fliegende Waffe des Ragdolls.
/// Physik wird von avian2d gesteuert (Dynamic RigidBody).
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RagdollWeapon;

/// Eine einzelne Gliedmaße.
/// Wenn `chest_entity` Some ist, wird die Position jedes Frame hart an den
/// Joint-Punkt des Chests geknüpft (Position Constraint). Nur die Rotation
/// ist frei.
#[derive(Component, Debug, Clone)]
pub(crate) struct RagdollLimb {
    pub(crate) angular_velocity: f32,
    /// Entity des Chests, an dem dieses Glied hängt.
    pub(crate) chest_entity: Option<Entity>,
    /// Offset des Joint-Ankerpunkts vom Chest-Zentrum (Chest-Local-Space, y-up, skaliert).
    pub(crate) chest_joint_local: Vec2,
    /// Offset des Pivot-Punkts von diesem Glied-Zentrum (Limb-Local-Space, y-up, skaliert).
    pub(crate) limb_pivot_local: Vec2,
}

/// Marker: Bobs Sprite ist versteckt weil ein Ragdoll aktiv ist.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RagdollActive;

/// Wird einmalig gefeuert wenn Bobs HP auf 0 fällt.
#[derive(Event, Debug, Clone, Copy)]
pub(crate) struct PlayerDiedEvent {
    pub(crate) player_position: Vec2,
    pub(crate) killer_position: Option<Vec2>,
}
