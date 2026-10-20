extends SceneTree
# Oráculo «a API que funciona» (CodeEditor item 6): o motor do Godot 4.7.2 CORRIDO sem janela, a 60 quadros fixos,
# sobre casos NOSSOS. Mede o QUE acontece (nunca como): W esperar · N achar por nome · S criar/apagar · G ler outro
# objecto · I teclado e rato · M dados por objecto. Cada linha da saída = «f<quadro> <caso> <facto>».
# Corrida: corre.sh (grava a fixtura com cabeçalho).

var F := 0
var L: Array[String] = []
var raiz: Node2D

func fato(s: String) -> void:
	L.append("f%d %s" % [F, s])

class Relogio extends Node:
	var o
	func _init(t) -> void:
		o = t; process_priority = -1000; name = "Relogio"
	func _process(_d: float) -> void:
		o.F += 1
		o.injeta()

# W — esperar dentro do gancho de cada quadro
class EsperaNoGancho extends Node:
	var o; var chamadas := 0; var acum := 0.0; var ativo := false
	func _init(t) -> void:
		o = t; name = "EsperaNoGancho"
	func _process(d: float) -> void:
		acum += d; chamadas += 1
		if o.F >= 2 and o.F <= 12:
			o.fato("W gancho chamada=%d" % chamadas)
		if o.F == 2 and not ativo:
			ativo = true
			var a0 := acum
			o.fato("W gancho antes do await 0.1")
			await o.create_timer(0.1).timeout
			o.fato("W gancho depois do await 0.1: tempo de jogo passado=%.6f chamadas do gancho entretanto=%d" % [acum - a0, chamadas])

class EsperaTempos extends Node:
	var o; var acum := 0.0
	func _init(t) -> void:
		o = t; name = "EsperaTempos"
	func _process(d: float) -> void:
		acum += d
		if o.F == 3:
			espera(0.0, "0")
			espera(1.0 / 60.0, "1/60")
			espera(0.05, "0.05")
			espera(0.1, "0.1")
			quadro()
	func espera(s: float, r: String) -> void:
		var a0 := acum; var f0: int = o.F
		await o.create_timer(s).timeout
		o.fato("W espera(%s): quadros=%d tempo=%.6f" % [r, o.F - f0, acum - a0])
	func quadro() -> void:
		var f0: int = o.F
		await o.process_frame
		o.fato("W await process_frame: quadros=%d" % [o.F - f0])

class EsperaApagado extends Node:
	var o
	func _init(t) -> void:
		o = t; name = "EsperaApagado"
	func _process(_d: float) -> void:
		if o.F == 2:
			fica()
		if o.F == 3:
			o.fato("W apagado: queue_free a meio da espera")
			queue_free()
	func fica() -> void:
		await o.create_timer(0.1).timeout
		o.fato("W apagado: a continuação CORREU")

class Sonda extends Node2D:
	var o
	func _init(t, n: String) -> void:
		o = t; name = n
	func _ready() -> void:
		o.fato("S %s _ready" % name)
	func _process(_d: float) -> void:
		if o.F >= 5 and o.F <= 10:
			o.fato("S %s _process" % name)

# N / S / G / M — um objecto «Fabrica» a meio da ordem de processamento
class Fabrica extends Node2D:
	var o; var n: Node
	func _init(t) -> void:
		o = t; name = "Fabrica"
	func _process(_d: float) -> void:
		var r: Node = o.raiz
		if o.F == 4:
			var a = r.find_child("Alvo", true, false)
			o.fato("N find_child(Alvo, recursivo): %s" % (a.get_path() if a else "null"))
			o.fato("N get_node(Pasta/Alvo): %s" % r.get_node("Pasta/Alvo").get_path())
			o.fato("N find_child(Nenhum): %s" % str(r.find_child("Nenhum", true, false)))
			var d := Node2D.new(); d.name = "Alvo"; r.add_child(d)
			o.fato("N add_child de outro «Alvo» ao lado do «Alvo»: o nome ficou «%s»" % d.name)
			var d2 := Node2D.new(); d2.name = "Alvo"; r.add_child(d2, true)
			o.fato("N add_child(force_readable_name) de outro «Alvo»: o nome ficou «%s»" % d2.name)
			o.fato("G posição de um objecto SEM script (Pasta): %s" % str(r.get_node("Pasta").position))
		if o.F == 5:
			n = Sonda.new(o, "Nova")
			o.fato("S antes de add_child")
			r.add_child(n)
			o.fato("S depois de add_child; find_child(Nova)=%s" % ("achou" if r.find_child("Nova", true, false) else "null"))
		if o.F == 8:
			n.queue_free()
			o.fato("S queue_free; is_instance_valid=%s find_child(Nova)=%s" % [is_instance_valid(n), "achou" if r.find_child("Nova", true, false) else "null"])
		if o.F == 9:
			o.fato("S quadro seguinte ao queue_free: is_instance_valid=%s find_child(Nova)=%s" % [is_instance_valid(n), "achou" if r.find_child("Nova", true, false) else "null"])
		if o.F == 6:
			var m := Node.new()
			m.set_meta("k", 1)
			o.fato("M set_meta(k,1): has_meta=%s lista=%s" % [m.has_meta("k"), m.get_meta_list()])
			m.set_meta("k", null)
			o.fato("M set_meta(k,null): has_meta=%s lista=%s" % [m.has_meta("k"), m.get_meta_list()])
			m.set_meta("lista", [10, 20, 30])
			var l = m.get_meta("lista")
			o.fato("M set_meta(lista,[10,20,30]): tipo=%s tamanho=%d [0]=%s" % [type_string(typeof(l)), l.size(), str(l[0])])
			m.set_meta("tab", {"a": 1, "b": [1, 2]})
			o.fato("M set_meta(tab,{a:1,b:[1,2]}): volta=%s" % str(m.get_meta("tab")))
			m.remove_meta("lista")
			o.fato("M remove_meta(lista): lista=%s" % [m.get_meta_list()])
			m.free()

class Leitor extends Node2D:
	var o
	func _init(t) -> void:
		o = t; name = "Leitor"
	func _process(_d: float) -> void:
		if o.F >= 19 and o.F <= 26:
			o.fato("I A: is_key_pressed=%s | ui_left: pressed=%s just_pressed=%s just_released=%s | ui_right(toque): pressed=%s just_pressed=%s just_released=%s" % [
				Input.is_key_pressed(KEY_A), Input.is_action_pressed("ui_left"), Input.is_action_just_pressed("ui_left"),
				Input.is_action_just_released("ui_left"), Input.is_action_pressed("ui_right"),
				Input.is_action_just_pressed("ui_right"), Input.is_action_just_released("ui_right")])
		if o.F >= 30 and o.F <= 32:
			o.fato("I rato: viewport=%s global(câmara zoom 2, em 300,0)=%s botão esq=%s janela=%s" % [
				get_viewport().get_mouse_position(), get_global_mouse_position(),
				Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT), get_viewport().get_visible_rect().size])

func tecla(k: Key, p: bool) -> void:
	var e := InputEventKey.new(); e.keycode = k; e.physical_keycode = k; e.pressed = p
	Input.parse_input_event(e)

func injeta() -> void:
	if F == 20:
		tecla(KEY_A, true); tecla(KEY_LEFT, true)
		fato("I (injectado no início do quadro: A e ← carregados)")
	if F == 23:
		tecla(KEY_A, false); tecla(KEY_LEFT, false)
		fato("I (injectado: A e ← largados)")
	if F == 24:
		tecla(KEY_RIGHT, true); tecla(KEY_RIGHT, false)
		fato("I (injectado: → carregado E largado no mesmo quadro)")
	if F == 30:
		var m := InputEventMouseMotion.new(); m.position = Vector2(100, 50)
		Input.parse_input_event(m)
		var b := InputEventMouseButton.new(); b.button_index = MOUSE_BUTTON_LEFT; b.pressed = true; b.position = Vector2(100, 50)
		Input.parse_input_event(b)
		fato("I (injectado: rato em 100,50 do ecrã e botão esquerdo carregado)")
	if F == 40:
		var out := OS.get_cmdline_user_args()[0]
		var f := FileAccess.open(out, FileAccess.WRITE)
		for s in L:
			f.store_line(s)
		f.close()
		quit()

func _initialize() -> void:
	raiz = Node2D.new(); raiz.name = "Raiz"
	root.add_child(raiz)
	raiz.add_child(Relogio.new(self))
	raiz.add_child(EsperaNoGancho.new(self))
	raiz.add_child(EsperaTempos.new(self))
	raiz.add_child(EsperaApagado.new(self))
	var pasta := Node2D.new(); pasta.name = "Pasta"; pasta.position = Vector2(7, 8); raiz.add_child(pasta)
	var fundo := Node2D.new(); fundo.name = "Alvo"; pasta.add_child(fundo)
	raiz.add_child(Fabrica.new(self))
	var raso := Node2D.new(); raso.name = "Alvo"; raiz.add_child(raso)
	var cam := Camera2D.new(); cam.zoom = Vector2(2, 2); cam.position = Vector2(300, 0); cam.anchor_mode = Camera2D.ANCHOR_MODE_FIXED_TOP_LEFT
	raiz.add_child(cam); cam.make_current()
	raiz.add_child(Leitor.new(self))
