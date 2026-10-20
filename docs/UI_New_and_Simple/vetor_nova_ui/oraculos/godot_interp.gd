# Oráculo PH2D: como o AnimationPlayer do Godot interpola cada tipo de propriedade.
extends SceneTree

var done := false
var poly: Polygon2D
var poly2: Polygon2D
var lab: Label
var lab2: Label
var ap: AnimationPlayer

func add_val(anim: Animation, path: String, v0, v1, mode := Animation.UPDATE_CONTINUOUS) -> void:
	var t := anim.add_track(Animation.TYPE_VALUE)
	anim.track_set_path(t, NodePath(path))
	anim.value_track_set_update_mode(t, mode)
	anim.track_set_interpolation_type(t, Animation.INTERPOLATION_LINEAR)
	anim.track_insert_key(t, 0.0, v0)
	anim.track_insert_key(t, 1.0, v1)

func _initialize() -> void:
	poly = Polygon2D.new(); poly.name = "P"
	poly2 = Polygon2D.new(); poly2.name = "Q"
	lab = Label.new(); lab.name = "L"
	lab2 = Label.new(); lab2.name = "M"
	ap = AnimationPlayer.new(); ap.name = "AP"
	for n in [poly, poly2, lab, lab2, ap]:
		root.add_child(n)
	var a := Animation.new()
	a.length = 1.0
	add_val(a, "P:rotation", 0.0, 1.0)
	add_val(a, "P:color", Color(1,0,0,1), Color(0,0,1,0))
	add_val(a, "P:position", Vector2(0,0), Vector2(10,20))
	add_val(a, "P:polygon", PackedVector2Array([Vector2(0,0), Vector2(10,0), Vector2(10,10)]), PackedVector2Array([Vector2(0,10), Vector2(20,0), Vector2(30,30)]))
	add_val(a, "Q:polygon", PackedVector2Array([Vector2(0,0), Vector2(10,0), Vector2(10,10)]), PackedVector2Array([Vector2(0,10), Vector2(20,0), Vector2(30,30), Vector2(40,40)]))
	add_val(a, "P:visible", true, false)
	add_val(a, "L:text", "abc", "WXYZ12")
	add_val(a, "L:horizontal_alignment", 0, 2)  # int/enum, continuous
	add_val(a, "M:horizontal_alignment", 0, 2, Animation.UPDATE_DISCRETE)
	add_val(a, "M:text", "abc", "WXYZ12", Animation.UPDATE_DISCRETE)
	add_val(a, "P:z_index", 0, 7)  # int continuous
	add_val(a, "Q:z_index", 0, 7, Animation.UPDATE_DISCRETE)
	add_val(a, "Q:visible", true, false, Animation.UPDATE_DISCRETE)
	var lib := AnimationLibrary.new()
	lib.add_animation("a", a)
	ap.add_animation_library("", lib)
	# Leitura direta do recurso (sem player): value_track_interpolate
	print("== Animation.value_track_interpolate (recurso, sem player) ==")
	for t in [0.0, 0.25, 0.5, 0.75, 1.0]:
		var row := []
		for i in a.get_track_count():
			row.append("%s=%s" % [a.track_get_path(i), str(a.value_track_interpolate(i, t))])
		print("t=", t, " ", " | ".join(row))

func dump(t: float) -> void:
	print("t=%.2f rot=%s color=%s pos=%s P.polygon=%s Q.polygon(3->4)=%s P.visible(cont)=%s Q.visible(disc)=%s L.text(cont)=%s M.text(disc)=%s L.halign(cont)=%s M.halign(disc)=%s P.z(cont)=%s Q.z(disc)=%s" % [
		t, poly.rotation, poly.color, poly.position, poly.polygon, poly2.polygon, poly.visible, poly2.visible, lab.text, lab2.text, lab.horizontal_alignment, lab2.horizontal_alignment, poly.z_index, poly2.z_index])

func _process(_d: float) -> bool:
	if done:
		return true
	done = true
	print("== AnimationPlayer.seek(t, true) ==")
	ap.play("a")
	ap.pause()
	for t in [0.0, 0.25, 0.5, 0.75, 1.0]:
		ap.seek(t, true)
		dump(t)
	print("== AnimationPlayer: seek reverso 1.0 -> 0.5 ==")
	ap.seek(0.5, true)
	dump(0.5)
	return true
