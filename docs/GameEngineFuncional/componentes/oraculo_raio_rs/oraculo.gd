# Oráculo das melhorias do Ray Sensor (GEF 07 RS, 15/11) — Godot 4 (MIT), GodotPhysics2D, headless.
# A cena é a NOSSA, em metros (100 px = 1 m, y para cima no PH2D = -y aqui); a saída já vem em metros,
# y para cima, e o gate `oraculo_godot_das_melhorias` corre a mesma cena no nosso motor, caso a caso.
extends SceneTree
const PX := 100.0
var out := ""
func L(s): out += s + "\n"
func m(v: Vector2) -> String: return "(%.4f, %.4f)" % [v.x / PX, -v.y / PX]
func box(parent, kind, nome, pos_m, half_m):
	var b
	if kind == "static": b = StaticBody2D.new()
	else: b = CharacterBody2D.new()
	b.name = nome; b.position = Vector2(pos_m.x, -pos_m.y) * PX
	var c = CollisionShape2D.new(); var r = RectangleShape2D.new(); r.size = half_m * 2.0 * PX; c.shape = r
	b.add_child(c); parent.add_child(b); return b
func step():
	await physics_frame
	await physics_frame
func rep(tag, r):
	r.force_raycast_update()
	if r.is_colliding():
		var p = r.get_collision_point()
		L("%s | hit=%s | point=%s | normal=%s | dist=%.4f" % [tag, r.get_collider().name, m(p), "(%.4f, %.4f)" % [r.get_collision_normal().x, -r.get_collision_normal().y], (p - r.global_position).length() / PX])
	else:
		L("%s | hit=none" % tag)
func _init():
	_run.call_deferred()
func _run():
	var w = Node2D.new(); root.add_child(w)
	L("# Oraculo GEF 07 RS: Godot RayCast2D/ShapeCast2D/intersect_ray, headless, GodotPhysics2D. Metros, y para cima.")
	L("# Cena: Owner 0,2x0,2 em (0,0) com o raio filho a olhar para baixo 3 m; Floor 4x0,2 em y=-1,1; Floor2 em y=-1,6; Side 0,1x0,1 em (0,25 ; -0,5).")
	L("Godot " + Engine.get_version_info().string)
	var chao = box(w, "static", "Floor", Vector2(0, -1.1), Vector2(2.0, 0.1))
	box(w, "static", "Floor2", Vector2(0, -1.6), Vector2(2.0, 0.1))
	box(w, "static", "Side", Vector2(0.25, -0.5), Vector2(0.05, 0.05))
	var dono = box(w, "char", "Owner", Vector2(0, 0), Vector2(0.1, 0.1))
	var r = RayCast2D.new(); r.target_position = Vector2(0, 3.0 * PX); dono.add_child(r)
	await step()
	rep("line", r)
	r.enabled = false; await step(); L("disabled | is_colliding=%s" % r.is_colliding())
	r.enabled = true; await step()
	r.add_exception(chao); rep("also_ignore_floor", r)
	r.clear_exceptions()
	var hits = []
	for i in 10:
		r.force_raycast_update()
		if not r.is_colliding(): break
		hits.append(r.get_collider().name); r.add_exception(r.get_collider())
	L("all_hits | %s" % ", ".join(hits))
	r.clear_exceptions()
	var s = ShapeCast2D.new(); var cs = CircleShape2D.new(); cs.radius = 0.3 * PX; s.shape = cs; s.target_position = Vector2(0, 3.0 * PX)
	dono.add_child(s)
	await step(); s.force_shapecast_update()
	if s.is_colliding():
		L("thick_0.6 | hit=%s | point=%s | safe=%.4f | unsafe=%.4f" % [s.get_collider(0).name, m(s.get_collision_point(0)), s.get_closest_collision_safe_fraction() * 3.0, s.get_closest_collision_unsafe_fraction() * 3.0])
	else:
		L("thick_0.6 | hit=none")
	var q = PhysicsRayQueryParameters2D.create(Vector2(0, -0.5) * PX, Vector2(0, 3.0) * PX)
	var res = w.get_world_2d().direct_space_state.intersect_ray(q)
	L("raycast_code | hit=%s | point=%s | normal=%s" % [res.collider.name, m(res.position), "(%.4f, %.4f)" % [res.normal.x, -res.normal.y]])
	chao.free(); await step()
	rep("target_deleted", r)
	var f = FileAccess.open("res://godot_rs.txt", FileAccess.WRITE); f.store_string(out); f.close()
	quit()
