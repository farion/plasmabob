Architecture Review — PlasmaBob
=================================

Summary
-------
This review inspects the repository layout, Bevy usage patterns, system registration, resource and IO choices, and general architecture. Overall the project is well-organised: systems are one-per-file, game/view separation is clear, and SystemSets are used to control ordering. The code follows many Bevy best practices already (plugins, sets, run_if by AppState). Below are prioritized findings and concrete recommendations to improve stability, performance, and maintainability.

Strengths
---------
- Clear module separation: src/game/{components,systems,hud,setup,gfx} and src/views are well structured.
- One file per system/component rule is followed which improves discovery and maintenance (see src/game/systems/plugin.rs and AGENTS.md).
- Uses SystemSets and explicit ordering (GameplaySet chain) to structure game loop stages — good for concurrency and clarity (src/game/systems/plugin.rs).
- Uses run_if(in_state(AppState::GameView)) to avoid running gameplay systems outside GameView. Good resource life-cycle management via OnEnter/OnExit in GameViewPlugin (src/game/game_view.rs).
- UI scaling, custom fonts, and plugin composition show good use of Bevy plugin model (src/main.rs, src/helper/fonts.rs).

High Severity Findings (addressed)
---------------------------------
The two high-priority issues identified in the original review have been implemented.

1) Non-blocking level loading
   - Status: Implemented.
   - What changed: level JSON and entity-type parsing are performed on the compute task pool instead of blocking the main thread. The LoadView now spawns an async task (AsyncComputeTaskPool) to run game::level::loader::load_level_from_asset(...) and polls the task each frame; when ready it inserts the CachedLevelDefinition resource and continues asset loading. GameView no longer performs blocking loads and redirects to LoadView when the cache is missing.
   - Files: src/views/load_view.rs, src/game/game_view.rs

2) Fixed-step physics
   - Status: Implemented (configurable).
   - What changed: physics-related systems (gravity integration, movement resolution, projectile movement & collision, grounding evaluation, related collision steps) run in the FixedUpdate stage instead of Update. The physics tick is configurable via environment variable PLASMABOB_PHYSICS_HZ and defaults to 60 Hz. Non-physics systems (input, AI, finalization, UI) still run on Update.
   - Files: src/game/systems/plugin.rs, src/main.rs

Medium Severity Findings
------------------------
3) Large Query pairwise iterations can be costly
   - Where: projectile_collision_system, movement_resolution_system, other collision loops (src/game/systems/*).
   - Problem: naive O(n*m) pairing scales poorly as entity counts rise.
   - Recommendation: add a broadphase spatial partition (grid / quadtree) or use the physics library's broadphase if available. At minimum, add an axis-aligned bounding-box (AABB) cull to avoid expensive swept-AABB computations when bounding boxes don't overlap.

4) Use of ButtonInput<KeyCode> across many UI and gameplay systems
   - Where: many files (e.g. src/main.rs, views/*, systems/*).
   - Note: it works, but Bevy's more typical pattern is Input<KeyCode> / Input<MouseButton> which provides higher-level helpers. Consider standardizing to Input<> for intent clarity and easier testing.

Low Severity / Style
---------------------
5) Resources containing potentially large data (Strings, HashMaps)
   - Where: CachedLevelDefinition and LevelSelection resources (src/game/level/types.rs, src/main.rs).
   - Recommendation: ensure such resources are removed (commands.remove_resource) when not needed — GameView already removes CachedLevelDefinition on exit, good. Consider wrapping large optional fields in Arc to avoid large clones when shared.

6) Some single() / windows.single() patterns
   - Where: src/main.rs::update_ui_scale uses windows.single() with pattern matching which is safe. Just be aware single() returns error when the query has zero or multiple matches. This code already handles the error.

7) Logging and diagnostics
   - Suggestion: add bevy_diagnostic::FrameTimeDiagnosticsPlugin during development to gain insight into frame time and system timings. Tracing is used (tracing::info) which is good — ensure tracing_subscriber is initialised in main if not already.

Remaining Recommendations & Next Steps
------------------------------------
1) Broadphase culling for collisions (medium)
   - Why: collision loops (movement resolution and projectile collision) still perform pairwise checks and will become expensive as entity counts grow.
   - Suggestion: implement a coarse uniform grid or quadtree, or at minimum add an AABB broadphase cull before swept-AABB computations.
   - Files to target: src/game/systems/movement_resolution_system.rs, src/game/systems/projectile_collision_system.rs

2) Unit tests for deterministic math utilities (low effort)
   - Why: many math helpers (swept_aabb_toi, ray_axis_times, resolve_axis) are pure and make for stable unit tests.
   - Suggestion: add small test modules next to those helpers to guard against regressions.

3) Input API consistency (low)
   - Why: codebase uses ButtonInput<KeyCode> in many places. Input<KeyCode> is more idiomatic and offers convenience helpers.
   - Suggestion: gradually standardize on Input<KeyCode> in UI/game systems where appropriate.

Files to inspect first when implementing changes
-----------------------------------------------
- Level loader & game setup: src/game/level/loader.rs, src/game/game_view.rs
- Gameplay systems orchestration: src/game/systems/plugin.rs
- Movement & physics systems: src/game/systems/movement_resolution_system.rs, gravity_integration_system.rs, projectile_* files
- HUD, views and plugin composition: src/views/mod.rs and src/main.rs

Closing
-------
The codebase is in good shape and follows many Bevy conventions already. The most important improvements are non-blocking IO for level loading and moving physics to a fixed timestep. Both changes will greatly increase perceived responsiveness and physics reliability. If you want, I can implement the non-blocking loader change (small, safe change) or add the fixed-timestep migration for physics systems — which would you prefer me to do first?
