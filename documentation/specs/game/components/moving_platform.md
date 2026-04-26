MovingPlatform

Description
Data-driven moving platform that follows authored world-space waypoints. Systems move the platform along the path and handle repeating/stop behaviour. The platform transfers motion to grounded entities.

Configuration (JSON keys, types, defaults and explanation)

| Key | Type | Default | Description |
|---|---:|---:|---|
| waypoints | array of [x,y] | [] | Path points in world coordinates. Include the start position as the first entry. |
| speed | number | 0.0 | Movement speed in world units/sec. |
| repeat | boolean | false | If true the platform loops to waypoint 0 after the last point. |
| enabled | boolean | true | Allows pausing the platform at runtime without removing the component. |

Notes
- Runtime helper fields like target_index are not authored. Movement requires at least two waypoints and speed>0 to be active.
