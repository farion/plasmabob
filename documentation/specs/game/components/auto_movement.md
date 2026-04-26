AutoMovement

Description
The AutoMovement component provides data for simple autonomous movement behaviors used by enemies, NPCs and some platform types. It supports patrols, aggro/follow behaviour, vision checks and kiting parameters. Several fields are runtime-only (target tracking, current state and timers) and are not authored in JSON.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| direction | [x,y] array | [0.0, 0.0] | Unit direction vector the entity will try to move in (Vec2). Use [0,0] to stop. |
| speed | number | 0.0 | Movement speed (virtual units/sec). |
| enabled | boolean | true | Whether autonomous movement is active. |
| aggro | boolean | true | Whether the entity can enter aggro mode when seeing targets. |
| aggro_range | number | 6.0 | Distance used to detect targets and enter aggro (world units). |
| deaggro_range | number | 8.0 | Distance beyond which the entity will give up target (must be > aggro_range). |
| aggro_strategy | string | "follow" | Aggro behaviour: "follow" or "kiting". |
| default_strategy | string | "randompatrol" | Default patrol strategy: "randompatrol", "waypointspatrol", or "standstill". |
| patrol_range | number | 4.0 | Radius for random patrol behaviour (world units). |
| patrol_pause_time | number | 0.6 | Pause time (seconds) between patrol movements. |
| patrol_waypoints | array of [x,y] | [] | Explicit list of world-space waypoints for waypoint patrol. First waypoint is expected to include the start position. |
| line_of_sight | boolean | true | If true, vision checks require an unobstructed line-of-sight. |
| vision_angle | number | 120.0 | Vision cone angle in degrees. |
| vision_check_interval | number | 0.2 | How often (seconds) the entity performs vision checks. |
| can_fall_when_following | boolean | true | Whether the entity may fall off ledges while following a target. |
| min_engage_distance | number | 3.5 | Preferred minimum distance for ranged engagements (used by kiting logic). |
| kiting_enabled | boolean | true | Enable kiting behaviour when aggro strategy is kiting. |
| kiting_hp_threshold | number | 0.3 | HP fraction threshold below which kiting behaviour activates. |
| jump_on_default | boolean | false | Allow jumping while in default (non-aggro) states. |
| jump_on_aggro | boolean | true | Allow jumping while in aggro state. |
| jump_on_return_to_origin | boolean | false | Allow jumping when returning to origin. |
| jump_force | number | 260.0 | Jump impulse strength. |
| follow_stop_distance | number | 0.0 | Distance at which followers stop approaching their target (0 = until contact). |
| jump_cooldown | number | 0.6 | Minimum seconds between automatic jumps. |
| max_speed | number | 3.0 | Maximum horizontal movement speed clamp used by movement controller. |
| acceleration | number | 10.0 | Acceleration used to reach max_speed. |
| target_timeout | number | 3.0 | Seconds before losing a previously acquired target position. |
| share_aggro_with_team | string or null | null | Optional team name to broadcast aggro to nearby allies. |
| aggro_sharing_radius | number | 12.0 | Radius used when sharing aggro with team members (world units). |

Notes
- Runtime-only fields (not authored) include: state, origin, internal timers, target_entity and other tracking helpers.

Enums
- AutoMovementState options: Idle, Patrol, Aggro, ReturnToOrigin. The component's runtime `state` holds one of these.
- AutoMovementDefaultStrategy options (string values accepted in JSON): RandomPatrol, WaypointsPatrol, StandStill. JSON may use names like "randompatrol", "waypoints", "waypoints_patrol" or "standstill".
- AutoMovementAggroStrategy options: Follow, Kiting (JSON strings: "follow", "kiting", "kite").

State-specific attributes
- `jump_on_default`: applies to Idle and Patrol states (jumping allowed while patrolling/default behavior).
- `jump_on_aggro`: applies to Aggro state (allows jumps while following/engaging a target).
- `jump_on_return_to_origin`: applies to ReturnToOrigin state (allows jumps while returning to origin).
- `min_engage_distance`, `kiting_enabled`, `kiting_hp_threshold`: used by kiting behaviour when `aggro_strategy` = Kiting and during Aggro state.
