extends SceneTree
# Oráculo Godot da Luz 2D (W5e, Bloco C). O Godot é CORRIDO sobre cenas NOSSAS
# (sementes fixas), nunca lido. Cenas: «Luz 2D estado da arte.md», plano de medição S1–S8.
# Cada configuração rende numa SubViewport do tamanho pedido (UPDATE_ALWAYS); a janela
# só mostra a textura dela. Tempo = RenderingServer.viewport_get_measured_render_time_*
# da SubViewport. Rodadas intercaladas e rodadas (início deslocado), resumo = o MÍNIMO.
# Saída em stdout, linhas META| · PROVA| · MEDIDA| (corre.sh grava a fixture).
# Argumentos (depois de `--`): --rodadas=N --aquecer=N --quadros=N --fotos=0|1

const BASE := Vector2i(1920, 1080)
const SEMENTE := 20261011
const CINZA := Color(0.25, 0.25, 0.25)

var rodadas := 3
var aquecer := 20
var quadros := 60
var fotos := true
var _vista: TextureRect
var _tex_branca: ImageTexture
var _mat_emissivo: CanvasItemMaterial


func _initialize() -> void:
	for a in OS.get_cmdline_user_args():
		var kv := a.trim_prefix("--").split("=")
		if kv.size() != 2:
			continue
		match kv[0]:
			"rodadas": rodadas = int(kv[1])
			"aquecer": aquecer = int(kv[1])
			"quadros": quadros = int(kv[1])
			"fotos": fotos = kv[1] != "0"
	_corre()


func _corre() -> void:
	await process_frame
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	OS.low_processor_usage_mode = false
	var img := Image.create(8, 8, false, Image.FORMAT_RGBA8)
	img.fill(Color.WHITE)
	_tex_branca = ImageTexture.create_from_image(img)
	_mat_emissivo = CanvasItemMaterial.new()
	_mat_emissivo.light_mode = CanvasItemMaterial.LIGHT_MODE_UNSHADED
	_vista = TextureRect.new()
	_vista.set_anchors_preset(Control.PRESET_FULL_RECT)
	_vista.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	_vista.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	root.add_child(_vista)
	_meta()

	# 1) PROVA — quantas luzes o Godot ACENDE de facto (antes de cronometrar).
	print("COLUNAS|PROVA|fundo|sombra|n_luzes|acesas|sombra_efectiva|celulas_fora_acesas")
	for fundo in ["um_retangulo", "retangulo_por_celula"]:
		for sombra in [false, true]:
			for n in [1, 10, 15, 16, 17, 100, 255, 256, 257, 500, 1000, 10000]:
				await _prova(fundo, sombra, n)

	# 2) MEDIDA — rodadas intercaladas, cada uma começa noutro ponto da lista.
	var cfgs := _configs()
	var amostras := {}
	var min_rodada := {}
	for c in cfgs:
		amostras[_chave(c)] = {"gpu": [], "cpu": [], "parede": []}
		min_rodada[_chave(c)] = []
	for r in rodadas:
		var desloc := int(r * cfgs.size() / float(rodadas))
		for i in cfgs.size():
			var c: Dictionary = cfgs[(i + desloc) % cfgs.size()]
			var m: Dictionary = await _mede(c, r == 0 and fotos)
			var a: Dictionary = amostras[_chave(c)]
			a.gpu.append_array(m.gpu)
			a.cpu.append_array(m.cpu)
			a.parede.append_array(m.parede)
			min_rodada[_chave(c)].append(_min(m.gpu))
			printerr("MARCA|%s|rodada %d/%d|gpu_min %.4f ms" % [_chave(c), r + 1, rodadas, _min(m.gpu)])

	print("COLUNAS|MEDIDA|cena|n|variante|resolucao|amostras_gpu|gpu_min_ms|gpu_mediana_ms|gpu_media_ms|cpu_min_ms|cpu_mediana_ms|parede_media_ms|gpu_min_por_rodada")
	for c in cfgs:
		var a: Dictionary = amostras[_chave(c)]
		var por_rodada := PackedStringArray()
		for v in min_rodada[_chave(c)]:
			por_rodada.append("%.4f" % v)
		print("MEDIDA|%s|%d|%s|%dx%d|%d|%.4f|%.4f|%.4f|%.4f|%.4f|%.3f|%s" % [
			c.cena, c.n, c.var, c.res.x, c.res.y, _validos(a.gpu).size(),
			_min(a.gpu), _mediana(a.gpu), _media(a.gpu),
			_min(a.cpu), _mediana(a.cpu), _media(a.parede), "/".join(por_rodada)])
	print("FIM|ok")
	quit(0)


func _meta() -> void:
	print("META|godot|%s" % Engine.get_version_info().string)
	print("META|adaptador|%s" % RenderingServer.get_video_adapter_name())
	print("META|fabricante|%s" % RenderingServer.get_video_adapter_vendor())
	print("META|api|%s" % RenderingServer.get_video_adapter_api_version())
	print("META|tipo_adaptador|%d" % RenderingServer.get_video_adapter_type())
	print("META|driver_render|%s" % RenderingServer.get_current_rendering_driver_name())
	print("META|metodo_render|%s" % RenderingServer.get_current_rendering_method())
	print("META|servidor_janela|%s" % DisplayServer.get_name())
	print("META|vsync|%d" % DisplayServer.window_get_vsync_mode())
	print("META|rodadas|%d|aquecer|%d|quadros|%d|semente|%d" % [rodadas, aquecer, quadros, SEMENTE])
	for p in ProjectSettings.get_property_list():
		var nome: String = p.name
		if nome.begins_with("rendering/2d/") or nome.begins_with("rendering/limits/") \
				or nome.ends_with("_2d"):
			print("META|ps|%s|%s" % [nome, str(ProjectSettings.get_setting(nome))])


func _configs() -> Array:
	var c := []
	for s in [false, true]:
		c.append({"cena": "S1", "n": 1, "var": _sv(s), "sombra": s, "res": BASE})
	for n in [1, 10, 100, 1000, 10000]:
		for s in [false, true]:
			c.append({"cena": "S2", "n": n, "var": _sv(s), "sombra": s, "res": BASE})
	for k in [0, 10, 100, 1000]:
		c.append({"cena": "S3", "n": k, "var": "com_sombra", "sombra": true, "res": BASE})
	c.append({"cena": "S6", "n": 100, "var": "pan_subpixel", "sombra": false, "res": BASE})
	for r in [Vector2i(1280, 720), BASE, Vector2i(2560, 1440), Vector2i(3840, 2160)]:
		c.append({"cena": "S7", "n": 100, "var": "sem_sombra", "sombra": false, "res": r})
	return c


func _sv(s: bool) -> String:
	return "com_sombra" if s else "sem_sombra"


func _chave(c: Dictionary) -> String:
	return "%s_n%d_%s_%dx%d" % [c.cena, c.n, c.var, c.res.x, c.res.y]


func _tex_luz(px: int) -> GradientTexture2D:
	var g := Gradient.new()
	g.set_color(0, Color.WHITE)
	g.set_color(1, Color.BLACK)
	var t := GradientTexture2D.new()
	t.gradient = g
	t.width = px
	t.height = px
	t.fill = GradientTexture2D.FILL_RADIAL
	t.fill_from = Vector2(0.5, 0.5)
	t.fill_to = Vector2(1.0, 0.5)
	return t


func _viewport(res: Vector2i) -> SubViewport:
	var sv := SubViewport.new()
	sv.size = res
	if res != BASE:
		sv.size_2d_override = BASE
		sv.size_2d_override_stretch = true
	sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	sv.transparent_bg = false
	sv.disable_3d = true
	root.add_child(sv)
	return sv


func _retangulo(pai: Node, pos: Vector2, tam: Vector2) -> void:
	var r := ColorRect.new()
	r.position = pos
	r.size = tam
	r.color = CINZA
	pai.add_child(r)


func _luz(pai: Node, pos: Vector2, tex: Texture2D, sombra: bool, escala := 1.0) -> PointLight2D:
	var l := PointLight2D.new()
	l.texture = tex
	l.texture_scale = escala
	l.position = pos
	l.color = Color(1.0, 0.85, 0.6)
	l.energy = 1.0
	l.shadow_enabled = sombra
	pai.add_child(l)
	return l


func _emissivo(pai: Node, pos: Vector2) -> void:
	var s := Sprite2D.new()
	s.texture = _tex_branca
	s.material = _mat_emissivo
	s.modulate = Color(1.0, 0.85, 0.6)
	s.position = pos
	pai.add_child(s)


func _oclusor(pai: Node, pos: Vector2, pontos: PackedVector2Array, rot := 0.0) -> void:
	var p := OccluderPolygon2D.new()
	p.polygon = pontos
	var o := LightOccluder2D.new()
	o.occluder = p
	o.position = pos
	o.rotation = rot
	pai.add_child(o)


func _monta(c: Dictionary) -> Dictionary:
	var sv := _viewport(c.res)
	var rng := RandomNumberGenerator.new()
	rng.seed = SEMENTE
	var cam: Camera2D = null
	_retangulo(sv, Vector2.ZERO, Vector2(BASE))
	match c.cena:
		"S1":
			var spr := Sprite2D.new()
			spr.texture = _tex_branca
			spr.scale = Vector2(16, 16)
			spr.modulate = Color(0.3, 0.6, 0.9)
			spr.position = Vector2(1100, 540)
			sv.add_child(spr)
			_oclusor(sv, spr.position, PackedVector2Array([
				Vector2(-64, -64), Vector2(64, -64), Vector2(64, 64), Vector2(-64, 64)]))
			_luz(sv, Vector2(760, 540), _tex_luz(1024), c.sombra)
		"S2", "S6", "S7":
			var tex := _tex_luz(96)
			for i in c.n:
				var p := Vector2(rng.randf() * BASE.x, rng.randf() * BASE.y)
				_emissivo(sv, p)
				_luz(sv, p, tex, c.sombra)
			if c.cena == "S6":
				cam = Camera2D.new()
				cam.position = Vector2(BASE) * 0.5
				sv.add_child(cam)
				cam.make_current()
		"S3":
			_luz(sv, Vector2(BASE) * 0.5, _tex_luz(2048), true, 1.1)
			var centro := Vector2(BASE) * 0.5
			var feitos := 0
			while feitos < c.n:
				var p := Vector2(rng.randf() * BASE.x, rng.randf() * BASE.y)
				if p.distance_to(centro) < 64.0:
					continue
				var h := rng.randf_range(4.0, 16.0)
				var w := rng.randf_range(4.0, 16.0)
				_oclusor(sv, p, PackedVector2Array([
					Vector2(-w, -h), Vector2(w, -h), Vector2(w, h), Vector2(-w, h)]),
					rng.randf() * TAU)
				feitos += 1
	_vista.texture = sv.get_texture()
	return {"sv": sv, "cam": cam}


func _mede(c: Dictionary, fotografa: bool) -> Dictionary:
	var m := _monta(c)
	var sv: SubViewport = m.sv
	var rid := sv.get_viewport_rid()
	RenderingServer.viewport_set_measure_render_time(rid, true)
	for i in aquecer:
		_anda(m)
		await process_frame
	var gpu := []
	var cpu := []
	var parede := []
	var t0 := Time.get_ticks_usec()
	for i in quadros:
		_anda(m)
		await process_frame
		var t1 := Time.get_ticks_usec()
		parede.append((t1 - t0) / 1000.0)
		t0 = t1
		gpu.append(RenderingServer.viewport_get_measured_render_time_gpu(rid))
		cpu.append(RenderingServer.viewport_get_measured_render_time_cpu(rid))
	if fotografa:
		var img := sv.get_texture().get_image()
		img.resize(480, 270, Image.INTERPOLATE_BILINEAR)
		img.save_png(ProjectSettings.globalize_path("res://fotos/%s.png" % _chave(c)))
	RenderingServer.viewport_set_measure_render_time(rid, false)
	_vista.texture = null
	sv.queue_free()
	await process_frame
	return {"gpu": gpu, "cpu": cpu, "parede": parede}


func _anda(m: Dictionary) -> void:
	if m.cam != null:
		m.cam.position += Vector2(0.37, 0.23)


# Grelha 100×100 de células de 40 px numa SubViewport 4000², luz de raio 16 px no
# centro de cada uma das N primeiras células: ACESA se o pixel central sobe acima do
# cinza de fundo. Um muro (oclusor 2×12 px) a +5 px; SOMBRA EFECTIVA se, acesa, o pixel
# a +10 px fica no cinza (sem sombra ele acende: o controlo é a variante sem_sombra).
# Sem sobreposição ⇒ conta-se luz a luz.
func _prova(fundo: String, sombra: bool, n: int) -> void:
	const CEL := 40
	const LADO := 100
	var sv := SubViewport.new()
	sv.size = Vector2i(CEL * LADO, CEL * LADO)
	sv.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	sv.disable_3d = true
	root.add_child(sv)
	if fundo == "um_retangulo":
		_retangulo(sv, Vector2.ZERO, Vector2(sv.size))
	else:
		for i in LADO * LADO:
			_retangulo(sv, Vector2((i % LADO) * CEL, (i / LADO) * CEL), Vector2(CEL, CEL))
	var tex := _tex_luz(32)
	var muro := PackedVector2Array([Vector2(-1, -6), Vector2(1, -6), Vector2(1, 6), Vector2(-1, 6)])
	for i in n:
		var centro := Vector2((i % LADO) * CEL + CEL / 2, (i / LADO) * CEL + CEL / 2)
		_luz(sv, centro, tex, sombra)
		_oclusor(sv, centro + Vector2(5, 0), muro)
	_vista.texture = sv.get_texture()
	for i in 6:
		await process_frame
	var img := sv.get_texture().get_image()
	var acesas := 0
	var sombreadas := 0
	var fora := 0
	for i in LADO * LADO:
		var x := (i % LADO) * CEL + CEL / 2
		var y := (i / LADO) * CEL + CEL / 2
		if img.get_pixel(x, y).r > CINZA.r + 0.05:
			if i < n:
				acesas += 1
				if img.get_pixel(x + 10, y).r <= CINZA.r + 0.02:
					sombreadas += 1
			else:
				fora += 1
	print("PROVA|%s|%s|%d|%d|%d|%d" % [fundo, _sv(sombra), n, acesas, sombreadas, fora])
	printerr("MARCA|PROVA_%s_%s_n%d" % [fundo, _sv(sombra), n])
	_vista.texture = null
	sv.queue_free()
	await process_frame


func _validos(a: Array) -> Array:
	return a.filter(func(v): return v > 0.0)


func _min(a: Array) -> float:
	var b := _validos(a)
	return b.min() if not b.is_empty() else -1.0


func _mediana(a: Array) -> float:
	var b := _validos(a)
	if b.is_empty():
		return -1.0
	b.sort()
	return b[int(b.size() / 2.0)]


func _media(a: Array) -> float:
	var b := _validos(a)
	if b.is_empty():
		return -1.0
	var s := 0.0
	for v in b:
		s += v
	return s / b.size()
