use std::collections::HashMap;

use avian2d::prelude::{AngularVelocity, Collider, LinearVelocity, RigidBody};
use bevy::prelude::*;

use crate::game::components::player::Player;
use crate::game::components::ragdoll::{PlayerDiedEvent, RagdollChest, RagdollLimb, RagdollWeapon};

use super::GameViewEntity;

const RAGDOLL_Z: f32 = 5.0;
/// Maximale Winkelabweichung eines Gelenks vom Chest-Winkel (±75°).
const MAX_JOINT_ANGLE_RAD: f32 = 75.0_f32 * std::f32::consts::PI / 180.0;

// ---------------------------------------------------------------------------
// Render-Skalierung der Rigging-PNGs (hier einstellen um Größe anzupassen)
// ---------------------------------------------------------------------------
const S: f32 = 0.23;

const CHEST_SIZE:  Vec2 = Vec2::new(250.0 * S, 250.0 * S);
/// Collision-Box etwas kleiner als das Sprite (ca. 60 %).
const CHEST_COLLIDER: Vec2 = Vec2::new(250.0 * S * 0.6, 250.0 * S * 0.6);
const HEAD_SIZE:   Vec2 = Vec2::new(200.0 * S, 200.0 * S);
const ARM_SIZE:    Vec2 = Vec2::new(250.0 * S, 250.0 * S);
const LEG_SIZE:    Vec2 = Vec2::new(200.0 * S, 350.0 * S);
const WEAPON_SIZE: Vec2 = Vec2::new(550.0 * S, 200.0 * S);
/// Weapon-Collision-Box: 60 % des Sprites.
const WEAPON_COLLIDER: Vec2 = Vec2::new(550.0 * S * 0.6, 200.0 * S * 0.6);

// ---------------------------------------------------------------------------
// Chest-Joint-Offsets vom Chest-Sprite-Zentrum (Chest-Local, y-up, ×S)
// Chest: 250×250 → Zentrum (125,125)
// Formel: ((px-125)*S,  -(py-125)*S)
// ---------------------------------------------------------------------------
const CHEST_HEAD_JOINT: Vec2 = Vec2::new((147.0-125.0)*S, -(47.0 -125.0)*S);
const CHEST_LARM_JOINT: Vec2 = Vec2::new((198.0-125.0)*S, -(82.0 -125.0)*S);
const CHEST_RARM_JOINT: Vec2 = Vec2::new((71.0 -125.0)*S, -(86.0 -125.0)*S);
const CHEST_RLEG_JOINT: Vec2 = Vec2::new((109.0-125.0)*S, -(232.0-125.0)*S);
const CHEST_LLEG_JOINT: Vec2 = Vec2::new((179.0-125.0)*S, -(220.0-125.0)*S);

// ---------------------------------------------------------------------------
// Limb-Pivot-Offsets vom Glied-Sprite-Zentrum (Limb-Local, y-up, ×S)
// Head 200×200 → (100,100) | Arms 250×250 → (125,125) | Legs 200×350 → (100,175)
// ---------------------------------------------------------------------------
const HEAD_PIVOT: Vec2 = Vec2::new((87.0 -100.0)*S, -(147.0-100.0)*S);
const LARM_PIVOT: Vec2 = Vec2::new((72.0 -125.0)*S, -(71.0 -125.0)*S);
const RARM_PIVOT: Vec2 = Vec2::new((140.0-125.0)*S, -(72.0 -125.0)*S);
const RLEG_PIVOT: Vec2 = Vec2::new((142.0-100.0)*S, -(66.0 -175.0)*S);
const LLEG_PIVOT: Vec2 = Vec2::new((36.0 -100.0)*S, -(49.0 -175.0)*S);

// ---------------------------------------------------------------------------
// System: spawn_ragdoll
// ---------------------------------------------------------------------------
pub(super) fn spawn_ragdoll(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<PlayerDiedEvent>,
    mut players: Query<(Entity, &mut Visibility, &Sprite), With<Player>>,
) {
    for event in events.read() {
        let Ok((player_entity, mut visibility, sprite)) = players.get_single_mut() else { continue; };
        *visibility = Visibility::Hidden;

        // Bobs Physics-Körper entfernen damit der Ragdoll-Chest nicht damit
        // kollidiert (beide wären sonst im selben Default-CollisionLayer).
        commands.entity(player_entity).remove::<(
            RigidBody,
            Collider,
            LinearVelocity,
            AngularVelocity,
        )>();

        let pos = event.player_position;
        let is_flipped = sprite.flip_x;
        let xf: f32 = if is_flipped { -1.0 } else { 1.0 };

        let kill_dir = event.killer_position
            .map(|kp| {
                let d = pos - kp;
                if d.length_squared() > f32::EPSILON { d.normalize() } else { Vec2::Y }
            })
            .unwrap_or(Vec2::Y);

        let bvx = kill_dir.x * 280.0;
        let bvy = kill_dir.y.abs() * 180.0 + 420.0;

        // Chest: avian2d übernimmt Gravity + Boden-Collision.
        // Collider = ganzes PNG als Quadrat (CHEST_SIZE).
        let chest_id = commands.spawn((
            Name::new("RagdollChest"),
            Sprite {
                image: asset_server.load("bob/rigging/Bob-Chest.png"),
                custom_size: Some(CHEST_SIZE),
                flip_x: is_flipped,
                ..default()
            },
            Transform::from_translation(pos.extend(RAGDOLL_Z + 0.2)),
            RigidBody::Dynamic,
            Collider::rectangle(CHEST_COLLIDER.x, CHEST_COLLIDER.y),
            LinearVelocity(Vec2::new(bvx, bvy)),
            AngularVelocity(1.8 * xf),
            RagdollChest,
            GameViewEntity,
        )).id();

        // Z-Layering: vordere Gliedmaßen über Chest (höheres Z), hintere darunter.
        // Blickrichtung rechts (not flipped): rechte Gliedmaßen = vorne.
        // Blickrichtung links (flipped):      linke  Gliedmaßen = vorne.
        let z_back   = RAGDOLL_Z + 0.1;  // hinter dem Chest
        let z_chest  = RAGDOLL_Z + 0.2;  // Chest-Referenz (oben gesetzt)
        let z_front  = RAGDOLL_Z + 0.35; // vor dem Chest
        let z_head   = RAGDOLL_Z + 0.45;
        let z_weapon = RAGDOLL_Z + 0.5;
        let _ = z_chest; // wird nur zur Dokumentation referenziert

        // facing right → right = front | facing left → left = front
        let (larm_z, rarm_z, lleg_z, rleg_z) = if is_flipped {
            (z_front, z_back, z_front, z_back)
        } else {
            (z_back, z_front, z_back, z_front)
        };

        // Hilfsfunktion: X-Achse spiegeln wenn Bob nach links schaut
        let fx = |v: Vec2| Vec2::new(v.x * xf, v.y);

        // Zufällige Startrotation pro Gliedmaße (deterministisch via chest-Entity-Index)
        // Bereich: ±(MAX_JOINT_ANGLE_RAD * 0.9) — also knapp unter dem Limit
        let seed = chest_id.index();
        let rand_rot = |s: u32| -> f32 {
            (hash_to_unit(seed.wrapping_add(s)) * 2.0 - 1.0) * MAX_JOINT_ANGLE_RAD * 0.9
        };

        // Gliedmaßen spawnen (Joint-Constraint an Chest)
        spawn_limb(&mut commands, &asset_server,
            "bob/rigging/Bob-Head.png",
            z_head, HEAD_SIZE, pos,
            fx(CHEST_HEAD_JOINT), fx(HEAD_PIVOT), -3.5 * xf, rand_rot(11), is_flipped, chest_id);

        spawn_limb(&mut commands, &asset_server,
            "bob/rigging/Bob-Left-Arm.png",
            larm_z, ARM_SIZE, pos,
            fx(CHEST_LARM_JOINT), fx(LARM_PIVOT), 5.0 * xf, rand_rot(23), is_flipped, chest_id);

        spawn_limb(&mut commands, &asset_server,
            "bob/rigging/Bob-Right-Arm.png",
            rarm_z, ARM_SIZE, pos,
            fx(CHEST_RARM_JOINT), fx(RARM_PIVOT), -4.5 * xf, rand_rot(37), is_flipped, chest_id);

        spawn_limb(&mut commands, &asset_server,
            "bob/rigging/Bob-Right-Leg.png",
            rleg_z, LEG_SIZE, pos,
            fx(CHEST_RLEG_JOINT), fx(RLEG_PIVOT), 7.0 * xf, rand_rot(53), is_flipped, chest_id);

        spawn_limb(&mut commands, &asset_server,
            "bob/rigging/Bob-Left-Leg.png",
            lleg_z, LEG_SIZE, pos,
            fx(CHEST_LLEG_JOINT), fx(LLEG_PIVOT), -6.0 * xf, rand_rot(71), is_flipped, chest_id);

        // Waffe – avian2d-Physics (Dynamic), kollidiert mit Boden + NPCs.
        // Bobs Collider ist zu diesem Zeitpunkt bereits entfernt worden,
        // daher trifft die Waffe den Spieler de facto nicht.
        commands.spawn((
            Name::new("RagdollWeapon"),
            Sprite {
                image: asset_server.load("bob/rigging/Bob-Weapon.png"),
                custom_size: Some(WEAPON_SIZE),
                flip_x: is_flipped,
                ..default()
            },
            Transform::from_translation(
                (pos + Vec2::new(55.0 * xf, 10.0)).extend(z_weapon),
            ),
            RigidBody::Dynamic,
            Collider::rectangle(WEAPON_COLLIDER.x, WEAPON_COLLIDER.y),
            LinearVelocity(Vec2::new(bvx * 1.5 + 130.0 * xf, bvy * 0.6 + 150.0)),
            AngularVelocity(10.0 * xf),
            RagdollWeapon,
            GameViewEntity,
        ));
    }
}

/// Spawnt eine Gliedmaße. Startposition: Chest-Joint − Limb-Pivot (bei Rotation 0).
#[allow(clippy::too_many_arguments)]
fn spawn_limb(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: &str,
    z: f32,
    size: Vec2,
    chest_world_pos: Vec2,
    chest_joint_local: Vec2,
    limb_pivot_local: Vec2,
    angular_velocity: f32,
    initial_rot_z: f32,
    flip_x: bool,
    chest_id: Entity,
) {
    let init_pos = chest_world_pos + chest_joint_local - limb_pivot_local;
    commands.spawn((
        Name::new(format!("RagdollLimb:{path}")),
        Sprite {
            image: asset_server.load(path),
            custom_size: Some(size),
            flip_x,
            ..default()
        },
        Transform::from_translation(init_pos.extend(z))
            .with_rotation(Quat::from_rotation_z(initial_rot_z)),
        RagdollLimb {
            angular_velocity,
            chest_entity: Some(chest_id),
            chest_joint_local,
            limb_pivot_local,
        },
        GameViewEntity,
    ));
}

// ---------------------------------------------------------------------------
// System: update_ragdoll_parts
// ---------------------------------------------------------------------------
pub(super) fn update_ragdoll_parts(
    mut commands: Commands,
    time: Res<Time>,
    chests: Query<(Entity, &Transform), With<RagdollChest>>,
    mut limbs: Query<(Entity, &mut RagdollLimb, &mut Transform), (Without<RagdollChest>, Without<RagdollWeapon>)>,
    weapons: Query<(Entity, &Transform), With<RagdollWeapon>>,
) {
    let dt = time.delta_secs();

    // Chest-Positionen/-Rotationen lesen (avian2d hat den Transform bereits aktualisiert)
    let mut chest_states: HashMap<Entity, (Vec2, Quat)> = HashMap::with_capacity(1);
    for (entity, tf) in &chests {
        chest_states.insert(entity, (tf.translation.xy(), tf.rotation));
        if tf.translation.y < -3000.0 {
            commands.entity(entity).despawn();
        }
    }

    // Waffe: avian2d steuert Position — wir despawnen nur bei Bedarf
    for (entity, tf) in &weapons {
        if tf.translation.y < -3000.0 {
            commands.entity(entity).despawn();
        }
    }

    // Gliedmaßen: freie Rotation + Position-Constraint an Chest-Joint
    for (entity, mut limb, mut tf) in &mut limbs {
        tf.rotation *= Quat::from_rotation_z(limb.angular_velocity * dt);

        if let Some(chest_id) = limb.chest_entity {
            if let Some(&(chest_pos, chest_rot)) = chest_states.get(&chest_id) {
                // --- Position-Constraint ---
                let chest_joint_world =
                    chest_pos + rotate_by_quat(limb.chest_joint_local, chest_rot);
                let pivot_offset = rotate_by_quat(limb.limb_pivot_local, tf.rotation);
                let new_pos = chest_joint_world - pivot_offset;
                tf.translation.x = new_pos.x;
                tf.translation.y = new_pos.y;

                // --- Winkel-Constraint (±MAX_JOINT_ANGLE_RAD relativ zum Chest) ---
                let chest_angle = quat_z_angle(chest_rot);
                let limb_angle  = quat_z_angle(tf.rotation);
                let relative    = angle_wrap(limb_angle - chest_angle);
                let clamped     = relative.clamp(-MAX_JOINT_ANGLE_RAD, MAX_JOINT_ANGLE_RAD);

                if relative.abs() > MAX_JOINT_ANGLE_RAD {
                    limb.angular_velocity *= -0.3;
                }

                tf.rotation = Quat::from_rotation_z(chest_angle + clamped);
            }
        }

        if tf.translation.y < -3000.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Rotiert einen 2D-Vektor mit dem Z-Anteil eines Quaternions.
fn rotate_by_quat(v: Vec2, quat: Quat) -> Vec2 {
    let r = quat * Vec3::new(v.x, v.y, 0.0);
    Vec2::new(r.x, r.y)
}

/// Extrahiert den Z-Rotationswinkel (in Radiant) aus einem Quaternion.
fn quat_z_angle(q: Quat) -> f32 {
    2.0 * q.z.atan2(q.w)
}

/// Normalisiert einen Winkel auf [-π, π].
fn angle_wrap(a: f32) -> f32 {
    let a = a % std::f32::consts::TAU;
    if a > std::f32::consts::PI {
        a - std::f32::consts::TAU
    } else if a < -std::f32::consts::PI {
        a + std::f32::consts::TAU
    } else {
        a
    }
}

/// Deterministischer Pseudo-Zufall [0, 1) aus einem u32-Seed (Wang Hash).
fn hash_to_unit(seed: u32) -> f32 {
    let mut v = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    v = (v >> ((v >> 28) + 4)) ^ v;
    v = v.wrapping_mul(277_803_737);
    (((v >> 22) ^ v) as f32) / (u32::MAX as f32)
}
