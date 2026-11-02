# HANDOFF de continuação — line/UIUX, F50 (02/11): Boolean e Convert na barra de ações ④, à Figma

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, commit `6382bcd1e6` sobre `ca6875143c`. **Nada integrado, nada
enviado.** Anterior: [F49](HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F49.md) (smoke OK). Plano, medição e decisões:
[F50_boolean_e_convert_a_figma.md](../F50_boolean_e_convert_a_figma.md) §4. Memória:
`project-memory/project_vector_advanced_figma_boolean_first_2026_11_02.md` (revista com as decisões da maquete).

## 1 — Decisões do dono (sobre a maquete, 02/11)

1. «Faça como o Figma»: booleana SEMPRE viva, a chave «Live» sai; o permanente é o Flatten.
2. «Tirar do painel e deixar os mais comuns na barra de ações e "..." na barra de ações para as menos comuns».

## 2 — O que mudou (detalhe no doc §4)

- ④: Union/Subtract/Intersect/Exclude (acesa a do grupo, `state::set_bool_group(Option<u8>)` publicado pela shell em
  `fase_vector_bool_shape_row`) + Flatten (`VECTOR_FLATTEN`) + Outline Stroke. «⋯» novo:
  `ToolBars::actions_more` → `ids::SELECTION_ACTIONS_MORE` → `ContextMenuKind::SelectionActionsMore`
  (`chrome/selection_actions.rs`); raras na paleta e o «⋯» pisca (`flash.rs`).
- Flatten = `ph2d_app_vec::bool_flatten::flatten` (bake dos grupos mais de fora + Union destrutiva). A destrutiva saiu
  da shell (`apply_vec_boolean` → `bool_flatten::pathfinder_selection`, testes movidos). Atalhos `Ctrl+Alt+U/I/D/X` armam
  a viva. Ids `VECTOR_BOOL_LIVE_ON/OFF`, `VECTOR_BOOL_APPLY`, `VECTOR_SECTION_CONVERT`, `VECTOR_SECTION_BOOLEAN` saíram
  (⚠️ `VECTOR_SECTIONS` é append-only partilhada: tirar é trabalho do integrador conferir).
- Painel: sem Boolean/Convert (`paint_boolean.rs` apagado; `seam_bool.rs` apagado — os gates de ponteiro real da ④
  cobrem; `bool_registration_parity` aponta agora a `bars_actions.rs`).
- Cena 6 segue a Union viva; cena **53** nova. Catraca app-vec: 7.º lote de `//!` → 93 584 / 93 637.

## 3 — Provas

- Vermelho primeiro: `bool_flatten` 3/4 vermelhos com o stub (o 4.º é controlo). Os gates da ④ (`f50_*`) e do «⋯»
  (`the_more_of_the_action_bar_*`) foram escritos antes da implementação mas só corridos depois (verdes) — ⚠️ falta a
  prova de mutação deles (ABERTO 2).
- Verdes por nome: app-vec `bool_` 69, `vector_mode` 42, `a_cena_5` 3; panel-vector `f50_` 5,
  `the_bars_carry_the_gesture` 21, `bool_registration_parity`, `paint_body_plan`, `every_word…`, `as_abas_do_titulo`;
  editor-core `icons` `float_bar` `context_menu` `menu_rows` `hr12` `global_palette` `flash` `left_rail`; registry-init
  `the_more_of_the_action_bar` 2, `the_vector_bars_answer_a_real_pointer` 3, `the_palette_reaches_the_vector_bars` 2;
  desktop `the_boolean_cooks_before_the_alignment` 4, `input_text` 9, `the_app_never_reshapes_a_still_screen`,
  `compound` 2; editor-core `the_shell_only_shrinks` 2. fmt dos ficheiros tocados; clippy por correr (fecho).
- Foto: `img_F50/cena53.png`.

## 4 — Aberto

1. **Smoke do dono da F50 (PARAR)** — cena 53.
2. Mutação dos gates F50 (④, «⋯», Flatten) + clippy `-D warnings` das crates tocadas (fecho).
3. Booleanas aninhadas no Flatten: usa o grupo mais de fora; não medido com aninhamento.
4. Herdados: grupo de formas como parte do Draw (F49 §2); Inner/Outer + «Above Effects» (F48); os quatro W do perfil
   ainda `slider_row`; `cargo fmt --all --check` alheio (pixel-preview, connector_handles).
5. Depois deste bloco (dono): Path Effects em lista (Contour, Pattern on Path, Envelope) → Blend + Morph.

## 5 — Smoke do dono, 1.ª volta (02/11)

- Dono: «retirar do app as operações booleanas menos comuns (que estão dentro dos ...)» e «retire os ...
  também». Feito: saíram da UI o «⋯» da ④, Minus Back/Trim/Crop/Merge e Make/Release Compound (ids,
  registo, despacho, i18n, testes; o gate `the_more_of_the_action_bar_opens_the_rare_verbs` foi apagado).
- O motor fica: projetos antigos com essas operações (códigos 4..7 em `bool_live`, compound paths)
  desenham igual.
- Prova nova: `f50_…::as_raras_e_o_mais_sairam_do_app`. Doc: `F50_boolean_e_convert_a_figma.md` §4 «Revisto
  pelo dono no smoke».
