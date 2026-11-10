# Handoff de continuação — `line/ScriptEditor`, acabamento 10/11 (A, B, C)

> Antes: [Q3+Q4](HANDOFF_line_ScriptEditor_2026-11-09_Q3_Q4.md) · maquete
> [14_quadros_do_depurador.html](../maquetes/14_quadros_do_depurador.html). Base `main` `bc4a6ce930`; cinco commits
> `12f2ea316e`, `a3317290c2`, `df71f85c63`, `e4416a14d8`, `a3ebed165a`. **Não integrado, sem push.** Smoke do dono desta
> onda **pendente**. Escolhas de acabamento decididas sem perguntar (estado da arte:
> `project-memory/feedback_finish_choices_go_to_the_state_of_the_art.md`).

## A — Script options

- **Disco da pílula** = `ph2d_tokens::switch_knob` (`state_layer.rs`): a mais clara de Text1/BgElev, piso OKLCH L 0,93
  (`KNOB_L`), croma ≤ 0,02. Medido: o Candy tem luminância 0,465. Vale nos DOIS estados: o desligado pintava Text1, quase
  preto no tema Light (achado na foto `ac_options_claro`, corrigido em `e4416a14d8`). `switch_inks`
  (`ph2d-editor-core/src/widget/toggle_classic.rs`) é puro.
- **Hover** = Text1 a `HOVER_LAYER_ALPHA` 0,08 por cima da linha (state layer do Material 3): clareia no escuro, escurece
  no claro; antes era Bg2. Pintura em `ph2d-panel-code/src/options_card.rs`.
- Gate novo: `ph2d-tokens` `the_switch_knob_is_light_in_every_theme`; testes em `ph2d-panel-code` `opcoes_do_script`.

## B — Tabelas da casa

- `ph2d-ui-table`: `Metrics::twist` (reserva do ▸ só quando alguma linha é expansível); `rules_bottom` (fios até à
  última linha INTEIRA — o corpo só mostra linhas inteiras). Testes: `tests/it/reserva_e_fios.rs`.
- Grelha, balão e Problems pintam por `CodeCells` (mono do `CodeView`); `paint_bench`/`paint_popover`/`ConsoleView::paint`
  recebem `&mut CodeView`; `ph2d-app-code` passa `doc.view` (`doc.rs`, `surface.rs`).
- Teste `o_passo_1_da_cena_13_abre_o_balao_de_speed_com_os_dois_herois` (`ph2d-app-code` `onda_b`): conduz `STEPS_13[0]`
  pelo `Conductor` sobre `bancada_player.luau`. A app-code não depende de `ph2d-code-scenes`: o teste monta Hero/Hero 2 com
  `speed` 180 como a cena.

## C — Restos da Onda B

- `parse_vec2` (`ph2d-code-bancada/src/act.rs`): aceita «x; y» (vírgula decimal), «x y»/«x, y», «x,y», `( )`/`[ ]`;
  «1,5,2» é ambíguo e recusa. Testes em `tests/it/acto.rs`.
- Hook `.claude/hooks/prazo-do-agente.sh`: a palavra «bancada» só é recusada com comando pesado + `--ignored` /
  `--include-ignored` (essas provas são `#[ignore = "bancada…"]`); `PH2D_REGERA_` sempre. `scripts/processo.prova.d/I.sh`
  +6 casos; `processo.prova.sh` tudo verde.
- ⚠️ **O hook que governa os subagentes é o do checkout PRIMÁRIO**: a cura só vale depois da integração (medido: o agente
  de mutação foi recusado 3× com a regra velha).
- `FOTO_TECLAS`: `docs/CodeEditor/ferramentas/smoke/cena14/fotos_q3.sh` ganhou `TECLAS=` (XTEST na Xvfb, foco por
  `windowfocus`) e `LOG=`; prova: `CENA=13 PH2D_UNDO_LOG=1 TECLAS=ctrl+z`, passo 4 → «Ctrl+Z RECEBIDA pela janela».
  `docs/Components/ferramentas/fotografa_cena.sh` com `FOTO_TECLAS` recusa alto (exit 3) e aponta para `fotos_q3.sh`.

## Superfície de colisão (delta contra `bc4a6ce930`)

Crates: `ph2d-tokens`, `ph2d-editor-core` (`widget/mod.rs`, `toggle_classic.rs`), `ph2d-ui-table` (`lib.rs`, `paint.rs`),
`ph2d-panel-code` (`options_card.rs`, `bancada/{grid,popover}.rs`, `console.rs`, `console/problems.rs`), `ph2d-app-code`
(`doc.rs`, `surface.rs`), `ph2d-code-bancada` (`act.rs`, `lib.rs`). Fora de crates: `.claude/hooks/prazo-do-agente.sh`,
`scripts/processo.prova.d/I.sh`, `fotografa_cena.sh`, `fotos_q3.sh`. Contratos §6: nenhum tocado. `PROJECT_SCHEMA`: intacto.

**O que um merge pode partir:** `ui-table/src/lib.rs` está a 692/700 — outra linha que lhe acrescente estoura o tecto;
`paint_bench`/`paint_popover`/`ConsoleView::paint` mudaram de assinatura (`&mut CodeView`), qualquer chamador novo no
`main` não compila; o hook e o `I.sh` também são tocados por outras linhas (conferir no rebase).

## Provas (mutação, vermelho → verde)

18 mutações, todas vermelhas: `switch_knob`=AccentFg · sem piso `KNOB_L` · `HOVER` 0 · `switch_inks` AccentFg · desligado
Text1 · hover Bg2 · `twist` sempre chevron · `natural_widths` chevron · `rules_bottom` = fundo do corpo · `UiText` em
grid/popover/problems (3) · `parse_vec2` regra velha · sem ramo «;» · `STEPS_13` `Mark(0)`→`Holder(0)` · changed/default
trocados · hook regra velha (4 ✗ nos allow) · hook sem o ramo «bancada» (2 ✗ nos deny).
**Não testável por teste:** `paint.rs` `x += twist` → `m.chevron()` (a posição pintada não é sondada; a foto prova).

## Folgas medidas (após os commits)

- `ph2d-app-code` 26 579 / 26 628 (folga 49) · `ph2d-code-bancada` 969 / 1 028.
- **CHEIOS:** `code-smoke` 614/614 · `code-debugcard` 663/663 · `code-viewmodel` 304/304. `code-scenes` 665/666 · `code-assist`
  3 698 / 3 925.
- `ph2d-ui-table/src/lib.rs` 692/700, `paint.rs` 447 · `panel-code`: `grid.rs` 383, `popover.rs` 227, `problems.rs` 337,
  `options_card.rs` 356 (cap 600 por ficheiro).

## Fotos (`docs/CodeEditor/ferramentas/smoke/`)

`ac_options.png` · `ac_options_hover.png` · `ac_options_claro.png` (`PH2D_THEME=light`) · `ac_problems_hover.png` (o
Problems sai no mesmo quadro da cena 11 passo 2: ver `ac_options.png`) · `cena13/ac_passo_1..3.png` ·
`ac_lado_a_lado.png`. Medido na foto: faixa da frase do pé 3 px mais baixa (y 705→708); «In this script» com folga 6→4 px.

## Smoke do dono (pendente)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && bash scripts/ph2d-run.sh cargo run --profile smoke -p ph2d-host-desktop` com `PH2D_CODE_SMOKE=11` (passo 2: Script options e Problems) e depois `=13` (balão).
2. Abrir «Script options»: os discos das três pílulas são claros, ligadas e desligadas; passar o rato numa linha clareia o cartão.
3. Repetir com `PH2D_THEME=light`: o disco continua claro e o hover escurece.
4. Deu errado se: disco quase preto no tema claro, ▸ reservado numa tabela sem árvore, ou fios que passam da última linha.

## Desvios ainda visíveis contra a maquete 14 (não mexidos: densidade da casa)

Peso da letra (app semibold, maquete regular) · altura de linha (app ~22 px, maquete 30–45) · balão mais compacto e com
«Select Hero 2» · Problems: coluna do ponto ~45 px com o ponto à esquerda (maquete centrado) · grelha sem rótulo na coluna
do nome.

## ABERTO

- Smoke do dono desta onda.
- Não apagar `backup/line-ScriptEditor-2026-11-08` sem o sim do dono.
- Fecho da linha (`/pd-linha-fechar`) só depois do smoke aprovado; integração e troca do link no CLAUDE.md §5.1 são do integrador.

## D — Molduras de valor (smoke do dono 10/11, depois deste handoff)

- Causa: a integração `411a89fa5c` passou o número ao vivo no código, a caixa de campo (Cards), a dica
  do traço e o gráfico por `stroke_frame(.., Feel::Rest, ..)`; num tema moderno a pele plana apaga a
  moldura de repouso. Não são molduras de controlo.
- Cura `74d6a5f77b` + `e85f3b0bd5`: `cards::value_outline` (traço sempre, `FRAME-RAW-OK`). Os botões da
  cabeça do quadro ficam pela porta (são controlos).
- Testes: `a_moldura_do_numero_ao_vivo_pinta_nos_temas_modernos_e_nos_classicos` (mutação: voltar a
  `stroke_frame` ⇒ vermelho «Dark: a moldura do valor sumiu») e `o_contorno_de_valor_e_um_traco_em_todos_os_temas`.
- Fotos: `smoke/ac_molduras_valores_codigo.png` (cena 12 passo 3), `smoke/ac_molduras_valores_cartoes.png` (cena 14 passo 9).
