extends SceneTree

func _initialize():
	run()

func amostra(s):
	var seq = []
	for k in 48:
		await process_frame
		seq.append(s.frame)
	return seq

func run():
	var sf = SpriteFrames.new()
	sf.add_animation("a")
	var t = ImageTexture.create_from_image(Image.create(1, 1, false, Image.FORMAT_RGBA8))
	for i in 4:
		sf.add_frame("a", t)
	for m in [0, 1, 2]:
		sf.set_animation_loop_mode("a", m)
		print("loop_mode ", m, " -> get_animation_loop=", sf.get_animation_loop("a"))
		var s = AnimatedSprite2D.new()
		s.sprite_frames = sf
		root.add_child(s)
		s.animation_finished.connect(func(): print("   finished at frame ", s.frame))
		s.animation_looped.connect(func(): print("   looped at frame ", s.frame))
		s.play("a")
		print("  frente:    ", await amostra(s), " playing=", s.is_playing())
		s.stop()
		s.play_backwards("a")
		print("  backwards: ", await amostra(s), " playing=", s.is_playing())
		s.stop()
		s.speed_scale = -1.0
		s.play("a")
		print("  speed -1:  ", await amostra(s), " playing=", s.is_playing())
		s.queue_free()
	quit()
