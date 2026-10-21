@tool
extends EditorPlugin
# Sonda do oráculo «arrastar para o código» (CodeEditor item 8). SÓ MEDE: diz em que pixel do ecrã está cada coisa
# (linha da árvore da cena, ficheiro, propriedade do Inspector, linha:coluna do script) e lê o texto do script.
# Os arrastos são gestos reais do rato (xdotool, corre.sh). Protocolo: corre.sh escreve "<id> <cmd> <args>" em
# $SONDA_DIR/cmd; a sonda responde "<id> <json>" em $SONDA_DIR/out.

var dir := OS.get_environment("SONDA_DIR")
var visto := ""
var base := {}  # caminho do script -> texto inicial

func _enter_tree() -> void:
	var t := Timer.new()
	t.wait_time = 0.1
	t.autostart = true
	t.timeout.connect(_tick)
	add_child(t)

func _tick() -> void:
	var p := dir + "/cmd"
	if not FileAccess.file_exists(p):
		return
	var l := FileAccess.get_file_as_string(p).strip_edges()
	if l == "" or l == visto:
		return
	visto = l
	var a := l.split("\t", false)
	var r = _faz(a[1], a.slice(2))
	var f := FileAccess.open(dir + "/out.tmp", FileAccess.WRITE)
	f.store_string(a[0] + " " + JSON.stringify(r))
	f.close()
	DirAccess.rename_absolute(dir + "/out.tmp", dir + "/out")

func _ecra(c: Control, p: Vector2) -> Array:
	var g := c.get_global_transform_with_canvas() * p
	var w := DisplayServer.window_get_position()
	return [int(g.x) + w.x, int(g.y) + w.y]

func _codigo() -> CodeEdit:
	var e := EditorInterface.get_script_editor().get_current_editor()
	return e.get_base_editor() if e else null

func _arvore_da_cena() -> Tree:
	for d in EditorInterface.get_base_control().find_children("*", "SceneTreeDock", true, false):
		for t in d.find_children("*", "Tree", true, false):
			if t.is_visible_in_tree():
				return t
	return null

func _arvore_dos_ficheiros() -> Tree:
	for t in EditorInterface.get_file_system_dock().find_children("*", "Tree", true, false):
		if t.is_visible_in_tree():
			return t
	return null

func _itens(t: Tree) -> Array:
	var r := []
	var i := t.get_root()
	while i:
		r.append(i)
		i = i.get_next_in_tree()
	return r

func _no(caminho: String) -> Node:
	var raiz := EditorInterface.get_edited_scene_root()
	return raiz if caminho == "." else raiz.get_node_or_null(caminho)

func _ponto_do_item(t: Tree, alvo) -> Array:
	for i in _itens(t):
		var m = i.get_metadata(0)
		var casa := false
		if alvo is Node:
			casa = m is NodePath and get_tree().root.get_node_or_null(m) == alvo
		else:
			casa = str(m) == str(alvo)
		if casa:
			t.scroll_to_item(i)
			var r := t.get_item_area_rect(i, 0)
			return _ecra(t, r.position + Vector2(min(40.0, r.size.x / 2), r.size.y / 2))
	return []

func _faz(cmd: String, a: Array):
	match cmd:
		"abre":  # abre <cena> <script>
			EditorInterface.open_scene_from_path(a[0])
			EditorInterface.set_main_screen_editor("Script")
			var s: Script = load(a[1])
			EditorInterface.edit_script(s)
			var c := _codigo()
			if not base.has(a[1]):
				base[a[1]] = c.text
			return {"ok": true, "script": a[1],
				"type_hints": EditorInterface.get_editor_settings().get_setting("text_editor/completion/add_type_hints")}
		"repoe":  # repoe <script>: o texto inicial, sem histórico
			var c := _codigo()
			c.text = base[a[0]]
			c.clear_undo_history()
			c.set_caret_line(0)
			c.set_caret_column(0)
			return {"ok": true}
		"sel":  # sel <caminho>[,<caminho>...]
			var s := EditorInterface.get_selection()
			s.clear()
			var ultimo: Node = null
			for k in a[0].split(","):
				ultimo = _no(k)
				s.add_node(ultimo)
			EditorInterface.edit_node(ultimo)
			return {"ok": ultimo != null}
		"no":  # no <caminho>: o pixel da linha dele na árvore da cena
			return {"xy": _ponto_do_item(_arvore_da_cena(), _no(a[0]))}
		"arquivo":  # arquivo res://...
			EditorInterface.get_file_system_dock().navigate_to_path(a[0])
			return {"xy": _ponto_do_item(_arvore_dos_ficheiros(), a[0])}
		"prop":  # prop <nome>: o pixel do rótulo da propriedade no Inspector
			for p in EditorInterface.get_inspector().find_children("*", "EditorProperty", true, false):
				if p.get_edited_property() == a[0] and p.is_visible_in_tree():
					# o rótulo: na 1.ª linha da propriedade (a de várias linhas tem o editor por BAIXO do rótulo)
					var y: float = 10.0 if p.size.y > 40 else p.size.y / 2
					return {"xy": _ecra(p, Vector2(12, y)), "classe": p.get_class()}
			return {"xy": []}
		"pos":  # pos <linha> <coluna> (base 0)
			var c := _codigo()
			# o rect devolvido é o do carácter ANTES da coluna (medido 21/10: +2 px caía na coluna 0 com um Tab antes)
			var r := c.get_rect_at_line_column(int(a[0]), int(a[1]))
			var x := r.end.x + 2 if int(a[1]) > 0 else r.position.x + 2
			return {"xy": _ecra(c, Vector2(x, r.position.y + r.size.y / 2))}
		"debaixo":  # o que está sob o rato agora
			var h := get_viewport().gui_get_hovered_control()
			var r := {"controlo": h.get_class() if h else ""}
			if h is Tree:
				var i := (h as Tree).get_item_at_position(h.get_local_mouse_position())
				r["item"] = str(i.get_metadata(0)) if i else ""
			if h is TextEdit:
				r["linha_coluna"] = h.get_line_column_at_pos(Vector2i(h.get_local_mouse_position()), false)
			return r
		"texto":
			var c := _codigo()
			return {"texto": c.text, "cursor": [c.get_caret_line(), c.get_caret_column()],
				"selecionado": c.get_selected_text()}
		"avisos":  # todo o texto visível no editor do script fora do CodeEdit (barra de erros/avisos)
			var e := EditorInterface.get_script_editor().get_current_editor()
			var r := []
			for k in e.find_children("*", "Control", true, false):
				if k is CodeEdit or not k.is_visible_in_tree():
					continue
				var tx := ""
				if k is Label or k is Button:
					tx = k.text
				elif k is RichTextLabel:
					tx = k.get_parsed_text()
				if tx.strip_edges() != "":
					r.append(tx.strip_edges())
			return {"avisos": r}
		"completar":  # as opções da lista de sugestões aberta agora
			var c := _codigo()
			return {"opcoes": c.get_code_completion_options().map(func(o): return o["display_text"] + " → " + o["insert_text"])}
		"ecra":  # o ecrã principal do editor e o script aberto
			var s := EditorInterface.get_script_editor().get_current_script()
			return {"script": s.resource_path if s else "", "inspector": str(EditorInterface.get_inspector().get_edited_object())}
		"selecao":
			return {"selecao": EditorInterface.get_selection().get_selected_nodes().map(func(n): return str(n.name))}
		"nome":  # nome <caminho>: o nome actual do nó
			var n := _no(a[0])
			return {"nome": n.name if n else null}
	return {"erro": "cmd desconhecido " + cmd}
