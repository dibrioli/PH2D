extends Node
# M1: conduz Manual/Auto/Stack do Pixelorama 1.2.3 numa camada de tilemap. Tudo por API publica/reflexao.
const T := 8   # tile 8x8; projeto 32x32 = 4x4 celulas
var panel: Node
func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame
func _find(n: Node, cls: String, out: Array) -> void:
	var sc = n.get_script()
	if sc != null and sc is Script and sc.get_global_name() == cls:
		out.append(n)
	for c in n.get_children():
		_find(c, cls, out)
func _cls(nm: String) -> Script:
	for c in ProjectSettings.get_global_class_list():
		if c["class"] == nm:
			return load(c["path"])
	return null
func _motion(pos: Vector2i) -> void:
	var e := InputEventMouseMotion.new()
	e.position = Vector2(pos)
	e.pressure = 1.0
	e.button_mask = MOUSE_BUTTON_MASK_LEFT if Tools.active_button != -1 else 0
	Tools.handle_draw(pos, e)
func _button(pos: Vector2i, down: bool) -> void:
	var e := InputEventMouseButton.new()
	e.position = Vector2(pos)
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = down
	Tools.handle_draw(pos, e)
func _tap(pos: Vector2i) -> void:
	_motion(pos + Vector2i(1, 0)); _motion(pos); _button(pos, true); _button(pos, false)
func _img(c: Color, ptsx: Array) -> Image:
	var im := Image.create(T, T, false, Image.FORMAT_RGBA8)
	for p in ptsx: im.set_pixelv(p, c)
	return im
func _tiles_json(ts) -> Array:
	var a := []
	for i in ts.tiles.size():
		var t = ts.tiles[i]
		var d := {"i": i, "h": t.image.get_data().hex_encode().sha256_text().substr(0, 8), "px": 0, "used": t.times_used, "col": ""}
		var n := 0
		for y in T:
			for x in T:
				if t.image.get_pixel(x, y).a > 0:
					n += 1
					d["col"] = t.image.get_pixel(x, y).to_html()
		d["px"] = n
		a.append(d)
	return a
func _cells_json(cel) -> Dictionary:
	var d := {}
	for k in cel.cells:
		var c = cel.cells[k]
		d["%d,%d" % [k.x, k.y]] = [c.index, c.flip_h, c.flip_v, c.transpose]
	return d
func _setup(mode: int, name_: String):
	var proj = ExtensionsApi.project.new_project([], name_, Vector2(4 * T, 4 * T))
	Global.current_project_index = Global.projects.size() - 1
	proj = Global.current_project
	await _frames(5)
	var TS := _cls("TileSetCustom"); var LT := _cls("LayerTileMap")
	var ts = TS.new(Vector2i(T, T), "ts")
	proj.add_tileset(ts)
	var layer = LT.new(proj, ts, "tm")
	proj.layers.append(layer)
	for f in proj.frames:
		f.cels.append(layer.new_empty_cel())
	proj.selected_cels.clear()
	proj.selected_cels.append([0, proj.layers.size() - 1])
	proj.change_cel(0, proj.layers.size() - 1)
	await _frames(5)
	var cel = proj.frames[0].cels[proj.layers.size() - 1]
	# paineis
	var out := []
	_find(get_tree().root, "TileSetPanel", out)
	panel = out[0]
	var bs = panel.mode_buttons_container.get_children()
	if name_.begins_with("init"):
		var st := []
		for b in bs: st.append([b.name, b.button_pressed, b.get("tooltip_text")])
		print("PROBE modebuttons ", JSON.stringify(st))
		print("PROBE layer_class ", layer.get_class_name(), " cel ", cel.get_class_name(), " tiles=", ts.tiles.size())
		print("PROBE static tile_editing_mode=", _cls("TileSetPanel").get("tile_editing_mode"))
	_cls("TileSetPanel").set("tile_editing_mode", mode)
	await _frames(3)
	return [proj, cel, ts]
func _cell_origin(c: Vector2i) -> Vector2i:
	return c * T
func _dump(tag: String, cel, ts) -> void:
	var f := FileAccess.open(OS.get_environment("PROBE_OUT").path_join("log.jsonl"), FileAccess.READ_WRITE)
	if f == null: f = FileAccess.open(OS.get_environment("PROBE_OUT").path_join("log.jsonl"), FileAccess.WRITE)
	f.seek_end()
	f.store_line(JSON.stringify({"tag": tag, "tiles": _tiles_json(ts), "cells": _cells_json(cel)}))
	f.close()
func _prep_map(cel, ts) -> void:
	# tile1 = 1 px vermelho (0,0); tile2 = 1 px verde (1,1); mapa: tile1 em (0,0),(1,0),(2,0); tile2 em (0,1); (3,3) vazio.
	ts.add_tile(_img(Color.RED, [Vector2i(0, 0)]), cel, 0)
	ts.add_tile(_img(Color.GREEN, [Vector2i(1, 1)]), cel, 0)
	cel.set_index(cel.get_cell_at(Vector2i(0, 0)), 1, true)
	cel.set_index(cel.get_cell_at(Vector2i(1, 0)), 1, true)
	cel.set_index(cel.get_cell_at(Vector2i(2, 0)), 1, true)
	cel.set_index(cel.get_cell_at(Vector2i(0, 1)), 2, true)
	cel.update_texture()
func _color(c: Color) -> void:
	var sl = Tools._slots[MOUSE_BUTTON_LEFT]
	sl.color = c
	Tools.assign_tool("Pencil", MOUSE_BUTTON_LEFT)
func _run_mode(mode: int, mname: String) -> void:
	var cenarios := ["vazia", "uma_vez", "varias", "apaga", "igual_a_outro", "dois_iguais", "apaga_varias"]
	for sc in cenarios:
		var r = await _setup(mode, "%s_%s" % [mname, sc])
		var proj = r[0]; var cel = r[1]; var ts = r[2]
		_prep_map(cel, ts)
		await _frames(3)
		Global.can_draw = true
		Tools.assign_tool("Pencil", MOUSE_BUTTON_LEFT)
		await _frames(2)
		var tn = Tools.get_tool(MOUSE_BUTTON_LEFT).tool_node
		tn._on_BrushSize_value_changed(1)
		Tools._slots[MOUSE_BUTTON_LEFT].color = Color.BLUE
		Tools.set_tool_color(Color.BLUE) if Tools.has_method("set_tool_color") else null
		_dump(mname + "/" + sc + "/antes", cel, ts)
		match sc:
			"vazia": _tap(_cell_origin(Vector2i(3, 3)) + Vector2i(2, 2))
			"uma_vez": _tap(_cell_origin(Vector2i(0, 1)) + Vector2i(5, 5))
			"varias": _tap(_cell_origin(Vector2i(1, 0)) + Vector2i(5, 5))
			"igual_a_outro": _tap(_cell_origin(Vector2i(3, 3)) + Vector2i(0, 0))  # vazio -> 1 px azul em (0,0); cor azul != vermelho: ver abaixo
			"dois_iguais":
				_tap(_cell_origin(Vector2i(3, 3)) + Vector2i(2, 2))
				await _frames(3)
				_tap(_cell_origin(Vector2i(3, 2)) + Vector2i(2, 2))
			"apaga_varias":
				Tools.assign_tool("Eraser", MOUSE_BUTTON_LEFT)
				await _frames(2)
				_tap(_cell_origin(Vector2i(1, 0)) + Vector2i(0, 0))
			"apaga":
				Tools.assign_tool("Eraser", MOUSE_BUTTON_LEFT)
				await _frames(2)
				_tap(_cell_origin(Vector2i(0, 1)) + Vector2i(1, 1))
		await _frames(4)
		var im: Image = cel.get_image()
		var n := 0
		for y in im.get_height():
			for x in im.get_width():
				if im.get_pixel(x, y).a > 0: n += 1
		print("PROBE dbg ", mname, sc, " imgpx=", n, " tool=", Tools.get_tool(MOUSE_BUTTON_LEFT).name, " cur=", Global.current_project.get_current_cel().get_class_name(), " placing=", Tools.is_placing_tiles(), " can_draw=", Global.can_draw, " same=", Global.current_project.get_current_cel() == cel, " sel=", Global.current_project.selected_cels, " layers=", Global.current_project.layers.size(), " cels0px=", _px(Global.current_project.frames[0].cels[0].get_image()), " curidx=", Global.current_project.current_layer, " curimgpx=", _px(Global.current_project.get_current_cel().get_image()))
		await _frames(4)
		_dump(mname + "/" + sc + "/depois", cel, ts)
func _grid() -> void:
	var TS := _cls("TileSetCustom"); var LT := _cls("LayerTileMap")
	var rows := []
	for shape in [0, 1, 2, 3]:
		for axis in [0, 1]:
			for sz in [Vector2i(16, 16), Vector2i(15, 15), Vector2i(32, 16), Vector2i(33, 17)]:
				var proj = ExtensionsApi.project.new_project([], "g", Vector2(256, 256))
				Global.current_project_index = Global.projects.size() - 1
				proj = Global.current_project
				await _frames(2)
				var ts = TS.new(sz, "g", shape)
				ts.tile_offset_axis = axis
				var layer = LT.new(proj, ts, "g")
				layer.tile_shape = shape
				layer.tile_offset_axis = axis
				layer.tile_size = sz
				proj.layers.append(layer)
				for f in proj.frames: f.cels.append(layer.new_empty_cel())
				var cel = proj.frames[0].cels[proj.layers.size() - 1]
				var out := []
				for y in range(-2, 3):
					for x in range(-2, 3):
						var pc = cel.get_pixel_coords(Vector2i(x, y))
						var centre = Vector2i(pc) + sz / 2
						var back = cel.get_cell_position(centre)
						out.append([x, y, pc.x, pc.y, back.x, back.y])
				rows.append({"forma": shape, "eixo": axis, "tile": [sz.x, sz.y], "ts_shape": ts.tile_shape, "cel_shape": cel.get_tile_shape(), "linhas": out})
	var f := FileAccess.open(OS.get_environment("PROBE_OUT").path_join("grid.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(rows))
	f.close()
func _ready() -> void:
	await _frames(60)
	if OS.get_environment("PROBE_MODE") == "grid":
		await _grid()
		print("PROBE griddone")
		get_tree().quit()
		return
	var r = await _setup(0, "init")
	await _run_mode(0, "manual")
	await _run_mode(1, "auto")
	await _run_mode(2, "stack")
	print("PROBE done")
	get_tree().quit()
func _px(im: Image) -> int:
	var n := 0
	for y in im.get_height():
		for x in im.get_width():
			if im.get_pixel(x, y).a > 0: n += 1
	return n
