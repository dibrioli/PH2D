-- Fixtura de ph2d-code-syntax: cópia de `BOB_LUAU` (crates/ph2d-app-components/src/script_smoke.rs,
-- commit b3f98d4773). Cópia, não a fonte: se a amostra canónica mudar, esta fica como estava.
-- bob.luau — sobe e desce.
--
-- Os números abaixo aparecem no painel (secção Script) de cada boneco.
-- Mude um default aqui e grave: o boneco que NÃO tem número próprio muda na hora;
-- os outros ficam com os números deles.

ph2d.property("amplitude", 1.0, { min = 0, max = 4 })  -- quanto sobe (metros)
ph2d.property("speed", 2.0, { min = 0, max = 10 })     -- quão depressa
ph2d.property("active", true)                          -- desligado = fica parado
ph2d.property("top_signal", "")                        -- o grito, a cada topo (vazio = calado)
-- ⭐ A LISTA: o painel pinta um CHIP em vez de um campo livre, e um valor que não esteja aqui
--    não chega a este script — ele é NOMEADO no painel em vez de cair no ramo errado calado.
ph2d.property("wave", "sine", { options = { "sine", "square", "still" } })
-- ⭐ UMA FILEIRA COM DOIS CAMPOS: uma posição é UMA coisa, e não duas propriedades que você tem
--    de se lembrar de manter juntas. `(0, 1)` é a reta para cima — o de sempre, ao bit.
ph2d.property("direction", ph2d.vec2(0, 1), { min = -1, max = 1 })
-- ⭐ UMA AMOSTRA: carregue nela e o selector de cor do app abre.
--    ⚠️ O script ainda NÃO sabe pintar — o vocabulário dele é só posição —, então aqui a cor
--    entra como os três números que ela é: quanto mais CLARA, mais alto ele sobe.
ph2d.property("tint", ph2d.color(1, 1, 1))

function init(self)
  self.base_x = ph2d.get(self.id, "x")
  self.base_y = ph2d.get(self.id, "y")
  self.t = 0
  self.was_up = false

function update(self, dt)
  if not self.active then return end
  self.t = self.t + dt
  -- ⭐ A FORMA da onda sai da lista: o chip do painel escolhe qual destas corre.
  local s
  if self.wave == "square" then
    s = math.sin(self.t * self.speed) >= 0 and 1 or -1
  elseif self.wave == "still" then
    s = 0
  else
    s = math.sin(self.t * self.speed)
  end
  -- ⭐ O BRILHO da cor escala o passeio, e a DIREÇÃO diz para onde ele é.
  local brilho = (self.tint.r + self.tint.g + self.tint.b) / 3
  local d = s * self.amplitude * brilho
  ph2d.set(self.id, "x", self.base_x + d * self.direction.x)
  ph2d.set(self.id, "y", self.base_y + d * self.direction.y)
  local up = s > 0.98
  if up and not self.was_up and self.top_signal ~= "" then
    ph2d.emit(self.top_signal)
  end
  self.was_up = up
end
