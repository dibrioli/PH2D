-- Fixture of tests/it/ganchos.rs: the engine hooks (`-- @hook` lines of ph2d.d.luau) are not «unused»,
-- and NOTHING else loses the warning. Lines ending in `-- ERR <code> <needle>` must yield exactly that.
local function helper() end -- ERR 8 'helper'
function updte(self, dt) end -- ERR 8 'updte'
function init(self) end
function update(self, dt) end
function on_signal(self, signal) end
do
  local function on_signal() end -- ERR 8 'on_signal'
end
