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
| B | Piloto + receita | Physics em cartões; [11_RECEITA](11_RECEITA_PAINEL_EM_CARTOES.md) nasce daqui | ✅ |
| C | Alfinete | [Plano 09](09_alfinete_nos_cartoes_PLANO.md): Material, Ordering, Object → Appearance → Fill + Stroke | ⬜ |
| D | Painéis pequenos | Tokens, Grid Settings, Wet Paint, Macros, Tags, Padding, Upscale, Equalize Sizes, Background Removal, Color Equalization, Audio Mixer, Assets, Authored, Widget Lab | ⬜ |
| E | Ferramentas | Flip, Frames, Pixel Tools | ⬜ |
| F | Hierarchy | A árvore dentro da lista flutuante | ⬜ |
| G | Vector | 41 k linhas; respeita F43/F44 (layout Figma já decidido) | ⬜ |
| H | Painter | Respeita E10 (decisão do dono) | ⬜ |
| I | Desenho próprio | Timeline, Motion Graph, Audio Editor, Pixel Timeline, Pixel Preview, Code; o que neles é chrome, não o canvas | ⬜ |
| J | Resto de 05 §9 | Mola do Kanban (família 3), solto sob a barra de modos, corda sobre soltos de outro painel | ⬜ |

## Decisões tomadas sem o dono (para ele confirmar no smoke)

- Physics, as barras somem nos cartões: o número muda arrastando o ícone (Figma), a barra só existe com `PH2D_CARDS=0`.
- Physics, o cartão Debug nasce dobrado com o resumo «Bodies: n»: paga a altura de abertura, que em cartões subia 53 px.
- Lista de cartões com «×»: fica na ponta e o «…» encosta-se a ele; só nos painéis que se fecham por ele (Physics). O Bones segue sem «×».
- Facto só de leitura (Scale, Bodies) mostra o valor sem caixa: a caixa promete edição.
- Physics, glifo dos campos = a letra (regra T56), não ícone desenhado. Interaction e Joints sem marca de animação (ajustes de sessão, D17).

## Registo por bloco

(cada bloco fechado acrescenta aqui: commits, fotos, testes, o que ficou aberto)

**Bloco B (04/11).** Physics em cartões: `ph2d-panel-physics/src/cards.rs`, 9 cartões, mesmos ids, `populate.rs` e `event.rs`
intocados; `PH2D_CARDS=0` dá o corpo de antes (foto vista, `tests/seam.rs` 27/27 nesse caminho). Fundação: `Widget::Readout`;
`CardCtx::hint`, `check`, `toggle_triangle` (`panel/cards_linhas_extra.rs`); «×» opcional da lista
(`mark_card_list_close`); `CardCtx::choice` declara o grupo no censo; métricas `GRID_CELL`/`GRID_GAP`. Cura de família:
o Esc a meio do scrub agora avisa a barra ligada (`dispatch/card_field.rs`). Testes: `tests/it/os_cartoes.rs` (15, gesto real:
inventário de ids, scrub e Esc de cada número, escrever, cada opção/caixa/verbo/célula, arrasto na matriz, dobra, reordenar,
soltar e usar no canvas, rolar, menu do cabeçalho, marca de animação, tooltips, corpo antigo); `linhas_extra` (3). Contraprovas
vistas vermelhas: tirar o espelho do Esc, tirar o fecho da lista, tirar o grupo da escolha. Catracas medidas: comandos
`2 → 3` (o «…»), abertura `1317 → 1236`; `a_marca_tem_a_altura_da_linha` aprendeu a altura do cartão. Fotos:
`maquete/b_physics_abertura.png`, `b_physics_todos_abertos.png` (estados por patch temporário: Pull, IK, células desligadas,
verbo da fita), `b_physics_debug_dobrado_hud_por_cima.png`. Commits `2baa0f6bff`, `c4454ae705`, `03b4f5b3be`, `0eb72eac36` e o
da receita. **Aberto:** o HUD «EDIT · fps» passa por cima da lista numa janela de 1200 px (visto no Physics, a 1930 não);
o alfinete não se oferece no Physics (o mundo não tem sujeito); clippy fica para o fecho da linha.
