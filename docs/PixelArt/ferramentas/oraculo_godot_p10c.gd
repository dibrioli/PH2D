extends SceneTree
# P10c M3b e M2iso: o clique de volta, pixel a pixel, e o terreno em iso. Sem janela.
# Saida: linhas "M3B <json>" e "M2ISO <json>".

const FORMAS = {
	"iso": [TileSet.TILE_SHAPE_ISOMETRIC, TileSet.TILE_LAYOUT_DIAMOND_DOWN, TileSet.TILE_OFFSET_AXIS_HORIZONTAL],
	"hex": [TileSet.TILE_SHAPE_HEXAGON, TileSet.TILE_LAYOUT_STACKED, TileSet.TILE_OFFSET_AXIS_HORIZONTAL],
	"hex_flat": [TileSet.TILE_SHAPE_HEXAGON, TileSet.TILE_LAYOUT_STACKED, TileSet.TILE_OFFSET_AXIS_VERTICAL],
}
const TAMANHOS = [[32, 16], [16, 16], [16, 12], [15, 15], [33, 17]]

# Em diamond_down: (c+1,r) desce a direita; (c,r+1) desce a esquerda.
const LADOS = [TileSet.CELL_NEIGHBOR_TOP_RIGHT_SIDE, TileSet.CELL_NEIGHBOR_BOTTOM_RIGHT_SIDE, TileSet.CELL_NEIGHBOR_BOTTOM_LEFT_SIDE, TileSet.CELL_NEIGHBOR_TOP_LEFT_SIDE]
const CANTOS = [TileSet.CELL_NEIGHBOR_RIGHT_CORNER, TileSet.CELL_NEIGHBOR_BOTTOM_CORNER, TileSet.CELL_NEIGHBOR_LEFT_CORNER, TileSet.CELL_NEIGHBOR_TOP_CORNER]
const D4 = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]
const E4 = [Vector2i(1, -1), Vector2i(1, 1), Vector2i(-1, 1), Vector2i(-1, -1)]

func _m3b():
	for nome in FORMAS:
		for t in TAMANHOS:
			var ts = TileSet.new()
			ts.tile_size = Vector2i(t[0], t[1])
			ts.tile_shape = FORMAS[nome][0]
			ts.tile_layout = FORMAS[nome][1]
			ts.tile_offset_axis = FORMAS[nome][2]
			var l = TileMapLayer.new()
			l.tile_set = ts
			root.add_child(l)
			var centros = []
			for r in range(-3, 4):
				for c in range(-3, 4):
					var m = l.map_to_local(Vector2i(c, r))
					centros.append([c, r, m.x, m.y])
			# Cada linha de pixeis em corridas: [quantos, c, r].
			var x0 = -2 * t[0]
			var y0 = -2 * t[1]
			var linhas = []
			for y in range(y0, 2 * t[1]):
				var corridas = []
				for x in range(x0, 2 * t[0]):
					var p = l.local_to_map(Vector2(x + 0.5, y + 0.5))
					if corridas.size() > 0 and corridas[-1][1] == p.x and corridas[-1][2] == p.y:
						corridas[-1][0] += 1
					else:
						corridas.append([1, p.x, p.y])
				linhas.append(corridas)
			print("M3B ", JSON.stringify({"forma": nome, "tile": t, "x0": x0, "y0": y0, "centros": centros, "linhas": linhas}))
			l.free()

func _valid_masks() -> Array:
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

func _esperada(cells: Dictionary, p: Vector2i) -> Array:
	var s = 0
	var c = 0
	for i in range(4):
		if cells.has(p + D4[i]): s |= 1 << i
	for i in range(4):
		if (s & (1 << i)) and (s & (1 << ((i + 1) % 4))) and cells.has(p + E4[i]): c |= 1 << i
	return [s, c]

func _m2iso():
	var masks = _valid_masks()
	var ts = TileSet.new()
	ts.tile_size = Vector2i(32, 16)
	ts.tile_shape = TileSet.TILE_SHAPE_ISOMETRIC
	ts.tile_layout = TileSet.TILE_LAYOUT_DIAMOND_DOWN
	ts.add_terrain_set()
	ts.set_terrain_set_mode(0, TileSet.TERRAIN_MODE_MATCH_CORNERS_AND_SIDES)
	ts.add_terrain(0)
	var src = TileSetAtlasSource.new()
	src.texture = ImageTexture.create_from_image(Image.create(32 * masks.size(), 16, false, Image.FORMAT_RGBA8))
	src.texture_region_size = Vector2i(32, 16)
	ts.add_source(src, 0)
	for k in range(masks.size()):
		src.create_tile(Vector2i(k, 0))
		var td = src.get_tile_data(Vector2i(k, 0), 0)
		td.terrain_set = 0
		td.terrain = 0
		for i in range(4):
			if masks[k][0] & (1 << i): td.set_terrain_peering_bit(LADOS[i], 0)
			if masks[k][1] & (1 << i): td.set_terrain_peering_bit(CANTOS[i], 0)
	var offs = [Vector2i(0,-1), Vector2i(1,-1), Vector2i(1,0), Vector2i(1,1), Vector2i(0,1), Vector2i(-1,1), Vector2i(-1,0), Vector2i(-1,-1)]
	var difere = 0
	var amostras = []
	for n in range(256):
		var cells = {Vector2i(4, 4): true}
		for b in range(8):
			if n & (1 << b): cells[Vector2i(4, 4) + offs[b]] = true
		var l = TileMapLayer.new()
		l.tile_set = ts
		root.add_child(l)
		var lista: Array[Vector2i] = []
		for p in cells: lista.append(p)
		l.set_cells_terrain_connect(lista, 0, 0, false)
		var a = l.get_cell_atlas_coords(Vector2i(4, 4))
		var got = masks[a.x] if a.x >= 0 else [-1, -1]
		var want = _esperada(cells, Vector2i(4, 4))
		if got != want:
			difere += 1
			if amostras.size() < 8: amostras.append([n, want, got])
		l.free()
	print("M2ISO ", JSON.stringify({"vizinhancas": 256, "diferem": difere, "amostras": amostras, "mascaras": masks.size()}))

func _init():
	print("M0 ", JSON.stringify({"versao": Engine.get_version_info()["string"]}))
	_m3b()
	_m2iso()
	quit()
