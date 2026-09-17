# ORÁCULO: o HUD (canvas em screen-space, escala por resolução, e o clique de um botão) no
# Godot 4.7.2 (MIT), corrido SEM INTERFACE — docs/Components/15_plano_hud.md §2.
# Correr: godot --headless --fixed-fps 60 --script docs/Components/ferramentas/godot_hud_probe.gd
#
# ⚠️ As entradas são NOSSAS (as nossas resoluções e os nossos rectângulos); o que se lê é o número
# que ele devolve. Nenhum fonte é lido — a triagem parou na porta aberta (MIT), e mesmo assim a
# regra do §0.9 é correr, não ler.
#
# ⛔⛔ A 1.ª REDACÇÃO DESTA SONDA MEDIU O NADA EM DOIS DOS TRÊS BLOCOS, e foram os CONTROLOS que o
# disseram — fica registado porque a cura é o método:
#   * L1: o irmão NO MUNDO lia a mesma posição com a câmera em três sítios ⇒ a câmera não estava a
#     enquadrar nada (`make_current` erra quando se chama do `_initialize`, antes de a árvore estar
#     pronta). Um CONTROLO que não reproduz o fenómeno transforma o teste inteiro em vácuo.
#   * L2: a escala vinha IDÊNTICA para cinco tamanhos de janela ⇒ `Window.size` não pega numa
#     janela fantasma (headless). ⇒ a experiência INVERTE-SE: a janela fica fixa (é o que ela é) e
#     quem varia é a RESOLUÇÃO DE REFERÊNCIA, que é um campo e obedece sempre.
#   * L3: «carregar dentro e largar fora» disparava, porque eu nunca enviei um MOUSE MOTION — o
#     controlo nunca soube que o cursor tinha saído. O evento que falta mede outro programa.
#
# Blocos:
#   L1) IMUNIDADE: a transformação de canvas do VIEWPORT (que a câmera move) contra a de um
#       CanvasLayer (que ela não move). C0 = o CONTROLO: a do viewport TEM de mudar.
#   L2) A ESCALA POR RESOLUÇÃO: janela fixa × content_scale_size variável × content_scale_aspect
#       ∈ {ignore, keep, keep_width, keep_height, expand} → escala e deslocamento finais.
#   L3) O CLIQUE: quando um Button dispara, com o MOUSE MOTION que torna o hover honesto.
extends SceneTree

var frame := 0
var cam: Camera2D
var layer: CanvasLayer
var botao: Button
var disparos := []

func _p(txt: String) -> void:
	print(txt)

# ---------------------------------------------------------------- L1: imunidade
func l1_imunidade() -> void:
	_p("\n### L1 — IMUNIDADE À CÂMERA (com o CONTROLO ao lado)")
	var vp := get_root()
	for c in [Vector2(0, 0), Vector2(200, 0), Vector2(200, 120)]:
		cam.position = c
		# ⚠️ a câmera só reescreve a transformação do canvas no fim do quadro dela
		cam.force_update_scroll()
		var v: Transform2D = vp.get_canvas_transform()      # CONTROLO: tem de mexer
		var l: Transform2D = layer.get_final_transform()    # o canvas: não pode mexer
		_p("camera=%-14s viewport=(%.1f, %.1f)  canvas_layer=(%.1f, %.1f)"
			% [str(c), v.origin.x, v.origin.y, l.origin.x, l.origin.y])

# ---------------------------------------------- L2: a escala por resolução
func l2_escala() -> void:
	_p("\n### L2 — A ESCALA POR RESOLUÇÃO (janela FIXA, referência VARIÁVEL)")
	var w := get_root()
	w.content_scale_mode = Window.CONTENT_SCALE_MODE_CANVAS_ITEMS
	var janela: Vector2i = w.size
	_p("janela (fixa, é a que o headless dá) = %s" % str(janela))
	var aspectos := {
		"ignore": Window.CONTENT_SCALE_ASPECT_IGNORE,
		"keep": Window.CONTENT_SCALE_ASPECT_KEEP,
		"keep_width": Window.CONTENT_SCALE_ASPECT_KEEP_WIDTH,
		"keep_height": Window.CONTENT_SCALE_ASPECT_KEEP_HEIGHT,
		"expand": Window.CONTENT_SCALE_ASPECT_EXPAND,
	}
	# referências com aspecto MAIOR, IGUAL e MENOR que o da janela
	var refs := [Vector2i(320, 180), Vector2i(640, 360), Vector2i(1280, 360), Vector2i(320, 480), Vector2i(500, 400)]
	for nome in aspectos:
		w.content_scale_aspect = aspectos[nome]
		for r in refs:
			w.content_scale_size = r
			var t: Transform2D = w.get_final_transform()
			var esperado_x := float(janela.x) / float(r.x)
			var esperado_y := float(janela.y) / float(r.y)
			_p("aspect=%-11s ref=%-11s escala=(%.6f, %.6f) desloc=(%.6f, %.6f)   [j/ref=(%.4f, %.4f)]"
				% [nome, str(r), t.x.x, t.y.y, t.origin.x, t.origin.y, esperado_x, esperado_y])

# ------------------------------------------------------------- L3: o clique
func _mv(pos: Vector2) -> void:
	var e := InputEventMouseMotion.new()
	e.position = pos
	e.global_position = pos
	get_root().push_input(e)

func _ev(tipo: String, pos: Vector2, carregado: bool) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = carregado
	e.position = pos
	e.global_position = pos
	get_root().push_input(e)
	_p("  %s em (%.0f, %.0f) [quadro %d] -> disparos=%s" % [tipo, pos.x, pos.y, frame, str(disparos)])

func _initialize() -> void:
	var mundo := Node2D.new()
	get_root().add_child(mundo)
	cam = Camera2D.new()
	mundo.add_child(cam)
	layer = CanvasLayer.new()
	get_root().add_child(layer)
	botao = Button.new()
	botao.position = Vector2(10, 10)
	botao.size = Vector2(80, 30)
	botao.pressed.connect(func(): disparos.append(frame))
	# ⭐ DENTRO do CanvasLayer: e' isso que um HUD e', e a 2.ª passagem provou-o pelo avesso —
	# com o botao no root o L3 deu ZERO disparos, porque a camera desloca o canvas do viewport.
	layer.add_child(botao)

func _process(_d: float) -> bool:
	frame += 1
	match frame:
		1:
			cam.enabled = true
			cam.make_current()
			l1_imunidade()
			l2_escala()
			# ⚠️ o L2 deixou a referencia no ultimo caso; sem este neutro o clique entra
			# transformado e mede outro programa.
			get_root().content_scale_size = get_root().size
			get_root().content_scale_aspect = Window.CONTENT_SCALE_ASPECT_IGNORE
			_p("\n### L3 — QUANDO UM BOTÃO DISPARA (com MOUSE MOTION, botao DENTRO do canvas)")
			_p("(a) carregar DENTRO e largar DENTRO")
		2:
			_mv(Vector2(50, 25)); _ev("down", Vector2(50, 25), true)
		3:
			_ev("up  ", Vector2(50, 25), false)
		4:
			_p("(b) carregar DENTRO, MOVER para fora, largar FORA")
			_mv(Vector2(50, 25)); _ev("down", Vector2(50, 25), true)
		5:
			_mv(Vector2(300, 300)); _ev("up  ", Vector2(300, 300), false)
		6:
			_p("(c) carregar FORA e largar DENTRO")
			_mv(Vector2(300, 300)); _ev("down", Vector2(300, 300), true)
		7:
			_mv(Vector2(50, 25)); _ev("up  ", Vector2(50, 25), false)
		8:
			_p("(d) desactivado: carregar e largar DENTRO")
			botao.disabled = true
			_mv(Vector2(50, 25)); _ev("down", Vector2(50, 25), true)
		9:
			_ev("up  ", Vector2(50, 25), false)
		11:
			_p("\n### RESUMO\ndisparos nos quadros: %s" % str(disparos))
			return true
	return false
