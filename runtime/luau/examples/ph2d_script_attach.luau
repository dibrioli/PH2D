--!strict
-- ph2d_script_attach.luau
--
-- ADR-0025 M14.2 example: per-entity Luau script with lateral state.
--
-- Demonstrates the canonical "MonoBehaviour"-style pattern in PH2D:
--   1. The script is just a top-level function that runs once at
--      `init()` and once per `tick(dt)`. (The host calls these by
--      name on the bytecode this script compiles to.)
--   2. Per-instance state lives in the lateral [`StateTable`], read
--      and written via `ph2d.state_get` / `ph2d.state_set`. Survives
--      hot reload by design.
--   3. The script never holds a live world handle (HR-8); all data
--      transfer goes through the typed bindings declared in
--      `runtime/luau/ph2d.d.luau`.
--
-- This file is parsed by `luau-analyze --strict` in CI (HR-17), so
-- any type drift between the catalog and this example fails the
-- build loudly.

local function init(self: number)
    -- Initialize hp once on first attach. Re-reads after a hot
    -- reload pick up the existing value (state_get returns the last
    -- set value, never nil after first set).
    local hp = ph2d.state_get(self, "hp")
    if hp == nil then
        ph2d.state_set(self, "hp", 100)
    end
    ph2d.state_set(self, "name", "Enemy")
end

local function tick(self: number, dt: number)
    -- Trivial decay so the example produces visible state churn.
    local hp = ph2d.state_get(self, "hp") :: number?
    if hp and hp > 0 then
        ph2d.state_set(self, "hp", hp - dt)
    end
end

-- Idiomatic pattern: spawn a sibling entity by name. The
-- `find_by_name` resolve happens *next* tick once the host applies
-- the SpawnQueue, so the typical use is "spawn at init, find at
-- tick".
local function ensure_companion(self_id: number)
    local target = ph2d.find_by_name("Companion")
    if target == nil then
        ph2d.scene.spawn_entity()
        -- Tag the freshly-spawned entity once the host gives it a
        -- Name via `scene.add_component` — out of scope for this
        -- minimal example.
    else
        ph2d.message.send(target, "alert", { source = self_id })
    end
end

return {
    init = init,
    tick = tick,
    ensure_companion = ensure_companion,
}
