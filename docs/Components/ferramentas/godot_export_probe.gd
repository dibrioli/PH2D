# ⭐ ORÁCULO do TOP-20 #16 (`ScriptProperties`) — Godot 4.7.2, **MIT** (arsenal §2: porta ABERTA,
# porta de consola medida `godot --headless --script <s.gd> --quit`).
#
# ⚠️ **Corre-se, não se lê** (CLAUDE.md §0.9). O que se colhe é a LEI DO VALOR POR OBJECTO quando o
# SCRIPT muda depois de o objecto ter sido gravado — a pergunta que nenhum doc responde de forma
# executável e que decide o nosso desenho:
#
#   Q1  objecto SEM valor próprio; o default do script muda       ⇒ segue o default novo?
#   Q2  objecto COM valor próprio; o default muda                 ⇒ guarda o dele?
#   Q3  objecto com valor próprio IGUAL ao default antigo         ⇒ o que é um «valor próprio»?
#   Q4  a propriedade SAI do script                               ⇒ o valor gravado sobrevive?
#   Q4b …e volta com o mesmo nome                                 ⇒ o valor volta com ela?
#   Q5  a propriedade muda de TIPO (int → String)                 ⇒ o que o objecto lê?
#   Q6  a faixa do script ESTREITA abaixo do valor gravado         ⇒ o load prende à faixa?
#   Q7  a ORDEM em que o painel mostra as propriedades            ⇒ a da declaração?
#   Q8  um float gravado numa propriedade que passou a int        ⇒ trunca? arredonda?
#   Q9  duas instâncias do MESMO script                           ⇒ valores independentes?
#
# Cada caso grava uma cena com o script v1, reescreve o ficheiro do script (v2) e recarrega a cena
# com `CACHE_MODE_IGNORE_DEEP` — o CONTROLO (C0) recarrega SEM mudar o script e tem de ler o que
# gravou, senão a sonda mede a cache e não o alvo.
#
# Uso: godot --headless --script docs/Components/ferramentas/godot_export_probe.gd --quit
extends SceneTree

var _dir := ""

func _log(s: String) -> void:
	print(s)

func _escreve(caminho: String, texto: String) -> void:
	var f := FileAccess.open(caminho, FileAccess.WRITE)
	f.store_string(texto)
	f.close()

## Grava uma cena com UM nó que carrega o script `v1` e os valores `proprios` (nome → valor).
## Devolve o caminho da cena.
func _grava(caso: String, v1: String, proprios: Dictionary, n_nos: int = 1) -> String:
	var sp := _dir + "/" + caso + ".gd"
	_escreve(sp, v1)
	var script: GDScript = ResourceLoader.load(sp, "", ResourceLoader.CACHE_MODE_IGNORE_DEEP)
	var raiz := Node2D.new()
	raiz.name = "Raiz"
	for i in n_nos:
		var no := Node2D.new()
		no.name = "N%d" % i
		no.set_script(script)
		raiz.add_child(no)
		no.owner = raiz
		var meus: Dictionary = proprios.get(i, {}) if proprios.has(i) else proprios
		for k in meus:
			no.set(k, meus[k])
	var ps := PackedScene.new()
	var err := ps.pack(raiz)
	if err != OK:
		_log("  !! pack falhou: %s" % err)
	var cp := _dir + "/" + caso + ".tscn"
	ResourceSaver.save(ps, cp)
	raiz.free()
	return cp

func _recarrega(caso: String, cp: String, v2: String) -> Node:
	if v2 != "":
		_escreve(_dir + "/" + caso + ".gd", v2)
	var ps: PackedScene = ResourceLoader.load(cp, "", ResourceLoader.CACHE_MODE_IGNORE_DEEP)
	return ps.instantiate()

func _tscn(cp: String) -> String:
	var f := FileAccess.open(cp, FileAccess.READ)
	var t := f.get_as_text()
	f.close()
	var linhas: Array[String] = []
	for l in t.split("\n"):
		if l.begins_with("speed") or l.begins_with("label") or l.begins_with("alpha") or l.begins_with("zeta") or l.begins_with("[node name=\"N"):
			linhas.append(l)
	return " | ".join(linhas)

func _fmt(v) -> String:
	return "%s (%s)" % [var_to_str(v), type_string(typeof(v))]

func _initialize() -> void:
	_dir = OS.get_cache_dir() + "/ph2d_godot_export_probe"
	DirAccess.make_dir_recursive_absolute(_dir)
	_log("# godot " + Engine.get_version_info().string)
	_log("# dir " + _dir)

	var V_INT_4 := "extends Node2D\n@export var speed: int = 4\n"
	var V_INT_7 := "extends Node2D\n@export var speed: int = 7\n"

	# ── C0 CONTROLO: grava 9, recarrega SEM mudar o script ⇒ tem de ler 9 ─────────────────────
	var cp := _grava("c0", V_INT_4, {"speed": 9})
	var n := _recarrega("c0", cp, "")
	_log("C0 controlo (grava 9, script igual)      -> %s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _tscn(cp)])
	n.free()

	# ── Q1 sem valor próprio, default 4 -> 7 ────────────────────────────────────────────────────
	cp = _grava("q1", V_INT_4, {})
	n = _recarrega("q1", cp, V_INT_7)
	_log("Q1 sem proprio, default 4->7             -> %s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _tscn(cp)])
	n.free()

	# ── Q2 com valor próprio 9, default 4 -> 7 ──────────────────────────────────────────────────
	cp = _grava("q2", V_INT_4, {"speed": 9})
	n = _recarrega("q2", cp, V_INT_7)
	_log("Q2 proprio 9, default 4->7               -> %s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _tscn(cp)])
	n.free()

	# ── Q3 valor próprio IGUAL ao default antigo (4), default 4 -> 7 ────────────────────────────
	cp = _grava("q3", V_INT_4, {"speed": 4})
	n = _recarrega("q3", cp, V_INT_7)
	_log("Q3 proprio 4 (= default), default 4->7   -> %s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _tscn(cp)])
	n.free()

	# ── Q4 a propriedade SAI; Q4b re-grava e ela VOLTA ──────────────────────────────────────────
	cp = _grava("q4", V_INT_4, {"speed": 9})
	n = _recarrega("q4", cp, "extends Node2D\n@export var outra: int = 1\n")
	var no4: Node = n.get_node("N0")
	_log("Q4 proprio 9, propriedade SAI            -> get=%s  has=%s   tscn: %s" % [_fmt(no4.get("speed")), "speed" in no4, _tscn(cp)])
	# re-grava a cena carregada (o que um editor faz ao salvar) e devolve a propriedade
	for c in n.get_children():
		c.owner = n
	var ps4 := PackedScene.new()
	ps4.pack(n)
	var cp4b := _dir + "/q4b.tscn"
	ResourceSaver.save(ps4, cp4b)
	n.free()
	_escreve(_dir + "/q4.gd", V_INT_4)
	var n4b: Node = (ResourceLoader.load(cp4b, "", ResourceLoader.CACHE_MODE_IGNORE_DEEP) as PackedScene).instantiate()
	_log("Q4b re-gravada sem ela, ela VOLTA        -> %s   tscn(q4b): %s" % [_fmt(n4b.get_node("N0").get("speed")), _tscn(cp4b)])
	n4b.free()

	# ── Q5 muda de TIPO int -> String ───────────────────────────────────────────────────────────
	cp = _grava("q5", V_INT_4, {"speed": 9})
	n = _recarrega("q5", cp, "extends Node2D\n@export var speed: String = \"lento\"\n")
	_log("Q5 proprio 9 (int), tipo passa a String  -> %s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _tscn(cp)])
	n.free()

	# ── Q5b String -> int, com texto que NÃO é número ───────────────────────────────────────────
	cp = _grava("q5b", "extends Node2D\n@export var speed: String = \"a\"\n", {"speed": "rapido"})
	n = _recarrega("q5b", cp, V_INT_4)
	_log("Q5b proprio \"rapido\", tipo passa a int   -> %s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _tscn(cp)])
	n.free()

	# ── Q6 a faixa ESTREITA abaixo do valor gravado ─────────────────────────────────────────────
	cp = _grava("q6", "extends Node2D\n@export_range(0, 20) var speed: int = 4\n", {"speed": 15})
	n = _recarrega("q6", cp, "extends Node2D\n@export_range(0, 10) var speed: int = 4\n")
	_log("Q6 proprio 15, faixa 0..20 -> 0..10      -> %s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _tscn(cp)])
	n.free()

	# ── Q7 a ORDEM das propriedades ─────────────────────────────────────────────────────────────
	var sp7 := _dir + "/q7.gd"
	_escreve(sp7, "extends Node2D\n@export var zeta: int = 1\n@export var alpha: int = 2\n@export var label: String = \"x\"\n")
	var s7: GDScript = ResourceLoader.load(sp7, "", ResourceLoader.CACHE_MODE_IGNORE_DEEP)
	var no7 := Node2D.new()
	no7.set_script(s7)
	var ordem: Array[String] = []
	for p in no7.get_property_list():
		if p["usage"] & PROPERTY_USAGE_SCRIPT_VARIABLE:
			ordem.append(p["name"])
	_log("Q7 ordem (declarado zeta, alpha, label)  -> %s" % [ordem])
	no7.free()

	# ── Q8 float gravado; a propriedade passa a int ─────────────────────────────────────────────
	cp = _grava("q8", "extends Node2D\n@export var speed: float = 1.0\n", {"speed": 2.75})
	n = _recarrega("q8", cp, V_INT_4)
	_log("Q8 proprio 2.75 (float), passa a int     -> %s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _tscn(cp)])
	n.free()

	# ── Q8b int gravado; a propriedade passa a float ────────────────────────────────────────────
	cp = _grava("q8b", V_INT_4, {"speed": 9})
	n = _recarrega("q8b", cp, "extends Node2D\n@export var speed: float = 1.5\n")
	_log("Q8b proprio 9 (int), passa a float       -> %s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _tscn(cp)])
	n.free()

	# ── Q9 duas instâncias, valores independentes ───────────────────────────────────────────────
	cp = _grava("q9", V_INT_4, {0: {"speed": 9}, 1: {}}, 2)
	n = _recarrega("q9", cp, V_INT_7)
	_log("Q9 N0 proprio 9, N1 sem; default 4->7    -> N0=%s N1=%s   tscn: %s" % [_fmt(n.get_node("N0").get("speed")), _fmt(n.get_node("N1").get("speed")), _tscn(cp)])
	n.free()

	# ── Q10 o SCRIPT não existe mais ────────────────────────────────────────────────────────────
	cp = _grava("q10", V_INT_4, {"speed": 9})
	DirAccess.remove_absolute(_dir + "/q10.gd")
	var ps10: PackedScene = ResourceLoader.load(cp, "", ResourceLoader.CACHE_MODE_IGNORE_DEEP)
	if ps10 == null:
		_log("Q10 o ficheiro do script SUMIU           -> a CENA nao carrega (null)")
	else:
		var n10 := ps10.instantiate()
		var no10: Node = n10.get_node_or_null("N0")
		_log("Q10 o ficheiro do script SUMIU           -> no=%s script=%s speed=%s" % [no10 != null, no10.get_script() if no10 else null, _fmt(no10.get("speed")) if no10 else "-"])
		n10.free()

	quit()
