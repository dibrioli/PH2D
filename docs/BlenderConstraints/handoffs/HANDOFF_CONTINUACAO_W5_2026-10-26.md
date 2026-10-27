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
