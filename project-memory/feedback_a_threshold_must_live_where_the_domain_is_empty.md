---
name: feedback_a_threshold_must_live_where_the_domain_is_empty
description: Um limiar de classificação não pode cair num valor que uma entrada legítima produz — escolha onde o domínio é vazio e gateie enumerando as entradas.
metadata:
  type: feedback
---

Um limiar que separa "é uma feature" de "não é" (ou qualquer classificação por corte) **não pode
sentar num valor que uma entrada legítima consegue produzir exatamente**. Se sentar, o `f64` decide
de que lado cada empate cai pelo último bit, e entradas idênticas se classificam diferente.

**Caso medido (`ph2d-vec-blend`, o Blend do Vector, 2026-07-14):** a peneira "o que é uma quina" usou
`cos(15°)`. Um polígono regular de **24 lados vira exatamente 15°**. As 24 quinas idênticas caíam dos
dois lados da cerca por ruído — **transladar a cena** (sem deformar) mudava o intermediário em 5,5%;
1e-13 de ruído no raio dava 68 resultados distintos. Fix: **`cos(16°)`** — `360/16 = 22,5` não é
inteiro, então nenhum polígono regular produz 16°, e a cerca fica num vale vazio.

**Por que:** é o mesmo `f64`-decide-um-empate de [[feedback_a_snapshot_must_be_a_fixed_point_of_the_systems]]
e do z-order desta linha (*"não se escolhe um desempate melhor — não se tem empate"*). A regra não é
"calibre o limiar para quase nunca cair num valor real": é **escolher um valor que NENHUMA entrada
legítima consegue produzir**.

**How to apply:** ao pôr um corte num domínio conhecido (ângulos de polígono regular = `360/N`,
frações diádicas, valores de catálogo), enumere o domínio e escolha o limiar longe de todos. **Gateie
enumerando** — um gate que roda N até o teto do catálogo (`MAX_POLYGON_SIDES=128`) e exige margem é o
que impede o próximo a mexer no número de reintroduzir o empate. E o oráculo do usuário é
**invariância**: mover a arte pela tela não pode mudar o resultado ([[feedback_oracle_must_model_appearance_not_implementation]]).
