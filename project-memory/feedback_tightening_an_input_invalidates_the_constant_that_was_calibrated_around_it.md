---
name: feedback-tightening-an-input-invalidates-the-constant-that-was-calibrated-around-it
description: "Apertar uma entrada FROUXA de uma fórmula calibrada consome a margem que fazia a constante bastar — a tabela tem de ser re-medida, não a linha trocada"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-01T19:13:47.690Z
---

Uma fórmula de segurança validada por **tabela** (*«com este divisor `‖∇f‖ ≤ 1` em todo o alcance»*)
foi medida com os valores que as entradas dela tinham **naquele dia**. Se uma delas era um majorante
folgado, parte da margem que fez a tabela fechar **vinha da folga**, não da fórmula. ⇒ apertar essa
entrada — mesmo para o valor **correcto** — pode furar, e a fórmula continua «certa».

Caso medido (PH2D, `line/3DModeling`, 2026-09-01, report do Enio *«muitíssimo lento»*): o divisor da
torção é `σ = t/2 + √(1 + t²/4)` com `t = κ·R`, derivado **exactamente** (valor singular do
jacobiano) e confirmado por tabela **sem constante ajustada**. O `R` era `‖(cx,cy)‖ + raio_da_esfera`.
Trocá-lo pelo alcance honesto (o canto da **caixa**, `0,505` contra `0,717` numa barra
`0,34 × 0,11 × 0,62`) corta o divisor de `9,12` para `6,50` — e a peça **fura**: `1` pixel muda ao
dividir o passo por quatro.

**How to apply:**
- ⭐ Antes de apertar uma entrada, pergunte *«que medição justifica a constante que a consome?»* e
  **releia as condições dessa medição**. Se a entrada folgada estava lá dentro, apertar é re-abrir a
  medição inteira, não trocar uma linha ([[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]]).
- ⚠️ **«A álgebra é exacta» não imuniza**: ali era, e o que a salvava era o divisor ser *uma
  constante tirada no pior ponto* — a folga de `R` cobria a diferença entre o pior ponto e o resto.
  *Exacto no ponto ≠ exacto como constante.*
- ⭐⭐ **A prova é a INVARIÂNCIA AO PASSO, e ela não precisa de oráculo**: um passo seguro nunca acha
  mais peça ao ser encurtado. `1` pixel a mudar entre `passo` e `passo/4` é prova completa
  ([[feedback_a_gradient_gate_says_may_punch_only_the_image_says_punches]]) — e nesse mesmo dia o
  gate de `‖∇f‖` sobre **1 000 trios** ficou VERDE com o campo já a furar.
- ⇒ escreva a recusa **no sítio da entrada**, com o número que ela daria e o gate que reprovou.
