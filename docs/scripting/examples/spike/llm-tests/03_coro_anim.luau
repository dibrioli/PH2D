--!strict
-- Coroutine de animação: tween de cor de sprite ao longo de 30 frames (~0.5s).
-- Cada chamada cria coroutine que dura 30 frames depois finaliza.

type Color = { r: number, g: number, b: number, a: number }

local function lerp(a: number, b: number, t: number): number
    return a + (b - a) * t
end

local function fade_color(entity: ph2d.Entity, from: Color, to: Color)
    local FRAMES = 30
    local DT = 1.0 / 60.0
    for frame = 0, FRAMES - 1 do
        local t = frame / (FRAMES - 1)
        ph2d.set(entity, "Color", {
            r = lerp(from.r, to.r, t),
            g = lerp(from.g, to.g, t),
            b = lerp(from.b, to.b, t),
            a = lerp(from.a, to.a, t),
        })
        ph2d.wait(DT)
    end
end

return fade_color
