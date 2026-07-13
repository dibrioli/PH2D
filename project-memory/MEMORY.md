# Memory index — PH2D project (versionada em `project-memory/`, multi-máquina)

> Estado por-módulo em **CLAUDE.md §5**; contratos em **§6**; histórico em **git**/`docs/HANDOFF_*`.
> Aqui só LIÇÕES duráveis, perfil, facts não-deriváveis. Uma linha por memória; detalhe no arquivo.

## Perfil & referência
- [User: Enio (dibrioli)](user_role.md) — dono/decisor da PH2D; único dev de código é a LLM
- [Reference: canonical files & paths](reference_canonical_files.md) — onde estão SKILL, HANDOFF, ADRs
- [Reference: aquarela estado-da-arte](reference_watercolor_state_of_art.md) — motor = Curtis 1997 + capilar; K–M multi-pigmento #1
- [Reference: GPU tests headless (Metal)](reference_gpu_tests_run_headless_metal.md) — `cargo test --features gpu -- --ignored` roda no sandbox; só pen-input precisa do Enio

## Comunicação & decisão
- [Decida, não pergunte](feedback_decide_dont_ask_gold_standard.md) — decida no padrão-ouro e execute, reporte a decisão
- [Estilo](feedback_communication_style.md) — pt-BR direto, opções concretas, recomendação primeiro
- ["Difícil de ajustar" = bug de DESIGN](feedback_ergonomics_verdict_is_a_design_bug.md) — pare de calibrar; questione o modelo
- [Affordance herdada por analogia](feedback_inherited_affordance_must_be_rederived.md) — tinta é substância, sculpt é operação; um gate verde pode pinar um bug de DESIGN
- [Simplicidade](feedback_communication_simplicity.md) — sem AskUserQuestion-spam; não antecipe decisões
- [Comando de rodar inclui o `cd`](feedback_run_command_include_cd.md) — smoke/`cargo run` com `cd <worktree> &&` junto
- [Exemplo pronto pra smoke](feedback_ready_to_smoke_example.md) — feature nova = exemplo auto-play no doc demo; não peça pro Enio montar
- [Perfeição sem adiamentos](feedback_perfection_no_deferrals.md) — gaps in-scope fecham na sessão; padrão-ouro vence cronograma
- [Painter: 4 causas + DIRETIVA](feedback_painter_inefficiency_4_causes.md) — costura não-testada/audit=compilar/isolamento órfão/alvo irrefutável
- [Comentário velho / código morto MENTEM](feedback_stale_comment_and_dead_code_lie.md) — se removeu a UI remova o encanamento; nunca aja só pelo comentário
- [Cerca de Chesterton](feedback_documented_decision_chesterton_fence.md) — comentário "intentionally NOT X" = decisão ratificada; não sobrescrever
- [Convenção vs inércia](feedback_convention_vs_inertia.md) — checar se "convenção" tem gate ou é inércia; default = mais isolamento

## Git & colisão multi-agente
> **Modo C** (Mac): colisão de git real. **Modo L** (workstation): worktree próprio → só conflitos de merge (§1.5.5).
- [Commit collision paralelo](feedback_parallel_agent_collision.md) — `git status` antes de stage; não agarrar staged alheio
- [Scoped commit índice compartilhado](feedback_scoped_commit_shared_index.md) — `git commit -m msg -- <meus paths>`; `git add` específico
- [Git destrutivo fora da pasta](feedback_destructive_git_outside_pasta.md) — nunca reset/checkout/restore em paths alheios sem coordenar
- [Reset alheio apaga WIP](feedback_destructive_reset_collision_2026_05_28.md) — `git add -- <meus paths>` cedo cria fence (sobrevive a reset)
- [git stash multiagente](feedback_git_stash_multiagent_danger.md) — stash pop com índice sujo injeta conflict markers em arquivo alheio
- [Desfazer mutação com `cp`, nunca `git checkout`](feedback_mutation_undo_with_cp_never_git_checkout.md) — o checkout apaga a feature junto e o gate "passa"; 3× na linha do Painter
- [cargo fmt -p reformata WIP alheio](feedback_cargo_fmt_p_reformats_foreign_wip.md) — formata a crate inteira; use `rustfmt <meus arquivos>`
- [Worktree agent stale base](feedback_worktree_agent_stale_base.md) — `Agent(worktree)` ramifica do HEAD de início; só audit read-only
- [`str.replace()` que não casa é no-op silencioso](feedback_python_replace_silent_noop_after_fmt.md) — `fmt` reflowa o texto entre edições; `assert old in s` sempre
- [`sed -i` relativo escreve no repo errado](feedback_sed_relative_path_hits_primary_cwd.md) — mutação SEMPRE por caminho absoluto (Modo L: senão edita o `main`)
- [perl/sed em arquivo UTF-8 = mojibake](feedback_perl_utf8_mojibake_use_edit_tool.md) — literal não-ASCII no `-e` corrompe o arquivo inteiro; texto acentuado só via Edit tool

## Ship / CI / cadência
- [Multi-máquina Mac/Linux/Windows](project_multi_machine_setup.md) — GitHub = fonte única, clone local; memória em `project-memory/` via symlink
- [Fast mode / ship](feedback_fast_mode_ship.md) — dia: `git commit --no-verify` sem push; fim: `ship.sh` + push + babysit
- [Ship = Enio-only, fim de TODAS as linhas](feedback_ship_only_enio_end_of_all_lines.md) — nunca ofereça/rode ship no fim da SUA linha
- [Integração = Enio-only, via integrador](feedback_integration_only_enio_command_end_of_all_lines.md) — Modo L: NÃO integre/shippe; feche → handoff (§1.5.9) → PARE
- [Integrador: ship drena latentes — 2-4 iterações](project_integrator_ship_catches_latents_budget_iterations.md) — gate per-linha NÃO roda fmt/clippy-all/machete/deny; só ship vermelha
- [Pipe mascara o exit code do script](feedback_pipe_masks_script_exit_code.md) — `| grep` faz `$?` virar o do grep; ship/integrate falha e você lê 0. Verifique o ESTADO
- [Crase na msg de commit = substituição de comando](feedback_backticks_in_commit_message_are_command_substitution.md) — fish/zsh EXECUTA e a palavra some em silêncio; use `git commit -F <arquivo>` e releia o log
- [Merge limpo no texto pode estar quebrado por dentro](feedback_clean_text_merge_can_be_semantically_broken.md) — uma linha remove o símbolo, a outra o usa; `merge-tree` passa e a árvore não compila. Só o `check --workspace` cruza
- [Resolva pelos ESTÁGIOS do índice, não pelos marcadores](feedback_resolve_conflicts_from_index_stages_not_markers.md) — `:1`base/`:2`ours/`:3`theirs; Mergiraf emite 2 vias sem base. Portão anti-marcador antes do `git add`
- [Varra marcadores de conflito em CADA commit](feedback_sweep_conflict_markers_every_commit.md) — `<<<<<<< HEAD` órfão commitado não compila; a árvore limpa não prova o histórico
- [Foundational editável — crie com isolamento](feedback_foundational_editable_design_for_isolation.md) — Modo L PODE tocar; ao CRIAR projete p/ isolamento + anote ids no handoff
- [CI direto + fmt-skew](feedback_ci_direct_lint_gates_and_fmt_skew.md) — lint gates local antes; `cargo fmt` plain = skew, use `rustup run <pin> cargo fmt`
- [Ship committed vs WIP alheio](feedback_ship_committed_vs_worktree_wip.md) — valide o committed via `git worktree --detach HEAD`, sem tocar WIP
- [CI handling](feedback_ci_handling.md) — Enio confere visual; forneça link da run, não fique em polling
- [CI batching em waves](feedback_ci_batching.md) — acumular commits locais; push único no fim da wave
- [Commit cadence](feedback_commit_cadence.md) — não commitar a cada fix; acumular em blocos
- [Smoke no fim](feedback_smoke_at_end.md) — smoke 1× no fim de TODA a implementação
- [Refactor workflow](feedback_refactor_workflow.md) — commits locais; Enio testa manual antes de push/PR/CI
- [Phase cascade](feedback_phase_cascade_2026_05_19.md) — cada fase fecha + handoff + spawna próxima; última faz PR+CI
- [Codificação rápida](feedback_codificacao_rapida.md) — `cargo check/test -p <crate>`, não `--workspace`
- [Pre-commit arch gates](feedback_precommit_arch_gates.md) — arch-gate do crate antes de commit estrutural; `git commit` em background
- [LOC cap = split, não allowlist](feedback_loc_cap_split_not_allowlist_and_fmt_reexpands.md) — extraia módulo-irmão; `fmt` re-expande → fmt ANTES de medir
- [Full gate periodicamente](feedback_full_gate_periodically.md) — ship.sh/nextest na wave; re-lock cook hash ao mudar serialização
- [Ship-prep no-fail-fast](feedback_ship_prep_no_fail_fast.md) — `nextest --no-fail-fast` enumera TODAS as falhas; ship.sh é fail-fast
- [CI cold-build = @stable rust-cache drift](project_ci_rustcache_stable_drift_pin.md) — `@stable` rotaciona rustc-hash → cache bust; pin `@1.95`
- [ship.sh ≠ 100% paridade CI](feedback_ship_parity_gaps_ci_only.md) — bindgen/advisory-db/nextest-impacted escapam do ship; cutover quebra impacted
- [Números que SOMAM entre linhas: conte, não escolha](feedback_numbers_that_sum_across_lines_count_dont_pick.md) — registry/LOC-cap/schema: o valor certo não existe em nenhum lado do conflito. Prove com o teste
- [Allowlist duplicada mata o gate no PARSE](feedback_duplicate_allowlist_key_kills_the_gate_at_parse.md) — união duplica chave, TOML morre, o typos nem escaneia (e esconde erro real embaixo)
- [Integrar pré-cutover = drift de ship](project_integration_prefork_lines_ship_drift.md) — foundational-integrate NÃO roda fmt/typos; rode ship completo no fechamento

## Auditoria
- [Menu "não faz nada" = falta populate](feedback_context_menu_closes_on_down_repaint.md) — grep o id no `populate_*` PRIMEIRO; close-on-Down = red herring
- [Meça a ESCALA do sintoma antes da causa](feedback_measure_perf_symptom_scale.md) — fixe o nº (ms); frame(4-16ms) vs ⅓s muda a classe; bench-verde≠vivo
- [Harness reproduz mecanismo, não contexto](feedback_harness_reproduces_mechanism_not_context.md) — smoke contradiz fix ⇒ instrumente o guard no app real
- [1º caso salvo por efeito colateral](feedback_first_case_rescued_by_side_effect_test_repetition.md) — fixture de 1 traço não continha o bug do 2º; teste a REPETIÇÃO, no ritmo real do app
- [Não-reprodução ≠ correção](feedback_nonreproduction_is_not_proof_of_fix.md) — bug intermitente que some segue VIVO; cheque o `git diff` antes de aceitar "resolveu"
- [Unit-verde ≠ funciona no produto](feedback_tool_unit_green_integration_dead.md) — tool passa unit+CI e está morta (pill/input não wirado); só audit e2e pega
- [Um clique é um press que DESLIZOU](feedback_a_click_is_a_press_that_drifted.md) — mão humana move 1px; Down/Up na mesma coord é robô, não teste
- [Pintado ≠ populado: teste a PINTURA](feedback_painted_is_not_populated_paint_gate.md) — nenhum gate rodava `paint`; "o botão não existe" passava em tudo
- [Teste com os números do PRODUTO](feedback_test_with_product_numbers_not_convenient_ones.md) — `px_to_world = 1.0` é o único valor que esconde erro de unidade
- [Geometria sobre eixos de unidades diferentes](feedback_geometry_over_mixed_units_needs_the_consumers_conversion.md) — ângulo/normal só existe depois de converter; use a constante que o RENDERIZADOR usa
- [Coordenada derivada: seed = sample](feedback_derived_coordinate_seed_must_match_sample.md) — tempo remapeado quebrou 3×; todo caminho de autoria usa a MESMA transform do de leitura
- [Âncora invariante sob o que o usuário mexe](feedback_anchor_must_be_invariant_under_user_transforms.md) — geometria assada ancorada em APARÊNCIA (silhueta/borda) quebra com zoom-depois; ancore em geometria pura (eixo) e varra espessura×zoom
- [Gizmo errado = cheque o HIT](feedback_gizmo_verify_hit_target_before_transform_math.md) — logue o target resolvido no grab ANTES da math; era colisão de id
- [Overlay cortado na fronteira = cheque a ORDEM de draw](feedback_overlay_cut_at_boundary_check_draw_order.md) — mesma cena, draw posterior cobre; liste os writers ANTES de caçar clamp
- [Oráculo modela a APARÊNCIA, não o código](feedback_oracle_must_model_appearance_not_implementation.md) — oráculo derivado do shader fica verde com o bug na tela; esperado sai da definição do objeto
- [Gate verde contradito? RENDERIZE e olhe](feedback_render_and_look_when_a_green_gate_is_contradicted.md) — igualdade-de-conjuntos não vê névoa; o pixel é o oráculo, a métrica é sombra dele
- [Heurística exige TAXA de falso-positivo](feedback_heuristic_needs_false_positive_rate.md) — detector verde num fixture é sorte do seed; meça FP sobre 200 realizações de ruído
- [Tolerância folgada esconde viés sistemático](feedback_loose_oracle_hides_systematic_bias.md) — pitch shifter passou 3 jornadas 54 cents baixo; asserte o valor EXATO na unidade que o usuário ouve
- [Gateie as BORDAS do domínio](feedback_gate_the_edges_of_the_domain.md) — o miolo é onde não há bug; DC/Nyquist, 1ª/última coluna, 0 e 1 é onde o dado some
- [Mute o CÓDIGO, não só o teste](feedback_mutate_the_code_not_just_the_test.md) — gate verde na mutação = ou o gate é frouxo, ou o seu COMENTÁRIO está errado
- [Mutação que sobrevive pode ser gate FALTANDO](feedback_a_mutation_that_survives_may_mean_a_missing_gate.md) — 3ª causa: o gate verde está CERTO, só não fala daquilo. Explique por que ela é inofensiva ALI — a resposta nomeia o caminho sem gate
- [Gate de AUSÊNCIA precisa do irmão de PRESENÇA](feedback_absence_gate_needs_a_presence_sibling.md) — "a cor não vaza" fica verde com o fill INVISÍVEL; e varredura só vale se a coisa medida está em quadro
- [Regra que não OBSERVA não dispara](feedback_a_rule_that_never_observes_cannot_fire.md) — HR-13 somava declarações no boot; editor chegou a 4351 MB sem piscar
- [Faça a MESMA pergunta ao outro lado](feedback_ask_the_same_question_of_the_other_side.md) — gate do editor verde → mesmo gate no runtime nasceu VERMELHO (65,9 MB/música)
- [Mesma conta, escrituração diferente = 1 ulp](feedback_same_math_different_bookkeeping_diverges.md) — residente faz wrap do cursor, stream não fazia; gate byte-idêntico pega, "soa igual" não
- [Barra congelada vermelha? cheque a ARITMÉTICA dela](feedback_frozen_bar_check_the_arithmetic_before_gaming_it.md) — o piso pode já estourar a barra; e nunca passe pelo caminho que o produto não usa
- [Refactor mecânico largo = impressão digital](feedback_wide_mechanical_refactor_use_a_fingerprint.md) — digest antes/depois na MESMA máquina; golden pinado = mina no CI (ulp de transcendental)
- [Lens diversity](feedback_audit_lens_diversity.md) — rotacionar lentes; ≥2 paralelas; gates executáveis > claims verbais
- [Scope discipline](feedback_audit_scope_discipline.md) — bug em crate alheio = handoff pro owner, não fixo eu mesmo
- [No industrial claims](feedback_no_industrial_claims_without_verification.md) — zero claim técnico em ADR sem grep/cargo-search/WebFetch
- [Internal-state grep](feedback_audit_internal_state_grep.md) — sweep-grep de símbolos internos antes de escrever ADR
- [Commit-msg claim aging](feedback_audit_commit_msg_claim_verification.md) — claims numéricos envelhecem; framing relativo + literal do grep
- [Gate de emenda precisa de avanço fracionário](feedback_seam_gates_need_fractional_advance.md) — taxa 1:1 = frac 0, o 2º frame nunca é lido; frame segurado invisível
- [Determinism sweep grep](feedback_determinism_sweep_grep_all_transcendentals.md) — grepar `\.(sin|cos|tan|atan2|exp|sqrt|pow)\b`, não só `sin_cos`

## Padrões de código (gotchas silenciosos)
- [UI source of truth = gallery + inspector](feedback_ui_source_of_truth_gallery_inspector.md) — UI nova espelha widget-gallery + inspector; não improvisar chrome
- [UI em inglês](feedback_app_ui_english_only.md) — labels/toasts SEMPRE inglês; conferir AQUI antes de "traduzir" por HR-15
- [Nada de `→` em string literal](feedback_no_tofu_arrows_in_string_literals.md) — gate `no_tofu_glyphs`; `assert!`/`expect()` são strings
- [Tool nova exige IconId](feedback_new_tool_icon_needs_iconid.md) — SVG sem IconId variant (ordem alfabética) quebra TODOS os ícones
- [Fan-out registry-init](feedback_fanout_registry_init_friction.md) — tool-sync NÃO regenera os 2 testes hand-maintained (cluster order + icon slug)
- [node-sync glob prefix](feedback_node_sync_glob_prefix_gotcha.md) — crate na área de nós não pode começar com `ph2d-node-`
- [Hier companion allowlist](feedback_hier_companion_dispatch_allowlist.md) — bits novos em 2 sites de `pointer.rs` senão click dropado
- [Botão dimmed ainda despacha](feedback_disabled_button_still_dispatches.md) — dim é cosmético; não-registrar hit + recusar no event.rs
- [Panel populate register](feedback_panel_populate_register.md) — botão novo exige register em `populate.rs`; pintar + hit_index não basta
- [Painel docado = 4 sites](feedback_docked_panel_registration_four_sites.md) — crate+sync+EXPECTED · feature-proxy shell · z-order walk hero/paint.rs · visibility
- [Panel arch-gates + clamp/const](feedback_panel_arch_gates_scope_and_clamp_const.md) — no_magic_numeric+clamp escaneiam ph2d-panel-*/src; const de bound precisa `// CLAMP-OK`
- [Panel 2D-drag precisa dispatch](reference_panel_2d_drag_needs_dispatch.md) — 2D-livre = InteractiveState+dispatch (BlenderHit); Slider 1D é o único per-Move
- [NumberInput registra range](reference_number_input_register_range.md) — caixa LIMITADA chama `set_number_range(id,min,max,step)` senão drag escala errado
- [Pipeline inject, don't cap](feedback_pipeline_inject_dont_cap.md) — feature nova injeta no buffer do pipeline, não capeia o resultado
- [Pixel center vs edge](feedback_pixel_center_vs_edge_coord.md) — bilinear espera center; `(local/size+0.5)*W` é edge → subtrair 0.5
- [Exact-pin substring gate](feedback_exact_pin_needs_substring_gate.md) — `=version` pin precisa arch-gate substring senão rebasing "limpa"
- [ISPC cross-process](feedback_ispc_cross_process_concurrency.md) — asset-cooker ISPC crasha com cargo CONCORRENTE; um de cada vez
- [Zero-alloc gate = capacidade](feedback_zero_alloc_gate_capacity_not_global_counter.md) — dhat `total_blocks` é global → flaky; asserte CAPACIDADE dos buffers (e 1 `#[test]` por binário)
- [`Arc::from(Vec)` SEMPRE copia](reference_arc_from_vec_always_copies.md) — refcount inline; `collect::<Arc<[T]>>()` de TrustedLen aloca 1× sem unsafe (`Chain` NÃO é TrustedLen)
- [Visual bug debug](feedback_visual_bug_debug.md) — aritmética de pixels CEDO + simular visual + instrumentação >> leitura estática
- [Áudio: meter vivo, sem som = mute WirePlumber](project_audio_multichannel_silence.md) — NÃO é bug de código; `stream-properties` salva `mute:true`; fix = sed + restart
- [Claimed-green ≠ seu-OS-green](project_painter_t19_latent_red_macos_2026_05_28.md) — "W1 green" pode ser CI/linux; build o commit claimed-green ANTES
- [Painter "low-res" = canvas 64px](project_painter_canvas_res_64_not_sim_scale.md) — cheque a res do source ANTES do shader

## Arquitetura / norte / perf (duráveis, não-git)
- [Blindagem — Fase 0](project_blindagem_phase0_2026_06_20.md) — aparato mede ESTRUTURAL não COMPORTAMENTAL; Fase 0 = `ph2d-ui-testkit` seam headless + 3 gates
- [Pintura VOLTOU = clean-room Blender](project_painter_brush_came_back_cleanroom.md) — `ph2d-painter-brush` engine NOVO (Blender Texture Paint); confie no repo, não na nota "deletada"
- [Rebecca → PH2D Wet Paint clean-room](project_rebecca_watercolor_cleanroom.md) — rebecca era derivada do Rebelle (©Escape Motions), gitignorada; substituta em `docs/Painter/ph2d_wet_paint/`; fingering é arquitetura
- [Blender Texture Paint = referência](project_blender_texture_paint_reference.md) — recorte em `reference/blender-texture-paint/`; GPL = só clean-room
- [Texture Layer = raster-backed](project_texture_layer_design.md) — `LayerKind::Texture` pré-renderizado em `images[id]`; via `route_texture_layer_event`
- [Norte node-centric](project_node_centric_decision_2026_05_21.md) — engine = sistema de nós multi-domínio; `ph2d-nodegraph`+`ph2d-expr`; FBP = unidade multi-agente
- [Motion keyframes adiados p/ timeline](project_motion_keyframes_deferred_timeline_integration.md) — M2.W1 ADIADO 2026-07-09; pesquisa pré-impl preservada
- [Vector cutover ADR-0108](project_vector_cutover_adr0108.md) — módulo REPOSICIONADO (Rive-ref, GPU/editor-first); `ph2d-vec-*`+`ph2d-tool-vector`; icon-sort=slug, gate-doc FICA
- [Flip = port 2D do Grease Pencil](project_flip_module_grease_pencil_2d.md) — 4º meio (animação quadro-a-quadro); W0-W4 fechadas (2026-07-12): traço, frames/ghosts/tween, balde (âncora = EIXO da linha). Timeline global ADIADA. Autokey é POR-TOOL (borracha sempre duplica)
- [Flip: traço = UNIÃO GLOBAL da polilinha](project_flip_stroke_analytic_coverage_gp.md) — mordida MORTA (2026-07-12): janela p0/p3 + vizinhos GEOMÉTRICOS (broadphase no pack) + capsule_dn única + clamp/fade sub-pixel; 1 passe, 5 mutações provadas
- [Composição de clips ≠ NLA do Blender](project_clip_composition_not_blender_nla.md) — o Blender abandonou o próprio strip-stack; no 2D o idioma é NESTING. Overlap=crossfade; apply já é O(bindings²); TranslationX é absoluta (blend-to-default joga o sprite na origem)
- [Modelo multi-agente = função do HW](project_multiagent_modo_l_2026_07_05.md) — workstation=Modo L (worktree, sem coordenador); constrained=Modo C. ADR-0106/0107
- [Tool isolation ADR-0040 frozen](project_tool_isolation_freeze_2026_05_22.md) — tools = drop-crates + tool-sync codegen; tool nova = fan-out drop-in
- [Vector node carrier opaco](project_vector_node_opaque_carrier.md) — nós vetoriais emitem VectorNetwork via `CookValue::Opaque(Arc<dyn Any>)`
- [Brush bridge = satélite, não node](project_brush_along_path_satellite_not_node.md) — cross-module p/ 1 consumidor → crate satélite que só LÊ contratos; defira foundational até ≥2
- [Node renderer-consumido = Effect::Pure](project_node_effect_pure_for_renderer_consumed.md) — nó cozido pelo renderer DEVE ser Pure; Boolean exato usa `linesweeper`
- [Perf: não otimizar prematuro](project_m5_perf_validated.md) — sprite renderer escala 100k @ 60Hz Mac M-series
- [Perf: gates de velocidade](project_perf_audit_2026_05_19.md) — nextest 17→1.5min via `TextSystem::without_system_fonts()`; lld/ld-prime 1.5-3% macOS
- [Perf: composite bandwidth-bound](project_painter_w3_block2_persist_ktx2_2026_06_01.md) — 50×4K = 1.66GB/~70GB/s = 23ms; gate dirty-rect; postcard → depth-guard
- [Perf: textured-brush = cache o stamp](project_painter_texture_brush_stamp_cache.md) — FPS drop algorítmico (re-amostra falloff×tex); fix StampMask cacheado
- [Perf: painter preview → GPU compositor](project_painter_composite_perf_2026_06_03.md) — GPU `LayerOp::Adjustment` WGSL, Metal 1.7ms vs 55ms; medir em `--release`
- [Perf: fluid sim 4K = GPU-residente](project_painter_fluid_4k_perf_architecture.md) — hot loop O(grid) CPU + readback; alvo água GPU-residente + cs_splat
- [Watercolor v2 GPU-first](project_watercolor_v2_gpu_first_refactor.md) — pintura-lenta é submit/copy-bound; GPU-first single-submit/sparse; ADR-0085
- [Wash undo "mancha volta" = solver twin](project_wash_undo_event_driven_rebuild.md) — write parcial de ping-pong = atualize AMBOS gêmeos
- [Wash cor pigmento = Mixbox residual](project_wash_pigment_color_mixbox_residual.md) — c=unmix + residual r=rgb−mix(c); ADR-0091
- [Rendering Modes + Wet Mix](project_painter_rendering_modes_research.md) — Procreate Glaze/Blending/Wet+Burnt; design NÃO implementado
- [Painter W3 audit-2 + dirty-rect GPU](project_painter_w3_audit2_perf_2026_06_01.md) — 6-lens zero-critical + partial GPU upload + checked Q16.16; SMOKE pendente
- [Vector W7 WoS fora de budget](project_wos_diffusion_over_budget_2026_06_06.md) — GPU diffusion-curve CORRETO mas ~20-100× fora do budget; JBU low-res
- [Spatial GPU = reconcilia contra apply_*](project_painter_w4_spatial_gpu_bloom_sh.md) — Bloom/S-H GPU reconciliados bit-a-bit contra CPU via dev-dep
- [Panel LOC-gate parser bug — RESOLVIDO](project_panel_loc_gate_parser_masked_debt.md) — gate mentia p/ BAIXO (apóstrofo em `//`); fix comment-aware. Aberto: split das 14 fns
- [Painter core files NO TETO 600 LOC](project_painter_core_files_at_loc_cap.md) — paint/brush_settings/stroke/trait_impls exatos em 600; campo novo → orce split
- [KTX2 Basis rejeitado](project_ktx2_phase1_done_phase2_aborted_2026_05_26.md) — runtime Basis abortado; cooking offline nativo per-platform (ADR-0055-v4)
- [imageio AVIF *-sys deps](project_imageio_avif_pathc_2026_05_28.md) — libavif-sys precisa meson/nasm/cmake no CI; único format-crate sem `forbid(unsafe)`
- [8GB RAM = full-gate ~10min](project_solo_coord_backlog_ship_2026_05_29.md) — use `clippy --keep-going`; nextest-impacted teve false-green
- [Wash GPU-resident reimpl](project_wash_gpu_resident_reimpl.md) — reimplementar wash GPU-first; portar física B1-B9; zero fallback CPU
- [Wash → Curtis g/d](project_wash_curtis_gd_migration_2026_06_15.md) — divergiram por NÃO implementar Curtis; ADR-0095
- [Aquarela: Paper Colors ramp](project_aquarela_paper_ramp_broken.md) — REVERTIDA 2026-07-06 (papel volta ao grayscale); não reconstruir sem pedir
