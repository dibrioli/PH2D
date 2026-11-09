# P14d: o SINAL da normal no Godot. O quadro 0 do NOSSO heroi.tres, ×8, PointLight2D à esquerda,
# render real (xvfb, opengl3); imprime o brilho médio das metades esquerda e direita do disco, com
# a normal e sem ela (o controlo: a mesma AtlasTexture sobre a folha de cor só).
# Uso: xvfb-run -a godot --rendering-driver opengl3 --path <pasta> --script <este>
extends SceneTree

var passos = 0
var fase = 0
var medidas = {}
var spr
var luz

func _init():
	root.size = Vector2i(256, 256)
	var sf = load("res://heroi.tres")
	spr = Sprite2D.new()
	spr.texture = sf.get_frame_texture("idle", 0)
	spr.scale = Vector2(8, 8)
	spr.position = Vector2(128, 128)
	spr.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	root.add_child(spr)
	var cm = CanvasModulate.new()
	cm.color = Color(0, 0, 0)
	root.add_child(cm)
	luz = PointLight2D.new()
	var gt = GradientTexture2D.new()
	gt.fill = GradientTexture2D.FILL_RADIAL
	gt.fill_from = Vector2(0.5, 0.5)
	gt.fill_to = Vector2(1.0, 0.5)
	gt.width = 512
	gt.height = 512
	var gr = Gradient.new()
	gr.colors = PackedColorArray([Color(1, 1, 1), Color(1, 1, 1)])
	gt.gradient = gr
	luz.texture = gt
	luz.position = Vector2(-40, 128)
	luz.texture_scale = 4.0
	luz.height = 0.0
	root.add_child(luz)

func mede():
	var img = root.get_texture().get_image()
	img.convert(Image.FORMAT_RGBA8)
	var e = 0.0
	var d = 0.0
	for y in range(96, 160):
		for x in range(72, 120):
			e += img.get_pixel(x, y).get_luminance() * 255.0
		for x in range(136, 184):
			d += img.get_pixel(x, y).get_luminance() * 255.0
	return [e / (64 * 48), d / (64 * 48)]

func _process(_d):
	passos += 1
	if passos < 10:
		return false
	var m = mede()
	if fase == 0:
		medidas["esquerda"] = m[0]
		medidas["direita"] = m[1]
		var at = spr.texture.duplicate()
		at.atlas = spr.texture.atlas.diffuse_texture
		spr.texture = at
		fase = 1
		passos = 0
		return false
	medidas["controlo_esquerda"] = m[0]
	medidas["controlo_direita"] = m[1]
	print("SINAL ", JSON.stringify(medidas))
	quit(0)
	return true
