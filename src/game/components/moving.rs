use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub(crate) struct Moving {
    pub(crate) origin_x: f32,
    pub(crate) direction: f32,
    pub(crate) direction_change_timer: Timer,
    pub(crate) rng_state: u32,
    pub(crate) speed: f32,
}

impl Moving {
    pub(crate) fn new(origin_x: f32) -> Self {
        let mut moving = Self {
            origin_x,
            direction: 1.0,
            direction_change_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            rng_state: origin_x.to_bits() ^ 0xA5A5_A5A5,
            speed: 180.0,
        };

        moving.reset_direction_timer();
        // Set initial random speed variation
        moving.randomize_speed();
        moving
    }

    pub(crate) fn next_random_unit(&mut self) -> f32 {
        self.rng_state = self
            .rng_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);

        (self.rng_state >> 8) as f32 / (u32::MAX >> 8) as f32
    }

    pub(crate) fn reset_direction_timer(&mut self) {
        let seconds = 0.8 + self.next_random_unit() * 1.6;
        self.direction_change_timer = Timer::from_seconds(seconds, TimerMode::Repeating);
    }

    pub(crate) fn randomize_speed(&mut self) {
        // Speed varies between 70% and 130% of base speed (126 - 234)
        let speed_multiplier = 0.7 + self.next_random_unit() * 0.6;
        self.speed = 180.0 * speed_multiplier;
    }
}

pub(crate) fn insert(entity: &mut EntityCommands, origin_x: f32) {
    entity.insert(Moving::new(origin_x));
}
