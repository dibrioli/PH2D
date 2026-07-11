# Memory index — PH2D project (versionada em `project-memory/`, multi-máquina)

> Estado por-módulo (waves/tasks) vive em **CLAUDE.md §5**; contratos congelados em **§6**;
> histórico em **git** + `docs/HANDOFF_*`/`docs/AUDIT_*`. Aqui só LIÇÕES duráveis,
> perfil, e facts não-deriváveis. Uma linha curta por memória; detalhe no arquivo.

## Perfil & referência
- [User: Enio (dibrioli)](user_role.md) — dono/decisor da PH2D; único dev de código é a LLM
- [Reference: canonical files & paths](reference_canonical_files.md) — onde estão SKILL, HANDOFF, ADRs
- [Reference: aquarela estado-da-arte](reference_watercolor_state_of_art.md) — motor = Curtis 1997 + capilar; reading list (K–M multi-pigmento #1)
- [Reference: GPU tests headless (Metal)](reference_gpu_tests_run_headless_metal.md) — `cargo test --features gpu -- --ignored` roda no sandbox; só pen-input precisa do Enio

## Comunicação & decisão
- [Decida, não pergunte](feedback_decide_dont_ask_gold_standard.md) — decida no padrão-ouro e execute, reporte a decisão
- [Estilo](feedback_communication_style.md) — pt-BR direto, opções concretas, recomendação primeiro
- [Simplicidade](feedback_communication_simplicity.md) — direto; sem AskUserQuestion-spam; não antecipe decisões
- [Comando de rodar inclui o `cd`](feedback_run_command_include_cd.md) — smoke/`cargo run` sempre com `cd <worktree> &&` junto, copiável de uma vez
- [Exemplo pronto pra smoke](feedback_ready_to_smoke_example.md) — feature nova = autore o exemplo no documento default/demo (auto-play); nunca peça pro Enio montar à mão
- [Perfeição sem adiamentos](feedback_perfection_no_deferrals.md) — gaps in-scope fecham na sessão; padrão-ouro vence cronograma
- [Painter: 4 causas + DIRETIVA por-etapa](feedback_painter_inefficiency_4_causes.md) — costura não-testada/audit=compilar/isolamento órfão/alvo irrefutável; antídoto `DIRETIVA_IMPLEMENTACAO.md`
- [Cerca de Chesterton](feedback_documented_decision_chesterton_fence.md) — comentário "intentionally NOT X" = decisão ratificada; não sobrescrever por 1º-princípios
- [Convenção vs inércia](feedback_convention_vs_inertia.md) — checar se "convenção" tem gate ou é inércia; default refactor = mais isolamento

## Git & colisão multi-agente
> **Escopo: Modo C** (shared tree / Mac). No **Modo L** (workstation) cada linha tem worktree+índice próprios → colisão de git extinta; valem só os conflitos de merge (DIRETRIZ §1.5.5). Vide [Modelo = função do HW](project_multiagent_modo_l_2026_07_05.md).
- [Commit collision paralelo](feedback_parallel_agent_collision.md) — `git status` antes de stage; não agarrar staged alheio
- [Scoped commit índice compartilhado](feedback_scoped_commit_shared_index.md) — `git commit -m msg -- <só meus paths>`; `git add` específico p/ untracked
- [Git destrutivo fora da pasta](feedback_destructive_git_outside_pasta.md) — nunca reset/checkout/restore em paths alheios sem coordenar
- [Reset alheio apaga WIP](feedback_destructive_reset_collision_2026_05_28.md) — `git add -- <meus paths>` cedo cria fence (staged+untracked sobrevivem a reset alheio)
- [git stash multiagente](feedback_git_stash_multiagent_danger.md) — stash pop com índice sujo injeta conflict markers em arquivo alheio
- [cargo fmt -p reformata WIP alheio](feedback_cargo_fmt_p_reformats_foreign_wip.md) — formata TODA a crate incl. WIP alheio; use `rustfmt <meus arquivos>`
- [Worktree agent stale base](feedback_worktree_agent_stale_base.md) — `Agent(worktree)` ramifica do HEAD de início; só p/ audit read-only
- [`sed -i` relativo escreve no repo errado](feedback_sed_relative_path_hits_primary_cwd.md) — `cd` composto pode não valer; mutação de arquivo SEMPRE por caminho absoluto (Modo L: senão edita o `main`)

## Ship / CI / cadência
- [Multi-máquina Mac/Linux/Windows](project_multi_machine_setup.md) — GitHub = fonte única, clone local por máquina; memória vendorizada em `project-memory/` via symlink; runbook `docs/DevOps/MULTI_MACHINE_SETUP.md`
- [Fast mode / ship](feedback_fast_mode_ship.md) — dia: `git commit --no-verify` sem push; fim: `./scripts/ship.sh` + push + babysit
- [Ship = Enio-only, fim de TODAS as linhas](feedback_ship_only_enio_end_of_all_lines.md) — nunca ofereça/rode ship no fim da SUA linha; só o Enio decide, no fim da rodada de todas as linhas
- [Integração = Enio-only, via integrador dedicado + handoff](feedback_integration_only_enio_command_end_of_all_lines.md) — Modo L: NÃO integre/shippe sozinho; feche → escreva handoff (DIRETRIZ §1.5.9) → PARE; um **agente integrador dedicado** funde tudo por ordem do Enio
- [Integrador: ship drena latentes das linhas — orce 2-4 iterações](project_integrator_ship_catches_latents_budget_iterations.md) — gate per-linha + foundational-integrate NÃO rodam fmt/clippy-all-targets/machete/deny; fmt-skew e lints só vermelham no ship. Atalho: clippy --workspace --all-targets 1× pega todos. Fmt do integrador pode re-expandir arquivo pro >700 (LOC). `/dev/shm/ph2d-target` some no reboot
- [Foundational é editável — mas crie com isolamento](feedback_foundational_editable_design_for_isolation.md) — Modo L: agentes duvidam, mas PODEM tocar foundational; ao CRIAR foundational projete p/ isolamento (módulo irmão/extensão append-only) + anote ids/consts novos no handoff (colisão `NodeId(832)` 2026-07-07)
- [CI direto + fmt-skew](feedback_ci_direct_lint_gates_and_fmt_skew.md) — lint gates local antes; `cargo fmt` plain = skew, use `rustup run <pin> cargo fmt`
- [Ship committed vs WIP alheio](feedback_ship_committed_vs_worktree_wip.md) — valide/conserte o committed via `git worktree --detach HEAD`, sem tocar WIP
- [CI handling](feedback_ci_handling.md) — Enio confere visual; forneça link da run, não fique em polling
- [CI batching em waves](feedback_ci_batching.md) — acumular commits locais; push único no fim da wave
- [Commit cadence](feedback_commit_cadence.md) — não commitar a cada fix; acumular em blocos
- [Smoke no fim](feedback_smoke_at_end.md) — smoke 1× no fim de TODA a implementação
- [Refactor workflow](feedback_refactor_workflow.md) — commits locais; Enio testa manual antes de push/PR/CI
- [Phase cascade](feedback_phase_cascade_2026_05_19.md) — cada fase fecha + handoff + spawna próxima; última faz PR+CI
- [Codificação rápida](feedback_codificacao_rapida.md) — `cargo check/test -p <crate>`, não `--workspace`
- [Pre-commit arch gates](feedback_precommit_arch_gates.md) — arch-gate do crate antes de commit estrutural; `git commit` em background (hook estoura 2min)
- [LOC cap = split, não allowlist](feedback_loc_cap_split_not_allowlist_and_fmt_reexpands.md) — extraia módulo-irmão; `fmt` re-expande multi-arg → rode fmt ANTES de medir
- [Full gate periodicamente](feedback_full_gate_periodically.md) — ship.sh/nextest na wave; cargo-check esconde gates; re-lock cook hash ao mudar serialização
- [Ship-prep no-fail-fast](feedback_ship_prep_no_fail_fast.md) — `nextest --no-fail-fast` enumera TODAS as falhas; ship.sh é fail-fast
- [CI cold-build = @stable rust-cache drift](project_ci_rustcache_stable_drift_pin.md) — `@stable` rotaciona rustc-hash → cache bust; fix timeout 45→90 + pin `@1.95`. Lockfile igual+rustc-hash mudou = drift
- [ship.sh ≠ 100% paridade CI](feedback_ship_parity_gaps_ci_only.md) — bindgen --check FORA do ship; advisory-db local envelhece (RUSTSEC novo escapa); nextest-impacted quebra em cutover que deleta crates
- [Integrar linha pré-cutover = drift de ship latente](project_integration_prefork_lines_ship_drift.md) — foundational-integrate roda nextest (pega arch-gates) mas NÃO fmt/typos; linha forkada pré-cutover tem fmt não-canônico (style_edition 2024) → só ship.sh/CI pega; rode ship completo no fechamento

## Auditoria
- [Menu "não faz nada" = falta registro no populate](feedback_context_menu_closes_on_down_repaint.md) — grep o id no `populate_*` PRIMEIRO; repaint/close-on-Down = red herring; não mexer no dispatch global
- [Meça a ESCALA do sintoma antes da causa](feedback_measure_perf_symptom_scale.md) — perf: fixe o nº (ms) primeiro; frame(4-16ms) vs ⅓s muda a classe de causa; bench-verde≠vivo
- [Harness reproduz mecanismo, não contexto](feedback_harness_reproduces_mechanism_not_context.md) — smoke contradiz fix provado ⇒ instrumente o guard no app real (eprintln por evento), não itere mecanismo
- [Unit-verde ≠ funciona no produto](feedback_tool_unit_green_integration_dead.md) — tool passa unit+CI e está morta (pill não registrada/input não wirado); só audit e2e pega
- [Gizmo errado = cheque o HIT, não a math](feedback_gizmo_verify_hit_target_before_transform_math.md) — logue o target resolvido no grab ANTES da math; era colisão de id
- [Lens diversity](feedback_audit_lens_diversity.md) — rotacionar lentes; ≥2 paralelas; gates executáveis > claims verbais
- [Scope discipline](feedback_audit_scope_discipline.md) — bug em crate alheio = handoff pro owner, não fixo eu mesmo
- [No industrial claims](feedback_no_industrial_claims_without_verification.md) — zero claim técnico em ADR sem grep/cargo-search/WebFetch
- [Internal-state grep](feedback_audit_internal_state_grep.md) — sweep-grep de símbolos internos antes de escrever ADR
- [Commit-msg claim aging](feedback_audit_commit_msg_claim_verification.md) — claims numéricos envelhecem; framing relativo + literal do grep
- [Determinism sweep grep](feedback_determinism_sweep_grep_all_transcendentals.md) — grepar `\.(sin|cos|tan|atan2|exp|sqrt|pow)\b`, não só `sin_cos`

## Padrões de código (gotchas silenciosos)
- [UI source of truth = gallery + inspector](feedback_ui_source_of_truth_gallery_inspector.md) — UI nova espelha widget-gallery + inspector; não improvisar chrome
- [UI em inglês](feedback_app_ui_english_only.md) — labels/toasts SEMPRE inglês; conferir AQUI antes de "traduzir" por HR-15
- [Nada de `→` em string literal](feedback_no_tofu_arrows_in_string_literals.md) — gate `no_tofu_glyphs` em editor-core/shell; `assert!`/`expect()` são strings (comentário é livre)
- [Tool nova exige IconId](feedback_new_tool_icon_needs_iconid.md) — SVG sem IconId variant (ordem alfabética) quebra TODOS os ícones
- [Fan-out registry-init](feedback_fanout_registry_init_friction.md) — tool-sync NÃO regenera os 2 testes hand-maintained (cluster order + icon slug)
- [node-sync glob prefix](feedback_node_sync_glob_prefix_gotcha.md) — crate na área de nós não pode começar com `ph2d-node-` (gera `::register` inexistente); use outro prefixo
- [Hier companion allowlist](feedback_hier_companion_dispatch_allowlist.md) — bits novos em 2 sites de `pointer.rs` senão click dropado
- [Botão dimmed ainda despacha](feedback_disabled_button_still_dispatches.md) — dim é cosmético; não-registrar hit + recusar no event.rs; cuidado com `unwrap_or(escopo_maior)` atrás de op destrutiva
- [Panel populate register](feedback_panel_populate_register.md) — botão novo exige register em `populate.rs`; pintar + hit_index não basta
- [Painel docado = 4 sites de registro](feedback_docked_panel_registration_four_sites.md) — crate+sync+EXPECTED · **feature-proxy no shell** · **z-order walk em hero/paint.rs** · visibility default+canonical; 2 gates verdes mascaram (painel morto na tela); widgets pela Widget Gallery
- [Panel arch-gates scope + clamp/const](feedback_panel_arch_gates_scope_and_clamp_const.md) — no_magic_numeric + arch_safe_clamp_only escaneiam TODO ph2d-panel-*/src; hoist de bound de clamp p/ const dispara o clamp-gate (precisa `// CLAMP-OK`)
- [Panel 2D-drag precisa dispatch](reference_panel_2d_drag_needs_dispatch.md) — 2D-livre = InteractiveState+dispatch em editor-core (padrão BlenderHit); Slider 1D é o único per-Move no painel
- [NumberInput registra range](reference_number_input_register_range.md) — caixa LIMITADA chama `set_number_range(id,min,max,step)` senão drag escala por `rate×step`
- [Pipeline inject, don't cap](feedback_pipeline_inject_dont_cap.md) — feature nova injeta no buffer do pipeline, não capeia o resultado final
- [Pixel center vs edge](feedback_pixel_center_vs_edge_coord.md) — bilinear espera center-coord; `(local/size+0.5)*W` é edge → subtrair 0.5
- [Exact-pin substring gate](feedback_exact_pin_needs_substring_gate.md) — `=version` pin precisa arch-gate substring senão rebasing "limpa"
- [ISPC cross-process](feedback_ispc_cross_process_concurrency.md) — asset-cooker ISPC crasha com cargo CONCORRENTE; um cargo de cada vez
- [Zero-alloc gate = capacidade, não contador global](feedback_zero_alloc_gate_capacity_not_global_counter.md) — dhat `total_blocks` é global do processo → gate flaky; asserte estabilidade de CAPACIDADE dos buffers (determinístico)
- [Visual bug debug](feedback_visual_bug_debug.md) — aritmética de pixels CEDO + simular visual + instrumentação >> leitura estática
- [Áudio: meter vivo, sem som = mute salvo no WirePlumber](project_audio_multichannel_silence.md) — NÃO é bug de código; `stream-properties` salva `mute:true` por-app e re-aplica no launch; fix = sed no arquivo + restart wireplumber (ou pactl). Multicanal era pista falsa (channelMap já era FL/FR)
- [Claimed-green ≠ seu-OS-green](project_painter_t19_latent_red_macos_2026_05_28.md) — "W1 green" pode ser CI/linux; build o commit claimed-green ANTES de caçar regressão
- [Painter "low-res" = canvas 64px](project_painter_canvas_res_64_not_sim_scale.md) — render macio borra pelo canvas pequeno, não pela escala do sim; cheque a res do source ANTES do shader

## Arquitetura / norte / perf (duráveis, não-git)
- [Blindagem — diagnóstico + Fase 0](project_blindagem_phase0_2026_06_20.md) — aparato mede ESTRUTURAL não COMPORTAMENTAL; Fase 0 = `ph2d-ui-testkit` seam headless + 3 gates
- [Pintura VOLTOU = clean-room Blender](project_painter_brush_came_back_cleanroom.md) — `ph2d-painter-brush` engine NOVO (Blender Texture Paint, não ADR-0099); confie no repo, não na nota "deletada"; NÃO contract-gateado
- [Rebecca era port do sketch.js proprietário → REMEDIADO: PH2D Wet Paint clean-room (2026-07-02)](project_rebecca_watercolor_cleanroom.md) — rebecca/→1.4 = obra derivada (nomes ofuscados+constantes+fórmulas do Rebelle © Escape Motions), agora gitignorada/quarentena; substituta legítima commitada em `docs/Painter/ph2d_wet_paint/` (commit 27e0c069: SPEC comportamental → implementador fresco → verificação por métricas); lições: espec de textura procedural exige banda-média+uniformidade espacial, aceitação sempre com bounds dos 2 lados e ≥2 raios; fingering é arquitetura (2 velocidades + gather + gravidade não-freada), não tuning
- [Blender Texture Paint = referência](project_blender_texture_paint_reference.md) — recorte em `reference/blender-texture-paint/`; GPL-2.0 vs proprietário = port literal proibido, só clean-room
- [Texture Layer = raster-backed](project_texture_layer_design.md) — `LayerKind::Texture` pré-renderizado em `images[id]`; compõe de graça; roteamento via `route_texture_layer_event`
- [Brush audit 2026-06-18](project_brush_audit_2026_06_18.md) — (HISTÓRICO, brush deletado) claims de paridade CPU↔GPU MENTEM (latentes); meça antes de confiar
- [Norte node-centric](project_node_centric_decision_2026_05_21.md) — engine = sistema de nós multi-domínio; `ph2d-nodegraph`+`ph2d-expr`; FBP isolation = unidade multi-agente
- [Motion keyframes adiados p/ a timeline](project_motion_keyframes_deferred_timeline_integration.md) — M2.W1 do Motion ADIADO 2026-07-09 (timeline nasce em outra linha); pesquisa pré-impl preservada
- [Vector cutover ADR-0108](project_vector_cutover_adr0108.md) — módulo REPOSICIONADO (Rive-referenced, GPU/editor-first); motor novo `ph2d-vec-*` + tool `ph2d-tool-vector`; 30 crates antigas retiradas; gotchas: icon-sort=slug, gate-doc FICA, Painter reusa IconId::Vector*, teardown compiler-guided
- [Modelo multi-agente = função do HW (Modo L/C)](project_multiagent_modo_l_2026_07_05.md) — workstation=**Modo L** (linhas por worktree, SEM coordenador, foundational concorrente via gate testado + Mergiraf); constrained=Modo C (v7.1). ADR-0106/0107 + [guia](../docs/IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md)
- [HISTÓRICO: reescrita DIRETRIZ v6.8](project_diretriz_v68_2026_05_22.md) — v6.7→v6.8 (ADR-0040); o "2 papéis/Coord absorve PRCI" é **Modo-C-only** agora (superseded acima)
- [Tool isolation ADR-0040 frozen](project_tool_isolation_freeze_2026_05_22.md) — tools = drop-crates + tool-sync codegen + canal genérico; tool nova = fan-out drop-in
- [Vector node carrier opaco](project_vector_node_opaque_carrier.md) — nós vetoriais emitem VectorNetwork via `CookValue::Opaque(Arc<dyn Any>)`; construa contra o substrato real
- [Brush bridge = satélite, não node](project_brush_along_path_satellite_not_node.md) — cross-module p/ 1 consumidor → crate satélite que só LÊ contratos; defira foundational até ≥2 consumidores
- [Node renderer-consumido = Effect::Pure](project_node_effect_pure_for_renderer_consumed.md) — nó cozido pelo renderer DEVE ser Pure; Cook memo já é o cache; Boolean exato usa `linesweeper`
- [Perf: não otimizar prematuro](project_m5_perf_validated.md) — sprite renderer escala 100k @ 60Hz Mac M-series
- [Perf: gates de velocidade](project_perf_audit_2026_05_19.md) — nextest 17→1.5min via `TextSystem::without_system_fonts()`; lld/ld-prime ganho real 1.5-3% macOS
- [Perf: composite bandwidth-bound](project_painter_w3_block2_persist_ktx2_2026_06_01.md) — 50×4K = 1.66GB/~70GB/s = 23ms; gate dirty-rect. DoS: postcard recursivo → depth-guard
- [Perf: textured-brush = cache o stamp](project_painter_texture_brush_stamp_cache.md) — FPS drop é algorítmico (re-amostra falloff×tex por-dab); fix StampMask cacheado; textura visível no arraste
- [Perf: painter preview → GPU compositor](project_painter_composite_perf_2026_06_03.md) — RESOLVIDO: GPU `LayerOp::Adjustment` WGSL, Metal 1.7ms vs 55ms; medir em `--release`
- [Perf: fluid sim 4K = reescrita GPU-residente](project_painter_fluid_4k_perf_architecture.md) — hot loop O(grid) CPU + upload/readback; micro-opt só ~10%; alvo água GPU-residente + cs_splat
- [Watercolor v2 GPU-first refactor](project_watercolor_v2_gpu_first_refactor.md) — pintura-lenta é submit/copy-bound (não compute); GPU-first single-submit/sparse; ADR-0085
- [Wash undo "mancha volta" = solver twin](project_wash_undo_event_driven_rebuild.md) — write parcial de ping-pong = atualize AMBOS gêmeos; instrumente o caminho ativo antes de reescrever
- [Wash cor pigmento = Mixbox residual](project_wash_pigment_color_mixbox_residual.md) — c=unmix + residual r=rgb−mix(c); cor de pigmento pesquise o publicado; ADR-0091
- [Rendering Modes + Wet Mix (research+design)](project_painter_rendering_modes_research.md) — Procreate Glaze/Blending/Wet+Burnt; design pronto NÃO implementado; enabler = stroke buffer premul-linear 1× no pen-up
- [Painter W3 audit-2 + dirty-rect GPU](project_painter_w3_audit2_perf_2026_06_01.md) — 6-lens zero-critical + partial GPU upload + checked Q16.16; commits local; SMOKE pendente
- [Vector W7 WoS fora de budget](project_wos_diffusion_over_budget_2026_06_06.md) — GPU diffusion-curve CORRETO mas ~20-100× fora do budget; JBU low-res é o caminho
- [Spatial GPU = reconcilia contra apply_*](project_painter_w4_spatial_gpu_bloom_sh.md) — Bloom/S-H GPU reconciliados bit-a-bit contra CPU via dev-dep; impl liga via gpu_spatial_code
- [Panel LOC-gate parser bug — RESOLVIDO 2026-07-10](project_panel_loc_gate_parser_masked_debt.md) — o gate mentia p/ BAIXO (apóstrofo em `//` fechava a fn cedo: 353 vs 477 reais); fix comment-aware + re-baseline + guarda anti-fóssil. Lição: verde não prova ausência de dívida, prova que o medidor não a viu. Aberto: split das 14 fns
- [Painter core files NO TETO 600 LOC](project_painter_core_files_at_loc_cap.md) — paint/brush_settings/stroke/trait_impls exatos em 600; campo novo transborda → orce split
- [KTX2 Basis rejeitado](project_ktx2_phase1_done_phase2_aborted_2026_05_26.md) — runtime Basis abortado; cooking offline nativo per-platform (ADR-0055-v4)
- [imageio AVIF *-sys deps](project_imageio_avif_pathc_2026_05_28.md) — libavif-sys (dav1d+rav1e) precisa meson/nasm/cmake no CI; único format-crate sem `forbid(unsafe)`
- [8GB RAM = full-gate ~10min](project_solo_coord_backlog_ship_2026_05_29.md) — use `clippy --keep-going`; nextest-impacted tinha false-green determinism (corrigido)
- [Wash GPU-resident reimpl](project_wash_gpu_resident_reimpl.md) — reimplementar wash GPU-first/tempo-real, portar física B1-B9 do backup, zero fallback CPU
- [Wash → Curtis g/d](project_wash_curtis_gd_migration_2026_06_15.md) — 3 versões divergiram por NÃO implementar Curtis (falta g/d+TransferPigment); ADR-0095 + plano shallow-water
- [Aquarela: Paper Colors ramp](project_aquarela_paper_ramp_broken.md) — REVERTIDA 2026-07-06 a pedido do Enio (papel volta ao grayscale; seção Paper fica); não reconstruir sem pedir
