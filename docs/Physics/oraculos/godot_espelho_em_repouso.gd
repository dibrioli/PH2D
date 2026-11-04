# Oraculo: Godot 4.7.2 (MIT). Y do Godot cresce para BAIXO; 100 px = 1 m do PH2D.
# A cena do espelho_a_simular: chao fino (meia-altura 0,1 m), bola r = 0,5 m com o colisor
# 1 m acima da origem, rotacao travada; pousa e vira em Y com o relogio a correr.
extends SceneTree
var fase := 0
var n := 0
var casos := ["corpo_scale_y", "transform_global", "forma_scale_y", "forma_pos_y", "pai_scale_y"]
var i := 0
var corpo: RigidBody2D
var forma: CollisionShape2D
var pai: Node2D
var chao: StaticBody2D
var y0 := 0.0

func _initialize():
	print("gravity ", ProjectSettings.get_setting("physics/2d/default_gravity"), " engine ", ProjectSettings.get_setting("physics/2d/physics_engine"), " server ", PhysicsServer2D.get_class())
	monta()

func monta():
	pai = Node2D.new()
	root.add_child(pai)
	chao = StaticBody2D.new()
	var fc := CollisionShape2D.new()
	var r := RectangleShape2D.new()
	r.size = Vector2(10000, 20)
	fc.shape = r
	chao.add_child(fc)
	root.add_child(chao)
	corpo = RigidBody2D.new()
	corpo.lock_rotation = true
	corpo.can_sleep = false
	forma = CollisionShape2D.new()
	var b := CircleShape2D.new()
	b.radius = 50.0
	forma.shape = b
	forma.position = Vector2(0, -100)
	corpo.add_child(forma)
	corpo.position = Vector2(0, -300)
	pai.add_child(corpo)
	fase = 0
	n = 0

func _physics_process(_d):
	n += 1
	if fase == 0 and n >= 240:
		y0 = corpo.global_position.y
		print("v=", corpo.linear_velocity, " in_tree=", corpo.is_inside_tree(), " freeze=", corpo.freeze, " active=", PhysicsServer2D.get_process_info(PhysicsServer2D.INFO_ACTIVE_OBJECTS))
		print("%s pousado origem_y=%.3f forma_y=%.3f" % [casos[i], y0, forma.global_position.y])
		match casos[i]:
			"corpo_scale_y":
				corpo.scale = Vector2(1, -1)
			"transform_global":
				corpo.global_transform = corpo.global_transform.scaled_local(Vector2(1, -1))
			"forma_scale_y":
				forma.scale = Vector2(1, -1)
			"forma_pos_y":
				forma.position = Vector2(0, 100)
			"pai_scale_y":
				pai.position = Vector2(0, y0)
				corpo.position = Vector2(0, 0)
				pai.scale = Vector2(1, -1)
		fase = 1
		n = 0
	elif fase == 1 and (n == 1 or n == 2 or n == 10 or n == 60 or n == 120 or n == 300):
		print("%s  t+%3d origem_y=%.3f forma_y=%.3f scale=%s forma_gscale=%s" % [casos[i], n, corpo.global_position.y, forma.global_position.y, corpo.scale, forma.global_scale])
		if n == 300:
			pai.queue_free()
			chao.queue_free()
			i += 1
			if i >= casos.size():
				quit()
			else:
				monta()
	return false
