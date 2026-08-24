# Memory index — PH2D (versionada em `project-memory/`, multi-máquina)

> Estado por-módulo: **CLAUDE.md §5**; contratos: **§6**; histórico: git/`docs/HANDOFF_*`.
> Aqui só lições duráveis, perfil, facts não-deriváveis. 1 linha/memória; famílias → `reference_topic_*` (2 saltos).

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
- [Apps à mão em `~/Apps` são invisíveis ao cachy-update (23/08)](reference_manual_apps_in_home_apps_are_invisible_to_cachy_update.md) — `pacman -Qo $(which app)` sem dono = instalação manual; Chrome migrado ao AUR, os outros 5 seguem fora
- [Atalho global no Plasma 6](reference_kde_plasma6_global_shortcut.md) — `[services][x.desktop]` + o grab que falta após o login
- [Extensão VSCode recusa bypass e edits sempre pedem em `default` (23/08)](reference_vscode_extension_refuses_bypass_and_edits_always_prompt_in_default.md) — allowlist não alcança o diff de aprovação; cura = 2 chaves `claudeCode.*` no settings DO VSCODE, por máquina
- [HISTÓRICO: aquarela/wash](reference_topic_watercolor_historical.md) — ADR-0096/0099/0108; 17 memórias da era

## Comunicação & decisão
- [Decida, não pergunte](feedback_decide_dont_ask_gold_standard.md) — padrão-ouro, execute, reporte
- [Os PRINCÍPIOS decidem, não o Enio](feedback_the_principles_decide_not_the_enio.md) — padrão-ouro · estado da arte · intuitivo para artistas · poderoso · fácil. Duas saídas medidas não é empate: é a régua à espera
- [«checksum» vermelho: o agente AGE, não escala (22/08)](feedback_a_red_checksum_is_acted_on_by_the_agent_not_escalated.md) — DIRETIVA_FIM_DE_DIA §6; ao Enio só o que exige reboot/senha, com o comando
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
- [A cwd do Bash VOLTA ao primário](feedback_bash_cwd_resets_and_slips_to_the_primary.md) — Modo L: prefixe todo comando com o `cd` da worktree (22/08: diagnostiquei no repo errado)
- [Exemplo pronto pra smoke](feedback_ready_to_smoke_example.md) — feature nova = auto-play
- [Perfeição sem adiamentos](feedback_perfection_no_deferrals.md) — gaps in-scope fecham na sessão
- [O teto é do HARDWARE](feedback_the_ceiling_is_the_hardwares_never_the_fallbacks.md) — meça antes de limitar
- [Medir um teto pode CONFIRMÁ-LO, e isso é o resultado](feedback_measuring_a_ceiling_can_confirm_it_and_that_is_the_result.md) — o entregável é a DERIVAÇÃO executável; e o recurso certo decide (alvo de ponteiro, não legibilidade)
- [Barra de RAZÃO aperta sozinha se o denominador é um knob](feedback_a_ratio_bar_tightens_itself_when_the_denominator_is_a_knob.md) — «atravessa a peça» é fração da peça; a razão triplicou sem defeito nenhum
- [Produto final, não MVP: params PRO por nó](feedback_final_product_every_node_ships_the_full_pro_param_set.md) — o superset do catálogo, conferido por nó (o miss da rotação)
- [Wave de pesquisa RECURSA](feedback_a_research_fanout_recurses_bound_it.md) — limite; verifique você o fato decisivo
- [Painter: 4 causas](feedback_painter_inefficiency_4_causes.md) — costura não-testada / audit=compilar / órfão
- [Comentário velho e código morto MENTEM](feedback_stale_comment_and_dead_code_lie.md)
- ["O design rejeita X"? grepe o gate](feedback_before_declaring_the_design_rejects_an_invariant_grep_for_its_gate.md)
- [Nota de diferido não é spec](feedback_a_deferral_notes_bar_may_exceed_the_projects_policy.md) — confira e corrija a nota
- [Coluna de sonda sem rótulo é lida ao contrário](feedback_an_unlabelled_probe_column_gets_read_backwards.md) — reportei «17 buracos» onde a linha dizia `0 bordo · 17 dobradas`; e quase culpei o instrumento certo
- [Pergunte que número a resposta CONTRÁRIA imprimiria](feedback_ask_what_number_the_opposite_answer_would_print.md) — **as três faces:** a régua tem de exprimir a resposta (ALCANCE: `90` não cabia numa grandeza limitada a `45`), sobre amostras que a contenham (EXTENSÃO: «não contrai» dito de UMA varredura), e não depender do que não importa (INVARIÂNCIA: a translação de uma costura é de calibre)
- [Cura medida numa fixtura que NÃO contém o fenômeno lê como inútil](feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless.md) — meça a fração alcançável ANTES do resultado; um zero pode ser implementação meio-feita
- ["NÃO toque neste arquivo" é uma AFIRMAÇÃO](feedback_a_handoff_can_be_wrong_about_its_own_dirty_file.md) — o handoff errou sobre a própria crate; meça antes de honrar
- [A regra tem de estar no CAMINHO de quem a executa](feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it.md) — doc órfão do roteador = regra inexistente
- [Ferramenta só é adotada se um PASSO a chama pelo nome](feedback_a_tool_is_adopted_only_when_a_written_step_names_it.md) — medido: 5 usos contra 13.791 do comando cru; ponteiro ≠ adoção
- [Arquivar sem indexar as RECUSAS é apagá-las](feedback_archiving_without_indexing_the_refusals_deletes_them.md) — e a cura de um doc inchado pode REALOCAR a doença; o teto se mede (80-110 KB)
- [Mecanismo certo, cura errada](feedback_a_correct_mechanism_can_prescribe_the_wrong_cure.md) — meça o mecanismo antes de construir o que a nota prescreve
- [Uma cerca DECLARADA escolhe a forma da própria cura](feedback_a_declared_fence_chooses_the_shape_of_its_own_cure.md) — a célula acerta no sintoma e não na forma; leia o doc-comment do knob de que ela se queixa
- [Recusa medida responde UMA pergunta](feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another.md) — reconfira se a sua é outra (velocidade≠escrita) ou se o substrato mudou (`noswap`); e churn≠escrita
- [Cerca de Chesterton](feedback_documented_decision_chesterton_fence.md) — "intentionally NOT X" = decisão
- [Revert pode diferir só no TEMPO DE VIDA](feedback_a_reverted_attempt_may_differ_only_in_lifetime_read_the_revert_reason.md) — leia o MOTIVO do revert, não o diff; escopo é o que mata tentativa boa
- [`match` exaustivo NÃO guarda a lista que um laço itera](feedback_an_exhaustive_match_does_not_guard_the_list_a_loop_iterates.md) — variante nova = braço morto sem warning; agulha com espaço nunca casa
- [Um nome de param da CASA carrega contrato](feedback_a_house_param_name_carries_a_contract_pick_another_word.md) — `channel` tem gate atrás; reusá-lo reprova sobre produto correcto, e a cura é o nome (irmã do `substeps`)
- [Convenção vs inércia](feedback_convention_vs_inertia.md) — tem gate? default = mais isolamento

## Git & colisão multi-agente
> Modo C (Mac): colisão real. Modo L (workstation): worktree próprio → só merge (§1.5.5).
- [Commit collision](feedback_parallel_agent_collision.md) + [scoped commit](feedback_scoped_commit_shared_index.md) — `git status` antes de stage; `git commit -m msg -- <meus paths>`
- [Perigos e armadilhas de git/edição (12)](reference_topic_git_hazards.md) — stash · reset alheio · fence · worktree-base · mojibake · `fmt -p` · `str.replace` no-op · `sed -i` relativo · rewrite de token · mover doc · mutação desfeita com `cp`
- [`Write` num caminho que já existe diz «updated», não «created»](feedback_write_on_an_existing_path_says_updated_not_created.md) — um nome BOM para o arquivo novo é o que o velho do mesmo assunto já tem; leia o verbo
- [O symlink da MEMÓRIA aponta para o primário](feedback_the_memory_symlink_points_at_the_primary_tree_not_your_worktree.md) — Modo L: salvar pelo caminho do Claude Code escreve no `main`

## Ship / CI / integração
- [Multi-máquina](project_multi_machine_setup.md) — GitHub fonte única; memória via symlink
- [Fast mode / ship](feedback_fast_mode_ship.md) — dia: commit sem push; fim: ship + babysit
- [Ship = Enio-only](feedback_ship_only_enio_end_of_all_lines.md) · [Integração = Enio-only](feedback_integration_only_enio_command_end_of_all_lines.md) — feche → handoff → PARE
- [Integração multi-linha (12)](reference_topic_integration_discipline.md) — ordem se MEDE · estágios `:1/:2/:3` · ⚠️ `checkout --ours` DESCARTA o que o git já fundiu · mesmo report = mesmo símbolo · números que SOMAM se contam · marcadores em CADA commit · o ship do integrador drena latentes (2-4 iterações)
- [✗ do ship pode ser AMBIENTE](feedback_a_ship_x_can_be_the_environment_not_the_code.md) — tmpfs evapora · disco cheio vira "linking failed"
- ["Está em uso?" → config GLOBAL](feedback_in_use_is_answered_by_the_global_config_and_a_probe_can_start_what_it_measures.md) — apaguei 101 GB de sccache ATIVO; e `sccache -s` SOBE o servidor que ele mede
- [Pipe mascara exit code](feedback_pipe_masks_script_exit_code.md) — verifique o ESTADO
- [Laço colável em idioma bash NÃO itera em zsh](feedback_a_pastable_bash_loop_never_iterates_under_zsh.md) — `for p in $VAR` roda 1× com a string inteira; portão que ENUMERA exige array citado + controle positivo
- [Crase em msg de commit executa](feedback_backticks_in_commit_message_are_command_substitution.md) — `git commit -F`
- [LOC cap = split](feedback_loc_cap_split_not_allowlist_and_fmt_reexpands.md) · [cap de FN ≠ cap de ARQUIVO](feedback_a_fn_cap_and_a_file_cap_measure_different_things.md) — fmt ANTES de medir; corte para o IRMÃO
- [Gate que VARRE uma árvore não é alcançado por filtro de nome](feedback_a_tree_scanning_gate_is_never_reached_by_a_name_filter.md) — ele vive onde a REGRA mora (LOC no shell, tofu no editor-core), não onde o arquivo mora; a suíte SEM filtro é o que o apanha
- [O clippy do fecho cobre TODA crate que a linha tocou](feedback_the_closing_clippy_must_cover_every_crate_the_line_touched.md) — alvo derivado do DIFF, nunca escrito à mão; um `-p` a dedo mede a minha memória e o integrador paga
- [Um vermelho de FLAKE esconde o resto da suíte](feedback_a_flake_red_hides_the_rest_of_the_suite.md) — o nextest cancela no 1º ✗ e deixou 1.007 por correr; leia o `X/Y tests run` antes de riscar o ✗
- [Cadência de processo + armadilhas de CI (17)](reference_topic_process_cadence.md) — gist em CLAUDE.md §2-§3 · fmt-skew · ship committed vs WIP · cold-build drift · paridade CI · `rustup default` · allowlist duplicada · seletor de impacto cego **duas vezes** (prefixo, e `A...` cego ao não-commitado). ⚠️ o babysit do CI É polling de 15 min (§3)

## Auditoria (famílias — 2 saltos)
- [Reprodução/diagnóstico (18)](reference_topic_repro_discipline.md) — harness/mecanismo · cursor real · não-repro ≠ fix · escala antes de causa · controle positivo
- [O 1º cruzamento de uma resposta RESSONANTE não é a fronteira](feedback_the_first_crossing_of_a_resonant_response_is_not_the_boundary.md) — prefixo-máximo; e imprima a varredura INTEIRA (o platô prova o instrumento)
- [Uma grelha uniforme não representa uma ESQUINA](feedback_a_uniform_grid_cannot_represent_a_corner.md) — leia a TAXA de convergência antes de aumentar o número; num degrau o que encolhe é a largura da banda
- [Réguas do quad remesh (13)](reference_topic_quad_remesh_rulers.md) — ⛔ suíte topológica cega a geometria · extremo global ≠ régua por-face · cura que falha delimita a causa · gate tautológico · régua que deduplica · `round` sem resíduo · invariante conservada · proveniência do defeito · provas de ótimo
- [Régua ancorada no MUNDO mede o gesto, não a forma](feedback_a_ruler_anchored_in_the_world_measures_the_gesture_not_the_shape.md) — um corpo que balança afasta-se de onde nasceu; ancore no centroide. E a extensão conta INTERVALOS, a contagem conta PONTOS
- [Generalizar lei de ÍNDICE pede espessura DERIVADA, não epsilon](feedback_generalising_an_index_law_needs_a_derived_thickness_not_an_epsilon.md) — sobre a grelha as duas concordam e a suíte fica verde; sobre um disco o epsilon prega UM ponto
- [Balde que ninguém enche lê-se como PERFEITO](feedback_a_bucket_nobody_fills_reads_as_perfect.md) — mediana de vector vazio = 0; ponha a CONTAGEM ao lado, e `else` em vez de `continue` quando a escrituração vive no fim do laço
- [Cura que MOVE o defeito nomeia-o](feedback_a_cure_that_moves_the_defect_names_it.md) — meça as duas pontas da fase; total igual + folga fechada = a fase foi isolada, não é fracasso
- [Melhorar a propriedade e PIORAR o produto refuta a premissa](feedback_a_better_instrument_can_make_the_product_worse_and_that_is_the_finding.md) — meça a régua da própria promessa; o mecanismo está na fase intermédia que a versão pior mascarava
- [Correlação perfeita sobre TODO o corpus ainda é N amostras](feedback_a_perfect_correlation_across_the_whole_corpus_is_still_n_samples.md) — derive o mecanismo; e se o efeito é «a fase seguinte recusa», a guarda É a fase seguinte
- [N fontes pedem a comparação CRUZADA](feedback_n_sources_need_the_cross_check_not_n_self_checks.md) — conferir cada fonte consigo própria passa sobre malhas diferentes; e um comentário que nomeia o risco não é o portão
- [Ofício de gate (32)](reference_topic_gate_discipline.md) — ausência+presença · razão doente · verde por acidente · paridade CPU/GPU · fixture contém o fenômeno
- [Modo SUPERSET ganha a dedup, nunca perde](feedback_a_superset_mode_must_win_the_dedup_never_lose_it.md) — desistir do modo rico apaga o que só ele desenha; a pergunta é qual CONTÉM o outro, não quem chegou primeiro
- [Estado autorado & relógios (19)](reference_topic_authored_state_and_clocks.md) — seed=sample · âncora · id-counter · load adota · ponto fixo · unidades mistas
- [As TRÊS perguntas de seam ficam verdes e a feature é inalcançável](feedback_the_three_ui_seam_questions_miss_the_fourth_the_sequence.md) — a quarta é a SEQUÊNCIA; e a cura de "não há rota" é a FACE VAZIA, nunca o desaparecimento
- [Pintar e agarrar projectam por UMA porta](feedback_paint_and_hit_test_must_project_through_one_door.md) — janela cheia na tinta e janela da cena no hit-test dá DOIS sintomas de uma causa (deslocado + não clica); a lei estava no módulo irmão, que eu li
- [Hit-test próprio herda a pergunta da REGIÃO](feedback_a_consumer_that_bypasses_the_hit_index_inherits_the_region_question.md) — sem o `on_canvas` o gizmo engolia cliques do painel do grafo, e o sintoma era «não consigo ligar um fio»
- [Blindar o hit-index MUDA o que as sondas medem](feedback_shielding_the_hit_index_changes_what_every_probe_measures.md) — o fundo do último hit-rect passa a saturar na janela: quem o usava como tamanho media a janela, não a coisa
- [Costura de UI (13)](reference_topic_ui_seam_discipline.md) — pintado/populado/clicado · duas portas · dimmed despacha · default é lei
- [Pintar e despachar têm de ler a MESMA fonte](feedback_paint_and_dispatch_must_read_the_same_source.md) — caixa que pinta do mundo e decide do store só diverge quando o MOTOR escreve o facto; sintoma é «às vezes, ao 2º clique»
- [Lista escrita à mão ao lado de um predicado = duas respostas](feedback_a_hand_written_list_beside_a_predicate_is_two_answers.md) — quem ENUMERA copia a lista de quem DECIDE; o diálogo oferecia 4 de 11 formatos há meses, e só uma extensão nova gerou report
- [«Acabou» lê-se igual a «foi pausado»](feedback_stopped_because_it_ended_reads_the_same_as_stopped_by_hand.md) — religar um transporte esgotado é gesto MORTO sem o predicado; e rebobinar tem de mover a IMAGEM
- [O seed é dono do VALOR, o dispatch do ESTADO](feedback_the_seed_owns_the_value_the_dispatch_owns_the_state.md) — espelho por-quadro REMENDA; `register` inteiro apaga o hover, e fica inerte até alguém dar cor ao estado
- [Desigualdade ≠ oráculo, e área somada ≠ REGIÃO](feedback_an_inequality_accepts_a_whole_interval_only_an_oracle_accepts_an_answer.md) — "o meio está entre as pontas" deixa passar produto errado; `Σ|área|` lê 400 e 272 para a MESMA região
- [Afirmação que mutação nenhuma mata é afirmação sobre NADA](feedback_a_claim_no_mutation_can_kill_is_a_claim_about_nothing.md) — encolha a afirmação até ao que a máquina faz; perseguir o gate antes de a reler custa duas voltas
- [Contar o trabalho FEITO não é contar o ENTREGUE](feedback_counting_the_work_done_is_not_counting_the_work_delivered.md) — o gate contava cozeduras e o consumidor recebia zero; ponha a sonda dentro de quem consome
- [Duas hipóteses boas que falham refutam a FAMÍLIA](feedback_two_good_hypotheses_failing_refutes_the_family_not_the_two.md) — ao 2.º falhanço pare de propor curas e construa a régua que LOCALIZA; ilibe também o suspeito improvável
- [Provas de mutação (7)](reference_topic_mutation_proofs.md) — ⚠️ os 3 controles vão NO ARNÊS: verde-antes · `Compiling <pkg>` · `running 1 test` (sem o 1º, um gate que rebenta sozinho lê como mutação apanhada)
- [Restore que preserva o mtime deixa o cargo STALE](feedback_a_mutation_restore_that_preserves_mtime_leaves_cargo_stale.md) — `shutil.copy2` devolve conteúdo E carimbo; a suíte fica vermelha acusando código já correto. Carimbe o restore e exija o verde de volta; ⛔ `git status` é cego dentro de uma crate `??`
- [Disciplina de oráculo (9)](reference_topic_oracle_discipline.md) — aparência, não regra
- [Disciplina de fixture (6)](reference_topic_fixture_discipline.md) — só prova o que contém; ordem de setup mascara bug de ordem
- [Protocolo de auditoria (6)](reference_topic_audit_protocol.md) — lentes · claims · state-grep
- [Espec limpa é MENOS específica que o paper de que descende](feedback_a_clean_spec_is_less_specific_than_the_paper_it_descends_from.md) — quem traduz código herda as constantes, quem descreve herda a lei; e audite por `grep` no texto da fonte, não pela memória
- [Documento de instruções pode mandar o que a CERCA do leitor proíbe](feedback_an_instruction_doc_can_order_what_its_readers_own_fence_forbids.md) — confronte-o com o passo 0 / as regras permanentes de quem executa, não só com o assunto; regra herdada de molde fala do mundo do molde
- [Física do impasto/sculpt (8)](reference_topic_impasto_physics.md)
- [O oráculo grava as FASES INTERMÉDIAS](feedback_the_oracle_writes_its_intermediate_stages_compare_phase_by_phase.md) — `ls` na saída dele antes de reimplementar; ler saída ≠ obra derivada
- [A peça que falta pode JÁ estar construída](feedback_the_missing_piece_may_already_be_built_measure_its_structure_first.md) — meça a estrutura do que já lá está; material produzido-e-ignorado não aparece em régua nenhuma
- [Laço de reparo pode ESCONDER o que agrava](feedback_a_repair_loop_can_hide_the_defect_it_worsens.md) — parar por «não há sinalizados» é critério sobre o DETECTOR; dê-lhe um invariante que ele não pode piorar
- [Guloso que explode com um termo novo pede SEMENTE](feedback_a_perturbation_that_breaks_a_greedy_needs_a_seed_not_a_smaller_perturbation.md) — não baixe o peso nem suavize o guia; olhe o estado sobre o qual ele toma a 1.ª decisão
- [Saída muda com TODOS os elos internos verdes = o elo está FORA do processo](feedback_a_silent_output_channel_can_be_muted_outside_the_process.md) — mute por-aplicação do PipeWire gravado pelo NOME da app; `cpal` não o vê, e o controlo positivo forte é que aponta para fora
- [Gate vermelho ao ligar algo novo? corra-o DESLIGADO](feedback_a_new_features_gate_can_expose_a_pre_existing_bug_check_the_control_first.md) — a feature perturba e muda QUAL caso cai; o defeito pode ser antigo e nunca medido
- [Tornar um nó elegível pode REGREDIR um claim parcial — RECUE, não refute inteiro](feedback_making_a_node_eligible_can_regress_a_partial_claim_retreat_dont_refuse_whole.md) — re-meça o doc REAL; regra tudo-ou-nada vira regressão; a cura é un-claim, não refutar o plano

## Padrões de código (gotchas silenciosos)
- [UI = gallery + inspector](feedback_ui_source_of_truth_gallery_inspector.md) — espelhe
- [UI em inglês](feedback_app_ui_english_only.md)
- [Nada de `→` em string literal](feedback_no_tofu_arrows_in_string_literals.md) — vale em mensagem de `assert!` de teste também (gate `no_tofu_glyphs`, mordeu 22/08)
- [Registro de painel (5 sites)](reference_topic_panel_registration.md)
- [Fonte pré-multiplicada INVERTE o único modo cujo neutro é 1](feedback_a_premultiplied_source_breaks_the_blend_whose_identity_is_one.md) — `Multiply` a α=0 pintava PRETO e subir a alfa clareava; e o gate media só α=1, o único ponto em que os seis modos concordam
- [Clone segurado + detecção por ponteiro = copy-on-write por op](feedback_a_held_clone_plus_pointer_identity_change_detection_forces_copy_on_write.md) — versão, não `as_ptr` (ADR-0124 reincide: Painter 10ms/move @4K)
- [Gotchas de código (13)](reference_topic_code_gotchas.md) — IconId · registry-init · node-sync glob · companion allowlist · inject don't cap · pixel center · exact-pin · ISPC · zero-alloc · `Arc::from` copia · áudio mudo · OS-green · low-res

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
