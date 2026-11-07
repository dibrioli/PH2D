extends Node
# M9: casos de borda de tilemap no Pixelorama 1.2.3. So corrido (API publica/reflexao); fonte nao lido.
# PROBE_MODE: q12 (borracha, arrasto, duas camadas) | q34 (tamanho, vida do tileset) | discover
const T := 8
const BLUE := Color(0, 0, 1, 1)
const RED := Color(1, 0, 0, 1)
const GREEN := Color(0, 1, 0, 1)
var log_rows := []
var cur_tag := ""
func _frames(n: int) -> void:
	for i in n: await get_tree().process_frame
func _cls(nm: String) -> Script:
	for c in ProjectSettings.get_global_class_list():
		if c["class"] == nm: return load(c["path"])
	return null
func _motion(pos: Vector2i) -> void:
	var e := InputEventMouseMotion.new()
	e.position = Vector2(pos); e.pressure = 1.0
	e.button_mask = MOUSE_BUTTON_MASK_LEFT if Tools.active_button != -1 else 0
	Tools.handle_draw(pos, e)
func _button(pos: Vector2i, down: bool) -> void:
	var e := InputEventMouseButton.new()
	e.position = Vector2(pos); e.button_index = MOUSE_BUTTON_LEFT; e.pressed = down
	Tools.handle_draw(pos, e)
func _drag(a: Vector2i, b: Vector2i) -> void:
	_motion(a + Vector2i(1, 0)); _motion(a); await _frames(1)
	_button(a, true); await _frames(1)
	var p := a
	while p != b:
		p += Vector2i(signi(b.x - p.x), signi(b.y - p.y))
		_motion(p); await _frames(1)
	_button(b, false); await _frames(4)
func _img(c: Color, pts: Array) -> Image:
	var im := Image.create(T, T, false, Image.FORMAT_RGBA8)
	for p in pts: im.set_pixelv(p, c)
	return im
func _row(y: int, x0: int, x1: int) -> Array:
	var a := []
	for x in range(x0, x1 + 1): a.append(Vector2i(x, y))
	return a
func _ascii(im: Image) -> Array:
	var r := []
	for y in im.get_height():
		var s := ""
		for x in im.get_width():
			var c := im.get_pixel(x, y)
			if c.a <= 0.0: s += "."
			elif c.r > 0.5 and c.g < 0.5 and c.b < 0.5: s += "R"
			elif c.g > 0.5 and c.r < 0.5: s += "G"
			elif c.b > 0.5 and c.r < 0.5: s += "B"
			else: s += "?"
		r.append(s)
	return r
func _state(proj, cel, ts) -> Dictionary:
	var tiles := []
	for i in ts.tiles.size():
		tiles.append({"i": i, "used": ts.tiles[i].times_used, "img": _ascii(ts.tiles[i].image)})
	var cells := {}
	for k in cel.cells:
		var c = cel.cells[k]
		if c.index != 0: cells["%d,%d" % [k.x, k.y]] = c.index
	return {"tiles": tiles, "cells": cells, "canvas": _ascii(cel.get_image()), "undo": proj.undo_redo.get_history_count()}
func _short(s: Dictionary) -> String:
	var t := []
	for x in s["tiles"]:
		var n := 0
		for r in x["img"]: n += r.count("R") + r.count("G") + r.count("B")
		t.append("t%d:%dpx,u%d" % [x["i"], n, x["used"]])
	return "tiles[%s] cells=%s undo=%d" % [",".join(t), JSON.stringify(s["cells"]), s["undo"]]
func _rec(tag: String, st: Dictionary) -> void:
	st["tag"] = tag
	log_rows.append(st)
	print("PROBE ", tag, " ", _short(st))
func _flush(nm: String) -> void:
	var f := FileAccess.open(OS.get_environment("PROBE_OUT").path_join(nm), FileAccess.WRITE)
	f.store_string(JSON.stringify(log_rows)); f.close()
func _new_proj(name_: String, w := 4 * T, h := 4 * T):
	ExtensionsApi.project.new_project([], name_, Vector2(w, h))
	Global.current_project_index = Global.projects.size() - 1
	await _frames(4)
	return Global.current_project
func _add_tm(proj, ts, name_: String):
	var LT := _cls("LayerTileMap")
	var layer = LT.new(proj, ts, name_)
	var cels := []
	for f in proj.frames: cels.append(layer.new_empty_cel())
	proj.add_layers([layer], [proj.layers.size()], [cels])
	await _frames(3)
	var idx: int = proj.layers.find(layer)
	proj.selected_cels.clear(); proj.selected_cels.append([0, idx])
	proj.change_cel(0, idx)
	await _frames(3)
	return proj.frames[0].cels[idx]
func _mode(m: int) -> void:
	_cls("TileSetPanel").set("tile_editing_mode", m)
func _tool(nm: String, c := BLUE) -> void:
	Global.can_draw = true
	Tools.assign_tool(nm, MOUSE_BUTTON_LEFT)
	await _frames(2)
	Tools.get_tool(MOUSE_BUTTON_LEFT).tool_node._on_BrushSize_value_changed(1)
	Tools._slots[MOUSE_BUTTON_LEFT].color = c
	await _frames(1)
func _setidx(cel, c: Vector2i, i: int) -> void:
	cel.set_index(cel.get_cell_at(c), i, true)
const MODES := {0: "manual", 1: "auto", 2: "stack"}
# ---------- Q1
func _q1() -> void:
	# controlo: a borracha muda mesmo uma camada normal
	var p0 = await _new_proj("ctrl")
	var im: Image = p0.frames[0].cels[0].get_image()
	im.set_pixel(3, 3, RED); im.set_pixel(4, 3, RED)
	p0.frames[0].cels[0].update_texture()
	await _tool("Eraser")
	print("PROBE ctrl_raster antes ", _ascii(p0.frames[0].cels[0].get_image())[3])
	await _drag(Vector2i(3, 3), Vector2i(3, 3))
	print("PROBE ctrl_raster depois ", _ascii(p0.frames[0].cels[0].get_image())[3], " undo=", p0.undo_redo.get_history_count())
	for m in MODES:
		for sc in ["a_1x_um", "b_1x_todos", "c_3x_um", "d_3x_todos", "b1_1x_pixelunico", "d1_3x_pixelunico", "ctrl_lapis_3x"]:
			var proj = await _new_proj("%s_%s" % [MODES[m], sc])
			var ts = _cls("TileSetCustom").new(Vector2i(T, T), "ts")
			proj.add_tileset(ts)
			var cel = await _add_tm(proj, ts, "tm")
			_mode(m)
			var unico: bool = sc.ends_with("pixelunico")
			var pa := [Vector2i(2, 3)] if unico else _row(3, 2, 5)
			var pb := [Vector2i(2, 5)] if unico else _row(5, 2, 5)
			ts.add_tile(_img(RED, pa), cel, 0)    # tile1, usado 1x
			ts.add_tile(_img(GREEN, pb), cel, 0)  # tile2, usado 3x
			_setidx(cel, Vector2i(0, 1), 1)
			for c in [Vector2i(0, 0), Vector2i(1, 0), Vector2i(2, 0)]: _setidx(cel, c, 2)
			cel.update_texture()
			await _frames(3)
			await _tool("Pencil" if sc.begins_with("ctrl") else "Eraser")
			var tag: String = "q1/%s/%s" % [MODES[m], sc]
			_rec(tag + "/antes", _state(proj, cel, ts))
			match sc:
				"a_1x_um": await _drag(Vector2i(3, 11), Vector2i(3, 11))
				"b_1x_todos": await _drag(Vector2i(2, 11), Vector2i(5, 11))
				"c_3x_um": await _drag(Vector2i(11, 5), Vector2i(11, 5))
				"d_3x_todos": await _drag(Vector2i(10, 5), Vector2i(13, 5))
				"b1_1x_pixelunico": await _drag(Vector2i(2, 11), Vector2i(2, 11))
				"d1_3x_pixelunico": await _drag(Vector2i(10, 5), Vector2i(10, 5))
				"ctrl_lapis_3x": await _drag(Vector2i(11, 1), Vector2i(11, 1))
			_rec(tag + "/depois", _state(proj, cel, ts))
			proj.undo_redo.undo(); await _frames(4)
			_rec(tag + "/undo1", _state(proj, cel, ts))
# ---------- Q2
func _q2() -> void:
	for m in MODES:
		for sc in ["a_2x_diferente", "a_rev_2x_diferente_inverso", "b_2x_igual", "c_3x_diferente", "c2_3x_igual", "e_2x_nao_no_traco_um"]:
			var proj = await _new_proj("%s_%s" % [MODES[m], sc])
			var ts = _cls("TileSetCustom").new(Vector2i(T, T), "ts")
			proj.add_tileset(ts)
			var cel = await _add_tm(proj, ts, "tm")
			_mode(m)
			ts.add_tile(_img(RED, _row(5, 2, 5)), cel, 0)  # tile1
			_setidx(cel, Vector2i(0, 0), 1); _setidx(cel, Vector2i(1, 0), 1)
			if sc.begins_with("c"): _setidx(cel, Vector2i(3, 0), 1)
			cel.update_texture(); await _frames(3)
			await _tool("Pencil")
			var tag: String = "q2/%s/%s" % [MODES[m], sc]
			_rec(tag + "/antes", _state(proj, cel, ts))
			match sc:
				"a_2x_diferente", "c_3x_diferente": await _drag(Vector2i(6, 1), Vector2i(9, 1))
				"a_rev_2x_diferente_inverso": await _drag(Vector2i(9, 1), Vector2i(6, 1))
				"b_2x_igual", "c2_3x_igual": await _drag(Vector2i(0, 1), Vector2i(15, 1))
				"e_2x_nao_no_traco_um": await _drag(Vector2i(2, 1), Vector2i(4, 1))
			_rec(tag + "/depois", _state(proj, cel, ts))
			proj.undo_redo.undo(); await _frames(4)
			_rec(tag + "/undo1", _state(proj, cel, ts))
# ---------- Q5
func _q5() -> void:
	for m in [1, 2, 0]:
		var proj = await _new_proj("q5_%d" % m)
		var ts = _cls("TileSetCustom").new(Vector2i(T, T), "ts")
		proj.add_tileset(ts)
		var c1 = await _add_tm(proj, ts, "a")
		var c2 = await _add_tm(proj, ts, "b")
		_mode(m)
		ts.add_tile(_img(RED, _row(5, 2, 5)), c2, 0)
		_setidx(c1, Vector2i(0, 0), 1); _setidx(c2, Vector2i(1, 1), 1)
		c1.update_texture(); c2.update_texture(); await _frames(3)
		await _tool("Pencil")
		var tag: String = "q5/%s" % MODES[m]
		_rec(tag + "/antes", _state(proj, c2, ts))
		await _drag(Vector2i(10, 9), Vector2i(10, 9))
		_rec(tag + "/depois_b", _state(proj, c2, ts))
		_rec(tag + "/depois_a", _state(proj, c1, ts))
# ---------- descoberta
func _scan(n: Node, out: Array) -> void:
	var sc = n.get_script()
	if sc != null and sc is Script:
		var ms := []
		for mm in sc.get_script_method_list():
			var nm: String = mm["name"]
			if nm.to_lower().contains("convert") or nm.to_lower().contains("tileset"): ms.append(nm + "/" + str(mm["args"].size()))
		if not ms.is_empty(): out.append([str(n.get_path()), sc.resource_path, ms])
	for c in n.get_children(): _scan(c, out)
func _discover() -> void:
	var out := []
	_scan(get_tree().root, out)
	for o in out: print("PROBE scan ", o)
	var S = Global.current_project.selection_map
	var ml := []
	for mm in S.get_script().get_script_method_list(): ml.append(mm["name"] + "/" + str(mm["args"].size()))
	print("PROBE selmap ", ml)
	print("PROBE extapi ", ExtensionsApi.get_children().size(), " sel ", ExtensionsApi.get("selection"))
	var lt = _cls("LayerTileMap").new(Global.current_project, _cls("TileSetCustom").new(Vector2i(T, T), "z"), "z")
	var lm := []
	for mm in lt.get_script().get_script_method_list(): lm.append(mm["name"] + "/" + str(mm["args"].size()))
	print("PROBE layertm ", lm)
	var cm := []
	var cl = lt.new_empty_cel()
	for mm in cl.get_script().get_script_method_list(): cm.append(mm["name"] + "/" + str(mm["args"].size()))
	print("PROBE celtm ", cm)
# ---------- Q3
func _tmcel(proj):
	for i in proj.layers.size():
		if proj.layers[i].get_class_name() == "LayerTileMap" or proj.layers[i].get_layer_type() == 5 or "tileset" in proj.layers[i]:
			return proj.frames[0].cels[i]
	return null
func _st3(proj, ts, tag: String) -> void:
	var cel = _tmcel(proj)
	var st := _state(proj, cel, ts)
	st["size"] = [proj.size.x, proj.size.y]
	st["offset"] = [cel.offset.x, cel.offset.y] if "offset" in cel else null
	_rec(tag, st)
	print("PROBE   size=", proj.size, " offset=", st["offset"], " canvas_px=", _cnt(st["canvas"]), " canvas_img=", cel.get_image().get_size())
func _cnt(rows: Array) -> int:
	var n := 0
	for r in rows: n += r.length() - r.count(".")
	return n
func _prep3(name_: String, m: int):
	var proj = await _new_proj(name_)
	var ts = _cls("TileSetCustom").new(Vector2i(T, T), "ts")
	proj.add_tileset(ts)
	var cel = await _add_tm(proj, ts, "tm")
	_mode(m)
	ts.add_tile(_img(RED, _row(1, 1, 6)), cel, 0)
	ts.add_tile(_img(GREEN, _row(3, 1, 6)), cel, 0)
	ts.add_tile(_img(BLUE, _row(5, 1, 6)), cel, 0)
	_setidx(cel, Vector2i(0, 0), 1); _setidx(cel, Vector2i(1, 1), 2); _setidx(cel, Vector2i(3, 3), 3); _setidx(cel, Vector2i(2, 2), 1)
	cel.update_texture(); await _frames(3)
	return [proj, ts]
func _q3() -> void:
	for m in [1, 0]:
		var mn: String = MODES[m]
		var r = await _prep3("enc", m); var proj = r[0]; var ts = r[1]
		_st3(proj, ts, "q3/%s/enc/antes" % mn)
		DrawingAlgos.resize_canvas(16, 16, 0, 0); await _frames(4)
		_st3(proj, ts, "q3/%s/enc/16x16" % mn)
		DrawingAlgos.resize_canvas(32, 32, 0, 0); await _frames(4)
		_st3(proj, ts, "q3/%s/enc/volta32" % mn)
		proj.undo_redo.undo(); await _frames(4); proj.undo_redo.undo(); await _frames(4)
		_st3(proj, ts, "q3/%s/enc/undo2" % mn)
		r = await _prep3("nm", m); proj = r[0]; ts = r[1]
		DrawingAlgos.resize_canvas(20, 20, 0, 0); await _frames(4)
		_st3(proj, ts, "q3/%s/naomult/20x20" % mn)
		DrawingAlgos.resize_canvas(32, 32, 0, 0); await _frames(4)
		_st3(proj, ts, "q3/%s/naomult/volta32" % mn)
		r = await _prep3("of", m); proj = r[0]; ts = r[1]
		DrawingAlgos.resize_canvas(40, 40, 8, 8); await _frames(4)
		_st3(proj, ts, "q3/%s/offset8/40x40" % mn)
		r = await _prep3("of3", m); proj = r[0]; ts = r[1]
		DrawingAlgos.resize_canvas(35, 35, 3, 3); await _frames(4)
		_st3(proj, ts, "q3/%s/offset3/35x35" % mn)
		r = await _prep3("neg", m); proj = r[0]; ts = r[1]
		DrawingAlgos.resize_canvas(16, 16, -16, -16); await _frames(4)
		_st3(proj, ts, "q3/%s/negativo/16x16" % mn)
		DrawingAlgos.resize_canvas(32, 32, 16, 16); await _frames(4)
		_st3(proj, ts, "q3/%s/negativo/volta32" % mn)
		for rc in [Rect2i(8, 8, 16, 16), Rect2i(3, 3, 16, 16)]:
			r = await _prep3("crop", m); proj = r[0]; ts = r[1]
			proj.selection_map.select_rect(rc, true)
			proj.selection_map_changed(); await _frames(3)
			DrawingAlgos.crop_to_selection(); await _frames(4)
			_st3(proj, ts, "q3/%s/crop_%d_%d/depois" % [mn, rc.position.x, rc.position.y])
		r = await _prep3("cc", m); proj = r[0]; ts = r[1]
		DrawingAlgos.crop_to_content(); await _frames(4)
		_st3(proj, ts, "q3/%s/crop_content/depois" % mn)
# ---------- Q4
const DLG := "/root/Control/MenuAndUI/UI/DockableContainer/Animation Timeline/NewTileMapLayerDialog"
func _opts(n: Node, out: Array) -> void:
	if n is OptionButton:
		var a := []
		for i in n.item_count: a.append(n.get_item_text(i))
		out.append([n.name, a, n.selected])
	for c in n.get_children(): _opts(c, out)
func _dlg_list() -> Array:
	var d = get_node(DLG)
	d.about_to_popup.emit()
	await _frames(2)
	var o := []
	_opts(d, o)
	return o
func _st4(tag: String) -> void:
	var proj = Global.current_project
	var sets := []
	for t in proj.tilesets: sets.append([t.name, t.tiles.size()])
	var lays := []
	for l in proj.layers: lays.append(l.name + ":" + l.get_script().get_global_name())
	var o = await _dlg_list()
	print("PROBE ", tag, " tilesets=", sets, " layers=", lays, " dialog=", o, " undo=", proj.undo_redo.get_history_count())
	log_rows.append({"tag": tag, "tilesets": sets, "layers": lays, "dialog": o})
func _prep4(name_: String):
	var proj = await _new_proj(name_)
	var ts = _cls("TileSetCustom").new(Vector2i(T, T), "ts_m9")
	proj.add_tileset(ts)
	var cel = await _add_tm(proj, ts, "tm")
	_mode(1)
	ts.add_tile(_img(RED, _row(1, 1, 6)), cel, 0)
	_setidx(cel, Vector2i(0, 0), 1); _setidx(cel, Vector2i(1, 1), 1)
	cel.update_texture(); await _frames(3)
	return proj
func _saveopen(tag: String, proj) -> void:
	var path := OS.get_environment("PROBE_OUT").path_join(tag.replace("/", "_") + ".pxo")
	var ok = OpenSave.save_pxo_file(path, false, false, proj)
	await _frames(5)
	var n: int = Global.projects.size()
	OpenSave.open_pxo_file(path, false, false)
	await _frames(10)
	print("PROBE ", tag, " save=", ok, " size=", FileAccess.get_file_as_bytes(path).size(), " projects ", n, "->", Global.projects.size())
	await _st4(tag + "/reaberto")
func _layer_menus(n: Node) -> void:
	if n is PopupMenu:
		var a := []
		for i in n.item_count: a.append(n.get_item_text(i))
		if not a.is_empty(): print("PROBE popup ", n.get_path(), " ", a)
	for c in n.get_children(): _layer_menus(c)
func _q4() -> void:
	var m := []
	for mm in OpenSave.get_script().get_script_method_list():
		if mm["name"] in ["save_pxo_file", "open_pxo_file"]: m.append([mm["name"], mm["args"].map(func(a): return a["name"])])
	for mm in Global.animation_timeline.get_script().get_script_method_list():
		if mm["name"] in ["flatten_layers", "add_layer", "on_add_layer_list_id_pressed"]: m.append([mm["name"], mm["args"].map(func(a): return a["name"])])
	print("PROBE args ", m)
	var proj = await _prep4("ctrl4")
	await _st4("q4/controlo")
	await _saveopen("q4/controlo", proj)
	_layer_menus(Global.animation_timeline)
	_layer_menus(get_node("/root/Control/Dialogs/LayerProperties"))
	proj = await _prep4("del")
	Global.animation_timeline._on_RemoveLayer_pressed(); await _frames(5)
	await _st4("q4/a_apagada")
	await _saveopen("q4/a_apagada", proj)
	proj = await _prep4("merge")
	Global.animation_timeline._on_MergeDownLayer_pressed(); await _frames(5)
	await _st4("q4/b_merge_down")
	await _saveopen("q4/b_merge_down", proj)
	proj = await _prep4("flat")
	var idxs := []
	for i in proj.layers.size(): idxs.append(i)
	Global.animation_timeline.flatten_layers(idxs, false); await _frames(5)
	await _st4("q4/b_flatten")
	await _saveopen("q4/b_flatten", proj)
	proj = await _prep4("flat1")
	Global.animation_timeline.flatten_layers([proj.layers.size() - 1], false); await _frames(5)
	await _st4("q4/c_flatten_so_a_tm")
	await _saveopen("q4/c_flatten_so_a_tm", proj)
	var pa = Global.projects[Global.projects.size() - 2]
	print("PROBE q4/c pix=", _cnt(_ascii(pa.frames[0].cels[pa.layers.size() - 1].get_image())))
# ---------- real: ponteiro real (xdotool, Xvfb) em vez de Tools.handle_draw
func _coords() -> Dictionary:
	var vp = Global.canvas.get_viewport()
	var xf: Transform2D = vp.get_canvas_transform()
	var cont = vp.get_parent()
	var org: Vector2 = cont.get_global_rect().position if cont is Control else Vector2.ZERO
	var win := DisplayServer.window_get_position()
	return {"o": [xf.origin.x + org.x + win.x, xf.origin.y + org.y + win.y], "s": xf.x.x, "win": [win.x, win.y]}
func _wait_go(k: String) -> void:
	var p := OS.get_environment("PROBE_OUT").path_join("go_" + k)
	while not FileAccess.file_exists(p): await _frames(5)
	await _frames(10)
func _real() -> void:
	for sc in [["manual_d", 0, "Eraser"], ["stack_b", 2, "Eraser"], ["auto_d", 1, "Eraser"], ["auto_q2a", 1, "Pencil"], ["ctrl_manual_lapis", 0, "Pencil"]]:
		var proj = await _new_proj(sc[0])
		var ts = _cls("TileSetCustom").new(Vector2i(T, T), "ts")
		proj.add_tileset(ts)
		var cel = await _add_tm(proj, ts, "tm")
		_mode(sc[1])
		ts.add_tile(_img(RED, _row(3, 2, 5)), cel, 0)
		ts.add_tile(_img(GREEN, _row(5, 2, 5)), cel, 0)
		_setidx(cel, Vector2i(0, 1), 1)
		if sc[0] == "auto_q2a":
			_setidx(cel, Vector2i(0, 0), 2); _setidx(cel, Vector2i(1, 0), 2)
		else:
			for c in [Vector2i(0, 0), Vector2i(1, 0), Vector2i(2, 0)]: _setidx(cel, c, 2)
		cel.update_texture(); await _frames(3)
		await _tool(sc[2])
		await _frames(10)
		_rec("real/%s/antes" % sc[0], _state(proj, cel, ts))
		var f := FileAccess.open(OS.get_environment("PROBE_OUT").path_join("coords_" + sc[0] + ".json"), FileAccess.WRITE)
		f.store_string(JSON.stringify(_coords())); f.close()
		await _wait_go(sc[0])
		_rec("real/%s/depois" % sc[0], _state(proj, cel, ts))
		var st := _state(proj, cel, ts)
		print("PROBE   canvas ", st["canvas"].filter(func(l): return l.count(".") < l.length()))
# ---------- seguimento: depois de apagar TUDO em Manual/Stack o canvas e o tile divergem; o que os reconcilia?
func _q1x() -> void:
	for sc in [["manual_b", 0, Vector2i(2, 11), Vector2i(5, 11), Vector2i(3, 9)], ["stack_b", 2, Vector2i(2, 11), Vector2i(5, 11), Vector2i(3, 9)], ["manual_d", 0, Vector2i(10, 5), Vector2i(13, 5), Vector2i(11, 1)], ["stack_d", 2, Vector2i(10, 5), Vector2i(13, 5), Vector2i(11, 1)]]:
		for fim in ["reabre", "lapis"]:
			var proj = await _new_proj(sc[0])
			var ts = _cls("TileSetCustom").new(Vector2i(T, T), "ts")
			proj.add_tileset(ts)
			var cel = await _add_tm(proj, ts, "tm")
			_mode(sc[1])
			ts.add_tile(_img(RED, _row(3, 2, 5)), cel, 0)
			ts.add_tile(_img(GREEN, _row(5, 2, 5)), cel, 0)
			_setidx(cel, Vector2i(0, 1), 1)
			for c in [Vector2i(0, 0), Vector2i(1, 0), Vector2i(2, 0)]: _setidx(cel, c, 2)
			cel.update_texture(); await _frames(3)
			await _tool("Eraser")
			await _drag(sc[2], sc[3])
			var tag: String = "q1x/%s/%s" % [sc[0], fim]
			_rec(tag + "/apagado", _state(proj, cel, ts))
			if fim == "lapis":
				await _tool("Pencil")
				await _drag(sc[4], sc[4])
				_rec(tag + "/lapis_mesma_celula", _state(proj, cel, ts))
			else:
				var path := OS.get_environment("PROBE_OUT").path_join(sc[0] + ".pxo")
				OpenSave.save_pxo_file(path, false, false, proj); await _frames(4)
				OpenSave.open_pxo_file(path, false, false); await _frames(10)
				var np = Global.current_project
				_rec(tag + "/reaberto", _state(np, np.frames[0].cels[np.layers.size() - 1], np.tilesets[0]))
# ---------- M9b: Auto, A=1 B=2 C=3; ordem da lista e quem guarda o original
const M9B := [
	["c1_iguais", [[0, 0], [1, 0]], [[0, 2]], [[1, 2]], [0, 1], [15, 1]],
	["c2_diferentes", [[0, 0], [1, 0]], [[0, 2]], [[1, 2]], [6, 1], [9, 1]],
	["c2_diferentes_inv", [[0, 0], [1, 0]], [[0, 2]], [[1, 2]], [9, 1], [6, 1]],
	["c3_vert_desce", [[0, 0], [0, 1]], [[2, 0]], [[3, 0]], [1, 6], [1, 9]],
	["c3_vert_sobe", [[0, 0], [0, 1]], [[2, 0]], [[3, 0]], [1, 9], [1, 6]],
	["c3_vert_desce_3x", [[0, 0], [0, 1], [3, 3]], [[2, 0]], [[3, 0]], [1, 6], [1, 9]],
	["c3_vert_sobe_3x", [[0, 0], [0, 1], [3, 3]], [[2, 0]], [[3, 0]], [1, 9], [1, 6]],
	["c4_diag_1_0_para_0_1", [[0, 1], [1, 0]], [[3, 3]], [[2, 3]], [10, 5], [5, 10]],
	["c4_diag_0_1_para_1_0", [[0, 1], [1, 0]], [[3, 3]], [[2, 3]], [5, 10], [10, 5]],
	["c4_diag_1_0_para_0_1_3x", [[0, 1], [1, 0], [3, 0]], [[3, 3]], [[2, 3]], [10, 5], [5, 10]],
	["c4_diag_0_1_para_1_0_3x", [[0, 1], [1, 0], [3, 0]], [[3, 3]], [[2, 3]], [5, 10], [10, 5]],
]
func _m9b() -> void:
	for sc in M9B:
		var proj = await _new_proj(sc[0])
		var ts = _cls("TileSetCustom").new(Vector2i(T, T), "ts")
		proj.add_tileset(ts)
		var cel = await _add_tm(proj, ts, "tm")
		_mode(1)
		ts.add_tile(_img(RED, _row(2, 2, 4)), cel, 0)
		ts.add_tile(_img(GREEN, _row(3, 2, 5)), cel, 0)
		ts.add_tile(_img(GREEN, _row(6, 2, 5)), cel, 0)
		for k in [[sc[1], 1], [sc[2], 2], [sc[3], 3]]:
			for c in k[0]: _setidx(cel, Vector2i(c[0], c[1]), k[1])
		cel.update_texture(); await _frames(3)
		await _tool("Pencil")
		var tag: String = "m9b/%s" % sc[0]
		_rec(tag + "/antes", _state(proj, cel, ts))
		await _drag(Vector2i(sc[4][0], sc[4][1]), Vector2i(sc[5][0], sc[5][1]))
		_rec(tag + "/depois", _state(proj, cel, ts))
func _ready() -> void:
	await _frames(60)
	var mode := OS.get_environment("PROBE_MODE")
	if mode == "q12":
		await _q1(); await _q2(); await _q5()
		_flush("q125.json")
	elif mode == "discover":
		await _discover()
	elif mode == "q1x":
		await _q1x(); _flush("q1x.json")
	elif mode == "m9b":
		await _m9b(); _flush("m9b.json")
	elif mode == "q34":
		await _q3(); await _q4(); _flush("q34.json")
	elif mode == "real":
		await _real(); _flush("real.json")
	print("PROBE done")
	get_tree().quit()
