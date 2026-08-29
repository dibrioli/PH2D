---
name: feedback_a_knob_whose_range_is_derived_from_the_object_it_rewrites_is_not_idempotent
description: "Um slider cuja faixa sai de uma medida da malha que o botão vai SUBSTITUIR muda de significado a cada aperto — ancore a faixa numa grandeza da SUPERFÍCIE (área), não da tesselação"
metadata:
  type: feedback
---

Medido 2026-08-28: o `Detail` da retopologia tirava o alvo de
`0,75 × aresta_média(malha_da_cena)`. Isso é honesto na 1.ª passagem (*não se resolve mais
fino do que a entrada tesselou*) e é uma armadilha na 2.ª — **depois de uma retopologia a
malha da cena É a saída**, então o piso sobe com ela. Com o slider **parado** em `0,50`:
`19 786 → 1 747 → 520 → 281` quads em três apertos, **−98,6 %**. O artista fotografou
*«pontas com baixa resolução»* e chamou-lhe piora severa.

⭐ A cura é **ancorar a faixa numa grandeza da SUPERFÍCIE, não da tesselação**: a **área** não
muda quando a malha é substituída, então `lado = √(área/contagem)` pede a mesma coisa em todo
aperto (`1 377 → 1 413 → 1 494`, e a deriva que fica **é** a área a crescer com o alisamento —
o que dá a barra do gate: `|√(área₁/área₀) − 1|`, derivada em vez de escolhida). ⭐ É também o
que as três referências fazem — ZRemesher *Target Polygons Count*, QuadriFlow *Number of
Faces*, Instant Meshes alvo de vértices: **nenhuma** deriva a faixa da malha que tem na mão.

**Why:** um botão que reescreve o objecto de que o seu próprio knob se mede compõe consigo
mesmo. O primeiro aperto parece certo, e é o **segundo** que revela — por isso nenhum gate de
saída via nada: cada corrida, isolada, estava correcta.

**How to apply:** sempre que um knob for consumido como *fracção de uma medida do objecto*,
pergunte **o que acontece ao apertar duas vezes**. Se a operação substitui o objecto medido,
a faixa tem de sair de um invariante da coisa (área, volume, caixa) e não da representação
dela. E ponha o **CONTROLE no mesmo gate** — a lei antiga a mover-se — senão a asserção nova
passa por vácuo. Relacionado:
[[feedback_a_knob_consumed_as_a_per_step_rate_is_a_target_not_a_rate]] ·
[[feedback_a_new_features_gate_can_expose_a_pre_existing_bug_check_the_control_first]] ·
[[feedback_a_target_derived_from_the_box_makes_a_finer_input_pure_waste]]
