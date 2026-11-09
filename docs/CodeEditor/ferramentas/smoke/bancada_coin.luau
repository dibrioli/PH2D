function init(self)
  self.t = 0
end

function update(self, dt)
  self.t = self.t + dt
  ph2d.set(self.id, "rotation", self.t * 2)
  if self.t > 2 then
    self.t = 0
    ph2d.emit("coin")
  end
end
