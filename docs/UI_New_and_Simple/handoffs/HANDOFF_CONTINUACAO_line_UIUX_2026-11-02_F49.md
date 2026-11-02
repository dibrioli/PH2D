# HANDOFF de continuação — line/UIUX, F49 (02/11): o Draw do vetor FICA ao escolher outras formas

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, sobre `9362272bff` (F48d). **Nada integrado, nada enviado.**
Anterior: [F48d](HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_F48d.md) (smoke do dono OK em 02/11, cenas 50 e 51 aprovadas).
Decisão do dono em memória: `project-memory/project_vector_draw_stays_on_vector_selection_2026_11_02.md`.

## 1 — Pedido (dono, 02/11, verbatim)

*«se estou num objeto vector modo draw e seleciono outro objeto vetor ou vários objetos vector, não deve sair do modo
draw para o modo Object. Deve permanecer em draw. Diferente de outros tipos de objetos, temos boolean com vector e não é
legal sair do modo draw ao selecionar mais de um objeto»* — EXCEÇÃO Vector → Vector à lei 2 da W1 de 24/10
([01_W1_modos_selecao_inspector.md](../../GameEngineFuncional/01_W1_modos_selecao_inspector.md), agora com a linha 2b).

## 2 — Mecanismo

- Medido antes: as quatro portas (clique no canvas `despacho_clique_pick.rs:198`, linha da Hierarchy
  `fase_hierarchy_select_lock.rs`, laço `lasso_chosen`, intervalo `range`) chamam UMA lei, `object_mode::switches`,
  que desde 24/10 tratava as partes de um modo do TIPO (`whole_kind`) como «outro objecto» ⇒ sai. As booleanas, o
  Shape Builder e o Offset operam sobre `pen.selected_paths()`, que já segue a selecção do herói
  (`vec_selection::sync_selection`, ramo 2) ⇒ ficar no modo basta; o trancado não precisa de mudar.
- `ModeFamily::stays_on_its_kind(mode)` (omissão `false`; `ph2d-app-vec` responde `true` em Draw; o Image Edit, também
  `whole_kind`, continua `false`) → publicado no passo 4 do `drive` como `ModeState::kind_stays` → `switches(locked,
  parts, whole_kind, kind_stays, target, range)`: a parte não sai quando `kind_stays`; o intervalo também não pede a
  saída — a porta não nomeia as linhas, e a rede `still_holds` (passo 2b) decide pela selecção que ficou: forma + objecto
  de outro tipo ⇒ Object, com a selecção do artista. (2b sai sem o toast «left»; o 2a dá-o.)
- ⚠️ Fora do âmbito, por medir com o dono: um GRUPO de formas escolhido na Hierarchy não é parte (as `parts` são só
  formas) ⇒ sai para Object.

## 3 — Provas

- Vermelho primeiro: `vector_mode_tests::choosing_other_vector_shapes_in_the_draw_stays_in_the_draw` (uma forma, Ctrl+
  clique, laço de três, intervalo só de formas; intervalo e laço com um sprite SAEM) falhou em `:279` antes da cura. O
  gate antigo virou `the_draw_holds_every_shape_and_choosing_another_kind_leaves_it`. Lei pura em `object_mode_tests`.
- Verdes por nome: editor-core `object_mode`, `mode_drive`; app-vec `vector_mode::tests`, `a_cena_5*`; painter
  `image_edit_mode`; panel-vector `f48_offset_path_tool`; `cargo check` da shell. fmt + clippy `-D warnings` das três
  crates limpos. Portões de fecho: ver §6.
- **Mutação F49: 6/7 sangraram**; o sobrevivente (`leave` sem `kind_stays = false`) é EQUIVALENTE — o passo 4 republica a
  cada quadro e `switches` devolve `false` sem trancado.
- **Mutação F48d (a pendente): 17/20 sangraram**; os três sobreviventes eram LACUNAS e foram fechados nesta onda, cada um
  visto vermelho com a mutação e verde sem ela: Offset no `PRECISE` (`apply_e_remove_so_com_offset_vivo`, fila de baixo
  vazia com 2 formas), `overlay.edit` desligado no Offset (`overlay::tests::the_offset_tool_shows_no_node_overlays`),
  `close_list_on_pick` fecha Holes/Result (`escolher_na_lista_fecha_a_lista`). Sem gate: os ganchos da shell do arrasto
  do Offset (`despacho_clique_vetor_premido/solto`, `fase_vector_tool_handles`) — só o smoke do dono os cobre (já OK).

## 4 — Smoke (cena 52)

`PH2D_OBJECT_MODE_SMOKE=52` = a cena 22 (rectângulo, elipse, estrela, Select na mão, layout Vector) com uma IMAGEM
256 px em branco à esquerda (pelo `Ctrl+N`, movida para `x = −4 m`) e SÓ o rectângulo escolhido, em Draw.
`vector_mode_alinhar::step(…, cena_52)`, `handles` 38 → `alinhar::FIRST`. Foto: `img_F49/cena52.png` (a borda azul
na imagem é o hover do ponteiro parado no centro da janela virtual).

## 5 — Aberto

1. Smoke do dono da F49 (PARAR).
2. Grupo de formas como parte do Draw (§2) — perguntar no smoke, se o dono tropeçar.
3. ABERTO 2 da F48: Inner/Outer + «Above Effects». 4. Os quatro W do perfil ainda são `slider_row`.
5. `cargo fmt --all --check` acusa `ph2d-panel-pixel-preview/src/paint.rs:95` e
   `ph2d-vec-entities/src/connector_handles.rs:142` — não são desta linha (para o integrador).

## 6 — Portões de fecho

Verdes (por nome): `workspace_src_files_under_loc_cap`, `overage_allowlist_has_no_stale_entries`,
`no_new_hardcoded_ui_strings_in_widgets`, `no_hex_color_literals_in_widget_or_screens`, `node_id_collisions` (14),
`architecture_docs_paths_and_smokes_resolve`, `as_faixas_de_estagios_nao_se_cruzam`, `cargo check` da shell.
⛔→✓ `the_shell_only_shrinks_and_every_app_crate_too`: o diff pôs o `ph2d-app-vec` em `93 740 / 93 637`; cura por
CORTE — 6.º lote de cabeçalhos `//!` verbatim (`resize_box_edit`, `bucket`, `vector_bridge`, `driven_style`, `snap`,
`envelope_gesture`, `svg_import`; −135) para `docs/Vector Module/50_cabecalhos_do_app_vec.md` ⇒ `93 605 / 93 637`.
