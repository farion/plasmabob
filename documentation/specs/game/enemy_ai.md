# Enemy AI

This document specifies the AutoMovement-driven enemy AI used by runtime entities. It describes component fields, the state machine, sensing and selection algorithms, movement heuristics, kiting, shared-aggro, debug tooling, and tuning defaults.

All distances are world units.

---

## Components & Fields

AutoMovement (extended)
- aggro: bool
- aggro_range: f32
- deaggro_range: f32            # MUST be greater than aggro_range (hysteresis)
- aggro_strategy: enum { Follow }
- default_strategy: enum { RandomPatrol, Waypoints, StandStill }
- patrol_range: f32
- patrol_pause_time: f32       # seconds to wait on direction/waypoint change
- patrol_waypoints: Option<Vec2[]> # world positions for Waypoints mode
- vision_enabled: bool
- vision_angle: f32            # degrees
- vision_check_interval: f32   # seconds (throttle raycasts)
- can_fall_when_following: bool # default true
- jump_cooldown: f32           # seconds
- max_speed: f32               # units / s
- acceleration: f32
- target_timeout: f32          # seconds to remember last known position
- share_aggro_with_team: Option<String> # team name
- aggro_sharing_radius: f32
- state: enum { Idle, Patrol, Aggro, ReturnToOrigin }
- last_known_target_pos: Option<Vec2>
- last_target_seen_at: Option<Time>

AutoRangeAttack (notes)
- aggro_range: f32             # authoritative aggro for ranged attack
- attack_range: f32
- min_engage_distance: f32     # back off if closer than this
- attack_cooldown: f32
- kiting_enabled: bool
- kiting_hp_threshold: f32    # HP fraction at which kiting starts (0.3)

AutoMeleeAttack (notes)
- attack_range: f32
- attack_cooldown: f32

Team
- name: String                 # existing component used for shared-aggro

ControlledMovement
- (player/AI-controlled entities used as targets)

Movement/Physics
- jump_force: f32              # already exists and used for jump heuristics

---

## State Machine

States: Idle, Patrol, Aggro, ReturnToOrigin

Transitions (high-level):
- Idle/Patrol -> Aggro
  - Conditions: aggro == true AND there exists at least one valid ControlledMovement candidate within aggro_range, inside vision cone and with LoS (raycast) clear.
  - Action: select a target (uniform random among valid candidates). Set target_entity, state = Aggro, last_known_target_pos, last_target_seen_at.
  - Shared-aggro: if share_aggro_with_team is set, propagate target to same-team entities within aggro_sharing_radius (they accept the target instantly and ignore LoS).

- Aggro -> ReturnToOrigin
  - Conditions: target distance > deaggro_range OR target not seen for target_timeout seconds and unreachable by heuristics.
  - Action: clear target when returning is complete; set state to default (Idle/Patrol) when origin reached.

- Aggro behaviour
  - Follow strategy: move toward target. Respect attack-component rules (melee closes distance; ranged maintains engagement band). Use local heuristics for jumps/falls/step-ups.

- Patrol behaviour
  - RandomPatrol: move left/right within patrol_range. Pause patrol_pause_time on direction change.
  - Waypoints: follow patrol_waypoints in order, pausing patrol_pause_time at each.

---

## Sensing & Target Selection

Vision pipeline (run per-enemy every vision_check_interval, staggered per-entity):
1) Broad-phase: query ControlledMovement entities within aggro_range (use spatial partition/grid for performance).
2) Angle test: check candidate within vision_angle/2 relative to facing vector.
3) Raycast LoS: raycast from enemy "eye" to candidate center. Raycast is blocked by EnvironmentTag entities (these colliders block vision). If a hit on an EnvironmentTag occurs before the candidate, LoS is blocked.
4) Valid targets: candidates that pass steps 1–3.
5) Selection: pick uniformly at random from valid targets.

Performance notes: use squared-distance checks and angle check before raycast. Stagger vision checks across frames to avoid raycast spikes.

---

## Movement Heuristics (no global navmesh by default)

This approach uses local platformer heuristics and a waypoint graph fallback. It is the recommended starting point.

Local raycasts / sensors (relative to enemy feet/eye):
- forward_ground_check: forward-down short ray to detect ground ahead (avoid falling in patrol).
- forward_obstacle_check: feet-level forward ray to detect blocking obstacles.
- step_up_check: short forward ray at step-up height (detect low obstacles to climb without jumping).
- head_clear_check: upward ray to ensure jump headroom.

Decision flow when pursuing a target:
- If forward_obstacle_check && step_up_check -> perform step-up.
- Else if forward_obstacle_check && head_clear_check && jump_cooldown expired && obstacle height <= max_jump_height -> jump.
- Else if forward_ground_check is false:
  - If can_fall_when_following -> allow falling to get closer to target.
  - Else -> stop and attempt alternative route or turn around (in patrol mode do not fall).

If heuristics repeatedly fail (stuck or waypoint unreachable), run A* on the waypoint graph (see Pathfinding) to compute a route and follow it segment-by-segment using the same heuristics.

---

## Range Attack Behavior & Kiting

Range attackers maintain an engagement band:
- min_engage_distance (suggested default 3.5)
- max_engage_distance = AutoRangeAttack.aggro_range (authoritative)

Behaviour:
- If distance < min_engage_distance -> back off (step back or strafe).
- If distance > max_engage_distance -> advance, while respecting fall/jump heuristics.

Kiting:
- Activation: if HP_fraction <= kiting_hp_threshold (0.30) AND kiting_enabled == true.
- Strategy: while attacking, strafe laterally and perform short backoff bursts to maintain distance. Parameters:
  - kite_strafe_speed = 0.6 * max_speed
  - kite_backoff_time = 0.75s
  - kite_direction_change_interval = 0.75s

Range firing: require a clear firing LoS (raycast check) before shooting so enemies do not shoot through walls. This is separate from aggro LoS checks used for target acquisition.

---

## Shared Aggro (Team)

When an entity enters Aggro on target T and share_aggro_with_team == Some(team_name):
- Query entities with Team.name == team_name within aggro_sharing_radius.
- For each teammate hit: set their target_entity = T and state = Aggro immediately. Teammates accept the target regardless of their own LoS (shared-aggro ignores LoS).
- Behaviour: shared target overwrites the current target/state immediately (no cooldown by default).

---

## Debugging / Developer Tools

- Global debug toggle (Ctrl+Shift+A) to show/hide debug draws.
- Draw circular outlines at entity origin:
  - Purple: auto_movement.aggro_range
  - Red: auto_range_attack.aggro_range
  - Green: auto_movement.deaggro_range
- Draw line to current target and render state label.
- Recommended draw library: bevy_prototype_lyon (or engine-native debug draw). Use pooling and avoid allocations each frame.

---

## Pathfinding

Approach: A (recommended) — heuristics + waypoint graph + occasional A* on graph.
- Build a small waypoint graph for complex navigation areas (platform edges, ladder tops, ground nodes).
- Use pathfinding::astar (pathfinding crate) for graph searches.
- Only run A* when heuristics fail or when following a waypoint route that requires complex movement.

---

## Defaults (initial tuning)
- aggro_range: 6.0
- deaggro_range: 8.0
- patrol_range: 4.0
- vision_angle: 120.0
- vision_check_interval: 0.2
- target_timeout: 3.0
- step_up_height: 0.5
- max_jump_height: ~1.5 (derived from jump_force)
- jump_cooldown: 0.6
- max_speed: 3.0
- acceleration: 10.0
- patrol_pause_time: 0.6
- min_engage_distance (range): 3.5
- aggro_sharing_radius: 12.0
- kiting_hp_threshold: 0.30

These are starting points for playtesting; tune per-enemy-type in the editor/level JSON.

---

## Performance Notes (target ~50 enemies)
- Use squared-distance checks and cheap angle tests before raycasts.
- Stagger vision checks across frames (per-entity time offset) to spread raycasts.
- Use a spatial partition (fixed grid/buckets) to find nearby ControlledMovement entities instead of querying all targets.
- Limit A* usage to entities that need it (stuck/unreachable). Cache routes where possible.

---

## Tests & QA Checklist
- Unit tests:
  - Aggro/deaggro transitions using simulated positions and time.
  - Target selection randomness determinism with seeded RNG.
  - Kiting activation on HP threshold.
  - Shared-aggro propagation and overwrite behaviour.
- Integration tests (physics):
  - Ground/ledge detection and patrol not falling.
  - Obstacle step-up and jump success/failure cases.
  - Waypoint A* routes on representative level geometry.
- Performance tests:
  - Spawn 50 enemies and profile vision/raycast/staggering behavior. Tune vision_check_interval and staggering.

---

## Pseudocode (systems summary)

VisionCheckSystem (throttled per-enemy):
1) if time_since_last_vision_check < vision_check_interval + per_entity_offset -> return
2) candidates = spatial_query_nearby_controlled(aggro_range)
3) valid_targets = []
4) for candidate in candidates:
   - if (candidate.pos - enemy.pos).len2() > aggro_range^2 -> continue
   - if angle_between(facing, dir_to_candidate) > vision_angle/2 -> continue
   - if raycast(enemy.eye, candidate.center) hits EnvironmentTag before candidate -> continue
   - push candidate to valid_targets
5) if valid_targets not empty:
   - chosen = uniform_random(valid_targets)
   - enemy.target_entity = chosen; enemy.state = Aggro; enemy.last_known_target_pos = chosen.pos; enemy.last_target_seen_at = now
   - if share_aggro_with_team set: propagate to teammates within aggro_sharing_radius (they accept target regardless of LoS)

MovementDecisionSystem:
- If state == Patrol: execute RandomPatrol or Waypoints behavior
- If state == Aggro: use heuristics to approach/maintain distance or kite (if ranged & low HP)
- If heuristics fail repeatedly: request path from waypoint graph using pathfinding::astar

ActionSystem:
- Range firing requires a firing LoS raycast; only shoot when clear and attack_cooldown expired
- Melee attacks if within attack_range

DebugDrawSystem:
- If debug enabled: draw circles for each entity and state label

---

If any of the fields, defaults, or behaviours above should be changed, specify the change and the spec will be updated accordingly. Implementation can follow this spec.
