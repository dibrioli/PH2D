---
name: feedback_a_depth_bias_constant_in_ndc_is_not_constant_in_the_world
description: "Viés de profundidade escrito em NDC vale distâncias de mundo diferentes por ordens de grandeza conforme a profundidade — escreva-o como fracção da DISTÂNCIA, e aí a constante vira a resolução da oclusão"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-05T02:34:30.446Z
---

`WIRE_DEPTH_NUDGE = 3e-3` parecia minúsculo (`0,3 %` do alcance do buffer) e puxava a aresta
**`30 %` da distância do olho** para a frente na profundidade da própria peça — mais do que a
peça inteira tem de profundidade, o que **desarmava o teste de profundidade do wireframe
inteiro**. O NDC é hiperbólico e a câmera ancorava os planos na distância (`near = 0,01 d`,
`far = 100 d`), então `3e-3` de NDC valia `0,075` · `0,300` · `1,200` unidades a `0,5 d` · `1 d` ·
`2 d`.

A cura é uma linha: `z' = z + (z − w)·k`, que é exactamente a profundidade que o vértice teria a
`d·(1 − k)` (porque `ndc = C₀ + C₁/d`) ⇒ `Δd ≈ k·d`, sem precisar de `near`/`far` no shader.

**Why:** ⭐ o ganho não é só a correcção — **a constante passa a nomear o recurso** (§0.0): `k` É
a resolução da oclusão, e a varredura confirma a álgebra (o limiar em que a tinta reaparece É o
próprio `k`). Antes ela não nomeava nada e o valor tinha sido calibrado só pela metade
COPLANARIDADE, que puxa ao contrário.

**How to apply:** todo viés/epsilon de profundidade é suspeito de estar escrito na unidade
errada — converta-o para unidades de mundo **na profundidade em que a cena de facto vive** antes
de acreditar que é pequeno. E quando as duas metades de um número puxam em sentidos opostos
(coplanaridade × oclusão), escolha o ponto em que a primeira **satura**, medindo as duas:
[[feedback_two_halves_of_a_cure_each_refused_alone_do_not_refute_the_cure]].
