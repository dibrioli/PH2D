# Memory index — PH2D (versionada em `project-memory/`, multi-máquina)

> Estado por-módulo: **CLAUDE.md §5**; contratos: **§6**; histórico: git/`docs/HANDOFF_*`.
> Só lições duráveis, perfil, facts não-deriváveis. Uma linha por memória, CURTA; o detalhe vive no
> ficheiro; famílias e lições dobradas → `reference_topic_*` (2 saltos). Entrada nova longa vai para
> o `reference_topic_*` da secção, **nunca para aqui**.
>
> ⚠️ **O tecto é o que o CARREGADOR leva (medido 2026-09-17: `26 007` bytes; ele cortou o ficheiro
> na linha `158` de `224` sem avisar).** O recurso é o BYTE: alvo **≤ 17 KB**.
>
> ⛔⛔ **O que o estourou foi a FUSÃO de seis linhas** reescrevendo a MESMA entrada (`60` ponteiros
> sobre `51` ficheiros, zero linhas iguais byte a byte). ⇒ **um ponteiro por ficheiro**, e **a
> contagem de uma família CONTA-SE no ficheiro dela**.

## Perfil & referência
- ⛔⛔⛔ [Dono 05/11: lei nova ARQUIVADA, não apagar; produto na antiga](project_new_paint_law_for_all_brushes_2026_11_05.md)
- [Enio (dibrioli): dono/decisor; o dev é a LLM · ⛔ SEMPRE PT-BR](user_role.md)
- 📚 [Decisões de produto do dono, por módulo (21) — Luau, Pixel Art, Kanban, macros, constraints, imagem, modos…](reference_topic_owner_product_decisions.md)
- ⭐⭐ [Dono 02/11: UI em cartões — referência = os nossos painéis com smoke OK, depois Figma → Rive → Inkscape/Godot/Unity](project_card_ui_reference_order_2026_11_02.md)
- ⭐⭐ [Dono 04/11: cartão compacto como o Figma — irmãos dividem a linha, nada grudado, uma grelha](feedback_card_layout_is_compact_like_figma_2026_11_04.md)
- ⭐⭐ [Dono 04/11 noite: missão autónoma, fechar a linha Painter sem perguntas; smoke no dia seguinte](project_autonomous_close_of_painter_line_2026_11_04.md)
- ⭐⭐ [Dono 04/11: Smooth Edges por traço, um por pincel (revoga 27/10); Wet Liner e Wet Wash 0,12 + Edge Darkening 0](project_smooth_edges_per_brush_owner_decision_2026_11_04.md)
- ⭐⭐ [Dono 05/11: Pincel e Borracha com a sua Hardness; Smooth Edges fica fora do Smear/Blend/Water](project_hardness_per_tool_owner_decision_2026_11_05.md)
- ⭐⭐ [Smoke do dono 05/11: Smooth Edges, Grain, Hardness OK; decisões ficam; macro com Grain bugada FORA desta linha](project_smoke_2026_11_05_owner_verdicts.md)
- ⭐⭐⭐ [Dono 05/11: «adoro os pincéis como estão»: o app pinta com a lei VELHA e ela é a referência; furos não se perguntam mais](project_brushes_as_they_are_are_the_reference_2026_11_05.md)
- ⭐ [Bug #71 (tinta afundada em relevo): cura pronta mas DESLIGADA; o dono decide ligar `PH2D_PAPEL_SEM_DEGRAU` (05/11)](project_relief_step_awaits_owner_eye_2026_11_05.md)
- ⭐⭐ [Dono 01/11 (F43): Effects/Filters do Vector no layout Figma — «+» dropdown, lista, propriedades à ESQUERDA](project_vector_effects_filters_figma_layout_2026_11_01.md) · ⭐⭐ [F44: Fill e Stroke no MESMO padrão; Cap/Join/Align com ícones](project_vector_fill_stroke_figma_layout_2026_11_01.md) · ⭐⭐ [campo que não se explica sozinho leva rótulo](feedback_a_field_that_is_not_self_evident_gets_a_label.md) · [unidade no rótulo, nunca na caixa (iPad)](feedback_units_live_in_the_label_not_in_the_box.md)
- ⭐⭐ [E12 (05/11) NÃO feita: a régua de furos reprova quem passa a cobrir; não apague a lei velha antes de refazê-la](project_e12_blocked_by_mother_gate_ruler_2026_11_05.md)
- ⭐⭐ [Dono 05/11: Pixel — filtros ao vivo com Apply/Cancel, girar de volta devolve o original, só 12 fontes](project_pixel_text_and_filters_owner_decisions_2026_11_05.md)
- ⛔⛔ [Espera sem prazo dentro de job do rayon é deadlock: o operário rouba o job por cima do pedaço (Bug #66)](feedback_a_blocking_wait_inside_a_pool_job_is_a_deadlock.md)
- ⛔ [`fila-pesada.sh correr` corre TUDO: `ver` antes (05/11)](feedback_clear_the_heavy_queue_before_running_it.md)
- ⭐⭐ [Dono 03/11: teste de pincel = alarme BARATO que aponta o que mudou + o DONO decide (gizmo × traço); fim das tabelas de 40 tamanhos](project_brush_tests_cheap_alarm_owner_decides_2026_11_03.md)
- ⭐⭐ [Dono 04/11: o mesmo traço dá o mesmo byte em qualquer computador; o motor não chama a matemática do sistema](project_same_stroke_same_byte_on_any_machine_2026_11_04.md)
- ⭐⭐ [Dono 04/11: aprovou Marker Liner em Mix, Bokeh com a curva dele, Dabber 4 e Knife Stain — não voltar ao Multiply por paridade](project_brush_tweaks_owner_approved_2026_11_04.md)
- ⭐⭐ [Dono 03/11: manter a cura da cobertura mas devolver o molhado antigo (Wet Round/Drip); Blend > círculo](project_coverage_cure_must_keep_the_old_wetness_2026_11_03.md)
- ⭐⭐⭐ [Dono 31/10 noite: o Rebelle DEIXA de ser referência — o Painter busca a própria cara (física + consistência + olho do dono)](project_rebelle_no_longer_the_reference_2026_10_31.md) · antes: [superar o Rebelle](project_superar_rebelle_autonomous_2026_10_31.md) · [02/11: Wet Liner sem fio escuro; cena macro 1 fica](project_wet_liner_no_dark_edge_2026_11_02.md)
- ⭐⭐⭐ [Dono 01/11: óleo diluído escorre — UMA lei, sem 2.º líquido; Medium → seletor «Paint type: Water · Oil»](project_paint_type_water_oil_one_law_2026_11_01.md)
- ⭐⭐ [Dono 31/10 noite: refatorar o Wet Paint por áreas (grandeza física, uma lei) + ajuda ilustrada estilo Fusion 360](project_wetpaint_refactor_and_illustrated_help_2026_10_31.md)
- ⭐⭐ [Auto Key grava ABSOLUTAMENTE TUDO (19/10): cor, traço, pontos — não só transformações](project_autokey_records_everything_2026_10_19.md)
- [Onde mora cada coisa — tabela verificada](reference_canonical_files.md)
- [transcripts medem o agente — agent-loop-profile.sh](reference_session_transcripts_are_a_measurable_instrument.md)
- ⛔ [meters mexem e não ouço = mute no WirePlumber; `scripts/audio-mudo.sh`](project_audio_multichannel_silence.md)
- ⛔ [Caneta Wacom no Wayland: KWin NÃO emula mouse p/ cliente sem tablet-v2; XWayland clica (sem pressão)](project_wacom_pen_needs_tablet_v2_on_kwin_wayland.md)
- ⛔ [Piso de pressão para casar o Rebelle tirou a resposta da mão do dono — revogado 18/10](feedback_pen_pressure_floor_must_keep_the_owners_hand_range.md)
- 📚 [Apps instalados, ambiente da máquina e referências (16)](reference_topic_installed_apps.md)
- [HISTÓRICO: aquarela/wash e 3D (20)](reference_topic_watercolor_historical.md)
- ⭐ [Dono 28/10: a tinta que AMASSA o relevo do papel é controlo do pincel (0 segue · 1 amassa), nunca apagar](project_paper_press_is_a_brush_control_2026_10_28.md)

## Comunicação & decisão
- ⭐⭐⭐ [Painter (31/10): Pigmento + Água = só a DILUIÇÃO; o VOLUME ao papel é outro controlo — um controlo, uma grandeza](project_painter_volume_is_independent_of_dilution_2026_10_31.md)
- ⛔⛔⛔ [O APP é SÓ em inglês — toda string de UI, e a ajuda/tooltip em Plain English para qualquer país (31/10)](feedback_app_ui_english_only.md)
- ⛔⛔ [Hora de trocar de janela = prompt de continuação COMPLETO sem o dono pedir — handoff e prompt só DEPOIS do smoke (25/10)](feedback_always_hand_the_full_continuation_prompt.md)
- ⭐ [Decisões TÉCNICAS são delegadas (padrão-ouro); produto e envio são do Enio](feedback_architecture_decisions_are_delegated_to_the_gold_standard.md)
- ⛔ [A linha acaba no HANDOFF — ordem das linhas e árvore suja são do INTEGRADOR](feedback_the_line_ends_at_the_handoff_never_ask_the_owner_an_integrators_question.md)
- ⭐ [**Como um CONTROLO mente** — a família (25)](reference_topic_control_design_hazards.md)
- ⛔ [O produto de referência do DONO decide a lei, não o oráculo; capturas dele MEDEM-SE](feedback_the_owners_reference_product_decides_the_law.md)
- ⭐⭐ [Chave e texto do MESMO tipo é defeito à espera](feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting.md)
- ⛔⛔⛔ [Smoke do Motion = FORMAS + SIMULAÇÃO; fotografe antes](feedback_a_motion_smoke_scene_needs_shapes_and_simulation.md)
- ⛔ [Smoke ao dono explica o que cada coisa na tela É; cada passo conduzido e fotografado](feedback_a_smoke_for_the_owner_explains_what_each_thing_on_screen_is.md)
- ⛔ [Imagem ao dono = link CLICÁVEL em markdown, nunca caminho em texto (30/10)](feedback_images_for_the_owner_are_clickable_links.md)
- ⛔⛔ [NUNCA `spectacle` em `kwin --virtual` (fotografa o ecrã REAL); `import -window`](feedback_spectacle_in_a_virtual_kwin_photographs_the_owners_real_screen.md)
- ⛔ [Só o Wet Paint evolui (07/10): Digital/Impasto/Watercolor ficam](project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07.md)
- ⭐ [Óleo Real (26/10): o mais importante é relevo, shader, luz e sombra realistas](project_oleo_real_priority_relief_shader_light_2026_10_26.md)
- ⭐⭐ [MISSÃO 19/10: melhores pincéis do mundo — ENCERRADA 22/10: 18 aprovados, fábrica 45, o resto trava em motor](project_the_worlds_best_brushes_mission_2026_10_19.md)
- ⛔⛔ [Missão autónoma NUNCA pára num pedido de permissão: hook allow + formas proibidas no briefing (20/10)](feedback_an_autonomous_mission_must_never_stop_on_a_permission_prompt.md)
- ⛔⛔ [Missão de pincéis (28/10): o DONO faz a triagem, SIMULA-SE o conserto antes de construir, 45 min/agente, pesado 1× no fim](feedback_brush_method_owner_triage_and_simulate_before_building.md)
- ⛔⛔ [Passar a régua do recto não é ser um pincel: Filbert «nada novo», lâminas «inúteis» (P53)](feedback_a_brush_that_passes_the_ruler_can_still_be_useless.md)
- ⛔ **Nada armengado: SILENCIAR um diagnóstico é armengo** ([§SUPRESSÃO, 29/08](feedback_perfection_no_deferrals.md)) — meio-feito é pior que não começar
- ⭐⭐ [Máximo AMOSTRADO que vira limite de segurança erra sempre PARA BAIXO](feedback_a_sampled_maximum_that_becomes_a_safety_bound_errs_only_downwards.md)
- 📚 [Comunicação & decisão: mais (90) lições, uma linha cada, verbatim](reference_topic_communication_and_decision_lessons.md)

## Git & colisão multi-agente
- [Perigos de git/edição (15)](reference_topic_git_hazards.md)
- ⛔⛔ [`git commit -- <paths>` ignora ficheiros por RASTREAR: o commit «tem sucesso» e a árvore dele não compila](feedback_a_commit_with_paths_never_picks_up_an_untracked_file.md)
- 📚 [Git & colisão multi-agente: mais (5) lições, uma linha cada, verbatim](reference_topic_git_lessons_loose.md)

## Ship / CI / integração
- ⛔⛔ [Regra de BRIEFING sem medição é «confirmada» por quem a segue](feedback_an_unmeasured_rule_in_a_briefing_is_confirmed_by_the_agents_who_follow_it.md)
- ⭐ [CI 04/11: 27–30 min com cache · semente `gh workflow run spike.yml --ref main` após o push · teto de CPU 50 vs 100 é do DONO](project_ci_acceleration_2026_11_04.md)
- ⛔ [A régua de uma linha é o MERGE-BASE, nunca o main a andar](feedback_the_ruler_is_the_merge_base_not_a_moving_main.md)
- [Integração multi-linha (15) — ordem se MEDE](reference_topic_integration_discipline.md)
- ⛔⛔⛔ [Partilhar `CARGO_TARGET_DIR` entre worktrees TROCA os `.rlib` — o erro acusa o código e a medição vale ZERO](feedback_sharing_a_target_dir_between_worktrees_corrupts_the_build.md)
- ⛔⛔⛔ [`kill` do PAI de um órfão = `systemd --user` = LOGOUT do dono (05/10)](feedback_killing_the_parent_of_an_orphan_kills_the_owners_session.md)
- [Cadência de processo + CI (19)](reference_topic_process_cadence.md)
- ⛔⛔ [Dono 23/10 e 25/10: a suíte COMPLETA só antes da INTEGRAÇÃO; na linha só testes por nome](feedback_full_suite_only_before_integration.md)
- ⛔ [Selo «árvore já passou» no PRIMÁRIO casa pelo HEAD^{tree}, nunca pela pasta (29/10)](feedback_a_tree_key_on_the_primary_must_be_the_head_tree.md)
- 📚 [Ship / CI / integração: mais (42) lições — ⛔ a suíte inteira 1× por linha](reference_topic_ship_ci_integration_lessons.md)

## Auditoria (famílias — 2 saltos)
- ⛔⛔ [Contar operações ≠ perfil; o PERFIL DE BUILD decide o número](feedback_an_operation_count_is_not_a_profile_and_the_build_profile_decides_the_number.md)
- ⛔⛔⛔ [Régua de calma que não nomeia o PERFIL: a leitura a 98 % ociosa foi a PIOR das quatro — era debug](feedback_a_calm_ruler_that_does_not_name_the_build_profile_reads_as_a_verdict.md)
- [Reprodução/diagnóstico (26) — ⛔ onde um defeito ATERRA ≠ o que o PROVOCA](reference_topic_repro_discipline.md)
- ⛔⛔ [Fixtura ALINHADA AOS EIXOS não mede uma BASE](feedback_an_axis_aligned_fixture_cannot_measure_a_basis.md)
- ⛔⛔ [Campo que varia sob consumidor com TAMANHO mede-se a esse tamanho](feedback_a_field_under_an_extent_is_measured_at_that_extent.md)
- ⛔⛔ [Modo caro com orçamento GLOBAL fica inerte na cena cheia e pode continuar a PAGAR](feedback_an_inert_mode_can_still_pay_its_full_price.md)
- [Estado autorado & relógios (21)](reference_topic_authored_state_and_clocks.md) · [modo que não exclui ninguém é outro produtor](feedback_a_mode_whose_entry_excludes_nobody_is_just_another_producer.md)
- [Oráculo (27) — paridade da LEI ≠ o PRODUTO do dono](reference_topic_oracle_discipline.md)
- ⛔⛔ [A chave que o ARTISTA fez manda mais que a correção automática](feedback_the_artists_key_outranks_the_automatic_correction.md)
- ⛔⛔ [A animação de DEMO de um app mostra a ferramenta, não a lei](feedback_a_vendors_demo_fixture_demonstrates_the_tool_not_the_truth.md)
- ⛔⛔ [Nenhum portão pergunta se um GESTO é REVERSÍVEL](feedback_a_gesture_that_returns_must_give_back_the_pose.md)
- ⛔⛔ [Feature PIOR que não existir — só a linha de CONTROLE mostra](feature_worse_than_not_existing.md)
- [Física do impasto/sculpt (10)](reference_topic_impasto_physics.md)
- ⭐ [**Como uma RÉGUA mente** — a família (222)](reference_topic_measurement_discipline.md)
- ⛔ [Vermelho bimodal: 6/6 no main era sorte e outro build; 12×12 intercalado decide](feedback_a_bimodal_red_needs_interleaved_samples_with_the_same_build.md)
- ⛔⛔⛔ [Uma régua enviesada manda a wave seguinte para onde ela própria está errada](feedback_a_biased_ruler_sends_work_to_where_it_is_itself_wrong.md)
- ⛔⛔⛔ [Régua que pára no MUNDO aprova um clique partido no ECRÃ (340 px: o chrome desenha numa BANDA)](feedback_a_ruler_that_stops_at_world_space_approves_a_broken_click.md)
- [Costura de UI (26)](reference_topic_ui_seam_discipline.md) — ⭐ [o hit rect é também o DENOMINADOR](feedback_a_hit_rect_is_also_the_denominator_not_only_the_target.md)
- [**Ofício de gate** — a família (141)](reference_topic_gate_discipline.md)
- [Protocolo de auditoria (9) — ⛔ num artefacto ATESTADO toda linha nova é população nova](reference_topic_audit_protocol.md)
- 📚 [Auditoria (famílias — 2 saltos): mais (56) lições](reference_topic_audit_lessons.md)

## Padrões de código (gotchas silenciosos)
- ⛔⛔ [Cura de CONTRASTE carrega a SUPERFÍCIE contra a qual foi calibrada](feedback_a_contrast_cure_carries_the_surface_it_was_calibrated_against.md)
- ⛔⛔ [Mudar a ARITMÉTICA de um widget partilhado varre-se por CONSUMIDOR](feedback_changing_a_shared_widgets_arithmetic_is_swept_by_consumer_not_by_call_site.md)
- ⛔⛔⛔ [Impor uma propriedade DEPOIS de um óptimo por-peça é o que ONDULA — antes de forçar suavidade, meça se a VERDADE a tem](feedback_forcing_a_property_after_the_optimum_is_what_ripples.md)
- [Registro de painel — (6) sítios](reference_topic_panel_registration.md) · [Gotchas de código (36)](reference_topic_code_gotchas.md)
- 📚 [Padrões de código (gotchas silenciosos): mais (58) lições, uma linha cada, verbatim](reference_topic_code_pattern_gotchas.md)

## Arquitetura / norte / perf
- ⛔ [REGRA DO DONO: editar pixels ou tamanho/margem = imagem SEM a dobra (exceto Liquify, cor, filtros)](project_pixel_tools_flatten_bone_bound_art.md)
- ⭐⭐⭐ [Porta de tipo LARGO: uma cena ensina o idioma, não basta compilar](feedback_a_port_that_accepts_anything_still_teaches_what_you_plug_into_it.md)
- ⭐⭐ [Partilhar um nó para igualar entradas torna o grafo ilegível](feedback_a_correct_property_can_cost_the_legibility_that_was_the_whole_point.md)
- ⭐⭐ [**Dinâmica dos ciclos** do Motion: params no cartão; o smoke é o TUTORIAL em PDF](project_motion_cycle_dynamic.md)
- [Perf do Painter (5)](reference_topic_painter_perf.md) · [spatial GPU reconcilia vs CPU](project_painter_w4_spatial_gpu_bloom_sh.md)
- 📚 [Arquitetura / norte / perf: mais (29) lições — caminho DERIVADO × do ARTISTA · ASSAR (7,7×)](reference_topic_architecture_north_perf_lessons.md)
- [fixture (19)](reference_topic_fixture_discipline.md)
- [Provas de mutação (43) — 4 controlos, pré-voo das âncoras · cercas em SEQUÊNCIA](reference_topic_mutation_proofs.md)
- ⭐ [Dono 05/11 noite: próxima fase Painter (redo · mescla aguada/óleo · travamento · Clip Studio); Watercolor Basic NÃO muda](project_next_phase_painter_2026_11_06.md)
