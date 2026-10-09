# Oráculo do Bug #16 (doc 125): uma CORRENTE de contas com raio, ligadas em fila por molas rígidas,
# com o contacto resolvido pelo SOLVER do Godot (MIT). Corre-se, nunca se lê o fonte (CLAUDE.md §0.9):
#
#   godot --headless --script corrente.gd -- <modo>
#
# <modo>: `nu` (sem colisão) · `lei` (os pares que o REPOUSO sobrepõe não colidem — a exceção por
# par, `add_collision_exception_with`) · `todos` (todos os pares colidem, os vizinhos incluídos).
#
# A fixtura é a NOSSA (`rope_thickness` do `ph2d-node-registry-init`): `24` contas, comprimento `6,0`
# (segmento `6/23`), raio `0,5`, gravidade `9,8`, a cabeça presa a uma âncora que varre
# `3,0·sin(2πt/0,6)` em x, `4 s` de simulação (os `240` quadros a `60 Hz` do gate; a 2.ª metade é
# `t ≥ 2 s`). A corrente nasce RECTA ao longo de +x a partir da âncora, como a semente da corda.
# O Godot trabalha em píxeis e com y para BAIXO: tudo ×100, e a gravidade é +y.
#
# ⚠️ O Godot 2D não tem uma junta de DISTÂNCIA (a `PinJoint2D` prende um ponto e deixa o disco rodar
# à volta dele), por isso a restrição da corda é uma mola rígida com amortecimento ao longo do elo,
# a um ritmo alto o bastante para ela não esticar (o comprimento sai na linha, para se ver).
extends SceneTree

const ESCALA := 100.0
const CONTAS := 24
const COMPRIMENTO := 6.0 * ESCALA
const RAIO := 0.5 * ESCALA
const PERIODO := 0.6
const AMPLITUDE := 3.0 * ESCALA
const SEGUNDOS := 4.0
const HZ := 4800
const RIGIDEZ := 1.0e6
const AMORTECIMENTO := 280.0
# O amortecimento da corda (`damping 0,02` por quadro a `60 Hz` ⇒ `−ln(0,98)·60 ≈ 1,21` por segundo).
const ARRASTO := 1.21

var modo := "lei"
var contas: Array = []
var repouso: Array = []
var segmento := COMPRIMENTO / float(CONTAS - 1)
var quadro := 0
var t := 0.0
var pior_nao_ligado := INF
var pior_nao_ligado_fim := INF
var maior_arco := 0.0

func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	if args.size() > 0:
		modo = args[0]
	Engine.physics_ticks_per_second = HZ
	# ⚠️ MEDIDO no oráculo do corpo mole: com `--script` o servidor de física nasce PARADO.
	PhysicsServer2D.set_active(true)
	var raiz := Node2D.new()
	root.add_child(raiz)
	for i in CONTAS:
		var q := Vector2(i * segmento, 0.0)
		repouso.append(q)
		var b := RigidBody2D.new()
		b.position = q
		b.mass = 1.0
		b.gravity_scale = 1.0
		b.linear_damp_mode = RigidBody2D.DAMP_MODE_REPLACE
		b.linear_damp = ARRASTO
		if modo != "nu":
			var forma := CollisionShape2D.new()
			var circ := CircleShape2D.new()
			circ.radius = RAIO
			forma.shape = circ
			b.add_child(forma)
		if i == 0:
			b.freeze = true
			b.freeze_mode = RigidBody2D.FREEZE_MODE_KINEMATIC
		raiz.add_child(b)
		contas.append(b)
	if modo == "lei":
		for i in CONTAS:
			for j in range(i + 1, CONTAS):
				if _ligado(i, j):
					contas[i].add_collision_exception_with(contas[j])

# Os pares LIGADOS: os que o repouso já sobrepõe (distância de repouso < 2·raio) — na corrente
# recta, `|i − j| ≤ 3`. É a mesma pergunta que o filtro do `ph2d_contact` faz.
func _ligado(i: int, j: int) -> bool:
	return repouso[i].distance_to(repouso[j]) < 2.0 * RAIO

func _physics_process(dt: float) -> bool:
	t += dt
	quadro += 1
	contas[0].position = Vector2(AMPLITUDE * sin(TAU * t / PERIODO), 0.0)
	for i in CONTAS - 1:
		var a: RigidBody2D = contas[i]
		var b: RigidBody2D = contas[i + 1]
		var d := b.position - a.position
		var n := d.length()
		if n < 1e-6:
			continue
		var u := d / n
		var vrel := (b.linear_velocity - a.linear_velocity).dot(u)
		var f: Vector2 = u * (RIGIDEZ * (n - segmento) + AMORTECIMENTO * vrel)
		if not a.freeze:
			a.apply_central_force(f)
		if not b.freeze:
			b.apply_central_force(-f)
	if t >= SEGUNDOS * 0.5:
		pior_nao_ligado = min(pior_nao_ligado, _pior_nao_ligado())
		maior_arco = max(maior_arco, _arco())
	if t >= SEGUNDOS:
		pior_nao_ligado_fim = _pior_nao_ligado()
		_relata()
		quit()
	return false

func _pior_nao_ligado() -> float:
	var m := INF
	for i in CONTAS:
		for j in range(i + 1, CONTAS):
			if not _ligado(i, j):
				m = min(m, contas[i].position.distance_to(contas[j].position))
	return m

func _arco() -> float:
	var s := 0.0
	for i in CONTAS - 1:
		s += contas[i].position.distance_to(contas[i + 1].position)
	return s

func _relata() -> void:
	print("CORRENTE modo=%s quadros=%d hz=%d pior_nao_ligado_fim=%.4f pior_nao_ligado_2a_metade=%.4f arco_fim=%.4f maior_arco_2a_metade=%.4f repouso=%.4f diametro=%.4f" % [
		modo, quadro, HZ,
		pior_nao_ligado_fim / ESCALA, pior_nao_ligado / ESCALA,
		_arco() / ESCALA, maior_arco / ESCALA, COMPRIMENTO / ESCALA, 2.0 * RAIO / ESCALA])
