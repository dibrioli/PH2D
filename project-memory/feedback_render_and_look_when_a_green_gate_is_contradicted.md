---
name: feedback-render-and-look-when-a-green-gate-is-contradicted
description: Quando o Enio contradiz um gate VERDE, renderize a imagem e OLHE — um gate de igualdade-de-conjuntos não vê o que o olho vê
metadata:
  type: feedback
---

Quando o smoke do Enio contradiz um gate **verde**, não defenda o gate: **renderize a saída num PNG e
olhe para ela** (`Read` de imagem). O pixel é o oráculo; a métrica é só uma sombra dele.

**Why:** um gate pode ser *logicamente correto* e **cego para o sintoma**. No Impasto (2026-07-12) o gate
afirmava *"o pigmento existe exatamente onde a luz modela"* — verdade, provada, mutação-vermelha — e a
foto do Enio mostrava névoa mesmo assim. O suporte dos dois conjuntos era idêntico; o que ele via era
**quanta** tinta e **quanta** forma havia em cada pixel: a tinta desvanecia em 8 px de rampa suave, e uma
rampa suave sem forma 3D **é** uma névoa. Igualdade-de-conjuntos não distingue uma parede de um banco de
neblina. Renderizei o traço, olhei, e a névoa estava lá — no meu próprio harness, idêntica à foto dele.
Duas horas de teoria (build stale? outro caminho de depósito? pressão do mouse?) que uma imagem matou em
um minuto.

**How to apply:**
1. Escreva um teste `#[ignore = "measurement"]` que pinta e despeja o composite num PPM/PNG (zlib puro
   basta — sem dependência nova), e faça `Read` do arquivo.
2. Depois re-enuncie o gate **onde o olho lê**. Sintoma visual → métrica **de área ou de contraste**, não
   de suporte: *"de toda a tinta do traço, quanta não é nem sólida nem ausente?"* (52% → 28% → 13,5%
   pelas três versões — o número separa as três, o suporte não separava nenhuma).
3. Mantenha a mesma imagem como oráculo entre iterações: renderize de novo depois do fix e **compare**.

Irmã de [[feedback_oracle_must_model_appearance_not_implementation]] (o oráculo tem de modelar a
APARÊNCIA) e de [[feedback_painted_is_not_populated_paint_gate]] (teste a PINTURA). A diferença aqui: o
oráculo estava certo *e* era insuficiente — a lição é que **verde não é prova de que você mediu a coisa
certa**.
