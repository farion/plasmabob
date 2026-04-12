# Agent information for game


## Structure

- `src/game/gfx` - effects like particle filters, shaders
- `src/game/systems` - systems that affect gameplay, like health, damage, movement, collision etc. 
- `src/game/components` - components that are used by the systems. Contains only components that can be used in the entity json files, like health, movement, attack, etc. Components that are not defined in the entity json files, but are added by the systems, like Projectile, are defined in `src/game/runtime_components` instead.
- `src/game/hud` - heads up display, like health bar, ammo count, etc.
- `src/game/tags` - marker/tag components, like Player, Enemy, NPC, etc.
- `src/game/setup` - setup code for the agent, like loading models, animations, etc.

**Important** Components and systems are written in a single file for each system, component, effect, etc.

## Architectural decisions

- **One file per system, component, effect, etc. for better organization and maintainability. This is important and must not be ignored by agents.**

## Basic concept

- Levels are loaded via a json file. This file contains all the information about the level, including the layout, objects, enemies, etc. The game will parse this file and create the level accordingly.
- Positions in the level are defined in a virtual coordinate system, where the top left corner of the level is (0, 0) and the bottom right corner is (width, height). This allows for easy scaling and positioning of objects in the level, regardless of the actual pixel dimensions of the level.

## Level data structure

Levels are defined in `assets/worlds/{worldname}/levels/{stagename}_level{number}.json`. Each level json file contains the following information:

- `entities` - a list of entities that are present in the level, along with their position. Each entity is defined by an entity type, which is defined in `assets/entity_types/*.json`. Each attribute from the entity type can be overridden in the level json file, allowing for easy customization of entities in different levels. For example, a player entity type might have a health component with a value of 100, but in a certain level, the player might start with only 50 health. This can be easily achieved by overriding the health component value in the level json file.
- `bounds` - the size of the level, which is used for collision detection and other systems that need to know the size of the level. This is width and height.
- `background` - the background image for the level, which is used for rendering the level. 
- `music` - the music that is played in the level, which is used for audio. This is an array of music tracks that can be played in the level, allowing for variety and dynamic music changes during gameplay.
- `story` - the story that is displayed before and after the level, which is used for storytelling and immersion. 
  - "start" - the story that is displayed before the level starts, which can be used to set up the story and provide context for the player.
  - "win" - the story that is displayed after the player wins the level, which can be used to reward the player and provide closure for the level.
  - "lose" - the story that is displayed after the player loses the level, which

## Entity Types

Entity Types are defined in `assets/entity_types/*.json`. Each entity type defines the components that the entity has, and the values for those components. For example a player entity type might have a health component with a value of 100, a ControlledMovement component with a speed of 5, etc. This allows for easy creation of new entities by simply defining a new entity type in a json file.


## Components

### Components that can be defined in the entity json files

- `Collider` - defines the shape and size of the entity for collision detection. This can be a rectangle, circle, polygon, etc.
- `RigidBody` - defines the physics properties of the entity, like mass, friction, etc.
- `Health` - defines the health of the entity, and how it can be damaged or healed.
- `AutoMovement` - defines the movement of the entity, like speed, direction, etc. This is used for enemies that move on their own.
- `ControlledMovement` - defines the movement of the entity, like speed, direction that is controlled by the player. This is used for the player character.
- `ControlledRangeAttack` - defines the attack of the entity, like damage, range, etc. This is used for the player character.
- `AutoRangeAttack` - defines the attack of the entity, like damage, range, etc. This is used for enemies that attack on their own.
- `AutoMeleeAttack` - defines the attack of the entity, like damage, range, etc. This is used for enemies that attack on their own.
- `ControlledMeleeAttack` - defines the attack of the entity, like damage, range, etc. This is used for the player character.
- `Blocking` - defines whether the entity blocks movement or attacks, like walls, crates, etc.
- `Gravity` - defines whether the entity is affected by gravity, like the player character, enemies, etc.
- `Damageable` - defines whether the entity can take damage, like the player character, enemies, etc.
- `StateMachine` - keeps track of the current state of the entity, like idle, moving, attacking, etc. This is used to determine the animation and effects that are triggered by certain actions.


### Components that are not defined in the entity json files, but are added by the systems

- `Projectile` - defines whether the entity is a projectile, like bullets, arrows, etc. This is added by the attack systems when an entity performs a range attack.


## Categories

Categories are Marker/Tag Components ...Tag

- `Player` - the main character controlled by the player
- `Enemy` - the main antagonist controlled by the game
- `NPC` - non-player characters that can interact with the player
- `Doodad` - objects that are not interactive but add to the environment, like trees, rocks, etc.
- `Item` - objects that block the player or enemies, like crates, barrels, etc.
- `Environment` - objects that make up the game world, like walls, floors, etc
- `MovingPlatform` - platforms that move in a predetermined path, which the player can use to reach different areas of the level.
- `Effect` - visual or audio effects that are triggered by certain actions, like explosions,
- `Trigger` - objects that trigger certain events when the player interacts with them, like pressure plates, levers, etc.
- `Portal` - objects that teleport the player to another location in the game world.
- `Exit` - objects that allow the player to exit the level or game.
- `Collectible` - objects that can be collected by the player for points, achievements, items, buffs etc.

## Collision Rules For Components

Basically collision filtering is based on `Collider` and `RigidBody`. Gameplay mechanic is based only on these components: `Gravity`, `Blocking`, `Damageable`, `Projectile`. The rules are as follows:

- Gravity entities collide with Blocking entities.
- Gravity entities falling if not on top of a Blocking entity.
- Gravity entities are grounded on top of Blocking entities. This means that if a Gravity entity is on top of a Blocking entity, it is considered grounded and can jump or perform other actions that require being on the ground. 
- Grounded (weight‑based): An entity is considered grounded when the sum of upward reaction forces from its supporting contacts is at least its weight — i.e. ΣF_normal_y ≥ m * g * support_threshold (use support_threshold ≈ 0.9–1.0); add a short hysteresis window (e.g. 0.05–0.15 s) before clearing grounded to avoid edge jitter.
- While grounded on a moving blocker/platform, the entity inherits the platform's displacement/velocity for that step.
- Side or bottom contacts never grant grounded and never transfer platform motion.
- Gravity entities are not grounded if they are on the side or bottom of a Blocking entity. This means that if a Gravity entity is on the side or bottom of a Blocking entity, it is not considered grounded and cannot jump or perform other actions that require being on the ground. Also if the blocking entity moves, the gravity entity will not move with it, like standing next to a moving platform.
- Grounded defaults:
  - `support_threshold = 0.95`
  - `ground_exit_hysteresis_sec = 0.10`
  - `max_ground_angle_deg = 45.0`
- If a gravity entity is grounded or not depends on the angle of the surface it is on. If the surface is too steep, the gravity entity will not be considered grounded and will slide down the surface instead. This allows for more realistic movement and interactions with the environment, like sliding down a hill or being unable to stand on a steep slope. The threshold angle is 45 degrees, meaning that if the angle of the surface is greater than 45 degrees, the gravity entity will not be considered grounded and will slide down the surface instead. Vector based: dot(contact_normal,up) >= cos(45°)
- Projectiles collide basically with Blocking and Damageable entities and are destroyed on impact. This means that if a projectile collides with a Blocking or Damageable entity, it is destroyed and does not continue to exist in the game world. This allows for more realistic interactions with the environment, like bullets being stopped by walls or crates. A projectile will always hit the first entity it collides with, meaning that if a projectile collides with a Blocking entity and a Damageable entity at the same time, it will hit the Blocking entity and be destroyed, and will not hit the Damageable entity. This allows for more realistic interactions with the environment, like bullets being stopped by walls or crates before they can hit enemies behind them. "First" is determined by **earliest time-of-impact (TOI)** within the simulation step. Projectile never collides with its owner entity. toi_epsilon=1e-4
- First hit is the earliest TOI. If TOI is equal within epsilon, resolve ties by:
  1) lower distance from projectile origin,
  2) `Blocking` before `Damageable`,
  3) lower entity id.
- Friendly fire rule: Projectiles keep track which entity fired them, and will not collide with entities that have the same team name (team component). This allows for more strategic gameplay, like being able to shoot through your own allies to hit enemies behind them, or being able to avoid friendly fire by shooting around your allies. For example, if the player character is on the "Player" team and fires a projectile, that projectile will not collide with other entities that are also on the "Player" team, but will collide with entities that are on the "Enemy" team. This allows for more strategic gameplay, like being able to shoot through your own allies to hit enemies behind them, or being able to avoid friendly fire by shooting around your allies. Default for team (if not specified in the json files) is "Neutral", meaning that if an entity does not have a team component, it is considered to be on the "Neutral". Means "Neutral" is a normal team label that work like the others - projectiles from "Neutral" entities will not collide with other "Neutral" entities.

## State System

Each entity does have exactly one state it is in at a time. The state defines the animation sequence and effects. The state also defines the collider hitbox, which can be different for different states. For example, the player character might have a larger hitbox when it is standing still, and a smaller hitbox when it is crouching. The state also defines the effects that are triggered by certain actions, like taking damage, attacking, etc. For example, when the player character takes damage, it might flash red and play a sound effect. When the player character attacks, it might play an attack animation and spawn a projectile.

- `Idle` - the default state of the entity, when it is not performing any actions.
- `Moving` - the state of the entity when it is moving, like walking, running, etc.
- `MeleeAttacking` - the state of the entity when it is performing an attack
- `RangeAttacking` - the state of the entity when it is performing a range attack
- `Damaged` - the state of the entity when it is taking damage, which can be used to trigger damage effects, like flashing red, playing a sound, etc.
- `Dying` - the state of the entity when it is dying, which can be used to trigger death effects, like playing a death animation, dropping loot, etc.
- `Dead` - the state of the entity when it is dead, which can be used to trigger death effects, like removing the entity from the game world, playing a death sound, etc.
- `Jumping` - the state of the entity when it is jumping, which can be used to trigger jump effects, like playing a jump animation, applying a jump force, etc.
- `Falling` - the state of the entity when it is falling, which can be used to trigger fall effects, like playing a fall animation, applying a fall force, etc.
- `Crouching` - the state of the entity when it is crouching, which can be used to trigger crouch effects, like playing a crouch animation, reducing the hitbox size, etc.

## Player Movement and Controls

The player can be controlled by the keyboard using the configurable key bindings defined in `src/key_bindings.rs`. 
Left, Right, Jump, Crouch (not yet implemented), Melee Attack (not yet implemented), Range Attack

## Enemy AI

Enemies can have different AI behaviors, which are defined by the components they have. For example, an enemy with the `AutoMovement` component will move on its own, while an enemy with the `AutoRangeAttack` component will attack on its own. The specific behavior of the enemy is determined by the values of these components. For example, an enemy with a high speed value in the `AutoMovement` component will move faster than an enemy with a low speed value. An enemy with a long range value in the `AutoRangeAttack` component will be able to attack from a greater distance than an enemy with a short range value. This allows for a wide variety of enemy behaviors and interactions with the player, like fast-moving enemies that rush towards the player, or slow-moving enemies that keep their distance and attack from afar.