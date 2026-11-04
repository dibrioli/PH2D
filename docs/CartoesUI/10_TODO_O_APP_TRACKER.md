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
| A | Dívidas do Inspector | Camera card ([05 §8](05_ESPECIFICACAO_UI_COMPLETA.md)), 2.ª metade do T29 (§9), popover no tom do cartão, caixas/listas do preso | ✅ |
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
- Camera/par de caixas (bloco A): a marca de animação da caixa da esquerda fica colada ao nome dela; no fim da metade ela encostava na caixa da direita e lia-se como sendo dela. A marca da direita segue no fim da linha.
- Camera, Game Format sem marca de animação no cartão: é definição do projecto, não se anima (D17). Frame, Outside, Editor View e Preview ficaram com a marca de hoje, a rever no smoke.
- Subsecção dentro de um cartão (Cull Mask, Visibility Layer) tem a cara do grupo (Camera Follow), nunca um 2.º título; a fronteira de secção lá dentro não pinta chapa.
- Um booleano de script no cartão é a linha de marcar com o nome à direita; o «Reset» desce para baixo dela.
- Lista suspensa aberta de um chip de cartão: a cara da lista T16 (sombra da lista, sem moldura, nome à esquerda, realce no tom do hover), tema próprio incluído.

## Registo por bloco

(cada bloco fechado acrescenta aqui: commits, fotos, testes, o que ficou aberto)

**Bloco A (04/11).** Mecanismo do Camera (a hipótese de 05 §8 estava errada): `close_section` dentro do corpo fechava uma chapa
`Bg1` do topo do cabeçalho até à fronteira do Cull Mask, pintada DEPOIS do título, que cobria título e caixas (o mesmo em
Anchors, com ficha aberta). Cura em `section_cards::close_with`. Os outros dois: `cabe` do par não contava a coluna de animação
(Look Through elidido) e a marca ficava a meio da linha. T29, recontagem com critério: `Checkbox::new` nas secções = 6 sítios, 4
passam por `rows::caixa_row`, 2 (Anchors, script) têm ramo do cartão antes; `property_label_row`/`paint_label_row` = 11 + 14
chamadores directos que a conta antiga não via: a porta sabe o cartão e todos fecham por `PropertyRow::fim` (fora do cartão, a
mesma conta dá `y + passo`); `paint_text` directos = 81 (72 + 9 por variável local), todos por `resolve`, que nas secções passou a
ser `card_ink::tinta` (censo `a_section_paints_through_the_card_ink`). `number_input`, `icon_button`, `slider_with_chip`,
`bitmask_grid32`, listras e realces de lista pedem a tinta; `paint_text_row` é a do cartão. Popover: o chip aberto lembra a tinta
(`lembra_popover`) e `paint_list` pinta dentro dela. Preso: as caixas já editavam o objecto certo; as LISTAS não: os popovers
pintavam com o retrato da seleção e a opção fora do rect do solto caía na seleção. Cura: `toma_sujeito_da_lista` e a guarda em
`note_card_press`. Cenas novas `PH2D_VEC_CARTOES_SMOKE=8` (Camera aberto) e `9` (lista aberta). Commits `76b2cae0ba`, `ebad56ecd1`,
`847fb583ee`, `32f3ee4b91`, `ac821bee14`. Fotos: `maquete/a_camera_antes_depois.png`, `a_lista_do_formato_no_cartao.png`; vistas e
sem arquivo: cena 6 (Player igual ao aprovado) e `PH2D_CARDS=0` na cena 8 (painel de antes). Testes por nome:
`tests/it/o_bloco_a_camera_e_presas.rs` (5), `a_section_boundary_inside_a_card_body_paints_no_plate`,
`a_pair_of_checks_never_cuts_a_name_at_any_width`, `a_chip_list_in_a_card_takes_the_card_ink`; verdes também
`every_form_row_reserves`, `nenhuma_linha_pinta_o_nome`, `no_row_paints_its_name_above`, `os_corpos_dos_componentes`, `o_bloco9`,
`o_alfinete_do_inspector`, `every_icon_in_a_card`. Mutação: 12/12 vermelhas com a cura desligada. O T56 passou a rolar: o par que
deixou de caber empilha, o cartão cresceu e o Damping saía da janela de 1800 px do teste.
ABERTO: as secções só de sprite (slice, sampling, visibility, material, render source) foram convertidas, mas hoje não se pintam
em cartões (com sprite o Inspector é o de antes): sem foto. Só Camera, Player e a lista foram fotografados; Anchors, Script,
Joint, Wheel, Constraints e o solto preso têm teste ou nada, não foto, e o tema próprio na lista só o teste viu. `paint_color_row`
(só `color_tint`, de sprite) e a amostra de cor do script seguem na forma de hoje. O corpo de letra de `paint_text`/avisos segue o
`Sm`: só a cor mudou. Dobra partilhada (05 §9) fora deste bloco. Sem clippy (fecho da linha) nem suíte.

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
