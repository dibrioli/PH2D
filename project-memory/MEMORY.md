# Memory index — PH2D (versionada em `project-memory/`, multi-máquina)

> Estado por-módulo: **CLAUDE.md §5**; contratos: **§6**; histórico: git/`docs/HANDOFF_*`.
> Aqui só lições duráveis, perfil, facts não-deriváveis. 1 linha/memória; famílias → `reference_topic_*`.

## Perfil & referência
- [User: Enio (dibrioli)](user_role.md) — dono/decisor; o único dev é a LLM
- [Onde mora cada coisa](reference_canonical_files.md) — a tabela verificada; ⛔ não guarda versão nem contagem (foi assim que apodreceu)
- [GPU tests headless](reference_gpu_tests_run_headless_metal.md) — `--features gpu -- --ignored` roda no sandbox (⚠️ evidência é de Mac/Metal sobre crate já removida; o dev hoje é Linux/RTX)
- [Transcripts são INSTRUMENTO](reference_session_transcripts_are_a_measurable_instrument.md) — `~/.claude/projects/*.jsonl` mede o comportamento do agente; sonda: `scripts/agent-loop-profile.sh`
- [Monitores da workstation](reference_display_topology_workstation.md) — perf no LG (RTX); AOC read-only
- [A workstation travou 2× (08/08)](project_workstation_freeze_memory_reclaim.md) — livelock de reclaim, não bug do PH2D; 577 GB de `target/` é o combustível
- [O VSCode morre por POLÍTICA, não por escolha (14/08)](project_vscode_dies_by_oompolicy_not_by_choice.md) — `OOMPolicy=stop` derruba o scope; o AND do earlyoom nunca fecha quando é o swap que acaba
- [Disco cheio CORROMPE os .o e o mold morre em SIGBUS (22/08)](project_disk_full_corrupts_objects_mold_sigbus.md) — linker a 0% de CPU com `wchan=vfs_coredump`; cura é `cargo clean -p`, e o `df` já não mostra a causa
- [«Disco cheio» com 526 GB livres = METADATA do btrfs; swap 100% = target em tmpfs no zram; csum corrompido = kernel 7.2.0 (22/08)](project_btrfs_metadata_starved_not_disk_full_2026_08_22.md) — três doenças, um instrumento: `scripts/btrfs-health.sh`; cura de metadata é balance (root), não `rm -rf`
- [Prompt Deck](reference_prompt_deck_app.md) — apps pessoais em "Meus Apps"; fonte única `prompts.json`, 3 saídas geradas
- [Atalho global no Plasma 6](reference_kde_plasma6_global_shortcut.md) — `[services][x.desktop]` + o grab que falta após o login
- [HISTÓRICO: aquarela/wash](reference_topic_watercolor_historical.md) — ADR-0096/0099/0108; 17 memórias da era

## Comunicação & decisão
- [Decida, não pergunte](feedback_decide_dont_ask_gold_standard.md) — padrão-ouro, execute, reporte
- [Estilo](feedback_communication_style.md) + [simplicidade](feedback_communication_simplicity.md) — ⚠️ **corrigidas 18/08**: ao Enio, curto e sem jargão (§0.8); denso só para a próxima LLM
- ["Difícil de ajustar" = bug de DESIGN](feedback_ergonomics_verdict_is_a_design_bug.md) — questione o modelo
- [Knob por-passo é ALVO, não taxa](feedback_a_knob_consumed_as_a_per_step_rate_is_a_target_not_a_rate.md) — resposta exponencial e composta por OUTRO knob; meça a fração ÚTIL do curso
- [Remédio novo → velho é CONTAGEM DUPLA](feedback_a_new_remedy_makes_the_old_one_double_counting.md) — 3º ajuste da mesma constante = modelo errado
- [Parâmetro que não muda NADA](feedback_a_parameter_that_changes_nothing_is_discarded_downstream.md) — grepe o consumidor
- [Campo COLAPSADO não fica neutro — ele MANDA](feedback_a_collapsed_field_does_not_go_neutral_it_takes_over.md) — `min=mediana=max` a 2,5× o alvo: o knob grosseirava a peça; dois valores não-neutros idênticos é a assinatura
- [Rótulo promete o que o MODELO entrega](feedback_a_label_must_promise_what_the_model_delivers.md) — "Air Drag" sobre damping uniforme
- [Affordance herdada por analogia](feedback_inherited_affordance_must_be_rederived.md) — gate verde pode pinar bug de design
- [Alvo não-idempotente não exclui autoria](feedback_a_nonidempotent_target_excludes_nothing_split_authoring_from_deposit.md) — separe autoria de depósito; funil no commit
- [Comando de rodar inclui o `cd`](feedback_run_command_include_cd.md)
- [A cwd do Bash VOLTA ao primário](feedback_bash_cwd_resets_and_slips_to_the_primary.md) — Modo L: prefixe todo comando com o `cd` da worktree
- [Exemplo pronto pra smoke](feedback_ready_to_smoke_example.md) — feature nova = auto-play
- [Perfeição sem adiamentos](feedback_perfection_no_deferrals.md) — gaps in-scope fecham na sessão
- [O teto é do HARDWARE](feedback_the_ceiling_is_the_hardwares_never_the_fallbacks.md) — meça antes de limitar
- [Barra de RAZÃO aperta sozinha se o denominador é um knob](feedback_a_ratio_bar_tightens_itself_when_the_denominator_is_a_knob.md) — «atravessa a peça» é fração da peça; a razão triplicou sem defeito nenhum
- [Produto final, não MVP: params PRO por nó](feedback_final_product_every_node_ships_the_full_pro_param_set.md) — o superset do catálogo, conferido por nó (o miss da rotação)
- [Wave de pesquisa RECURSA](feedback_a_research_fanout_recurses_bound_it.md) — limite; verifique você o fato decisivo
- [Painter: 4 causas](feedback_painter_inefficiency_4_causes.md) — costura não-testada / audit=compilar / órfão
- [Comentário velho e código morto MENTEM](feedback_stale_comment_and_dead_code_lie.md)
- ["O design rejeita X"? grepe o gate](feedback_before_declaring_the_design_rejects_an_invariant_grep_for_its_gate.md)
- [Nota de diferido não é spec](feedback_a_deferral_notes_bar_may_exceed_the_projects_policy.md) — confira e corrija a nota
- [Coluna de sonda sem rótulo é lida ao contrário](feedback_an_unlabelled_probe_column_gets_read_backwards.md) — reportei «17 buracos» onde a linha dizia `0 bordo · 17 dobradas`; e quase culpei o instrumento certo
- [Cura medida numa fixtura que NÃO contém o fenômeno lê como inútil](feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless.md) — meça a fração alcançável ANTES do resultado; um zero pode ser implementação meio-feita
- ["NÃO toque neste arquivo" é uma AFIRMAÇÃO](feedback_a_handoff_can_be_wrong_about_its_own_dirty_file.md) — o handoff errou sobre a própria crate; meça antes de honrar
- [A regra tem de estar no CAMINHO de quem a executa](feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it.md) — doc órfão do roteador = regra inexistente
- [Ferramenta só é adotada se um PASSO a chama pelo nome](feedback_a_tool_is_adopted_only_when_a_written_step_names_it.md) — medido: 5 usos contra 13.791 do comando cru; ponteiro ≠ adoção
- [Arquivar sem indexar as RECUSAS é apagá-las](feedback_archiving_without_indexing_the_refusals_deletes_them.md) — e a cura de um doc inchado pode REALOCAR a doença; o teto se mede (80-110 KB)
- [Mecanismo certo, cura errada](feedback_a_correct_mechanism_can_prescribe_the_wrong_cure.md) — meça o mecanismo antes de construir o que a nota prescreve
- [Cerca de Chesterton](feedback_documented_decision_chesterton_fence.md) — "intentionally NOT X" = decisão
- [Revert pode diferir só no TEMPO DE VIDA](feedback_a_reverted_attempt_may_differ_only_in_lifetime_read_the_revert_reason.md) — leia o MOTIVO do revert, não o diff; escopo é o que mata tentativa boa
- [`match` exaustivo NÃO guarda a lista que um laço itera](feedback_an_exhaustive_match_does_not_guard_the_list_a_loop_iterates.md) — variante nova = braço morto sem warning; agulha com espaço nunca casa
- [Convenção vs inércia](feedback_convention_vs_inertia.md) — tem gate? default = mais isolamento

## Git & colisão multi-agente
> Modo C (Mac): colisão real. Modo L (workstation): worktree próprio → só merge (§1.5.5).
- [Commit collision](feedback_parallel_agent_collision.md) — `git status` antes de stage
- [Scoped commit](feedback_scoped_commit_shared_index.md) — `git commit -m msg -- <meus paths>`
- [Desfaça mutação com `cp`](feedback_mutation_undo_with_cp_never_git_checkout.md) — nunca `git checkout`
- [`cargo fmt -p` reformata WIP alheio](feedback_cargo_fmt_p_reformats_foreign_wip.md) — `rustfmt <arquivos>`
- [`str.replace()` sem casar é no-op](feedback_python_replace_silent_noop_after_fmt.md) — `assert old in s`
- [`sed -i` relativo erra de repo](feedback_sed_relative_path_hits_primary_cwd.md) — caminho absoluto
- [Rewrite de token = só arquivos MUDADOS](feedback_a_token_rewrite_scopes_to_changed_files_not_the_whole_tree.md) — `git grep` corrompeu .ttf
- [Mover doc = RESOLVER link, não casar string](feedback_moving_a_doc_means_resolving_links_not_matching_strings.md) — gate antes/depois por path resolvido; `ls-files` pós-`mv` mente
- [Mais perigos de git (6)](reference_topic_git_hazards.md) — stash · reset alheio · fence · worktree-base · mojibake
- [O symlink da MEMÓRIA aponta para o primário](feedback_the_memory_symlink_points_at_the_primary_tree_not_your_worktree.md) — Modo L: salvar pelo caminho do Claude Code escreve no `main`

## Ship / CI / cadência
- [Multi-máquina](project_multi_machine_setup.md) — GitHub fonte única; memória via symlink
- [Fast mode / ship](feedback_fast_mode_ship.md) — dia: commit sem push; fim: ship + babysit
- [Ship = Enio-only](feedback_ship_only_enio_end_of_all_lines.md)
- [Integração = Enio-only](feedback_integration_only_enio_command_end_of_all_lines.md) — feche → handoff → PARE
- [Ship do integrador drena latentes](project_integrator_ship_catches_latents_budget_iterations.md) — 2-4 iterações
- [Ordem de integração se MEDE](feedback_integration_order_comes_from_measured_overlap.md) — sobreposição par-a-par
- [✗ do ship pode ser AMBIENTE](feedback_a_ship_x_can_be_the_environment_not_the_code.md) — tmpfs evapora · disco cheio vira "linking failed"
- ["Está em uso?" → config GLOBAL](feedback_in_use_is_answered_by_the_global_config_and_a_probe_can_start_what_it_measures.md) — apaguei 101 GB de sccache ATIVO; e `sccache -s` SOBE o servidor que ele mede
- [Pipe mascara exit code](feedback_pipe_masks_script_exit_code.md) — verifique o ESTADO
- [Laço colável em idioma bash NÃO itera em zsh](feedback_a_pastable_bash_loop_never_iterates_under_zsh.md) — `for p in $VAR` roda 1× com a string inteira; portão que ENUMERA exige array citado + controle positivo
- [Crase em msg de commit executa](feedback_backticks_in_commit_message_are_command_substitution.md) — `git commit -F`
- [Merge limpo pode estar quebrado](feedback_clean_text_merge_can_be_semantically_broken.md) — `check --workspace`
- [Resolva pelos ESTÁGIOS do índice](feedback_resolve_conflicts_from_index_stages_not_markers.md) — `:1`base `:2`ours `:3`theirs
- [Lista compartilhada funde contra a main de HOJE](feedback_a_shared_list_is_merged_against_todays_main.md) — só ADICIONE; remover é integração
- [Varra marcadores em CADA commit](feedback_sweep_conflict_markers_every_commit.md)
- [Foundational editável = crie isolado](feedback_foundational_editable_design_for_isolation.md) — anote ids
- [CI direto + fmt-skew](feedback_ci_direct_lint_gates_and_fmt_skew.md) — `rustup run <pin> cargo fmt`
- [Ship committed vs WIP](feedback_ship_committed_vs_worktree_wip.md) — `git worktree --detach HEAD`
- [LOC cap = split](feedback_loc_cap_split_not_allowlist_and_fmt_reexpands.md) — fmt ANTES de medir
- [Cap de FN e cap de ARQUIVO são grandezas diferentes](feedback_a_fn_cap_and_a_file_cap_measure_different_things.md) — extrair no mesmo arquivo cura um e estoura o outro; corte para o IRMÃO
- [CI cold-build drift](project_ci_rustcache_stable_drift_pin.md) — pin `@1.95`
- [ship.sh ≠ paridade CI](feedback_ship_parity_gaps_ci_only.md) — bindgen/advisory-db escapam
- [O seletor de impacto é CEGO fora de `crates/`](feedback_an_impacted_test_selector_that_maps_paths_by_prefix_is_blind_outside_it.md) — diff só em `shells/` roda 4 testes e sai verde
- [`rustup default` PERDE para o `rust-toolchain.toml`](feedback_rustup_default_loses_to_the_toolchain_file.md) — o job de MSRV testava o PIN; meça com `RUSTUP_TOOLCHAIN`
- [Números que SOMAM: conte](feedback_numbers_that_sum_across_lines_count_dont_pick.md)
- [Allowlist duplicada mata o gate](feedback_duplicate_allowlist_key_kills_the_gate_at_parse.md) — TOML morre no parse
- [Integrar pré-cutover = drift](project_integration_prefork_lines_ship_drift.md)
- [Cadência de processo (10)](reference_topic_process_cadence.md) — gist em CLAUDE.md §2-§3. ⚠️ o babysit do CI É polling de 15 min (§3)

## Auditoria (famílias — 2 saltos)
- [Reprodução/diagnóstico (18)](reference_topic_repro_discipline.md) — harness/mecanismo · cursor real · não-repro ≠ fix · escala antes de causa · controle positivo
- [⛔ Suíte TOPOLÓGICA é cega a geometria](feedback_a_suite_of_topological_assertions_is_blind_to_geometry.md) — 10.515 verdes sobre o produto destruído; use DUAS réguas de aresta
- [Um parâmetro, dois papéis = erro DEFENSÁVEL](feedback_one_parameter_two_roles_makes_the_wrong_call_defensible.md) — o comentário que justifica o argumento é o alarme
- [Régua que DEDUPLICA não vê duplicação](feedback_a_ruler_that_deduplicates_cannot_report_duplication.md) — χ verde sobre malha não-variedade; conte por ocorrência
- [`round` sem resíduo é mentira silenciosa](feedback_a_round_that_never_reports_its_residual_is_a_silent_lie.md) — resíduo 0,4999 = empate; e a ordem de grandeza dele NOMEIA a parcela que falta
- [Invariante CONSERVADA não mede qualidade](feedback_a_conserved_invariant_cannot_grade_quality.md) — Σ índice = 4·χ é verde por construção; a régua é a CONTAGEM
- [Polilinha sobre malha vira onde a estrutura não vira](feedback_a_polyline_on_a_mesh_turns_where_the_structure_does_not.md) — decida pelo GRAU do grafo; a geometria é desempate
- [Contagem de defeito sem PROVENIÊNCIA culpa a fase errada](feedback_a_defect_count_without_provenance_names_the_wrong_phase.md) — 47 irregulares: 100% do layout, zero da montagem
- [Curva que achata pode precisar de mais pontos](feedback_a_flattening_curve_may_need_more_points.md) — 4 pontos diziam "2º mecanismo"; 2 pontos a mais diziam "é a causa"
- [Ofício de gate (32)](reference_topic_gate_discipline.md) — ausência+presença · razão doente · verde por acidente · paridade CPU/GPU · fixture contém o fenômeno
- [Estado autorado & relógios (19)](reference_topic_authored_state_and_clocks.md) — seed=sample · âncora · id-counter · load adota · ponto fixo · unidades mistas
- [Costura de UI (13)](reference_topic_ui_seam_discipline.md) — pintado/populado/clicado · duas portas · dimmed despacha · default é lei
- [O seed é dono do VALOR, o dispatch do ESTADO](feedback_the_seed_owns_the_value_the_dispatch_owns_the_state.md) — espelho por-quadro REMENDA; `register` inteiro apaga o hover, e fica inerte até alguém dar cor ao estado
- [Provas de mutação (6)](reference_topic_mutation_proofs.md) — RED só sobre visto-VERDE · sobrevivente = gate faltando
- [Duas provas do mesmo ótimo não podem discordar](feedback_two_proofs_of_the_same_optimum_cannot_disagree.md) — gate a INVARIANTE (a partição), não o resultado; instância pequena acerta por acaso
- [«Ótimo provado» é afirmação sobre o OBJETIVO](feedback_proven_optimal_is_a_claim_about_the_objective_not_the_answer.md) — custo linear não separa «esmagar» de «espalhar»; a razão vai DENTRO do quadrado
- [Disciplina de oráculo (9)](reference_topic_oracle_discipline.md) — aparência, não regra
- [Disciplina de fixture (6)](reference_topic_fixture_discipline.md) — só prova o que contém; ordem de setup mascara bug de ordem
- [Protocolo de auditoria (6)](reference_topic_audit_protocol.md) — lentes · claims · state-grep
- [Física do impasto/sculpt (8)](reference_topic_impasto_physics.md)
- [O oráculo grava as FASES INTERMÉDIAS](feedback_the_oracle_writes_its_intermediate_stages_compare_phase_by_phase.md) — `ls` na saída dele antes de reimplementar; ler saída ≠ obra derivada
- [A peça que falta pode JÁ estar construída](feedback_the_missing_piece_may_already_be_built_measure_its_structure_first.md) — meça a estrutura do que já lá está; material produzido-e-ignorado não aparece em régua nenhuma
- [Laço de reparo pode ESCONDER o que agrava](feedback_a_repair_loop_can_hide_the_defect_it_worsens.md) — parar por «não há sinalizados» é critério sobre o DETECTOR; dê-lhe um invariante que ele não pode piorar
- [Guloso que explode com um termo novo pede SEMENTE](feedback_a_perturbation_that_breaks_a_greedy_needs_a_seed_not_a_smaller_perturbation.md) — não baixe o peso nem suavize o guia; olhe o estado sobre o qual ele toma a 1.ª decisão
- [Gate vermelho ao ligar algo novo? corra-o DESLIGADO](feedback_a_new_features_gate_can_expose_a_pre_existing_bug_check_the_control_first.md) — a feature perturba e muda QUAL caso cai; o defeito pode ser antigo e nunca medido
- [Tornar um nó elegível pode REGREDIR um claim parcial — RECUE, não refute inteiro](feedback_making_a_node_eligible_can_regress_a_partial_claim_retreat_dont_refuse_whole.md) — re-meça o doc REAL; regra tudo-ou-nada vira regressão; a cura é un-claim, não refutar o plano

## Padrões de código (gotchas silenciosos)
- [UI = gallery + inspector](feedback_ui_source_of_truth_gallery_inspector.md) — espelhe
- [UI em inglês](feedback_app_ui_english_only.md)
- [Nada de `→` em string literal](feedback_no_tofu_arrows_in_string_literals.md)
- [Tool nova exige IconId](feedback_new_tool_icon_needs_iconid.md)
- [Fan-out registry-init](feedback_fanout_registry_init_friction.md) — 2 testes à mão
- [node-sync glob prefix](feedback_node_sync_glob_prefix_gotcha.md) — crate de nó ≠ `ph2d-node-`
- [Hier companion allowlist](feedback_hier_companion_dispatch_allowlist.md) — 2 sites em `pointer.rs`
- [Registro de painel (5 sites)](reference_topic_panel_registration.md)
- [Inject, don't cap](feedback_pipeline_inject_dont_cap.md)
- [Pixel center vs edge](feedback_pixel_center_vs_edge_coord.md) — subtraia 0.5
- [Exact-pin exige gate substring](feedback_exact_pin_needs_substring_gate.md)
- [ISPC cross-process](feedback_ispc_cross_process_concurrency.md) — crasha com cargo concorrente
- [Zero-alloc gate = capacidade](feedback_zero_alloc_gate_capacity_not_global_counter.md)
- [`Arc::from(Vec)` SEMPRE copia](reference_arc_from_vec_always_copies.md) — `collect` TrustedLen não
- [Clone segurado + detecção por ponteiro = copy-on-write por op](feedback_a_held_clone_plus_pointer_identity_change_detection_forces_copy_on_write.md) — versão, não `as_ptr` (ADR-0124 reincide: Painter 10ms/move @4K)
- [Áudio: meter vivo, sem som](project_audio_multichannel_silence.md) — mute do WirePlumber
- [Claimed-green ≠ seu-OS-green](project_painter_t19_latent_red_macos_2026_05_28.md)
- [Painter "low-res" = canvas 64px](project_painter_canvas_res_64_not_sim_scale.md)

## Arquitetura / norte / perf
- [Dois motores, um estado](feedback_two_engines_one_state_is_worse_than_a_slow_engine.md) — assume o LAÇO inteiro ou nada
- [Contrato congelado ESCOLHE a arquitetura](feedback_frozen_contract_can_pick_the_architecture.md)
- [Tipo em N sítios → componente opcional](feedback_widely_constructed_type_favors_optional_component_over_appended_field.md)
- [A REPRESENTAÇÃO apaga o caso especial](feedback_the_representation_can_delete_the_special_case.md)
- [Invariante na DERIVAÇÃO, não em cada gesto](feedback_enforce_the_invariant_at_the_derivation_not_at_each_gesture.md) — conte os gestos; meça qual LADO machuca (piso ≠ re-derivação)
- [Marca de EVENTO é canal próprio](feedback_a_transient_event_marker_is_its_own_channel.md) — event-sourced, não derivada do estado; teste onde o evento é mais curto que o estado
- [Blindagem — Fase 0](project_blindagem_phase0_2026_06_20.md) — `ph2d-ui-testkit`
- [Pintura VOLTOU](project_painter_brush_came_back_cleanroom.md) = [clean-room Blender](project_blender_texture_paint_reference.md) + [Texture Layer](project_texture_layer_design.md)
- [Norte node-centric](project_node_centric_decision_2026_05_21.md)
- [Motion keyframes adiados](project_motion_keyframes_deferred_timeline_integration.md)
- [Vector cutover ADR-0108](project_vector_cutover_adr0108.md) — `ph2d-vec-*`
- [Flip = Grease Pencil 2D](project_flip_module_grease_pencil_2d.md); [traço = UNIÃO global](project_flip_stroke_analytic_coverage_gp.md)
- [Composição de clips ≠ NLA](project_clip_composition_not_blender_nla.md) — 2D = NESTING
- [Multi-agente = f(HW)](project_multiagent_modo_l_2026_07_05.md) — workstation = Modo L
- [Modo L lento = disco + build 6×](project_modo_l_speed_hole_worktree_targets_slow_path.md)
- [Tool isolation ADR-0040](project_tool_isolation_freeze_2026_05_22.md)
- [Nó consumido pelo renderer = Pure](project_node_effect_pure_for_renderer_consumed.md)
- [Não otimize prematuro](project_m5_perf_validated.md) — 100k @ 60Hz
- [Gates de velocidade](project_perf_audit_2026_05_19.md) — `without_system_fonts()`
- [Perf do Painter (3)](reference_topic_painter_perf.md)
- [Spatial GPU reconcilia vs CPU](project_painter_w4_spatial_gpu_bloom_sh.md)
- [HISTÓRICO: Painter no teto](project_painter_core_files_at_loc_cap.md) — ⚠️ **premissa dissolvida**: cap é 700, os arquivos medem 315/621/650/627. Sobra a técnica de split
- [8GB RAM = full-gate ~10min](project_solo_coord_backlog_ship_2026_05_29.md)
