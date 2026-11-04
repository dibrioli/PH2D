# 10 — Nova UI em todo o app: tracker da linha (reaberta em 04/11/2026)

Leitor: a LLM que continua a linha `line/NovaUI` (worktree `Worktrees/line-NovaUI`, base `main` @ `c12143dcdf`).
Ordem do dono (04/11): «reabra a linha NovaUI, continue a nova UI em todo o app, painel por painel, padrão-ouro
absoluto; sem perguntas, sem permissões, sem propor smoke; quando eu voltar dou o sinal para pausa e smoke».
Sem push, sem integração (CLAUDE.md §0.7). Lei visual: [05](05_ESPECIFICACAO_UI_COMPLETA.md) + [`maquete/`](maquete/).

## Censo de partida (medido no código, 04/11)

Em cartões: `inspector`, `skeleton`, `widget_gallery`. API reutilizável em `ph2d-editor-core/src/panel/cards*.rs`;
o modelo mais curto é `ph2d-panel-skeleton/src/cards.rs` + `paint.rs:98-108`. Restam 26 painéis registados.

## Como se trabalha aqui

- Uma árvore, um `target`; agentes em paralelo só em crates disjuntas. Commit por caminho (`git add -- <paths>`,
  `git commit --no-verify -m … -- <paths>`), sem `-A`, sem stash.
- `ph2d-editor-core` só em passos pequenos que compilam: o WIP dele para os outros agentes.
- `PH2D_CARDS=0` continua a dar o painel de antes, em todo painel convertido.
- Decisão de produto sem o dono: segue a recomendação registada no plano, e fica nesta lista para o smoke.

## Blocos, por ordem

Estado: ⬜ por fazer · 🟨 em curso · ✅ feito (foto vista, testes por nome verdes) · o smoke do dono é à parte.

| # | Bloco | Conteúdo | Estado |
|---|---|---|---|
| A | Dívidas do Inspector | Camera card ([05 §8](05_ESPECIFICACAO_UI_COMPLETA.md)), 2.ª metade do T29 (§9), popover no tom do cartão, caixas/listas do preso | ⬜ |
| B | Piloto + receita | Physics em cartões; [11_RECEITA](11_RECEITA_PAINEL_EM_CARTOES.md) nasce daqui | ⬜ |
| C | Alfinete | [Plano 09](09_alfinete_nos_cartoes_PLANO.md): Material, Ordering, Object → Appearance → Fill + Stroke | ⬜ |
| D | Painéis pequenos | Tokens, Grid Settings, Wet Paint, Macros, Tags, Padding, Upscale, Equalize Sizes, Background Removal, Color Equalization, Audio Mixer, Assets, Authored, Widget Lab | ⬜ |
| E | Ferramentas | Flip, Frames, Pixel Tools | ⬜ |
| F | Hierarchy | A árvore dentro da lista flutuante | ⬜ |
| G | Vector | 41 k linhas; respeita F43/F44 (layout Figma já decidido) | ⬜ |
| H | Painter | Respeita E10 (decisão do dono) | ⬜ |
| I | Desenho próprio | Timeline, Motion Graph, Audio Editor, Pixel Timeline, Pixel Preview, Code; o que neles é chrome, não o canvas | ⬜ |
| J | Resto de 05 §9 | Mola do Kanban (família 3), solto sob a barra de modos, corda sobre soltos de outro painel | ⬜ |

## Decisões tomadas sem o dono (para ele confirmar no smoke)

- (nenhuma ainda)

## Registo por bloco

(cada bloco fechado acrescenta aqui: commits, fotos, testes, o que ficou aberto)
