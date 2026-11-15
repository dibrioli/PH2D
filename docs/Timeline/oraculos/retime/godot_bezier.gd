extends SceneTree
# Oráculo: Godot Bezier track — alça (influência a, velocidade s); a curva vai de (0,0) a (1,1).
func _init():
	var G = [[1.0/3, 1.0, 1.0/3, 1.0], [0.33, 3.0, 1.0/3, 1.0], [0.5, 3.0, 1.0/3, 1.0], [0.9, 2.0, 1.0/3, 1.0],
		[0.8, 1.0, 0.8, 1.0], [1.5, 1.0, 1.0/3, 1.0], [1.5, 2.0, 1.0/3, 1.0], [0.5, -1.0, 1.0/3, 1.0]]
	var out = []
	for g in G:
		var a = Animation.new()
		a.length = 1.0
		var t = a.add_track(Animation.TYPE_BEZIER)
		a.track_set_path(t, ".:x")
		a.bezier_track_insert_key(t, 0.0, 0.0, Vector2(0, 0), Vector2(g[0], g[0] * g[1]))
		a.bezier_track_insert_key(t, 1.0, 1.0, Vector2(-g[2], -g[2] * g[3]), Vector2(0, 0))
		var ys = []
		for i in range(201):
			ys.append(a.bezier_track_interpolate(t, i / 200.0))
		var mono = true
		for i in range(1, 201):
			if ys[i] < ys[i - 1] - 1e-9:
				mono = false
		out.append({"pedido": g, "alca_lida": [a.bezier_track_get_key_out_handle(t, 0).x, a.bezier_track_get_key_out_handle(t, 0).y],
			"mapa": ys, "monotono": mono, "max": ys.max(), "min": ys.min(),
			"vel_ponta_saida": (ys[1] - ys[0]) * 200.0})
	var f = FileAccess.open(OS.get_cmdline_user_args()[0], FileAccess.WRITE)
	f.store_string(JSON.stringify({"app": "Godot " + Engine.get_version_info().string, "curvas": out}))
	f.close()
	quit()
