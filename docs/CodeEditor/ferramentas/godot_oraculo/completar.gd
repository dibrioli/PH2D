extends SceneTree
# Oráculo da lista de completação (CodeEdit do Godot, corrido e não lido): para cada linha de
# completar.txt, os candidatos que o CodeEdit MOSTRA, pela ordem dele. Uso:
#   godot --headless --path <projeto> -s completar.gd -- completar.txt saída.txt
func _init():
	var args := OS.get_cmdline_user_args()
	var lines := FileAccess.get_file_as_string(args[0]).split("\n")
	var names := PackedStringArray()
	var out := PackedStringArray()
	var ce := CodeEdit.new()
	ce.code_completion_enabled = true
	get_root().add_child(ce)
	for l in lines:
		if l.begins_with("#") or l.strip_edges() == "":
			continue
		if l.begins_with("nomes:"):
			names = l.substr(6).strip_edges().split(" ", false)
			continue
		ce.text = "ph2d." + l
		ce.set_caret_line(0)
		ce.set_caret_column(ce.text.length())
		for n in names:
			ce.add_code_completion_option(CodeEdit.KIND_FUNCTION, n, n)
		ce.update_code_completion_options(true)
		var shown := PackedStringArray()
		for o in ce.get_code_completion_options():
			shown.append(o["display_text"])
		out.append(l + "\t" + " ".join(shown))
		ce.cancel_code_completion()
	var f := FileAccess.open(args[1], FileAccess.WRITE)
	f.store_string("\n".join(out) + "\n")
	f.close()
	quit()
