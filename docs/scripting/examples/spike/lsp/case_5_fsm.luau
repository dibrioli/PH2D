--!strict
-- Caso 5: FSM via state_table. Hover em `store` deve mostrar
-- `{ [string]: any }`. Cast `:: string?` deve dar autocomplete em strings.

local function tick_fsm(entity: ph2d.Entity)
    local store = ph2d.state_table(entity)
    local current = (store.state :: string?) or "idle"
    local next_state: string

    if current == "idle" then
        next_state = "walk"
    elseif current == "walk" then
        next_state = "run"
    else
        next_state = "idle"
    end

    store.state = next_state
end

return tick_fsm
