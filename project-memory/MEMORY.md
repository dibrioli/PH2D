# Memory index — PH2D project (versionada em `project-memory/`, multi-máquina)

> Estado por-módulo em **CLAUDE.md §5**; contratos em **§6**; histórico em **git**/`docs/HANDOFF_*`.
> Aqui só LIÇÕES duráveis, perfil, facts não-deriváveis. Uma linha por memória; famílias coesas
> viram um arquivo-tópico `reference_topic_*` (2 saltos). Detalhe sempre no arquivo.

## Perfil & referência
- [User: Enio (dibrioli)](user_role.md) — dono/decisor; o único dev de código é a LLM
- [Arquivos e paths canônicos](reference_canonical_files.md) — onde estão SKILL, HANDOFF, ADRs
- [GPU tests headless (Metal)](reference_gpu_tests_run_headless_metal.md) — `--features gpu -- --ignored` roda no sandbox
- [Monitores da workstation](reference_display_topology_workstation.md) — meça perf no LG (RTX); AOC é read-only em DDC/CI
- [HISTÓRICO: aquarela/wash + satélites aposentados](reference_topic_watercolor_historical.md) — ADR-0096/0099/0108; 17 memórias da era

## Comunicação & decisão
- [Decida, não pergunte](feedback_decide_dont_ask_gold_standard.md) — decida no padrão-ouro, execute, reporte
- [Estilo](feedback_communication_style.md) + [simplicidade](feedback_communication_simplicity.md) — pt-BR direto; recomendação primeiro; sem AskUserQuestion-spam
- ["Difícil de ajustar" = bug de DESIGN](feedback_ergonomics_verdict_is_a_design_bug.md) — pare de calibrar; questione o modelo
- [Affordance herdada por analogia](feedback_inherited_affordance_must_be_rederived.md) — gate verde pode pinar bug de DESIGN
- [Comando de rodar inclui o `cd`](feedback_run_command_include_cd.md) — smoke com `cd <worktree> &&` junto
- [Exemplo pronto pra smoke](feedback_ready_to_smoke_example.md) — feature nova = exemplo auto-play; não peça montagem
- [Perfeição sem adiamentos](feedback_perfection_no_deferrals.md) — gaps in-scope fecham na sessão
- [Wave de pesquisa RECURSA — limite-a](feedback_a_research_fanout_recurses_bound_it.md) — agentes geram filhos; dê prioridade, verifique você o fato decisivo, mate quando decidir
- [Painter: 4 causas + DIRETIVA](feedback_painter_inefficiency_4_causes.md) — costura não-testada / audit=compilar / órfão
- [Comentário velho e código morto MENTEM](feedback_stale_comment_and_dead_code_lie.md) — removeu a UI? remova o encanamento
- ["O design rejeita X"? grepe o gate](feedback_before_declaring_the_design_rejects_an_invariant_grep_for_its_gate.md) — o repo pode já afirmar o contrário; racionalizar gate verde vira lei falsa
- [Cerca de Chesterton](feedback_documented_decision_chesterton_fence.md) — "intentionally NOT X" = decisão; não sobrescreva
- [Convenção vs inércia](feedback_convention_vs_inertia.md) — tem gate ou é inércia? default = mais isolamento

## Git & colisão multi-agente
> **Modo C** (Mac): colisão real. **Modo L** (workstation): worktree próprio → só merge (§1.5.5).
- [Commit collision paralelo](feedback_parallel_agent_collision.md) — `git status` antes de stage; não agarre staged alheio
- [Scoped commit, índice compartilhado](feedback_scoped_commit_shared_index.md) — `git commit -m msg -- <meus paths>`
- [Desfaça mutação com `cp`](feedback_mutation_undo_with_cp_never_git_checkout.md) — `git checkout` apaga a feature e o gate "passa"
- [`cargo fmt -p` reformata WIP alheio](feedback_cargo_fmt_p_reformats_foreign_wip.md) — use `rustfmt <meus arquivos>`
- [`str.replace()` sem casar é no-op mudo](feedback_python_replace_silent_noop_after_fmt.md) — `assert old in s` sempre
- [`sed -i` relativo erra de repo](feedback_sed_relative_path_hits_primary_cwd.md) — mutação SEMPRE por caminho absoluto
- [Mais perigos de git (5)](reference_topic_git_hazards.md) — stash · reset/checkout alheio · fence · worktree-base · mojibake

## Ship / CI / cadência
- [Multi-máquina Mac/Linux/Windows](project_multi_machine_setup.md) — GitHub = fonte única; memória via symlink
- [Fast mode / ship](feedback_fast_mode_ship.md) — dia: commit sem push; fim: `ship.sh` + push + babysit
- [Ship = Enio-only](feedback_ship_only_enio_end_of_all_lines.md) — nunca ofereça/rode ship no fim da SUA linha
- [Integração = Enio-only](feedback_integration_only_enio_command_end_of_all_lines.md) — feche → handoff → PARE
- [Ship do integrador drena latentes](project_integrator_ship_catches_latents_budget_iterations.md) — 2-4 iterações; gate per-linha não basta
- [Pipe mascara o exit code](feedback_pipe_masks_script_exit_code.md) — `| grep` troca o `$?`; verifique o ESTADO
- [Crase em msg de commit = execução](feedback_backticks_in_commit_message_are_command_substitution.md) — use `git commit -F <arquivo>`
- [Merge limpo pode estar quebrado](feedback_clean_text_merge_can_be_semantically_broken.md) — só `check --workspace` cruza
- [Resolva pelos ESTÁGIOS do índice](feedback_resolve_conflicts_from_index_stages_not_markers.md) — `:1`base `:2`ours `:3`theirs
- [Lista compartilhada só se funde contra a main de HOJE](feedback_a_shared_list_is_merged_against_todays_main.md) — "limpei o MEMORY.md" apagou 4 memórias que a main ganhou pós-fork; só ADICIONE, remover é operação de integração
- [Varra marcadores em CADA commit](feedback_sweep_conflict_markers_every_commit.md) — árvore limpa não prova o histórico
- [Foundational editável = crie isolado](feedback_foundational_editable_design_for_isolation.md) — projete p/ isolamento; anote ids
- [CI direto + fmt-skew](feedback_ci_direct_lint_gates_and_fmt_skew.md) — use `rustup run <pin> cargo fmt`
- [Ship committed vs WIP alheio](feedback_ship_committed_vs_worktree_wip.md) — valide via `git worktree --detach HEAD`
- [LOC cap = split, não allowlist](feedback_loc_cap_split_not_allowlist_and_fmt_reexpands.md) — fmt re-expande → fmt ANTES de medir
- [CI cold-build = drift do `@stable`](project_ci_rustcache_stable_drift_pin.md) — rotaciona rustc-hash; pin `@1.95`
- [ship.sh ≠ paridade CI](feedback_ship_parity_gaps_ci_only.md) — bindgen/advisory-db/nextest-impacted escapam
- [Números que SOMAM: conte](feedback_numbers_that_sum_across_lines_count_dont_pick.md) — o valor certo não está em nenhum lado
- [Allowlist duplicada mata o gate](feedback_duplicate_allowlist_key_kills_the_gate_at_parse.md) — TOML morre no parse e nada escaneia
- [Integrar pré-cutover = drift](project_integration_prefork_lines_ship_drift.md) — rode ship completo no fechamento
- [Cadência de processo (10)](reference_topic_process_cadence.md) — commit/CI/smoke/fase; o gist mora em CLAUDE.md §2-§3

## Auditoria
- [Menu "não faz nada" = falta populate](feedback_context_menu_closes_on_down_repaint.md) — grep o id no `populate_*` PRIMEIRO
- [Busca NEGATIVA precisa de controle POSITIVO](feedback_a_negative_search_needs_a_positive_control.md) — grep vazio mente; prove que a busca acha o que você sabe estar lá
- [Meça a ESCALA antes da causa](feedback_measure_perf_symptom_scale.md) — frame (4-16ms) vs ⅓s muda a classe
- [Harness reproduz mecanismo, não contexto](feedback_harness_reproduces_mechanism_not_context.md) — instrumente o app real
- [1º caso salvo por efeito colateral](feedback_first_case_rescued_by_side_effect_test_repetition.md) — teste a REPETIÇÃO
- [Não-reprodução ≠ correção](feedback_nonreproduction_is_not_proof_of_fix.md) — cheque o `git diff` antes de aceitar
- [Unit-verde ≠ funciona no produto](feedback_tool_unit_green_integration_dead.md) — tool passa CI e está morta; só e2e pega
- [Um clique é um press que DESLIZOU](feedback_a_click_is_a_press_that_drifted.md) — Down/Up na mesma coord é robô
- [Pintado ≠ populado](feedback_painted_is_not_populated_paint_gate.md) — teste a PINTURA; nenhum gate rodava `paint`
- [Widget pronto = um teste CLICA nele](feedback_widget_is_done_when_a_test_clicks_it.md) — sem `WidgetStore` não há Click
- [Teste com os números do PRODUTO](feedback_test_with_product_numbers_not_convenient_ones.md) — `1.0` esconde erro de unidade
- [Geometria com unidades mistas](feedback_geometry_over_mixed_units_needs_the_consumers_conversion.md) — converta com a const do RENDERER
- [Snapshot = PONTO FIXO dos sistemas](feedback_a_snapshot_must_be_a_fixed_point_of_the_systems.md) — senão a normalização vira "ação"
- [Uma régua mede UM relógio](feedback_one_ruler_measures_one_clock.md) — dois dados no mesmo eixo com BASES diferentes; o "confuso" do usuário era bug de modelo
- [Lacuna não é silêncio](feedback_a_gap_is_not_silence_two_answers_across_one_pixel.md) — "ausente" e "presente com influência 0" têm de coincidir no limite, senão é salto
- [Espelhar tempo espelha a FORMA](feedback_mirroring_time_must_mirror_the_shape.md) — o interp mora no key de saída: muda de dono E se espelha
- [Publicador de view não exige cache primed](feedback_a_view_publisher_must_not_require_a_primed_cache.md) — quem AUTORA num instante pode; quem PUBLICA prima sozinho
- [Coordenada derivada: seed = sample](feedback_derived_coordinate_seed_must_match_sample.md) — autoria usa a MESMA transform da leitura
- [Âncora invariante sob transform](feedback_anchor_must_be_invariant_under_user_transforms.md) — ancore em geometria, não em aparência
- [Gizmo errado = cheque o HIT](feedback_gizmo_verify_hit_target_before_transform_math.md) — logue o target ANTES da math
- [Overlay cortado = ORDEM de draw](feedback_overlay_cut_at_boundary_check_draw_order.md) — liste os writers antes de caçar clamp
- [Limiar mora onde o domínio é VAZIO](feedback_a_threshold_must_live_where_the_domain_is_empty.md) — escolha um valor que nenhuma entrada produz
- [Escape que nunca ajuda é enfeite](feedback_an_escape_that_never_helps_is_a_design_bug.md) — meça em vários casos; às vezes remova
- [Defesa em camadas = gate POR camada](feedback_layered_defenses_need_per_layer_gates.md) — mutação de UMA não sangra; pergunte o que cada camada protege sozinha
- [Capture a sessão ANTES do pen-up](feedback_capture_stroke_session_before_pen_up.md) — o Up carimba dabs de CAUDA e mata a sessão
- [Peça sem área pinta uma LINHA](feedback_a_boolean_leaves_slivers_and_a_zero_area_piece_paints_a_line.md) — `area > 0` fica verde; use densidade
- ["Funciona e depois esquece"](feedback_works_then_silently_forgets_recook_wipes_authored_state.md) — recook varre o autorado dentro do derivado
- [CONSTRUA o harness antes de desistir](feedback_try_to_build_the_harness_before_declaring_it_impossible.md) — "o App exige janela" era falso
- [O que sobrevive a um load é ADOTADO](feedback_what_survives_a_load_is_adopted_not_stale.md) — cura por nome vira contaminação
- [Sentinel exige gate no LEITOR](feedback_a_sentinel_needs_a_gate_on_its_reader.md) — `from_bits(0)` entra em PÂNICO
- [Gate de AUSÊNCIA precisa do de PRESENÇA](feedback_absence_gate_needs_a_presence_sibling.md) — "não vaza" fica verde com fill invisível
- [Regra que não OBSERVA não dispara](feedback_a_rule_that_never_observes_cannot_fire.md) — HR-13 somava declarações; 4351 MB
- [Faça a MESMA pergunta ao outro lado](feedback_ask_the_same_question_of_the_other_side.md) — o gate gêmeo nasceu VERMELHO
- [Mesma conta, escrituração diferente](feedback_same_math_different_bookkeeping_diverges.md) — 1 ulp; "soa igual" não é gate
- [round ≠ round entre CPU e GPU](feedback_cpu_gpu_rounding_conventions_diverge.md) — Rust half-away, WGSL half-even; enum-por-param diverge de RAMO no meio-ponto
- [Barra congelada? cheque a ARITMÉTICA](feedback_frozen_bar_check_the_arithmetic_before_gaming_it.md) — o piso pode já estourar a barra
- [Refactor mecânico = impressão digital](feedback_wide_mechanical_refactor_use_a_fingerprint.md) — golden pinado é mina no CI
- [Gate de emenda precisa de frac](feedback_seam_gates_need_fractional_advance.md) — taxa 1:1 nunca lê o 2º frame
- [Determinism sweep](feedback_determinism_sweep_grep_all_transcendentals.md) — grepe todos os transcendentais
- [Gate vermelho no seu código CORRETO? pode ser herdado](feedback_a_gate_red_on_your_correct_code_may_predate_you.md) — rode contra o HEAD shipado; não amplifique
- [Chave de cache: keye no que VARIA o artefato](feedback_a_cache_key_must_key_on_what_varies_the_artifact.md) — derivar do resultado colide; crash, não número errado
- [Gate verde de 1ª pode ser verde por ACIDENTE](feedback_a_green_gate_may_be_green_by_accident.md) — mutação sobreviveu? o suspeito é o fixture (3 em 20, no §4.B)
- [Provas de mutação (5 regras)](reference_topic_mutation_proofs.md) — mute o código · RED só sobre visto-VERDE · oráculo alcançável · otimização dispara · sobrevivente = gate faltando
- [Fixture em regime CAÓTICO](feedback_a_fixture_can_land_in_a_chaotic_regime.md) — Δ enorme? compare com uma quantidade física; magnitude limitada + sinal virando = divergência máxima
- [Disciplina de oráculo (4)](reference_topic_oracle_discipline.md) — aparência, não regra · renderize o contradito · taxa de falso-positivo · valor exato
- [Disciplina de fixture (3)](reference_topic_fixture_discipline.md) — só prova o que contém · zero não falha · gateie as bordas
- [Protocolo de auditoria (5)](reference_topic_audit_protocol.md) — lentes · escopo · claims verificados · state-grep · claims de commit
- [Física do impasto/sculpt (8)](reference_topic_impasto_physics.md) — unidade relativa · clamp=borracha · matéria · lateral · análogo 2.5D · âncora no corpo · acumulação sequencial · canal write-once

## Padrões de código (gotchas silenciosos)
- [UI = gallery + inspector](feedback_ui_source_of_truth_gallery_inspector.md) — espelhe; não improvise chrome
- [UI em inglês](feedback_app_ui_english_only.md) — labels/toasts SEMPRE inglês
- [Nada de `→` em string literal](feedback_no_tofu_arrows_in_string_literals.md) — `assert!`/`expect()` são strings
- [Tool nova exige IconId](feedback_new_tool_icon_needs_iconid.md) — SVG sem variant quebra TODOS os ícones
- [Fan-out registry-init](feedback_fanout_registry_init_friction.md) — tool-sync não regenera os 2 testes à mão
- [node-sync glob prefix](feedback_node_sync_glob_prefix_gotcha.md) — crate de nó não pode começar com `ph2d-node-`
- [Hier companion allowlist](feedback_hier_companion_dispatch_allowlist.md) — 2 sites em `pointer.rs` senão click dropa
- [Condição que ENUMERA seus leitores apodrece](feedback_a_condition_that_enumerates_its_readers_rots.md) — o 3º consumidor degrada em silêncio
- ["O card mais cheio" apodrece](feedback_the_fullest_card_premise_rots.md) — sweep de UI: pergunte a CADA modo, não arme o superset
- [Duas portas p/ a mesma pergunta DIVERGEM](feedback_two_doors_to_the_same_question_diverge.md) — botão e atalho: MESMA função
- [Botão dimmed ainda despacha](feedback_disabled_button_still_dispatches.md) — dim é cosmético; recuse no event.rs
- [Registro de painel/widget (5 sites)](reference_topic_panel_registration.md) — populate · docado-4-sites · clamp-const · 2D-drag · NumberInput range
- [Pipeline: inject, don't cap](feedback_pipeline_inject_dont_cap.md) — injete no buffer, não capeie o resultado
- [Pixel center vs edge](feedback_pixel_center_vs_edge_coord.md) — bilinear espera center; subtraia 0.5
- [Exact-pin exige gate substring](feedback_exact_pin_needs_substring_gate.md) — senão o rebase "limpa" o pin
- [ISPC cross-process](feedback_ispc_cross_process_concurrency.md) — cooker crasha com cargo CONCORRENTE
- [Zero-alloc gate = capacidade](feedback_zero_alloc_gate_capacity_not_global_counter.md) — dhat global é flaky
- [`Arc::from(Vec)` SEMPRE copia](reference_arc_from_vec_always_copies.md) — `collect::<Arc<[T]>>()` de TrustedLen não
- [Áudio: meter vivo, sem som](project_audio_multichannel_silence.md) — não é código; mute do WirePlumber
- [Claimed-green ≠ seu-OS-green](project_painter_t19_latent_red_macos_2026_05_28.md) — build o commit claimed-green ANTES
- [Painter "low-res" = canvas 64px](project_painter_canvas_res_64_not_sim_scale.md) — cheque a res do source ANTES do shader

## Arquitetura / norte / perf
- [Dois motores, um estado = pior que devagar](feedback_two_engines_one_state_is_worse_than_a_slow_engine.md) — caminho rápido com estado assume o LAÇO inteiro ou nada
- [Contrato congelado ESCOLHE a arquitetura](feedback_frozen_contract_can_pick_the_architecture.md) — o que sobra traz um invariante
- [A REPRESENTAÇÃO pode apagar o caso especial](feedback_the_representation_can_delete_the_special_case.md) — fallback/wrap/verruga da referência eram artefato do `IndexRange` dela
- [Blindagem — Fase 0](project_blindagem_phase0_2026_06_20.md) — mede ESTRUTURAL, não comportamental; `ph2d-ui-testkit`
- [Pintura VOLTOU](project_painter_brush_came_back_cleanroom.md) = [clean-room do Blender Texture Paint](project_blender_texture_paint_reference.md) (GPL = só comportamento) + [Texture Layer raster-backed](project_texture_layer_design.md)
- [Norte node-centric](project_node_centric_decision_2026_05_21.md) — sistema de nós multi-domínio; FBP = unidade
- [Motion keyframes adiados](project_motion_keyframes_deferred_timeline_integration.md) — pesquisa pré-impl preservada
- [Vector cutover ADR-0108](project_vector_cutover_adr0108.md) — Rive-ref, GPU/editor-first; `ph2d-vec-*`
- [Flip = port 2D do Grease Pencil](project_flip_module_grease_pencil_2d.md); [traço = UNIÃO global](project_flip_stroke_analytic_coverage_gp.md) — autokey POR-TOOL; vizinhos geométricos, 1 passe
- [Composição de clips ≠ NLA](project_clip_composition_not_blender_nla.md) — no 2D o idioma é NESTING; overlap = crossfade
- [Modelo multi-agente = f(HW)](project_multiagent_modo_l_2026_07_05.md) — workstation = Modo L; constrained = Modo C
- [Modo L lento não é RAM, é disco + build 6×](project_modo_l_speed_hole_worktree_targets_slow_path.md) — tmpfs só no primário; sccache pequeno → recompila por worktree
- [Tool isolation ADR-0040](project_tool_isolation_freeze_2026_05_22.md) — tools = drop-crates + tool-sync
- [Nó consumido pelo renderer = Pure](project_node_effect_pure_for_renderer_consumed.md) — Boolean exato usa `linesweeper`
- [Não otimize prematuro](project_m5_perf_validated.md) — sprite renderer escala 100k @ 60Hz
- [Gates de velocidade](project_perf_audit_2026_05_19.md) — nextest 17→1.5min via `without_system_fonts()`
- [Perf do Painter (3 fatos)](reference_topic_painter_perf.md) — composite bandwidth-bound · cache o stamp · GPU compositor 1.7ms
- [Spatial GPU reconcilia vs CPU](project_painter_w4_spatial_gpu_bloom_sh.md) — Bloom/S-H bit-a-bit via dev-dep
- [Painter core files NO TETO](project_painter_core_files_at_loc_cap.md) — 600 LOC exatos; campo novo → orce split
- [8GB RAM = full-gate ~10min](project_solo_coord_backlog_ship_2026_05_29.md) — nextest-impacted teve false-green
