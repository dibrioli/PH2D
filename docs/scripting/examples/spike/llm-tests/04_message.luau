--!strict
-- Mensageria pub/sub: system A envia on_damage para entity B; B reduz Health.
-- Estilo Defold (FIFO same-sender→same-target, hash interning).

ph2d.message_handler("damage", function(_sender: ph2d.Entity, payload: ph2d.MessagePayload)
    local target = payload.target :: ph2d.Entity?
    local amount = payload.amount :: number?
    if not target or not amount then return end
    local hp = ph2d.get(target, "Health") :: number?
    if hp then
        ph2d.set(target, "Health", math.max(0, hp - amount))
    end
end)

local function attack(attacker: ph2d.Entity, victim: ph2d.Entity, amount: number)
    ph2d.message_send(victim, "damage", {
        target = victim,
        amount = amount,
        from = attacker,
    })
end

return attack
