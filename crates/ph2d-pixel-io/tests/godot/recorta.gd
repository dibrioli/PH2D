# Lê a folha e o JSON como um importador faria, recorta cada quadro e compara com o PNG do quadro.
# Uso: godot --headless --script recorta.gd -- <pasta>. A pasta tem folha.png, folha.json e q<n>.png.
# Sai 0 e imprime RECORTOU <n> se todos batem; sai 1 no primeiro que não bate.
extends SceneTree

func _init():
	var pasta = OS.get_cmdline_user_args()[0]
	var folha = Image.load_from_file(pasta + "/folha.png")
	folha.convert(Image.FORMAT_RGBA8)
	var j = JSON.parse_string(FileAccess.get_file_as_string(pasta + "/folha.json"))
	var n = 0
	for q in j["frames"]:
		var fr = q["frame"]
		var so = q["spriteSourceSize"]
		var alvo = Image.load_from_file(pasta + "/q%d.png" % (n + 1))
		alvo.convert(Image.FORMAT_RGBA8)
		# A margem interna entra no `frame` e não no `spriteSourceSize` (medido no LibreSprite).
		var pad = (int(fr["w"]) - int(so["w"])) / 2
		var a = folha.get_region(Rect2i(fr["x"] + pad, fr["y"] + pad, so["w"], so["h"]))
		var b = alvo.get_region(Rect2i(so["x"], so["y"], so["w"], so["h"]))
		if a.get_data() != b.get_data():
			print("DIFERE no quadro ", n)
			quit(1)
			return
		n += 1
	print("RECORTOU ", n)
	quit(0)
