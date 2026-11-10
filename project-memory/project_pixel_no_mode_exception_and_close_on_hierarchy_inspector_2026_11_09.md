---
name: project_pixel_no_mode_exception_and_close_on_hierarchy_inspector_2026_11_09
description: "Dono 09/11 — o Pixel NÃO ganha a exceção do Vector (escolher/duplicar outro Pixel Sprite em Draw volta a Object); Hierarchy e Inspector ganham o «×» como os outros painéis"
metadata:
  type: project
---

Dono, 09/11/2026 (depois do smoke OK das pendências pequenas do P12):

1. *«Quer a mesma exceção para o Pixel? como vc sugere»* → **sem exceção**: escolher ou duplicar (`Ctrl+D`) outro
   Pixel Sprite em Draw volta a Object, a regra de 24/10 ([[project_modes_share_names_and_switching_object_leaves_2026_10_24]]).
   Só o Vector fica em Draw ([[project_vector_draw_stays_on_vector_selection_2026_11_02]]).
2. *«sim»* → **Hierarchy e Inspector ganham o «×»** como Physics, Pixel e as janelas acessórias; reabrem pelos
   chips do topo e pelo menu Window (o Inspector também volta ao selecionar, regra de 24/10).

**Why:** no Pixel não há operação sobre vários sprites (a exceção do Vector é pelas booleanas); o «×» igual em
todo painel que se fecha.
**How to apply:** não declarar `stays_on_its_kind` no Pixel; não tirar o «×» da Hierarchy/Inspector.

- Dono 09/11 («Decisão ok»): no visual antigo (`PH2D_CARDS=0`) a Hierarchy fica SEM ×, como está. Smoke do × OK (f7688dbaad).
