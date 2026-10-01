# Variante NODE: NavigationRegion2D + 2 NavigationAgent2D (pathing + avoidance via velocity_computed)
extends SceneTree
var f := 0
var bodies := []
var agents := []
var goals := [Vector2(350, 150), Vector2(50, 150)]
func pv(v): return "(%.6f, %.6f)" % [v.x, v.y]
func _initialize():
	var geom := NavigationMeshSourceGeometryData2D.new()
	geom.add_traversable_outline(PackedVector2Array([Vector2(0,0), Vector2(400,0), Vector2(400,300), Vector2(0,300)]))
	geom.add_obstruction_outline(PackedVector2Array([Vector2(150,100), Vector2(250,100), Vector2(250,200), Vector2(150,200)]))
	var np := NavigationPolygon.new(); np.agent_radius = 10.0
	NavigationServer2D.bake_from_source_geometry_data(np, geom)
	var reg := NavigationRegion2D.new(); reg.navigation_polygon = np
	root.add_child(reg)
	for i in 2:
		var n := Node2D.new(); n.position = goals[1 - i]
		var ag := NavigationAgent2D.new()
		ag.avoidance_enabled = true; ag.radius = 10.0; ag.max_speed = 100.0
		ag.neighbor_distance = 200.0; ag.time_horizon_agents = 1.0
		n.add_child(ag); root.add_child(n)
		ag.velocity_computed.connect(_on_v.bind(i))
		bodies.append(n); agents.append(ag)
	print("# map rid valid=", root.world_2d.navigation_map.is_valid())
func _on_v(v: Vector2, i: int):
	if f <= 4 or f % 30 == 0:
		print("vc f=%d agent=%d pos=%s safe=%s" % [f, i, pv(bodies[i].position), pv(v)])
	bodies[i].position += v / 60.0
func _physics_process(_d):
	f += 1
	for i in 2:
		var ag: NavigationAgent2D = agents[i]
		if f == 1: ag.target_position = goals[i]
		var nxt := ag.get_next_path_position()
		if f <= 4 or f % 30 == 0:
			print("path f=%d agent=%d next=%s path_n=%d finished=%s" % [f, i, pv(nxt), ag.get_current_navigation_path().size(), ag.is_navigation_finished()])
		ag.velocity = (nxt - bodies[i].position).normalized() * 100.0
	if f == 10:
		var s := []
		for v in agents[0].get_current_navigation_path(): s.append(pv(v))
		print("agent0 path: ", " ".join(s))
	return f >= 150
