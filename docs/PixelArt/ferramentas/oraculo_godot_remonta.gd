extends SceneTree
# P10c M8: o Godot remonta um mapa a partir do PNG + JSON nossos e fotografa cada quadro.
# Uso: godot --rendering-driver opengl3 --script este.gd -- entrada.json pasta_de_saida
# Precisa de janela (xvfb-run): em --headless nao ha renderizador.

const FORMAS = {
	"square": [TileSet.TILE_SHAPE_SQUARE, TileSet.TILE_LAYOUT_STACKED, TileSet.TILE_OFFSET_AXIS_HORIZONTAL],
	"iso": [TileSet.TILE_SHAPE_ISOMETRIC, TileSet.TILE_LAYOUT_DIAMOND_DOWN, TileSet.TILE_OFFSET_AXIS_HORIZONTAL],
	"hex": [TileSet.TILE_SHAPE_HEXAGON, TileSet.TILE_LAYOUT_STACKED, TileSet.TILE_OFFSET_AXIS_HORIZONTAL],
	"hex_flat": [TileSet.TILE_SHAPE_HEXAGON, TileSet.TILE_LAYOUT_STACKED, TileSet.TILE_OFFSET_AXIS_VERTICAL],
}

func _initialize():
	_corre()

func _tileset(d: Dictionary, pasta: String) -> TileSet:
	var ts = TileSet.new()
	var w = int(d["tile_w"])
	var h = int(d["tile_h"])
	ts.tile_size = Vector2i(w, h)
	var f = FORMAS[d["shape"]]
	ts.tile_shape = f[0]
	ts.tile_layout = f[1]
	ts.tile_offset_axis = f[2]
	var img = Image.load_from_file(pasta.path_join(d["image"]))
	var src = TileSetAtlasSource.new()
	src.texture = ImageTexture.create_from_image(img)
	src.texture_region_size = Vector2i(w, h)
	ts.add_source(src, 0)
	var colunas = int(d["columns"])
	for id in range(1, int(d["tile_count"])):
		src.create_tile(Vector2i(id % colunas, id / colunas))
	return ts

func _camada(ts: TileSet, d: Dictionary, cel: Dictionary) -> TileMapLayer:
	var l = TileMapLayer.new()
	l.tile_set = ts
	# O bloco de omissão (16) parte a ordem de desenho a cada 16 células: P10_oraculos.md, M8a.
	l.rendering_quadrant_size = 4096
	var o = cel["origin"]
	l.position = Vector2(int(o[0]), int(o[1]))
	var bits = d["flip_bits"]
	var colunas = int(d["columns"])
	var tiles = cel["tiles"]
	var w = int(cel["w"])
	for i in range(tiles.size()):
		var t = int(tiles[i])
		var id = t & int(bits["id_mask"])
		if id == 0:
			continue
		var alt = 0
		if t & int(bits["x"]): alt |= TileSetAtlasSource.TRANSFORM_FLIP_H
		if t & int(bits["y"]): alt |= TileSetAtlasSource.TRANSFORM_FLIP_V
		if t & int(bits["d"]): alt |= TileSetAtlasSource.TRANSFORM_TRANSPOSE
		var p = Vector2i(int(cel["col"]) + i % w, int(cel["row"]) + i / w)
		l.set_cell(p, 0, Vector2i(id % colunas, id / colunas), alt)
	return l

func _corre():
	var a = OS.get_cmdline_user_args()
	var pasta = a[0].get_base_dir()
	var d = JSON.parse_string(FileAccess.get_file_as_string(a[0]))
	if d == null:
		printerr("sem entrada: ", a[0])
		quit(1)
		return
	var ts = _tileset(d, pasta)
	var c = d["canvas"]
	var n_quadros = d["frames"].size()
	for q in range(n_quadros):
		var vp = SubViewport.new()
		vp.size = Vector2i(int(c[0]), int(c[1]))
		vp.transparent_bg = true
		vp.canvas_item_default_texture_filter = Viewport.DEFAULT_CANVAS_ITEM_TEXTURE_FILTER_NEAREST
		vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		root.add_child(vp)
		for camada in d["layers"]:
			if not camada["visible"]:
				continue
			for cel in camada["cels"]:
				if int(cel["frame"]) == q:
					var l = _camada(ts, d, cel)
					l.modulate.a = int(camada["opacity"]) / 255.0
					vp.add_child(l)
		for _i in range(3):
			await process_frame
			RenderingServer.force_draw(false)
		vp.get_texture().get_image().save_png(a[1].path_join("godot_%d.png" % q))
		vp.queue_free()
	print("REMONTOU ", n_quadros, " ", RenderingServer.get_video_adapter_name())
	quit()
