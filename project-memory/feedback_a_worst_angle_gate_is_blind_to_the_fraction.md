---
name: feedback-a-worst-angle-gate-is-blind-to-the-fraction
description: "«Quão mau é o pior» e «quantos sítios estão maus» são DUAS perguntas — um gate de pior-ângulo fica verde sobre 11 % da superfície estragada"
metadata:
  type: feedback
---

⭐⭐ **Um gate que mede o PIOR de uma grandeza é cego à FRACÇÃO dela, e vice-versa. São duas
réguas, e o olho do dono lê a segunda.**

**Why:** medido em 2026-09-08 (W142). O `the_chamfer_never_makes_an_edge_worse_than_the_fillet_alone`
compara o pior giro da normal com barra de razão, e estava **verde** enquanto a estrela mantinha
`11 %` da superfície sobre um vinco de `48°` — porque o pior giro não mudava de classe. O report do
dono (*«o fillet não pega todas as arestas»*) é sobre **quantos sítios**, não sobre o pior.

**How to apply:**
- Ao escrever um gate sobre um defeito de superfície, pergunte qual das duas o **report** descreve,
  e escreva a outra também se ela não existir.
- A régua da fracção compara-se com o **mesmo knob dos dois lados** e só a variável em estudo muda —
  comparar contra a peça **viva** mede a remoção, não a lei.
- A barra sai do **vale medido** entre a população que fecha e a que fica aberta, e a lista de
  excepções leva **censo de obsolescência**.

Irmãs: [[feedback_a_blend_never_rounds_an_edge_that_lives_inside_one_piece]] ·
[[feedback_a_ruler_that_counts_leftover_defect_rewards_overshooting]] ·
[[feedback_an_aggregate_that_already_measures_item_by_item_must_return_the_table]]
