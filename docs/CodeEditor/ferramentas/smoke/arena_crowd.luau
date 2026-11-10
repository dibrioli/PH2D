-- arena.luau: ALL fourteen shapes run THIS script, and it is full of mistakes on purpose.
-- Problems (below) lists every one; Who has what (right) shows each shape's values.
-- An amber cell is a value that shape changed for itself: hover it to read why.
ph2d.property("speed", 2.0, { min = 0, max = 10 })
ph2d.property("jump_height", 3)
ph2d.property("visible", true)
ph2d.property("label", "enemy")
ph2d.property("tint", ph2d.color(1, 0.5, 0))
ph2d.property("offset", ph2d.vec2(0, 1))
ph2d.property("mode", "walk", { options = { "walk", "run", "fly" } })

local unused_timer = 0

local function clamp(v, lo, hi)
  return math.max(lo, math.min(hi, v))
end

function init(self)
  self.t = 0
  self.base_y = ph2d.get(self.id, "y")
end

function update(self, dt)
  self.t = self.t + dt
  local math = 2
  local s = clamp(self.t * self.sped, 0, 4, 9)
  ph2d.set(self.id, "y", self.base_y + s * self.jump_hieght)
  ph2d.set(self.id, "rotaton", self.t)
  ph2d.set(self.id, "xx", 0)
  print(self.ofset)
  if self.visibel then
    print(self.lable)
    ph2d.spawn("Bulet", 0, 0)
  end
  print(self.tnit)
  print(ph2d.object("Runer"))
  if self.t > 1 then
    local hud = ph2d.find_by_name("Score HUD")
    print(hud.x)
  end
end
