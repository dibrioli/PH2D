# Continuação — `line/BlenderConstraints`, W4 FECHADA (W4.6 smoke OK 26/10), abre a W5 (diferenciais)

> Anterior: [`W4.6`](HANDOFF_CONTINUACAO_W4_6_2026-10-26.md). Doc da onda: [`04_W4_cad.md`](../04_W4_cad.md) — **§3.7** (o
> desenho da tool, lido no código), **§3.8** (o que ficou feito, o report do dono, os números e o ABERTO), §4 (gates ✓).
> **Não duplicado aqui.** Plano: [`01_plano.md`](../01_plano.md) §7.

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4`, ~90 commits à
frente do main. `PROJECT_SCHEMA` = 195 (reconta-se na integração), `VEC_SCENE_SCHEMA_VERSION = 24`, família de canal
`CAD = 9` (`ph2d_ecs::canal_cad`; `ANGULO = 2`). Cena de smoke 7 «Caixa com tampa» (`PH2D_CONSTRAINT_SMOKE=7`).

## Feito na W4.6 (não refazer)

Tool «Rules» na coluna esquerda, gizmos, cota clicável com Auto Key, cena 7, e a cura do report do dono
(`AlvosDasFamilias`, selos que não tapam números, REF entre parênteses): **§3.7/§3.8**. Smoke do dono OK em 26/10.

## Próximo passo

00. ⛔ **PRIMEIRO: os 14 vermelhos da linha** que o `PH2D_SUITE=onda nextest-impacted.sh` achou no fecho da W4.6
    (26/10, código parado, `HEAD` = o commit deste handoff). Um ÚNICO bloco (§0.10): triagem por mecanismo, cura,
    re-corrida SÓ por nome. São da LINHA (W1–W4), não do main — a cura é desta janela, não do integrador:
    - `ph2d-app-components` `o_registo_das_fixturas_e_o_do_produto` — o produto tem `register_constraint_components`,
      as fixturas não.
    - `ph2d-app-vec` `envelope_live::tests::the_authored_source_survives_in_each_child` — `VertId(1)` × `VertId(0)`
      na fonte autorada (W4.2: ids por vértice).
    - `ph2d-editor-core` `panel_files_under_loc_cap` — `ph2d-panel-inspector/src/ids/inspector_constraint.rs` 1327 LOC
      (tecto 600: partir/mover, nunca isentar).
    - `ph2d-editor-core` `changing_the_chain_recaptures_the_bend_side` — `fase_bone_smart_and_knobs.rs` deixou de
      escrever o `Chain` (o gate perdeu o sujeito: o código mudou-se? seguir a porta, não o caminho).
    - shell `os_verbos_do_osso_chegam_do_botao_ate_a_lei::{o_campo_do_desvio_chega_ao_componente_em_radianos,
      o_espelho_pergunta_a_porta_e_nao_reconstroi_a_conta}`.
    - shell `the_pencil_owns_its_whole_gesture::{the_pencil_press_runs_before_the_pen_and_the_shape,
      the_scanner_finds_what_it_scans_for}` — «o `input_dispatch` não chama `self.vec.pen.on_press(`» (scanner de
      texto; ver se a W4.6 — `on::press` desviado para o `cad_tool` — ou a W4.4 moveu a chamada).
    - shell `project::tests::{marcas::um_v191_e_um_v192_abrem_sem_marcas, opacidade::um_v191_abre_com_a_opacidade_da_forma_nas_poses,
      scripts::a_v189_…, scripts::a_v190_project_with_a_script_path_imports_it_and_runs}` — ficheiros v189–v192
      recusados (W4.2: `fio_do_projeto` / degrau 195).
    - `ph2d-panel-registry-init` `o_inspector_armado::{toda_porta_do_inspector_e_armada, …_desarmada}` —
      `set_current_inspector_constraints` não entra na varredura de elisões (W1).
    - Herdados do `main` (do integrador, NÃO curar aqui): `shell_files_respect_hr18_loc_cap`
      (`despacho_metodos_picks_e_arrastos.rs` 611), `shell_functions_respect_their_ceiling` (`fase_snapshots_publish` 201).
    Log completo: re-corra `PH2D_SUITE="motivo: re-verificar os vermelhos" bash scripts/nextest-impacted.sh` no fim.
0. **Fechar os ABERTOS 1–3 e 7 do §3.8 num único bloco** (§0.10 — um plano, um lote de gates): chip ② que re-semeia,
   coluna de nomes da timeline que elide «1234 px», unidade em `CANTO`/`TRACO_LARGURA`/`PARAM_*`, e conferir o gate
   `each_vertex_mutation_site_keeps_or_assigns_ids` (✓ na tabela §4). Red-first, por nome.
1. **W5 — diferenciais** (`01_plano.md` §7, linha W5): «A e B colados» bidirecional sobre o solver da W4 · física
   parâmetro→parâmetro (**MEDIR antes** se `drive`+`spring` já exprime) · *behaviours* um-clique como presets dos nós
   existentes (`wiggle`, `oscillator`, `orbit`, `falloff`, `force-attractor`) · layout de HUD (`kasuari` 0.4.13).
   - **Oráculo antes do código** (`oraculo_blender.py` na abertura da onda, §0.9); o `explorador` lê antes.
   - **O desenho vai escrito num doc novo `docs/BlenderConstraints/05_W5_diferenciais.md` ANTES do código.**
   - Perguntar ao dono só o que é PRODUTO (qual dos 4 itens primeiro, se não estiver decidido).
   - Kill-criterion: **0,5 ms / 1 000 itens**.

## Portões por nome (verdes no fim da W4.6)

Os W4.6 do §3.7 que existem (`the_rules_tool_sits_in_the_left_column`, `a_choice_offers_only_the_rules_it_allows`,
`a_rule_made_by_hand_goes_through_junta_regra_and_keeps_the_drawing`,
`clicking_a_dimension_number_selects_it_and_typing_moves_the_drawing`, `delete_removes_the_chosen_rule_in_one_step`,
`badges_only_with_the_tool_dimensions_always`, `the_hit_rect_is_the_painted_rect`, `hover_lights_what_the_rule_binds`,
`the_shape_colour_says_free_defined_conflict`, `typing_a_dimension_with_autokey_records_a_key`,
`the_lid_opens_and_scrubs_to_the_same_bit`) + `a_chosen_rule_points_the_autokey_at_every_path_the_sheet_can_move` +
`no_badge_covers_a_number_or_another_badge_and_a_reference_reads_in_parentheses` + `autokey_pass::` (56) +
`the_shell_only_shrinks` + `workspace_src_files_under_loc_cap` + `no_magic_numeric` + `no_literal_color` + i18n
`every_key_of_this_family_exists_on_both_sides`.

## Armadilhas

- `git commit -- <paths>` não leva ficheiro por rastrear: `git add` antes, `git show --stat HEAD` depois.
- `rustfmt --check` no ficheiro da shell antes de comitar; um teste por chamada no `cargo-test-narrow`.
- O hook recusa comandos cujo texto nomeia programas de placa (`kwin_wayland --virtual`): `ps -eo pid,comm | grep`.
- A entidade do path tem de existir antes de gravar chaves (o `step` da cena espera-a).
- **A população do Auto Key é seleção + mãos + `AlvosDasFamilias`**: uma tool que edita o que não está selecionado
  publica os alvos ANTES do gesto (o diff precisa do quadro anterior).
- Uma sonda temporária no app real (env var + `fotografa_cena.sh` com `FOTO_RAJADA`) achou o mecanismo em minutos:
  comitar o WIP antes e `git checkout` dos ficheiros da sonda depois.
- **Tecto da shell com folga 1** (189 040 / 189 041): a próxima linha da shell tem de MOVER código.
