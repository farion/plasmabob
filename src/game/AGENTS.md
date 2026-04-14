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

## Animation, States and Hitboxes

The animation of the entities is determined by their current state. Each state has a corresponding `animation` sequence, which is defined in the entity type json file. For example, the player character might have an idle animation sequence for the `Idle` state, a walking animation sequence for the `Moving` state, an attack animation sequence for the `MeleeAttacking` state, etc. The animation system will play the corresponding animation sequence based on the current state of the entity. This allows for more immersive and visually appealing gameplay, as the entities will have different animations for different actions and states.

The animation sprites are loaded during level setup, which means that the game will load the necessary sprites for the animations when the level is loaded. This allows for faster loading times and smoother gameplay, as the game will not need to load the sprites on the fly during gameplay. The sprites are defined in the entity type json file, which allows for easy customization of the animations for different entities. For example, the player character might have a different set of sprites for its animations than an enemy character, which allows for more variety and visual interest in the game.

If a state does not have a defined `animation` sequence or the sequence is empty, it will default to the idle animation sequence. This allows for more flexibility in defining the animations for the entities, as not every state needs to have a unique animation sequence. For example, if the player character does not have a unique animation sequence for the `Falling` state, it can simply use the idle animation sequence instead, which still allows for a visually appealing and immersive gameplay experience.

Also the `animation_frame_ms` value defines the speed of the animation, which can be used to make the animations faster or slower. For example, a fast animation might have a frame time of 100 ms, while a slow animation might have a frame time of 200 ms. This allows for more variety and customization in the animations, as different entities can have different animation speeds based on their characteristics and behaviors.

The `collider_box` value defines the hitbox of the entity for collision detection, which can be different for different states. For example, the player character might have a larger hitbox when it is standing still, and a smaller hitbox when it is crouching. This allows for more realistic interactions with the environment, as the hitbox will change based on the entity's current state and actions.

Additionally there is the `lock_ms` which defines how long the state is locked for, meaning that the entity cannot change its state until the lock time has passed. This can be used to prevent the entity from changing states too quickly, which can lead to more realistic and visually appealing animations. For example, when the player character performs a melee attack, it might be locked in the `MeleeAttacking` state for 500 ms, which allows the attack animation to play fully before the player can change to another state. This adds a layer of strategy and timing to the gameplay, as the player will need to consider the lock time when performing actions and planning their moves. This applies on top of existing state lock or timing rules.

States are also responsible for entity sounds. A state might have `sound_start`, `sound_loop`, and `sound_end` values, which define the sounds that are played when the state is entered, while the state is active, and when the state is exited, respectively. For example, when the player character enters the `MeleeAttacking` state, it might play a sword swing sound effect. While the player character is in the `MeleeAttacking` state, it might play a looping sound effect of the sword swinging. When the player character exits the `MeleeAttacking` state, it might play a sound effect of the sword hitting something or being sheathed. This adds another layer of immersion and feedback to the gameplay, as the sounds will enhance the visual animations and provide audio cues for the player's actions and interactions with the environment. It is important, that for a specific entity instance the sounds are played in a row:
  * state enter
  * play sound_start (if defined)
  * sound_start ends (or is not defined)
  * play sound_loop (if defined)
  * sound_loop ends (or is not defined)
  * state exit
  * play sound_end (if defined)
  * sound_end ends (or is not defined)
If state sounds for the same entity are overlapping this is okay. So if state1 has a sound_end and the state2 afterwards has a sound_start or sound_loop, the sound_end and sound_start/loop will overlap.

## HUD

The heads up display (HUD) contains the following elements:

* Health bar - displays the current health of the player character, which is updated in real-time as the player takes damage or heals. This allows the player to keep track of their health and make strategic decisions based on their current health status.
* Plasma cooldown - displays the cooldown time for the player's range attack, which is updated in real-time as the player uses their range attack. This allows the player to keep track of when they can use their range attack again and make strategic decisions based on the cooldown status.
* Ego bar - displays the current ego level of the player character, which is updated in real-time as the player performs certain actions or takes damage. This allows the player to keep track of their ego level and make strategic decisions based on their current ego status. The ego bar can be used for various gameplay mechanics, like unlocking special abilities, triggering certain events, etc. (ego is currently not implemented, but the HUD element is there for future use)
* Level time - displays the current time elapsed in the level, which is updated in real-time as the player progresses through the level. This allows the player to keep track of how long they have been playing the level and make strategic decisions based on the time elapsed, like trying to beat a certain time or managing their time effectively. Format is mm:ss, so minutes and seconds. For example, if the player has been playing the level for 2 minutes and 30 seconds, the level time would display as "02:30".
* Score - displays the current score of the player, which is updated in real-time as the player performs certain actions, like defeating enemies, collecting items, etc. This allows the player to keep track of their score and make strategic decisions based on their current score status. The score can be used for various gameplay mechanics, like unlocking special abilities, triggering certain events, etc. (score is currently not implemented, but the HUD element is there for future use)
* Lives - displays the current number of lives the player has, which is updated in real-time as the player loses lives or gains extra lives. This allows the player to keep track of their remaining lives and make strategic decisions based on their current life status. The lives can be used for various gameplay mechanics, like allowing the player to continue after losing all their health, triggering certain events, etc. (lives are currently not implemented, but the HUD element is there for future use)

Positioning of the elements:
Health bar, plasma cooldown, and ego bar are positioned in the top left corner of the screen, with the health bar being the closest to the corner, followed by the plasma cooldown and then the ego bar. The level time is positioned in the top center of the screen, while the score is positioned in the top right corner of the screen. This allows for easy visibility and access to important information for the player during gameplay. Lives are displayed as icons in the bottom left corner of the screen, allowing the player to easily see how many lives they have remaining at a glance.

Styling of the elements:
Basically all bars are looking the same, just with different colors and icons. The health bar is red, the plasma cooldown is blue, and the ego bar is yellow. The bars also have a white border. Each bar has an icon on the left side that represents what the bar is for, a heart icon (icons/heart.png) for the health bar, a lightning bolt icon (icons/plasma.png) for the plasma cooldown, and a brain icon (icons/ego.png) for the ego bar. The level time and score are displayed in white text with a black outline for better visibility against different backgrounds. The lives are displayed as heart icons (icons/heart.png) in the bottom left corner of the screen, with each icon representing one life. 

The implementation for the HUD lives in src/game/hud


If an entity is dying or dead it should not be able to deal damage, interact with the player in any way or got hit/block projectiles, but it can still be affected by gravity and other environmental factors until it is removed from the game world.
The health component defines the despawn. `despawn_on_death` which is a bool defines if the entity despawns on death and `despawn_delay_ms` defines how long it will take. If the `despawn_delay_ms` is over the entity will fade out for 500 ms and then be removed from the game world. If `despawn_on_death` is false, the entity will not despawn on death and will remain in the game world, but it will still be considered dead.