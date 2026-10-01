<!-- Dossiê BRUTO da pesquisa de navegação (2026-10-01), colhido por um agente de pesquisa na web.
     Mantido VERBATIM, em inglês, como fonte do doc 29. Cada afirmação tem URL; [unverified] marca o que
     o agente não conseguiu conferir. A síntese e as decisões vivem no doc 29 e no plano 30. -->

# State of the art — AI navigation / pathfinding for 2D games (research for PH2D)

*Researched 2026-10-01. Target feature: "enemies that find their own way around walls" in a Rust 2D engine
(bevy_ecs 0.19, rapier2d 0.35, kurbo vector shapes). Every factual claim carries a URL; claims I could not
verify directly are flagged **[unverified]**.*

---

## 0. TL;DR (the shape of the answer)

1. **Every general-purpose engine converged on the same three-layer pipeline**: (a) a *walkable-area
   representation* (navmesh of convex polygons, or a grid), (b) a *global planner* (A* over polygons/cells
   producing a **corridor**, then a **funnel / string-pull** to get corner waypoints), (c) a *local layer*
   (steering + reciprocal velocity-obstacle avoidance, RVO/ORCA) that runs every frame and **ignores the
   navmesh**. Unity documents this split explicitly
   ([Unity "Inner Workings"](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavInnerWorkings.html)),
   Godot implements it as NavigationServer + RVO2
   ([NavigationServer2D](https://docs.godotengine.org/en/stable/classes/class_navigationserver2d.html)),
   Unreal as Recast/Detour + RVO or DetourCrowd
   ([UE avoidance](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-avoidance-with-the-navigation-system-in-unreal-engine)).
2. **Agent radius is baked into the mesh (erosion / Minkowski offset), not carried by the agent** in Godot,
   Unity and Unreal. Consequence: one navmesh per agent size. This is the #1 source of "stuck on corners"
   reports when designers assume `NavigationAgent2D.radius` affects the path (it only affects avoidance —
   [Godot docs](https://docs.godotengine.org/en/stable/classes/class_navigationagent2d.html),
   [proposal #5977](https://github.com/godotengine/godot-proposals/issues/5977)).
3. **The "simple" 2D engines (Construct, GDevelop, GameMaker, RPG Maker) all chose a grid + A*** with a
   handful of knobs (cell size, border/inflation, diagonals, smoothing) and *no* inter-agent avoidance.
   That is what they decided non-programmers can reason about.
4. **Truly state of the art for a 2D top-down game today**: a navmesh built by **polygon boolean +
   offset + constrained triangulation/convex partition** (Godot 4.3+ does exactly this with Clipper2 +
   polypartition — [PR #80796](https://github.com/godotengine/godot/pull/80796)), queried by
   **Polyanya** (optimal any-angle, no preprocessing — [Cui, Harabor, Grastien, IJCAI 2017](https://www.ijcai.org/proceedings/2017/0070.pdf))
   or A*+funnel, with a **path corridor** maintained incrementally (Detour `dtPathCorridor`), and **ORCA**
   for local avoidance. Flow fields win when *many agents share one goal* (RTS, zombies).
5. **For Rust specifically, the whole pipeline exists with permissive licences**: `i_overlay` (boolean +
   **buffering/offset**), `i_triangle` (triangulation + **convex decomposition**), `polyanya` (Polyanya,
   MIT/Apache), `landmass` (agents + corridor + ORCA via `dodgy_2d`, engine-agnostic core, MIT/Apache),
   `rerecast` (Recast port, MIT/Apache), `pathfinding` (generic A*). Only `vleue_navigator` drags in full
   Bevy (asset/mesh/camera features), which PH2D (bare `bevy_ecs`) would not want.
6. **Platformers are a different problem**: navmeshes do not model gravity. The standard answer is a
   *platform graph* with precomputed **jump/fall edges** simulated with the character's own movement
   parameters (Pignole; Branicki/Tuts+; Levi's *Surfacer* for Godot, MIT).

---

## 1. Godot 4.x (4.3 → 4.7)

**Version context.** Docs consulted are for **Godot 4.7** (stable 4.7.2,
[changelog](https://github.com/godotengine/godot/blob/master/CHANGELOG.md)). Key milestones:
- **4.3**: 2D navmesh *baking* for `NavigationRegion2D`/`NavigationServer2D`, using **Clipper2**
  Union/Difference ([PR #80796](https://github.com/godotengine/godot/pull/80796)).
- **4.4**: navigation **map synchronization moved to a background thread** — two map "iterations", one
  serving queries while the other rebuilds, swapped on completion
  ([PR #100497](https://github.com/godotengine/godot/pull/100497)); path query parameter limits added
  ([4.5 beta notes](https://godotengine.org/article/dev-snapshot-godot-4-5-beta-1/)).
- **4.5**: **dedicated 2D navigation server** (previously NavigationServer2D proxied the 3D server with an
  axis locked), 2D/3D navigation modules can be compiled out independently; region/link updates async;
  2D avoidance callbacks switched from `Vector3` to `Vector2`
  ([PR #101504](https://github.com/godotengine/godot/pull/101504),
  [4.5 release](https://godotengine.org/releases/4.5/)).
- **4.6/4.7**: I found only build/gizmo fixes in navigation, no new user-facing features
  ([changelog](https://github.com/godotengine/godot/blob/master/CHANGELOG.md)). **[partially verified]**

### 1.1 Nodes the designer gets

| Node / resource | Role |
|---|---|
| `NavigationRegion2D` + `NavigationPolygon` | the walkable area of one region; baked or hand-drawn |
| `NavigationAgent2D` | per-character path follower + optional RVO avoidance |
| `NavigationObstacle2D` | carves the bake *and/or* is an avoidance obstacle |
| `NavigationLink2D` | off-mesh connection (jump, ladder, teleport) |
| TileSet navigation layers | per-tile navigation polygons painted with the tile |
| `AStar2D`, `AStarGrid2D` | standalone graph/grid A*, independent of the server |

**`NavigationPolygon` (bake settings, 4.7 defaults)** — [class ref](https://docs.godotengine.org/en/stable/classes/class_navigationpolygon.html):
`agent_radius = 10.0` (erosion distance; keep > 0 to avoid precision errors), `cell_size = 1.0` (raster
grid for vertices; **must match the map's cell size**), `border_size = 0.0` (non-navigable border, for
tile-aligned chunks), `baking_rect` (restrict bake area), `parsed_geometry_type = BOTH` (meshes + static
colliders), `source_geometry_mode = ROOT_NODE_CHILDREN` (or group modes), `parsed_collision_mask = all`,
`sample_partition_type = CONVEX_PARTITION` (vs. TRIANGULATE).

Under the hood: outlines + parsed colliders → Clipper2 boolean/offset → **polypartition** convex partition
or triangulation ([PR #80796](https://github.com/godotengine/godot/pull/80796);
[COPYRIGHT.txt](https://raw.githubusercontent.com/godotengine/godot/master/COPYRIGHT.txt) lists
Clipper2 BSL-1.0 and polypartition Expat/MIT). Note: Recast is used **only for 3D**.

**Authoring gestures**: draw outline polygons on `NavigationRegion2D` with the polygon editor and press
*Bake NavigationPolygon* in the toolbar; or paint navigation polygons per tile in the TileSet. Docs warn
not to nest outlines of the same type (hole calculation becomes unpredictable and can flip polygons), and
that TileMap built-in navigation has **zero margin** to collision shapes — baking a region with
`agent_radius` is recommended instead
([Using navigation meshes](https://docs.godotengine.org/en/latest/tutorials/navigation/navigation_using_navigationmeshes.html);
[forum/bug write-up](https://bugnet.io/blog/fix-godot-navigation-agent-2d-stuck-on-obstacle)).
Runtime: `bake_navigation_polygon(on_thread)`; **source-geometry parsing must run on the main thread**
(SceneTree not thread-safe) and is named by the docs as *the* common runtime performance problem; advice is
to bake small chunks/regions rather than one large mesh.

**Map settings** — [NavigationServer2D](https://docs.godotengine.org/en/stable/classes/class_navigationserver2d.html):
`map_set_cell_size`, `map_set_edge_connection_margin` (regions whose edges are within the margin get
welded), `map_set_link_connection_radius`, `map_set_use_edge_connections`,
`map_set_use_async_iterations`. **Most changes take effect after the next physics frame** — querying in
`_ready()` returns an empty path (docs recommend deferring;
[agents tutorial](https://docs.godotengine.org/en/latest/tutorials/navigation/navigation_using_navigationagents.html)).
The server is thread-safe for queuing commands.

**`NavigationAgent2D` (4.7 defaults)** — [class ref](https://docs.godotengine.org/en/stable/classes/class_navigationagent2d.html):

| knob | default | meaning |
|---|---|---|
| `path_desired_distance` | 20.0 | distance at which the next waypoint counts as reached |
| `target_desired_distance` | 10.0 | distance at which the target counts as reached |
| `path_max_distance` | 100.0 | deviation from ideal path that triggers a repath |
| `path_postprocessing` | CORRIDORFUNNEL | or EDGECENTERED (grid-like layouts) or NONE |
| `simplify_path` / `simplify_epsilon` | false / 0.0 | drop non-critical waypoints |
| `navigation_layers` | 1 | which regions/links the agent may use |
| `pathfinding_algorithm` | A* | only A* exists |
| `path_metadata_flags` | 7 | return types, RIDs, owners per waypoint |
| `avoidance_enabled` | false | register in RVO |
| `radius` | 10.0 | **avoidance only, not pathfinding** |
| `neighbor_distance` | 500.0 | RVO neighbour search range |
| `max_neighbors` | 10 | |
| `time_horizon_agents` | 1.0 | RVO safety horizon vs agents |
| `time_horizon_obstacles` | 0.0 | vs static obstacles |
| `max_speed` | 100.0 | |
| `avoidance_layers` / `avoidance_mask` | 1 / 1 | who avoids whom |
| `avoidance_priority` | 1.0 | |
| `debug_enabled` + custom colors | false | per-agent path drawing |

Runtime API: set `target_position`; each physics frame call **`get_next_path_position()`** (the docs say
it must be called once per physics frame to advance internal state) until `is_navigation_finished()`;
with avoidance, write the desired `velocity` and move with the safe velocity delivered by the
**`velocity_computed`** signal. Signals: `path_changed`, `target_reached`, `navigation_finished`,
`waypoint_reached(details)`, and `link_reached(details)` **[link_reached not shown in the fetched summary — unverified]** ([class ref](https://docs.godotengine.org/en/stable/classes/class_navigationagent2d.html)).

**Path query** (`NavigationPathQueryParameters2D`, [class ref](https://docs.godotengine.org/en/stable/classes/class_navigationpathqueryparameters2d.html)):
`path_postprocessing`, `simplify_path/epsilon`, `included_regions`/`excluded_regions`,
`path_search_max_polygons = 4096` (search resets to nearest found polygon beyond this),
`path_search_max_distance`, `path_return_max_length`, `path_return_max_radius`, `metadata_flags`,
`navigation_layers`. Result carries per-point type/RID/owner metadata.

**`NavigationObstacle2D`** ([class ref](https://docs.godotengine.org/en/stable/classes/class_navigationobstacle2d.html),
[tutorial](https://docs.godotengine.org/en/latest/tutorials/navigation/navigation_using_navigationobstacles.html)):
two independent jobs.
- *Bake*: `affect_navigation_mesh` (removes geometry inside the shape at bake),
  `carve_navigation_mesh` (stencil not offset by agent radius). Resolution limited by cell size.
- *Avoidance*: **static** obstacle = `vertices` (hard boundary, winding decides push-in/push-out, expensive
  to move, agents cannot predict a warp and can get trapped) vs **dynamic** obstacle = `radius` (soft
  repulsion, can move every frame, `velocity` lets agents predict it). The docs recommend toggling a
  moving door from static vertices to radius while it moves.

**`NavigationLink2D`** ([tutorial](https://docs.godotengine.org/en/latest/tutorials/navigation/navigation_using_navigationlinks.html)):
`start_position`, `end_position`, `bidirectional`, `enter_cost`, `travel_cost`, `navigation_layers`;
connects to polygons within `link_connection_radius`. **It provides no traversal movement** — game code
must react (NavigationAgent2D is believed to expose a `link_reached` signal with metadata **[unverified]**).

**`AStarGrid2D`** ([class ref](https://docs.godotengine.org/en/stable/classes/class_astargrid2d.html)):
`cell_size`, `region`, `offset`, `cell_shape` (square / isometric right / isometric down),
`diagonal_mode` (ALWAYS / NEVER / AT_LEAST_ONE_WALKABLE / ONLY_IF_NO_OBSTACLES), heuristics
(EUCLIDEAN default, MANHATTAN, OCTILE, CHEBYSHEV), `jumping_enabled` (JPS) — **enabling JPS disables
`weight_scale`**. API: `set_point_solid`, `set_point_weight_scale`, `get_id_path`, `get_point_path`
(with partial-path option), `update()`.

**Debug**: editor *Debug → Visible Navigation* draws navmesh, edge connections and paths at runtime;
colours under ProjectSettings `debug/shapes/navigation`; per-agent `debug_enabled`
([debug tools](https://docs.godotengine.org/en/stable/tutorials/navigation/navigation_debug_tools.html)).

### 1.2 Avoidance internals and licences

RVO2 (Apache-2.0, © 2016 UNC Chapel Hill) for avoidance; Recast (Zlib, Mononen) for 3D baking;
Clipper2 (BSL-1.0); polypartition (Expat) — [COPYRIGHT.txt](https://raw.githubusercontent.com/godotengine/godot/master/COPYRIGHT.txt).
Godot itself is MIT. **Avoidance ignores navmesh regions**: "Using the modified velocity directly may move
an agent outside of the traversable area" ([server docs](https://docs.godotengine.org/en/stable/classes/class_navigationserver2d.html)).

### 1.3 Known pain points (GitHub / forums)
- **Agents leave the mesh / hit walls at corners** when avoidance is on, because avoidance runs in its own
  space ([#60354](https://github.com/godotengine/godot/issues/60354), open since 2022).
- **Bake-radius mismatch**: `agent.radius` does not affect the path; clearance is baked
  ([write-up](https://bugnet.io/blog/fix-godot-navigation-agent-2d-stuck-on-obstacle),
  [forum](https://forum.godotengine.org/t/navigationagent2d-keeps-geting-stuck-on-corners/126027)).
- **Multiple agent sizes require one map + navmesh per size** ([different actor types](https://docs.godotengine.org/en/stable/tutorials/navigation/navigation_different_actor_types.html);
  proposals [#5977](https://github.com/godotengine/godot-proposals/issues/5977),
  [#5542](https://github.com/godotengine/godot-proposals/issues/5542)).
- **Empty path on first frame / after map changes** (sync delay), including a 4.3→4.4 regression with the
  singleton API returning `[]` ([#104283](https://github.com/godotengine/godot/issues/104283)).
- **"Dancing" from repathing every frame, backtracking when `path_desired_distance` is smaller than the
  per-frame step, waypoints "behind" the agent near edges** — listed in the official agents tutorial
  ([link](https://docs.godotengine.org/en/latest/tutorials/navigation/navigation_using_navigationagents.html)).
- Agents not avoiding `NavigationObstacle2D` (configuration confusion between bake vs avoidance roles)
  ([#82562](https://github.com/godotengine/godot/issues/82562), [#64220](https://github.com/godotengine/godot/issues/64220)).

---

## 2. Unity (AI Navigation package 2.0, Unity 6)

Package `com.unity.ai.navigation` 2.0.x ([manual](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/index.html)).
Core runtime is Recast/Detour-derived (convex polygons, A* on polygon graph, corridor, string pulling)
([Inner workings](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavInnerWorkings.html));
the C++ runtime itself is closed.

**Components**
- **NavMeshSurface** ([ref](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavMeshSurface.html)):
  Agent Type, Default Area (Walkable / Not Walkable / Jump), Generate Links, Use Geometry (render meshes vs
  physics colliders), Collect Objects (All / Volume / Current hierarchy / Modifier components), Include
  Layers; advanced: Override Voxel Size (**default = agent radius / 3**, i.e. 3 voxels per radius;
  1–2 for open areas, 4–6 for tight interiors), Override Tile Size, Minimum Region Area, Build Height Mesh.
- **NavMeshModifier / NavMeshModifierVolume**: override area type or exclude objects/volumes.
- **NavMeshLink** ([ref](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavMeshLink.html)):
  start/end points or transforms, Width, Cost override, Bidirectional, Area Type, Auto Update Positions,
  Activated.
- **NavMeshObstacle** ([ref](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavMeshObstacle.html)):
  Box/Capsule, **Carve**, Move Threshold, Time To Stationary, Carve Only Stationary. Non-carving obstacles
  only affect local avoidance; carving punches a hole in the navmesh. Unity's guidance: carve stationary
  obstacles, avoid moving ones ([inner workings](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavInnerWorkings.html)).
  **Never enable Agent and Obstacle on the same object** — the agent avoids itself, and with carving it
  keeps remapping to the edge of its own hole
  ([mixing components](https://docs.unity3d.com/2020.1/Documentation/Manual/nav-MixingComponents.html)).
- **NavMeshAgent** ([ref](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavMeshAgent.html)):
  Agent Type, Base Offset; Steering: Speed, Angular Speed, Acceleration, Stopping Distance, Auto Braking;
  Obstacle Avoidance: Radius, Height, Quality, Priority (0–99, lower = more important, default 50 —
  [API](https://docs.unity3d.com/530/Documentation/ScriptReference/NavMeshAgent-avoidancePriority.html));
  Path Finding: Auto Traverse Off-Mesh Link, Auto Repath, Area Mask. Editor defaults commonly seen are
  speed 3.5, angular speed 120, acceleration 8, stopping distance 0, auto braking on, radius 0.5,
  height 2, quality High **[unverified — not on the fetched pages]**.
- **Area costs** and **Agent Types** (each agent type = separate bake with its own radius/height/step/slope)
  — same "radius baked per mesh" decision as Godot.

**Avoidance**: Unity documents "reciprocal velocity obstacles (RVO)" choosing a velocity balancing desired
direction against future collisions with agents *and navmesh edges*
([inner workings](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavInnerWorkings.html)).
**[uncertain]** whether this is ORCA, Detour's sampling avoidance, or a custom variant — closed source.

**2D**: no official XY-plane support. Community **NavMeshPlus** (h8man, MIT, ~2.4k★, active 2026-08)
extends NavMeshSurface to collect Tilemaps, sprites and 2D colliders; the surface is rotated X-90 to face
the camera ([repo](https://github.com/h8man/NavMeshPlus)); agents need `updateRotation = false` and
`updateUpAxis = false` or the sprite gets rotated out of the plane
([HOW-TO](https://github.com/h8man/NavMeshPlus/wiki/HOW-TO)). The dominant commercial alternative is
Aron Granberg's **A* Pathfinding Project** (grid/point graphs incl. 2D in Free; recast graph, runtime
updates, multithreading, RVO/ECS movement in Pro; v5 moved to Burst/Jobs/ECS)
([free vs pro](https://arongranberg.com/astar/freevspro), [5.0 post](https://arongranberg.com/2024/02/a-pathfinding-project-5-0/)).

**Complaints** (recurring forum themes): 2D needs a third-party package; agents rotated out of plane;
agent + obstacle mixing; carving cost when obstacles move
([discussions](https://discussions.unity.com/t/2d-navmesh-pathfinding/682119)). **[themes summarized from
search results, not a systematic survey]**

---

## 3. Unreal Engine 5 (brief)

- **Recast/Detour navmesh**, tiled; polygons carry costs ([Navigation System](https://dev.epicgames.com/documentation/unreal-engine/navigation-system-in-unreal-engine)).
  Authoring: `NavMeshBoundsVolume` + `RecastNavMesh` actor settings; **P** toggles the navmesh overlay in the
  editor; Display options draw poly edges / tile bounds; Gameplay Debugger for runtime paths
  ([Basic navigation](https://dev.epicgames.com/documentation/en-us/unreal-engine/basic-navigation-in-unreal-engine)).
- **Generation modes**: Static, Dynamic (tiles regenerate at runtime), **Dynamic Modifiers Only** (mesh
  baked offline; at runtime only modifiers/areas/links change costs or block — up to 50% cheaper tile
  processing) ([modify navmesh](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-how-to-modify-the-navigation-mesh-in-unreal-engine)).
- **Navigation Invokers**: generate tiles only around registered actors (generation radius ~3000, removal
  ~5000 recommended) for open worlds ([invokers](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-navigation-invokers-in-unreal-engine)).
- **Modifiers & links**: Nav Modifier Volumes (area class/cost), **Nav Link Proxy** (simple links) and
  **Smart Links** (event on traversal, e.g. jump between platforms) ([same page](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-how-to-modify-the-navigation-mesh-in-unreal-engine)).
- **Avoidance — two exclusive systems**: RVO in CharacterMovementComponent (simple, navmesh-agnostic, can
  push agents out of bounds) vs **DetourCrowd** (`DetourCrowdAIController`/`CrowdFollowingComponent`:
  adaptive velocity sampling + corridor visibility/topology optimisation; fixed max agents in project
  settings) ([UE avoidance](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-avoidance-with-the-navigation-system-in-unreal-engine)).
- `PathFollowingComponent` executes paths; **EQS** chooses *where* to go (queries scored by tests), not how.
- **Mass + ZoneGraph**: for thousands of agents, *authored lane networks* (ZoneShape splines → baked
  ZoneGraphData, lane profiles and tags) rather than navmesh coverage
  ([Santori, ZoneGraph tips](https://christiansantori.com/blog/zone-graphs-unreal-engine-crowd-navigation)).

---

## 4. 2D-focused engines — what "simple" engines expose

| Engine | Built-in? | Representation | Exposed knobs | Avoidance |
|---|---|---|---|---|
| GameMaker | yes | grid (`mp_grid_*`), potential steering (`mp_potential_*`), linear (`mp_linear_*`) | cell size, region, diagonals, which instances are obstacles; potential: maxrot, rotstep, ahead, onspot | none (potential fn is a local hack) |
| Construct 3 | yes (behavior) | grid A* | cell size, cell border, obstacles = Solids or Custom, max speed, accel/decel, rotate speed, rotate object, diagonals; actions: find path, regenerate map / region / region around object, add obstacle/cost | none |
| GDevelop | yes (behavior, MIT) | grid A* | allow diagonals, accel, max speed, rotate + rotate speed, angle offset, cell width/height, grid offset, extra border, smoothing max cell gap; obstacle: impassable or cost | none |
| RPG Maker MV/MZ | yes | tile A* in `findDirectionTo` | none in UI; `searchLimit()` = 12 steps (override in plugin) | none |
| Defold | no (extension) | `defold-astar` (MicroPather), Defold Detour | grid/graph | — |
| Phaser | no (plugins) | easystar.js grid A*; `navmesh` polygon plugin | — | — |
| Cocos Creator 3.x | no | store plugins (Easy NavMesh: A* + funnel) | — | — |
| Stencyl | no | community A* extensions | — | — |
| LÖVE | no | Jumper (A*/JPS) | — | — |

Details and sources:

- **GameMaker** — three families: linear, potential ("step toward goal, deviate when blocked", Step event),
  and grid A* through cell centres; docs stress the cell size trade-off (large enough that a centred
  instance doesn't touch obstacles, small enough to keep gaps open)
  ([manual](https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Movement_And_Collisions/Motion_Planning/Motion_Planning.htm)).
- **Construct 3** — grid; larger cells are faster but close small gaps; *Cell border* expands obstacles;
  diagonal cost ×√2; *Regenerate obstacle map* is CPU-heavy, *Regenerate region (around object)* is the
  intended dynamic-obstacle tool; scripting `findPath(x, y)` is async and returns a promise
  ([manual](https://www.construct.net/en/make-games/manuals/construct-3/behavior-reference/pathfinding) —
  page returned 403 to me, facts taken from search snippets and the
  [script interface](https://www.construct.net/en/make-games/manuals/construct-3/scripting/scripting-reference/behavior-interfaces/pathfinding)) **[partially verified]**.
- **GDevelop** — properties listed above ([wiki](https://wiki.gdevelop.io/gdevelop5/behaviors/pathfinding/));
  docs warn *Move to a position* should run once, not every frame. Source:
  [`Extensions/PathfindingBehavior/pathfindingruntimebehavior.ts`](https://raw.githubusercontent.com/4ian/GDevelop/master/Extensions/PathfindingBehavior/pathfindingruntimebehavior.ts)
  — A* over virtual cells (default 20 px), Euclidean heuristic with diagonals / Manhattan without, cell cost
  = 1 + sum of passable obstacle costs, impassable = −1, edge cost = mean of the two cell costs × step
  factor, search aborted beyond a complexity factor (50), obstacles tested by **hitboxes inside the cell
  plus `extraBorder`** (an older issue reported bounding-box-only obstacles —
  [#2689](https://github.com/4ian/GDevelop/issues/2689)), path simplified with `smoothingMaxCellGap`.
  **Licence: Core, GDJS, IDE and Extensions are MIT** ([LICENSE.md](https://github.com/4ian/GDevelop/blob/master/LICENSE.md)).
  There is also an experimental **NavMesh** extension (D8H) built from `D8H/NavMeshGenerator` + `mikewesthad/navmesh`, MIT
  ([repo](https://github.com/D8H/NavMesh-GDevelop-Extension)).
- **RPG Maker** — `Game_Character.findDirectionTo` is A* with depth limit 12; plugins override
  `searchLimit` ([forum](https://forums.rpgmakerweb.com/threads/smarter-pathfinding-for-chasing-events.174229/)).
- **Defold** — `selimanac/defold-astar` native extension on MicroPather; Defold Detour for navmesh
  ([repo](https://github.com/selimanac/defold-astar)).
- **Phaser** — `easystar.js` (MIT, async A*, `setAcceptableTiles`, `enableDiagonals`, `setTileCost`,
  `setIterationsPerCalculation`) ([repo](https://github.com/prettymuchbryce/easystarjs));
  `mikewesthad/navmesh` (MIT, A* over polygons + funnel, 5×–150× faster than Phaser's grid A* plugin,
  `buildPolysFromGridMap`, last push 2023) ([repo](https://github.com/mikewesthad/navmesh)).
- **Cocos Creator 3.x** — no built-in 2D navigation; *Easy NavMesh* store plugin (A* + funnel, <40 KB)
  ([search summary / forum](https://forum.cocosengine.org/t/3d-automatic-pathfinding-comes-easy-with-the-navmesh-navigation-mesh-plug-in/55999)).
- **Stencyl** — community A* tile extensions ([jihem/Stencyl-DecisionMaking](https://github.com/jihem/Stencyl-DecisionMaking)).
- **LÖVE** — Jumper (MIT; A*, Dijkstra, BFS, DFS, JPS; last push 2022) ([repo](https://github.com/Yonaba/Jumper)).

**What the simple engines decided:** a *uniform grid* the designer can picture; obstacle membership comes
from the object's own collision (Solids / obstacle behavior), inflated by a *border*; *costs* as a number
on an obstacle; *diagonals* as a checkbox; *smoothing* as one number; the behavior both **finds and drives**
the object (speed, acceleration, rotation) so the designer never touches waypoints; dynamic change is a
*regenerate region* action, not automatic. None of them ships inter-agent avoidance.

---

## 5. Rust ecosystem (crates.io data fetched 2026-10-01)

| Crate | Version / last release | Downloads (all / recent) | Licence | What it is |
|---|---|---|---|---|
| [`pathfinding`](https://github.com/evenfurther/pathfinding) | 4.16.0 / 2026-09-07 | 2.98M / 548k | MIT/Apache | generic A*, BFS, DFS, Dijkstra (incl. bidirectional), fringe, IDA*, IDDFS, Yen; `Grid` type. **No JPS / Theta*** ([docs](https://docs.rs/pathfinding/latest/pathfinding/)) |
| [`polyanya`](https://github.com/vleue/polyanya) | 0.17.1 / 2026-08-25 | 97k / 7k | MIT OR Apache | Polyanya any-angle optimal search on navmesh; layers (one-way, costs, enable/disable); `Triangulation::from_outer_edges` + obstacles; deps spade, geo, glam 0.32, rstar ([README](https://github.com/vleue/polyanya)) |
| [`vleue_navigator`](https://github.com/vleue/vleue_navigator) | 0.16.0 / 2026-08-25 | 65k / 2k | MIT OR Apache | Bevy plugin over polyanya; navmesh from obstacle components; `NavMeshSettings`: `agent_radius`, `agent_radius_on_outer_edge`, `simplify`, `merge_steps`, `build_timeout`, layers/stitches; Bevy 0.19 ↔ 0.16 ([docs](https://docs.rs/vleue_navigator/latest/vleue_navigator/prelude/struct.NavMeshSettings.html)). Depends on `bevy` (mesh, camera, asset features) |
| [`landmass`](https://github.com/andriyDev/landmass) / `bevy_landmass` | 0.9.2 / 2026-07-18; bevy 0.12.1 / 2026-09-30 | 31k; 25k | MIT OR Apache | full navigation system: Archipelago/Island/Agent/Character/AnimationLink (off-mesh); path finding + SSFA funnel + steering + ORCA via `dodgy_2d`; 2D and 3D coordinate systems; engine-agnostic core; `Landmass2dPlugin`; debug draw ([README](https://github.com/andriyDev/landmass/blob/main/crates/landmass/README.md), [FAQ](https://github.com/andriyDev/landmass/blob/main/FAQ.md)) |
| [`rerecast`](https://github.com/janhohenheim/rerecast) (+`bevy_rerecast`, `avian_rerecast`) | 0.4.0 / 2026-08-05 | 46k / 19k | MIT OR Apache | Rust port of Recast (poly mesh + detail mesh), C++ comparison tests; 3D-oriented ([readme](https://github.com/janhohenheim/rerecast)) |
| [`oxidized_navigation`](https://github.com/TheGrimsey/oxidized_navigation) | 0.12.0 / 2024-12-25 | 56k | MIT OR Apache | **deprecated** in favour of rerecast; 3D tiled runtime Recast-like generation from parry3d colliders |
| [`navmesh`](https://github.com/PsichiX/navmesh) | 0.12.1 / 2021-11-30 | 25k | MIT OR Apache | NavMesh/NavGrid/NavIslands; unmaintained |
| [`recastnavigation-sys`](https://github.com/andriyDev/recastnavigation-rs-sys) | 1.0.3 / 2024-01-11 | 37k | Zlib (non-standard field) | raw FFI to C++ Recast/Detour |
| [`dodgy_2d`](https://github.com/andriyDev/dodgy) | 0.5.5 / 2025-04-24 | 27k | MIT OR Apache | ORCA (RVO2 algorithm) in Rust, 2D; `dodgy_3d`; old `dodgy` 0.3 MIT |
| [`bevy_flowfield_tiles_plugin`](https://github.com/BlondeBurrito/bevy_flowfield_tiles_plugin) | 0.15.0 / 2026-08-29 | 20k | MIT OR Apache (crates.io) | Emerson-style flow field tiles for Bevy |
| [`bevy_northstar`](https://github.com/jtothethree/bevy_northstar) | 0.7.0 / 2026-07-08 | 10k | MIT | hierarchical (HPA*-style) grid pathfinding for Bevy |
| [`grid_pathfinding`](https://github.com/tbvanderwoude/grid_pathfinding) | 0.3.0 / 2025-11-30 | 8.5k | MIT | JPS + connected components on 2D grids |
| [`spade`](https://github.com/Stoeoef/spade) | 2.15.1 / 2026-03-24 | 20M | MIT OR Apache | (constrained) Delaunay triangulation; used by polyanya |
| [`cdt`](https://github.com/Formlabs/foxtrot) | 0.1.0 / 2021-06-25 | 76k | MIT OR Apache | fast robust CDT; stale |
| [`i_triangle`](https://github.com/iShape-Rust/iTriangle) | 0.49.0 / 2026-09-19 | 83k | MIT OR Apache | integer-core triangulation, holes, self-intersections, Delaunay, **convex decomposition**, Steiner points ([README](https://github.com/iShape-Rust/iTriangle)) |
| [`earcutr`](https://github.com/frewsxcv/earcutr/) | 0.5.0 / 2025-05-29 | 12.8M | ISC | earcut port (ear clipping; no quality guarantees) |
| [`i_overlay`](https://github.com/iShape-Rust/iOverlay) | 9.0.0 / 2026-09-19 | 9.0M | MIT OR Apache | boolean ops + **buffering (offset polygon/path, line joins/caps)**; powers `geo` booleans ([README](https://github.com/iShape-Rust/iOverlay)) |
| [`clipper2`](https://github.com/tirithen/clipper2) | 0.6.0 / 2026-05-06 | 418k | MIT OR Apache (wrapper; Clipper2 C++ is BSL-1.0) | Rust API over Clipper2 |
| [`geo-clipper`](https://github.com/lelongg/geo-clipper) | 0.9.0 / 2025-02-08 | 937k | ISC | Clipper1 bindings |

Observations:
- **`i_overlay` buffering + `i_triangle` convex decomposition reproduce Godot's 2D bake pipeline**
  (Clipper2 offset + polypartition) in pure Rust with permissive licences.
- `polyanya` + `landmass` are the two maintained, engine-agnostic query/agent layers; both active in 2026.
- `vleue_navigator` requires `bevy` crate features beyond `bevy_ecs` — a hosting cost for an engine that
  uses only `bevy_ecs` **[inference from its Cargo.toml]**.
- There is **no maintained crate named `rvo2`** on crates.io (lookup returned not-found); `dodgy_2d` is the
  ORCA option. Upstream RVO2 C++ is Apache-2.0 ([snape/RVO2](https://github.com/snape/RVO2)).

---

## 6. Algorithms & literature

### 6.1 Global search on grids
- **A\*** with admissible heuristic; octile for 8-connected; tie-breaking toward larger g (or by
  cross-product) reduces node expansions on open maps **[classic, Amit Patel's notes — not fetched]**.
  Weighted A* (`f = g + w·h`, w>1) trades optimality for speed.
- **JPS** (Harabor & Grastien, AAAI 2011): prunes grid symmetries by jumping to "jump points"; optimal,
  no preprocessing; reported 25–30× faster on Baldur's Gate / Dragon Age maps
  ([paper](https://pdfs.semanticscholar.org/1745/ac37af62fad54aa680d8ea30008d429c2328.pdf)).
  Uniform-cost only (Godot disables weights with JPS).
- **JPS+** (Rabin, Game AI Pro 2): precomputes jump distances per cell/direction; ~2 orders of magnitude
  over A*; with **Goal Bounding** ~1500× ([chapter](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter14_JPS_Plus_An_Extreme_A_Star_Speed_Optimization_for_Static_Uniform_Cost_Grids.pdf),
  [repo](https://github.com/SteveRabin/JPSPlusWithGoalBounding) — no licence file). Static maps only.
- **Theta\*** (Nash, Daniel, Koenig, Felner, AAAI 2007): any-angle on grids by line-of-sight to grandparent;
  paths up to ~13% shorter than A*; **Lazy Theta\*** defers LOS checks (one per expansion)
  ([Theta*](http://idm-lab.org/bib/abstracts/papers/aaai07a.pdf), [Lazy Theta*](http://idm-lab.org/bib/abstracts/papers/aaai10b.pdf)).
- **HPA\*** (Botea, Müller, Schaeffer 2004): clusters + entrances, abstract graph, 4–6% suboptimal
  ([Semantic Scholar](https://www.semanticscholar.org/paper/Near-Optimal-Hierarchical-Path-Finding-Botea-M%C3%BCller/b0f0432ba69e4d730b93a75e3d19c8e9d811efac)).
- **Dijkstra maps** (Brian Walker, Brogue): goal-seeded distance field, monsters "roll downhill"; flee map by
  multiplying by ~−1.2 and rescanning; weighted sums of maps for multiple desires; shared by all monsters
  ([RogueBasin](https://www.roguebasin.com/index.php/The_Incredible_Power_of_Dijkstra_Maps)).

### 6.2 Global search on navmeshes
- **A\* over polygons → corridor → funnel.** Mononen's *Simple Stupid Funnel* restarts the loop at each new
  corner; assumes portals already shrunk by agent radius (one radius per mesh)
  ([blog](http://digestingduck.blogspot.com/2010/03/simple-stupid-funnel-algorithm.html), no explicit
  licence on the blog code; Detour's equivalent is Zlib).
- **Polyanya** (Cui, Harabor, Grastien, IJCAI 2017): "compromise-free" — optimal any-angle, online, no
  preprocessing, searches over (root, interval) nodes; beats Theta*-style and visibility-graph approaches;
  assumes static mesh and point agent (clearance must be baked)
  ([paper](https://www.ijcai.org/proceedings/2017/0070.pdf)). Original C++ is on Bitbucket
  ([link](https://bitbucket.org/dharabor/pathfinding/src/master/anyangle/polyanya/)) — licence
  **[not verified]**; the Rust port is MIT/Apache.
- **Path corridor** (Detour `dtPathCorridor`): keeps the polygon list and adjusts it as the agent moves
  (`movePosition`, `moveTargetPosition`), straightens by local visibility (`optimizePathVisibility`),
  partially replans locally (`optimizePathTopology`), validates after mesh changes (`isValid`,
  `trimInvalidPath`) — i.e. **no full repath per frame** ([docs](https://recastnav.com/classdtPathCorridor.html)).
  Unity describes the same idea ([inner workings](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavInnerWorkings.html)).

### 6.3 Many agents / shared goals
- **Flow field tiles** (Emerson, *Supreme Commander 2*, Game AI Pro ch. 23): 10×10 m sectors of 1 m cells;
  three fields — 8-bit **cost** (255 = wall), 24-bit **integration** (Eikonal wavefront from goal + flags),
  8-bit **flow** (direction index + flags); A* over a **portal graph** first ("merging A*" so groups share
  portals), then flow fields per portal window, **cached by portal** and shared between requests; a
  **line-of-sight pass** marks cells that can steer straight to the goal (removes diamond artifacts);
  dynamic walls = mark sector cost dirty, rebuild affected portal nodes and paths through a time-sliced
  priority queue; **cost stamps** for placed buildings; per-movement-type cost fields; large units via
  "wall cushioning" (dilation); **island IDs** to reject unreachable goals instantly; agents slide along
  walls and push each other with physics
  ([chapter PDF](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf)).
- **Continuum crowds** (Treuille, Cooper, Popović, SIGGRAPH 2006): dynamic potential field unifying global
  navigation and moving obstacles, no explicit collision avoidance ([ACM](https://dl.acm.org/doi/10.1145/1179352.1142008)).

### 6.4 Local layer
- **Steering behaviors** (Reynolds, GDC 1999): seek, flee, arrive, pursue, evade, wander, obstacle
  avoidance, wall/path following, leader following — locomotion-independent and composable
  ([paper](https://www.red3d.com/cwr/papers/1999/gdc99steer.html)).
- **RVO / ORCA**: RVO (van den Berg, Lin, Manocha, ICRA 2008) — each agent takes half the responsibility;
  ORCA (van den Berg, Guy, Lin, Manocha, ISRR 2011) — half-planes in velocity space, low-dimensional LP;
  RVO2 library (Apache-2.0) ([ORCA site](https://gamma.cs.unc.edu/ORCA/),
  [paper](https://gamma.cs.unc.edu/ORCA/publications/ORCA.pdf)).
- **Practitioner caveats** (Sunshine-Hill, Game AI Pro 3 ch. 19): RVO's collision-free proof holds only for
  two agents and no obstacles; with three agents oscillation returns but in practice small displacements
  accumulate and resolve it; **ORCA gets stuck at corners** because its sidedness constraint forbids the
  "snap" of switching passing sides while the preferred direction rotates around a corner; recommends a
  scenario library + automated test rig + keeping old algorithms around for A/B, and eyes-on checks for
  velocity flicker ([chapter PDF](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter19_RVO_and_ORCA_How_They_Really_Work.pdf)).

### 6.5 Dynamic obstacles — the three strategies
1. **Re-bake tiles/regions** (Godot region rebake, Unreal Dynamic, Construct *regenerate region*, Emerson
   dirty sectors). Cost proportional to tile size; needs async.
2. **Carving** (Unity NavMeshObstacle *carve*, Godot obstacle `carve_navigation_mesh` at bake, Unreal
   Dynamic Modifiers Only). Good for things that stop moving.
3. **Local avoidance only** (non-carving obstacles, RVO dynamic obstacles). Cheap, but agents can be trapped
   because the global path still goes through the obstacle.
Unity's explicit rule: moving → avoidance; stationary → carve
([inner workings](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavInnerWorkings.html)).

### 6.6 Other recurring mechanisms
- **Agent radius via erosion/Minkowski sum** of walkable area (or dilation of obstacles); one mesh per size
  (Godot/Unity/Unreal), or per movement type (Emerson).
- **Off-mesh links**: Godot NavigationLink2D, Unity NavMeshLink, Unreal Nav Link Proxy / Smart Link,
  landmass AnimationLink. In all of them the link only enters the graph; **traversal motion is game code**.
- **Area costs / layers**: Godot navigation layers + link costs + region costs; Unity areas + cost; Unreal
  NavArea classes + filters; GDevelop/Construct per-obstacle cost.
- **Repath policy**: Godot repaths on target change and on deviation > `path_max_distance`; Unity *Auto
  Repath* when a partial path ends; Detour keeps a corridor; GDevelop/RPG Maker recompute only when asked.
  Repathing every frame is an explicitly documented anti-pattern (Godot "dancing").
- **Stuck detection**: none of the engines exposes a first-class "stuck" signal in the pages consulted;
  landmass exposes an `AgentState` (e.g. `AgentNotOnNavMesh`) ([FAQ](https://github.com/andriyDev/landmass/blob/main/FAQ.md));
  Godot users build it from waypoint-progress timers **[community practice, not documented]**.
- **Velocity feedback**: landmass warns that agents slow at corners if the real (physics) velocity is not
  fed back — the steering needs it to anticipate corners ([FAQ](https://github.com/andriyDev/landmass/blob/main/FAQ.md)).

### 6.7 2D platformers (gravity)
A navmesh describes *where you can stand*, not *how you get there under gravity*. Approaches:
- **Platform graph + simulated jump links**: nodes on walkable surfaces, edges = walk / fall / jump
  computed by simulating the character's own jump at the movement parameters against static geometry
  (Pignole, "The Hobbyist Coder #3: 2D platformers pathfinding",
  [Game Developer](https://www.gamedeveloper.com/design/the-hobbyist-coder-3-2d-platformers-pathfinding---part-1-2);
  Unity port [PlatformerPathfinding2D](https://github.com/Candescence/PlatformerPathfinding2D), no licence).
- **Grid A* with a jump-value in the node state** (Branicki, Tuts+ series: theory, implementation, one-way
  platforms, multi-cell characters, ledge grabbing; code BSD-2-Clause)
  ([theory](https://code.tutsplus.com/how-to-adapt-a-pathfinding-to-a-2d-grid-based-platformer-theory--cms-24662t),
  [repo](https://github.com/tutsplus/A-Star-Pathfinding-for-Platformers)).
- **Surfacer** (Levi, Godot, MIT): parses tilemaps into floor/wall/ceiling surfaces, edges from constant-
  acceleration equations of motion matching the player's abilities, A* at runtime, edge *instructions*
  played back with corrections; costs: heavy precompute, no moving platforms, memory
  ([devlog](https://devlog.levi.dev/2021/09/building-platformer-ai-from-low-level.html),
  [repo](https://github.com/SnoringCatGames/surfacer)).
- In general engines this is expressed as **off-mesh links** (Unreal Smart Links "jump between platforms",
  Godot NavigationLink2D) with game-side traversal.

---

## 7. Debug visualisation & authoring gestures

| Engine | Overlay | Authoring gesture |
|---|---|---|
| Godot | *Debug → Visible Navigation*: polygons (cyan), edge connections, links, agent paths (`debug_enabled`), avoidance radii; colours in ProjectSettings ([docs](https://docs.godotengine.org/en/stable/tutorials/navigation/navigation_debug_tools.html)) | draw outlines in the NavigationRegion2D polygon editor → Bake; paint per-tile nav polygons |
| Unity | Scene view navmesh + agent path/avoidance gizmos (AI Navigation overlay) **[general knowledge]**; links black/red when active/inactive ([link ref](https://docs.unity3d.com/Packages/com.unity.ai.navigation@2.0/manual/NavMeshLink.html)) | add NavMeshSurface → Bake; modifiers/volumes by component |
| Unreal | **P** toggles navmesh; Draw Poly Edges / Tile Bounds; Gameplay Debugger ([basic nav](https://dev.epicgames.com/documentation/en-us/unreal-engine/basic-navigation-in-unreal-engine)) | drop NavMeshBoundsVolume; modifier volumes; link proxies |
| GameMaker | `mp_grid_draw` | code |
| Construct/GDevelop | debugger shows obstacle grid/path **[GDevelop has a "Pathfinding painter" extension — [wiki](https://wiki.gdevelop.io/gdevelop5/extensions/draw-pathfinding/)]** | tick "obstacle" behavior on objects; no area painting |
| landmass / vleue | `LandmassDebugPlugin`, `draw_archipelago_debug`; vleue gizmos feature | code / obstacle components |

---

## 8. Comparison table (feature × engine)

| Feature | Godot 4.7 | Unity AI Nav 2 | UE5 | Construct 3 | GDevelop | GameMaker | landmass (Rust) | polyanya/vleue (Rust) |
|---|---|---|---|---|---|---|---|---|
| Representation | navmesh (convex polys) + AStarGrid2D | navmesh (Recast-like) | navmesh (Recast) | grid | grid (+exp. navmesh ext.) | grid / potential | navmesh | navmesh (CDT) |
| Native 2D | yes (dedicated server 4.5+) | no (NavMeshPlus) | no | yes | yes | yes | yes (`Landmass2dPlugin`) | yes |
| Bake from colliders | yes (Clipper2) | yes (voxel) | yes (voxel) | yes (solids) | yes (obstacle behavior) | yes (instances) | no (bring your own mesh) | yes (obstacle components) |
| Agent radius | per mesh | per agent type (bake) | per agent config (bake) | cell border | extra border | cell size | per archipelago (`from_agent_radius`) | per mesh (`agent_radius`) |
| Query algorithm | A* + funnel / edge-centred | A* + string pull | A* + string pull | A* | A* + smoothing | A* | A* + SSFA funnel | **Polyanya (optimal any-angle)** |
| Path corridor / incremental | partial (repath on deviation) | yes | yes (Detour) | no | no | no | yes (agent path state) **[inferred]** | no |
| Off-mesh links | NavigationLink2D | NavMeshLink | Nav Link Proxy / Smart Link | no | no | no | AnimationLink | layers + stitches |
| Area costs | region/link costs, layers | areas | NavArea + filters | add cost | obstacle cost | — | node/polygon types | layer costs |
| Dynamic obstacles | rebake regions; avoidance obstacles | carve / avoidance | Dynamic / Modifiers-only / invokers | regenerate region | auto (obstacles queried live) **[inferred from source]** | re-add cells | swap islands | live rebuild from obstacles |
| Local avoidance | RVO2 (2D/3D) | RVO (closed) | RVO or DetourCrowd | none | none | potential only | ORCA (`dodgy_2d`) | none |
| Async | bake on thread; async map sync (4.4+) | bake jobs | tiled async | async findPath | — | — | — | async build with timeout |
| Debug overlay | yes | yes | yes | debugger | painter ext. | `mp_grid_draw` | yes | gizmos |

---

## 9. What is truly state of the art vs. legacy

**State of the art (2026) for a 2D top-down game**
- Exact-geometry navmesh from polygon booleans + robust offset + constrained triangulation/convex partition
  (Godot 4.3+, `i_overlay` + `i_triangle`, `vleue_navigator`). Better than voxel rasterisation for 2D:
  exact edges, no voxel stair-stepping, cheap rebuilds.
- **Polyanya** for optimal any-angle queries without preprocessing (vs A*+funnel which is optimal only
  within the chosen corridor, not globally) ([paper](https://www.ijcai.org/proceedings/2017/0070.pdf)).
- **Path corridor** maintenance instead of periodic full replans ([dtPathCorridor](https://recastnav.com/classdtPathCorridor.html)).
- **ORCA** for local avoidance, *with* awareness of its corner failure mode and an automated scenario rig
  ([Sunshine-Hill](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter19_RVO_and_ORCA_How_They_Really_Work.pdf)).
- **Async/background map rebuilds with double-buffered map iterations** (Godot 4.4+).
- **Flow-field tiles** when hundreds+ agents share goals (RTS, hordes), JPS+/goal bounding for static
  uniform grids.

**Legacy / limited**
- Plain grid A* with post-hoc smoothing (Construct, GDevelop, GameMaker, RPG Maker): simple, but jagged paths,
  cell-size vs gap trade-off, no clearance per agent, cost of full regenerate.
- Waypoint graphs placed by hand (pre-navmesh era) **[general knowledge]**.
- Potential-field-only steering (`mp_potential_*`): local minima, no completeness
  ([GameMaker manual](https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Movement_And_Collisions/Motion_Planning/Motion_Planning.htm)).
- Original VO (oscillation) and RVO without the practical damping described by Sunshine-Hill.
- Voxel (Recast) baking for flat 2D: works (Unity+NavMeshPlus) but is a 3D tool forced onto a plane.

---

## 10. Decisions every engine had to make — and what they chose

| Decision | Options | Godot | Unity | Unreal | Simple 2D engines | Notes |
|---|---|---|---|---|---|---|
| Grid vs navmesh | grid / navmesh / both | **both** (navmesh server + AStarGrid2D) | navmesh | navmesh (+ZoneGraph lanes) | **grid** | grid wins for tile games & non-programmers; navmesh for free-form geometry |
| Bake vs runtime | editor bake / runtime rebuild / tiles | both; async bake; chunked regions | bake; runtime NavMeshSurface.BuildNavMesh; carving | Static / Dynamic / Modifiers-only / invokers | regenerate on demand | all expose *both*, with runtime as an explicit opt-in |
| Agent radius | per mesh / per agent | per mesh (1 map per size) | per agent type (1 bake each) | per agent config | per grid (border) | Godot users ask to move it to the agent ([#5977](https://github.com/godotengine/godot-proposals/issues/5977)); nobody ships true per-agent clearance on one mesh |
| Avoidance | global sim / per agent / none | global RVO sim, layer/mask per agent | global, priority per agent | per character (RVO) or crowd manager | none | always *ignores the navmesh* in Godot and UE-RVO |
| Repath policy | per frame / on event / corridor | target change + deviation threshold | corridor + auto repath on partial | corridor (Detour) | on request | per-frame repath is a documented anti-pattern |
| Path smoothing | funnel / LOS skipping / none / simplify | funnel (default), edge-centred, simplify epsilon | string pulling | string pulling | LOS/gap smoothing | any-angle optimal (Polyanya) makes smoothing unnecessary |
| Who moves the body | navigation moves it / returns a velocity | returns next position + safe velocity | agent moves transform by default (can disable) | path following component drives movement | behavior moves object | landmass deliberately returns *desired velocity* and never moves the agent ([README](https://github.com/andriyDev/landmass/blob/main/crates/landmass/README.md)) — fits a physics engine |
| Links traversal | engine animates / game code | game code | auto-traverse (simple) or manual | Smart Link events | n/a | links = graph edges; motion is gameplay |

---

## 11. Failure modes users actually report

1. **Corner clipping / leaving the mesh** when avoidance pushes agents (Godot [#60354](https://github.com/godotengine/godot/issues/60354); UE RVO "can move out of bounds" [docs](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-avoidance-with-the-navigation-system-in-unreal-engine)).
2. **Radius confusion**: avoidance radius ≠ bake clearance ([Godot class ref](https://docs.godotengine.org/en/stable/classes/class_navigationagent2d.html)).
3. **First-frame empty path / sync latency** ([agents tutorial](https://docs.godotengine.org/en/latest/tutorials/navigation/navigation_using_navigationagents.html), [#104283](https://github.com/godotengine/godot/issues/104283)).
4. **Overshoot / backtracking** when per-frame step > waypoint acceptance radius (same tutorial).
5. **Jitter ("dancing")** from repathing every frame (same tutorial; GDevelop "run Move once" warning).
6. **Symmetric deadlocks / ORCA stuck at corners** ([Sunshine-Hill](https://www.gameaipro.com/GameAIPro3/GameAIPro3_Chapter19_RVO_and_ORCA_How_They_Really_Work.pdf); Godot tutorial on symmetrical cases).
7. **Agent + obstacle on the same object** (Unity [docs](https://docs.unity3d.com/2020.1/Documentation/Manual/nav-MixingComponents.html)).
8. **Gaps closed by coarse cells** (Construct, GameMaker manuals) / **obstacle uses bounding box not mask** ([GDevelop #2689](https://github.com/4ian/GDevelop/issues/2689)).
9. **Short search limits** silently stop agents in front of walls (RPG Maker `searchLimit = 12`).
10. **Slow at corners when physics velocity isn't fed back** ([landmass FAQ](https://github.com/andriyDev/landmass/blob/main/FAQ.md)).
11. **Runtime rebake cost dominated by source-geometry parsing on main thread** (Godot docs).
12. **Target off-mesh / agent off-mesh** → no path, no movement, no message (landmass FAQ; Godot empty paths).

---

## 12. Licences — what can be ported into an MIT/Apache Rust project

| Code | Licence | Portable? |
|---|---|---|
| Godot engine (navigation server, agent logic, funnel) | MIT ([Godot](https://raw.githubusercontent.com/godotengine/godot/master/COPYRIGHT.txt)) | **yes, with attribution** |
| RVO2 / RVO2-3D (UNC) | Apache-2.0 ([snape/RVO2](https://github.com/snape/RVO2)) | yes (Apache notice/NOTICE requirements) |
| Recast/Detour (incl. DetourCrowd, dtPathCorridor, SSFA in Detour) | Zlib ([repo](https://github.com/recastnavigation/recastnavigation)) | yes (Zlib: keep notice, mark altered) |
| Clipper2 | BSL-1.0 | yes (permissive; keep licence text) |
| polypartition | MIT/Expat | yes |
| GDevelop (Core/GDJS/Extensions/IDE) | MIT ([LICENSE](https://github.com/4ian/GDevelop/blob/master/LICENSE.md)) | yes |
| mikewesthad/navmesh, easystar.js, Jumper, NavMeshPlus, Surfacer | MIT | yes |
| Tuts+ platformer A* | BSD-2-Clause | yes |
| polyanya, vleue_navigator, landmass, rerecast, dodgy_2d, pathfinding, i_overlay, i_triangle, spade | MIT OR Apache | **directly usable as deps** |
| earcutr, geo-clipper | ISC | yes |
| Polyanya original C++ (Harabor) | **not verified** | treat as walled until checked; Rust port avoids the issue |
| JPS+ reference (Rabin repo) | no licence file | **walled** (algorithm from the paper is fine) |
| PlatformerPathfinding2D (Candescence) | no licence | walled |
| Unity navigation runtime, Construct 3, GameMaker, RPG Maker, Unreal (EULA, source-available) | proprietary | **walled** — run as oracles only; do not read/port |
| Mononen funnel blog snippet | no explicit licence | use Detour's Zlib implementation instead |

Algorithms in academic papers (A*, JPS, Theta*, Polyanya, HPA*, ORCA, flow fields, continuum crowds) are not
copyrightable as ideas; implementing from the papers is clean.

---

## 13. Notes specific to PH2D (inferences, not research findings)

- PH2D already has polygon geometry (kurbo) and rapier2d colliders: the *source geometry* side of a 2D
  bake (Godot's step 1) is native; `i_overlay` (already in the `geo` world) gives offset by agent radius,
  `i_triangle` gives convex decomposition → a polygon navmesh equivalent to Godot's.
- `polyanya` (no Bevy dependency) for queries; `landmass` core (no Bevy dependency, ORCA via `dodgy_2d`,
  returns desired velocity — never moves the agent) matches a "physics owns the pose" engine. Both use
  `glam` (0.32 / 0.30 respectively) — check version alignment with rapier2d 0.35's `glam`.
- Godot's NavigationAgent2D knob set (path/target desired distance, path_max_distance repath, layers,
  avoidance radius/neighbours/time horizons/priority, debug) is the most complete *designer-facing*
  reference; the simple engines show the minimal set non-programmers accept (cell/border, diagonals,
  smoothing, speed/accel/rotation, "regenerate region").
- Highest-value guardrails from the pain-point list: avoidance that cannot leave the walkable area
  (clamp to mesh, which Godot does not do), clear distinction between bake clearance and body radius,
  no per-frame repath, waypoint acceptance tied to speed×dt, explicit "no path / off mesh / stuck" states.
