use bevy::prelude::Component;

/// State enum for entities. Mirrors the states described in the project AGENTS.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityState {
    Idle,
    Moving,
    MeleeAttacking,
    RangeAttacking,
    Damaged,
    Dying,
    Dead,
    Jumping,
    Falling,
    Crouching,
}

/// Component that holds the current state of an entity and how long it has been in that state.
#[derive(Component, Debug, Clone, Copy)]
pub struct StateMachine {
    pub state: EntityState,
    pub prev_state: Option<EntityState>,
    /// Time in seconds the entity has been in `state`.
    pub state_time: f32,
    /// Duration in seconds the entity stays in the Dying state before transitioning to Dead.
    /// Can be overridden via JSON key `dying_duration_secs`. Defaults to 1.0.
    pub dying_duration_secs: f32,
}

impl StateMachine {
    /// Create a new state machine in the `Idle` state.
    pub fn new(state: EntityState) -> Self {
        StateMachine {
            state,
            prev_state: None,
            state_time: 0.0,
            dying_duration_secs: 1.0,
        }
    }

    /// Convenience constructor for the default Idle state.
    pub fn idle() -> Self {
        StateMachine::new(EntityState::Idle)
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

    /// Advance the internal timer by `dt` seconds. Systems should call this every frame.
    pub fn tick(&mut self, dt: f32) {
        self.state_time += dt;
    }

    /// Reset the state timer without changing state.
    pub fn reset_timer(&mut self) {
        self.state_time = 0.0;
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        StateMachine::idle()
    }
}

impl StateMachine {
    /// Apply overrides from a JSON `components.state_machine` object.
    ///
    /// Supported keys:
    /// - `initial_state`: string name of the starting state (e.g. `"idle"`, `"moving"`)
    /// - `state_time`: optional number to initialize the state's timer
    /// - `dying_duration_secs`: number — how long the entity stays in Dying before transitioning to Dead
    pub fn override_from_json(mut self, comp_obj: Option<&serde_json::Value>) -> Self {
        if let Some(serde_json::Value::Object(map)) = comp_obj {
            if let Some(s) = map.get("initial_state").and_then(|v| v.as_str()) {
                self.state = Self::entity_state_from_name(s);
            }
            if let Some(t) = map.get("state_time").and_then(|v| v.as_f64()) {
                self.state_time = t as f32;
            }
            if let Some(d) = map.get("dying_duration_secs").and_then(|v| v.as_f64()) {
                self.dying_duration_secs = (d as f32).max(0.0);
            }
        }
        self
    }

    /// Convert a state name string into the typed `EntityState` enum.
    pub fn entity_state_from_name(s: &str) -> EntityState {
        match s.to_ascii_lowercase().as_str() {
            "idle" => EntityState::Idle,
            "moving" | "walking" | "running" => EntityState::Moving,
            "jumping" => EntityState::Jumping,
            "falling" => EntityState::Falling,
            "damaged" | "hit" => EntityState::Damaged,
            "dying" => EntityState::Dying,
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

    /// Create a StateMachine from a textual state name (convenience).
    pub fn from_state_name(s: &str) -> Self {
        StateMachine::new(Self::entity_state_from_name(s))
    }
}

impl EntityState {
    /// Return the canonical lowercase state name used in JSON and the asset cache.
    pub fn to_state_name(self) -> &'static str {
        match self {
            EntityState::Idle => "idle",
            EntityState::Moving => "moving",
            EntityState::Jumping => "jumping",
            EntityState::Falling => "falling",
            EntityState::Damaged => "damaged",
            EntityState::Dying => "dying",
            EntityState::Dead => "dead",
            EntityState::MeleeAttacking => "melee_attacking",
            EntityState::RangeAttacking => "range_attacking",
            EntityState::Crouching => "crouching",
        }
    }
}

