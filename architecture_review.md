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

High Severity Findings (should address first)
-------------------------------------------
1) Blocking IO on Bevy main thread during level load
   - Where: src/game/level/loader.rs (load_level_from_asset) and related helpers used from OnEnter systems (src/game/game_view.rs::load_selected_level).
   - Problem: loader performs synchronous file IO and directory listing; it calls blocking helpers and pollster::block_on. These calls run on Bevy's main thread during OnEnter and will stall the app (drop frames / freeze) for large assets.
   - Recommendation: perform level parsing off the main thread and integrate with Bevy's asset system or task pool. Options:
     1. Convert the load system to spawn a background task with AsyncComputeTaskPool::get().spawn(), then insert the CachedLevelDefinition resource when ready. Use a RunCriteria or state machine so other systems wait until resource is present. This keeps the main thread responsive.
     2. Alternatively, lean on AssetServer/AssetCollection for streaming assets where feasible (textures/audio). For JSON, consider an async loader that returns a handle/resource and completes via futures.
   - Concrete pointer: change load_selected_level to schedule blocking parse on AsyncComputeTaskPool and insert result when complete (src/game/game_view.rs, src/game/level/loader.rs).

2) Physics & movement systems run at variable Update rate
   - Where: many gameplay systems registered in Update (src/game/systems/plugin.rs).
   - Problem: Physics-like systems (gravity_integration_system, movement_resolution_system, projectile_movement_system) run every Update frame. On variable or low frame rates this yields non-deterministic behavior and physics instability.
   - Recommendation: run physics / movement systems on a fixed timestep (e.g. 60 Hz) using Bevy's FixedTimestep run criteria (bevy_time::FixedTimestep) or a custom accumulator RunCriteria. Keep non-physics logic (AI decision, animation tick) at Update rate or in a separate, looser fixed-step.
   - Concrete pointer: group physics systems into a SystemSet with .run_if(FixedTimestep::step(1.0 / 60.0)). See src/game/systems/plugin.rs for the membership of GameplaySet::Physics.

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

Recommendations & Next Steps (practical)
---------------------------------------
1) Non-blocking level load plan (high priority)
   - Implement a small loader state and background task:
     - OnEnter(GameView) -> spawn async task reading/parsing JSON on AsyncComputeTaskPool.
     - Keep a lightweight resource like LoadingLevelTask(JoinHandle<Result<CachedLevelDefinition, _>>).
     - Poll the handle in an Update system; when ready insert CachedLevelDefinition and remove the task resource.
   - This is a small, local change and immediately improves startup responsiveness.

2) Fixed-step physics (high priority)
   - Add a FixedTimestep run criteria and move physics systems into that criteria. Example approach:
     - Create a SystemSet PhysicsFixed with .run_if(FixedTimestep::step(1.0 / 60.0)).
     - Register gravity_integration_system, movement_resolution_system, projectile_movement_system, projectile_collision_system in that set.
   - Keep finalization/animation/AI at Update or a separate, lower frequency fixed-step if needed.

3) Add broadphase culling for collisions (medium)
   - Implement a coarse uniform grid or simple AABB cache during movement resolution to prefilter collision pairs.

4) Add unit tests for deterministic math utilities (low friction)
   - Systems like swept_aabb_toi, ray_axis_times and resolve_axis are pure functions and suitable for unit tests (see src/game/systems/AGENTS.md recommendation).

Files to inspect first when implementing changes
-----------------------------------------------
- Level loader & game setup: src/game/level/loader.rs, src/game/game_view.rs
- Gameplay systems orchestration: src/game/systems/plugin.rs
- Movement & physics systems: src/game/systems/movement_resolution_system.rs, gravity_integration_system.rs, projectile_* files
- HUD, views and plugin composition: src/views/mod.rs and src/main.rs

Closing
-------
The codebase is in good shape and follows many Bevy conventions already. The most important improvements are non-blocking IO for level loading and moving physics to a fixed timestep. Both changes will greatly increase perceived responsiveness and physics reliability. If you want, I can implement the non-blocking loader change (small, safe change) or add the fixed-timestep migration for physics systems — which would you prefer me to do first?
