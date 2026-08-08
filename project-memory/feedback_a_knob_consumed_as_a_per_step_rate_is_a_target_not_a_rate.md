---
name: feedback-a-knob-consumed-as-a-per-step-rate-is-a-target-not-a-rate
description: "Knob que o motor aplica a cada tick/eco/passo tem resposta EXPONENCIAL no slider e é composta por outro knob (o comprimento do laço) — o número autorado tem de ser o ESTADO FINAL, com a taxa DERIVADA"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 24c5926d-98e2-4eed-a7d7-acfbda48f858
  modified: 2026-08-08T16:35:37.378Z
---

Enio, sobre o `motion.trail` (2026-08-07): **"sliders mal balanceados. A menor mudança faz um extremo
efeito. Saturação 0.9 já fica quase todo dessaturado. Reveja tudo."** Estava certo, e eram DOIS defeitos
com um mecanismo só: o motor fazia `valor *= knob` a cada tick, então o que a ponta da cauda mostra é
`knob^vão` — **exponencial no slider** — e o `vão` é `(length−1)·spacing`, ou seja **outro knob
MULTIPLICA o efeito deste**. Medido: `saturation 0.90` entregava **0,17**; a faixa que dá entre 0,9 e 0,1
media **13,3% do curso** no default e **1,9%** com o spacing largo.

**Why:** um número por-passo não é uma grandeza que o artista consegue imaginar — ele imagina o
RESULTADO ("quero a ponta quase cinza"). Pior, a composição acopla dois controles em silêncio, e o
acoplamento vive dentro do motor, onde nenhum gate de valor único o vê. O mesmo mecanismo estava no
`motion.strobe` (`decay` por-tick: **86% do curso** cobria 5..34 ticks e **14%** cobria 34..551).

**How to apply:**
- Sintoma mensurável, e é ele que transforma "mal ajustado" em achado: **que fração do curso do slider
  fica na faixa útil?** Meça pela porta do PRODUTO, varrendo o knob. Abaixo de ~30% já é bug.
- A cura é de MODELO, não de calibragem ([[feedback_ergonomics_verdict_is_a_design_bug]]): o número
  autorado passa a ser o **ALVO no fim do processo** e o motor DERIVA a taxa (`rate^vão == alvo`). O
  slider fica linear e — a metade que importa — **o valor não se move quando o outro knob se move**.
- Ângulo por-passo vira **TOTAL** ao longo do processo, pelo mesmo motivo (`hue_shift 35` virava 700°
  reais, e a const chamava-se `HUE_PER_ECHO`: o nome e o doc mentiam juntos).
- **Derive os defaults, não os escolha** — resolva a lei nova para o número que a lei velha produzia no
  default do nó. Aí a arte existente não se move (e isso é gate). Ver
  [[feedback_an_absolute_unit_that_should_feel_relative_must_scale_with_the_geometry]], que usa a mesma
  âncora de referência.
- Ponha **PISO** vindo do consumidor, nunca um épsilon inventado: alvo 0 com taxa 0 colapsa no PRIMEIRO
  passo (penhasco onde o artista pediu rampa). No render, o piso é `1/255` — um nível de 8 bits.
- ⚠️ Cuidado ao expressar o knob em SEGUNDOS: se o motor lê um `dt` que pode ser 0 (dentro de um escopo
  de tempo) a taxa vira 1.0 e **o efeito nunca decai** — regressão pior que o slider torto. Prefira a
  unidade que o próprio módulo já fala (ticks/ecos).
