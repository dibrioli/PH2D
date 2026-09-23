# ORÁCULO DA PARALAXE — Godot 4.7.2 (MIT). Corre-se, não se lê.
#
# ⚠️ A 1.ª redacção mediu POSIÇÕES ABSOLUTAS e devolveu quatro células a dizer «não mexeu» —
# indistinguível de «a minha sonda não disparou o recálculo». O `Parallax2D` só recalcula quando a
# transformada do viewport MUDA, logo mexer num knob com a câmara parada não se vê.
# ⇒ esta redacção mede o DECLIVE (Δorigem da camada ÷ Δcâmara), que é a própria paralaxe, e leva
# CONTROLO: `scroll_scale = 1` tem de dar declive 1, e `0` tem de dar 0.
#
# godot --headless --quit-after 900 --script <este> -- <saida.txt>
extends SceneTree

var _out := ""
var cam: Camera2D
var px: Parallax2D

func _p(s: String) -> void:
	_out += s + "\n"
	print(s)

## Move a câmara de `a` para `b` e devolve (Δorigem do Parallax2D) / (Δcâmara).
func _declive(a: Vector2, b: Vector2) -> Vector2:
	cam.position = a
	await process_frame
	await process_frame
	var o0: Vector2 = px.transform.origin
	cam.position = b
	await process_frame
	await process_frame
	var o1: Vector2 = px.transform.origin
	var d := b - a
	return Vector2(
		(o1.x - o0.x) / d.x if absf(d.x) > 0.001 else 0.0,
		(o1.y - o0.y) / d.y if absf(d.y) > 0.001 else 0.0)

func _init() -> void:
	await process_frame
	var raiz := Node2D.new()
	root.add_child(raiz)
	cam = Camera2D.new()
	cam.anchor_mode = Camera2D.ANCHOR_MODE_DRAG_CENTER
	raiz.add_child(cam)
	await process_frame
	cam.make_current()
	px = Parallax2D.new()
	raiz.add_child(px)
	await process_frame

	_p("viewport = %s   (a origem da camada com a câmara em 0 é -viewport/2)" % str(root.get_visible_rect().size))
	_p("")
	_p("=== A LEI: declive = Δorigem da camada ÷ Δcâmara, com a câmara a andar 400 px em x ===")
	_p("%-16s %-18s %s" % ["scroll_scale", "declive medido", "veredito"])
	for s in [0.0, 0.25, 0.5, 1.0, 2.0]:
		px.scroll_scale = Vector2(s, s)
		var d: Vector2 = await _declive(Vector2(-200, 0), Vector2(200, 0))
		var ok := "= scroll_scale" if absf(d.x - s) < 1e-4 else "⚠️ DIVERGE"
		_p("%-16s %-18s %s" % [str(s), "%.4f" % d.x, ok])
	px.scroll_scale = Vector2(0.5, 0.5)

	_p("")
	_p("=== ZOOM: o declive em MUNDO muda com o zoom? (scroll_scale 0.5) ===")
	for z in [1.0, 2.0, 0.5]:
		cam.zoom = Vector2(z, z)
		await process_frame
		var d: Vector2 = await _declive(Vector2(-200, 0), Vector2(200, 0))
		_p("zoom %-6s declive %.4f   (o viewport visível passa a %s)" % [str(z), d.x, str(root.get_visible_rect().size / z)])
	cam.zoom = Vector2.ONE

	_p("")
	_p("=== ROTAÇÃO da câmara (scroll_scale 0.5) ===")
	for r in [0.0, 0.5, 1.5707963]:
		cam.rotation = r
		await process_frame
		var d: Vector2 = await _declive(Vector2(-200, 0), Vector2(200, 0))
		_p("rot %-10s declive x %.4f  y %.4f" % ["%.3f rad" % r, d.x, d.y])
	cam.rotation = 0.0

	_p("")
	_p("=== EIXOS SEPARADOS: scroll_scale = (0.2, 0.8) ===")
	px.scroll_scale = Vector2(0.2, 0.8)
	var dx: Vector2 = await _declive(Vector2(-200, 0), Vector2(200, 0))
	var dy: Vector2 = await _declive(Vector2(0, -200), Vector2(0, 200))
	_p("câmara em X → declive x %.4f | câmara em Y → declive y %.4f" % [dx.x, dy.y])
	px.scroll_scale = Vector2(0.5, 0.5)

	_p("")
	_p("=== REPETIÇÃO: repeat_size 256, a câmara varre 0..768 ===")
	px.repeat_size = Vector2(256, 0)
	px.repeat_times = 3
	var ant := 0.0
	for x in [0, 128, 256, 384, 512, 640, 768]:
		cam.position = Vector2(x, 0)
		await process_frame
		await process_frame
		var o: float = px.transform.origin.x
		_p("  cam.x %-6d origem.x %-10.1f  salto desde a anterior %.1f" % [x, o, o - ant])
		ant = o
	px.repeat_size = Vector2.ZERO
	px.repeat_times = 1

	_p("")
	_p("=== AUTOSCROLL 60 px/s, câmara PARADA — e o CONTROLO é o mesmo teste com 0 ===")
	for a in [60.0, 0.0]:
		px.autoscroll = Vector2(a, 0)
		cam.position = Vector2.ZERO
		await process_frame
		await process_frame
		var o0: float = px.transform.origin.x
		for i in 30:
			await process_frame
		var o1: float = px.transform.origin.x
		_p("autoscroll %-6s  andou %.3f px em 30 quadros" % [str(a), o1 - o0])
	px.autoscroll = Vector2.ZERO

	_p("")
	_p("=== LIMITES: limit_begin/limit_end (região -400..400) ===")
	px.limit_begin = Vector2(-400, -1e7)
	px.limit_end = Vector2(400, 1e7)
	for x in [-1200, -400, 0, 400, 1200]:
		cam.position = Vector2(x, 0)
		await process_frame
		await process_frame
		_p("  cam.x %-8d origem.x %.1f" % [x, px.transform.origin.x])
	px.limit_begin = Vector2(-1e7, -1e7)
	px.limit_end = Vector2(1e7, 1e7)

	_p("")
	_p("=== OS DOIS INTERRUPTORES, medidos pelo DECLIVE ===")
	for a in [[true, false], [false, false], [true, true], [false, true]]:
		px.follow_viewport = a[0]
		px.ignore_camera_scroll = a[1]
		await process_frame
		var d: Vector2 = await _declive(Vector2(-200, 0), Vector2(200, 0))
		_p("follow_viewport=%-6s ignore_camera_scroll=%-6s declive %.4f" % [str(a[0]), str(a[1]), d.x])

	var args := OS.get_cmdline_user_args()
	if args.size() > 0:
		var f := FileAccess.open(args[0], FileAccess.WRITE)
		f.store_string(_out); f.close()
	quit()
