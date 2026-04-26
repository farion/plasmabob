use bevy::prelude::{Component, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoMovementState {
    Idle,
    Patrol,
    Aggro,
    ReturnToOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoMovementDefaultStrategy {
    RandomPatrol,
    WaypointsPatrol,
    StandStill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoMovementAggroStrategy {
    Follow,
    Kiting,
}

/// Component for simple autonomous movement (used by enemies, platforms, etc.).
#[derive(Component, Debug, Clone)]
pub struct AutoMovement {
    /// Unit direction the entity will try to move in. Use Vec2::ZERO to stop.
    pub direction: Vec2,
    /// Speed in virtual units per second.
    pub speed: f32,
    /// Whether the movement is currently active.
    pub enabled: bool,
    pub aggro: bool,
    pub aggro_range: f32,
    pub deaggro_range: f32,
    pub aggro_strategy: AutoMovementAggroStrategy,
    pub default_strategy: AutoMovementDefaultStrategy,
    pub patrol_range: f32,
    pub patrol_pause_time: f32,
    pub patrol_waypoints: Vec<Vec2>,
    pub line_of_sight: bool,
    pub vision_angle: f32,
    pub vision_check_interval: f32,
    pub can_fall_when_following: bool,
    // Movement strategy fields related to ranged engagements (kiting/spacing)
    pub min_engage_distance: f32,
    pub kiting_enabled: bool,
    pub kiting_hp_threshold: f32,
    pub jump_on_default: bool,
    pub jump_on_aggro: bool,
    pub jump_on_return_to_origin: bool,
    pub jump_force: f32,
    /// Distance at which followers stop approaching the target. Default 0 = until contact.
    pub follow_stop_distance: f32,
    pub jump_cooldown: f32,
    pub jump_cooldown_remaining: f32,
    pub max_speed: f32,
    pub acceleration: f32,
    pub target_timeout: f32,
    pub share_aggro_with_team: Option<String>,
    pub aggro_sharing_radius: f32,
    pub state: AutoMovementState,
    pub origin: Vec2,
    pub has_origin: bool,
    pub patrol_direction: f32,
    pub patrol_pause_remaining: f32,
    pub patrol_waypoint_index: usize,
    pub vision_tick_remaining: f32,
    pub target_entity: Option<bevy::prelude::Entity>,
    pub last_known_target_pos: Option<Vec2>,
    pub last_target_seen_secs: f32,
}

impl Default for AutoMovement {
    fn default() -> Self {
        AutoMovement {
            direction: Vec2::ZERO,
            speed: 0.0,
            enabled: true,
            aggro: true,
            aggro_range: 6.0,
            deaggro_range: 8.0,
            aggro_strategy: AutoMovementAggroStrategy::Follow,
            default_strategy: AutoMovementDefaultStrategy::RandomPatrol,
            patrol_range: 4.0,
            patrol_pause_time: 0.6,
            patrol_waypoints: Vec::new(),
            line_of_sight: true,
            vision_angle: 120.0,
            vision_check_interval: 0.2,
            can_fall_when_following: true,
            min_engage_distance: 3.5,
            kiting_enabled: true,
            kiting_hp_threshold: 0.3,
            jump_on_default: false,
            jump_on_aggro: true,
            jump_on_return_to_origin: false,
            jump_force: 260.0,
            follow_stop_distance: 0.0,
            jump_cooldown: 0.6,
            jump_cooldown_remaining: 0.0,
            max_speed: 3.0,
            acceleration: 10.0,
            target_timeout: 3.0,
            share_aggro_with_team: None,
            aggro_sharing_radius: 12.0,
            state: AutoMovementState::Idle,
            origin: Vec2::ZERO,
            has_origin: false,
            patrol_direction: 1.0,
            patrol_pause_remaining: 0.0,
            patrol_waypoint_index: 0,
            vision_tick_remaining: 0.0,
            target_entity: None,
            last_known_target_pos: None,
            last_target_seen_secs: 0.0,
        }
    }
}

impl AutoMovement {
    pub fn jump_enabled_for_state(&self, state: AutoMovementState) -> bool {
        match state {
            AutoMovementState::Idle | AutoMovementState::Patrol => self.jump_on_default,
            AutoMovementState::Aggro => self.jump_on_aggro,
            AutoMovementState::ReturnToOrigin => self.jump_on_return_to_origin,
        }
    }

    pub fn override_from_config(
        mut self,
        entity_cfg: Option<&crate::game::level::configs::AutoMovementConfig>,
        level_cfg: Option<&crate::game::level::configs::AutoMovementConfig>,
    ) -> Self {
        if let Some(dir) =
            crate::helper::override_helpers::pick_vec2(entity_cfg, level_cfg, |c| c.direction)
        {
            self.direction = Vec2::new(dir[0], dir[1]);
        }

        self.speed = crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.speed)
            .unwrap_or(self.speed);
        self.enabled =
            crate::helper::override_helpers::pick_bool(entity_cfg, level_cfg, |c| c.enabled)
                .unwrap_or(self.enabled);
        self.aggro = crate::helper::override_helpers::pick_bool(entity_cfg, level_cfg, |c| c.aggro)
            .unwrap_or(self.aggro);
        self.aggro_range =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.aggro_range)
                .unwrap_or(self.aggro_range);
        self.deaggro_range =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.deaggro_range)
                .unwrap_or(self.deaggro_range);
        self.patrol_range =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.patrol_range)
                .unwrap_or(self.patrol_range);
        self.patrol_pause_time =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| {
                c.patrol_pause_time
            })
            .unwrap_or(self.patrol_pause_time);
        self.line_of_sight =
            crate::helper::override_helpers::pick_bool(entity_cfg, level_cfg, |c| c.line_of_sight)
                .unwrap_or(self.line_of_sight);
        self.vision_angle =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.vision_angle)
                .unwrap_or(self.vision_angle);
        self.vision_check_interval =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| {
                c.vision_check_interval
            })
            .unwrap_or(self.vision_check_interval);
        self.min_engage_distance =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| {
                c.min_engage_distance
            })
            .unwrap_or(self.min_engage_distance);
        self.kiting_enabled =
            crate::helper::override_helpers::pick_bool(entity_cfg, level_cfg, |c| {
                c.kiting_enabled
            })
            .unwrap_or(self.kiting_enabled);
        self.kiting_hp_threshold =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| {
                c.kiting_hp_threshold
            })
            .unwrap_or(self.kiting_hp_threshold);
        self.follow_stop_distance =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| {
                c.follow_stop_distance
            })
            .unwrap_or(self.follow_stop_distance);
        self.can_fall_when_following =
            crate::helper::override_helpers::pick_bool(entity_cfg, level_cfg, |c| {
                c.can_fall_when_following
            })
            .unwrap_or(self.can_fall_when_following);
        self.jump_on_default = crate::helper::override_helpers::pick_bool(
            entity_cfg,
            level_cfg,
            |c| c.jump_on_default,
        )
        .unwrap_or(self.jump_on_default);
        self.jump_on_aggro = crate::helper::override_helpers::pick_bool(
            entity_cfg,
            level_cfg,
            |c| c.jump_on_aggro,
        )
        .unwrap_or(self.jump_on_aggro);
        self.jump_on_return_to_origin = crate::helper::override_helpers::pick_bool(
            entity_cfg,
            level_cfg,
            |c| c.jump_on_return_to_origin,
        )
        .unwrap_or(self.jump_on_return_to_origin);
        self.jump_force =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.jump_force)
                .unwrap_or(self.jump_force);
        self.jump_cooldown =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.jump_cooldown)
                .unwrap_or(self.jump_cooldown);
        self.max_speed =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.max_speed)
                .unwrap_or(self.max_speed);
        self.acceleration =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.acceleration)
                .unwrap_or(self.acceleration);
        self.target_timeout =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| c.target_timeout)
                .unwrap_or(self.target_timeout);
        self.aggro_sharing_radius =
            crate::helper::override_helpers::pick_f32(entity_cfg, level_cfg, |c| {
                c.aggro_sharing_radius
            })
            .unwrap_or(self.aggro_sharing_radius);

        self.share_aggro_with_team =
            crate::helper::override_helpers::pick_string(entity_cfg, level_cfg, |c| {
                c.share_aggro_with_team.as_ref()
            })
            .or(self.share_aggro_with_team);

        if let Some(waypoints) =
            crate::helper::override_helpers::pick_waypoints(entity_cfg, level_cfg, |c| {
                c.patrol_waypoints.as_ref()
            })
        {
            self.patrol_waypoints = waypoints;
        }

        if let Some(strategy) =
            crate::helper::override_helpers::pick_string(entity_cfg, level_cfg, |c| {
                c.default_strategy.as_ref()
            })
        {
            self.default_strategy = match strategy.to_ascii_lowercase().as_str() {
                "waypoints" | "waypoints_patrol" => AutoMovementDefaultStrategy::WaypointsPatrol,
                "stand_still" | "standstill" => AutoMovementDefaultStrategy::StandStill,
                _ => AutoMovementDefaultStrategy::RandomPatrol,
            };
        }
        if let Some(strategy) =
            crate::helper::override_helpers::pick_string(entity_cfg, level_cfg, |c| {
                c.aggro_strategy.as_ref()
            })
        {
            self.aggro_strategy = match strategy.to_ascii_lowercase().as_str() {
                "follow" => AutoMovementAggroStrategy::Follow,
                "kite" | "kiting" => AutoMovementAggroStrategy::Kiting,
                _ => AutoMovementAggroStrategy::Follow,
            };
        }

        if self.deaggro_range <= self.aggro_range {
            self.deaggro_range = self.aggro_range + 0.01;
        }
        self.jump_cooldown_remaining = 0.0;
        self.patrol_pause_remaining = 0.0;
        self.vision_tick_remaining = 0.0;
        self
    }
}
