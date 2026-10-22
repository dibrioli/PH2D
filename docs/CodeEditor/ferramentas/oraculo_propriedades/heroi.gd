extends Node2D

@export var vida_max: int = 100
@export var vida: int = 100
@export var cor: Color
@export var alvo: Node2D
@export_range(0, 10) var velocidade: float = 2.0
var so_leitura: int:
	get:
		return 5
var nao_exportada: int = 7
