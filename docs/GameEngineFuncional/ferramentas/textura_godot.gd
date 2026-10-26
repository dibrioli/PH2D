# Oráculo do plano 12 (CLAUDE.md §0.9): o que a Godot faz quando se TROCA a textura de um objecto. Corre-se,
# nunca se lê o fonte:
#
#     godot --headless -s docs/GameEngineFuncional/ferramentas/textura_godot.gd -- <pasta_temporaria>
#
# Imprime uma linha JSON por caso. As imagens são nossas (geradas aqui, cores lisas) e ficam na pasta dada.
extends SceneTree

func png(dir: String, nome: String, w: int, h: int, cor: Color) -> Texture2D:
	var img = Image.create(w, h, false, Image.FORMAT_RGBA8)
	img.fill(cor)
	var caminho = dir.path_join(nome)
	img.save_png(caminho)
	return ImageTexture.create_from_image(Image.load_from_file(caminho))

func diz(caso: String, d: Dictionary):
	d["caso"] = caso
	print(JSON.stringify(d))

func _initialize():
	var args = OS.get_cmdline_user_args()
	var dir = args[0] if args.size() > 0 else OS.get_cache_dir()
	DirAccess.make_dir_recursive_absolute(dir)
	var folha_a = png(dir, "folha_a.png", 256, 128, Color(1, 0, 0))   # 8 × 4 de 32 px
	var folha_b = png(dir, "folha_b.png", 384, 128, Color(0, 1, 0))   # 12 × 4 de 32 px (outra largura)
	var montes = png(dir, "montes.png", 300, 100, Color(0, 0, 1))

	# 1. Sprite2D com grelha: trocar a textura mantém hframes/vframes/frame? O rect desenhado segue a textura?
	var s = Sprite2D.new()
	s.texture = folha_a
	s.hframes = 8
	s.vframes = 4
	s.frame = 13
	s.texture_repeat = CanvasItem.TEXTURE_REPEAT_ENABLED
	s.scale = Vector2(2, 2)
	var antes = s.get_rect()
	s.texture = folha_b
	diz("sprite_troca_folha", {
		"hframes": s.hframes, "vframes": s.vframes, "frame": s.frame,
		"rect_antes": [antes.size.x, antes.size.y], "rect_depois": [s.get_rect().size.x, s.get_rect().size.y],
		"scale": [s.scale.x, s.scale.y], "texture_repeat": s.texture_repeat,
	})

	# 2. Folha mais pequena que o frame: o frame é preso?
	var s2 = Sprite2D.new()
	s2.texture = folha_b
	s2.hframes = 12
	s2.vframes = 4
	s2.frame = 40
	s2.texture = folha_a
	diz("sprite_folha_menor_mantem_frame", {"hframes": s2.hframes, "frame": s2.frame})

	# 3. Parallax2D: o repeat_size acompanha a textura da filha? (não: é do nó)
	var par = Parallax2D.new()
	par.repeat_size = Vector2(300, 0)
	var camada = Sprite2D.new()
	camada.texture = montes
	par.add_child(camada)
	camada.texture = folha_a
	diz("parallax_repeat_size_apos_troca", {"repeat_size": [par.repeat_size.x, par.repeat_size.y],
		"scroll_scale": [par.scroll_scale.x, par.scroll_scale.y]})

	# 4. Gravar a cena: a textura vai como REFERÊNCIA (ext_resource) e duas sprites com a mesma imagem
	#    partilham UMA entrada?
	var raiz = Node2D.new()
	raiz.name = "Nivel"
	var t_disco = load(dir.path_join("montes.png")) if ResourceLoader.exists(dir.path_join("montes.png")) else null
	var a = Sprite2D.new()
	a.name = "A"
	var b = Sprite2D.new()
	b.name = "B"
	var tex = ImageTexture.create_from_image(Image.load_from_file(dir.path_join("montes.png")))
	tex.resource_path = dir.path_join("montes_tex.tres")
	ResourceSaver.save(tex, dir.path_join("montes_tex.tres"))
	var tex_ref = load(dir.path_join("montes_tex.tres"))
	a.texture = tex_ref
	b.texture = tex_ref
	raiz.add_child(a)
	raiz.add_child(b)
	a.owner = raiz
	b.owner = raiz
	var cena = PackedScene.new()
	cena.pack(raiz)
	var tscn = dir.path_join("nivel.tscn")
	ResourceSaver.save(cena, tscn)
	var texto = FileAccess.get_file_as_string(tscn)
	diz("cena_grava_referencia", {
		"ext_resources": texto.count("[ext_resource"),
		"sub_resources": texto.count("[sub_resource"),
		"bytes_tscn": texto.length(),
		"usa_ExtResource": texto.count("ExtResource("),
		"carregar_2x_mesma_instancia": load(dir.path_join("montes_tex.tres")) == tex_ref,
		"disco_png_e_recurso": t_disco != null,
	})

	# 5. Conteúdo partilhado: mudar os pixels do recurso muda as DUAS sprites (é o mesmo objecto).
	diz("conteudo_partilhado", {"a_e_b_mesma_textura": a.texture == b.texture})
	raiz.free()
	par.free()
	s.free()
	s2.free()
	quit()
