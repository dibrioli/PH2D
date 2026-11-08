# Oráculo do Shape «Outline» (CLAUDE.md §0.9): o que a Godot faz em Sprite2D ▸ «Create CollisionPolygon2D
# Sibling» — BitMap.create_from_image_alpha + BitMap.opaque_to_polygons — e a decomposição convexa
# Geometry2D.decompose_polygon_in_convex. Corre-se, nunca se lê o fonte:
#
#     godot --headless -s docs/GameEngineFuncional/oraculo_contorno/contorno_godot.gd -- <raiz_do_repo> <saida.json>
#
# Entradas NOSSAS: a folha heroi_azul (8×4 células de 100 px, recortadas aqui) e as sintéticas de gera_sinteticas.py.
extends SceneTree

const LIMIARES = [0.1, 0.5]
const EPSILONS = [0.5, 2.0, 4.0]
const REPETE = 15

func r2(v: float) -> float:
	return snappedf(v, 0.01)

func pts(p: PackedVector2Array) -> Array:
	var out = []
	for v in p:
		out.append([r2(v.x), r2(v.y)])
	return out

func area(p: PackedVector2Array) -> float:
	var s = 0.0
	for i in p.size():
		var a = p[i]
		var b = p[(i + 1) % p.size()]
		s += a.x * b.y - b.x * a.y
	return s * 0.5

# União das alfas (OR = máximo) de várias células da folha.
func uniao(folha: Image, celulas: Array, w: int, h: int) -> Image:
	var u = Image.create(w, h, false, Image.FORMAT_RGBA8)
	u.fill(Color(0, 0, 0, 0))
	for c in celulas:
		var reg = folha.get_region(Rect2i(c[0] * w, c[1] * h, w, h))
		for y in h:
			for x in w:
				var a = reg.get_pixel(x, y).a
				if a > u.get_pixel(x, y).a:
					u.set_pixel(x, y, Color(1, 1, 1, a))
	return u

func mede(img: Image, limiar: float, eps: float) -> Dictionary:
	var rect = Rect2i(Vector2i.ZERO, img.get_size())
	var t_bm = 1e18
	var t_pol = 1e18
	var t_dec = 1e18
	var bm: BitMap
	var polys
	var pecas = []
	for k in REPETE:
		var t0 = Time.get_ticks_usec()
		bm = BitMap.new()
		bm.create_from_image_alpha(img, limiar)
		var t1 = Time.get_ticks_usec()
		polys = bm.opaque_to_polygons(rect, eps)
		var t2 = Time.get_ticks_usec()
		pecas = []
		for p in polys:
			pecas.append(Geometry2D.decompose_polygon_in_convex(p))
		var t3 = Time.get_ticks_usec()
		t_bm = min(t_bm, t1 - t0)
		t_pol = min(t_pol, t2 - t1)
		t_dec = min(t_dec, t3 - t2)
	var lista = []
	var total_v = 0
	var total_p = 0
	for i in polys.size():
		var p: PackedVector2Array = polys[i]
		total_v += p.size()
		var conv = []
		for q in pecas[i]:
			conv.append(pts(q))
		total_p += pecas[i].size()
		lista.append({"vertices": p.size(), "area_assinada": r2(area(p)), "pontos": pts(p),
			"pecas_convexas": pecas[i].size(), "pecas": conv})
	return {"limiar_alfa": limiar, "epsilon": eps, "pixeis_opacos": bm.get_true_bit_count(),
		"poligonos": polys.size(), "vertices_total": total_v, "pecas_convexas_total": total_p,
		"us_min": {"create_from_image_alpha": t_bm, "opaque_to_polygons": t_pol, "decompose_in_convex": t_dec},
		"lista": lista}

func _initialize():
	var args = OS.get_cmdline_user_args()
	var raiz = args[0]
	var saida = args[1]
	var dir = raiz.path_join("docs/GameEngineFuncional/oraculo_contorno/entradas")
	var folha_path = raiz.path_join("docs/GameEngineFuncional/arte/heroi_azul.png")
	var folha = Image.load_from_file(folha_path)
	folha.convert(Image.FORMAT_RGBA8)
	var casos = []
	casos.append(["heroi_f00_parado", folha.get_region(Rect2i(0, 0, 100, 100)), "heroi_azul.png célula (0,0)"])
	casos.append(["heroi_f11_corre", folha.get_region(Rect2i(300, 100, 100, 100)), "heroi_azul.png célula (3,1)"])
	casos.append(["heroi_f20_agachado", folha.get_region(Rect2i(400, 200, 100, 100)), "heroi_azul.png célula (4,2)"])
	var corrida = []
	for c in 8:
		corrida.append([c, 1])
	casos.append(["heroi_uniao_corrida_8", uniao(folha, corrida, 100, 100), "OR das alfas das 8 células da linha 1"])
	var todas = []
	for r in 4:
		for c in 8:
			todas.append([c, r])
	casos.append(["heroi_uniao_folha_32", uniao(folha, todas, 100, 100), "OR das alfas das 32 células"])
	for n in ["sint_L", "sint_anel", "sint_crescente", "sint_moeda_ilha"]:
		var im = Image.load_from_file(dir.path_join(n + ".png"))
		im.convert(Image.FORMAT_RGBA8)
		casos.append([n, im, "entradas/" + n + ".png"])
	var resultados = []
	for c in casos:
		var img: Image = c[1]
		var grade = []
		for l in LIMIARES:
			for e in EPSILONS:
				grade.append(mede(img, l, e))
		resultados.append({"caso": c[0], "origem": c[2], "w": img.get_width(), "h": img.get_height(), "grade": grade})
		# PNG da máscara da união: para as sobreposições (não é fixture).
		if c[0].begins_with("heroi"):
			img.save_png(saida.get_base_dir().path_join("_alfa_" + c[0] + ".png"))
	var sha = {"arte/heroi_azul.png": FileAccess.get_sha256(folha_path)}
	for n in ["sint_L", "sint_anel", "sint_crescente", "sint_moeda_ilha"]:
		sha["oraculo_contorno/entradas/" + n + ".png"] = FileAccess.get_sha256(dir.path_join(n + ".png"))
	var v = Engine.get_version_info()
	var doc = {"cabecalho": {
		"oraculo": "Godot " + v.string, "licenca": "MIT (pacman -Qi godot)",
		"comando": "godot --headless -s docs/GameEngineFuncional/oraculo_contorno/contorno_godot.gd -- <raiz> <saida.json>",
		"apis": ["BitMap.create_from_image_alpha(image, threshold)", "BitMap.opaque_to_polygons(rect, epsilon)", "Geometry2D.decompose_polygon_in_convex(polygon)"],
		"entradas_sha256": sha, "repeticoes_tempo": REPETE, "tempo": "microssegundos, o MÍNIMO das repetições",
		"data": Time.get_datetime_string_from_system(true) + "Z"},
		"casos": resultados}
	var f = FileAccess.open(saida, FileAccess.WRITE)
	f.store_string(JSON.stringify(doc, "", false))
	f.close()
	print("ok ", resultados.size(), " casos -> ", saida)
	quit()
