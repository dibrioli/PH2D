extends SceneTree
# Sonda das PROPRIEDADES (CodeEditor): o Godot corrido como oráculo de caixa preta, nunca lido (CLAUDE.md §0.9).
# Cada caso imprime em stderr «@@CASO <rótulo>» ANTES da operação e «@@R <json>» depois; o que o motor imprimir
# no meio (ERROR/WARNING) é recolhido pelo resume.py como o «stderr» do caso.

var porta: Sprite2D
var camera: Camera2D
var heroi: Node2D

const USOS := {
	"STORAGE": PROPERTY_USAGE_STORAGE, "EDITOR": PROPERTY_USAGE_EDITOR, "INTERNAL": PROPERTY_USAGE_INTERNAL,
	"CHECKABLE": PROPERTY_USAGE_CHECKABLE, "CHECKED": PROPERTY_USAGE_CHECKED, "GROUP": PROPERTY_USAGE_GROUP,
	"CATEGORY": PROPERTY_USAGE_CATEGORY, "SUBGROUP": PROPERTY_USAGE_SUBGROUP,
	"CLASS_IS_BITFIELD": PROPERTY_USAGE_CLASS_IS_BITFIELD, "NO_INSTANCE_STATE": PROPERTY_USAGE_NO_INSTANCE_STATE,
	"RESTART_IF_CHANGED": PROPERTY_USAGE_RESTART_IF_CHANGED, "SCRIPT_VARIABLE": PROPERTY_USAGE_SCRIPT_VARIABLE,
	"STORE_IF_NULL": PROPERTY_USAGE_STORE_IF_NULL, "UPDATE_ALL_IF_MODIFIED": PROPERTY_USAGE_UPDATE_ALL_IF_MODIFIED,
	"CLASS_IS_ENUM": PROPERTY_USAGE_CLASS_IS_ENUM, "NIL_IS_VARIANT": PROPERTY_USAGE_NIL_IS_VARIANT,
	"ARRAY": PROPERTY_USAGE_ARRAY, "ALWAYS_DUPLICATE": PROPERTY_USAGE_ALWAYS_DUPLICATE,
	"NEVER_DUPLICATE": PROPERTY_USAGE_NEVER_DUPLICATE, "HIGH_END_GFX": PROPERTY_USAGE_HIGH_END_GFX,
	"NODE_PATH_FROM_SCENE_ROOT": PROPERTY_USAGE_NODE_PATH_FROM_SCENE_ROOT,
	"RESOURCE_NOT_PERSISTENT": PROPERTY_USAGE_RESOURCE_NOT_PERSISTENT,
	"KEYING_INCREMENTS": PROPERTY_USAGE_KEYING_INCREMENTS, "DEFERRED_SET_RESOURCE": PROPERTY_USAGE_DEFERRED_SET_RESOURCE,
	"EDITOR_INSTANTIATE_OBJECT": PROPERTY_USAGE_EDITOR_INSTANTIATE_OBJECT,
	"EDITOR_BASIC_SETTING": PROPERTY_USAGE_EDITOR_BASIC_SETTING, "READ_ONLY": PROPERTY_USAGE_READ_ONLY,
	"SECRET": PROPERTY_USAGE_SECRET,
}
const DICAS := {
	PROPERTY_HINT_NONE: "NONE", PROPERTY_HINT_RANGE: "RANGE", PROPERTY_HINT_ENUM: "ENUM",
	PROPERTY_HINT_FLAGS: "FLAGS", PROPERTY_HINT_RESOURCE_TYPE: "RESOURCE_TYPE", PROPERTY_HINT_NODE_TYPE: "NODE_TYPE",
	PROPERTY_HINT_LINK: "LINK", PROPERTY_HINT_COLOR_NO_ALPHA: "COLOR_NO_ALPHA",
	PROPERTY_HINT_LAYERS_2D_RENDER: "LAYERS_2D_RENDER", PROPERTY_HINT_ENUM_SUGGESTION: "ENUM_SUGGESTION",
	PROPERTY_HINT_EXP_EASING: "EXP_EASING", PROPERTY_HINT_NODE_PATH_VALID_TYPES: "NODE_PATH_VALID_TYPES",
	PROPERTY_HINT_TYPE_STRING: "TYPE_STRING", PROPERTY_HINT_MULTILINE_TEXT: "MULTILINE_TEXT",
}

func usos(u: int) -> Array:
	var r := []
	for k in USOS:
		if u & USOS[k]:
			r.append(k)
	return r

func val(x) -> Dictionary:
	var d := {"tipo": type_string(typeof(x)), "str": str(x)}
	if x is Object and x != null:
		d["classe"] = x.get_class()
		if x is Resource:
			d["resource_path"] = x.resource_path
		if x is Node:
			d["nome"] = String(x.name)
	return d

func caso(rot: String) -> void:
	printerr("@@CASO " + rot)

func r(d: Dictionary) -> void:
	printerr("@@R " + JSON.stringify(d, "", false))

func lista(rot: String, o: Object) -> void:
	caso("a. get_property_list " + rot)
	var pl := o.get_property_list()
	var ed := 0
	var linhas := []
	for p in pl:
		if p.usage & PROPERTY_USAGE_EDITOR:
			ed += 1
		linhas.append([p.name, type_string(p.type), p.class_name, usos(p.usage), DICAS.get(p.hint, str(p.hint)), p.hint_string])
	r({"total": pl.size(), "com_EDITOR": ed, "props": linhas})

func _initialize() -> void:
	var main := Node2D.new()
	main.name = "Main"
	root.add_child(main)
	porta = Sprite2D.new()
	porta.name = "Porta"
	porta.texture = load("res://icon.svg")
	main.add_child(porta)
	camera = Camera2D.new()
	camera.name = "Camera"
	main.add_child(camera)
	heroi = Node2D.new()
	heroi.name = "Heroi"
	heroi.set_script(load("res://heroi.gd"))
	main.add_child(heroi)

	# a
	lista("Sprite2D", porta)
	lista("Camera2D", camera)
	lista("Heroi(heroi.gd)", heroi)

	# b
	caso("b. nome de get/set vs String.capitalize()")
	var cap := {}
	for n in ["texture", "zoom", "vida_max", "position", "modulate", "z_index", "texture_filter", "global_position", "region_rect", "hframes", "offset", "position_smoothing_speed", "cor", "alvo", "velocidade"]:
		cap[n] = n.capitalize()
	r({"capitalize": cap})

	# c
	caso("c1. get('texture')")
	r({"v": val(porta.get("texture"))})
	caso("c2. set('texture', load('res://icon.svg'))")
	porta.set("texture", load("res://icon.svg"))
	r({"depois": val(porta.get("texture"))})
	caso("c3. set('texture', 'res://icon.svg') (String)")
	porta.set("texture", "res://icon.svg")
	r({"depois": val(porta.get("texture"))})
	porta.texture = load("res://icon.svg")
	caso("c4. set('texture', null)")
	porta.set("texture", null)
	r({"depois": val(porta.get("texture"))})
	porta.texture = load("res://icon.svg")
	# d
	caso("d1. set('nao_existe', 5) em Sprite2D")
	porta.set("nao_existe", 5)
	r({"get_depois": val(porta.get("nao_existe")), "has_meta": porta.has_meta("nao_existe")})
	caso("d2. get('nao_existe') em Heroi")
	r({"v": val(heroi.get("nao_existe"))})
	caso("d3. set('nao_existe', 5) em Heroi")
	heroi.set("nao_existe", 5)
	r({"get_depois": val(heroi.get("nao_existe"))})

	# e
	caso("e1. Heroi.set('vida', 'abc')")
	heroi.set("vida", "abc")
	r({"vida": val(heroi.get("vida"))})
	heroi.set("vida", 100)
	caso("e2. Heroi.set('vida', 3.7)")
	heroi.set("vida", 3.7)
	r({"vida": val(heroi.get("vida"))})
	heroi.set("vida", 100)
	caso("e3. Heroi.set('vida', '42') (String numérica)")
	heroi.set("vida", "42")
	r({"vida": val(heroi.get("vida"))})
	heroi.set("vida", 100)
	caso("e4. Heroi.set('velocidade', 50) com @export_range(0,10)")
	heroi.set("velocidade", 50)
	r({"velocidade": val(heroi.get("velocidade"))})
	caso("e5. Heroi.set('velocidade', -3)")
	heroi.set("velocidade", -3)
	r({"velocidade": val(heroi.get("velocidade"))})
	caso("e6. Sprite2D.set('position', 5)")
	porta.set("position", 5)
	r({"position": val(porta.get("position"))})
	caso("e7. Sprite2D.set('position', 'abc')")
	porta.set("position", "abc")
	r({"position": val(porta.get("position"))})
	caso("e8. Sprite2D.set('z_index', 3.9) (int nativo)")
	porta.set("z_index", 3.9)
	r({"z_index": val(porta.get("z_index"))})
	caso("e9. Sprite2D.set('z_index', 99999) (fora do range do editor -4096..4096)")
	porta.set("z_index", 99999)
	r({"z_index": val(porta.get("z_index"))})
	porta.z_index = 0
	caso("e10. Sprite2D.set('modulate', 'red') (String de cor)")
	porta.set("modulate", "red")
	r({"modulate": val(porta.get("modulate"))})
	porta.modulate = Color.WHITE
	caso("e11. Heroi.set('cor', '#ff0000')")
	heroi.set("cor", "#ff0000")
	r({"cor": val(heroi.get("cor"))})
	caso("e12. Heroi.set('alvo', porta) com @export alvo: Node2D (Sprite2D é Node2D)")
	heroi.set("alvo", porta)
	r({"alvo": val(heroi.get("alvo"))})
	caso("e13. Heroi.set('alvo', Node.new()) (Node não é Node2D)")
	var solto := Node.new()
	heroi.set("alvo", solto)
	r({"alvo": val(heroi.get("alvo"))})
	solto.free()
	heroi.set("alvo", null)
	caso("e14. Sprite2D.set('hframes', 0) (setter nativo com validação)")
	porta.set("hframes", 0)
	r({"hframes": val(porta.get("hframes"))})
	porta.hframes = 1

	# f
	porta.position = Vector2(1, 2)
	caso("f1. get('position:x')")
	r({"v": val(porta.get("position:x"))})
	caso("f2. get_indexed('position:x')")
	r({"v": val(porta.get_indexed("position:x"))})
	caso("f3. set_indexed('position:x', 10)")
	porta.set_indexed("position:x", 10)
	r({"position": val(porta.position)})
	caso("f4. set('position:x', 20)")
	porta.set("position:x", 20)
	r({"position": val(porta.position)})
	caso("f5. set('position.x', 30)")
	porta.set("position.x", 30)
	r({"position": val(porta.position), "get_position.x": val(porta.get("position.x"))})
	caso("f6. get_indexed('position.x')")
	r({"v": val(porta.get_indexed("position.x"))})
	caso("f7. get_indexed('texture:resource_path')")
	r({"v": val(porta.get_indexed("texture:resource_path"))})
	caso("f8. get_indexed('position:z') (componente inexistente)")
	r({"v": val(porta.get_indexed("position:z"))})
	caso("f9. set_indexed('modulate:a', 0.5)")
	porta.set_indexed("modulate:a", 0.5)
	r({"modulate": val(porta.modulate)})
	porta.modulate = Color.WHITE

	# g
	caso("g1. Camera2D get('zoom')")
	r({"v": val(camera.get("zoom"))})
	caso("g2. Camera2D set('zoom', 2)")
	camera.set("zoom", 2)
	r({"v": val(camera.get("zoom"))})
	camera.zoom = Vector2.ONE
	caso("g3. Camera2D set('zoom', 2.5)")
	camera.set("zoom", 2.5)
	r({"v": val(camera.get("zoom"))})
	camera.zoom = Vector2.ONE
	caso("g4. Camera2D set('zoom', Vector2(0,0))")
	camera.set("zoom", Vector2(0, 0))
	r({"v": val(camera.get("zoom"))})
	camera.zoom = Vector2.ONE

	# h
	heroi.vida_max = 300
	camera.zoom = Vector2(3, 3)
	porta.position = Vector2(7, 7)
	for par in [[heroi, "vida_max"], [heroi, "velocidade"], [heroi, "cor"], [heroi, "alvo"], [heroi, "nao_exportada"],
			[camera, "zoom"], [porta, "position"], [porta, "texture"], [porta, "nao_existe"]]:
		caso("h. property_can_revert/get_revert %s.%s" % [par[0].name, par[1]])
		var o: Object = par[0]
		r({"atual": val(o.get(par[1])), "can_revert": o.property_can_revert(par[1]), "revert": val(o.property_get_revert(par[1]))})
	caso("h2. ClassDB.class_get_property_default_value Camera2D.zoom / Sprite2D.position")
	r({"Camera2D.zoom": val(ClassDB.class_get_property_default_value("Camera2D", "zoom")),
		"Sprite2D.position": val(ClassDB.class_get_property_default_value("Sprite2D", "position"))})
	caso("h3. Script.get_property_default_value vida_max / velocidade")
	var sc: Script = heroi.get_script()
	r({"vida_max": val(sc.get_property_default_value("vida_max")), "velocidade": val(sc.get_property_default_value("velocidade")),
		"cor": val(sc.get_property_default_value("cor")), "alvo": val(sc.get_property_default_value("alvo"))})

	# i
	caso("i. heroi.get_node('../Porta').set('modulate', Color.RED)")
	heroi.get_node("../Porta").set("modulate", Color.RED)
	r({"modulate": val(porta.modulate)})
	caso("i2. heroi.get_node('../NaoExiste') (caminho errado)")
	var nn = heroi.get_node_or_null("../NaoExiste")
	var n2 = heroi.get_node("../NaoExiste")
	r({"get_node_or_null": val(nn), "get_node": val(n2)})

	# j
	heroi.alvo = null
	caso("j1. get('alvo') não definido")
	r({"v": val(heroi.get("alvo"))})
	caso("j2. set('alvo', camera)")
	heroi.set("alvo", camera)
	r({"v": val(heroi.get("alvo"))})
	caso("j3. alvo depois de camera.free() (instância liberada)")
	var cam2 := Camera2D.new()
	main.add_child(cam2)
	heroi.alvo = cam2
	cam2.free()
	var a = heroi.get("alvo")
	r({"is_instance_valid": is_instance_valid(a), "typeof": type_string(typeof(a)), "str": str(a)})

	# k
	caso("k0. propriedades com READ_ONLY nas três listas e em classes comuns")
	var ro := []
	for o in [porta, camera, heroi, Label.new(), CharacterBody2D.new(), Timer.new(), AnimationPlayer.new(), Area2D.new()]:
		for p in o.get_property_list():
			if p.usage & PROPERTY_USAGE_READ_ONLY:
				ro.append(o.get_class() + "." + p.name)
	r({"read_only": ro})
	caso("k1. Heroi.set('so_leitura', 9) (var só com get)")
	heroi.set("so_leitura", 9)
	r({"v": val(heroi.get("so_leitura"))})
	caso("k2. Node.set('name', 'Porta2') (escrevível)")
	porta.set("name", "Porta2")
	r({"name": val(porta.name)})
	porta.name = "Porta"
	caso("k3. Node.set('scene_file_path', 'res://x.tscn')")
	porta.set("scene_file_path", "res://x.tscn")
	r({"scene_file_path": val(porta.get("scene_file_path"))})
	caso("k4. Timer.set('time_left', 3.0) (time_left sem setter)")
	var t := Timer.new()
	main.add_child(t)
	t.set("time_left", 3.0)
	r({"time_left": val(t.get("time_left"))})
	caso("k5. AnimationPlayer.set('current_animation_length', 9) (só leitura)")
	var ap := AnimationPlayer.new()
	main.add_child(ap)
	ap.set("current_animation_length", 9.0)
	r({"v": val(ap.get("current_animation_length"))})
	caso("k6. Sprite2D.set('global_position', Vector2(5,5)) (deriva do pai)")
	porta.set("global_position", Vector2(5, 5))
	r({"position": val(porta.position), "global_position": val(porta.global_position)})

	caso("fim")
	r({})
	quit()
