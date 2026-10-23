@tool
extends EditorPlugin
# Sonda do oráculo «o @export aparece no Inspector sem salvar?» (CodeEditor 1b). SÓ MEDE: a cada 50 ms lê o texto do
# script aberto e as propriedades DE SCRIPT que o Inspector mostra (os EditorProperty cujo nome não é da classe do nó),
# e regista cada mudança com o relógio. A escrita são teclas reais (xdotool, corre.sh). Protocolo igual ao do
# oraculo_arrastar: corre.sh escreve "<id>\t<cmd>\t<args>" em $SONDA_DIR/cmd; a resposta vai para $SONDA_DIR/out.

var dir := OS.get_environment("SONDA_DIR")
var visto := ""
var t0 := 0
var log := []
var ultimo := ""

func _enter_tree() -> void:
	var t := Timer.new()
	t.wait_time = 0.05
	t.autostart = true
	t.timeout.connect(_tick)
	add_child(t)

func _tick() -> void:
	_regista()
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

func _codigo() -> CodeEdit:
	var e := EditorInterface.get_script_editor().get_current_editor()
	return e.get_base_editor() if e else null

func _props() -> Array:
	var o = EditorInterface.get_inspector().get_edited_object()
	if not o is Node:
		return []
	var da_classe := {}
	for p in ClassDB.class_get_property_list(o.get_class()):
		da_classe[p["name"]] = true
	var r := []
	for e in EditorInterface.get_inspector().find_children("*", "EditorProperty", true, false):
		var n: String = e.get_edited_property()
		if not da_classe.has(n) and n != "script" and e.is_visible_in_tree():
			r.append(n + "=" + str(o.get(n)))
	return r

func _regista() -> void:
	var c := _codigo()
	var tx := c.text if c else ""
	var linha := {"texto_len": tx.length(), "props": _props(), "obj": str(EditorInterface.get_inspector().get_edited_object())}
	var chave := JSON.stringify(linha)
	if chave != ultimo:
		ultimo = chave
		linha["t_ms"] = Time.get_ticks_msec() - t0
		log.append(linha)

func _no(caminho: String) -> Node:
	var raiz := EditorInterface.get_edited_scene_root()
	return raiz if caminho == "." else raiz.get_node_or_null(caminho)

func _faz(cmd: String, a: Array):
	match cmd:
		"nome":
			return {"nome": EditorInterface.get_edited_scene_root().name if EditorInterface.get_edited_scene_root() else null}
		"abre":  # abre <cena> <script>
			EditorInterface.open_scene_from_path(a[0])
			EditorInterface.set_main_screen_editor("Script")
			EditorInterface.edit_script(load(a[1]))
			var es := EditorInterface.get_editor_settings()
			return {"ok": true, "idle_parse_delay": es.get_setting("text_editor/completion/idle_parse_delay"),
				"code_complete_delay": es.get_setting("text_editor/completion/code_complete_delay"),
				"auto_reload_scripts_on_external_change": es.get_setting("text_editor/behavior/files/auto_reload_scripts_on_external_change")}
		"sel":  # sel <caminho>: selecciona o nó (o Inspector passa a mostrá-lo)
			var s := EditorInterface.get_selection()
			s.clear()
			var n := _no(a[0])
			s.add_node(n)
			EditorInterface.edit_node(n)
			return {"ok": n != null}
		"vigia":  # zera o relógio e o registo
			t0 = Time.get_ticks_msec()
			log = []
			ultimo = ""
			_regista()
			return {"ok": true}
		"marca":  # marca <rótulo>: uma linha no registo, com o relógio
			log.append({"t_ms": Time.get_ticks_msec() - t0, "marca": a[0]})
			return {"ok": true}
		"lido":  # o registo desde o último «vigia»
			return {"log": log}
		"caret_em":  # caret_em <texto>: o cursor no FIM da 1.ª ocorrência
			var c := _codigo()
			for i in c.get_line_count():
				var k := c.get_line(i).find(a[0])
				if k >= 0:
					c.set_caret_line(i)
					c.set_caret_column(k + a[0].length())
					c.grab_focus()
					return {"ok": true, "linha": i}
			return {"ok": false}
		"setv":  # setv <caminho> <prop> <valor float>: o valor PRÓPRIO do nó (o que o artista pôs no Inspector)
			var n := _no(a[0])
			n.set(a[1], float(a[2]))
			n.notify_property_list_changed()
			return {"ok": true, "lido": n.get(a[1])}
		"getv":
			var n := _no(a[0])
			return {"valor": n.get(a[1])}
		"texto":
			var c := _codigo()
			return {"texto": c.text}
		"avisos":  # o texto visível no editor do script fora do CodeEdit (a barra de erros)
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
		"abas":  # o nome da aba do script (o «(*)» = por salvar)
			var lista := EditorInterface.get_script_editor().find_children("*", "ItemList", true, false)
			var r := []
			for il in lista:
				for i in il.item_count:
					r.append(il.get_item_text(i))
			return {"abas": r}
	return {"erro": "cmd desconhecido " + cmd}
