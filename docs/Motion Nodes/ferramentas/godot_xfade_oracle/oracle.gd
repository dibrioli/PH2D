# O ORÁCULO do crossfade (ciclo 19, doc 128 §5.2): o `AnimationNodeTransition` do Godot com
# `xfade_time`, marchado à mão a 60 Hz, sobre DUAS animações nossas (uma propriedade parada em 0 e
# a outra em 1). O valor da propriedade a cada quadro É o peso do estado novo.
extends SceneTree

const DT := 1.0 / 60.0

func corre(xfade: float, quadros: int, pedido_no: int) -> void:
	var raiz := Node2D.new()
	get_root().add_child(raiz)
	var alvo := Node2D.new()
	alvo.name = "T"
	raiz.add_child(alvo)
	var lib := AnimationLibrary.new()
	for par in [["A", 0.0], ["B", 1.0]]:
		var a := Animation.new()
		a.length = 100.0
		var tr := a.add_track(Animation.TYPE_VALUE)
		a.track_set_path(tr, NodePath("T:rotation"))
		a.track_insert_key(tr, 0.0, par[1])
		a.track_insert_key(tr, 100.0, par[1])
		lib.add_animation(par[0], a)
	var bt := AnimationNodeBlendTree.new()
	for nome in ["A", "B"]:
		var n := AnimationNodeAnimation.new()
		n.animation = nome
		bt.add_node(nome, n)
	var tn := AnimationNodeTransition.new()
	tn.xfade_time = xfade
	tn.add_input("A")
	tn.add_input("B")
	bt.add_node("X", tn)
	bt.connect_node("X", 0, "A")
	bt.connect_node("X", 1, "B")
	bt.connect_node("output", 0, "X")
	var tree := AnimationTree.new()
	raiz.add_child(tree)
	tree.add_animation_library("", lib)
	tree.root_node = tree.get_path_to(raiz)
	tree.tree_root = bt
	tree.callback_mode_process = AnimationMixer.ANIMATION_CALLBACK_MODE_PROCESS_MANUAL
	tree.active = true
	# Sem estado corrente (índice -1) o primeiro pedido não cruza nada: entra-se em A primeiro.
	tree.set("parameters/X/transition_request", "A")
	var linha := PackedStringArray()
	for k in range(quadros):
		if k == pedido_no:
			tree.set("parameters/X/transition_request", "B")
		tree.advance(DT)
		linha.append("%.7f" % alvo.rotation)
	print("XFADE %.3f pedido_no %d | %s" % [xfade, pedido_no, " ".join(linha)])
	raiz.queue_free()

func _initialize() -> void:
	for xfade in [0.25, 0.5, 1.0]:
		corre(xfade, 80, 5)
	quit()
