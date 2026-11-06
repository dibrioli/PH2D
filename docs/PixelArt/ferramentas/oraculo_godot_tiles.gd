extends SceneTree
# P10 M2 e M3: Godot corre sobre mapas NOSSOS. Saida: linhas "M2A|M2B|M2C|M3 <json>" no stdout.
const SIDES = [TileSet.CELL_NEIGHBOR_TOP_SIDE, TileSet.CELL_NEIGHBOR_RIGHT_SIDE, TileSet.CELL_NEIGHBOR_BOTTOM_SIDE, TileSet.CELL_NEIGHBOR_LEFT_SIDE]
const CORNERS = [TileSet.CELL_NEIGHBOR_TOP_RIGHT_CORNER, TileSet.CELL_NEIGHBOR_BOTTOM_RIGHT_CORNER, TileSet.CELL_NEIGHBOR_BOTTOM_LEFT_CORNER, TileSet.CELL_NEIGHBOR_TOP_LEFT_CORNER]
const D4 = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]
const E4 = [Vector2i(1, -1), Vector2i(1, 1), Vector2i(-1, 1), Vector2i(-1, -1)]
var seed_ := 0x2545F491

func rnd() -> int:
	seed_ = (seed_ ^ ((seed_ << 13) & 0xFFFFFFFF)) & 0xFFFFFFFF
	seed_ = seed_ ^ (seed_ >> 17)
	seed_ = (seed_ ^ ((seed_ << 5) & 0xFFFFFFFF)) & 0xFFFFFFFF
	return seed_

func valid_masks() -> Array:
	var out = []
	for s in range(16):
		for c in range(16):
			var ok = true
			for i in range(4):
				if c & (1 << i) and not ((s & (1 << i)) and (s & (1 << ((i + 1) % 4)))):
					ok = false
			if ok:
				out.append([s, c])
	return out

func expected(cells: Dictionary, p: Vector2i) -> Array:
	var s = 0
	var c = 0
	for i in range(4):
		if cells.has(p + D4[i]): s |= 1 << i
	for i in range(4):
		if (s & (1 << i)) and (s & (1 << ((i + 1) % 4))) and cells.has(p + E4[i]): c |= 1 << i
	return [s, c]

# tileset com as mascaras de `keep` (indices em masks); devolve [ts, mapa atlas.x -> mascara]
func make_ts(masks: Array, keep: Array) -> TileSet:
	var ts = TileSet.new()
	ts.tile_size = Vector2i(16, 16)
	ts.add_terrain_set()
	ts.set_terrain_set_mode(0, TileSet.TERRAIN_MODE_MATCH_CORNERS_AND_SIDES)
	ts.add_terrain(0)
	var src = TileSetAtlasSource.new()
	src.texture = ImageTexture.create_from_image(Image.create(16 * masks.size(), 16, false, Image.FORMAT_RGBA8))
	src.texture_region_size = Vector2i(16, 16)
	ts.add_source(src, 0)
	for k in keep:
		src.create_tile(Vector2i(k, 0))
		var td = src.get_tile_data(Vector2i(k, 0), 0)
		td.terrain_set = 0
		td.terrain = 0
		for i in range(4):
			if masks[k][0] & (1 << i): td.set_terrain_peering_bit(SIDES[i], 0)
			if masks[k][1] & (1 << i): td.set_terrain_peering_bit(CORNERS[i], 0)
	return ts

func paint(ts: TileSet, cells: Dictionary) -> TileMapLayer:
	var layer = TileMapLayer.new()
	layer.tile_set = ts
	root.add_child(layer)
	var list: Array[Vector2i] = []
	for p in cells: list.append(p)
	layer.set_cells_terrain_connect(list, 0, 0, false)
	return layer

func got_of(layer: TileMapLayer, masks: Array, p: Vector2i) -> Array:
	var a = layer.get_cell_atlas_coords(p)
	return masks[a.x] if a.x >= 0 else [-1, -1]

func _init():
	var masks = valid_masks()
	var all: Array = range(masks.size())
	print("M0 ", JSON.stringify({"versao": Engine.get_version_info()["string"], "mascaras_validas": masks.size()}))
	# ---- M2A: as 256 vizinhancas do centro (4,4) num mapa 9x9, resto vazio (sem borda)
	var ts_all = make_ts(masks, all)
	var offs = [Vector2i(0,-1), Vector2i(1,-1), Vector2i(1,0), Vector2i(1,1), Vector2i(0,1), Vector2i(-1,1), Vector2i(-1,0), Vector2i(-1,-1)]
	var bad = 0
	var rows = []
	for n in range(256):
		var cells = {Vector2i(4, 4): true}
		for b in range(8):
			if n & (1 << b): cells[Vector2i(4, 4) + offs[b]] = true
		var layer = paint(ts_all, cells)
		var g = got_of(layer, masks, Vector2i(4, 4))
		var w = expected(cells, Vector2i(4, 4))
		if g != w: bad += 1
		rows.append([n, g, w])
		layer.queue_free()
	print("M2A ", JSON.stringify({"vizinhancas": 256, "diferem": bad, "bits": "n: bit b = vizinho offs[b] (N,NE,E,SE,S,SW,W,NW); mascara = [lados TRBL, cantos TR,BR,BL,TL]", "linhas": rows}))
	# ---- M2B: mapa com buracos e corredores de 1 celula, 24x20 + encostado nas bordas (x=0,y=0) e coordenadas negativas
	var cells = {}
	for y in range(-3, 20):
		for x in range(-3, 24):
			var v = rnd() % 100
			var keep = v < 62
			# corredores de 1 celula: linha y=7 e coluna x=11, a paredes de 0 ao lado
			if y == 7 or x == 11: keep = true
			elif y == 6 or y == 8 or x == 10 or x == 12: keep = (rnd() % 100) < 12
			if keep: cells[Vector2i(x, y)] = true
	var layer = paint(ts_all, cells)
	var bad2 = 0
	var rowsb = []
	for p in cells:
		var g = got_of(layer, masks, p)
		var w = expected(cells, p)
		if g != w: bad2 += 1
		rowsb.append([p.x, p.y, g, w])
	var hist = {}
	for r in rowsb: hist[str(r[3])] = hist.get(str(r[3]), 0) + 1
	print("M2B ", JSON.stringify({"celulas": cells.size(), "diferem": bad2, "mascaras_distintas_exercitadas": hist.size(), "linhas": rowsb}))
	layer.queue_free()
	# ---- M2C: faltam tiles no conjunto. Mesmo mapa; tres cortes
	var cortes = {}
	var sem_cantos = []
	for k in all: if masks[k][1] == 0: sem_cantos.append(k)
	cortes["so_lados_16"] = sem_cantos
	var sem_isolado = []
	for k in all: if masks[k][0] != 0 or masks[k][1] != 0: sem_isolado.append(k)
	cortes["sem_o_isolado"] = sem_isolado
	var r5 = all.duplicate()
	for i in range(5):
		r5.remove_at(rnd() % r5.size())
	cortes["menos_5_sorteados"] = r5
	var sem_cheio = []
	for k in all: if not (masks[k][0] == 15 and masks[k][1] == 15): sem_cheio.append(k)
	cortes["sem_o_miolo_15_15"] = sem_cheio
	for nome in cortes:
		var keep: Array = cortes[nome]
		var ts = make_ts(masks, keep)
		var lay = paint(ts, cells)
		var presentes = {}
		for k in keep: presentes[str(masks[k])] = true
		var exato = 0; var falta = 0; var vazio = 0; var troca = []
		var sub = 0; var dist = {}
		for p in cells:
			var w = expected(cells, p)
			var a = lay.get_cell_atlas_coords(p)
			var g = masks[a.x] if a.x >= 0 else [-1, -1]
			if presentes.has(str(w)):
				if g == w: exato += 1
				else: troca.append([p.x, p.y, g, w])
			else:
				falta += 1
				if a.x < 0: vazio += 1
				else:
					# o escolhido e um subconjunto da mascara pedida?
					if (g[0] & ~w[0]) == 0 and (g[1] & ~w[1]) == 0: sub += 1
					var d = 0
					for i in range(4):
						d += (1 if ((g[0] ^ w[0]) >> i) & 1 else 0) + (1 if ((g[1] ^ w[1]) >> i) & 1 else 0)
					dist[str(d)] = dist.get(str(d), 0) + 1
		var amostra = []
		for p in cells:
			var w = expected(cells, p)
			if not presentes.has(str(w)) and amostra.size() < 25:
				var a = lay.get_cell_atlas_coords(p)
				amostra.append([p.x, p.y, w, masks[a.x] if a.x >= 0 else [-1, -1]])
		print("M2C ", JSON.stringify({"corte": nome, "tiles_no_conjunto": keep.size(), "celulas": cells.size(), "mascara_pedida_existe_e_bateu": exato, "mascara_pedida_existe_mas_outra": troca.size(), "outras_amostra": troca.slice(0, 10), "mascara_pedida_NAO_existe": falta, "dessas_sem_tile": vazio, "dessas_escolhido_e_subconjunto": sub, "dessas_distancia_de_bits": dist, "amostra_pedida_x_escolhida": amostra}))
		lay.queue_free()
	# ---- M3: centro e volta
	var shapes = {"square": TileSet.TILE_SHAPE_SQUARE, "iso": TileSet.TILE_SHAPE_ISOMETRIC, "half_offset": TileSet.TILE_SHAPE_HALF_OFFSET_SQUARE, "hex": TileSet.TILE_SHAPE_HEXAGON}
	var layouts = {"stacked": TileSet.TILE_LAYOUT_STACKED, "stacked_offset": TileSet.TILE_LAYOUT_STACKED_OFFSET, "stairs_right": TileSet.TILE_LAYOUT_STAIRS_RIGHT, "stairs_down": TileSet.TILE_LAYOUT_STAIRS_DOWN, "diamond_right": TileSet.TILE_LAYOUT_DIAMOND_RIGHT, "diamond_down": TileSet.TILE_LAYOUT_DIAMOND_DOWN}
	var axes = {"horizontal": TileSet.TILE_OFFSET_AXIS_HORIZONTAL, "vertical": TileSet.TILE_OFFSET_AXIS_VERTICAL}
	var sizes = [Vector2i(16, 16), Vector2i(15, 15), Vector2i(32, 16), Vector2i(33, 17)]
	for sn in shapes:
		for ln in layouts:
			for an in axes:
				for sz in sizes:
					var t = TileSet.new()
					t.tile_size = sz
					t.tile_shape = shapes[sn]
					t.tile_layout = layouts[ln]
					t.tile_offset_axis = axes[an]
					var l = TileMapLayer.new()
					l.tile_set = t
					root.add_child(l)
					var rr = []
					var volta_ok = 0
					var n = 0
					for y in range(-2, 3):
						for x in range(-2, 3):
							var p = Vector2i(x, y)
							var c = l.map_to_local(p)
							var b = l.local_to_map(c)
							# 4 pontos a 1/4 do tile do centro: ainda na mesma celula?
							var js = []
							for d in [Vector2(-0.25, 0), Vector2(0.25, 0), Vector2(0, -0.25), Vector2(0, 0.25)]:
								js.append(l.local_to_map(c + d * Vector2(sz)))
							n += 1
							if b == p: volta_ok += 1
							rr.append([x, y, c.x, c.y, b.x, b.y, js.map(func(q): return [q.x, q.y])])
					print("M3 ", JSON.stringify({"forma": sn, "layout": ln, "eixo": an, "tile": [sz.x, sz.y], "celulas": n, "volta_ok": volta_ok, "linhas": rr}))
					l.queue_free()
	quit()
