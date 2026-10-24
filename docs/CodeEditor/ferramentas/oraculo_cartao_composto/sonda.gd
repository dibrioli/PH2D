@tool
extends EditorPlugin
# Sonda do oráculo «como o Inspector do Godot mostra um nó que junta várias partes?» (CodeEditor, pergunta (c) de
# 22/10). SÓ MEDE: selecciona cada nó, regista as secções do Inspector (nome, profundidade, altura, se dobra) e
# escreve a fase em $SONDA_DIR/fase para o corre.sh fotografar. Fases: 1 Camera2D dobrado · 2 Camera2D aberto ·
# 3 Parallax2D aberto.

var dir := OS.get_environment("SONDA_DIR")
var t := 0.0
var fase := 0
var nos := ["Camera2D", "Parallax2D"]

func _process(d: float) -> void:
	t += d
	var alvo := int(t / 4.0)
	if alvo <= fase or fase >= 3:
		return
	fase += 1
	var raiz := EditorInterface.get_edited_scene_root()
	if raiz == null:
		fase -= 1
		return
	var no: Node = raiz.get_node(nos[0] if fase <= 2 else nos[1])
	EditorInterface.edit_node(no)
	await get_tree().process_frame
	await get_tree().process_frame
	if fase >= 2:
		for s in EditorInterface.get_inspector().find_children("*", "", true, false).filter(func(n): return n.get_class() == "EditorInspectorSection"):
			s.call("unfold")
	await get_tree().process_frame
	await get_tree().process_frame
	var r := []
	for s in EditorInterface.get_inspector().find_children("*", "", true, false).filter(func(n): return n.get_class() == "EditorInspectorSection"):
		if not s.is_visible_in_tree():
			continue
		var prof := 0
		var p = s.get_parent()
		while p != null and not p is EditorInspector:
			if p.get_class() == "EditorInspectorSection":
				prof += 1
			p = p.get_parent()
		r.append({"rotulo": s.get("label"), "prof": prof,
			"y": s.global_position.y, "h": s.size.y})
	var f := FileAccess.open(dir + "/fase_%d.json" % fase, FileAccess.WRITE)
	f.store_string(JSON.stringify({"no": no.name, "classe": no.get_class(), "seccoes": r}))
	f.close()
	var g := FileAccess.open(dir + "/fase", FileAccess.WRITE)
	g.store_string(str(fase))
	g.close()
