extends SceneTree

# ORACULO DAS PONTAS DA ANIMACAO: Godot 4.7.2 (MIT), corrido SEM INTERFACE sobre animacoes NOSSAS.
# Como se corre e o que cada coluna quer dizer: README.md desta pasta.
# O relogio e' o de --fixed-fps (delta constante). A primeira linha de cada caso e' o estado logo
# apos o play(); a linha do tique k e' lida DEPOIS de o no' processar k vezes, garantido pela
# prioridade do Amostrador e conferivel pela coluna pos.

var TICKS := 80

class Amostrador extends Node:
    var cb: Callable
    func _process(delta: float) -> void:
        cb.call(delta)

class Caso:
    var nome: String
    var no: String
    var n: int
    var sprite: Node
    var player: AnimationPlayer
    var fim := -1

var casos: Array[Caso] = []
var tick := 0
var montado := false
var dur := 0.1
var quadros_aquecendo := 5
var delta_min := INF
var delta_max := 0.0

# O 1.o delta do processo NAO e' o de --fixed-fps (medido: 1/60 a 40 fps, com o sprite ja' a contar).
# Por isso a montagem espera o relogio assentar e acontece DENTRO do Amostrador, depois de os outros
# nos terem processado neste quadro: o caso nasce sem nenhum delta contado.
func _initialize() -> void:
    # Com o jitter fix ligado o delta a 80 fps variou (medido: min 0,01212 contra 0,0125).
    Engine.physics_jitter_fix = 0.0
    var t := OS.get_environment("TICKS")
    if t != "":
        TICKS = int(t)

func _process(_delta: float) -> bool:
    if not montado:
        montado = true
        var am := Amostrador.new()
        am.process_priority = 1000
        am.cb = passo
        root.add_child(am)
    return false

func censo():
    for classe in ["SpriteFrames", "AnimatedSprite2D"]:
        var nomes := []
        for m in ClassDB.class_get_method_list(classe, true):
            var s: String = m["name"].to_lower()
            if s.contains("loop") or s.contains("pong"):
                nomes.append(m["name"])
        for p in ClassDB.class_get_property_list(classe, true):
            var s: String = p["name"].to_lower()
            if s.contains("loop") or s.contains("pong"):
                nomes.append(p["name"])
        nomes.sort()
        print("# api ", classe, ": ", ",".join(nomes))
    print("# api Animation.LOOP_PINGPONG=", Animation.LOOP_PINGPONG, " Animation.LOOP_NONE=", Animation.LOOP_NONE)

func folha(n: int, modo: int) -> SpriteFrames:
    var sf := SpriteFrames.new()
    sf.add_animation("a")
    sf.set_animation_speed("a", 1.0 / dur)
    sf.set_animation_loop_mode("a", modo)
    for i in n:
        var t := PlaceholderTexture2D.new()
        t.size = Vector2(8, 8)
        sf.add_frame("a", t, 1.0)
    return sf

func caso_as2d(rotulo: String, modo: int, n: int):
    var c := Caso.new()
    c.nome = "as2d_%s_n%d" % [rotulo, n]
    c.no = "AnimatedSprite2D"
    c.n = n
    var s := AnimatedSprite2D.new()
    s.sprite_frames = folha(n, modo)
    root.add_child(s)
    s.animation_finished.connect(func(): c.fim = tick)
    c.sprite = s
    casos.append(c)
    s.play("a")

func caso_ap(modo: String, n: int, passos: int, update: int, sufixo: String):
    var c := Caso.new()
    c.nome = "ap_%s_n%d_len%d_%s" % [modo, n, passos, sufixo]
    c.no = "AnimationPlayer"
    c.n = n
    var s := Sprite2D.new()
    s.hframes = n
    var img := PlaceholderTexture2D.new()
    img.size = Vector2(8 * n, 8)
    s.texture = img
    root.add_child(s)
    var a := Animation.new()
    a.length = passos * dur
    a.loop_mode = Animation.LOOP_PINGPONG if modo == "pingpong" else Animation.LOOP_NONE
    var tr := a.add_track(Animation.TYPE_VALUE)
    a.track_set_path(tr, NodePath(".:frame"))
    a.value_track_set_update_mode(tr, update)
    a.track_set_interpolation_type(tr, Animation.INTERPOLATION_NEAREST)
    for i in n:
        a.track_insert_key(tr, i * dur, i)
    var lib := AnimationLibrary.new()
    lib.add_animation("a", a)
    var ap := AnimationPlayer.new()
    s.add_child(ap)
    ap.root_node = NodePath("..")
    ap.add_animation_library("", lib)
    ap.animation_finished.connect(func(_nome): c.fim = tick)
    c.sprite = s
    c.player = ap
    casos.append(c)
    ap.play("a")

func montar():
    var d := OS.get_environment("DUR_MS")
    dur = float(d) / 1000.0 if d != "" else 0.1
    censo()
    for n in [1, 2, 4]:
        caso_as2d("once", SpriteFrames.LOOP_NONE, n)
        caso_as2d("pingpong", SpriteFrames.LOOP_PINGPONG, n)
    for n in [1, 2, 4]:
        for u in [[Animation.UPDATE_DISCRETE, "disc"], [Animation.UPDATE_CONTINUOUS, "cont"]]:
            for passos in [n, n - 1]:
                if passos < 1:
                    continue
                caso_ap("pingpong", n, passos, u[0], u[1])
                caso_ap("once", n, passos, u[0], u[1])
    print("# oraculo=Godot ", Engine.get_version_info()["string"], " (MIT) -- corrido sem interface")
    print("# data=", Time.get_date_string_from_system())
    print("# dur_ms=", dur * 1000.0, " ticks=", TICKS)
    print("caso,no,tick,quadro,pos,tocando,fim_no_tick")

func linha(c: Caso):
    var q: int
    var pos: float
    var tocando: bool
    if c.player != null:
        q = c.sprite.frame
        pos = c.player.current_animation_position if c.player.is_playing() else -1.0
        tocando = c.player.is_playing()
    else:
        q = c.sprite.frame
        pos = c.sprite.frame_progress
        tocando = c.sprite.is_playing()
    print("%s,%s,%d,%d,%.6f,%d,%d" % [c.nome, c.no, tick, q, pos, int(tocando), c.fim])

func passo(delta: float):
    if quadros_aquecendo > 0:
        quadros_aquecendo -= 1
        return
    if casos.is_empty():
        montar()
    else:
        delta_min = min(delta_min, delta)
        delta_max = max(delta_max, delta)
    for c in casos:
        linha(c)
    tick += 1
    if tick > TICKS:
        print("# delta_min=%.17f delta_max=%.17f" % [delta_min, delta_max])
        print("# comando=env DUR_MS=%s godot --headless --fixed-fps %d --script oraculo.gd" % [OS.get_environment("DUR_MS"), roundi(1.0 / delta_max)])
        quit()
