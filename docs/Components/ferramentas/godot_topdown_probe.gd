# ORÁCULO: o MOVER DE VISTA DE CIMA no Godot 4.7.2 (MIT), corrido sem interface — docs/Components/10_plano_topdown_player.md §1.
# Correr: godot --headless --script docs/Components/ferramentas/godot_topdown_probe.gd
#
# O alvo é o `CharacterBody2D` com `motion_mode = MOTION_MODE_FLOATING` — que é, na
# própria doc dele, «o modo de vista de cima» — mais `move_and_slide()`, que é a lei
# do DESLIZAR: o caso que a síntese chama de «o caso chato que trava iniciante».
#
# Mede, uma pergunta por bloco:
#   A) DESLIZAR contra uma parede: quanto do movimento tangencial sobrevive, tique a tique
#   B) a VARREDURA DO ÂNGULO: de rasante a frontal, quanto anda em UM tique (é aqui que
#      o `wall_min_slide_angle` aparece, se aparecer)
#   C) a QUINA interior (duas paredes a 90°): pára ou desliza?
#   D) a VELOCIDADE depois do deslize — o `move_and_slide` reescreve `velocity`; com quê?
#   E) FLOATING contra GROUNDED com as MESMAS entradas: o dropdown muda mesmo a semântica?
#   F) a PAREDE INCLINADA a 45°: a projecção conserva a rapidez ou só a componente?
#   G) o custo de N corpos a deslizar por tique
#
# ⚠️ Corre a partir do 1.º _process (a lição dos probes irmãos: antes disso os nós não
#    estão na árvore, e o 1.º _physics_process vem ANTES do 1.º _process).
# ⚠️ Cada bloco que precisa de ver o quadro SEGUINTE vive num frame próprio, de propósito.
# ⚠️ As unidades do Godot são PIXELS e a margem dele (`safe_margin`, 0,08 px por omissão)
#    NÃO é invariante à escala — por isso a sonda imprime a margem e corre a geometria em
#    números grandes (paredes a centenas de px), para o leitor poder decidir se ela conta.
extends SceneTree

const DT := 1.0 / 60.0          # o passo fixo de fábrica do Godot; impresso no cabeçalho
const RAIO := 16.0              # o corpo é um círculo — sem quinas próprias a confundir a leitura
const V := 240.0                # px/s: 4 px por tique, longe da margem de 0,08

var frame := 0
var root2d: Node2D
var corpo: CharacterBody2D
var forma: CollisionShape2D
var fase := 0
var fase_tick := 0
var registo := []


func _initialize():
	print("# ORACULO godot_topdown — CharacterBody2D FLOATING + move_and_slide")
	print("# godot: ", Engine.get_version_info().string)
	print("# dt=%.6f  raio=%.1fpx  V=%.1fpx/s (%.4f px/tique)" % [DT, RAIO, V, V * DT])
	print("# safe_margin (omissao) e max_slides sao impressos no bloco A")


func _parede(pos: Vector2, tam: Vector2) -> StaticBody2D:
	var b := StaticBody2D.new()
	b.position = pos
	var cs := CollisionShape2D.new()
	var r := RectangleShape2D.new()
	r.size = tam
	cs.shape = r
	b.add_child(cs)
	root2d.add_child(b)
	return b


func _novo_corpo(pos: Vector2) -> CharacterBody2D:
	var c := CharacterBody2D.new()
	c.position = pos
	c.motion_mode = CharacterBody2D.MOTION_MODE_FLOATING
	var cs := CollisionShape2D.new()
	var s := CircleShape2D.new()
	s.radius = RAIO
	cs.shape = s
	c.add_child(cs)
	root2d.add_child(c)
	return c


func _limpar():
	for f in root2d.get_children():
		root2d.remove_child(f)
		f.queue_free()


# ⚠️ No `MainLoop` do Godot, devolver TRUE **encerra a aplicacao** — a 1.ª redacao
# desta sonda devolvia `true` aqui e o programa saía depois do cabecalho, com zero
# blocos corridos e sem erro nenhum. Quem termina é o `quit()` do fim do bloco G.
func _process(_delta):
	if root2d == null:
		root2d = Node2D.new()
		get_root().add_child(root2d)
		return false
	frame += 1
	return false


func _physics_process(_delta):
	if root2d == null:
		return false
	match fase:
		0:
			_fase_a()
		1:
			_fase_b()
		2:
			_fase_c()
		3:
			_fase_d()
		4:
			_fase_e()
		5:
			_fase_f()
		6:
			_fase_h()
		7:
			_fase_i()
		8:
			_fase_g()
		_:
			return false
	return false


func _avanca():
	fase += 1
	fase_tick = 0
	registo.clear()
	_limpar()


# ─────────────────────────────────────────────────────────────────────────────
# A) DESLIZAR contra uma parede VERTICAL, empurrado na diagonal (45°).
#    A pergunta: a componente ao longo da parede sobrevive INTEIRA, ou é cortada?
func _fase_a():
	if fase_tick == 0:
		_parede(Vector2(0, 0), Vector2(200, 2000))       # parede vertical, face esquerda em x=-100
		corpo = _novo_corpo(Vector2(-200, 0))
		print("")
		print("## A) deslizar contra parede vertical, empurrado a 45 graus (dir=(1,1)/raiz2)")
		print("# max_slides=", corpo.max_slides, "  wall_min_slide_angle=%.6f rad (%.2f graus)" % [corpo.wall_min_slide_angle, rad_to_deg(corpo.wall_min_slide_angle)])
		print("# safe_margin=", corpo.safe_margin)
		print("# face da parede em x=-100; o corpo tem raio 16 => encosta em x=-116")
		print("# tique  x  y  vx_depois  vy_depois  on_wall")
	if fase_tick < 40:
		var dir := Vector2(1, 1).normalized()
		corpo.velocity = dir * V
		corpo.move_and_slide()
		if fase_tick < 12 or fase_tick == 39:
			print("A %d %.6f %.6f %.6f %.6f %s" % [fase_tick, corpo.position.x, corpo.position.y, corpo.velocity.x, corpo.velocity.y, corpo.is_on_wall()])
		fase_tick += 1
	else:
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# B) VARREDURA DO ÂNGULO — de 90° (paralelo à parede) a 0° (frontal).
#    Um corpo NOVO por ângulo, já encostado, e mede-se UM tique.
func _fase_b():
	if fase_tick == 0:
		print("")
		print("## B) varredura do angulo de incidencia, UM tique cada, corpo ja encostado")
		print("# ang_graus = angulo entre a direcao pedida e a normal INTERIOR (para dentro da parede)")
		print("#   0 = frontal (de cabeca contra a parede)   90 = rasante (ao longo dela)")
		print("# ⚠️ A 1.a redaccao usava dir=(-cos,sin), que aponta para FORA: a varredura")
		print("#    inteira mediu movimento LIVRE (|d|~4.0 em todos os angulos) e leu-se como")
		print("#    «o deslize e' perfeito». Uma fixtura que nao contem o obstaculo aprova tudo.")
		print("# ang  dx  dy  |d|  esperado_tangencial  razao")
		_parede(Vector2(0, 0), Vector2(200, 2000))
	var ang_g := float(fase_tick) * 5.0
	if ang_g <= 90.0:
		var c := _novo_corpo(Vector2(-116.0, 0))
		var a := deg_to_rad(ang_g)
		# A parede esta' em +x; entrar nela e' +x. A 0 graus e' de cabeca, a 90 e' rasante.
		var dir := Vector2(cos(a), sin(a))
		var antes := c.position
		c.velocity = dir * V
		c.move_and_slide()
		var d: Vector2 = c.position - antes
		var tang := sin(a) * V * DT
		var razao := 0.0
		if abs(tang) > 1e-9:
			razao = d.length() / tang
		print("B %.0f %.6f %.6f %.6f %.6f %.6f" % [ang_g, d.x, d.y, d.length(), tang, razao])
		root2d.remove_child(c)
		c.queue_free()
		fase_tick += 1
	else:
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# C) QUINA INTERIOR — duas paredes a 90°, empurrado para dentro dela.
func _fase_c():
	if fase_tick == 0:
		_parede(Vector2(0, 0), Vector2(200, 2000))        # vertical, face em x=-100
		_parede(Vector2(-1000, 200), Vector2(2000, 200))  # horizontal, face de cima em y=100
		corpo = _novo_corpo(Vector2(-300, -300))
		print("")
		print("## C) quina interior (parede vertical + parede horizontal), empurrado a 45 graus")
		print("# tique  x  y  on_wall")
	if fase_tick < 200:
		corpo.velocity = Vector2(1, 1).normalized() * V
		corpo.move_and_slide()
		if fase_tick % 20 == 0 or fase_tick >= 196:
			print("C %d %.6f %.6f %s" % [fase_tick, corpo.position.x, corpo.position.y, corpo.is_on_wall()])
		fase_tick += 1
	else:
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# D) A VELOCIDADE depois do deslize — o `move_and_slide` REESCREVE `velocity`.
#    A pergunta: o que lá fica é a projecção tangencial, ou a rapidez conservada?
func _fase_d():
	if fase_tick == 0:
		print("")
		print("## D) o que o move_and_slide DEIXA em `velocity` (parede vertical, face x=-100)")
		print("# ang  v_pedida  v_depois_x  v_depois_y  |v_depois|  |v_pedida|")
		_parede(Vector2(0, 0), Vector2(200, 2000))
	var ang_g := float(fase_tick) * 15.0
	if ang_g <= 90.0:
		var c := _novo_corpo(Vector2(-116.0, 0))
		var a := deg_to_rad(ang_g)
		var dir := Vector2(cos(a), sin(a))
		c.velocity = dir * V
		c.move_and_slide()
		print("D %.0f %.6f %.6f %.6f %.6f %.6f" % [ang_g, V, c.velocity.x, c.velocity.y, c.velocity.length(), V])
		root2d.remove_child(c)
		c.queue_free()
		fase_tick += 1
	else:
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# E) FLOATING contra GROUNDED com as MESMAS entradas — o dropdown muda a semântica?
func _fase_e():
	if fase_tick == 0:
		print("")
		print("## E) FLOATING vs GROUNDED, mesmas entradas, parede HORIZONTAL (chao) em y=100")
		print("# ⚠️ A 1.a redaccao corria 20 tiques (56,6 px) contra um chao a 84: as duas")
		print("#    colunas saiam IDENTICAS por nenhuma delas ter tocado em nada.")
		print("# ⚠️⚠️ E a 2.a redaccao TAMBEM nao tocava: a parede vivia em x=[-2000,0] e o")
		print("#    corpo anda para +x. Tres fixturas seguidas desta sonda mediram o VAZIO,")
		print("#    e as tres liam-se como um resultado («as duas colunas concordam»).")
		print("# modo  tique  x  y  vx  vy  on_floor  on_wall")
		_parede(Vector2(0, 200), Vector2(4000, 200))
	if fase_tick < 2:
		var c := _novo_corpo(Vector2(0, 0))
		var floating := fase_tick == 0
		c.motion_mode = CharacterBody2D.MOTION_MODE_FLOATING if floating else CharacterBody2D.MOTION_MODE_GROUNDED
		var nome := "FLOAT" if floating else "GROUND"
		for t in 60:
			c.velocity = Vector2(1, 1).normalized() * V
			c.move_and_slide()
			if t % 15 == 0 or t == 59:
				print("E %s %d %.6f %.6f %.6f %.6f %s %s" % [nome, t, c.position.x, c.position.y, c.velocity.x, c.velocity.y, c.is_on_floor(), c.is_on_wall()])
		root2d.remove_child(c)
		c.queue_free()
		fase_tick += 1
	else:
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# F) PAREDE INCLINADA a 45° — a projecção corta a rapidez, ou o corpo acelera na diagonal?
func _fase_f():
	if fase_tick == 0:
		var b := StaticBody2D.new()
		# ⚠️ A 1.a redaccao punha um rectangulo 200x2000 rodado sobre a ORIGEM e o corpo
		# em (-300,0): ele encontrava a PONTA da peca em vez da face, e a leitura nao
		# dizia nada sobre a lei. Aqui o plano e' largo, esta' longe, e entra-se na FACE.
		b.position = Vector2(400, 0)
		b.rotation = deg_to_rad(45.0)
		var cs := CollisionShape2D.new()
		var r := RectangleShape2D.new()
		r.size = Vector2(400, 4000)
		cs.shape = r
		b.add_child(cs)
		root2d.add_child(b)
		corpo = _novo_corpo(Vector2(-100, 0))
		print("")
		print("## F) parede INCLINADA 45 graus, empurrado na horizontal (dir=(1,0))")
		print("# tique  x  y  vx  vy  |v|")
	if fase_tick < 90:
		corpo.velocity = Vector2(1, 0) * V
		corpo.move_and_slide()
		if fase_tick % 10 == 0 or fase_tick == 89:
			print("F %d %.6f %.6f %.6f %.6f %.6f" % [fase_tick, corpo.position.x, corpo.position.y, corpo.velocity.x, corpo.velocity.y, corpo.velocity.length()])
		fase_tick += 1
	else:
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# H) O motor REESCREVE `velocity`? — o bloco D diz que nao, mas ali ela e' reposta
#    todo tique. Aqui pede-se UMA vez e deixa-se correr: se o motor a projectasse,
#    ela cairia para a tangente (ou para zero, de cabeca contra a parede).
func _fase_h():
	if fase_tick == 0:
		_parede(Vector2(0, 0), Vector2(200, 2000))
		corpo = _novo_corpo(Vector2(-116.0, 0))
		corpo.velocity = Vector2(1, 1).normalized() * V     # UMA vez, e so'
		print("")
		print("## H) `velocity` pedida UMA vez; 10 tiques sem a repor (45 graus contra a parede)")
		print("# tique  x  y  vx  vy")
	if fase_tick < 10:
		corpo.move_and_slide()
		print("H %d %.6f %.6f %.6f %.6f" % [fase_tick, corpo.position.x, corpo.position.y, corpo.velocity.x, corpo.velocity.y])
		fase_tick += 1
	else:
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# I) O LIMIAR, ao grau — o bloco B viu o penhasco entre 15 e 20; aqui mede-se 1 a 1,
#    e com o `wall_min_slide_angle` a ZERO ao lado, que e' o CONTROLO: se o penhasco
#    for do knob, ele desaparece na segunda coluna.
func _fase_i():
	if fase_tick == 0:
		print("")
		print("## I) o limiar ao grau, e o CONTROLO com wall_min_slide_angle = 0")
		print("# ang  |d|_knob_15  |d|_knob_0")
		_parede(Vector2(0, 0), Vector2(200, 2000))
	var ang_g := 10.0 + float(fase_tick)
	if ang_g <= 25.0:
		var d := []
		for knob in [deg_to_rad(15.0), 0.0]:
			var c := _novo_corpo(Vector2(-116.0, 0))
			c.wall_min_slide_angle = knob
			var a := deg_to_rad(ang_g)
			var antes := c.position
			c.velocity = Vector2(cos(a), sin(a)) * V
			c.move_and_slide()
			d.append((c.position - antes).length())
			root2d.remove_child(c)
			c.queue_free()
		print("I %.0f %.6f %.6f" % [ang_g, d[0], d[1]])
		fase_tick += 1
	else:
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# G) CUSTO — N corpos a deslizar por tique.
func _fase_g():
	if fase_tick == 0:
		print("")
		print("## G) custo de N corpos a chamar move_and_slide num tique (parede vertical)")
		print("# n  ms_total  us_por_corpo")
		_parede(Vector2(0, 0), Vector2(200, 20000))
		fase_tick += 1
		return
	var ns := [100, 1000, 5000]
	var i := fase_tick - 1
	if i < ns.size():
		var n: int = ns[i]
		var corpos := []
		for k in n:
			corpos.append(_novo_corpo(Vector2(-116.0, -9000.0 + float(k) * 3.0)))
		# um tique para o mundo se assentar com os corpos dentro
		var t0 := Time.get_ticks_usec()
		for c in corpos:
			c.velocity = Vector2(1, 1).normalized() * V
			c.move_and_slide()
		var t1 := Time.get_ticks_usec()
		print("G %d %.3f %.3f" % [n, float(t1 - t0) / 1000.0, float(t1 - t0) / float(n)])
		for c in corpos:
			root2d.remove_child(c)
			c.queue_free()
		fase_tick += 1
	else:
		print("")
		print("# FIM")
		quit()
