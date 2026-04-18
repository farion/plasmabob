use bevy::prelude::Component;
use crate::game::level::types::{StateMachineConfig, StateConfig};
use std::collections::HashMap;

/// State enum for entities. Mirrors the states described in the project AGENTS.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityState {
    Idle,
    Moving,
    MeleeAttacking,
    RangeAttacking,
    Damaged,
    Dying,
    Collected,
    Dead,
    Jumping,
    Falling,
    Crouching,
}

/// Component that holds the current state of an entity and how long it has been in that state.
#[derive(Component, Debug, Clone)]
pub struct StateMachine {
    pub state: EntityState,
    pub prev_state: Option<EntityState>,
    /// Time in seconds the entity has been in `state`.
    pub state_time: f32,
    /// Duration in seconds the entity stays in the Dying state before transitioning to Dead.
    /// Can be overridden via JSON key `dying_duration_secs`. Defaults to 1.0.
    pub dying_duration_secs: f32,
    /// Authored initial state as an enum value parsed from the JSON name.
    pub initial_state: EntityState,
    /// Authored per-state configuration indexed by typed `EntityState`.
    pub states: HashMap<EntityState, StateConfig>,
}

impl StateMachine {
    /// Create a new state machine in the `Idle` state.
    pub fn new(state: EntityState) -> Self {
        StateMachine {
            state,
            prev_state: None,
            state_time: 0.0,
            dying_duration_secs: 1.0,
            initial_state: state,
            states: HashMap::new(),
        }
    }

    /// Build a runtime StateMachine from authored StateMachineConfig.
    pub fn from_config(cfg: &StateMachineConfig) -> Self {
        let state = Self::entity_state_from_name(&cfg.initial_state);
        let mut sm = StateMachine::new(state);
        sm.initial_state = state;
        // Convert string-keyed states into typed map. Unknown state names are
        // skipped with a warning.
        for (name, sc) in cfg.states.iter() {
            let es = Self::entity_state_from_name(name);
            // If the name didn't map to a known state the function already
            // logs a warning and returns Idle; avoid clobbering by comparing
            // the lowercase equality to ensure mapping was explicit.
            if name.to_ascii_lowercase() == es.to_state_name() {
                sm.states.insert(es, sc.clone());
            } else {
                tracing::warn!(state = %name, "StateMachine::from_config: unknown state name, skipping");
            }
        }
        sm
    }


    /// Transition to a new state. Resets the state timer and records the previous state.
    pub fn set_state(&mut self, new_state: EntityState) {
        if self.state != new_state {
            self.prev_state = Some(self.state);
            self.state = new_state;
            self.state_time = 0.0;
        }
    }

    /// Returns true if the current state matches `s`.
    pub fn is(&self, s: EntityState) -> bool {
        self.state == s
    }

    /// Returns true when the entity should no longer interact with combat/gameplay systems.
    pub fn is_non_interactive(&self) -> bool {
        self.state.is_non_interactive()
    }

    /// Advance the internal timer by `dt` seconds. Systems should call this every frame.
    pub fn tick(&mut self, dt: f32) {
        self.state_time += dt;
    }

}

// JSON override removed: prefer typed `override_from_config` for StateMachine.
// JSON override removed: prefer typed `override_from_config` for StateMachine.

impl StateMachine {

    /// Convert a state name string into the typed `EntityState` enum.
    pub fn entity_state_from_name(s: &str) -> EntityState {
        match s.to_ascii_lowercase().as_str() {
            "idle" => EntityState::Idle,
            "moving" | "walking" | "running" => EntityState::Moving,
            "jumping" => EntityState::Jumping,
            "falling" => EntityState::Falling,
            "damaged" | "hit" => EntityState::Damaged,
            "dying" => EntityState::Dying,
            "collected" => EntityState::Collected,
            "dead" => EntityState::Dead,
            "melee_attacking" | "meleeattacking" => EntityState::MeleeAttacking,
            "range_attacking" | "rangeattacking" => EntityState::RangeAttacking,
            "crouching" => EntityState::Crouching,
            _ => {
                tracing::warn!(state = %s, "Unknown entity state string, defaulting to Idle");
                EntityState::Idle
            }
        }
    }
}

impl EntityState {
    /// Dying/Dead entities should not be targetable or deal damage anymore.
    pub fn is_non_interactive(self) -> bool {
        matches!(self, EntityState::Dying | EntityState::Dead | EntityState::Collected)
    }

    /// Return the canonical lowercase state name used in JSON and the asset cache.
    pub fn to_state_name(self) -> &'static str {
        match self {
            EntityState::Idle => "idle",
            EntityState::Moving => "moving",
            EntityState::Jumping => "jumping",
            EntityState::Falling => "falling",
            EntityState::Damaged => "damaged",
            EntityState::Dying => "dying",
            EntityState::Collected => "collected",
            EntityState::Dead => "dead",
            EntityState::MeleeAttacking => "melee_attacking",
            EntityState::RangeAttacking => "range_attacking",
            EntityState::Crouching => "crouching",
        }
    }
}

