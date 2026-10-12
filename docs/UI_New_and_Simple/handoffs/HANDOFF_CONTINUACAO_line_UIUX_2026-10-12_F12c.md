# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 12/10 — F12 depois da 3.ª janela (B · C · D · F feitas)

> Regras R1–R9: topo do [handoff da F10](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md). Prompt:
> [`20_prompt_F12_continuacao…`](../vetor_nova_ui/20_prompt_F12_continuacao_setas_no_lugar_dos_modais.md)
> (reafirma o [18](../vetor_nova_ui/18_prompt_F12_sem_modais_e_o_traco.md)). Janela anterior:
> [F12b](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F12b.md). O traço: doc 19 §5.

## Onde a linha está

- `line/UIUX`, `Worktrees/line-UIUX`. Rebaseada sobre o `main` `9ea782138` em 12/10 (21 commits
  da line/Vector, sem conflito; Mergiraf não interveio). F9–F11 fechadas e NÃO integradas; F12:
  A (traço) + B, C, D, F feitas nesta janela; **E e G por fazer**; fecho por fazer.
- Commits desta janela: `222f4630b1` (a fila de ⇕), `d4fa902bf6` (o cartão sai; a etiqueta sob o
  cursor), e o corte de LOC de `stroke_uniform.rs`/`symbols.rs` (ver «Aberto» 1).

## O que a janela entregou (técnico)

- **A fila de ⇕** (`shape_handles`): `Spot::Box` → `Spot::Rail` — o 1.º ⇕ na borda direita da
  caixa visível a ¼ do topo (o lugar aprovado da estrela), os seguintes para FORA a
  `RAIL_PITCH (1,5) · park_at` = 27 px (os discos de acerto, 24 px, não se tocam), pela ordem da
  tabela (`rail_slot`). `Law::Steps` lê o tipo do campo: inteiro/escolha = 1 degrau por 1/24 da
  caixa; CONTÍNUO = a faixa inteira na altura da caixa, no passo do campo (`step_world`).
- **Tabela** (`shape_handles_table`): B polígono — `EdgeMid(0)` da envergadura de volta + ⇕ de
  canto/curva/liso; C rectângulo — ⇕ do Corner; o *Smoothing* sai do Vetor por
  `shapes::shown_fields` (tabela `NOT_SHOWN` em `shapes_units`): o slot 4 FICA no documento — o
  Motion (`motion_shape_gen`) escreve-o e o `cook` honra-o; **nenhuma migração, `PROJECT_SCHEMA`
  intacto (188)**; um save antigo com squircle continua squircle mas sem chip; D círculo/pizza —
  o furo de volta (`Law::Radial`, estaciona) e o campo NOVO apenso `Corner round` (Px, neutro em
  zero; `ph2d_vec_scene::round_slice_corners` = o motor da quina viva sobre as quinas da fatia),
  com o ⊙ em `Spot::Rounding{corner:0,next:false}` que SOME sem quina (o `spot_at` pergunta
  `corner_at`); F estrela (canto, curva, liso, torção), espiral (expansão, interior), engrenagem,
  balão oval, pensamento, explosão, nuvem e os três isométricos — tudo ⇕.
- **O cartão saiu do produto** (`d4fa902bf6`, −865 linhas): `ToolBars::card`, `tool_bars_card.rs`,
  `card_chips`, `shape_handles_shown` e os três testes de integração dele. `selection_gap()` (a
  folga da ④) mora em `float_bar.rs` com o gate
  `the_action_bar_floats_one_handle_plus_lg_off_the_selection`.
- **A etiqueta** (`shape_handles_live::readout`): da alça agarrada; sem arrasto, da que está sob o
  cursor. Uma escolha diz o nome («Corner: Concave»). E o `hot` de `view` conta nos pontos
  desenhados (a quina sem ponto desalinhava o halo — defeito latente que o furo+quina acendeu).
- Gates novos/alterados: o censo `the_fields_without_a_gesture_are_counted_not_listed` = ZERO para
  todas; `shape_handles_f12_tests` (fila lado a lado em todas as formas; ⇕ contínuo; ⇕ de
  escolha); `the_slice_corners_round_and_zero_is_the_slice`; `what_was_on_the_card_is_an_arrow_on_the_side`;
  o anti-campo-morto `every_declared_field_of_every_shape_moves_its_geometry` procura o outro
  campo nos DOIS extremos (a quina só age com a elipse aberta); `the_hot_halo_counts_the_drawn_points`.
- Corridas dirigidas verdes: tool-vector lib 96/96 · app-vec lib 887 + live 18 · panel-vector
  45 + 264 · registry-init 133 · editor-core float_bar 13 · vec-scene round 7.

## O que FALTA, por ordem

1. **O corte de LOC** (`workspace_src_files_under_loc_cap`): `ph2d-vec-render/src/stroke_uniform.rs`
   (758, de `f4d47c8e83`) e `ph2d-vec-scene/src/symbols.rs` (715, A2) — confirmar que o commit
   do corte entrou e o gate está verde (`git log -3`).
2. **E — a espiral.** Hipótese (MEÇA antes de curar, pelo press/arrasto REAL da shell como as
   cenas 15–17): a `Law::Winding` arredonda `wound/360` e o *Turns* é inteiro (1..8) — um arrasto
   curto da ponta não muda nada; é preciso dar ~½ volta à volta do centro. O oráculo (Inkscape,
   `shape_handles_f9_oracle_tests`) tem `revolution` FRACCIONÁRIA (3,000 → 3,250 a +90°): a cura
   provável é o *Turns* contínuo (o `cook` da espiral aceita fracção? confirmar) com a ponta a
   seguir o ângulo da mão. Os ⇕ de *Expansion* e *Inner* já estão (F).
3. **G — cenas e fotos.** A cena 16 (`vector_mode_card.rs`, estágios 80–86) ensina o CARTÃO que já
   não existe — refazer (o nome do ficheiro também) para a fila de ⇕; refazer 15–17 se mudaram;
   cena NOVA `=19` (a contagem vive em `vector_mode.rs`): várias formas com as setas lado a lado
   (polígono com a envergadura, estrela com 5 ⇕, pizza com furo e quinas, espiral). Fotografar
   TODAS (`docs/Components/ferramentas/fotografa_cena.sh`) e comparar com
   `/var/tmp/ph2d_F12_dono/7_…png` antes do relatório. Confirmar por foto que o cartão NÃO aparece.
4. **Decisão para o dono (ver abaixo):** o ⇕ do *Viewed* dos isométricos sobe para «From below».
5. **Fecho** (DIRETRIZ §1.5.9): mutação ANTES do portão — as do F12b item 5 e as novas:
   `RAIL_PITCH`/`rail_slot`, o ramo contínuo do `Steps` (o `step_world`, o `int`), o
   `NOT_SHOWN`, `round_slice_corners` (o filtro `Corner`, o `r ≤ 0`), o `corner_at` que esconde
   a quina, o `hot` dos pontos desenhados, o ramo `hover` do `readout`; depois o portão batched,
   o handoff de INTEGRAÇÃO novo (F9–F12, cita o da F11), o CLAUDE.md §5.1 da UI/UX (tirar
   «o cartão lateral (D10)», smoke `…6..19`, «Último:» ≤ 700 bytes), `doc-index.sh`,
   `rm -rf target/*/incremental`, binário smoke 2×, smoke ao dono (F11 + F12 juntas).

## Perguntas ao dono (no relatório final, com recomendação)

- O bico cortado (doc 19 §5, a/b/c, fotos `/var/tmp/ph2d_F12_cena18/`) — ainda sem resposta.
- A F11: a *Resize Box* mantém a espessura do traço (*Scale Strokes & Effects*)? — sem resposta.
- O *Viewed* dos isométricos (De cima / De baixo) virou ⇕ — subir = «De baixo». Recomendação:
  inverter o sentido só para ele (subir = ver de cima), se o dono concordar.
- Um rectângulo arredondado gravado ANTES com *Smoothing* continua suave, sem controlo — aceitar
  (recomendado) ou zerá-lo ao abrir.
