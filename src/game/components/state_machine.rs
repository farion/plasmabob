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
}

impl StateMachine {
    /// Create a new state machine in the `Idle` state.
    pub fn new(state: EntityState) -> Self {
        StateMachine {
            state,
            prev_state: None,
            state_time: 0.0,
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

