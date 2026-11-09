---
name: project_same_as_paper_without_paper_has_no_grain_2026_11_08
description: Dono 08/11 — Grain «Same as Paper» com Paper: None não tem grão (revoga o papel de reserva de 28/10)
metadata:
  type: project
---

Dono 08/11: *«Grain: Same as paper não pode ser efeito de paper está com Paper: none»*. O traço usava o papel de
reserva (`default_paper`, Cold) sem o artista ter escolhido papel — a tela não mostrava papel e o grão mostrava.
Cura em `dab_spec.rs::poe_o_papel_como_grain` (commit 13b910f87e); o teste de 28/10 que exigia o papel de fábrica
foi substituído por `same_as_paper_sem_papel_escolhido_nao_tem_grao`.

**Why:** o grão «igual ao papel» tem de ser igual ao papel QUE SE VÊ; sem papel, nada.
**How to apply:** não reintroduzir o papel de reserva no grão. O filme do Óleo Real (`grao_no_filme`) ainda usa o de
reserva — se o dono pedir o mesmo lá, é a mesma porta (`papel_do_documento`).
