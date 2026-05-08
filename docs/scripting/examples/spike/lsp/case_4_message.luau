--!strict
-- Caso 4: Message handler. Hover em `payload` deve mostrar
-- `ph2d.MessagePayload = { [string]: any }`.

ph2d.message_handler("damage", function(sender, payload)
    -- sender: ph2d.Entity, payload: ph2d.MessagePayload (inferred from .d.luau)
    local amount = payload.amount :: number?
    local target = payload.target :: ph2d.Entity?
    if amount and target then
        local hp = ph2d.get(target, "Health") :: number?
        if hp then
            ph2d.set(target, "Health", hp - amount)
        end
    end
end)
