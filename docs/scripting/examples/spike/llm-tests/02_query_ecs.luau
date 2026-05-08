--!strict
-- Query ECS canônica: itera entities com Position + Velocity + Sprite,
-- atualiza posição. dt é segundos por frame.

ph2d.system("update_position", function(dt: number)
    for _, e in ph2d.query({ "Position", "Velocity", "Sprite" }) do
        local pos = ph2d.get(e.entity, "Position") :: ph2d.Vec2?
        local vel = ph2d.get(e.entity, "Velocity") :: ph2d.Vec2?
        if pos and vel then
            ph2d.set(e.entity, "Position", {
                x = pos.x + vel.x * dt,
                y = pos.y + vel.y * dt,
            })
        end
    end
end)
