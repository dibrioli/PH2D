@tool
extends EditorPlugin
# Sonda opcional (b): o editor (headless) inspeciona Sprite2D, Camera2D e o Heroi e lê, de cada EditorProperty,
# o nome da propriedade (get_edited_property) e o RÓTULO que o Inspector mostra (label). Imprime e sai.
const QUERO := ["texture", "zoom", "vida_max", "vida", "cor", "alvo", "velocidade", "position", "z_index", "hframes", "position_smoothing_speed", "modulate"]

func _enter_tree() -> void:
	_corre.call_deferred()

func _props(n: Node, acc: Array) -> void:
	if n is EditorProperty:
		acc.append(n)
	for c in n.get_children(true):
		_props(c, acc)

func _corre() -> void:
	await get_tree().process_frame
	var heroi := Node2D.new()
	heroi.set_script(load("res://heroi.gd"))
	for o in [Sprite2D.new(), Camera2D.new(), heroi]:
		EditorInterface.inspect_object(o)
		for i in 5:
			await get_tree().process_frame
		var acc := []
		_props(EditorInterface.get_inspector(), acc)
		var d := {}
		for ep in acc:
			var nome := String(ep.get_edited_property())
			if nome in QUERO:
				d[nome] = ep.label
		printerr("@@CASO b2. rótulo no Inspector do editor — " + o.get_class() + (" (heroi.gd)" if o.get_script() else ""))
		printerr("@@R " + JSON.stringify({"editor_properties": acc.size(), "nome→rótulo": d, "estilo": EditorInterface.get_editor_settings().get_setting("interface/inspector/default_property_name_style")}))
	get_tree().quit()
