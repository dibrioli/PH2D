# P14d-2: onde o Godot DESENHA o pivô de cada quadro. Um AnimatedSprite2D (centered=false, offset =
# metadata/ph2d_offset do NOSSO heroi.tres) no nó (64, 64), parado em cada quadro; render real (xvfb,
# opengl3), fundo preto. A translação T do desenho mede-se pela caixa dos píxeis acesos contra a do
# nosso cor<q>.rgba; o pivô (meta.pivots do nosso JSON) cai em T + pivô. Imprime o desvio ao nó.
# Uso: xvfb-run -a godot --rendering-driver opengl3 --path <pasta> --script <este>
extends SceneTree

const NO = Vector2i(64, 64)
var spr
var sf
var alvos = []
var q = 0
var passos = 0
var pivos
var saida = []

func caixa_nossa(cor):
	var mn = Vector2i(99, 99)
	for y in range(16):
		for x in range(16):
			if cor[(y * 16 + x) * 4 + 3] > 127:
				mn = Vector2i(min(mn.x, x), min(mn.y, y))
	return mn

func caixa_dele(img):
	var mn = Vector2i(9999, 9999)
	for y in range(img.get_height()):
		for x in range(img.get_width()):
			if img.get_pixel(x, y).get_luminance() > 0.1:
				mn = Vector2i(min(mn.x, x), min(mn.y, y))
	return mn

func _init():
	root.size = Vector2i(128, 128)
	RenderingServer.set_default_clear_color(Color(0, 0, 0, 1))
	sf = load("res://heroi.tres")
	var j = JSON.parse_string(FileAccess.get_file_as_string("res://heroi.json"))
	pivos = j["meta"]["pivots"]
	var regioes = {}
	var i = 0
	for e in j["frames"]:
		var pad = (int(e["frame"]["w"]) - int(e["spriteSourceSize"]["w"])) / 2
		regioes[Vector2i(int(e["frame"]["x"]) + pad, int(e["frame"]["y"]) + pad)] = i
		i += 1
	for k in range(i):
		alvos.append(null)
	for a in sf.get_animation_names():
		for k in range(sf.get_frame_count(a)):
			var x = regioes.get(Vector2i(sf.get_frame_texture(a, k).region.position), -1)
			if x >= 0 and alvos[x] == null:
				alvos[x] = [a, k]
	spr = AnimatedSprite2D.new()
	spr.sprite_frames = sf
	spr.centered = false
	spr.offset = sf.get_meta("ph2d_offset")
	spr.position = Vector2(NO)
	spr.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	root.add_child(spr)
	mostra()

func mostra():
	spr.animation = alvos[q][0]
	spr.frame = alvos[q][1]
	spr.pause()

func _process(_d):
	passos += 1
	if passos < 6:
		return false
	var img = root.get_texture().get_image()
	img.convert(Image.FORMAT_RGBA8)
	var t = caixa_dele(img) - caixa_nossa(FileAccess.get_file_as_bytes("res://cor%d.rgba" % q))
	var p = Vector2(float(pivos[q]["x"]), float(pivos[q]["y"]))
	var cai = Vector2(t) + p
	saida.append({"quadro": q, "dx": cai.x - NO.x, "dy": cai.y - NO.y})
	q += 1
	passos = 0
	if q >= alvos.size():
		print("PIVO ", JSON.stringify(saida))
		quit(0)
		return true
	mostra()
	return false
