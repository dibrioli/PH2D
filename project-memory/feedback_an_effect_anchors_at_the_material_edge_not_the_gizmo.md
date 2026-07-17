---
name: feedback_an_effect_anchors_at_the_material_edge_not_the_gizmo
description: "Efeito ancorado no footprint geométrico, mas a matéria (tinta macia) termina antes — ancore na borda do CORPO"
metadata:
  node_type: memory
  type: feedback
---

O aro de tinta empurrada do Push (impasto) nascia em `t = 1` — a circunferência do gizmo — mas num
falloff macio a tinta termina em `t ≈ 0,61` (W_TAIL). Resultado: um **colar duro, perfeitamente
circular, com tela nua entre a tinta e o anel** (smoke do Enio, 2026-07-15: *"é usada a circunferência
do gizmo do brush para empurrar a massa e não o alpha do falloff"*). O gate era verde porque media
PARA ONDE o volume vai (frente/lado/trás), nunca ONDE a borda interna do aro nasce.

**Why:** um efeito que DESLOCA/BANCA matéria macia tem que nascer onde a MATÉRIA termina, não onde a
geometria do pincel termina — são coisas diferentes num falloff macio (o mesmo erro do skirt de pigmento
que o filme já matava do outro lado). É o gêmeo de [[feedback_a_threshold_must_live_where_the_domain_is_empty]]
e de [[feedback_geometry_over_mixed_units_needs_the_consumers_conversion]]: a borda relevante é a do
DADO (cobertura ≥ W_TAIL), não a do índice geométrico. E, como [[feedback_two_doors_to_the_same_question_diverge]],
a borda tem que sair da MESMA função que define "onde a tinta termina" (o filme), senão os dois lados
divergem.

**How to apply:** ao ancorar um efeito radial (aro, sombra, halo, banco de massa), pergunte a `t0` à lei
que já define a borda do material (`height_film::body_edge_t` = onde a silhueta cruza W_TAIL), por
**porta única** compartilhada por todos os sítios (`rim_t0`→`rim_lift`, pros 2 kernels e 2 chamadores).
Borda DURA (Constant/hardness≥1/Shape image) ⇒ `t0 = 1` exato (fast-path + fingerprint = byte-idêntico).
Gate: mede a **borda interna** (1º texel do efeito, banco em ISOLAMENTO pra tirar ruído de vizinhos), e
refere o limiar ao RAIO (fato fixo), não a `t0` (que a mutação move junto — vira autorreferente). Ver
[[reference-topic-impasto-physics]].
