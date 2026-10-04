# O ORÁCULO DO CORPO LARGO QUE ANDA (plano 30 §22.5, W14) — o que faz o desvio do Godot quando um
# obstáculo LARGO vem de frente? Três formas do mesmo obstáculo (uma barreira `10 × 100` px, a andar a
# `10` px/s contra o agente), em malha fechada como o `desvio.gd`:
#
#   cd docs/Components/ferramentas/godot_nav_oraculo/desvio_projeto &&
#     godot --headless --fixed-fps 60 --path . --script ../largo.gd > saida_largo.txt
#
# - `poligono`: um obstáculo de VÉRTICES (`obstacle_set_vertices`) que se muda de sítio a cada quadro;
# - `disco`: um obstáculo de RAIO (o que envolve a barreira) com a velocidade dele;
# - `discos`: dez obstáculos de raio ao longo da barreira (o que a ponte do PH2D faz).
# Mais um CONTROLO parado (`poligono_parado`) e uma variante assimétrica (o agente `3` px fora do eixo).
extends SceneTree

const VMAX := 100.0
const R := 10.0
const VB := -10.0
const PASSOS := 900

var map: RID
var ag: RID
var obs := []
var pos := Vector2.ZERO
var vel := Vector2.ZERO
var safe := Vector2.ZERO
var alvo := Vector2(550, 150)
var frame := 0
var cena_i := 0
var cenas := []
var bx := 0.0


func _initialize():
	cenas = [
		{"nome": "poligono", "forma": "poligono", "vb": VB, "y0": 150.0},
		{"nome": "poligono_assimetrico", "forma": "poligono", "vb": VB, "y0": 153.0},
		{"nome": "poligono_parado", "forma": "poligono", "vb": 0.0, "y0": 150.0},
		{"nome": "disco", "forma": "disco", "vb": VB, "y0": 150.0},
		{"nome": "discos", "forma": "discos", "vb": VB, "y0": 150.0},
		{"nome": "discos_assimetrico", "forma": "discos", "vb": VB, "y0": 153.0},
	]
	print("# PH2D oraculo do corpo largo — Godot ", Engine.get_version_info().string)
	print("# fonte: docs/Components/ferramentas/godot_nav_oraculo/largo.gd · cd desvio_projeto && godot --headless --fixed-fps 60 --path . --script ../largo.gd | grep -E '^#|^CENA|^FIM|^  q'")
	print("# agente R=10 px, 100 px/s, de (50, y0) para (550, 150); barreira 10 x 100 px em x=300 a andar a vb px/s; malha fechada, uma linha de execucao")
	comeca()


func comeca():
	var c = cenas[cena_i]
	map = NavigationServer2D.map_create()
	NavigationServer2D.map_set_use_async_iterations(map, false)
	NavigationServer2D.map_set_active(map, true)
	bx = 300.0
	obs = []
	if c["forma"] == "poligono":
		var o := NavigationServer2D.obstacle_create()
		NavigationServer2D.obstacle_set_map(o, map)
		NavigationServer2D.obstacle_set_vertices(o, PackedVector2Array([Vector2(-5, -50), Vector2(5, -50), Vector2(5, 50), Vector2(-5, 50)]))
		NavigationServer2D.obstacle_set_avoidance_enabled(o, true)
		obs.append([o, Vector2.ZERO])
	elif c["forma"] == "disco":
		var o := NavigationServer2D.obstacle_create()
		NavigationServer2D.obstacle_set_map(o, map)
		NavigationServer2D.obstacle_set_radius(o, 50.0)
		NavigationServer2D.obstacle_set_avoidance_enabled(o, true)
		obs.append([o, Vector2.ZERO])
	else:
		for k in 10:
			var o := NavigationServer2D.obstacle_create()
			NavigationServer2D.obstacle_set_map(o, map)
			NavigationServer2D.obstacle_set_radius(o, 7.1)
			NavigationServer2D.obstacle_set_avoidance_enabled(o, true)
			obs.append([o, Vector2(0, -45 + 10 * k)])
	ag = NavigationServer2D.agent_create()
	NavigationServer2D.agent_set_map(ag, map)
	NavigationServer2D.agent_set_avoidance_enabled(ag, true)
	NavigationServer2D.agent_set_radius(ag, R)
	NavigationServer2D.agent_set_max_speed(ag, VMAX)
	NavigationServer2D.agent_set_time_horizon_obstacles(ag, 1.0)
	NavigationServer2D.agent_set_avoidance_callback(ag, _on_safe)
	pos = Vector2(50, c["y0"])
	vel = Vector2.ZERO
	safe = Vector2.ZERO
	frame = 0
	print("CENA %s" % c["nome"])


func _on_safe(v: Vector2):
	safe = v


func _physics_process(_dt: float) -> bool:
	var c = cenas[cena_i]
	if frame > 0:
		vel = safe
		pos += safe / Engine.physics_ticks_per_second
	frame += 1
	bx = 300.0 + c["vb"] * frame / Engine.physics_ticks_per_second
	for o in obs:
		NavigationServer2D.obstacle_set_position(o[0], Vector2(bx, 150) + o[1])
		NavigationServer2D.obstacle_set_velocity(o[0], Vector2(c["vb"], 0))
	var d: Vector2 = alvo - pos
	var pref := Vector2.ZERO
	if d.length() > VMAX / Engine.physics_ticks_per_second:
		pref = d.normalized() * VMAX
	NavigationServer2D.agent_set_position(ag, pos)
	NavigationServer2D.agent_set_velocity_forced(ag, vel)
	NavigationServer2D.agent_set_velocity(ag, pref)
	if frame % 60 == 0:
		print("  q %d agente (%.1f, %.1f) barreira x %.1f" % [frame, pos.x, pos.y, bx])
	if frame >= PASSOS or d.length() < 2.0:
		print("FIM %s quadro %d agente (%.1f, %.1f) chegou %s" % [c["nome"], frame, pos.x, pos.y, str(d.length() < 2.0)])
		NavigationServer2D.free_rid(ag)
		for o in obs: NavigationServer2D.free_rid(o[0])
		NavigationServer2D.free_rid(map)
		cena_i += 1
		if cena_i >= cenas.size():
			return true
		comeca()
	return false
