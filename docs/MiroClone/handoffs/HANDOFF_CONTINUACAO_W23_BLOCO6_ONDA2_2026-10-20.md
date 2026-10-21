# CONTINUAÇÃO — `line/MiroClone` W23: bloco 6, 2.ª onda (M4, mutações, realce das menções, a suíte) — 2026-10-20

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_BLOCO6](HANDOFF_CONTINUACAO_W23_BLOCO6_2026-10-20.md)
> (B12–B15 e a cena 28) · lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) ·
> mecanismo e tabelas: [17_bloco6_plano.md](../17_bloco6_plano.md) §9 «Feito depois» e «Provas de mutação do bloco», §10 (M4).
> Base = main `100d34c8e0`. Sem integração nem push: só por ordem do Enio.

## 1. Feito (por ordem; o detalhe está no doc 17, não aqui)

- `8c7021ed33` **M4** — gate `our_lasso_at_full_cover_is_their_contain` (`ph2d-board-excalidraw`, `fidelity_bloco6_tests.rs`):
  `Editor::lasso_pick_covering(doc, trail, 1.0)` = «contain» do Excalidraw next 0.18.0-143b5b6 nos casos 1, 1b, 2–9, 6b; o
  caso 10 (grupos) é divergência escolhida; controlo: com `LASSO_COVER` 0.9 diverge EXACTAMENTE em 1, 1b, 6b.
- `ff9703e2a3` **B14, realce** — as menções saem a azul (`ColorToken::Accent`) no texto do fio: `paint_text_block_spans`
  (`paint_text.rs`, cor por cluster na mesma passada de layout) + `mention_spans` (`board_team_paint.rs`).
- `712e79f899` `3d6a868046` **mutações** — 63 em 6 leis; 1.ª ronda 38 sangram, 25 sobrevivem (9 inofensivas por desenho, 16
  gates em falta); escritos os 16 ⇒ as 16 sangram (C15 só com `a_turned_frame_counts_by_its_turned_box`). Tabela: doc 17.
- ⭐ **Achado:** `tests::alt_drag_leaves_the_original_…` e `tests::alt_resizes_about_the_centre` (`ph2d-board-edit`) falhavam no
  HEAD desde o B12 (`Ctrl+Alt`+arrastar é o laço; o Ctrl deles era incidental). Cura nos testes (`ALT` só).
- `a33006e3eb` **arrumação** — fmt; clippy: `InkGesture::Draw.smart` em `Box` (large_enum_variant), `TabStack` em
  `documents_tabs.rs` (type_complexity), o gate da cena 28 lê `lasso_frame`.
- `f144873883` **a suíte da onda** — ver §3.
- Fotos da cena 28 (1366×768 e 1930×1040) refeitas em `ferramentas/w23_fotos/`: cena íntegra; o azul só se vê com o fio
  aberto («Show»).

## 2. O que um merge pode partir

- **Ficheiros movidos por CORTE** (outra linha que edite os de origem conflita): `element.rs` → `element_identity.rs`
  (Origin/OriginSlot/Rev); `gesture.rs` → `begin_lasso` + `region_move` em `select_region.rs`; board-edit `lib.rs` → o relógio
  do B13 em `smart.rs`; `menu_tables.rs` → `menu_tables_board.rs` (posto também na lista e no filtro do gate
  `every_menu_row_reaches_a_handler`).
- **DAG da fundação:** `Send`/`Action`/`Alert`/`LIFE_S` vivem agora em `crates/ph2d-editor-core/src/documents_alert.rs`
  (`documents::alert`); quem importava `screens::…` desses tipos reescreve o caminho.
- `paint_text_block_spans` é API nova do `paint_text.rs`; o `paint_text_block` antigo mantém-se.
- Isenção `authored` do gate `nenhum_painel_isento_deixou_de_abrigar…` **removida** (o `Design` do quadro entrou na tabela).
- Contadores (DELTA): zero em `PROJECT_SCHEMA`, formato do `Board` e protocolo (continuam 16 e 10); contratos §6 intactos.

## 3. Prova de fecho da onda

`nextest-impacted` (`PH2D_SUITE=onda`): **19 769 testes, 19 763 verdes, 6 vermelhos**; os 5 do bloco curados em `f144873883`:

1. DAG da fundação `documents`→`screens` (§2, `documents_alert.rs`);
2. tectos de LOC por CORTE: `element.rs` 729→674, `gesture.rs` 727→683, board-edit `lib.rs` 712→697, `menu_tables.rs` 715→660;
3. `no_magic_numeric`: proporções marcadas `LITERAL-PX-OK` (`board_bar_look` `more_dots`, a bolinha em `board_team_notify`);
4. `no_tofu_glyphs`: ↑/↓ fora das mensagens dos testes de menção;
5. a isenção morta `authored` (§2).

O 6.º, `ph2d-tool-painter … a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs`, passa sozinho a `load` 53:
flake de carga de OUTRA crate, sem diff nesta linha — NÃO curado; se repetir, entra na família
[FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md). Os 6 re-corridos por nome: verdes; clippy das crates tocadas limpo.
⚠️ NÃO corridos: clippy do workspace inteiro e a bateria do fecho da linha (só no `/pd-linha-fechar`).

## 4. ABERTO (nesta ordem)

1. **Smoke do dono da cena 28** (ainda não veio) — se chegar, trate-o primeiro (mecanismo, não sintoma).
2. Lista-mãe: `textAlignVertical` (top/bottom do Miro — só existe na API, sem UI nem oráculo: **perguntar ao dono** se o quer
   antes de construir); perguntas ao dono (YouTube Premium; «Rascunho/Final» troca também a letra importada?); os nomeados do
   bloco 5 (rect vivo/elipse sem a busca do «menor erro» para o ponto fixo do lado de lá — inexprimível, doc 07 §3; o caminho
   de longe do rascunho para preenchimento às riscas — por medir, doc 16 recusas).
3. Fecho da linha (`/pd-linha-fechar`): clippy do workspace e a bateria.

## 5. Smoke ao dono (curto)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build --profile smoke -p ph2d-host-desktop && PH2D_BOARD_SMOKE=28 ./target/smoke/ph2d-host-desktop`
2. No topo aparece «Rui mentioned you…»; clique em «Show».
3. Tem de acontecer: abre o comentário do Rui e o «@nome» na mensagem aparece a AZUL; o resto do texto fica na cor normal.
   Depois siga as quatro notas compridas (laço · desenhar e segurar · menção · mandar para o «Plan»).
4. Deu errado se: o «@nome» sai da mesma cor do resto, ou letras somem/sobrepõem-se na mensagem; e os sinais do handoff
   anterior (laço a levar «Half outside», caixa segurada que não vira rectângulo, «Move to board…» que não leva as notas, etc.).
