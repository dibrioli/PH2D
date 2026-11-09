# P14d: lê o NOSSO heroi.tres (res://), lista as animações, toca cada uma num AnimatedSprite2D a
# passos de 1 ms (--fixed-fps 1000) e imprime o quadro do sprite (pela região no JSON) em t = 0..2000.
# Uso: godot --headless --path <pasta> --fixed-fps 1000 --script <este>
extends SceneTree

class Gravador extends Node:
	var sprites = {}
	var logo_apos_play = {}
	var regioes = {}
	var seq = {}
	var t = 0
	func quadro(s):
		var tex = s.sprite_frames.get_frame_texture(s.animation, s.frame)
		return regioes.get(Vector2i(tex.region.position), -1)
	var progresso = {}
	func _process(_d):
		t += 1
		for k in sprites:
			seq[k].append(quadro(sprites[k]))
		# (b) o frame_progress do idle à volta da fronteira de 80 ms: soma de 0,0125 por passo.
		if t >= 78 and t <= 82 and sprites.has("idle"):
			progresso[str(t)] = [sprites["idle"].frame, "%.17f" % sprites["idle"].frame_progress]
		if t >= 2000:
			print("SEQ ", JSON.stringify(seq))
			print("PROGRESSO ", JSON.stringify(progresso))
			print("APOS_PLAY ", JSON.stringify(logo_apos_play))
			get_tree().quit(0)

func _init():
	var sf = load("res://heroi.tres")
	if sf == null:
		print("ABRE ", JSON.stringify({"erro": "load"}))
		quit(1)
		return
	var nomes = []
	for a in sf.get_animation_names():
		nomes.append(String(a))
	var tex0 = sf.get_frame_texture(nomes[0], 0)
	var nt = tex0.atlas is CanvasTexture and tex0.atlas.normal_texture != null
	print("ABRE ", JSON.stringify({"erro": null, "animacoes": nomes, "normal_texture": nt}))
	var j = JSON.parse_string(FileAccess.get_file_as_string("res://heroi.json"))
	var g = Gravador.new()
	g.process_priority = 1000
	var i = 0
	for q in j["frames"]:
		var pad = (int(q["frame"]["w"]) - int(q["spriteSourceSize"]["w"])) / 2
		g.regioes[Vector2i(int(q["frame"]["x"]) + pad, int(q["frame"]["y"]) + pad)] = i
		i += 1
	# O recorte da normal: a região de cada quadro na normal_texture, contra o nosso n<q>.rgba.
	var nimg = load("res://heroi_n.png").get_image()
	nimg.decompress()
	nimg.convert(Image.FORMAT_RGBA8)
	var rec = []
	for q in range(i):
		var t = sf.get_frame_texture(nomes[0], 0)
		var at = null
		for a in nomes:
			for k in range(sf.get_frame_count(a)):
				var x = sf.get_frame_texture(a, k)
				if g.regioes.get(Vector2i(x.region.position), -1) == q:
					at = x
		var nosso = FileAccess.get_file_as_bytes("res://n%d.rgba" % q)
		var cor = FileAccess.get_file_as_bytes("res://cor%d.rgba" % q)
		var dif = 0
		var comp = 0
		var r = at.region
		var m = at.margin.position
		for dy in range(int(r.size.y)):
			for dx in range(int(r.size.x)):
				var o = ((int(m.y) + dy) * 16 + int(m.x) + dx) * 4
				if cor[o + 3] == 0:
					continue
				comp += 1
				var c = nimg.get_pixel(int(r.position.x) + dx, int(r.position.y) + dy)
				var b = [c.r8, c.g8, c.b8, c.a8]
				if b != [nosso[o], nosso[o + 1], nosso[o + 2], nosso[o + 3]]:
					dif += 1
		rec.append({"quadro": q, "diferem": dif, "comparados": comp})
	print("RECORTE ", JSON.stringify(rec))
	for a in nomes:
		var s = AnimatedSprite2D.new()
		s.sprite_frames = sf
		root.add_child(s)
		s.play(a)
		g.sprites[a] = s
		g.seq[a] = [g.quadro(s)]
		g.logo_apos_play[a] = [s.frame, s.frame_progress]
	root.add_child(g)
