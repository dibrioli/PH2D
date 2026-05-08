--!strict
-- BUG 2: Determinism violation — itera state_table com `pairs()` em vez de
-- `pairs_sorted()` em pipeline determinístico (HR-16). Hash de save varia
-- run-to-run pela ordem não-determinística de chaves.

local function serialize_player_flags(entity: ph2d.Entity): string
    local store = ph2d.state_table(entity)
    local result = ""
    -- BUG: pairs() não tem ordem garantida; viola HR-16 (storage lateral
    -- exige ordem alfabética de chaves para serialização determinística).
    for k, v in pairs(store) do
        result = result .. tostring(k) .. "=" .. tostring(v) .. ";"
    end
    return result
end

return serialize_player_flags
