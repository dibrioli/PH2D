# ⭐ ORÁCULO do TOP-20 #15 (`StateMachine`) — Godot 4.7.2, **MIT** (arsenal §2: porta ABERTA,
# porta de consola medida `godot --headless --script <s.gd> --quit`).
#
# ⚠️ **Corre-se, não se lê** (CLAUDE.md §0.9). O que se colhe é a LEI DE ARBITRAGEM — a pergunta
# que nenhum doc responde e que decide o nosso desenho:
#
#   1. Duas transições do MESMO estado, as duas satisfeitas: qual ganha?
#   2. Com a MESMA prioridade: ganha a primeira declarada?
#   3. Uma transição acontece no tique em que fica verdadeira, ou no seguinte?
#   4. Quantos estados se atravessam num tique só (a cadeia A→B→C toda auto)?
#   5. Um estado pode transitar para SI MESMO?
#
# Uso: godot --headless --script docs/Components/ferramentas/godot_statemachine_probe.gd --quit
extends SceneTree

var _linhas: Array[String] = []

func _log(s: String) -> void:
	_linhas.append(s)
	print(s)

func _monta(nomes: Array) -> Array:
	var raiz := Node2D.new()
	get_root().add_child(raiz)
	var ap := AnimationPlayer.new()
	ap.name = "AP"
	raiz.add_child(ap)
	var lib := AnimationLibrary.new()
	for n in nomes:
		var a := Animation.new()
		a.length = 1.0
		lib.add_animation(n, a)
	ap.add_animation_library("", lib)

	var sm := AnimationNodeStateMachine.new()
	var i := 0
	for n in nomes:
		var no := AnimationNodeAnimation.new()
		no.animation = n
		sm.add_node(n, no, Vector2(i * 200, 0))
		i += 1

	var tree := AnimationTree.new()
	tree.name = "TREE"
	raiz.add_child(tree)
	tree.anim_player = tree.get_path_to(ap)
	tree.tree_root = sm
	tree.callback_mode_process = AnimationMixer.ANIMATION_CALLBACK_MODE_PROCESS_MANUAL
	tree.active = true
	return [raiz, sm, tree]

func _trans(auto: bool, prio: int, cond: String = "") -> AnimationNodeStateMachineTransition:
	var t := AnimationNodeStateMachineTransition.new()
	t.switch_mode = AnimationNodeStateMachineTransition.SWITCH_MODE_IMMEDIATE
	t.advance_mode = (AnimationNodeStateMachineTransition.ADVANCE_MODE_AUTO if auto
		else AnimationNodeStateMachineTransition.ADVANCE_MODE_ENABLED)
	t.priority = prio
	t.xfade_time = 0.0
	if cond != "":
		t.advance_condition = cond
	return t

func _estado(tree: AnimationTree) -> String:
	var pb = tree.get("parameters/playback")
	return str(pb.get_current_node())

# ⚠️ **ARRANQUE EXPLICITO.** A maquina do Godot tem um `Start` implicito, e sem esta chamada a
# sonda mede o VAZIO: a 1.a redacao leu «Start» nas CINCO perguntas e teria sido lida como
# «o Godot nao transita» (a lei do oraculo: uma fixtura que o alvo recusa nunca e' comparada).
func _arranca(tree: AnimationTree, estado: String) -> void:
	var pb = tree.get("parameters/playback")
	pb.start(estado)

func _initialize() -> void:
	_log("# oraculo: Godot 4.7.2 (MIT) · AnimationNodeStateMachine · headless")
	_log("# grandeza: o NOME do estado corrente depois de N avancos de 1/60 s")
	_log("")

	# ── 1+2. ARBITRAGEM: duas transicoes do mesmo estado, as duas satisfeitas ──
	for caso in [[1, 1], [1, 5], [5, 1]]:
		var m := _monta(["A", "B", "C"])
		var sm: AnimationNodeStateMachine = m[1]
		var tree: AnimationTree = m[2]
		# A→B declarada PRIMEIRO, A→C a seguir. As duas AUTO (sempre satisfeitas).
		sm.add_transition("A", "B", _trans(true, caso[0]))
		sm.add_transition("A", "C", _trans(true, caso[1]))
		_arranca(tree, "A")
		tree.advance(1.0 / 60.0)
		_log("arbitragem prio(A->B)=%d prio(A->C)=%d ⇒ %s" % [caso[0], caso[1], _estado(tree)])
		m[0].queue_free()
	_log("")

	# ── 3. QUANDO: o tique em que fica verdadeira, ou o seguinte? ──────────────
	var m2 := _monta(["A", "B"])
	var sm2: AnimationNodeStateMachine = m2[1]
	var tree2: AnimationTree = m2[2]
	sm2.add_transition("A", "B", _trans(false, 1, "ir"))
	_arranca(tree2, "A")
	_log("quando: antes de qualquer avanco ⇒ %s" % _estado(tree2))
	tree2.set("parameters/conditions/ir", true)
	tree2.advance(1.0 / 60.0)
	_log("quando: condicao a VERDADE, 1 avanco ⇒ %s" % _estado(tree2))
	m2[0].queue_free()
	_log("")

	# ── 4. CADEIA: quantos estados num avanco so'? ────────────────────────────
	var m3 := _monta(["A", "B", "C", "D", "E"])
	var sm3: AnimationNodeStateMachine = m3[1]
	var tree3: AnimationTree = m3[2]
	sm3.add_transition("A", "B", _trans(true, 1))
	sm3.add_transition("B", "C", _trans(true, 1))
	sm3.add_transition("C", "D", _trans(true, 1))
	sm3.add_transition("D", "E", _trans(true, 1))
	_arranca(tree3, "A")
	for n in range(1, 5):
		tree3.advance(1.0 / 60.0)
		_log("cadeia: depois de %d avanco(s) ⇒ %s" % [n, _estado(tree3)])
	m3[0].queue_free()
	_log("")

	# ── 3-bis. ⚠️ O MODO manda na condicao — e uma condicao no modo errado e' INERTE ──
	# A 1.a redacao desta sonda escreveu a condicao num `ADVANCE_MODE_ENABLED` e leu «nao
	# transita». Nao era a lei do alvo: era um knob morto por construcao. Mede-se AGORA nos dois.
	for modo_auto in [false, true]:
		var m2b := _monta(["A", "B"])
		var sm2b: AnimationNodeStateMachine = m2b[1]
		var tree2b: AnimationTree = m2b[2]
		sm2b.add_transition("A", "B", _trans(modo_auto, 1, "ir"))
		_arranca(tree2b, "A")
		tree2b.advance(1.0 / 60.0)
		var antes := _estado(tree2b)
		tree2b.set("parameters/conditions/ir", true)
		tree2b.advance(1.0 / 60.0)
		var depois := _estado(tree2b)
		_log("condicao em advance_mode=%s: falsa ⇒ %s · verdadeira (1 avanco) ⇒ %s"
			% ["AUTO" if modo_auto else "ENABLED", antes, depois])
		m2b[0].queue_free()
	_log("")

	# ── 4-bis. ⭐⭐⭐ O CICLO: A→B→A, as duas AUTO. O oraculo tem orcamento? ────
	# Esta e' a pergunta que decide o nosso desenho: o doc do `SignalActions` desta casa preve
	# por escrito que um verbo que EMITA um sinal «entra com um orcamento de profundidade».
	var m5 := _monta(["A", "B"])
	var sm5: AnimationNodeStateMachine = m5[1]
	var tree5: AnimationTree = m5[2]
	sm5.add_transition("A", "B", _trans(true, 1))
	sm5.add_transition("B", "A", _trans(true, 1))
	_arranca(tree5, "A")
	var t0 := Time.get_ticks_usec()
	tree5.advance(1.0 / 60.0)
	var dt_us := Time.get_ticks_usec() - t0
	_log("ciclo A<->B (as duas auto): 1 avanco ⇒ %s, custou %d us (nao pendurou)"
		% [_estado(tree5), dt_us])
	var vistos := {}
	for n in range(0, 6):
		tree5.advance(1.0 / 60.0)
		vistos[_estado(tree5)] = true
		_log("ciclo: avanco %d ⇒ %s" % [n + 2, _estado(tree5)])
	_log("ciclo: estados distintos visitados em 7 avancos = %d" % vistos.size())
	m5[0].queue_free()
	_log("")

	# ── 5. AUTO-TRANSICAO ─────────────────────────────────────────────────────
	var m4 := _monta(["A", "B"])
	var sm4: AnimationNodeStateMachine = m4[1]
	var tree4: AnimationTree = m4[2]
	sm4.add_transition("A", "A", _trans(true, 1))
	_arranca(tree4, "A")
	tree4.advance(1.0 / 60.0)
	_log("auto-transicao A->A declarada ⇒ %s (existe? %s)"
		% [_estado(tree4), str(sm4.has_transition("A", "A"))])
	m4[0].queue_free()

	quit()
