--!strict
-- FSM básica: Idle → Walk → Run baseado em magnitude de velocity.
-- Componente Velocity assumido como { x: number, y: number }.

local IDLE_THRESHOLD = 0.1
local RUN_THRESHOLD = 5.0

local function magnitude(v: ph2d.Vec2): number
    return math.sqrt(v.x * v.x + v.y * v.y)
end

local function decide_state(speed: number, current: string): string
    if current == "idle" then
        if speed > IDLE_THRESHOLD then return "walk" else return "idle" end
    elseif current == "walk" then
        if speed < IDLE_THRESHOLD then return "idle"
        elseif speed > RUN_THRESHOLD then return "run"
        else return "walk" end
    else -- run
        if speed < RUN_THRESHOLD then return "walk" else return "run" end
    end
end

ph2d.system("fsm_movement", function(_dt: number)
    for _, e in ph2d.query({ "Velocity" }) do
        local v = ph2d.get(e.entity, "Velocity") :: ph2d.Vec2?
        if not v then continue end
        local store = ph2d.state_table(e.entity)
        local current = (store.state :: string?) or "idle"
        store.state = decide_state(magnitude(v), current)
    end
end)
