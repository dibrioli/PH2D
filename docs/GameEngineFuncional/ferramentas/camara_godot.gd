# Oráculo do plano 11 (CLAUDE.md §0.9): o que a Camera2D da Godot mostra encostada a um limite, com e sem
# `offset`, e quanto anda uma camada Parallax2D. Corre-se, nunca se lê o fonte:
#
#     godot --headless --resolution 1600x900 -s docs/GameEngineFuncional/ferramentas/camara_godot.gd
#
# Imprime uma linha JSON por caso: o rectângulo VISÍVEL no mundo (px da Godot, y para baixo) lido da
# transformação do canvas, e a posição de ecrã de uma camada com scroll_scale 0,5.
extends SceneTree

var casos = [
	# nome, posição da câmara, offset, limites (esq, cima, dir, baixo)
	["livre", Vector2(0, 0), Vector2(0, 0), [-10000, -10000, 10000, 10000]],
	["encostada_em_cima", Vector2(0, -5000), Vector2(0, 0), [-2000, -1000, 2000, 1000]],
	["encostada_em_cima_offset", Vector2(0, -5000), Vector2(0, -100), [-2000, -1000, 2000, 1000]],
	["encostada_em_baixo_offset", Vector2(0, 5000), Vector2(0, -100), [-2000, -1000, 2000, 1000]],
	["encostada_a_direita_offset", Vector2(5000, 0), Vector2(100, 0), [-2000, -1000, 2000, 1000]],
	["nivel_menor_que_a_vista", Vector2(0, 0), Vector2(0, 0), [-300, -200, 300, 200]],
]
var i = 0
var cam: Camera2D
var par: Parallax2D
var marca: Node2D
var espera = 0

func _initialize():
	cam = Camera2D.new()
	root.add_child(cam)
	par = Parallax2D.new()
	par.scroll_scale = Vector2(0.5, 0.5)
	marca = Node2D.new()
	par.add_child(marca)
	root.add_child(par)
	aplica()

func aplica():
	var c = casos[i]
	cam.position = c[1]
	cam.offset = c[2]
	cam.limit_left = c[3][0]
	cam.limit_top = c[3][1]
	cam.limit_right = c[3][2]
	cam.limit_bottom = c[3][3]
	cam.make_current()
	cam.reset_smoothing()
	espera = 3

func _process(_dt):
	espera -= 1
	if espera > 0:
		return false
	var xf = root.get_canvas_transform()
	var tam = Vector2(root.get_visible_rect().size)
	var inv = xf.affine_inverse()
	var a = inv * Vector2(0, 0)
	var b = inv * tam
	var ecra_marca = marca.get_global_transform_with_canvas().origin
	print(JSON.stringify({
		"caso": casos[i][0],
		"vista": [a.x, a.y, b.x, b.y],
		"centro": [cam.get_screen_center_position().x, cam.get_screen_center_position().y],
		"camada_no_ecra": [ecra_marca.x, ecra_marca.y],
	}))
	i += 1
	if i >= casos.size():
		return true
	aplica()
	return false
