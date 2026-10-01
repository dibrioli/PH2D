# Sonda de navegacao 2D — Godot 4.7.2 (MIT), headless. Scratch, fora da arvore.
# godot --headless --fixed-fps 60 --quit-after 200 --script nav_probe.gd
extends SceneTree

const A := Vector2(50, 150)
const B := Vector2(350, 150)
var map_baked: RID
var map_manual: RID
var reg_baked: RID
var reg_manual: RID
var frame := 0
var agents := []
var targets := []
var cb_count := [0, 0]


func pv(v: Vector2) -> String:
	return "(%.9f, %.9f)" % [v.x, v.y]


func ppath(tag: String, p: PackedVector2Array) -> void:
	var s := []
	for v in p:
		s.append(pv(v))
	print("%s n=%d  %s" % [tag, p.size(), " ".join(s)])


func _initialize():
	print("# godot ", Engine.get_version_info().string)
	print("# physics_ticks_per_second=", Engine.physics_ticks_per_second)
	# ---------- (a1) BAKE from source geometry with agent radius
	var geom := NavigationMeshSourceGeometryData2D.new()
	geom.add_traversable_outline(PackedVector2Array([Vector2(0, 0), Vector2(400, 0), Vector2(400, 300), Vector2(0, 300)]))
	geom.add_obstruction_outline(PackedVector2Array([Vector2(150, 100), Vector2(250, 100), Vector2(250, 200), Vector2(150, 200)]))
	var np_baked := NavigationPolygon.new()
	np_baked.agent_radius = 10.0
	print("# baked: cell_size=%.4f agent_radius=%.4f border=%.4f partition=%d" % [np_baked.cell_size, np_baked.agent_radius, np_baked.border_size, np_baked.sample_partition_type])
	print("# is_baking before=", NavigationServer2D.is_baking_navigation_polygon(np_baked))
	NavigationServer2D.bake_from_source_geometry_data(np_baked, geom)
	print("# baked verts=%d polys=%d outlines=%d" % [np_baked.get_vertices().size(), np_baked.get_polygon_count(), np_baked.get_outline_count()])
	ppath("# baked vertices", np_baked.get_vertices())
	for i in np_baked.get_polygon_count():
		print("#   poly %d = %s" % [i, str(np_baked.get_polygon(i))])
	# ---------- (a2) MANUAL outline + hole, make_polygons_from_outlines
	var np_man := NavigationPolygon.new()
	np_man.add_outline(PackedVector2Array([Vector2(0, 0), Vector2(400, 0), Vector2(400, 300), Vector2(0, 300)]))
	np_man.add_outline(PackedVector2Array([Vector2(150, 100), Vector2(250, 100), Vector2(250, 200), Vector2(150, 200)]))
	np_man.make_polygons_from_outlines()
	print("# manual verts=%d polys=%d" % [np_man.get_vertices().size(), np_man.get_polygon_count()])
	# ---------- (b) maps
	map_baked = NavigationServer2D.map_create()
	NavigationServer2D.map_set_cell_size(map_baked, np_baked.cell_size)
	NavigationServer2D.map_set_active(map_baked, true)
	reg_baked = NavigationServer2D.region_create()
	NavigationServer2D.region_set_map(reg_baked, map_baked)
	NavigationServer2D.region_set_navigation_polygon(reg_baked, np_baked)
	map_manual = NavigationServer2D.map_create()
	NavigationServer2D.map_set_cell_size(map_manual, np_man.cell_size)
	NavigationServer2D.map_set_active(map_manual, true)
	reg_manual = NavigationServer2D.region_create()
	NavigationServer2D.region_set_map(reg_manual, map_manual)
	NavigationServer2D.region_set_navigation_polygon(reg_manual, np_man)
	print("# async_iterations(map)=", NavigationServer2D.map_get_use_async_iterations(map_baked))
	print("# iter before force=", NavigationServer2D.map_get_iteration_id(map_baked))
	ppath("init(no sync) baked", NavigationServer2D.map_get_path(map_baked, A, B, true))
	NavigationServer2D.map_force_update(map_baked)
	NavigationServer2D.map_force_update(map_manual)
	print("# iter after force=", NavigationServer2D.map_get_iteration_id(map_baked))
	ppath("init(after force) baked", NavigationServer2D.map_get_path(map_baked, A, B, true))
	# ---------- (d) AStarGrid2D
	astar_probe()
	# ---------- (e) avoidance agents (server API)
	for i in 2:
		var ag := NavigationServer2D.agent_create()
		NavigationServer2D.agent_set_map(ag, map_baked)
		NavigationServer2D.agent_set_avoidance_enabled(ag, true)
		NavigationServer2D.agent_set_radius(ag, 10.0)
		NavigationServer2D.agent_set_max_speed(ag, 100.0)
		NavigationServer2D.agent_set_neighbor_distance(ag, 200.0)
		NavigationServer2D.agent_set_max_neighbors(ag, 10)
		NavigationServer2D.agent_set_time_horizon_agents(ag, 1.0)
		NavigationServer2D.agent_set_time_horizon_obstacles(ag, 0.0)
		NavigationServer2D.agent_set_avoidance_callback(ag, _on_safe.bind(i))
		agents.append(ag)
	NavigationServer2D.agent_set_position(agents[0], Vector2(50, 50))
	NavigationServer2D.agent_set_position(agents[1], Vector2(350, 50))
	targets = [Vector2(350, 50), Vector2(50, 50)]


func astar_probe():
	var modes := {"ALWAYS": AStarGrid2D.DIAGONAL_MODE_ALWAYS, "NEVER": AStarGrid2D.DIAGONAL_MODE_NEVER,
		"AT_LEAST_ONE_WALKABLE": AStarGrid2D.DIAGONAL_MODE_AT_LEAST_ONE_WALKABLE,
		"ONLY_IF_NO_OBSTACLES": AStarGrid2D.DIAGONAL_MODE_ONLY_IF_NO_OBSTACLES}
	for jump in [false, true]:
		for name in modes:
			var g := AStarGrid2D.new()
			g.region = Rect2i(0, 0, 8, 6)
			g.cell_size = Vector2(16, 16)
			g.diagonal_mode = modes[name]
			g.jumping_enabled = jump
			g.update()
			# wall at x=3, y=0..3 ; one diagonal corner blocker at (5,4)
			for y in range(0, 4):
				g.set_point_solid(Vector2i(3, y), true)
			g.set_point_solid(Vector2i(5, 4), true)
			var ids := g.get_id_path(Vector2i(0, 0), Vector2i(7, 0))
			var pts := g.get_point_path(Vector2i(0, 0), Vector2i(7, 0))
			print("astar diag=%s jump=%s ids=%s" % [name, jump, str(ids)])
			ppath("astar   points", pts)
	# heuristic + partial path to unreachable cell
	var g2 := AStarGrid2D.new()
	g2.region = Rect2i(0, 0, 5, 5)
	g2.update()
	for y in 5:
		g2.set_point_solid(Vector2i(2, y), true)
	print("astar unreachable ids=", g2.get_id_path(Vector2i(0, 2), Vector2i(4, 2)), " partial=", g2.get_id_path(Vector2i(0, 2), Vector2i(4, 2), true))
	print("astar default heuristics compute=%d estimate=%d" % [g2.default_compute_heuristic, g2.default_estimate_heuristic])


func _on_safe(v: Vector2, i: int):
	cb_count[i] += 1
	var p := NavigationServer2D.agent_get_position(agents[i])
	if frame <= 6 or frame % 15 == 0:
		print("avoid f=%d agent=%d pos=%s safe=%s |safe|=%.6f" % [frame, i, pv(p), pv(v), v.length()])
	NavigationServer2D.agent_set_position(agents[i], p + v / Engine.physics_ticks_per_second)


func _physics_process(_dt: float) -> bool:
	frame += 1
	if frame <= 3:
		print("# f=%d iter=%d" % [frame, NavigationServer2D.map_get_iteration_id(map_baked)])
		ppath("f%d baked" % frame, NavigationServer2D.map_get_path(map_baked, A, B, true))
		ppath("f%d baked(raw,noopt)" % frame, NavigationServer2D.map_get_path(map_baked, A, B, false))
		ppath("f%d manual" % frame, NavigationServer2D.map_get_path(map_manual, A, B, true))
	if frame in [5, 10, 30, 60, 120]:
		print("# f=%d iter=%d regions=%d bounds=%s closest=%s owner_ok=%s" % [frame, NavigationServer2D.map_get_iteration_id(map_baked), NavigationServer2D.map_get_regions(map_baked).size(), str(NavigationServer2D.region_get_bounds(reg_baked)), pv(NavigationServer2D.map_get_closest_point(map_baked, Vector2(200, 150))), NavigationServer2D.map_get_closest_point_owner(map_baked, Vector2(20,20)) == reg_baked])
		ppath("f%d baked" % frame, NavigationServer2D.map_get_path(map_baked, A, B, true))
		ppath("f%d manual" % frame, NavigationServer2D.map_get_path(map_manual, A, B, true))
	if frame == 3:
		var q := NavigationPathQueryParameters2D.new()
		q.map = map_baked
		q.start_position = A
		q.target_position = B
		q.metadata_flags = NavigationPathQueryParameters2D.PATH_METADATA_INCLUDE_ALL
		var r := NavigationPathQueryResult2D.new()
		NavigationServer2D.query_path(q, r)
		ppath("query_path", r.get_path())
		print("query_path len=%.9f types=%s" % [r.get_path_length(), str(r.get_path_types())])
		print("closest_point(200,150)=", pv(NavigationServer2D.map_get_closest_point(map_baked, Vector2(200, 150))))
	for i in agents.size():
		var p := NavigationServer2D.agent_get_position(agents[i])
		var d: Vector2 = targets[i] - p
		var v := d.normalized() * 100.0 if d.length() > 2.0 else Vector2.ZERO
		NavigationServer2D.agent_set_velocity(agents[i], v)
	if frame >= 180:
		print("# callbacks agent0=%d agent1=%d" % cb_count)
		return true
	return false
