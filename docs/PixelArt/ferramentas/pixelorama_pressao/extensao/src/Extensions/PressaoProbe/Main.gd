extends Node
# Oracle probe: drives Pixelorama's Pencil with synthetic pen pressure and saves PNGs.
# Env: PROBE_OUT=<dir>. Writes <variant>.png + log lines "PROBE ..." to stdout.

const N_DOTS := 21
const SPACING := 20
const W := 440
const H := 80

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _motion(pos: Vector2i, p: float) -> void:
	var e := InputEventMouseMotion.new()
	e.position = Vector2(pos)
	e.pressure = p
	e.button_mask = MOUSE_BUTTON_MASK_LEFT if Tools.active_button != -1 else 0
	Tools.handle_draw(pos, e)

func _button(pos: Vector2i, down: bool) -> void:
	var e := InputEventMouseButton.new()
	e.position = Vector2(pos)
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = down
	Tools.handle_draw(pos, e)

func _variant(vname: String, pmin: float, pmax: float, size_dyn: bool, alpha_dyn: bool, base: int, inc: int) -> void:
	var out := OS.get_environment("PROBE_OUT")
	ExtensionsApi.project.new_project([], vname, Vector2(W, H))
	Global.current_project_index = Global.projects.size() - 1
	await _frames(5)
	Tools.assign_tool("Pencil", MOUSE_BUTTON_LEFT)
	await _frames(3)
	var tn = Tools.get_tool(MOUSE_BUTTON_LEFT).tool_node
	tn._on_BrushSize_value_changed(base)
	Tools.brush_size_max_increment = inc
	Tools.pen_pressure_min = pmin
	Tools.pen_pressure_max = pmax
	Tools.dynamics_size = Tools.Dynamics.PRESSURE if size_dyn else Tools.Dynamics.NONE
	Tools.dynamics_alpha = Tools.Dynamics.PRESSURE if alpha_dyn else Tools.Dynamics.NONE
	Global.can_draw = true
	# Row 1 (y=20): isolated dots, one press per pressure value.
	for i in N_DOTS:
		var p := float(i) / float(N_DOTS - 1)
		var pos := Vector2i(10 + i * SPACING, 20)
		_motion(pos + Vector2i(1, 0), p)
		_motion(pos, p)
		var eff := Tools.pen_pressure
		_button(pos, true)
		_button(pos, false)
		print("PROBE %s dot raw=%.3f eff=%.3f size_dyn=%d" % [vname, p, eff, tn._brush_size_dynamics])
		await _frames(1)
	# Row 2 (y=60): one continuous stroke, pressure 0 -> 1 left to right, 1 px per event.
	var x0 := 10
	var x1 := W - 10
	_motion(Vector2i(x0 - 1, 60), 0.0)
	_motion(Vector2i(x0, 60), 0.0)
	_button(Vector2i(x0, 60), true)
	for x in range(x0 + 1, x1 + 1):
		_motion(Vector2i(x, 60), float(x - x0) / float(x1 - x0))
	_button(Vector2i(x1, 60), false)
	await _frames(3)
	var img: Image = Global.current_project.frames[0].cels[0].get_image()
	print("PROBE img %s size=%s" % [vname, str(img.get_size())])
	if OS.get_environment("PROBE_HOLD") == vname:
		await _frames(600)
	var err := img.save_png(out.path_join(vname + ".png"))
	print("PROBE saved %s err=%d" % [vname, err])

func _ready() -> void:
	await _frames(60)
	print("PROBE start api=%d" % ExtensionsApi.get_api_version())
	await _variant("size_default_thr", 0.2, 0.8, true, false, 1, 15)
	await _variant("size_thr_0_1", 0.0, 1.0, true, false, 1, 15)
	await _variant("alpha_thr_0_1", 0.0, 1.0, false, true, 8, 4)
	print("PROBE done")
	await _frames(5)
	get_tree().quit()
