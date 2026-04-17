use bevy::prelude::{Timer, TimerMode, Vec2};
use std::time::Duration;

/// Generic pick helper: prefer level_cfg then entity_cfg for a selected Option
///
/// Prioritisation now is: level_cfg (entity-type / level default) ->
/// entity_cfg (per-entity override) -> None. This implements the requested
/// semantics where level-wide defaults override per-entity values unless the
/// type's config explicitly omits the value.
pub fn pick<T, C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<T>
where
    F: Fn(&C) -> Option<T>,
{
    level_cfg.and_then(|c| f(c)).or(entity_cfg.and_then(|c| f(c)))
}

/// Helper macro to generate an `override_from_config` impl where one u32 field
/// should default to another u32 field when not provided. This is useful for
/// patterns like `current` defaulting to `max` in `Health`.
#[macro_export]
macro_rules! impl_override_with_u32_default {
    ($ty:ident, $cfg:path, $field:ident => $default:ident, $( $pick_fn:ident => [$($fields:ident),*] ),* $(,)?) => {
        impl $ty {
            pub fn override_from_config(mut self, entity_cfg: Option<&$cfg>, level_cfg: Option<&$cfg>) -> Self {
                let entity_cfg = entity_cfg;
                let level_cfg = level_cfg;
                $( $crate::__impl_override_dispatch!($pick_fn, self, $ty, $cfg, entity_cfg, level_cfg, [$($fields),*]); )*
                // If the primary field was not provided at all, use the fallback field value if present.
                if $crate::helper::override_helpers::pick_u32(entity_cfg, level_cfg, |c| c.$field).is_none() {
                    self.$field = $crate::helper::override_helpers::pick_u32(entity_cfg, level_cfg, |c| c.$default).map(|v| v as i32).unwrap_or(self.$field);
                }
                self
            }
        }
    };
}

pub fn pick_i32<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<i32>
where
    F: Fn(&C) -> Option<i32>,
{
    pick(entity_cfg, level_cfg, f)
}

pub fn pick_f32<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<f32>
where
    F: Fn(&C) -> Option<f32>,
{
    pick(entity_cfg, level_cfg, f)
}

pub fn pick_bool<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<bool>
where
    F: Fn(&C) -> Option<bool>,
{
    pick(entity_cfg, level_cfg, f)
}

pub fn pick_u64<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<u64>
where
    F: Fn(&C) -> Option<u64>,
{
    pick(entity_cfg, level_cfg, f)
}

pub fn pick_string<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<String>
where
    F: Fn(&C) -> Option<&String>,
{
    level_cfg.and_then(|c| f(c).map(|s| s.clone())).or(entity_cfg.and_then(|c| f(c).map(|s| s.clone())))
}

/// Pick a 2-element float array ([f32;2]) and return it when present.
pub fn pick_vec2<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<[f32;2]>
where
    F: Fn(&C) -> Option<[f32;2]>,
{
    pick(entity_cfg, level_cfg, f)
}

/// Pick a sequence of 2-element float arrays (waypoints) and convert to Vec<Vec2>.
pub fn pick_waypoints<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<Vec<Vec2>>
where
    F: Fn(&C) -> Option<&Vec<[f32;2]>>,
{
    if let Some(arrs) = level_cfg.and_then(|c| f(c)).or(entity_cfg.and_then(|c| f(c))) {
        Some(arrs.iter().map(|a| Vec2::new(a[0], a[1])).collect())
    } else {
        None
    }
}

pub fn pick_u32<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<u32>
where
    F: Fn(&C) -> Option<u32>,
{
    pick(entity_cfg, level_cfg, f)
}

/// Pick an unsigned 8-bit integer from configs.
pub fn pick_u8<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<u8>
where
    F: Fn(&C) -> Option<u8>,
{
    pick(entity_cfg, level_cfg, f)
}

/// Pick a facing direction from a string field (e.g. "left"/"right") and map to the
/// typed `FacingDirection` enum used by the components. The selector should return
/// Option<String> from the config type.
pub fn pick_facing<C, F>(entity_cfg: Option<&C>, level_cfg: Option<&C>, f: F) -> Option<crate::game::components::orientation::FacingDirection>
where
    F: Fn(&C) -> Option<&String>,
{
    if let Some(s) = level_cfg.and_then(|c| f(c).map(|s| s.clone())).or(entity_cfg.and_then(|c| f(c).map(|s| s.clone()))) {
        match s.to_ascii_lowercase().as_str() {
            "left" => Some(crate::game::components::orientation::FacingDirection::Left),
            _ => Some(crate::game::components::orientation::FacingDirection::Right),
        }
    } else {
        None
    }
}


/// Pick cooldown timer from a milliseconds selector (preferred: level_cfg -> entity_cfg).
pub fn pick_timer<C, FM>(entity_cfg: Option<&C>, level_cfg: Option<&C>, ms_sel: FM) -> Option<Timer>
where
    FM: Fn(&C) -> Option<u64>,
{
    if let Some(ms) = pick(entity_cfg, level_cfg, ms_sel) {
        let dur = Duration::from_millis(ms.max(1));
        let mut t = Timer::new(dur, TimerMode::Repeating);
        t.set_elapsed(dur);
        Some(t)
    } else {
        None
    }
}

/// Pick a cooldown timer where the component expects a one-shot (Once) timer that
/// is already elapsed (ready). This mirrors previous per-component behaviour
/// where cooldowns used TimerMode::Once and were ticked to ready state.
pub fn pick_timer_once<C, FM>(entity_cfg: Option<&C>, level_cfg: Option<&C>, ms_sel: FM) -> Option<Timer>
where
    FM: Fn(&C) -> Option<u64>,
{
    if let Some(ms) = pick(entity_cfg, level_cfg, ms_sel) {
        let dur = Duration::from_millis(ms.max(1));
        let mut t = Timer::new(dur, TimerMode::Once);
        t.set_elapsed(dur);
        Some(t)
    } else {
        None
    }
}

/// Macro to generate a boilerplate `override_from_config` impl for a component
/// given the component type, the config type, and lists of fields grouped by
/// pick helper function names. Example usage:
///
/// impl_override_from_config!(
///     AutoMeleeAttack, crate::game::level::configs::AutoMeleeAttackConfig,
///     pick_i32 => [damage],
///     pick_f32 => [range],
///     pick_bool => [enabled],
///     pick_timer => [cooldown],
/// );
#[macro_export]
macro_rules! __impl_override_dispatch {
    // integer fields
    (pick_i32, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( $recv.$field = $crate::helper::override_helpers::pick_i32($entity_cfg, $level_cfg, |c| c.$field).unwrap_or($recv.$field); )*
    };
    // f32 fields
    (pick_f32, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( $recv.$field = $crate::helper::override_helpers::pick_f32($entity_cfg, $level_cfg, |c| c.$field).unwrap_or($recv.$field); )*
    };
    // bool fields
    (pick_bool, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( $recv.$field = $crate::helper::override_helpers::pick_bool($entity_cfg, $level_cfg, |c| c.$field).unwrap_or($recv.$field); )*
    };
    // u64 fields
    (pick_u64, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( $recv.$field = $crate::helper::override_helpers::pick_u64($entity_cfg, $level_cfg, |c| c.$field).unwrap_or($recv.$field); )*
    };
    // string fields (component fields are Option<String>; prefer picked value or keep existing Option)
    (pick_string, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( $recv.$field = $crate::helper::override_helpers::pick_string($entity_cfg, $level_cfg, |c| c.$field.as_ref()).or($recv.$field.clone()); )*
    };
    // required string fields where the component stores a plain String (not Option)
    (pick_string_required, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( $recv.$field = $crate::helper::override_helpers::pick_string($entity_cfg, $level_cfg, |c| c.$field.as_ref()).unwrap_or($recv.$field.clone()); )*
    };
    // vec2 fields (from config arrays [f32;2])
    (pick_vec2, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( if let Some(a) = $crate::helper::override_helpers::pick_vec2($entity_cfg, $level_cfg, |c| c.$field) { $recv.$field = ::bevy::prelude::Vec2::new(a[0], a[1]); } )*
    };
    // waypoints: sequence of [f32;2] arrays -> Vec<Vec2>
    (pick_waypoints, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( if let Some(v) = $crate::helper::override_helpers::pick_waypoints($entity_cfg, $level_cfg, |c| c.$field.as_ref()) { $recv.$field = v; } )*
    };
    // facing: config supplies a string ("left"/"right") — map to enum
    (pick_facing, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( if let Some(d) = $crate::helper::override_helpers::pick_facing($entity_cfg, $level_cfg, |c| c.$field.as_ref()) { $recv.$field = d; } )*
    };
    // u32 fields in configs that map to signed i32 component fields (cast)
    (pick_u32, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( $recv.$field = $crate::helper::override_helpers::pick_u32($entity_cfg, $level_cfg, |c| c.$field).map(|v| v as i32).unwrap_or($recv.$field); )*
    };
    // u32 fields with a fallback to another u32 config field (e.g. current defaulting to max)
    // Syntax: pick_u32_default => [field:default_field]
    (pick_u32_default, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($pair:tt),*]) => {
        __impl_override_dispatch!(@parse_u32_default_pairs $recv, $entity_cfg, $level_cfg, $($pair),*);
    };

    // Helper recursion to parse pairs like `field:default_field` and expand to code.
    (@parse_u32_default_pairs $recv:ident, $entity_cfg:ident, $level_cfg:ident, ) => {};
    (@parse_u32_default_pairs $recv:ident, $entity_cfg:ident, $level_cfg:ident, $field:ident : $default:ident $(, $rest:tt)*) => {
        $recv.$field = $crate::helper::override_helpers::pick_u32($entity_cfg, $level_cfg, |c| c.$field)
            .map(|v| v as i32)
            .or_else(|| $crate::helper::override_helpers::pick_u32($entity_cfg, $level_cfg, |c| c.$default).map(|v| v as i32))
            .unwrap_or($recv.$field);
        __impl_override_dispatch!(@parse_u32_default_pairs $recv, $entity_cfg, $level_cfg $(, $rest)*);
    };
    // u8 fields in configs that map to u8 component fields
    (pick_u8, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( $recv.$field = $crate::helper::override_helpers::pick_u8($entity_cfg, $level_cfg, |c| c.$field).unwrap_or($recv.$field); )*
    };
    // timers: pick_timer => [field] expects the config to supply milliseconds (u64)
    (pick_timer, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( if let Some(t) = $crate::helper::override_helpers::pick_timer($entity_cfg, $level_cfg, |c| c.$field) { $recv.$field = t; } )*
    };
    // timers that should be created as one-shot (Once) and ticked to ready state
    (pick_timer_once, $recv:ident, $ty:ident, $cfg:path, $entity_cfg:ident, $level_cfg:ident, [$($field:ident),*]) => {
        $( if let Some(t) = $crate::helper::override_helpers::pick_timer_once($entity_cfg, $level_cfg, |c| c.$field) { $recv.$field = t; } )*
    };
}

#[macro_export]
macro_rules! impl_override_from_config {
    // Variant with a post-processing block that is executed before returning `self`.
    ($ty:ident, $cfg:path, $( $pick_fn:ident => [$($field:ident),*] ),* , $post:block ) => {
        impl $ty {
            pub fn override_from_config(mut self, entity_cfg: Option<&$cfg>, level_cfg: Option<&$cfg>) -> Self {
                // Introduce local names for cfg vars so helper dispatch macro can reference them.
                let entity_cfg = entity_cfg;
                let level_cfg = level_cfg;
                $(
                    $crate::__impl_override_dispatch!($pick_fn, self, $ty, $cfg, entity_cfg, level_cfg, [$($field),*]);
                )*
                $post
                self
            }
        }
    };

    // Backwards-compatible variant without a post block.
    ($ty:ident, $cfg:path, $( $pick_fn:ident => [$($field:ident),*] ),* $(,)?) => {
        impl $ty {
            pub fn override_from_config(mut self, entity_cfg: Option<&$cfg>, level_cfg: Option<&$cfg>) -> Self {
                let entity_cfg = entity_cfg;
                let level_cfg = level_cfg;
                $(
                    $crate::__impl_override_dispatch!($pick_fn, self, $ty, $cfg, entity_cfg, level_cfg, [$($field),*]);
                )*
                self
            }
        }
    };
}













