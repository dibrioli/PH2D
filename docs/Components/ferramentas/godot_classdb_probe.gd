# ⭐ ORÁCULO DO CATÁLOGO — Godot 4.7.2, **MIT** (arsenal §2: porta ABERTA, consola
# `godot --headless --script <s.gd> --quit`).
#
# ⚠️ **Corre-se, não se lê** (CLAUDE.md §0.9). O dossiê de 2026-08-20 foi escrito LENDO
# docs.godotengine.org: ele acerta nos nós famosos e não pode responder *«o que é que ela TEM?»*,
# porque uma leitura não enumera — ela recorda. Esta sonda pergunta ao **ClassDB do binário
# instalado**, que é a base de dados que o próprio editor usa para desenhar o diálogo «Create New
# Node» das capturas do dono.
#
# O que ela colhe, por classe:
#   · o PAI e a cadeia de herança até `Object`  (⇒ a família: Node2D · Control · Resource · Server…)
#   · se é INSTANCIÁVEL                          (⇒ uma classe abstracta não é um objecto)
#   · as PROPRIEDADES **declaradas por ela**     (`no_inheritance = true` — sem isso, todo nó 2D
#                                                 herdaria as ~30 do CanvasItem e a contagem mentia)
#   · os SINAIS e os MÉTODOS declarados por ela
#   · o tipo e a DICA de cada propriedade        (⇒ é isso que decide a fileira do nosso Inspector)
#
# ⛔ O que ela NÃO colhe: a prosa. Uma descrição vive na documentação embutida e sai por
# `--doctool`; as duas correm lado a lado e o documento cita a medida, nunca a memória.
#
# Uso:
#   godot --headless --script docs/Components/ferramentas/godot_classdb_probe.gd --quit -- <saida>
extends SceneTree

func _init() -> void:
	var args := OS.get_cmdline_user_args()
	var dir := args[0] if args.size() > 0 else "/tmp/godot_classdb"
	DirAccess.make_dir_recursive_absolute(dir)

	var classes := ClassDB.get_class_list()
	classes.sort()

	# (1) O CENSO — uma linha por classe.
	var censo := FileAccess.open(dir + "/censo.tsv", FileAccess.WRITE)
	censo.store_line("classe\tpai\tinstanciavel\tfamilia\tprops\tsinais\tmetodos")
	for c in classes:
		var pai := ClassDB.get_parent_class(c)
		var props := ClassDB.class_get_property_list(c, true)
		var sinais := ClassDB.class_get_signal_list(c, true)
		var metodos := ClassDB.class_get_method_list(c, true)
		censo.store_line("%s\t%s\t%s\t%s\t%d\t%d\t%d" % [
			c, pai, str(ClassDB.can_instantiate(c)), _familia(c),
			props.size(), sinais.size(), metodos.size()])
	censo.close()

	# (2) A SUPERFÍCIE — propriedades com tipo e dica, sinais, métodos, por classe.
	var sup := FileAccess.open(dir + "/superficie.txt", FileAccess.WRITE)
	for c in classes:
		var props := ClassDB.class_get_property_list(c, true)
		var sinais := ClassDB.class_get_signal_list(c, true)
		if props.is_empty() and sinais.is_empty():
			continue
		sup.store_line("== %s (%s) [%s]" % [c, ClassDB.get_parent_class(c), _familia(c)])
		for p in props:
			# O grupo/categoria do inspector chega como uma pseudo-propriedade; ela diz como o
			# ALVO arruma o painel, e por isso fica — marcada.
			var usage: int = p["usage"]
			if usage & PROPERTY_USAGE_CATEGORY:
				sup.store_line("  # CATEGORIA %s" % p["name"])
				continue
			if usage & PROPERTY_USAGE_GROUP:
				sup.store_line("  # GRUPO %s" % p["name"])
				continue
			if usage & PROPERTY_USAGE_SUBGROUP:
				sup.store_line("  # SUBGRUPO %s" % p["name"])
				continue
			if not (usage & PROPERTY_USAGE_EDITOR):
				continue
			sup.store_line("  . %s : %s%s" % [
				p["name"], _tipo(p), _dica(p)])
		for s in sinais:
			var argn := PackedStringArray()
			for a in s["args"]:
				argn.append(str(a["name"]))
			sup.store_line("  ! sinal %s(%s)" % [s["name"], ", ".join(argn)])
		sup.store_line("")
	sup.close()

	print("classes: %d" % classes.size())
	print("saida: %s" % dir)

## A raiz da cadeia de herança que decide de que família a classe é.
func _familia(c: String) -> String:
	for raiz in ["Node2D", "Control", "Node3D", "CanvasItem", "Node", "Resource",
			"RefCounted", "Object"]:
		if c == raiz or ClassDB.is_parent_class(c, raiz):
			return raiz
	return "?"

func _tipo(p: Dictionary) -> String:
	var t: int = p["type"]
	if t == TYPE_OBJECT and p["class_name"] != "":
		return str(p["class_name"])
	return type_string(t)

func _dica(p: Dictionary) -> String:
	var h: int = p["hint"]
	var hs: String = str(p["hint_string"])
	if h == PROPERTY_HINT_RANGE:
		return "  faixa(%s)" % hs
	if h == PROPERTY_HINT_ENUM:
		return "  enum(%s)" % hs
	if h == PROPERTY_HINT_FLAGS:
		return "  flags(%s)" % hs
	if h == PROPERTY_HINT_RESOURCE_TYPE:
		return "  recurso(%s)" % hs
	if h == PROPERTY_HINT_FILE or h == PROPERTY_HINT_DIR:
		return "  ficheiro(%s)" % hs
	if h == PROPERTY_HINT_LAYERS_2D_PHYSICS:
		return "  camadas2d"
	return ""
