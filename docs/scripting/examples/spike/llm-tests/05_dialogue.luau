--!strict
-- Dialogue branching: árvore de fala com 2 ramos baseada em flag de mundo.
-- Storage lateral: state_table guarda flags persistentes por entity.

local function talk_to_elder(player: ph2d.Entity)
    local store = ph2d.state_table(player)
    local met_before = (store.met_elder :: boolean?) or false

    if met_before then
        ph2d.dialogue.say("Welcome back, traveler.")
        local choice = ph2d.dialogue.choice({
            "Tell me about the prophecy",
            "I must be going",
        })
        if choice == 1 then
            ph2d.dialogue.say("The stars will align in 3 cycles.")
        else
            ph2d.dialogue.say("Safe travels.")
        end
    else
        ph2d.dialogue.say("I have been waiting for you.")
        store.met_elder = true
        ph2d.dialogue.say("Take this — you will need it.")
    end
end

return talk_to_elder
