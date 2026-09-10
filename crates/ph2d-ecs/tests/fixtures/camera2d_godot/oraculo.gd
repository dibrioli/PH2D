extends SceneTree

# ORACULO DA CAMERA 2D — Godot 4.7.2 (MIT), corrido SEM INTERFACE sobre trajetorias NOSSAS.
# ⚠️ A montagem acontece no 1º _physics_process: em _init a arvore ainda nao esta' "inside tree"
# e o make_current() falha em silencio -- lendo-se depois uma camera que nao e' a activa.
# ⚠️ A leitura vem ANTES da escrita do alvo: assim cada linha e' "o que a camera respondeu ao
# alvo do frame anterior", que e' uma ordem que um consumidor consegue reproduzir.

const DT := 1.0 / 60.0
const FRAMES := 240

var cam: Camera2D
var alvo: Node2D
var f := -1
var traj := ""
var pronto := false

func env(k: String, d: String) -> String:
    var v := OS.get_environment(k)
    return v if v != "" else d

func montar():
    Engine.physics_ticks_per_second = 60
    root.size = Vector2i(1152, 648)
    traj = env("TRAJ", "degrau")
    alvo = Node2D.new()
    root.add_child(alvo)
    cam = Camera2D.new()
    cam.enabled = true
    cam.process_callback = Camera2D.CAMERA2D_PROCESS_PHYSICS
    cam.position_smoothing_enabled = env("SMOOTH", "1") != "0"
    cam.position_smoothing_speed = float(env("SPEED", "5.0"))
    var d := env("DRAG", "")
    if d != "":
        cam.drag_horizontal_enabled = true
        cam.drag_vertical_enabled = true
        cam.drag_left_margin = float(d)
        cam.drag_right_margin = float(d)
        cam.drag_top_margin = float(d)
        cam.drag_bottom_margin = float(d)
    var L := env("LIMIT", "")
    if L != "":
        cam.limit_left = int(-float(L))
        cam.limit_right = int(float(L))
        cam.limit_top = int(-float(L))
        cam.limit_bottom = int(float(L))
    if env("LIM_L", "") != "":
        cam.limit_left = int(float(env("LIM_L", "0")))
        cam.limit_right = int(float(env("LIM_R", "0")))
        cam.limit_top = -100000
        cam.limit_bottom = 100000
        L = "L=%s R=%s" % [env("LIM_L", ""), env("LIM_R", "")]
    alvo.add_child(cam)
    cam.make_current()
    cam.reset_smoothing()
    print("# oraculo=Godot ", Engine.get_version_info()["string"], " (MIT) -- corrido sem interface")
    print("# traj=", traj, " smoothing=", cam.position_smoothing_enabled,
          " speed=", cam.position_smoothing_speed, " drag=", d, " limit=", L,
          " dt=", DT, " viewport=", root.get_visible_rect().size,
          " current=", cam.is_current())
    print("frame,alvo_x,alvo_y,cam_x,cam_y")

func posicao(i: int) -> Vector2:
    var t := float(i) * DT
    match traj:
        "degrau":   return Vector2(0.0 if i < 30 else 300.0, 0.0)
        "rampa":    return Vector2(t * 200.0, 0.0)
        "vaivem":   return Vector2(sin(t * 2.0) * 250.0, 0.0)
        "diagonal": return Vector2(t * 150.0, t * 90.0)
        "parada":   return Vector2(min(t, 1.0) * 400.0, 0.0)
        _:          return Vector2.ZERO

func _physics_process(_d: float) -> bool:
    if not pronto:
        montar()
        pronto = true
        f = 0
        return false
    if f >= FRAMES:
        quit()
        return true
    var c := cam.get_screen_center_position()
    print("%d,%.6f,%.6f,%.6f,%.6f" % [f, alvo.position.x, alvo.position.y, c.x, c.y])
    alvo.position = posicao(f + 1)
    f += 1
    return false
