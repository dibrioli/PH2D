# O CORPUS DO ORÁCULO DA NAVEGAÇÃO (plano 30 §8.2, famílias F1 + F2).
#
# Corre-se sem interface, sobre entradas NOSSAS (§0.9 do roteador — a saída é livre, o fonte não se lê):
#
#   godot --headless --script docs/Components/ferramentas/godot_nav_oraculo/corpus.gd > saida.txt
#
# Para cada cena × raio: assa a NavigationPolygon (bake_from_source_geometry_data), imprime a área e os
# vértices assados, e o caminho (map_get_path, optimize=true) de uma lista fixa de pares.
#
# ⚠️ A ARMADILHA da pesquisa (doc 29 §6.1): com as iterações assíncronas de fábrica o mapa devolve
# caminhos VAZIOS sem erro até ao quadro 4–5. ⇒ `region_set_use_async_iterations(reg, false)` ANTES
# do polígono, `map_set_use_async_iterations(map, false)`, e `map_force_update` antes de perguntar.
extends SceneTree

const RAIOS := [0.0, 5.0, 10.0, 25.0]

func caixa(x0, y0, x1, y1) -> PackedVector2Array:
	return PackedVector2Array([Vector2(x0, y0), Vector2(x1, y0), Vector2(x1, y1), Vector2(x0, y1)])

func rodado(cx, cy, hx, hy, c, s) -> PackedVector2Array:
	var out := PackedVector2Array()
	for p in [Vector2(-hx, -hy), Vector2(hx, -hy), Vector2(hx, hy), Vector2(-hx, hy)]:
		out.append(Vector2(cx + p.x * c - p.y * s, cy + p.x * s + p.y * c))
	return out

func cenas() -> Array:
	return [
		{"nome": "quadrado", "regiao": caixa(0, 0, 400, 300), "obs": [caixa(150, 100, 250, 200)],
		 "pares": [[50, 150, 350, 150], [50, 150, 350, 160], [200, 20, 200, 280], [20, 20, 380, 280], [300, 250, 100, 60]]},
		{"nome": "dois_que_se_tocam", "regiao": caixa(0, 0, 400, 300), "obs": [caixa(100, 100, 200, 200), caixa(200, 150, 300, 250)],
		 "pares": [[50, 150, 350, 150], [150, 50, 250, 280], [30, 280, 380, 20], [350, 100, 60, 260]]},
		{"nome": "L", "regiao": caixa(0, 0, 400, 300), "obs": [caixa(100, 80, 140, 240), caixa(100, 200, 300, 240)],
		 "pares": [[200, 150, 50, 270], [200, 150, 380, 280], [60, 40, 250, 270], [380, 20, 60, 290]]},
		{"nome": "na_borda", "regiao": caixa(0, 0, 400, 300), "obs": [caixa(180, 0, 220, 200)],
		 "pares": [[50, 50, 350, 50], [50, 250, 350, 250], [100, 20, 300, 20]]},
		{"nome": "sobrepostos", "regiao": caixa(0, 0, 400, 300), "obs": [caixa(100, 100, 220, 180), caixa(180, 140, 300, 220), caixa(160, 60, 200, 260)],
		 "pares": [[50, 150, 350, 150], [200, 20, 200, 290], [40, 40, 360, 260], [360, 40, 40, 260]]},
		{"nome": "passagem", "regiao": caixa(0, 0, 400, 300), "obs": [caixa(180, 0, 220, 135), caixa(180, 165, 220, 300)],
		 "pares": [[50, 150, 350, 150], [50, 20, 350, 280], [100, 280, 300, 20]]},
		{"nome": "rodado", "regiao": caixa(0, 0, 400, 300), "obs": [rodado(200, 150, 60, 30, 0.8, 0.6)],
		 "pares": [[50, 150, 350, 150], [200, 20, 200, 280], [80, 60, 320, 240], [320, 60, 80, 240]]},
	]

func pv(arr) -> String:
	var s := []
	for v in arr: s.append("%.9f,%.9f" % [v.x, v.y])
	return " ".join(s)

func _initialize():
	print("# PH2D corpus do oraculo de navegacao — Godot ", Engine.get_version_info().string)
	print("# fonte: docs/Components/ferramentas/godot_nav_oraculo/corpus.gd · NavigationPolygon.bake + map_get_path(optimize=true)")
	print("# formato: CENA <nome> R <raio> | REG <x,y ...> | OBS <x,y ...>* | AREA <a> | POLIS <n> | VERT <x,y ...> | PATH <x0 y0 x1 y1> LEN <l> PTS <x,y ...>")
	for c in cenas():
		for r in RAIOS:
			var geom := NavigationMeshSourceGeometryData2D.new()
			geom.add_traversable_outline(c["regiao"])
			for o in c["obs"]:
				geom.add_obstruction_outline(o)
			var np := NavigationPolygon.new()
			np.agent_radius = r
			NavigationServer2D.bake_from_source_geometry_data(np, geom)
			var verts := np.get_vertices()
			var area := 0.0
			for i in np.get_polygon_count():
				var poly := np.get_polygon(i)
				var a := 0.0
				for j in poly.size():
					var p0 := verts[poly[j]]
					var p1 := verts[poly[(j + 1) % poly.size()]]
					a += p0.x * p1.y - p1.x * p0.y
				area += abs(a) * 0.5
			print("CENA %s R %.1f" % [c["nome"], r])
			print("REG " + pv(c["regiao"]))
			for o in c["obs"]:
				print("OBS " + pv(o))
			print("AREA %.9f" % area)
			print("POLIS %d" % np.get_polygon_count())
			print("VERT " + pv(verts))
			var map := NavigationServer2D.map_create()
			NavigationServer2D.map_set_use_async_iterations(map, false)
			NavigationServer2D.map_set_active(map, true)
			var reg := NavigationServer2D.region_create()
			NavigationServer2D.region_set_use_async_iterations(reg, false)
			NavigationServer2D.region_set_map(reg, map)
			NavigationServer2D.region_set_navigation_polygon(reg, np)
			NavigationServer2D.map_force_update(map)
			for q in c["pares"]:
				var a := Vector2(q[0], q[1])
				var b := Vector2(q[2], q[3])
				var path := NavigationServer2D.map_get_path(map, a, b, true)
				var l := 0.0
				for k in range(1, path.size()):
					l += path[k - 1].distance_to(path[k])
				print("PATH %d %d %d %d LEN %.9f PTS %s" % [q[0], q[1], q[2], q[3], l, pv(path)])
			NavigationServer2D.free_rid(reg)
			NavigationServer2D.free_rid(map)
	print("# fim")
	quit()
