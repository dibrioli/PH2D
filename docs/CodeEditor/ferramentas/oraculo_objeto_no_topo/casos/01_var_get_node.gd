extends Node

var porta = get_node("Porta")

func _process(_d: float) -> void:
	porta.position.x += 1
