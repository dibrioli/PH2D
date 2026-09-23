# O sistema ANTIGO (ParallaxBackground + ParallaxLayer) contra o NOVO, na MESMA régua.
# A pergunta: eles trocaram só o nó, ou também a PARAMETRIZAÇÃO?
# ⚠️ 0.5 não discrimina (1 − 0.5 = 0.5) — quem decide são o 0 e o 1.
extends SceneTree

var cam: Camera2D
var _out := ""
func _p(s): _out += s + "\n"; print(s)

func _declive(no, a: Vector2, b: Vector2) -> float:
	cam.position = a
	await process_frame; await process_frame
	var o0: float = no.get("scroll_offset").x if no is ParallaxBackground else no.transform.origin.x
	cam.position = b
	await process_frame; await process_frame
	var o1: float = no.get("scroll_offset").x if no is ParallaxBackground else no.transform.origin.x
	return (o1 - o0) / (b.x - a.x)

func _init() -> void:
	await process_frame
	var raiz := Node2D.new(); root.add_child(raiz)
	cam = Camera2D.new(); cam.anchor_mode = Camera2D.ANCHOR_MODE_DRAG_CENTER
	raiz.add_child(cam); await process_frame; cam.make_current()

	var bg := ParallaxBackground.new(); raiz.add_child(bg)
	var camada := ParallaxLayer.new(); bg.add_child(camada)
	var marca := Node2D.new(); camada.add_child(marca)
	await process_frame

	_p("=== ANTIGO: ParallaxLayer.motion_scale — declive da CAMADA ===")
	for s in [0.0, 0.25, 0.5, 1.0, 2.0]:
		camada.motion_scale = Vector2(s, s)
		cam.position = Vector2(-200, 0)
		await process_frame; await process_frame
		var o0: float = camada.transform.origin.x
		cam.position = Vector2(200, 0)
		await process_frame; await process_frame
		var d: float = (camada.transform.origin.x - o0) / 400.0
		_p("  motion_scale %-6s declive %.4f   (o NOVO, no mesmo ponto, dá %.4f)" % [str(s), d, 1.0 - s])
	quit()
