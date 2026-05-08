--!strict
-- BUG 3: Coroutine leak — `ph2d.cancel` chamado sem scope_guard, GPU resource
-- (texture handle) não-liberado quando coroutine é cancelado mid-flight.
-- Sintoma: VRAM cresce monotonicamente; após N hot reloads, OOM.

type TextureHandle = number

declare function gpu_alloc_texture(): TextureHandle
declare function gpu_free_texture(h: TextureHandle): ()

local function fade_with_temp_texture(entity: ph2d.Entity)
    -- Aloca recurso GPU para usar durante o fade.
    local tex = gpu_alloc_texture()
    -- BUG: se a coroutine for cancelada antes do fim do loop (ex: hot reload,
    -- entity destruída), `gpu_free_texture(tex)` nunca executa. Não há
    -- defer/scope_guard que garanta cleanup em path de cancelamento.
    for frame = 0, 30 do
        ph2d.set(entity, "Texture", tex)
        ph2d.wait(1.0 / 60.0)
    end
    gpu_free_texture(tex)
end

return fade_with_temp_texture
