# Oráculo do Espelhar da Sprite (CLAUDE.md §0.9): corre-se, nunca se lê o fonte. Par: espelhar_unity.cs.
#
#     PH2D_GPU=1 bash scripts/ph2d-run.sh xvfb-run -a godot --rendering-driver opengl3 \
#         -s docs/GameEngineFuncional/ferramentas/espelhar_godot.gd
#
# Imagem nossa de 64 × 32 px (metade esquerda vermelha, direita verde), 32 px = 1 unidade, objecto na origem,
# filho em (3, 0), colisão 1 × 1 com centro em (0,5; 0). Unidades com y para CIMA, como no par da Unity.
# Imprime `[EXP]` + uma linha JSON por caso, com o que saiu nos píxeis, não o que o campo diz.
extends SceneTree

const PX := 32.0
const W := 512
const H := 288

func centro(soma: Vector2, n: int):
	if n == 0:
		return null
	var c = soma / n
	return [snappedf((c.x - W / 2.0) / PX, 0.001), snappedf((H / 2.0 - c.y) / PX, 0.001)]

func le(img: Image) -> Dictionary:
	var sr := Vector2.ZERO
	var sg := Vector2.ZERO
	var nr := 0
	var ng := 0
	var caixa := Rect2i()
	var tem := false
	for y in H:
		for x in W:
			var c := img.get_pixel(x, y)
			if c.a < 0.5:
				continue
			var p := Vector2(x + 0.5, y + 0.5)
			if c.r > c.g:
				sr += p
				nr += 1
			else:
				sg += p
				ng += 1
			var r := Rect2i(x, y, 1, 1)
			caixa = caixa.merge(r) if tem else r
			tem = true
	var d := {"verm": centro(sr, nr), "verde": centro(sg, ng)}
	if tem:
		d["caixa"] = [
			snappedf((caixa.position.x - W / 2.0) / PX, 0.001),
			snappedf((H / 2.0 - caixa.end.y) / PX, 0.001),
			snappedf((caixa.end.x - W / 2.0) / PX, 0.001),
			snappedf((H / 2.0 - caixa.position.y) / PX, 0.001),
		]
	return d

func _initialize():
	correr()

func correr():
	await process_frame
	var img := Image.create(64, 32, false, Image.FORMAT_RGBA8)
	img.fill_rect(Rect2i(0, 0, 32, 32), Color(1, 0, 0))
	img.fill_rect(Rect2i(32, 0, 32, 32), Color(0, 1, 0))
	var tex := ImageTexture.create_from_image(img)

	var sv := SubViewport.new()
	sv.size = Vector2i(W, H)
	sv.transparent_bg = true
	sv.own_world_3d = true
	sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	sv.canvas_item_default_texture_filter = Viewport.DEFAULT_CANVAS_ITEM_TEXTURE_FILTER_NEAREST
	root.add_child(sv)
	sv.canvas_transform = Transform2D(0.0, Vector2(W / 2.0, H / 2.0))

	# Pivô = onde a origem do objecto fica na imagem; no Godot exprime-se pelo offset.
	var pivos := {"centro": Vector2.ZERO, "esquerda": Vector2(32, 0), "pe": Vector2(0, -16)}
	var modos := ["nada", "flip_x", "escala_x", "flip_e_escala_x", "flip_y", "escala_y"]

	for pivo in pivos:
		for modo in modos:
			for rot in [0.0, 30.0]:
				var corpo := StaticBody2D.new()
				var s := Sprite2D.new()
				s.texture = tex
				s.offset = pivos[pivo]
				corpo.add_child(s)
				var forma := CollisionShape2D.new()
				var caixa := RectangleShape2D.new()
				caixa.size = Vector2(PX, PX)
				forma.shape = caixa
				forma.position = Vector2(0.5 * PX, 0)
				corpo.add_child(forma)
				var filho := Node2D.new()
				filho.position = Vector2(3 * PX, 0)
				corpo.add_child(filho)
				sv.add_child(corpo)
				corpo.rotation_degrees = -rot
				match modo:
					"flip_x":
						s.flip_h = true
					"escala_x":
						corpo.scale.x = -corpo.scale.x
					"flip_e_escala_x":
						s.flip_h = true
						corpo.scale.x = -corpo.scale.x
					"flip_y":
						s.flip_v = true
					"escala_y":
						corpo.scale.y = -corpo.scale.y
				await process_frame
				await physics_frame
				await physics_frame
				RenderingServer.force_draw(false)
				var d := le(sv.get_texture().get_image())
				var espaco := sv.find_world_2d().direct_space_state
				var toca := func(x: float) -> bool:
					var q := PhysicsPointQueryParameters2D.new()
					q.position = Vector2(x * PX, 0)
					q.collide_with_bodies = true
					return espaco.intersect_point(q).size() > 0
				d["motor"] = "godot"
				d["pivo"] = pivo
				d["modo"] = modo
				d["rot"] = rot
				d["filho"] = [snappedf(filho.global_position.x / PX, 0.001), snappedf(-filho.global_position.y / PX, 0.001)]
				d["colisao"] = [snappedf(forma.global_position.x / PX, 0.001), snappedf(-forma.global_position.y / PX, 0.001)]
				d["toca_+0,5"] = toca.call(0.5)
				d["toca_-0,5"] = toca.call(-0.5)
				d["campo_escala"] = [snappedf(corpo.scale.x, 0.001), snappedf(corpo.scale.y, 0.001)]
				d["campo_rot"] = snappedf(corpo.rotation_degrees, 0.001)
				print("[EXP] ", JSON.stringify(d))
				sv.remove_child(corpo)
				corpo.free()
	quit()
