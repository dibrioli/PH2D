--!strict
-- Caso 2: Coroutine com ph2d.wait. Hover em `ph2d.wait` deve mostrar
-- `(seconds: number) -> ()`.

local function fade_routine()
    for _ = 1, 10 do
        ph2d.wait(0.1)
        -- autocomplete depois de `ph2d.` deve listar wait, spawn, query, etc.
    end
end

return fade_routine
