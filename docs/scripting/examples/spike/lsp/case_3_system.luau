--!strict
-- Caso 3: System callback. Hover em `dt` deve mostrar `number`.

ph2d.system("damage_tick", function(dt: number)
    -- dt deve ter inferência number automática.
    local _decay = dt * 0.5
    for _, e in ph2d.query({ "Health" }) do
        local hp = ph2d.get(e.entity, "Health")
        if hp then
            -- decremento idiomático
            ph2d.set(e.entity, "Health", math.max(0, (hp :: number) - 1))
        end
    end
end)
