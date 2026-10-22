# CONTINUAÇÃO — `line/MiroClone` W23: bloco 6, o Sketch («apertei e nada mudou») — 2026-10-21

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_BLOCO6_ONDA2](HANDOFF_CONTINUACAO_W23_BLOCO6_ONDA2_2026-10-20.md)
> · lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) · mecanismo, medidas e
> recusas: [17_bloco6_plano.md](../17_bloco6_plano.md) §9 «Auditoria do Sketch (21/10)» (e as duas secções «Smoke 21/10: o Sketch»).
> Base = main `100d34c8e0`. Sem integração nem push: só por ordem do Enio.

## 1. Feito (um bloco; o mecanismo está no doc 17, não aqui)

Smoke do dono, cena 28: *«sketch não tem aspecto de rascunho»*, *«o retângulo nem sequer aceita a opção de rascunho»*
(apertou a ondinha com o rectângulo seleccionado e nada mudou). Auditoria por mecanismo, sete achados, uma cura:

| # | achado | cura |
|---|---|---|
| R1 | a barra de estilo da forma não tem Rascunho (`style_groups(Selected::Shapes)`, `board_bar.rs`) | **não mexido** (o caso do dono era o R2) — ABERTO 2 |
| R2 | a zoom ~0,293 (cena 28 a 1366) o rascunho e o traço 0,5 são de MUNDO: no ecrã traço 0,15 px, desvio 1,04 px | nasce no tamanho do ecrã (decisão do dono) |
| R3 | espigão ~35 px do canto: o traçador da GPU nas passagens finas (3/60 rects a 0,5) | linhas à mão expandidas na CPU e preenchidas |
| R4 | tinta limpa da caneta (triângulo, polilinha) nunca à mão | `Ink::clean`, `smart_style`, pintor `rough_clean` |
| R5 | botão morto com só tinta livre/frame/vídeo/bloqueados | `sketch_targets`/`sketch_lit`: uma porta |
| R6 | Ctrl+Z da ondinha do quadro não repunha `board.sketch` | `apply_mode`/`take_mode` |
| R7 | `ink_finish` (largar perdido) gravava um passo ⇒ Ctrl+Z apagava a forma | `keep_drawn` partilhado com `ink_up` (dois passos) |

Decisões do dono 21/10: (1) longe, o rascunho NASCE ao tamanho do ecrã e fica (aproximar mostra-o maior); (2) a forma da
caneta mantém a espessura da FERRAMENTA (fina), como `8ccb4efe3f`.

- `89f50a57c5` **o modelo** — `ph2d-board-model`: `Style::birth: BirthScale(i8)` (quartos de oitava, `2^(k/4)` mundo/px, clamp ±48;
  `Style::reborn/chosen_width/choose_width` em `element_birth.rs`); `Ink::clean`; `Field::Birth` (ALL 40 → 41),
  `Field::InkBody` leva `clean`; `History::apply_mode/take_mode`; formato 16 → 17 (`legacy_v16.rs`, leitores 9–16 trocados).
- `89f50a57c5` **o pintor** — `ph2d-board-render/src/sketch.rs`: rascunho gerado no espaço de NASCENÇA (`size / f`, `born()`),
  devolvido por `f`; `hand_reach × f`; formas, ligações e tinta limpa; linhas à mão (e hachura importada) por `outline_of` +
  `expand_stroke` (tol. largura/16) PREENCHIDAS e em cache com o rascunho; `rough_clean` (Douglas–Peucker a 0,5 px de nascença).
- `89f50a57c5` **o editor** — `ph2d-board-edit` (`smart.rs`, `ink.rs`): `smart_style`/tinta limpa com o estilo da ferramenta,
  `sketchable` inclui tinta limpa, borracha de precisão guarda `clean`, `keep_drawn`; `Editor::sketch_targets/sketch_lit`.
  `ph2d-editor-core`: `board_view.rs` renasce o estilo a cada `pointer_down` (`ed.style.reborn(BirthScale::of_px(p.px))`),
  `board_keys::step` (a porta ÚNICA do Ctrl+Z) e `documents::undo_on` repõem o modo; `board_bar*.rs`.
- `89f50a57c5` (sync) · `24ef558201` (i18n) · `e26d7a277d` (oráculo) **sync/i18n/oráculo** — protocolo vivo 10 → 11 (`ph2d-board-sync/src/session.rs`); `ph2d-i18n/src/board_block6.rs`;
  oráculo Excalidraw re-corrido (§3).
- `24ef558201` **cena 28** — abre em Rascunho, duas caixas de exemplo (ferramenta, caneta) no canto de baixo do frame ② nascidas
  a `BirthScale(7)` (= o zoom de abertura a 1366); lição ② reescrita (`smoke_w23_block6.rs` + `_tests.rs`). Fotos refeitas:
  `ferramentas/w23_fotos/cena28_1366.png`, `cena28_1930.png`.
- (este commit) **docs** — doc 17 §9 «Auditoria do Sketch» (+ 3 linhas nas recusas; o ⚠️ do espigão aponta para lá) e este handoff.

## 2. O que um merge pode partir

- **Formato do `Board` 16 → 17 e protocolo vivo 10 → 11** (DELTA +1 cada): quem também subir o formato do quadro ou o
  protocolo conta de novo — a escada reconta-se no código (`python3 scripts/schema-recount.py`), não se escolhe. `PROJECT_SCHEMA`
  e contratos §6: zero.
- **`legacy_v16.rs` + `fixtures/format_v16.bin`** (escrita pelo formato 16 REAL, em `b832018d9e`): novos; `legacy.rs` e os
  `legacy_v8/v10/v12/v13/v15.rs` tocados (os leitores 9–16 passam a ler `V16 Style/Shape/Ink/Frame/Image/Embed/Connector`) —
  outra linha que mexa nesses ficheiros conflita.
- `Style` ≤ 40 bytes (gate `the_style_stays_in_forty_bytes`): quem acrescentar campo ao `Style` fala com este gate.
- `Field::ALL` 40 → 41: tabelas que contam campos (menus, oráculo) reconferem.
- Isto cobre, no pintor, `sketch.rs` (rascunho/linhas/hachura) e, nos testes, `video_tests.rs`, `mind_sketch_tests.rs`,
  `sketch_tests.rs`, `tests/it/sketch_paint.rs`: o hachurado de vídeo e os ramos do mapa mental passaram a ser PREENCHIMENTO, não
  traço — quem asseverar `Stroke` nesses sítios reescreve.
- Ficheiros `.ph2dproj` abertos no formato 16 nascem a zoom 1, tinta toda livre (`a_format_16_file_opens_born_at_zoom_one_with_every_ink_free`).
- Oráculo `docs/MiroClone/ferramentas/excalidraw_oracle/{entradas,saidas}`: regenerado, nunca à mão (ver §3).

## 3. Prova de fecho da onda

**Gates** (cada um visto vermelho pela razão certa; agora verdes):

- `board_bar::tests::smart_hand::sketch_ink::*` (CPU; o caminho real do ecrã: tinta gravada → traços expandidos → varrimento de
  colunas em px de ecrã): `a_tool_shape_drawn_far_reads_as_sketched_as_one_drawn_near` (vermelho: perto 4,49 px, longe 1,51),
  `a_pen_shape_drawn_far_reads_as_sketched_as_one_drawn_near` (4,84 × 2,12), `the_sketch_button_visibly_changes_a_far_rectangle`
  (antes 0,15, depois 1,51; limiar +2,5), `the_pens_clean_triangle_is_sketched_in_a_sketch_board_and_the_button_reaches_it`
  (1,08 px), `the_sketch_button_with_only_freehand_ink_selected_toggles_the_board`,
  `undo_of_the_board_sketch_toggle_gives_the_mode_back`.
- `ph2d-board-edit` `smart_tests::a_lost_release_with_the_shape_shown_still_gives_the_stroke_back`.
- GPU `sketch_paint::no_hand_stroke_paints_far_outside_its_box` (60 sementes × larguras 0,25/0,5/1/1,68/2; vermelho: 0,5 → ids
  1 (82 px) e 34 (60 px); 1 → id 7 (4 px); 1,68 → id 36 (5 px); agora 0).
- Legado: `legacy::v16::tests::a_format_16_file_opens_born_at_zoom_one_with_every_ink_free`, `format_17_keeps_the_birth_scale_and_the_clean_ink`.
- Adaptados à nova representação (a lei não mudou): `sketch::tests::a_far_sketched_shape_is_drawn_final_and_a_near_one_by_hand`
  (à mão ⇒ sem operação de traço), `mind::sketch_tests::a_sketched_map_draws_its_branches_by_hand` (conta também o
  preenchimento), `video::tests::a_hatched_video_background_is_painted_hatched` (hachura = preenchimento com > 1 subcaminho), e
  o gate de campos `pen_shapes_in_a_sketch_board_are_drawn_like_the_tools_shapes` (a tinta limpa afirma `clean && sketch`).

**Porque os verdes anteriores deixavam passar:** `sketch_paint::the_pens_held_box_reads_as_sketched_as_the_tools` media a tinta
só a zoom 1 e largura 0,5 (e passa verde sem placa); o gate de campos `pen_shapes_…` CODIFICAVA «o que fica tinta limpa
pinta-se como a tinta crua»; `the_sketch_button_flips_the_whole_board_and_what_is_born_after_it` e `shift_…` olham campos/
largura×altura, não tinta. 17 nomes de gate do doc 17 §6 não existem no código sob esse nome.

**Mutações: 11/11 sangram** — sem `reborn` no `pointer_down` · nascer a f=1 · `reborn` que guarda a largura · traço da GPU em
vez do contorno preenchido (o gate de espigões, na placa de verdade) · pintor da tinta limpa desligado · `sketchable` sem tinta
limpa · `smart_ink` com o estilo da caneta · `sketch_targets` morto · `apply` em vez de `apply_mode` · `step` sem repor o modo ·
`ink_finish` de um passo.

**Oráculo Excalidraw** re-corrido, nunca editado à mão: entradas W8, W9, W10, W15 regeneradas pelos testes «grava_a_entrada» e
re-corridas (`corre_esquema.mjs`, `corre_fidelidade.mjs` nas duas versões, `corre.sh w15`); os resultados do Excalidraw
iguais (PNG byte a byte); o `customData` ganha `"birth"`/`"clean"`. O W14 fica de propósito como ficheiro da época do W14
(o README dele nomeia um teste apagado: README velho, ABERTO 6).

**Fecho da onda:** `cargo fmt`; clippy `-D warnings` limpo em `ph2d-board-model/render/edit/sync/excalidraw`, `ph2d-editor-core`,
`ph2d-app-board`, `ph2d-i18n`; `cargo check --all-targets` do workspace limpo. Suítes correidas: board-sync 61/61, board-export
19/19, board-edit 260/260, board-render `it` 11/11 (+GPU 12/12), app-board 91/91, board-model 140/140, board-excalidraw 49/49;
editor-core `board_`/`documents` verdes. ⚠️ O doctest de `ph2d-app-board` falha com E0463 no perfil `smoke` — ambiente, sem
relação com o diff, NÃO comparado com o `main`. ⚠️ NÃO corridos: clippy do workspace inteiro com os alvos todos e a bateria
(só no `/pd-linha-fechar`).

**Varrimento de costura do B13:** sessão viva OK por leitura (mesmo id, sem órfão, Esc devolve o traço, Ctrl+Z lápide);
copiar/mover guarda rascunho e largura, a semente re-deriva do id novo (desenho); `.excalidraw` roughness 1/0 pelo rascunho.
Nenhum tinha teste com forma da caneta — agora a tinta limpa e as formas têm.

- `nextest-impacted` (`PH2D_SUITE=onda`): **19 789 testes, 2 vermelhos**, ambos de `ph2d-tool-painter` (sem diff nesta
  linha desde a base): `mask_gate_tests::the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` passa sozinho;
  `mask_tests::the_mask_stroke_cost_does_not_follow_the_canvas` reprova sozinho a `load` 20–30 (a suíte correu a `load`
  80) — régua de RELÓGIO de outra crate, família [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md); NÃO curado. Se
  reprovar a `load` < 5, é da linha do Painter.

## 4. ABERTO (nesta ordem)

> **Decisões do dono, 22/10:** (a) **SIM** — a opção Rascunho/Final também na barra da FORMA seleccionada (a
> «Sloppiness» do Excalidraw), além da ondinha da barra da esquerda: é o PRÓXIMO item. (b) O `Ctrl+Z` com a
> caneta pousada **fica como está** (desfaz o gesto e o passo anterior) — fechado, não mexer.

1. **Smoke do dono da cena 28** (§5) — se chegar, trate-o primeiro (mecanismo, não sintoma).
2. **R1 — perguntar ao dono** se a barra da PRÓPRIA forma deve oferecer Rascunho/Final (o Excalidraw põe aí o «Sloppiness»).
3. **Ctrl+Z com a caneta premida** cancela o gesto E desfaz o passo anterior — regra de TODOS os gestos do quadro; decidir se
   muda (decisão de produto), não curado aqui.
4. Elementos antigos (`birth` 0) desenhados longe antes desta mudança ficam em escala de mundo: só os nascimentos novos levam a
   escala do ecrã.
5. O tamanho da LETRA não nasce à escala do ecrã (rótulos de formas nascidas longe saem pequenos) — não pedido.
6. README do oráculo: o W14 nomeia um teste apagado.
7. Pendentes do handoff anterior (§4.2): `textAlignVertical` (perguntar), YouTube Premium, «Rascunho/Final» troca a letra
   importada?, os nomeados do bloco 5.
8. Fecho da linha (`/pd-linha-fechar`): clippy do workspace e a bateria.

## 5. Smoke ao dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build --profile smoke -p ph2d-host-desktop && PH2D_BOARD_SMOKE=28 ./target/smoke/ph2d-host-desktop`
2. O quadro abre em Rascunho (o botão da ondinha, na barra da esquerda, aceso). No canto de baixo do frame «2 · Draw here» há
   duas caixas: a da esquerda feita com a ferramenta Retângulo, a da direita com a caneta.
3. Tem de acontecer: as duas caixas têm cara de desenhadas à mão (linhas que não são retas, cantos que passam um pouco). Clique
   numa delas e carregue na ondinha: ela fica com linhas retas. Carregue de novo: volta à mão.
4. Desenhe a sua: com o quadro como está (afastado), ferramenta Retângulo, arraste uma caixa — sai à mão, como as do exemplo.
   Faça o mesmo com a caneta: desenhe uma caixa e fique parado no fim — vira retângulo, também à mão. Experimente um
   triângulo torto com a caneta: também sai à mão.
5. Carregue na ondinha sem nada selecionado: tudo fica reto; Ctrl+Z: volta tudo à mão e a ondinha acende de novo.
6. Deu errado se: carregar na ondinha não muda nada à vista; uma caixa tem um risco que sai do canto; a caixa feita de longe
   sai reta e a de perto à mão; ou depois do Ctrl+Z a ondinha fica com a luz errada.
