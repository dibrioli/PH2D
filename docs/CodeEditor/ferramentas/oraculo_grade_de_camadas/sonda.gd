@tool
extends EditorPlugin
# Sonda do oráculo «como o Godot mostra uma máscara de 32 camadas?» (CodeEditor, report do dono 24/10: a grade do
# «Cull Mask» sem números). SÓ MEDE: selecciona o Sprite2D, abre as secções, rola até à «Visibility Layer», grava a
# geometria do `EditorPropertyLayers` e escreve a fase para o corre.sh fotografar. Fase 1 = dock de omissão;
# fase 2 = o mesmo com o dock estreitado para ~260 px.

var dir := OS.get_environment("SONDA_DIR")
var t := 0.0
var fase := 0

func _mede(no: Node) -> Dictionary:
	var insp := EditorInterface.get_inspector()
	var r := []
	for p in insp.find_children("*", "EditorPropertyLayers", true, false):
		if not p.is_visible_in_tree():
			continue
		var filhos := []
		for c in p.find_children("*", "", true, false):
			if c.is_visible_in_tree() and c is Control:
				filhos.append({"classe": c.get_class(), "x": c.global_position.x, "y": c.global_position.y, "w": c.size.x, "h": c.size.y})
		r.append({"rotulo": p.get("label"), "x": p.global_position.x, "y": p.global_position.y, "w": p.size.x, "h": p.size.y, "filhos": filhos})
	return {"no": no.name, "inspector_w": insp.size.x, "camadas": r}

func _process(d: float) -> void:
	t += d
	var alvo := int(t / 4.0)
	if alvo <= fase or fase >= 2:
		return
	fase += 1
	var raiz := EditorInterface.get_edited_scene_root()
	if raiz == null:
		fase -= 1
		return
	var no: Node = raiz.get_node("Sprite2D")
	EditorInterface.edit_node(no)
	await get_tree().process_frame
	var insp := EditorInterface.get_inspector()
	if fase == 2:
		var dock: Node = insp.get_parent()
		while dock != null and not dock is SplitContainer:
			dock = dock.get_parent()
		if dock != null:
			dock.split_offset += int(insp.size.x) - 260
	for s in insp.find_children("*", "EditorInspectorSection", true, false):
		s.call("unfold")
	await get_tree().process_frame
	await get_tree().process_frame
	for p in insp.find_children("*", "EditorPropertyLayers", true, false):
		if p.get("label") == "Visibility Layer":
			insp.ensure_control_visible(p)
	await get_tree().process_frame
	await get_tree().process_frame
	var f := FileAccess.open(dir + "/fase_%d.json" % fase, FileAccess.WRITE)
	f.store_string(JSON.stringify(_mede(no)))
	f.close()
	var g := FileAccess.open(dir + "/fase", FileAccess.WRITE)
	g.store_string(str(fase))
	g.close()
