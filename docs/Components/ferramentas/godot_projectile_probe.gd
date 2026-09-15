# ORÁCULO: o PROJÉCTIL no Godot 4.7.2 (MIT), corrido sem interface — docs/Components/11_plano_projectile_motion.md §2.
# Correr: godot --headless --script docs/Components/ferramentas/godot_projectile_probe.gd
#
# A pergunta que sobra depois da sonda da composição (§1): **o que acontece ao ORÇAMENTO
# que RESTA quando o corpo bate?** No #13 mediu-se que o `FLOATING` o conserva e re-emite
# na tangente (1,414× a 45°); aqui a hipótese irmã é re-emiti-lo no ESPELHO.
#
# O alvo é a receita canónica de um projéctil cinemático no Godot — `move_and_collide()`
# mais `KinematicCollision2D.get_remainder()` e `Vector2.bounce(normal)`. ⚠️ As duas
# primeiras são comportamento do MOTOR (medível); a terceira é aritmética publicada.
#
# Mede, uma pergunta por bloco:
#   A) o RESTO: quanto do orçamento de um tique sobra depois do toque, e ele é gasto?
#   B) o ESPELHO: a direcção de saída contra `v - 2(v·n)n`, varrendo a incidência
#   C) o SEGUNDO toque no MESMO tique (uma quina): o laço fecha ou o resto evapora?
#   D) a NORMAL que ele devolve numa parede inclinada — o sinal e o referencial
#
# ⚠️ Corre a partir do 1.º _process (a lição dos probes irmãos: antes disso os nós não
#    estão na árvore, e o 1.º _physics_process vem ANTES do 1.º _process).
# ⚠️ Unidades do Godot são PIXELS; a geometria corre em números grandes para a margem
#    de des-penetração (`safe_margin`, 0,08 px) não decidir nada.
extends SceneTree

const DT := 1.0 / 60.0
const RAIO := 16.0
const V := 600.0                # px/s ⇒ 10 px por tique: o resto é grande e legível

var frame := 0
var root2d: Node2D
var corpo: CharacterBody2D
var fase := 0
var fase_tick := 0


func _initialize():
	print("# ORACULO godot_projectile — CharacterBody2D + move_and_collide + bounce")
	print("# godot: ", Engine.get_version_info().string)
	print("# dt=%.6f  raio=%.1fpx  V=%.1fpx/s (%.4f px/tique)" % [DT, RAIO, V, V * DT])


func _parede(pos: Vector2, tam: Vector2, ang := 0.0) -> StaticBody2D:
	var b := StaticBody2D.new()
	b.position = pos
	b.rotation = ang
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
	var cs := CollisionShape2D.new()
	var f := CircleShape2D.new()
	f.radius = RAIO
	cs.shape = f
	c.add_child(cs)
	root2d.add_child(c)
	return c


func _limpar():
	for n in root2d.get_children():
		n.queue_free()
	corpo = null


func _process(_delta):
	if root2d == null:
		root2d = Node2D.new()
		get_root().add_child(root2d)
		return false
	frame += 1
	return false


func _avanca():
	fase += 1
	fase_tick = 0
	_limpar()


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
		_:
			print("")
			print("# FIM")
			quit()
	return false


# ─────────────────────────────────────────────────────────────────────────────
# A) O RESTO. Um tiro frontal contra uma parede, medindo o orçamento gasto no
#    tique do toque: `move_and_collide` devolve o que SOBROU (`get_remainder`).
func _fase_a():
	if fase_tick == 0:
		_parede(Vector2(0, 0), Vector2(200, 2000))     # face esquerda em x = −100
		corpo = _novo_corpo(Vector2(-300, 0))
		print("")
		print("## BLOCO A — o RESTO do orcamento no tique do toque (tiro frontal)")
		print("# orcamento de um tique = %.4f px" % (V * DT))
	fase_tick += 1
	var orcamento := Vector2(V * DT, 0)
	var col := corpo.move_and_collide(orcamento)
	if col != null:
		var andou: float = orcamento.length() - col.get_remainder().length()
		print("RESTO andou=%.6f resto=%.6f soma=%.6f normal=(%.4f,%.4f)" % [
			andou, col.get_remainder().length(),
			andou + col.get_remainder().length(),
			col.get_normal().x, col.get_normal().y])
		print("# ⇒ a soma bate o orcamento? e' isso que diz se o resto se CONSERVA")
		_avanca()
	elif fase_tick > 120:
		print("RESTO (nunca tocou)")
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# B) O ESPELHO, varrendo a incidência. A saída contra `v - 2(v·n)n`.
func _fase_b():
	if fase_tick == 0:
		print("")
		print("## BLOCO B — a direccao de saida contra o ESPELHO")
		print("# graus | o angulo de entrada, o de saida, e o do espelho publicado — mais o RESTO")
	var graus := [90.0, 75.0, 60.0, 45.0, 30.0, 15.0]
	for g in graus:
		_limpar()
		_parede(Vector2(0, 0), Vector2(200, 4000))
		corpo = _novo_corpo(Vector2(-300, -600))
		var r: float = deg_to_rad(g)
		var v := Vector2(sin(r), cos(r)) * V
		# Anda até tocar, e no toque devolve o espelho da lei publicada.
		for _i in range(400):
			var col := corpo.move_and_collide(v * DT)
			if col != null:
				var n := col.get_normal()
				var espelho := v.bounce(n)
				var resto := col.get_remainder()
				var saida := resto.bounce(n)
				# ⚠️ **Compara-se o ANGULO**, nunca o comprimento: `saida` e' um DESLOCAMENTO
				# de um tique e `espelho` e' uma VELOCIDADE — duas unidades na mesma linha
				# leem-se como uma discordancia que nao existe.
				var erro: float = rad_to_deg(abs(saida.angle_to(espelho)))
				print("BOUNCE %5.1f | ent %7.2f° | sai %7.2f° | espelho %7.2f° | erro %.5f° | resto=%.4f |saida|=%.4f" % [
					g, rad_to_deg(v.angle()), rad_to_deg(saida.angle()),
					rad_to_deg(espelho.angle()), erro, resto.length(), saida.length()])
				break
	print("# ⇒ `Vector2.bounce(n)` e' `v - 2*(v·n)*n`? e o RESTO conserva o comprimento?")
	_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# C) DOIS toques no MESMO tique (uma quina): o laco fecha ou o resto evapora?
func _fase_c():
	if fase_tick == 0:
		print("")
		print("## BLOCO C — a QUINA: dois toques no mesmo tique")
		# ⚠️ **A 1.ª fixtura desta sonda mediu o VAZIO**: o corpo tocava uma parede e o resto
		# nunca alcancava a outra, logo ela lia «1 salto» sobre uma quina que nao existia no
		# alcance do tique. A geometria a seguir poe as duas faces em 0 e da' ao tique um
		# orcamento GRANDE, para os dois toques caberem nele por construcao.
		_parede(Vector2(100, 0), Vector2(200, 4000))      # face esquerda em x = 0
		_parede(Vector2(0, 100), Vector2(4000, 200))      # face de cima em y = 0 (o +y do Godot e' para BAIXO)
		corpo = _novo_corpo(Vector2(-200, -200))
	fase_tick += 1
	var v := Vector2(1, 1).normalized() * V
	# ⚠️ Um orcamento de 60 tiques num so': e' o que torna a quina alcancavel DENTRO do tique.
	var orcamento := v * DT * 60.0
	var saltos := 0
	for _i in range(8):
		var col := corpo.move_and_collide(orcamento)
		if col == null:
			break
		saltos += 1
		orcamento = col.get_remainder().bounce(col.get_normal())
		if orcamento.length() < 0.001:
			break
	if saltos > 0:
		print("QUINA saltos_num_tique=%d resto_final=%.6f (orcamento inicial %.4f)" % [
			saltos, orcamento.length(), V * DT * 60.0])
		print("# ⇒ quantos ricochetes cabem num tique, e o laco precisa de TECTO?")
		_avanca()
	elif fase_tick > 120:
		print("QUINA (nunca tocou)")
		_avanca()


# ─────────────────────────────────────────────────────────────────────────────
# D) A NORMAL de uma parede INCLINADA — o sinal e o referencial.
func _fase_d():
	if fase_tick == 0:
		print("")
		print("## BLOCO D — a NORMAL de uma parede a 45°")
		_parede(Vector2(0, 0), Vector2(100, 2000), deg_to_rad(45.0))
		corpo = _novo_corpo(Vector2(-400, 0))
	fase_tick += 1
	var col := corpo.move_and_collide(Vector2(V * DT, 0))
	if col != null:
		var n := col.get_normal()
		print("NORMAL (%.5f,%.5f) |n|=%.5f angulo=%.2f°" % [n.x, n.y, n.length(), rad_to_deg(n.angle())])
		print("# ⇒ ela aponta PARA FORA da parede (contra o movimento), em MUNDO?")
		_avanca()
	elif fase_tick > 120:
		print("NORMAL (nunca tocou)")
		_avanca()
