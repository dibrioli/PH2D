---
name: project_game_smoke_composes_pieces_never_ready_made_2026_11_02
description: "Dono 02/11 (J5 moedas): o jogo de plataforma serve para REVISAR e MELHORAR a engine — monta-se com as peças, nunca com objectos prontos; sons da Kenney"
metadata:
  node_type: memory
  type: project
  originSessionId: 5d5174d4-74f5-4c3b-9e86-f19ed63b6052
  modified: 2026-11-02T21:07:44.460Z
---

Dono, 02/11, ao planear J5 (moedas): a moeda é **montada à mão com as peças** (área/sensor + tabela
de acções + som + contador + HUD), **NÃO** um objecto pronto «Coin» no menu Add. Palavras dele:
*«o objetivo é revisar e melhorar a engine testando em um game real. Não é criar coisas prontas.»*
Sons: **a princípio os pacotes da Kenney** (CC0), como a arte ([[project_platformer_art_kenney_flat_vector_2026_10_25]]).

**Why:** o jogo é o smoke da engine ([[project_game_engine_plan_owner_decisions_2026_10_25]]); um
objecto pronto esconderia as costuras que o jogo devia expor.

**How to apply:** em J5–J10, quando falta comportamento, conserte/complete a PEÇA genérica que a
composição usa (sinal, verbo, componente, Inspector) e monte o item do jogo com ela; não proponha
objecto-preset para o menu Add salvo pedido dele. Defeito achado ao montar = trabalho da onda.
