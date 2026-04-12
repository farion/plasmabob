# Agent information for game


## Structure

- `src/game/gfx` - effects like particle filters, shaders
- `src/game/gameplay/systems` - systems that affect gameplay, like health, damage, movement, collision etc. 
- `src/game/gameplay/componets` - components that are used by the systems, like health, damage, movement
- `src/game/hud` - heads up display, like health bar, ammo count, etc.
- `src/game/input` - input handling, like keyboard, mouse, gamepad etc.
- `src/game/setup` - setup code for the agent, like loading models, animations, etc.

## Architectural decisions

- One file per system, component, effect, etc. for better organization and maintainability. This is important and must not be ignored by agents.

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

- `Collider` - defines the shape and size of the entity for collision detection. This can be a rectangle, circle, polygon, etc.
- `RigidBody` - defines the physics properties of the entity, like mass, friction, etc.
- `Health` - defines the health of the entity, and how it can be damaged or healed.
- `AutoMovement` - defines the movement of the entity, like speed, direction, etc. This is used for enemies that move on their own.
- `ControlledMovement` - defines the movement of the entity, like speed, direction that is controlled by the player. This is used for the player character.
- `ControlledRangeAttack` - defines the attack of the entity, like damage, range, etc. This is used for the player character.
- `AutoRangeAttack` - defines the attack of the entity, like damage, range, etc. This is used for enemies that attack on their own.
- `AutoMeleeAttack` - defines the attack of the entity, like damage, range, etc. This is used for enemies that attack on their own.
- `ControlledMeleeAttack` - defines the attack of the entity, like damage, range, etc. This is used for the player character.
- `PlayerInput` - defines the input for the player character, like keyboard, mouse, gamepad etc.
- `Blocking` - defines whether the entity blocks movement or attacks, like walls, crates, etc.
- `Gravity` - defines whether the entity is affected by gravity, like the player character, enemies, etc.
## Categories

Categories are Marker/Tag Components ...Tag

- `Player` - the main character controlled by the player
- `Enemy` - the main antagonist controlled by the game
- `NPC` - non-player characters that can interact with the player
- `Doodad` - objects that are not interactive but add to the environment, like trees, rocks, etc.
- `Item` - objects that block the player or enemies, like crates, barrels, etc.
- `Environment` - objects that make up the game world, like walls, floors, etc
- `MovingPlatform` - platforms that move in a predetermined path, which the player can use to reach different areas of the level.
- `Projectile` - objects that are fired by the player or enemies, like bullets, arrows, etc.
- `Effect` - visual or audio effects that are triggered by certain actions, like explosions,
- `Trigger` - objects that trigger certain events when the player interacts with them, like pressure plates, levers, etc.
- `Portal` - objects that teleport the player to another location in the game world.
- `Exit` - objects that allow the player to exit the level or game.
- `Collectible` - objects that can be collected by the player for points, achievements, items, buffs etc.

## Collision Rules For Components

- Gravity entities collide with Blocking entities.
- Gravity entities falling if not on top of a Blocking entity.
- Gravity entities are grounded on top of Blocking entities. This means that if a Gravity entity is on top of a Blocking entity, it is considered grounded and can jump or perform other actions that require being on the ground. Also if the blocking entity moves, the gravity entity will move with it, like standing on a moving platform.
- Gravity entities are not grounded if they are on the side or bottom of a Blocking entity. This means that if a Gravity entity is on the side or bottom of a Blocking entity, it is not considered grounded and cannot jump or perform other actions that require being on the ground. Also if the blocking entity moves, the gravity entity will not move with it, like standing next to a moving platform.
- If a gravity entity is grounded or not depends on the angle of the surface it is on. If the surface is too steep, the gravity entity will not be considered grounded and will slide down the surface instead. This allows for more realistic movement and interactions with the environment, like sliding down a hill or being unable to stand on a steep slope. The threshold angle is 45 degrees, meaning that if the angle of the surface is greater than 45 degrees, the gravity entity will not be considered grounded and will slide down the surface instead.