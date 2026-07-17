---
name: feedback_a_fixture_can_land_in_a_chaotic_regime
description: "Gate de paridade que exige um regime caótico: o problema é o FIXTURE, não o ε — magnitude limitada com sinal virando é divergência máxima"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 62ac077f-09f4-41be-9a44-14a0a85668a9
---

Um gate de paridade pode escolher, sem querer, um regime onde os dois lados **legitimamente** divergem.
O sintoma engana: parece ruído numérico, e a saída óbvia é afrouxar o ε. É a saída errada — o oráculo
passaria a modelar o filtro, não a verdade ([[reference_topic_oracle_discipline]]). **Mude o fixture.**

**O caso (GPU/M5 Fase 3, `force.buoyancy`, 2026-07-16).** O clamp `wave_length.max(1e-3)` só é
observável onde o domínio dele é vazio ([[feedback_a_threshold_must_live_where_the_domain_is_empty]]),
então o gate usa `wave_length = 0`. Mas o valor que o clamp **entrega** (`1e-3`) é ele próprio um mar
patológico: `slope = amp·2π/λ ≈ 3770·cos`, a normal da superfície deita quase horizontal, e a
**direção** dela vira com o **sinal do cosseno**. CPU e GPU divergiram **0,2022** — que é exatamente
`2·density·dt²`, a assinatura de um flip, não de ruído.

**Eu tinha raciocinado que era seguro, e o raciocínio estava meio certo:** *"a normal é unitária, logo
`|a| ≤ density`, logo limitado"*. Verdade sobre a **magnitude** — e irrelevante. **Magnitude limitada
com sinal virando É a divergência máxima**, e 1 ulp de fase decide qual lado. Perto de um cruzamento
por zero, "limitado" não é "estável".

O conserto foi `amplitude = 1e-4`: o **mesmo** mar clampado, agora com `slope ≈ 0,63` — bem
condicionado — e o clamp segue igualmente observável, porque **quem NaN-a um kernel sem clamp é a
`phase`**, e a amplitude não toca nela. Δ caiu de 0,2022 pra 7e-4, dentro do orçamento intocado.

**Why:** num sistema com realimentação (ADR-0123 D4) o ε cresce sozinho; o gate compara UM passo justo
por isso. Mas um passo só também estoura se a função for descontínua ali. "Divergiu muito" tem duas
causas — porte errado **ou** fixture caótico — e elas pedem consertos opostos.

**How to apply:** Δ grande num gate de paridade? Antes de mexer no ε, **compare o Δ com uma quantidade
FÍSICA do fixture** (aqui, `2·density·dt²`). Se bater com "a resposta inteira, com sinal trocado", é
descontinuidade: ache o param que endireita a função sem tirar do gate o que ele testa. Cheque
especialmente normalização (`normalize`, `1/sqrt(...)`) sob argumento grande — é onde a direção vira
e a magnitude não avisa. Ver [[feedback_frozen_bar_check_the_arithmetic_before_gaming_it]] (o mesmo
reflexo: faça a conta antes de mexer no número) e [[reference_topic_mutation_proofs]].
