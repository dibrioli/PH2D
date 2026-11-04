# HANDOFF de integração — `line/GameEngineFuncional`, 04/11 (GEF 22)

Smoke do dono: **OK** (04/11, «melhor embora não perfeito» sobre a secção Animation; o atrito aprovado). Ordem de
integrar: dono, 04/11. Plano e medições: [`22_animacao_revisao_plano.md`](../22_animacao_revisao_plano.md).

## O que entra

1. **Secção Animation revista contra a Godot 4.7.2** (oráculo: `ferramentas/godot_animacao.gd`): Direction e overrides
   só aparecem com valor gravado (o dado e `ph2d.set` não mudaram); «+ Add · Remove · Edit Animations…» colados à lista;
   «Selected Animation»; From/To em Advanced; nomes por extenso; regras em uma ou duas fileiras
   (`dropdown_label_budget`); frases viraram balões. Inspector ao abrir, árvore combinada: `1 507 → 1 375 px`.
2. **O Player nasce sem atrito contra paredes** (`object_add.rs`, entrada CHARACTER): `Collider.friction = 0` +
   `MaterialCombine.friction = Min`. Só novos Players; projetos salvos mantêm o valor (o dono ajusta em Friction e
   Friction Combine).

Medido em `platform_wall_sem_atrito` (30 tiques empurrando contra a parede): padrão `0,056 m` · atrito 0 com Average
`0,056` · atrito 0 + Min `0,865` · queda solta `0,865`. Andar 45 tiques no chão: `4,330 m` nos dois.

## Provas

Gates novos: os seis de `seam_anim_card`/`seam_anim` do plano 22 · `sem_atrito_o_personagem_desce_a_parede_como_quem_cai_solto`
(com a linha de controle e a de Average) · `o_personagem_nasce_sem_atrito`. Mutação com base verde: 4/4 overrides, 8/8 do
lote da secção. ⚠️ O atrito não levou mutação à parte: o teste carrega os próprios controles.

## ⚠️ Para o próximo

- **O histórico antigo da linha já estava no main:** `git cherry` deu `+` em dois commits só porque o integrador os
  emendou (`b13f56c6ab`, `8b9cba1cda`). O rebase foi `--onto main 98eeb1fecc`, levando só os commits novos. Rebase simples
  conflitaria à toa.
- **Colisão só na catraca de altura do Inspector:** o número certo não estava em nenhum lado (`1 362` × `1 507`); foi
  remedido na árvore combinada.
- **A vista nova foi aprovada na árvore antiga:** o smoke do dono foi anterior ao rebase. Os arquivos da secção Animation
  são iguais no main e na base da linha, mas a fonte Noto do main muda quebras de texto.
- **`cargo fmt --all` no main mexe em dois arquivos alheios** (`ph2d-panel-pixel-preview/src/paint.rs`,
  `ph2d-vec-entities/src/connector_handles.rs`): desfeito aqui, não é desta linha.

## Aberto

Placar «Coins: N» (e o nome «Path 0» × «Text»), som em gaveta/remoção, inimigos, conta-gotas/perf. O Inspector da Animation
não foi alinhado ao visual novo, por decisão do dono: é de outra linha. Balões e cliques: vistos pelo dono no smoke, não em foto.
