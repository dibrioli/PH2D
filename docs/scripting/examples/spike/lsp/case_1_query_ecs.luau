--!strict
-- Caso 1: Query ECS. Hover em `e` deve mostrar `ph2d.QueryResult`.
-- Autocomplete em `e.entity` deve sugerir `Entity = number`.

ph2d.system("update_pos", function(dt: number)
    for _, e in ph2d.query({ "Position", "Velocity" }) do
        local pos = ph2d.get(e.entity, "Position")
        local vel = ph2d.get(e.entity, "Velocity")
        if pos and vel then
            -- Hover em pos/vel deve sugerir os fields tipados.
        end
    end
end)
