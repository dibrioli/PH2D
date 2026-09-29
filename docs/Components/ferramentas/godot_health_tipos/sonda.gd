extends SceneTree
## ⭐ SONDA dos TIPOS de dano — corre o addon «Health, HitBoxes, HurtBoxes» (cluttered-code, MIT,
## commit c9d2185) SEM interface, sobre casos NOSSOS, e imprime uma linha por passo.
##
## uso (a partir da raiz do clone do addon, depois de um `godot --headless --path . --import`):
##   godot --headless --path . --script <este ficheiro>
##
## Cada linha: `caso · passo · pedido · vida antes → depois · aplicado (sinal)`.

const K := HealthActionType.Enum.KINETIC
const M := HealthActionType.Enum.MEDICINE
const D := Health.Affect.DAMAGE
const H := Health.Affect.HEAL
const N := Health.Affect.NONE

var aplicado := "-"

func _mod(inc: int, mult: float, conv: int = Health.Affect.NONE) -> HealthModifier:
	return HealthModifier.new(inc, mult, conv, HealthActionType.Enum.NONE)

func _caso(nome: String, start: int, mods: Dictionary, pedidos: Array) -> void:
	var h := Health.new()
	h.max = 100
	h.current = start
	var tipado: Dictionary[HealthActionType.Enum, HealthModifier] = {}
	for k in mods:
		tipado[k] = mods[k]
	h.modifiers = tipado
	root.add_child(h)
	h.damaged.connect(func(_e, _t, _a, _i, _m, ap): aplicado = "dano %d" % ap)
	h.healed.connect(func(_e, _t, _a, _i, _m, ap): aplicado = "cura %d" % ap)
	var passo := 0
	for p in pedidos:
		passo += 1
		aplicado = "-"
		var antes := h.current
		var ac := HealthAction.new(p[0], p[1], p[2])
		if p.size() > 3:
			h.apply_modified_action(HealthModifiedAction.new(ac, p[3]))
		else:
			h.apply_action(ac)
		print("%s · %d · %s %s %d%s · %d -> %d · %s" % [nome, passo, Health.Affect.find_key(p[0]),
			HealthActionType.Enum.find_key(p[1]), p[2], (" +hurtbox %s" % p[3]) if p.size() > 3 else "",
			antes, h.current, aplicado])
	h.queue_free()

func _init() -> void:
	_caso("neutro", 100, {}, [[D, K, 10]])
	_caso("mult_1", 100, {K: _mod(0, 1.0)}, [[D, K, 10]])
	_caso("imune", 100, {K: _mod(0, 0.0)}, [[D, K, 10]])
	_caso("meio", 100, {K: _mod(0, 0.5)}, [[D, K, 10]])
	_caso("dobro", 100, {K: _mod(0, 2.0)}, [[D, K, 10]])
	_caso("terco", 100, {K: _mod(0, 0.33)}, [[D, K, 10], [D, K, 10], [D, K, 10]])
	_caso("quarto", 100, {K: _mod(0, 0.25)}, [[D, K, 10], [D, K, 2]])
	_caso("tres_quartos", 100, {K: _mod(0, 0.75)}, [[D, K, 10], [D, K, 2]])
	_caso("negativo", 50, {K: _mod(0, -1.0)}, [[D, K, 10]])
	_caso("absorve", 50, {K: _mod(0, 1.0, H)}, [[D, K, 10]])
	_caso("absorve_cheio", 100, {K: _mod(0, 1.0, H)}, [[D, K, 10]])
	_caso("absorve_meio", 50, {K: _mod(0, 0.5, H)}, [[D, K, 10]])
	_caso("inc_menos3_mult2", 100, {K: _mod(-3, 2.0)}, [[D, K, 10]])
	_caso("inc_mais5_mult0", 100, {K: _mod(5, 0.0)}, [[D, K, 10]])
	_caso("inc_menos20", 100, {K: _mod(-20, 1.0)}, [[D, K, 10]])
	_caso("outro_tipo", 100, {K: _mod(0, 2.0)}, [[D, M, 10]])
	_caso("cura_tipada", 50, {M: _mod(0, 2.0)}, [[H, M, 10]])
	_caso("cura_imune", 50, {M: _mod(0, 0.0)}, [[H, M, 10]])
	_caso("hurtbox_e_vida", 100, {K: _mod(0, 0.5)}, [[D, K, 10, _mod(0, 2.0)]])
	_caso("hurtbox_so", 100, {}, [[D, K, 10, _mod(0, 2.0)]])
	_caso("morte", 15, {K: _mod(0, 2.0)}, [[D, K, 10], [D, K, 10]])
	quit()
