# ORÁCULO: NASCER e MORRER no Godot 4.7.2 (MIT), corrido sem interface — docs/Components/09_plano_spawner.md §1.
# Correr: godot --headless --script docs/Components/ferramentas/godot_spawn_lifetime_probe.gd
#
# Mede, uma pergunta por bloco:
#   A) o INSTANTE da morte — queue_free() adia até onde? o moribundo ainda TICA? ainda aparece na consulta?
#   B) free() imediato · queue_free() duas vezes
#   C) a DERIVA do relógio — um período que não é múltiplo do tique acumula, ou é re-zerado?
#   D) o NASCER a meio de um tique — o recém-nascido tica no MESMO tique?
#   E) fora-do-ecrã — o VisibleOnScreenNotifier2D dispara sem janela? e contra QUE rectângulo?
#   F) custo de instanciar e de matar N (1k/10k/100k)
#
# ⚠️ Corre a partir do 1.º _process (a lição do probe irmão: antes disso os nós não estão na árvore).
# ⚠️ Os blocos que precisam de ver o quadro SEGUINTE vivem em frames diferentes, de propósito.
extends SceneTree

const TICKER := "extends Node2D
var ticks := 0
var procs := 0
func _physics_process(_d):
	ticks += 1
func _process(_d):
	procs += 1
"

var frame := 0
var root2d: Node2D
var vitima: Node
var relogio: Timer
var disparos := []          # [tique_de_fisica] de cada timeout
var fisica_ticks := 0
var recem: Node
var recem_nasceu_no_tique := -1
var notif
var notif_log := []
var fase := 0
var fase_frame := 0
var _script_ticker: GDScript

func _initialize():
	_script_ticker = GDScript.new()
	_script_ticker.source_code = TICKER
	_script_ticker.reload()

func _physics_process(_delta):
	# ⚠️ O 1.º _physics_process corre ANTES do 1.º _process (medido: a 1.ª redacção
	# chamou add_child sobre um nulo). O tique só conta depois de a árvore existir.
	if root2d == null:
		return false
	fisica_ticks += 1
	# D) nascer a meio de um tique: o filho entra AGORA; ele tica neste mesmo tique?
	if fisica_ticks == 3 and recem == null:
		recem = Node2D.new()
		recem.name = "Recem"
		recem.set_script(_script_ticker)
		root2d.add_child(recem)
		recem_nasceu_no_tique = fisica_ticks
	if fisica_ticks == 4 and recem != null:
		print("D_NASCER nasceu_no_tique=", recem_nasceu_no_tique,
			" ticks_do_recem_no_tique_seguinte=", recem.ticks)
	return false

func _process(_delta):
	frame += 1
	match frame:
		1: _f1()
		2: _f2()
		3: _f3()
	# C) o relógio precisa de tempo de parede; E precisa de ver QUADROS passarem.
	if frame > 3 and fase == 0 and (disparos.size() >= 12 or frame > 2000):
		_relogio_e_longe()
		fase = 1
		fase_frame = frame
	elif fase == 1 and frame > fase_frame + 4:
		print("E_LONGE on_screen=", notif.is_on_screen(), " log=", notif_log)
		notif.position = Vector2(0, 0)
		fase = 2
		fase_frame = frame
	elif fase == 2 and frame > fase_frame + 4:
		print("E_DE_VOLTA on_screen=", notif.is_on_screen(), " log=", notif_log)
		_custo()
		print("FIM")
		return true
	return false

func _f1():
	print("GODOT ", Engine.get_version_info().string)
	print("PHYSICS_HZ ", Engine.physics_ticks_per_second, " MAX_FPS ", Engine.max_fps)
	root2d = Node2D.new()
	root2d.name = "R"
	get_root().add_child(root2d)

	# A) o instante da morte, com um moribundo que CONTA os próprios tiques.
	vitima = Node2D.new()
	vitima.name = "Vitima"
	vitima.set_script(_script_ticker)
	root2d.add_child(vitima)
	vitima.add_to_group("mob")
	vitima.queue_free()
	print("A_LOGO_APOS_QUEUE_FREE valido=", is_instance_valid(vitima),
		" na_fila=", vitima.is_queued_for_deletion(),
		" filhos_do_pai=", root2d.get_child_count(),
		" no_grupo=", get_nodes_in_group("mob").size(),
		" ainda_na_arvore=", vitima.is_inside_tree())
	# B) queue_free() duas vezes
	vitima.queue_free()
	print("B_QUEUE_FREE_DUAS_VEZES sem_erro_ate_aqui=true na_fila=", vitima.is_queued_for_deletion())

	# C) o relógio: período que NÃO é múltiplo do tique (0,105 s a 60 Hz = 6,3 tiques).
	relogio = Timer.new()
	relogio.wait_time = 0.105
	relogio.one_shot = false
	relogio.autostart = true
	relogio.process_callback = Timer.TIMER_PROCESS_PHYSICS
	relogio.timeout.connect(_no_disparo)
	root2d.add_child(relogio)

func _f2():
	print("A_QUADRO_SEGUINTE valido=", is_instance_valid(vitima),
		" filhos_do_pai=", root2d.get_child_count(),
		" no_grupo=", get_nodes_in_group("mob").size())

	# B-bis) free() imediato
	var outro = Node2D.new()
	root2d.add_child(outro)
	var antes = root2d.get_child_count()
	outro.free()
	print("B_FREE_IMEDIATO filhos_antes=", antes, " filhos_depois=", root2d.get_child_count(),
		" valido=", is_instance_valid(outro))

func _f3():
	# E) fora do ecrã, sem janela nenhuma.
	var cam = Camera2D.new()
	root2d.add_child(cam)
	cam.make_current()
	notif = VisibleOnScreenNotifier2D.new()
	notif.rect = Rect2(-10, -10, 20, 20)
	notif.screen_entered.connect(func(): notif_log.append("entrou@" + str(frame)))
	notif.screen_exited.connect(func(): notif_log.append("saiu@" + str(frame)))
	root2d.add_child(notif)
	notif.position = Vector2(0, 0)
	print("E_VIEWPORT rect_visivel=", get_root().get_visible_rect(),
		" notifier_rect=", notif.rect)

func _no_disparo():
	disparos.append(fisica_ticks)

func _relogio_e_longe():
	# C) os períodos medidos em TIQUES de física: 0,105 s a 60 Hz = 6,3 tiques.
	var periodos = []
	for i in range(1, disparos.size()):
		periodos.append(disparos[i] - disparos[i - 1])
	var total = disparos[disparos.size() - 1] - disparos[0]
	print("C_RELOGIO disparos_em_tiques=", disparos, " periodos=", periodos,
		" media_medida=", total / float(disparos.size() - 1),
		" alvo_em_tiques=", 0.105 * Engine.physics_ticks_per_second)
	# E) leva o notificador para longe do rectângulo visível.
	notif.position = Vector2(100000, 100000)

func _custo():
	# F) custo de instanciar de uma receita e de pedir a morte.
	var molde = Node2D.new()
	molde.name = "Molde"
	var filho = Sprite2D.new()
	filho.name = "Corpo"
	molde.add_child(filho)
	filho.owner = molde
	var receita = PackedScene.new()
	receita.pack(molde)
	for N in [1000, 10000, 100000]:
		var holder = Node2D.new()
		get_root().add_child(holder)
		var t0 = Time.get_ticks_usec()
		var vivos = []
		for i in N:
			var inst = receita.instantiate()
			holder.add_child(inst)
			vivos.append(inst)
		var t1 = Time.get_ticks_usec()
		for inst in vivos:
			inst.queue_free()
		var t2 = Time.get_ticks_usec()
		print("F_CUSTO N=", N, " instanciar_ms=", (t1 - t0) / 1000.0,
			" pedir_morte_ms=", (t2 - t1) / 1000.0, " por_objecto_us=", (t1 - t0) / float(N))
		holder.queue_free()
