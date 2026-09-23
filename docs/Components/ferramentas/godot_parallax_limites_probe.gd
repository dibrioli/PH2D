# ⚠️ As duas leis anteriores foram REFUTADAS porque eu media um DECLIVE MÉDIO sobre uma curva que
# tem JOELHOS: um clamp não tem um declive, tem pedaços. Aqui despeja-se a curva inteira.
extends SceneTree
var cam: Camera2D
var px: Parallax2D
func _p(s): print(s)

func _init() -> void:
	await process_frame
	var raiz := Node2D.new(); root.add_child(raiz)
	cam = Camera2D.new(); cam.anchor_mode = Camera2D.ANCHOR_MODE_DRAG_CENTER
	raiz.add_child(cam); await process_frame; cam.make_current()
	px = Parallax2D.new(); px.scroll_scale = Vector2(0.5, 0.5); raiz.add_child(px)
	await process_frame
	var ecra: float = root.get_visible_rect().size.x
	_p("ecrã %.0f · scroll_scale 0.5 · região −600..600 (1200 de largura)" % ecra)
	px.limit_begin = Vector2(-600, -1e7)
	px.limit_end = Vector2(600, 1e7)
	var ant := INF
	var ox := 0.0
	_p("%-9s %-12s %-10s" % ["cam.x", "origem.x", "declive"])
	for i in range(-10, 11):
		var x: float = i * 120.0
		cam.position = Vector2(x, 0)
		await process_frame; await process_frame
		var o: float = px.transform.origin.x
		var d: String = "—" if is_inf(ant) else "%+.4f" % ((o - ant) / 120.0)
		_p("%-9.0f %-12.1f %-10s" % [x, o, d])
		ant = o
	quit()
