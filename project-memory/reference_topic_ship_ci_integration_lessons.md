---
name: reference_topic_ship_ci_integration_lessons
description: Ship, CI e integração — as lições dobradas do índice (gates de fecho, flakes de carga, colisões de integração), verbatim, uma linha por memória
metadata:
  type: reference
---

# Ship / CI / integração — lições dobradas do índice (2026-09-10)

> Cada linha é uma memória com o gancho ORIGINAL do índice; abra o ficheiro para o mecanismo.
> Dobradas aqui porque o `MEMORY.md` passou o teto suportado (32 KB > 24 KB): ficam a dois saltos.

- [Ship = Enio-only](feedback_ship_only_enio_end_of_all_lines.md) · [integração = Enio-only](feedback_integration_only_enio_command_end_of_all_lines.md)
- ⭐ [Duas linhas refactoram o MESMO bloco de maneiras diferentes: funde limpo e deixa a cópia MORTA (4× num dia; só `dead_code` a vê)](feedback_two_lines_can_refactor_the_same_code_differently_and_both_survive_the_merge.md)
- ⭐ [E o veredito «MORTO» de uma linha EXPIRA quando outra o liga no mesmo dia — o painel ficou vivo e inalcançável](feedback_a_dead_code_verdict_from_a_parallel_line_expires_the_moment_another_line_wires_it.md)
- [Resolver conflito por script: um `if` que escolhe o LADO pelo símbolo apaga o doc do outro no mesmo hunk](feedback_a_diff3_resolver_that_branches_on_which_side_has_the_symbol_drops_the_other_sides_doc.md)
- [Mesmo literal nas 2 linhas: a sonda de colisão fica CEGA — meça o delta, não o valor](feedback_when_two_lines_pick_the_same_literal_the_collision_probe_goes_blind.md)
- [✗ do ship pode ser AMBIENTE](feedback_a_ship_x_can_be_the_environment_not_the_code.md) · [«está em uso?» → config GLOBAL](feedback_in_use_is_answered_by_the_global_config_and_a_probe_can_start_what_it_measures.md)
- [Pipe mascara exit code — e num portão em BACKGROUND o ficheiro VAZIO lê-se como verde: CONTE os verdes](feedback_pipe_masks_script_exit_code.md) · [laço bash não itera em zsh](feedback_a_pastable_bash_loop_never_iterates_under_zsh.md) · [crase em commit executa — -F](feedback_backticks_in_commit_message_are_command_substitution.md)
- [DOIS opt-outs no mesmo ficheiro = directório errado, não gate chato](feedback_two_opt_outs_for_the_same_file_mean_it_is_in_the_wrong_directory.md)
- [LOC cap = split p/ irmão](feedback_loc_cap_split_not_allowlist_and_fmt_reexpands.md) · [cap FN ≠ cap arquivo](feedback_a_fn_cap_and_a_file_cap_measure_different_things.md) · [catraca sem censo de obsolescência SOBE (3 achadas)](feedback_a_ratchet_without_a_staleness_census_only_ratchets_up.md)
- [Contagem LITERAL num gate faz cada feature nova editar o teste de outra pessoa — derive; piso contra vácuo pode ser literal](feedback_a_literal_corpus_count_in_a_gate_makes_every_new_feature_edit_someone_elses_test.md)
- ⭐ [Predicado-PROXY que vira constante deixa asserção VÁCUA — o sinal é a mensagem falar de outra coisa que a condição](feedback_a_proxy_predicate_that_becomes_constant_leaves_a_vacuous_assertion.md)
- [Registry não distingue feature ausente de erro de escrita — o oráculo é a ÁRVORE](feedback_a_registry_cannot_tell_a_missing_feature_from_a_typo_ask_the_tree.md)
- [⛔ `-p <crate>` sozinho usa as features POBRES: corra junto com o shell (147k px² mudos)](feedback_testing_a_crate_alone_hides_every_defect_in_a_feature_the_shell_enables.md)
- [Gate que varre UM DIRETÓRIO afirma sobre o diretório; piso contado sobre DECLARAÇÕES não vê consumidor morto](feedback_a_new_feature_can_empty_an_existing_gates_population.md)
- [Gate que varre árvore ≠ filtro de nome](feedback_a_tree_scanning_gate_is_never_reached_by_a_name_filter.md) · [a suíte SEM filtro corre antes de dizer VERDE (4×)](feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate.md)
- [⚠️ O «sozinho» que desmente uma flake exige a CARGA MEDIDA — li 3/3 vermelho a load 82](feedback_a_flake_red_hides_the_rest_of_the_suite.md)
- [Clippy do fecho: alvo do DIFF](feedback_the_closing_clippy_must_cover_every_crate_the_line_touched.md) · [flake esconde a suíte — leia X/Y](feedback_a_flake_red_hides_the_rest_of_the_suite.md) · [corrida de fixtura em /tmp mal arquivada como flake de carga](feedback_a_shared_tmp_fixture_race_is_misfiled_as_a_load_flake.md)
- ⛔ [Build da WORKSPACE unifica features: crate que nomeia dependência OPCIONAL fora do cfg compila verde em todo portão e tem 75 erros SOZINHA — `check-standalone-optional.sh` no ship](feedback_a_workspace_build_unifies_features_and_hides_a_crate_that_cannot_build_alone.md)
- ⛔ [Apagar uma crate não acorda portão local nenhum e parte o job do CI que a nomeia com `-p` — o envio de 13/09 reprovou nos 3 sistemas antes de compilar; `check-workflow-packages.sh` no ship](feedback_deleting_a_crate_breaks_only_the_ci_job_that_names_packages.md)

## Descidas do índice em 2026-09-17 — a rodada de seis linhas pôs o `MEMORY.md` a **224 linhas / 36,5 KB** contra o tecto dele (140 / 17 KB), e o carregador cortou **66 linhas em silêncio**. Estas entradas descem VERBATIM; o ponteiro para esta família continua no índice.

- ⛔⛔ [`--bins` NÃO alcança `tests/` — e o gate pode viver na `tests/it/` de OUTRA crate: 7 vermelhos em `ph2d-editor-core` sobre painéis que a linha editou](feedback_a_bins_run_never_reaches_the_gates_that_live_in_tests.md)
- ⛔ [`ph2d-field-render --test it`: 7-8 vermelhos são contadores GLOBAIS do próprio binário — `24/24` VERDE com `--test-threads=1` a load 75; pré-existente, confirme antes de culpar o seu diff](project_field_render_it_suite_is_green_single_threaded.md)
