# Memory index — PH2D (versionada em `project-memory/`, multi-máquina)

> Estado por-módulo: **CLAUDE.md §5**; contratos: **§6**; histórico: git/`docs/HANDOFF_*`.
> Aqui só lições duráveis, perfil, facts não-deriváveis. 1 linha/memória; famílias → `reference_topic_*`.

## Perfil & referência
- [User: Enio (dibrioli)](user_role.md) — dono/decisor; o único dev é a LLM
- [Paths canônicos](reference_canonical_files.md) — SKILL, HANDOFF, ADRs
- [GPU tests headless](reference_gpu_tests_run_headless_metal.md) — `--features gpu -- --ignored` roda no sandbox
- [Monitores da workstation](reference_display_topology_workstation.md) — perf no LG (RTX); AOC read-only
- [HISTÓRICO: aquarela/wash](reference_topic_watercolor_historical.md) — ADR-0096/0099/0108; 17 memórias da era

## Comunicação & decisão
- [Decida, não pergunte](feedback_decide_dont_ask_gold_standard.md) — padrão-ouro, execute, reporte
- [Estilo](feedback_communication_style.md) + [simplicidade](feedback_communication_simplicity.md) — pt-BR direto; recomendação primeiro
- ["Difícil de ajustar" = bug de DESIGN](feedback_ergonomics_verdict_is_a_design_bug.md) — questione o modelo
- [Remédio novo → velho é CONTAGEM DUPLA](feedback_a_new_remedy_makes_the_old_one_double_counting.md) — 3º ajuste da mesma constante = modelo errado
- [Parâmetro que não muda NADA](feedback_a_parameter_that_changes_nothing_is_discarded_downstream.md) — grepe o consumidor
- [Rótulo promete o que o MODELO entrega](feedback_a_label_must_promise_what_the_model_delivers.md) — "Air Drag" sobre damping uniforme
- [Affordance herdada por analogia](feedback_inherited_affordance_must_be_rederived.md) — gate verde pode pinar bug de design
- [Comando de rodar inclui o `cd`](feedback_run_command_include_cd.md)
- [Exemplo pronto pra smoke](feedback_ready_to_smoke_example.md) — feature nova = auto-play
- [Perfeição sem adiamentos](feedback_perfection_no_deferrals.md) — gaps in-scope fecham na sessão
- [O teto é do HARDWARE](feedback_the_ceiling_is_the_hardwares_never_the_fallbacks.md) — meça antes de limitar
- [Wave de pesquisa RECURSA](feedback_a_research_fanout_recurses_bound_it.md) — limite; verifique você o fato decisivo
- [Painter: 4 causas](feedback_painter_inefficiency_4_causes.md) — costura não-testada / audit=compilar / órfão
- [Comentário velho e código morto MENTEM](feedback_stale_comment_and_dead_code_lie.md)
- ["O design rejeita X"? grepe o gate](feedback_before_declaring_the_design_rejects_an_invariant_grep_for_its_gate.md)
- [Nota de diferido não é spec](feedback_a_deferral_notes_bar_may_exceed_the_projects_policy.md) — confira e corrija a nota
- [Cerca de Chesterton](feedback_documented_decision_chesterton_fence.md) — "intentionally NOT X" = decisão
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
- [Mais perigos de git (5)](reference_topic_git_hazards.md) — stash · reset alheio · fence · worktree-base · mojibake

## Ship / CI / cadência
- [Multi-máquina](project_multi_machine_setup.md) — GitHub fonte única; memória via symlink
- [Fast mode / ship](feedback_fast_mode_ship.md) — dia: commit sem push; fim: ship + babysit
- [Ship = Enio-only](feedback_ship_only_enio_end_of_all_lines.md)
- [Integração = Enio-only](feedback_integration_only_enio_command_end_of_all_lines.md) — feche → handoff → PARE
- [Ship do integrador drena latentes](project_integrator_ship_catches_latents_budget_iterations.md) — 2-4 iterações
- [Ordem de integração se MEDE](feedback_integration_order_comes_from_measured_overlap.md) — sobreposição par-a-par
- [✗ do ship pode ser AMBIENTE](feedback_a_ship_x_can_be_the_environment_not_the_code.md) — tmpfs evapora
- [Pipe mascara exit code](feedback_pipe_masks_script_exit_code.md) — verifique o ESTADO
- [Crase em msg de commit executa](feedback_backticks_in_commit_message_are_command_substitution.md) — `git commit -F`
- [Merge limpo pode estar quebrado](feedback_clean_text_merge_can_be_semantically_broken.md) — `check --workspace`
- [Resolva pelos ESTÁGIOS do índice](feedback_resolve_conflicts_from_index_stages_not_markers.md) — `:1`base `:2`ours `:3`theirs
- [Lista compartilhada funde contra a main de HOJE](feedback_a_shared_list_is_merged_against_todays_main.md) — só ADICIONE; remover é integração
- [Varra marcadores em CADA commit](feedback_sweep_conflict_markers_every_commit.md)
- [Foundational editável = crie isolado](feedback_foundational_editable_design_for_isolation.md) — anote ids
- [CI direto + fmt-skew](feedback_ci_direct_lint_gates_and_fmt_skew.md) — `rustup run <pin> cargo fmt`
- [Ship committed vs WIP](feedback_ship_committed_vs_worktree_wip.md) — `git worktree --detach HEAD`
- [LOC cap = split](feedback_loc_cap_split_not_allowlist_and_fmt_reexpands.md) — fmt ANTES de medir
- [CI cold-build drift](project_ci_rustcache_stable_drift_pin.md) — pin `@1.95`
- [ship.sh ≠ paridade CI](feedback_ship_parity_gaps_ci_only.md) — bindgen/advisory-db escapam
- [Números que SOMAM: conte](feedback_numbers_that_sum_across_lines_count_dont_pick.md)
- [Allowlist duplicada mata o gate](feedback_duplicate_allowlist_key_kills_the_gate_at_parse.md) — TOML morre no parse
- [Integrar pré-cutover = drift](project_integration_prefork_lines_ship_drift.md)
- [Cadência de processo (10)](reference_topic_process_cadence.md) — gist em CLAUDE.md §2-§3

## Auditoria (famílias — 2 saltos)
- [Reprodução/diagnóstico (9)](reference_topic_repro_discipline.md) — harness/mecanismo · cursor real · não-repro ≠ fix · escala antes de causa · controle positivo
- [Ofício de gate (21)](reference_topic_gate_discipline.md) — ausência+presença · razão doente · verde por acidente · paridade CPU/GPU · fixture contém o fenômeno
- [Estado autorado & relógios (19)](reference_topic_authored_state_and_clocks.md) — seed=sample · âncora · id-counter · load adota · ponto fixo · unidades mistas
- [Costura de UI (11)](reference_topic_ui_seam_discipline.md) — pintado/populado/clicado · duas portas · dimmed despacha · default é lei
- [Provas de mutação (5)](reference_topic_mutation_proofs.md) — RED só sobre visto-VERDE · sobrevivente = gate faltando
- [Disciplina de oráculo (5)](reference_topic_oracle_discipline.md) — aparência, não regra
- [Disciplina de fixture (3)](reference_topic_fixture_discipline.md) — só prova o que contém
- [Protocolo de auditoria (5)](reference_topic_audit_protocol.md) — lentes · claims · state-grep
- [Física do impasto/sculpt (8)](reference_topic_impasto_physics.md)

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
- [Áudio: meter vivo, sem som](project_audio_multichannel_silence.md) — mute do WirePlumber
- [Claimed-green ≠ seu-OS-green](project_painter_t19_latent_red_macos_2026_05_28.md)
- [Painter "low-res" = canvas 64px](project_painter_canvas_res_64_not_sim_scale.md)

## Arquitetura / norte / perf
- [Dois motores, um estado](feedback_two_engines_one_state_is_worse_than_a_slow_engine.md) — assume o LAÇO inteiro ou nada
- [Contrato congelado ESCOLHE a arquitetura](feedback_frozen_contract_can_pick_the_architecture.md)
- [Tipo em N sítios → componente opcional](feedback_widely_constructed_type_favors_optional_component_over_appended_field.md)
- [A REPRESENTAÇÃO apaga o caso especial](feedback_the_representation_can_delete_the_special_case.md)
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
- [Painter core NO TETO](project_painter_core_files_at_loc_cap.md) — 600 LOC
- [8GB RAM = full-gate ~10min](project_solo_coord_backlog_ship_2026_05_29.md)
