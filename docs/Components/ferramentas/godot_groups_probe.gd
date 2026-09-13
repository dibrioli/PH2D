# ORÁCULO: os grupos do Godot 4.7.2 (MIT), corridos sem interface — docs/Components/08_plano_tags.md §1.
# Correr: godot --headless --script docs/Components/ferramentas/godot_groups_probe.gd
# Mede: ordem de get_nodes_in_group · duplicado · caixa · hierarquia por ponto · call_group · sair da árvore · persistência numa PackedScene · custo a 1k/10k/100k.
# ⚠️ Corre no 1.º _process e não no _initialize: antes disso os nós não estão na árvore e toda consulta devolve [] (medido — a 1.ª versão leu zeros).
extends SceneTree

func _names(arr) -> Array:
	var out = []
	for n in arr:
		out.append(str(n.name))
	return out

var _done := false
func _process(_delta):
	if _done:
		return true
	_done = true
	_run()
	return true

func _run():
	print("GODOT ", Engine.get_version_info().string)
	var r = Node.new(); r.name = "R"; get_root().add_child(r)
	var a = Node.new(); a.name = "A"
	var b = Node.new(); b.name = "B"
	var c = Node.new(); c.name = "C"
	r.add_child(a); r.add_child(b); a.add_child(c)
	# 1) ordem: insercao B, C, A — arvore e' A, C, B
	b.add_to_group("enemy"); c.add_to_group("enemy"); a.add_to_group("enemy")
	print("ORDER_INSERT_B_C_A ", _names(get_nodes_in_group("enemy")))
	# 2) duplicado
	a.add_to_group("enemy")
	print("COUNT_AFTER_DUP ", get_node_count_in_group("enemy"))
	# 3) caixa
	print("CASE Enemy=", a.is_in_group("Enemy"), " enemy=", a.is_in_group("enemy"))
	# 4) hierarquia: um grupo com ponto casa o pai?
	b.add_to_group("enemy.flying")
	print("HIER b_in_enemy.flying=", b.is_in_group("enemy.flying"), " nodes_in_enemy.flying=", _names(get_nodes_in_group("enemy.flying")))
	c.remove_from_group("enemy")
	c.add_to_group("enemy.flying.boss")
	print("HIER_PARENT query enemy=", _names(get_nodes_in_group("enemy")), " query enemy.flying=", _names(get_nodes_in_group("enemy.flying")))
	# 5) espacos e vazio
	a.add_to_group("with space")
	print("SPACE ", a.is_in_group("with space"))
	print("GROUPS_A ", a.get_groups())
	# 6) call_group: ordem de chamada
	var s = GDScript.new()
	s.source_code = "extends Node\nfunc ping(l):\n\tl.append(str(name))\n"
	s.reload()
	for n in [a, b, c]:
		n.set_script(s)
	var log = []
	call_group("enemy.flying", "ping", log)
	call_group("enemy", "ping", log)
	print("CALL_ORDER enemy.flying+enemy ", log)
	# 7) sair da arvore
	r.remove_child(b)
	print("AFTER_REMOVE nodes=", _names(get_nodes_in_group("enemy")), " b.is_in_group=", b.is_in_group("enemy"))
	r.add_child(b)
	print("AFTER_READD nodes=", _names(get_nodes_in_group("enemy")))
	# 8) persistencia numa PackedScene
	var p = Node.new(); p.name = "P"
	var q = Node.new(); q.name = "Q"
	p.add_child(q); q.owner = p
	q.add_to_group("saved", true)
	q.add_to_group("volatile", false)
	var ps = PackedScene.new(); ps.pack(p)
	var inst = ps.instantiate()
	print("PERSIST ", inst.get_node("Q").get_groups())
	# 9) grupos globais do projecto
	var gg = []
	for prop in ProjectSettings.get_property_list():
		if str(prop.name).begins_with("global_group"):
			gg.append(str(prop.name))
	print("GLOBAL_GROUP_SETTINGS ", gg)
	# 10) custo: N nos no grupo, consulta e call_group
	for N in [1000, 10000, 100000]:
		var holder = Node.new(); get_root().add_child(holder)
		var t0 = Time.get_ticks_usec()
		for i in N:
			var m = Node.new()
			holder.add_child(m)
			m.add_to_group("mob")
		var t1 = Time.get_ticks_usec()
		var got = get_nodes_in_group("mob")
		var t2 = Time.get_ticks_usec()
		var got2 = get_nodes_in_group("mob")
		var t3 = Time.get_ticks_usec()
		print("COST N=", N, " add_ms=", (t1-t0)/1000.0, " first_query_ms=", (t2-t1)/1000.0, " second_query_ms=", (t3-t2)/1000.0, " count=", got.size(), "/", got2.size())
		holder.queue_free()
		for m in got:
			m.remove_from_group("mob")

