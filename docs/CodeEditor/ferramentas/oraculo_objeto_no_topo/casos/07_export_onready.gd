extends Node

@export @onready var porta: Node = $Porta

func _process(_d: float) -> void:
	porta.position.x += 1
