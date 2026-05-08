--!strict
-- C7 spike: representative gameplay-sized Luau (~3 KB).
-- Mix of FSM, message handlers, animation, dialogue branching.

local Player = {}
Player.__index = Player

function Player.new(name: string, hp: number)
    local self = setmetatable({}, Player)
    self.name = name
    self.hp = hp
    self.max_hp = hp
    self.state = "idle"
    self.position = { x = 0, y = 0 }
    self.velocity = { x = 0, y = 0 }
    self.inventory = {}
    self.flags = {}
    return self
end

function Player:tick(dt: number)
    self.position.x = self.position.x + self.velocity.x * dt
    self.position.y = self.position.y + self.velocity.y * dt

    if self.state == "idle" then
        if math.abs(self.velocity.x) > 0.1 or math.abs(self.velocity.y) > 0.1 then
            self:transition("walk")
        end
    elseif self.state == "walk" then
        local speed = math.sqrt(self.velocity.x ^ 2 + self.velocity.y ^ 2)
        if speed < 0.1 then
            self:transition("idle")
        elseif speed > 5 then
            self:transition("run")
        end
    elseif self.state == "run" then
        local speed = math.sqrt(self.velocity.x ^ 2 + self.velocity.y ^ 2)
        if speed < 5 then
            self:transition("walk")
        end
    end
end

function Player:transition(to: string)
    self.state = to
end

function Player:on_damage(amount: number, source: string)
    self.hp = math.max(0, self.hp - amount)
    if self.hp == 0 then
        self:transition("dead")
    end
end

function Player:on_heal(amount: number)
    self.hp = math.min(self.max_hp, self.hp + amount)
end

function Player:set_flag(name: string, value: boolean)
    self.flags[name] = value
end

function Player:has_flag(name: string): boolean
    return self.flags[name] == true
end

local Dialogue = {}

function Dialogue.run(player, npc_id: string)
    if npc_id == "elder" then
        if player:has_flag("met_elder") then
            return "Welcome back, traveler."
        else
            player:set_flag("met_elder", true)
            return "I have been waiting for you."
        end
    elseif npc_id == "merchant" then
        if #player.inventory > 5 then
            return "Looks like you've been busy."
        else
            return "Care to see my wares?"
        end
    end
    return "..."
end

local Animation = {}

function Animation.tween(start: number, finish: number, t: number): number
    local clamped = math.clamp(t, 0, 1)
    return start + (finish - start) * clamped
end

function Animation.ease_out(t: number): number
    local clamped = math.clamp(t, 0, 1)
    return 1 - (1 - clamped) * (1 - clamped)
end

local function on_message(player, message: string, payload)
    if message == "damage" then
        player:on_damage(payload.amount, payload.source)
    elseif message == "heal" then
        player:on_heal(payload.amount)
    elseif message == "set_velocity" then
        player.velocity.x = payload.x
        player.velocity.y = payload.y
    elseif message == "talk_to" then
        return Dialogue.run(player, payload.npc_id)
    end
    return nil
end

return {
    Player = Player,
    Dialogue = Dialogue,
    Animation = Animation,
    on_message = on_message,
}
