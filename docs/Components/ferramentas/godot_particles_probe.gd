# ORÁCULO: o RELÓGIO de um emissor de partículas no Godot 4.7.2 (MIT), corrido sem interface —
# docs/Components/14_plano_particle_emitter.md §2.
# Correr: godot --headless --fixed-fps 60 --script docs/Components/ferramentas/godot_particles_probe.gd
#
# ⚠️ A API do CPUParticles2D NÃO expõe as posições das partículas (medido pelo --doctool: 11 métodos,
# nenhum getter de partícula). O que se lê sem interface é o RELÓGIO — `emitting` e o sinal
# `finished` —, e é isso que um componente precisa de copiar: quando uma rajada acaba, e o que
# ligar/desligar faz. ⚠️ `--fixed-fps 60` é o que torna o quadro uma unidade de tempo exacta.
#
# Blocos:
#   A) rajada única (`one_shot`): o quadro do `finished` e o do `emitting = false`, por
#      (lifetime, explosiveness, preprocess, speed_scale, lifetime_randomness)
#   B) contínuo: desligar `emitting` a meio — `finished` dispara? quando?
#   C) religar depois de desligar — recomeça do zero ou continua?
#   D) `restart()` a meio de uma rajada única — o relógio volta a zero?
#   E) `amount = 0` e `lifetime` mínimo — o que o motor aceita
#   C0) CONTROLO: um nó que não emite nunca dispara `finished`.
extends SceneTree

const FPS := 60

var frame := 0
var root2d: Node2D
var casos := []      # [{nome, node, fim_frame, desligou_frame, emit_log}]

func _caso(nome: String, cfg: Dictionary) -> Dictionary:
	var p := CPUParticles2D.new()
	p.name = nome
	p.amount = cfg.get("amount", 8)
	p.lifetime = cfg.get("lifetime", 1.0)
	p.one_shot = cfg.get("one_shot", true)
	p.explosiveness = cfg.get("explosiveness", 0.0)
	p.preprocess = cfg.get("preprocess", 0.0)
	p.speed_scale = cfg.get("speed_scale", 1.0)
	p.lifetime_randomness = cfg.get("lifetime_randomness", 0.0)
	p.use_fixed_seed = true
	p.seed = 7
	p.emitting = cfg.get("emitting", true)
	var c := {"nome": nome, "node": p, "fim": [], "desligou": -1, "cfg": cfg}
	p.finished.connect(func(): c["fim"].append(frame))
	root2d.add_child(p)
	casos.append(c)
	return c

func _process(_delta):
	frame += 1
	if root2d == null:
		root2d = Node2D.new()
		root.add_child(root2d)
		# A) rajada única
		for lt in [1.0, 0.5]:
			for ex in [0.0, 0.5, 1.0]:
				_caso("A_lt%s_ex%s" % [lt, ex], {"lifetime": lt, "explosiveness": ex})
		_caso("A_pre0.5", {"lifetime": 1.0, "preprocess": 0.5})
		_caso("A_pre1.5", {"lifetime": 1.0, "preprocess": 1.5})
		_caso("A_speed2", {"lifetime": 1.0, "speed_scale": 2.0})
		_caso("A_rand0.5", {"lifetime": 1.0, "lifetime_randomness": 0.5})
		_caso("A_amount1", {"lifetime": 1.0, "amount": 1})
		# B) contínuo, desligado no quadro 30
		_caso("B_cont_off30", {"lifetime": 1.0, "one_shot": false})
		# C) contínuo, desligado no 30 e religado no 45
		_caso("C_cont_off30_on45", {"lifetime": 1.0, "one_shot": false})
		# D) rajada única com restart no quadro 30
		_caso("D_restart30", {"lifetime": 1.0})
		# E) limites
		_caso("E_amount_min", {"lifetime": 1.0, "amount": 1, "explosiveness": 1.0})
		# C0) controlo: nunca emite
		_caso("C0_off", {"lifetime": 1.0, "emitting": false})
		return false
	for c in casos:
		var p: CPUParticles2D = c["node"]
		if c["desligou"] < 0 and not p.emitting and c["cfg"].get("emitting", true):
			c["desligou"] = frame
		if frame == 30 and (c["nome"] == "B_cont_off30" or c["nome"] == "C_cont_off30_on45"):
			p.emitting = false
		if frame == 45 and c["nome"] == "C_cont_off30_on45":
			p.emitting = true
			c["religou_emitting_apos"] = p.emitting
		if frame == 30 and c["nome"] == "D_restart30":
			p.restart()
	if frame == 60 * 5:
		for c in casos:
			var p: CPUParticles2D = c["node"]
			print(c["nome"], " finished_frames=", c["fim"], " emitting_false_frame=", c["desligou"],
				" emitting_now=", p.emitting, " amount=", p.amount, " lifetime=", p.lifetime)
		print("FIM frames=", frame, " fps=", FPS)
		quit()
	return false
