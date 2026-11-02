---
name: project_image_use_selector_under_image_2026_10_26
description: "Dono 26/10 aprovou — logo abaixo de Image, seletor «Uso» Parada|Repetida|Animada (não checkbox); só os campos do uso; Animada abre a secção Animation ali; resto em «Avançado» fechado"
metadata:
  node_type: memory
  type: project
  originSessionId: 313ab8d4-937f-477c-9ee6-4353d54a60f7
  modified: 2026-10-27T01:17:41.041Z
---

Dono 26/10 (GameEngineFuncional, depois do smoke da onda 14): «tudo à vista logo de cara… intuitivo e fácil de usar».
Aprovado («OK. vamos fazer»): por baixo da ranhura Image, uma linha **Uso** com três botões exclusivos
**Parada | Repetida | Animada**; por baixo só os campos do uso (Parada: miniatura da folha com grelha, clicar o
quadro · Repetida: Width/Height · Animada: secção Animation ali mesmo — folha com grelha + Colunas×Linhas, lista de
animações com ▶, regras «quando → animação»). Filtro/Sampling/UV/Atlas descem para «Avançado» fechado. Animada numa
folha sem corte propõe a grelha (calhas transparentes) e cria «loop».

**Why:** o uso estava espalhado por Texture & Atlas, Sampling e Animation; o corte eram números sem ver a folha. Um
seletor de três torna o conflito «animada num bloco» irrepresentável — o aviso e o «Make it Animated» saem.

**How to apply:** plano `docs/GameEngineFuncional/15_uso_da_imagem_a_vista_plano.md` (linha GameEngineFuncional).
Nunca voltar a um checkbox «Animada» ao lado de «Tiled». Relacionadas: [[project_tiled_blocks_and_canvas_drop_always_creates_2026_10_26]],
[[project_animation_one_card_owner_decision_2026_10_25]], [[project_isolation_by_use_and_atlas_strategy_stays_on_object_2026_10_26]].
