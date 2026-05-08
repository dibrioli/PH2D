--!strict
-- C10 spike fixture: GC pause sob stress.
-- Setup: cria 10_000 tabelas Lua vivas + 1_000 closures pendentes.
-- Test runner ticka, alocando tabelas curtas a cada frame, e mede pause de GC.

local persistent = table.create(10000)
for i = 1, 10000 do
    persistent[i] = { x = i, y = i * 2, z = i * 3, name = "entity_" .. i }
end

local coroutines = table.create(1000)
for i = 1, 1000 do
    coroutines[i] = coroutine.create(function()
        ph2d.wait(1000.0) -- effectively never resumes during test
    end)
    -- Resume once to enter wait state
    coroutine.resume(coroutines[i])
end

-- Per-tick allocation: forces GC incremental work
function tick_alloc()
    local local_garbage = {}
    for j = 1, 50 do
        local_garbage[j] = { tag = "ephemeral", n = j, tbl = { 1, 2, 3 } }
    end
    return #local_garbage
end
