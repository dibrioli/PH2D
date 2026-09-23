---
name: feedback-timing-one-call-of-a-new-structure-measures-the-compiler
description: "Cronometrar UMA chamada de uma estrutura nova mede o compilador do driver, não o quadro — 1,4 a 4,4 s contra 6 ms de regime"
metadata:
  type: feedback
---

**A primeira chamada de uma estrutura NOVA paga a compilação do programa da placa, e ela é ordens de
grandeza acima do regime.** Medido 2026-09-21 (Render3d, `--release`, `99`–`100 %` de CPU ociosa): a
mesma cena pintada três vezes seguidas custa **`1,4`–`4,4 s`** na 1.ª e `6`–`130 ms` nas seguintes —
razão até **`245×`**. O custo é **`99,5 %` do DRIVER** (a nossa tradução WGSL→SPIR-V são `9,2` de
`1 776 ms`), e decompõe-se: `ms ≈ 1 307 + 2,62 × instruções-da-fita`, com o termo constante a ser o
kernel e a inclinação a ser a peça.

⚠️ **A prova de que não é a CPU é a segunda chamada**: ela refaz a mesma fita (a porta reconstrói o
campo a cada chamada) e custa `12 ms`.

**Why:** um gate que cronometra **uma** chamada por cena e lhe chama *«o quadro de movimento»* mede a
compilação. Sob essa régua, cenas individuais leram-se até **`11,7×`** diferentes entre duas corridas
do MESMO binário com a máquina parada, e o veredito do gate moveu-se de `8` para `10`–`12 de 22`
conforme a régua — *a deriva não era da máquina, era da primeira chamada*.

**How to apply:** toda régua de relógio sobre um caminho de GPU tira o **MÍNIMO DE N** (`N = 3` é o
joelho medido: a 1.ª compila, a 2.ª ainda apanha picos isolados). ⭐ E **a 1.ª chamada fica na
tabela, numa coluna própria** — ela é um preço real que o artista paga, e uma régua que a apaga faz
uma cura desaparecer com ela. ⛔ Para atribuir, cronometre `create_shader_module` (nosso) e
`create_compute_pipeline` (driver) **separados**: as duas metades têm curas opostas, e sem essa linha
*«compilar é caro»* é uma frase sem endereço. Ver [[reference_topic_measurement_discipline]] ·
[[feedback_a_cure_may_not_need_the_cause_ask_a_different_question]].
