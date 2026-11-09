ph2d.property("speed", 220, { min = 0, max = 600 })
ph2d.property("lives", 3)

function init(self)
  self.coins = 0
end

function update(self, dt)
  local x = ph2d.get(self.id, "x")
  if ph2d.key_down("right") then
    ph2d.set(self.id, "x", x + self.speed / 100 * dt)
  end
end

function on_signal(self, signal, from)
  if signal == "coin" then
    self.coins = self.coins + 1
    print("coins:", self.coins)
  end
end
