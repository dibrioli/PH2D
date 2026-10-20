extends Node
var amplitude = 1.0
func _ready():
	var p = Vector2(3, 4)
	print(p.length())
	var s = sin(amplitude)
	add_child(Node.new())
	var t = clamp(s, 0.0, 1.0)
	print_debug(PI, t)
