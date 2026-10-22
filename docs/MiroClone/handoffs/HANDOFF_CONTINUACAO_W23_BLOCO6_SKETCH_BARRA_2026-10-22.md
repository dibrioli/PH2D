# CONTINUAÇÃO — `line/MiroClone` W23: bloco 6, Rascunho/Final na barra da FORMA — 2026-10-22

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_BLOCO6_SKETCH](HANDOFF_CONTINUACAO_W23_BLOCO6_SKETCH_2026-10-21.md)
> · lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) · mecanismo, oráculo, medidas e
> recusas: [17_bloco6_plano.md](../17_bloco6_plano.md) §9 «Rascunho/Final na barra da FORMA (22/10, decisão do dono — o R1)».
> Base = main `100d34c8e0`. Sem integração nem push: só por ordem do Enio.

## 1. Feito

Decisões do dono 22/10: (a) **SIM** — Rascunho/Final também na barra da FORMA seleccionada (a «Sloppiness» do Excalidraw);
(b) o `Ctrl+Z` com a caneta pousada **fica como está** — fechado. Isto fecha o R1 e o ABERTO 2 e 3 do handoff anterior.

- `23566667e2` **feat** — `ph2d-board-edit`: `Editor::set_sketch` e `Editor::sketch_look` (em `look.rs`); `toggle_sketch` =
  `set_sketch(!sketch_lit)`; `sketch_lit` = `sketch_look == Some(true)` (uma porta). `ph2d-editor-core` (`board_bar*.rs`,
  `pen.rs`): `Item::Look(false|true)` ACRESCENTADO AO FIM de `items()` (nenhum id se move), posto depois de `Dash` nos grupos
  Shapes/Arrows/Both/Images e em `pen::ink_groups(looks)`; só aparece quando `sketch_targets` alcança a selecção (não o
  quadro inteiro); luz = `look_on` = `sketch_look == Some(look)`; clique em `pen::apply` → `set_sketch`.
  `float_bar::strip_rects_fitting` + `board_bar_layout::style_rects_of`: a barra de estilo fica à direita da barra de
  ferramentas e quebra em linhas equilibradas quando não cabe. Cena 28 abre com a caixa da ferramenta seleccionada; lição ②
  em `ph2d-i18n/src/board_block6.rs`; chaves novas `board.style.final` / `board.style.sketch` (`ph2d-i18n/src/board.rs`);
  só em teste, `thread_local board_bar::PAINTED_ON` regista a luz com que cada botão foi PINTADO. Sonda do oráculo:
  `docs/MiroClone/ferramentas/excalidraw_oracle/sonda_w23_sloppiness.mjs` + `saidas/w23_sloppiness/`.
- `946aa7ede8` **test+docs** — o gate «a barra cabe» a 800/700 (ver §3), doc 17 §9, fotos
  `docs/MiroClone/ferramentas/w23_fotos/cena28_1366.png` / `cena28_1930.png` com o exemplo seleccionado.
- `efc211c86a` **refactor** — os dois vermelhos que a suíte desta onda apanhou, vindos da onda ANTERIOR: tecto de LOC
  (`ph2d-board-edit/src/look.rs` recebe as portas do sketch e `sketchable`, `lib.rs` 743→644; `ph2d-board-render/src/sketch/ink.rs`
  recebe tinta da caneta e laser, `sketch.rs` 770→586) e `→` → `->` em `board_smart_hand_tests.rs:151` (`no_tofu_glyphs`).

**Oráculo (Excalidraw 0.18.1):** Sloppiness para rectangle/ellipse/diamond/arrow/line/embeddable; não para text/freedraw/image/
frame; depois de «Stroke style»; roughness misto ⇒ nenhuma luz; clique SETA na selecção inteira; clicar a opção acesa = sem mudança.
**Divergências deliberadas:** imagens mostram a opção (moldura/contorno é `sketchable` desde a W15); vídeos nunca (não
`sketchable`); notas não (o texto do Excalidraw não tem); só 2 estados (sem Cartoonist).

## 2. O que um merge pode partir

- **Ordem de `board_bar::items()`:** o par `Look` fica no FIM; outra linha que acrescente `Item` ao fim conflita/reordena.
  `images_w16::the_w16_items_come_last_and_move_no_id` foi ajustado (o par `Look` é o último).
- **Ficheiros novos:** `ph2d-board-edit/src/look.rs`, `ph2d-board-render/src/sketch/ink.rs`. Quem tocar nos mesmos blocos de
  `lib.rs` / `sketch.rs` na `main` reaplica nos novos sítios.
- `float_bar::strip_rects_fitting` (pub fn nova) e `board_bar_layout::style_rects_of`: **a barra de estilo já não cobre a barra de
  ferramentas** ⇒ posições mudam junto ao bordo esquerdo e em janelas estreitas; gates de posição de outras ondas reconferem.
- Chaves i18n `board.style.final` / `board.style.sketch`; texto da lição ② da cena 28.
- **Formato e protocolo: inalterados nesta onda** (continuam 17 / 11 do handoff anterior). `PROJECT_SCHEMA` e §6: zero. Contadores
  como DELTA desta onda: zero em formato/protocolo/`Field::ALL`.

## 3. Prova de fecho

Gates em `ph2d-editor-core`, módulo `board_bar::tests::smart_hand::sketch_ink` (`board_sketch_ink_tests.rs`); vermelho visto
primeiro: «Look(true) não está pintado».

- `the_shapes_own_bar_sketches_it_and_its_light_is_the_waves` (zoom 0,293 e 1; tinta +2,5 px; Final devolve < 0,25 px; modo do
  quadro intocado)
- `a_mixed_selection_lights_neither_and_final_sets_both` (um Ctrl+Z repõe a mistura)
- `the_option_shows_only_where_the_wave_reaches_the_selection` (forma da caneta e seta sim; tinta livre e nota não)
- `the_shapes_bar_never_covers_the_toolbar_nor_leaves_the_board` (1366, 1024, 800, 700)
- `ph2d-app-board` `smoke_w23_block6_tests::the_block6_scene_arms_its_four_stations` (a opção pintada na barra do exemplo)

**Mutações: 7/7 sangram** — luz por `sketch_lit` · clique por `toggle_sketch` · filtro de alcance removido · `strip_rects_at` em
vez de `_fitting` (**SOBREVIVEU** a 1366/1024 até o gate ganhar 800/700; sangrou a 800) · sala = área · `sketch_look` `None` em
vez de `Some(false)` · cena sem a selecção.

**Fecho da onda:** `cargo fmt --all`; clippy `-D warnings --all-targets` limpo em `ph2d-board-edit`, `ph2d-board-render`,
`ph2d-editor-core`, `ph2d-app-board`, `ph2d-i18n`. `nextest-impacted` (`PH2D_SUITE=onda`): **19 793 testes, 19 791 passam, 2
falham** = `workspace_src_files_under_loc_cap` e `no_tofu_glyphs_in_ui_strings`, ambos curados em `efc211c86a` e re-corridos
por nome, verdes; `ph2d-board-render` (260 / 60 / 11 `it`) e `ph2d-board-edit` completos verdes. Os flakes de relógio do
`ph2d-tool-painter` do handoff anterior NÃO apareceram. `load` durante a suíte ~20–45. Binário de smoke compilado
(`--profile smoke`). ⚠️ NÃO corridos: clippy do workspace inteiro e a bateria (só no `/pd-linha-fechar`).

## 4. ABERTO (nesta ordem)

1. **Smoke do dono da cena 28** (§5) — se chegar, trate-o primeiro (mecanismo, não sintoma).
2. Menor: com o exemplo seleccionado à abertura, os pontos de ligação / alça de rotação ficam por cima da 1.ª linha da lição ②
   a 1930 (o texto continua legível) — ver `cena28_1930.png`.
3. Elementos antigos (`birth` 0) desenhados longe antes da escala do ecrã ficam em escala de mundo.
4. O tamanho da LETRA não nasce à escala do ecrã (rótulos de formas nascidas longe saem pequenos) — não pedido.
5. README do oráculo: o W14 nomeia um teste apagado.
6. Pendentes do handoff W23 anterior (§4.2 da lista-mãe): `textAlignVertical` (perguntar), YouTube Premium, «Rascunho/Final»
   troca a letra importada?, os nomeados do bloco 5.
7. Fecho da linha (`/pd-linha-fechar`): clippy do workspace e a bateria.

## 5. Smoke ao dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build --profile smoke -p ph2d-host-desktop && PH2D_BOARD_SMOKE=28 ./target/smoke/ph2d-host-desktop`
2. O quadro abre com a caixa da esquerda, no canto de baixo do frame «2 · Draw here», já selecionada. Por cima dela aparece a
   barra da caixa; nela há duas opções lado a lado: uma linha reta (Final) e uma linha ondulada (Rascunho). A ondulada está
   acesa, tal como o botão ondulado da barra da esquerda.
3. Clique na linha reta: a caixa fica com linhas retas; acende a reta, apagam-se a ondulada e o botão da esquerda. Clique na
   ondulada: volta à mão e as luzes voltam. Clicar outra vez na que já está acesa não muda nada.
4. Selecione as duas caixas (arraste à volta delas) depois de pôr só uma em Final: nenhuma das duas opções fica acesa. Clique
   Final: as duas ficam retas. Ctrl+Z: volta a mistura.
5. Desenhe uma linha livre com a caneta e selecione-a: a barra dela não mostra as duas opções (o botão da esquerda continua a
   funcionar como antes). Uma nota também não as mostra.
6. Deu errado se: clicar uma opção não muda a caixa à vista; a luz da opção não bate com o botão da esquerda; a barra da caixa
   passa por cima da barra de ferramentas ou sai da janela.
