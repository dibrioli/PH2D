--!strict
-- BUG 1: Race condition — dois systems gravam Health sem ordering explícito.
-- Sintoma: valor final de Health é não-determinístico entre runs (depende de
-- ordem em que scheduler escalona os systems).

ph2d.system("damage_pass", function(_dt: number)
    for _, e in ph2d.query({ "Health", "TakingDamage" }) do
        local hp = ph2d.get(e.entity, "Health") :: number?
        local dmg = ph2d.get(e.entity, "TakingDamage") :: number?
        if hp and dmg then
            ph2d.set(e.entity, "Health", hp - dmg)
        end
    end
end)

ph2d.system("regen_pass", function(_dt: number)
    for _, e in ph2d.query({ "Health", "Regen" }) do
        local hp = ph2d.get(e.entity, "Health") :: number?
        local regen = ph2d.get(e.entity, "Regen") :: number?
        if hp and regen then
            ph2d.set(e.entity, "Health", hp + regen)
        end
    end
end)
