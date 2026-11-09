# P14d: a CONVENÇÃO de troca do AnimatedSprite2D sem erro de vírgula flutuante: --fixed-fps 1024,
# speed 16, duration 1 ⇒ cada passo soma exatamente 1/64. Imprime o passo em que o quadro troca.
# Uso: godot --headless --fixed-fps 1024 --script <este>
extends SceneTree

class G extends Node:
	var s
	var t = 0
	var trocas = []
	var ultimo = 0
	func _process(_d):
		t += 1
		if s.frame != ultimo:
			trocas.append(t)
			ultimo = s.frame
		if t >= 200:
			print("EXATO ", JSON.stringify({"trocas": trocas, "passo": "1/64"}))
			get_tree().quit(0)

func _init():
	var img = Image.create(2, 1, false, Image.FORMAT_RGBA8)
	var tex = ImageTexture.create_from_image(img)
	var sf = SpriteFrames.new()
	sf.set_animation_speed("default", 16.0)
	sf.add_frame("default", tex, 1.0)
	sf.add_frame("default", tex, 1.0)
	var s = AnimatedSprite2D.new()
	s.sprite_frames = sf
	root.add_child(s)
	s.play("default")
	var g = G.new()
	g.s = s
	g.process_priority = 1000
	root.add_child(g)
