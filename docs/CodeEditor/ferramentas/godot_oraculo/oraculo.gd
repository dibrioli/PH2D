## Oráculo do fechamento automático e do auto-indent: corre um `CodeEdit` do Godot sobre os casos
## de `casos.txt` (entrada NOSSA) e escreve, por caso, o texto e os cursores resultantes.
## Uso: godot --headless -s oraculo.gd -- <entrada> <saída>   (ver corre.sh e o README)
##
## Formato (entrada e saída), um campo por linha, texto com escapes `\n` `\t` `\\`:
##   == nome
##   indent: tab | <n>      (tab = o padrão do Godot; n = espaços de largura n)
##   keys: ( a ENTER BS TAB SPACE UNDO   (separadas por um espaço; um carácter = essa tecla)
##   text: f(|)             (`|` = cursor; vários `|` = multicursor; `^` = âncora da seleção do `|` seguinte)
##   map: :=> then          (só passa adiante: o mapa GDScript→Luau é do teste Rust)
## A saída acrescenta `godot: <texto com | e ^>`.
extends SceneTree

var ce: CodeEdit
var frame := 0
var cases: Array = []
var out_path := ""


func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	if args.size() != 2:
		push_error("uso: -- <entrada> <saída>")
		quit(2)
		return
	out_path = args[1]
	cases = parse_cases(FileAccess.get_file_as_string(args[0]))
	ce = CodeEdit.new()
	ce.auto_brace_completion_enabled = true
	ce.indent_automatic = true
	# Os delimitadores do Luau (o `CodeEdit` é genérico; o editor de GDScript põe `#`).
	ce.add_string_delimiter("\"", "\"", false)
	ce.add_string_delimiter("'", "'", false)
	ce.add_comment_delimiter("--", "", true)
	ce.size = Vector2(800, 600)
	root.add_child(ce)


func _process(_d: float) -> bool:
	frame += 1
	if frame == 1:
		ce.grab_focus()
	elif frame == 2:
		var out := PackedStringArray()
		for c in cases:
			run_case(c)
			out.append("== " + c.name)
			for raw in c.raw:
				out.append(raw)
			out.append("godot: " + escape(marked()))
		var f := FileAccess.open(out_path, FileAccess.WRITE)
		f.store_string("\n".join(out) + "\n")
		f.close()
		quit(0)
	return false


func parse_cases(src: String) -> Array:
	var res: Array = []
	var cur = null
	for line in src.split("\n"):
		if line.begins_with("== "):
			cur = {"name": line.substr(3), "keys": [], "text": "", "indent": "tab", "raw": []}
			res.append(cur)
		elif cur != null and line.begins_with("keys: "):
			cur.keys = line.substr(6).split(" ")
			cur.raw.append(line)
		elif cur != null and line.begins_with("text: "):
			cur.text = unescape(line.substr(6))
			cur.raw.append(line)
		elif cur != null and line.begins_with("indent: "):
			cur.indent = line.substr(8)
			cur.raw.append(line)
		elif cur != null and line.begins_with("map: "):
			cur.raw.append(line)
	return res


func unescape(s: String) -> String:
	var o := ""
	var i := 0
	while i < s.length():
		var ch := s[i]
		if ch == "\\" and i + 1 < s.length():
			var n := s[i + 1]
			o += "\n" if n == "n" else ("\t" if n == "t" else n)
			i += 2
		else:
			o += ch
			i += 1
	return o


func escape(s: String) -> String:
	return s.replace("\\", "\\\\").replace("\n", "\\n").replace("\t", "\\t")


## Tira os marcadores e devolve [texto, [[linha, col, âncora_linha, âncora_col]...]].
func strip_markers(s: String) -> Array:
	var text := ""
	var carets: Array = []
	var line := 0
	var col := 0
	var anchor = null
	for ch in s:
		if ch == "|":
			var a = anchor if anchor != null else [line, col]
			carets.append([line, col, a[0], a[1]])
			anchor = null
		elif ch == "^":
			anchor = [line, col]
		else:
			text += ch
			if ch == "\n":
				line += 1
				col = 0
			else:
				col += 1
	return [text, carets]


func run_case(c: Dictionary) -> void:
	if c.indent == "tab":
		ce.indent_use_spaces = false
		ce.indent_size = 4
	else:
		ce.indent_use_spaces = true
		ce.indent_size = int(c.indent)
	var sm := strip_markers(c.text)
	ce.remove_secondary_carets()
	ce.deselect()
	ce.text = sm[0]
	ce.clear_undo_history()
	var carets: Array = sm[1]
	for i in carets.size():
		var k: Array = carets[i]
		var idx := 0
		if i > 0:
			idx = ce.add_caret(k[0], k[1])
		if k[0] != k[2] or k[1] != k[3]:
			ce.select(k[2], k[3], k[0], k[1], idx)
		else:
			ce.set_caret_line(k[0], false, true, 0, idx)
			ce.set_caret_column(k[1], false, idx)
	for key in c.keys:
		if key == "":
			continue
		if key == "UNDO":
			ce.undo()
			continue
		var ev := InputEventKey.new()
		ev.pressed = true
		match key:
			"ENTER":
				ev.keycode = KEY_ENTER
			"BS":
				ev.keycode = KEY_BACKSPACE
			"TAB":
				ev.keycode = KEY_TAB
			"SPACE":
				ev.keycode = KEY_SPACE
				ev.unicode = 32
			_:
				var ks: String = key
				var u: int = ks.unicode_at(0)
				ev.unicode = u
				ev.keycode = ks.to_upper().unicode_at(0) if u < 128 else KEY_UNKNOWN
		ce.get_viewport().push_input(ev)
		var up := ev.duplicate()
		up.pressed = false
		ce.get_viewport().push_input(up)


## O texto do `CodeEdit` com `|` em cada cursor e `^` na âncora de cada seleção.
func marked() -> String:
	var marks := {}
	for i in ce.get_caret_count():
		var l := ce.get_caret_line(i)
		var k := ce.get_caret_column(i)
		marks[[l, k]] = marks.get([l, k], "") + "|"
		if ce.has_selection(i):
			var ol := ce.get_selection_origin_line(i)
			var ok := ce.get_selection_origin_column(i)
			marks[[ol, ok]] = "^" + marks.get([ol, ok], "")
	var lines := ce.text.split("\n")
	var o := PackedStringArray()
	for l in lines.size():
		var s: String = lines[l]
		var r := ""
		for k in s.length() + 1:
			r += marks.get([l, k], "")
			if k < s.length():
				r += s[k]
		o.append(r)
	return "\n".join(o)
