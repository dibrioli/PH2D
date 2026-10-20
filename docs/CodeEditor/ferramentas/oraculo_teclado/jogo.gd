extends Node2D
# Oráculo «de quem é o teclado com o jogo a correr» (CodeEditor item 7): o JOGO que o editor do Godot 4.7.2 corre.
# Regista cada tecla que lhe chega (`_input`, antes de qualquer UI do jogo) e um batimento a cada quadro, para que o
# roteiro saiba se o jogo andou, parou (pausa do editor) ou morreu (parar). Caminho do registo: ORACULO_LOG.

var f: FileAccess
var quadro := 0
var ultimo_batimento := 0

func _ready() -> void:
	f = FileAccess.open(OS.get_environment("ORACULO_LOG"), FileAccess.WRITE)
	escreve("pronto pid=%d" % OS.get_process_id())

func escreve(s: String) -> void:
	if f:
		f.store_line("%d %s" % [int(Time.get_unix_time_from_system() * 1000.0), s])
		f.flush()

func _process(_d: float) -> void:
	quadro += 1
	var agora := Time.get_ticks_msec()
	if agora - ultimo_batimento >= 100:
		ultimo_batimento = agora
		escreve("batimento quadro=%d" % quadro)
	queue_redraw()

func _draw() -> void:
	draw_rect(Rect2(Vector2(40 + (quadro % 200), 40), Vector2(60, 60)), Color(0.2, 0.5, 1.0))

func _input(e: InputEvent) -> void:
	if e is InputEventKey:
		escreve("tecla %s premida=%s eco=%s" % [e.as_text_physical_keycode(), e.pressed, e.echo])
	elif e is InputEventMouseButton:
		escreve("rato botao=%d premido=%s" % [e.button_index, e.pressed])

func _notification(n: int) -> void:
	if n == NOTIFICATION_WM_CLOSE_REQUEST or n == NOTIFICATION_PREDELETE:
		escreve("fim notificacao=%d" % n)
	elif n == NOTIFICATION_APPLICATION_FOCUS_OUT:
		escreve("foco perdido")
	elif n == NOTIFICATION_APPLICATION_FOCUS_IN:
		escreve("foco ganho")
