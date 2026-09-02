---
name: feedback-a-fraction-measured-against-a-reconstructed-whole-drifts-and-vibrates
description: "Um arrasto que calcula `t` somando as PARTES em vez de ler o TODO ganha offset quando uma terceira parte aparece — e TREMOR se a parte depender do resultado"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T21:19:44.342Z
---

Um divisor (splitter, seam, handle de proporção) escreve uma **fracção**. Se o denominador dela for
**reconstruído** — `parte_A + parte_B` — em vez de lido do dono do TODO, ele mente no dia em que
uma **terceira** parte aparece na mesma região.

**Medido na `line/UIUX`, 2026-08-31 (report do Enio: *«arrastar o topo do canvas de nós tem um bug,
um offset e um tremor»*):**

| quem | denominador |
|---|---|
| o painel, ao arrastar | `center_viewport + motion_graph` = `chrome_h − altura_da_timeline` |
| o layout, ao aplicar (`top = banda · t`) | `chrome_h` |

A soma **era** a banda — até a timeline docar dentro do split e comer o fundo de uma das metades.
Alvo 1366 × 1024: o dedo larga em `352` e o divisor vai para `448` (**96 px**).

⭐⭐ **E o TREMOR é a segunda metade, não ruído:** a altura da timeline é ela própria **clampada
pela altura do grafo**, logo o denominador **depende do resultado**. Com o dedo *parado* dez
quadros: `672 → 665,3 → 670,9 → 666,2 …` — oscilação de **6,7 px**.

> *Uma fracção medida contra uma grandeza que depende dela não converge: ela vibra.*

⇒ a cura é o TODO ter **um dono que o publica** (`HeroLayout::split_band`) e ninguém o
reconstruir.

⚠️ **E o gate tem de ser de IDA-E-VOLTA**, atravessando as duas metades: *a fracção que o ponteiro
pede põe o divisor DEBAIXO do ponteiro*. Medir só a fórmula confirma a fórmula — o defeito era ela
não ser a **inversa** de quem a aplica. Junte o irmão *«dedo parado, N quadros, deriva ≈ 0»*: é o
único que apanha o tremor.

⛔ E amostre **dentro do domínio legal**: com alvos em px fixos o `clamp_t` movia o resultado 4 px
e a 1.ª redacção do gate acusou a **cerca** de ser o defeito.

**How to apply:** ao ver um `x - a.x) / (b.x + b.w - a.x)` num arrasto, pergunte *«quem mais pode
entrar nesta região?»*. Se a resposta não for «ninguém», o denominador tem de vir do dono da região.

Relacionadas: [[feedback_paint_and_hit_test_must_project_through_one_door]] ·
[[feedback_a_ratio_bar_tightens_itself_when_the_denominator_is_a_knob]] ·
[[feedback_a_quantity_cannot_both_position_and_limit]]
