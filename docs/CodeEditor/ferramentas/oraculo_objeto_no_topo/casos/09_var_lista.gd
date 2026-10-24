extends Node

var portas = [$Porta, $Janela]

func _process(_d: float) -> void:
	portas[0].position.x += 1
