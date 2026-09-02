---
name: feedback-when-a-ratio-gate-accuses-and-the-image-does-not-look-for-the-formulas-singularity
description: Gate de razão acusa e a imagem não confirma? Procure a SINGULARIDADE da fórmula dentro do domínio de amostragem antes de mexer no modelo
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-31T21:58:23.440Z
---

Quando um gate de razão acusa e a **imagem não confirma**, a hipótese a testar **primeiro** não é
«o modelo está errado» — é *«a fórmula tem uma singularidade dentro do domínio onde a sonda
amostra?»*. Descontinuidades de **representação** (corte de ramo do `atan2`, `atan2(0,0)`, divisão
perto de zero, `acos` fora de `[−1,1]`) produzem gradientes enormes em pontos que **nenhum consumidor
visita**.

Caso medido (PH2D, `line/3DModeling`, 2026-08-30 → 31). A dobra media `‖∇f‖ = 1,72` sozinha e
`44,6` em par. **Três curas foram desenhadas, todas sobre a CURVATURA**, e uma foi shipada e
**rejeitada pelo dono** (*«VC danificou o Bend»* — a peça deixava de dobrar). A causa real:

1. a caixa de recorte alcança `x = 1,4036` com `rho = 1,3263` ⇒ `a = rho − x` fica **negativo**;
2. ali `atan2(b, a)` salta de `+pi` para `−pi` ao cruzar `b = 0`;
3. a banda clampa os dois lados do salto em **bordas opostas**.

Empurrar `a` para a parede da peça — o `piso` que a função **já declarava**, aplicado também ao
ÂNGULO e não só ao raio — levou `[Twist, Bend]` a `0,2077`, a dobra sozinha a `0,8130` e
`[Bend, Radial]` de `245,77` a `0,28`–`0,49`, **sem tocar no que o artista vê**.

**Why:** um gate de razão mede a **fórmula** no domínio que lhe deram; a imagem mede o que o
consumidor faz. Quando discordam, a diferença está entre os dois — e o sítio mais barato de a
procurar é a aritmética, não o modelo.

**How to apply:**
- Antes de mudar um modelo por causa de um número: **imprima ONDE o extremo está** e verifique se
  aquele ponto é alcançável pelo consumidor. Aqui o pior ponto era `x = 1,4036`, fora da peça.
- ⚠️ **Quem cura a causa de um número tem de reconferir TODO limite que aquele número justificava**
  (§0.0). Aqui a parede da curvatura ficou com uma tabela obsoleta a defendê-la; ao medi-la de novo,
  ela **ainda se paga — mas por outra razão** (sem ela a imagem parte em `478` de `1 610` pixels,
  enquanto o gradiente MELHORA).
- ⭐ ⇒ a lei completa tem os dois sentidos: *um gate de gradiente diz «pode furar» sem dizer «fura»,
  e diz «não fura» sem dizer «desenha certo»* — **nenhuma das duas réguas manda na outra**
  ([[feedback_a_gradient_gate_says_may_punch_only_the_image_says_punches]]).
- Uma cura que baixa o número **e** muda o produto é suspeita: ela pode estar a apagar o sintoma
  encolhendo o domínio ([[feedback_a_correct_mechanism_can_prescribe_the_wrong_cure]]).
