---
name: feedback_identical_fixtures_hide_the_tiebreak_you_meant_to_test
description: "Fixture com os dois lados IDÊNTICOS não arma empate — dá 0.0 exato de um lado e 1e-16 do outro, e o ruído decide certo por acidente"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8754297a-910d-4d2d-a819-e6c12b137235
---

Um gate que quer provar um **critério de desempate** (a regra que escolhe quando duas opções
parecem iguais) **não pode usar fixture simétrico/idêntico**. Contornos idênticos produzem
métricas **exatamente** iguais (`0.0`), e contornos meramente parecidos produzem `1e-16` de ruído
de arredondamento — então a comparação **não empata**, e o critério que você quer testar nunca é
consultado. O gate fica verde COM e SEM a regra.

**Why:** no `ph2d-vec-blend` (compound path, 2026-07-16), o filtro de "papel" (profundidade de
aninhamento) impede o contorno de FORA de casar com o BURACO. O 1º gate usava duas rosquinhas
concêntricas e idênticas, apostando que as 4 viagens seriam zero e o desempate cairia do lado
errado. Não caía: `travel(A.outer, B.outer)` era `0.0` (geometria idêntica) e
`travel(A.outer, B.hole)` era `1e-16` — a **distância**, por acidente de `f64`, já fazia o
trabalho do filtro. Mutação: removi o filtro, o gate seguiu VERDE. Sobrevivente = gate faltando
([[reference_topic_mutation_proofs]]).

**How to apply:** arme o fixture para que a resposta ERRADA seja **estritamente mais barata** pelo
critério secundário — não meramente empatada. (Ali: buraco de B descentrado, de modo que o par
errado custasse `0.0` e o certo `1.0`. Aí o filtro é a ÚNICA coisa que pode salvar, e a mutação
mata.) Corolário do mesmo dia: o **probe** do oráculo não pode cair em cima da fronteira da
hipótese contradita — um ponto sobre a borda não tem resposta, a contagem de cruzamentos vira
empate de `f64`, e o gate vira cara-ou-coroa.

Irmão de [[reference_topic_fixture_discipline]] (um fixture só prova o que contém) e de
[[reference_topic_oracle_discipline]].
