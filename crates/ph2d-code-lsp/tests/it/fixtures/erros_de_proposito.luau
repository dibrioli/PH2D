-- Fixture of the blindness control (crates/ph2d-code-lsp/tests/it/controlo_de_cegueira.rs), checked
-- against runtime/luau/ph2d.d.luau. Every line ending in `-- ERR <code> <needle>` must yield EXACTLY
-- one diagnostic with that code whose message contains <needle>; any other diagnostic is a failure.
local id = 1
ph2d.propertyy("speed", 2.0) -- ERR 1002 'propertyy'
ph2d.set(id, "x", "fast") -- ERR 1000 'string'
ph2d.emit("top")
