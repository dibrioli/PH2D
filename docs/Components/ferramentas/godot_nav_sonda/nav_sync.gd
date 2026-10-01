# Variante: async iterations OFF + map_force_update -> path no MESMO quadro?
extends SceneTree
var map: RID
var reg: RID
var f := 0
func pp(t, p):
	var s := []
	for v in p: s.append("(%.9f, %.9f)" % [v.x, v.y])
	print("%s n=%d %s" % [t, p.size(), " ".join(s)])
func _initialize():
	var geom := NavigationMeshSourceGeometryData2D.new()
	geom.add_traversable_outline(PackedVector2Array([Vector2(0,0), Vector2(400,0), Vector2(400,300), Vector2(0,300)]))
	geom.add_obstruction_outline(PackedVector2Array([Vector2(150,100), Vector2(250,100), Vector2(250,200), Vector2(150,200)]))
	var np := NavigationPolygon.new()
	np.agent_radius = 10.0
	NavigationServer2D.bake_from_source_geometry_data(np, geom)
	map = NavigationServer2D.map_create()
	NavigationServer2D.map_set_use_async_iterations(map, false)
	NavigationServer2D.map_set_active(map, true)
	reg = NavigationServer2D.region_create()
	NavigationServer2D.region_set_map(reg, map)
	NavigationServer2D.region_set_use_async_iterations(reg, false)
	NavigationServer2D.region_set_navigation_polygon(reg, np)
	print("# async=", NavigationServer2D.map_get_use_async_iterations(map), " iter0=", NavigationServer2D.map_get_iteration_id(map))
	NavigationServer2D.map_set_use_async_iterations(map, false)
	print("# async after active+region=", NavigationServer2D.map_get_use_async_iterations(map))
	NavigationServer2D.map_force_update(map)
	print("# iter after force#1=", NavigationServer2D.map_get_iteration_id(map), " regions=", NavigationServer2D.map_get_regions(map).size())
	pp("after force#1", NavigationServer2D.map_get_path(map, Vector2(50,150), Vector2(350,150), true))
	NavigationServer2D.map_force_update(map)
	print("# iter after force=", NavigationServer2D.map_get_iteration_id(map))
	pp("init after force", NavigationServer2D.map_get_path(map, Vector2(50,150), Vector2(350,150), true))
	# off-mesh endpoints: start inside the hole, target outside the outer rect
	pp("init off-mesh", NavigationServer2D.map_get_path(map, Vector2(200,150), Vector2(395,295), true))
	# raw corridor (optimize=false) vs funnel
	pp("init raw", NavigationServer2D.map_get_path(map, Vector2(50,150), Vector2(350,150), false))
	# tie broken off-axis: target slightly below
	pp("init y=151", NavigationServer2D.map_get_path(map, Vector2(50,150), Vector2(350,151), true))
	pp("init y=149", NavigationServer2D.map_get_path(map, Vector2(50,150), Vector2(350,149), true))
	var q := NavigationPathQueryParameters2D.new()
	q.map = map; q.start_position = Vector2(50,150); q.target_position = Vector2(350,150)
	q.metadata_flags = NavigationPathQueryParameters2D.PATH_METADATA_INCLUDE_ALL
	var r := NavigationPathQueryResult2D.new()
	NavigationServer2D.query_path(q, r)
	pp("query_path", r.get_path())
	print("query_path length=%.9f types=%s" % [r.get_path_length(), str(r.get_path_types())])
	# async bake: does callback fire, on which frame?
	var np2 := NavigationPolygon.new(); np2.agent_radius = 10.0
	NavigationServer2D.bake_from_source_geometry_data_async(np2, geom, func(): print("# async bake done f=", f, " verts=", np2.get_vertices().size()))
	print("# async bake queued; is_baking=", NavigationServer2D.is_baking_navigation_polygon(np2), " verts now=", np2.get_vertices().size())
func _physics_process(_d):
	f += 1
	if f <= 4:
		print("# f%d iter=%d async=%s" % [f, NavigationServer2D.map_get_iteration_id(map), NavigationServer2D.map_get_use_async_iterations(map)])
		pp("f%d before force" % f, NavigationServer2D.map_get_path(map, Vector2(50,150), Vector2(350,150), true))
		NavigationServer2D.map_force_update(map)
		pp("f%d after force (iter %d)" % [f, NavigationServer2D.map_get_iteration_id(map)], NavigationServer2D.map_get_path(map, Vector2(50,150), Vector2(350,150), true))
	if f >= 20:
		NavigationServer2D.free_rid(reg); NavigationServer2D.free_rid(map)
		return true
	return false
