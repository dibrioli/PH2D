---
name: feedback_a_closed_surface_can_contain_a_second_one_count_the_components
description: "Uma almofada (duas faces coincidentes, costas com costas) passa em χ, bordo e não-manifold — só a CONTAGEM DE COMPONENTES a apanha"
metadata:
  type: feedback
---

Medido 2026-08-28: o artista fotografou uma **face solta a flutuar** sobre uma ponta. O
ficheiro exportado tinha `23 630` quads, **`0` arestas de bordo, `0` não-manifold** — e
**dois componentes ligados**, de `23 628` e de `2`. A ilha era `[68,69,70,71]` e
`[71,70,69,68]`: *o mesmo quadrado emitido duas vezes, um virado ao contrário.*

⛔ **Nenhuma régua de superfície a via**, e não por descuido: uma almofada é uma superfície
fechada legítima. `χ` conta os dois lados dela e dá `2`; o bordo é zero; o não-manifold é
zero; a contagem de quads **sobe**. *O que ela não é, é parte da peça* — e essa é uma
pergunta sobre **conectividade**, não sobre a superfície.

**Why:** todas as réguas de topologia desta família (`χ`, bordo, não-manifold, valência)
medem a malha **como um objecto só**. Um segundo objecto dentro dela é invisível a todas, e
o único sintoma é visual.

**How to apply:** onde uma malha é **produzida** (extracção, remesh, booleana), conte os
**componentes ligados por aresta** e diga o tamanho de cada um — uma linha de log barata que
nomeia exactamente o defeito que de outra forma só chega por foto. ⭐ E quando o produtor é
um mapa, o par coincidente tem uma causa nomeável (uma **dobra**: a mesma região percorrida
nos dois sentidos) e a chave que o apanha é o **ciclo sem sentido** — rodar para o menor nó e
ficar com o menor entre o anel e o seu inverso; ⚠️ caem **os dois** lados, porque uma
almofada não tem lado certo. Relacionado:
[[feedback_a_gate_on_the_mark_i_chose_is_green_when_the_marks_premise_is_false]] ·
[[feedback_counting_the_work_done_is_not_counting_the_work_delivered]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]]
