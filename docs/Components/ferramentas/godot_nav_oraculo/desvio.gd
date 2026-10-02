# O CORPUS DO ORÁCULO DO DESVIO (plano 30 §8.2, família F4 — a W5).
#
# Corre-se sem interface, sobre entradas NOSSAS (§0.9 do roteador — a saída é livre, o fonte não se lê):
#
#   cd docs/Components/ferramentas/godot_nav_oraculo/desvio_projeto &&
#     godot --headless --fixed-fps 60 --path . --script ../desvio.gd > saida.txt
#
# ⛔ O `--path` é o que liga o projecto de UMA linha de execução (ver o `project.godot` dele): sem ele
# o desvio do Godot tem uma corrida de dados e duas corridas diferem.
#
# MALHA FECHADA, para comparar POR PASSO: o script é o dono do estado (posição e velocidade de cada agente);
# a cada quadro de física ENTREGA ao servidor a posição (`agent_set_position`), a velocidade corrente
# (`agent_set_velocity_forced`) e a preferida (`agent_set_velocity`), e grava-as numa linha `IN`. O que o
# servidor devolve no *callback* é a velocidade segura, gravada numa linha `OUT` com o quadro em que chegou.
# O passo seguinte integra `pos += safe / 60` e `vel = safe`. ⇒ cada `OUT` tem as entradas exactas que o
# resolveu, e o gate do lado de cá re-resolve CADA passo isolado (o erro não se acumula).
#
# Unidades: píxeis, segundos. Os parâmetros são os de fábrica do servidor (medidos por `agent_get_*`),
# salvo o horizonte dos obstáculos, que nasce `0` (desliga-os) e aqui vai a `1` nas cenas com paredes.
extends SceneTree

const VMAX := 100.0
const R := 10.0

var map: RID
var agents := []
var pos := []
var vel := []
var alvo := []
var safe := []
var frame := 0
var cena_i := 0
var cenas := []
var obstaculos := []
const PASSOS := 240


func circulo(n: int, cx: float, cy: float, raio: float) -> Array:
	var out := []
	for i in n:
		var a := TAU * float(i) / float(n)
		var p := Vector2(cx + raio * cos(a), cy + raio * sin(a))
		var q := Vector2(cx - raio * cos(a), cy - raio * sin(a))
		out.append([p, q])
	return out


# Área com sinal POSITIVA (anti-horária com o y para cima): o servidor empurra o agente para FORA. Medido
# na 1.ª corrida: com a ordem inversa os quatro agentes da `porta` acabaram DENTRO das paredes.
func caixa_ccw(x0, y0, x1, y1) -> PackedVector2Array:
	return PackedVector2Array([Vector2(x0, y0), Vector2(x1, y0), Vector2(x1, y1), Vector2(x0, y1)])


func monta_cenas():
	cenas = [
		{"nome": "frente", "th_o": 0.0, "obs": [],
		 "ag": [[Vector2(50, 150), Vector2(350, 150)], [Vector2(350, 150), Vector2(50, 150)]]},
		{"nome": "frente_desviado", "th_o": 0.0, "obs": [],
		 "ag": [[Vector2(50, 150), Vector2(350, 150)], [Vector2(350, 153), Vector2(50, 153)]]},
		{"nome": "cruzamento", "th_o": 0.0, "obs": [],
		 "ag": [[Vector2(50, 50), Vector2(350, 250)], [Vector2(350, 250), Vector2(50, 50)],
				[Vector2(350, 50), Vector2(50, 250)], [Vector2(50, 250), Vector2(350, 50)]]},
		{"nome": "circulo8", "th_o": 0.0, "obs": [], "ag": circulo(8, 200, 150, 120)},
		{"nome": "corredor", "th_o": 1.0,
		 "obs": [caixa_ccw(100, 90, 300, 120), caixa_ccw(100, 180, 300, 210)],
		 "ag": [[Vector2(60, 150), Vector2(340, 150)], [Vector2(340, 152), Vector2(60, 152)]]},
		{"nome": "porta", "th_o": 1.0,
		 "obs": [caixa_ccw(190, 0, 210, 125), caixa_ccw(190, 175, 210, 300)],
		 "ag": [[Vector2(100, 150), Vector2(320, 150)], [Vector2(80, 120), Vector2(320, 140)],
				[Vector2(80, 180), Vector2(320, 160)], [Vector2(50, 150), Vector2(320, 150)]]},
	]


func pv(v: Vector2) -> String:
	return "%.9f %.9f" % [v.x, v.y]


func _initialize():
	monta_cenas()
	print("# PH2D corpus do oraculo do desvio — Godot ", Engine.get_version_info().string)
	print("# fonte: docs/Components/ferramentas/godot_nav_oraculo/desvio.gd · NavigationServer2D agents (RVO2) em malha fechada")
	print("# formato: CENA <nome> | PARAM r vmax nd mn th_a th_o dt | OBS <x y ...> | IN <quadro> <i> px py vx vy prefx prefy | OUT <quadro> <i> sx sy")
	comeca_cena()


func comeca_cena():
	var c = cenas[cena_i]
	map = NavigationServer2D.map_create()
	NavigationServer2D.map_set_use_async_iterations(map, false)
	NavigationServer2D.map_set_active(map, true)
	agents = []; pos = []; vel = []; alvo = []; safe = []; obstaculos = []
	var ag0 := NavigationServer2D.agent_create()
	var nd := NavigationServer2D.agent_get_neighbor_distance(ag0)
	var mn := NavigationServer2D.agent_get_max_neighbors(ag0)
	var tha := NavigationServer2D.agent_get_time_horizon_agents(ag0)
	NavigationServer2D.free_rid(ag0)
	print("CENA %s" % c["nome"])
	print("PARAM %.9f %.9f %.9f %d %.9f %.9f %.9f" % [R, VMAX, nd, mn, tha, c["th_o"], 1.0 / Engine.physics_ticks_per_second])
	for o in c["obs"]:
		var ob := NavigationServer2D.obstacle_create()
		NavigationServer2D.obstacle_set_map(ob, map)
		NavigationServer2D.obstacle_set_vertices(ob, o)
		NavigationServer2D.obstacle_set_avoidance_enabled(ob, true)
		obstaculos.append(ob)
		var s := []
		for v in o: s.append(pv(v))
		print("OBS " + " ".join(s))
	for i in c["ag"].size():
		var ag := NavigationServer2D.agent_create()
		NavigationServer2D.agent_set_map(ag, map)
		NavigationServer2D.agent_set_avoidance_enabled(ag, true)
		NavigationServer2D.agent_set_radius(ag, R)
		NavigationServer2D.agent_set_max_speed(ag, VMAX)
		NavigationServer2D.agent_set_time_horizon_obstacles(ag, c["th_o"])
		NavigationServer2D.agent_set_avoidance_callback(ag, _on_safe.bind(i))
		agents.append(ag)
		pos.append(c["ag"][i][0])
		alvo.append(c["ag"][i][1])
		vel.append(Vector2.ZERO)
		safe.append(Vector2.ZERO)
	frame = 0


func acaba_cena():
	for ag in agents: NavigationServer2D.free_rid(ag)
	for ob in obstaculos: NavigationServer2D.free_rid(ob)
	NavigationServer2D.free_rid(map)


func _on_safe(v: Vector2, i: int):
	safe[i] = v
	print("OUT %d %d %s" % [frame, i, pv(v)])


func _physics_process(_dt: float) -> bool:
	if frame > 0:
		for i in agents.size():
			vel[i] = safe[i]
			pos[i] = pos[i] + safe[i] / Engine.physics_ticks_per_second
	frame += 1
	for i in agents.size():
		var d: Vector2 = alvo[i] - pos[i]
		var pref := Vector2.ZERO
		if d.length() > VMAX / Engine.physics_ticks_per_second:
			pref = d.normalized() * VMAX
		elif d.length() > 0.0:
			pref = d * Engine.physics_ticks_per_second
		NavigationServer2D.agent_set_position(agents[i], pos[i])
		NavigationServer2D.agent_set_velocity_forced(agents[i], vel[i])
		NavigationServer2D.agent_set_velocity(agents[i], pref)
		print("IN %d %d %s %s %s" % [frame, i, pv(pos[i]), pv(vel[i]), pv(pref)])
	if frame >= PASSOS:
		acaba_cena()
		cena_i += 1
		if cena_i >= cenas.size():
			return true
		comeca_cena()
	return false
