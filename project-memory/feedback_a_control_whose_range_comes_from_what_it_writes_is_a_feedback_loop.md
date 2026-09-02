---
name: feedback-a-control-whose-range-comes-from-what-it-writes-is-a-feedback-loop
description: "Alcance derivado do que o próprio controlo escreve: o valor diverge com a mão PARADA — e uma função pura nunca o revela"
metadata:
  node_type: memory
  type: feedback
---

Um controlo cujo **alcance** é derivado de algo que ele próprio escreve fecha um **laço de
realimentação**. Não é «um salto»: é uma recorrência, e acima de um limiar ela **diverge**.

Caso medido (PH2D, `line/3DModeling`, 2026-08-30). Três factos que sozinhos parecem certos:
1. o slider mapeia `valor = lo + track·(hi − lo)`, com `track` a posição **absoluta** do dedo;
2. o alcance saía de `span(raio_da_peça)`;
3. o raio da peça **é o que este slider escreve**.

⇒ `v ← t · span(4v)`, e para `t > ¼` diverge. Com o ponteiro **completamente parado** em
`track = 0,8`: `2,4 → 9,6 → 19,2 → … → 2 457,6`; a 20 quadros, `track = 0,76` dá `1 090 518`.
Report: *«arrastar os sliders ficou bizarro mudando valores aos pulos»*.

⚠️ **A fonte anterior (a câmera) era um insumo EXTERNO ao objecto.** Trocá-la pela peça — para curar
um report legítimo ([[feedback_a_control_range_derived_from_the_camera_moves_under_a_gesture_that_changes_nothing]]) — foi o que fechou o laço.

**Why:** o quantizador (oitavas) só troca um incómodo **contínuo** por um salto **discreto de 2×**,
que é pior. A cura é **travar** o alcance enquanto a mão está no controlo.

**How to apply:**
- Ao derivar uma faixa, pergunte: *este insumo é escrito por este controlo?* Se for, **trave-o** por
  gesto (ou por selecção) — [[feedback_a_knob_whose_range_is_derived_from_the_object_it_rewrites_is_not_idempotent]].
- ⛔⛔ **Uma função pura NUNCA revela um laço.** Gates que medem a lei isolada, com o insumo fixo,
  ficam verdes sobre a divergência. O gate tem de correr a **composição fechada** — `N` quadros com
  o **ponteiro parado** — e exigir um **ponto fixo**.
- ⚠️ Um gate que lê o **texto da chamada** apanha a dependência nomeada e mais nada: repor a fonte
  proibida através de um helper passa verde.
