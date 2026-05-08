--!strict
-- C3 spike fixture: waiter task body parametrizado. Test runner cria 1000
-- threads via `lua.create_thread(task_body)` e os spawna com waits no range
-- [0.500, 0.510), distribuindo erros em [0, dt) ao invés de coincidir
-- exatamente com tick boundaries.

function task_body(seconds: number)
    ph2d.wait(seconds)
end
