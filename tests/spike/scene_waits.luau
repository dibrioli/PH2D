-- C3 (CodeEditor item 6): cada objecto espera o seu `alvo` (segundos) no `init`, pela espera da cena,
-- e escreve no `x` quanto passou a MAIS do que pediu.
ph2d.property("alvo", 0.5)

function init(self)
  local passou = ph2d.wait(self.alvo)
  ph2d.set(self.id, "x", passou - self.alvo)
end
