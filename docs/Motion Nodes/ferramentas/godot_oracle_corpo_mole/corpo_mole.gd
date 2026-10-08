# Oráculo do Bug #15 (doc 124): um corpo de partículas com raio, ligado por molas, com o contacto
# resolvido pelo SOLVER do Godot (MIT). Corre-se, nunca se lê o fonte (CLAUDE.md §0.9):
#
#   godot --headless --script corpo_mole.gd -- <modo>
#
# <modo>: `nu` (sem colisão) · `lei` (pares ligados por mola NÃO colidem — a exceção por par,
# `add_collision_exception_with`) · `todos` (todos os pares colidem, os ligados incluídos).
#
# A fixtura é a NOSSA (`soft_body_radius` do `ph2d-node-registry-init`): `5 × 5`, passo `0,30`,
# raio `0,28`, gravidade `9,8`, a linha de topo presa a uma âncora que varre `±2,0` com período
# `0,7 s`, `3 s` de simulação. O Godot trabalha em píxeis: tudo ×100.
extends SceneTree

const ESCALA := 100.0
const LINHAS := 5
const COLUNAS := 5
const PASSO := 0.30 * ESCALA
const RAIO := 0.28 * ESCALA
const PERIODO := 0.7
const AMPLITUDE := 2.0 * ESCALA
const SEGUNDOS := 3.0
const HZ := 960
const RIGIDEZ := 20000.0
const AMORTECIMENTO := 40.0

var modo := "lei"
var corpos: Array = []
var repouso: Array = []
var ligados: Array = []
var quadro := 0
var t := 0.0
var pior_nao_ligado := INF
var pior_nao_ligado_fim := INF

func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	if args.size() > 0:
		modo = args[0]
	Engine.physics_ticks_per_second = HZ
	# ⚠️ MEDIDO: com `--script` o servidor de física nasce PARADO — sem esta linha os corpos não
	# saem do sítio (`_physics_process` corre, a posição fica `0,0` em 120 quadros), e as três
	# variantes saem IGUAIS ao bit. Uma corrida que não integra faz todo oráculo concordar.
	PhysicsServer2D.set_active(true)
	var raiz := Node2D.new()
	root.add_child(raiz)
	for r in LINHAS:
		for c in COLUNAS:
			var q := Vector2(c * PASSO, r * PASSO)
			repouso.append(q)
			var b := RigidBody2D.new()
			b.position = q
			b.mass = 1.0
			b.gravity_scale = 1.0
			if modo != "nu":
				var forma := CollisionShape2D.new()
				var circ := CircleShape2D.new()
				circ.radius = RAIO
				forma.shape = circ
				b.add_child(forma)
			if r == 0:
				b.freeze = true
				b.freeze_mode = RigidBody2D.FREEZE_MODE_KINEMATIC
			raiz.add_child(b)
			corpos.append(b)
	# Os pares LIGADOS: os que o repouso já sobrepõe (distância de repouso < 2·raio) — os quatro
	# vizinhos e as diagonais. São estes que uma mola segura mais perto que o diâmetro.
	for i in corpos.size():
		for j in range(i + 1, corpos.size()):
			if repouso[i].distance_to(repouso[j]) < 2.0 * RAIO:
				ligados.append([i, j, repouso[i].distance_to(repouso[j])])
				if modo == "lei":
					corpos[i].add_collision_exception_with(corpos[j])

func _e_ligado(i: int, j: int) -> bool:
	for l in ligados:
		if (l[0] == i and l[1] == j) or (l[0] == j and l[1] == i):
			return true
	return false

func _physics_process(dt: float) -> bool:
	t += dt
	quadro += 1
	var ancora := Vector2(AMPLITUDE * sin(TAU * t / PERIODO), 0.0)
	for c in COLUNAS:
		corpos[c].position = repouso[c] + ancora
	for l in ligados:
		var a: RigidBody2D = corpos[l[0]]
		var b: RigidBody2D = corpos[l[1]]
		var d := b.position - a.position
		var n := d.length()
		if n < 1e-6:
			continue
		var u := d / n
		var vrel := (b.linear_velocity - a.linear_velocity).dot(u)
		var repouso_l: float = l[2]
		var f: Vector2 = u * (RIGIDEZ * (n - repouso_l) + AMORTECIMENTO * vrel)
		if not a.freeze:
			a.apply_central_force(f)
		if not b.freeze:
			b.apply_central_force(-f)
	if t >= SEGUNDOS * 0.5:
		var p := _pior_nao_ligado()
		pior_nao_ligado = min(pior_nao_ligado, p)
	if t >= SEGUNDOS:
		pior_nao_ligado_fim = _pior_nao_ligado()
		_relata()
		quit()
	return false

func _pior_nao_ligado() -> float:
	var m := INF
	for i in corpos.size():
		for j in range(i + 1, corpos.size()):
			if not _e_ligado(i, j):
				m = min(m, corpos[i].position.distance_to(corpos[j].position))
	return m

func _relata() -> void:
	var extensao := 0.0
	var ligado_min := INF
	for i in corpos.size():
		for j in corpos.size():
			extensao = max(extensao, corpos[i].position.distance_to(corpos[j].position))
	for l in ligados:
		ligado_min = min(ligado_min, corpos[l[0]].position.distance_to(corpos[l[1]].position))
	print("ORACULO modo=%s quadros=%d hz=%d pares_ligados=%d pior_nao_ligado_fim=%.4f pior_nao_ligado_2a_metade=%.4f par_ligado_min=%.4f extensao=%.4f diametro=%.4f" % [
		modo, quadro, HZ, ligados.size(),
		pior_nao_ligado_fim / ESCALA, pior_nao_ligado / ESCALA,
		ligado_min / ESCALA, extensao / ESCALA, 2.0 * RAIO / ESCALA])
