# Flakes de carga — a família, a assinatura e os membros conhecidos

> **Doc vivo.** Era a lei mais longa do `CLAUDE.md §5.0` (a história verbatim, com cada promoção
> datada, está em [`docs/archive/estado-2026-10-02/leis-transversais.md`](../archive/estado-2026-10-02/leis-transversais.md)).
> Mudou-se para cá em 2026-10-02 porque uma lista que cresce a cada rodada não pode viver num ficheiro
> que todo agente carrega por inteiro.

## A lei

**Um gate que mede um RECURSO partilhado** — a razão entre dois relógios, uma contagem de alocações,
um contador atrás de estado por thread — **reprova sob o fan-out da suíte inteira e passa sozinho na
máquina calma.** É uma família, não uma lista: o que a identifica é o mecanismo, não o nome.

⛔ Um doc-comment que se declara *«imune ao relógio porque é uma RAZÃO»* não está imune: dividir dois
relógios continua a ser um relógio. Seis gates deste repo diziam isso de si mesmos e são membros.

## A assinatura (confirme ANTES de culpar o seu commit)

1. O teste reprovado mede tempo ou alocações.
2. O diff da linha tem **zero** linhas naquela crate/módulo.
3. Corrido **sozinho**, passa **3 de 3** — e o `/proc/loadavg` vai impresso **ao lado de cada
   corrida**, senão a régua que desmente a flake é a própria flake (`load > ~5` invalida a leitura).
4. Num grupo, o **conjunto** de reprovados MUDA entre corridas do mesmo binário (um defeito de lógica
   reprova sempre o mesmo caso).

⭐ O discriminador é o **fan-out**, não o relógio: há membros que passam a `load 38–60` sozinhos e
reprovam no meio de 16 000 testes em paralelo.

## A cura que fica (e o gate que a mantém)

Todo membro corre **sozinho e no fim** da suíte: a faixa `[[profile.default.overrides]]` do
[`.config/nextest.toml`](../../.config/nextest.toml) (`threads-required = 'num-cpus'`,
`priority = -100`). ⛔ **A barra de nenhum membro se afrouxa** — o que muda é a faixa.

⚠️ **Promover um membro são DUAS escritas, e o gate
`architecture_toda_flake_de_carga_corre_na_faixa_isolada` exige as duas:** uma linha na tabela abaixo
**e** o nome no `filter` daquela faixa. Até 2026-10-02 só a primeira era feita: oito membros
promovidos no `CLAUDE.md` nunca entraram na faixa e continuavam a reprovar portões.

⛔ Não existe padrão largo (`measure_*`) na faixa, de propósito: serializaria sondas que não medem
razão nenhuma. Cada membro entra pelo **nome**.

## Membros conhecidos

| teste | onde | o que mede |
|---|---|---|
| `a_round_live_offset_costs_like_the_other_joins` | [`ph2d-vec-boolean`](../../crates/ph2d-vec-boolean/tests/it/offset_live_cost.rs) | razão de relógios |
| `the_cost_of_depth_is_linear_not_explosive` | [`ph2d-timeline`](../../crates/ph2d-timeline/tests/it/nesting_clock.rs) | razão de relógios |
| `the_cost_of_sampling_a_path_is_flat_in_its_anchors` | [`ph2d-timeline`](../../crates/ph2d-timeline/tests/it/motion_path_perf.rs) | razão de relógios |
| `the_segment_lookup_is_a_binary_search_not_a_scan` | [`ph2d-timeline`](../../crates/ph2d-timeline/tests/it/motion_path_perf.rs) | razão de relógios (5,13× na suíte a `load 40`; 1,05× sozinho, 3/3) |
| `apply_from_doc_is_zero_alloc_steady_state` | [`ph2d-timeline`](../../crates/ph2d-timeline/tests/no_alloc_bridge.rs) | contagem de alocações |
| `no_expression_allocates_no_link_frame` | [`ph2d-timeline`](../../crates/ph2d-timeline/tests/no_expression_link_frame_alloc.rs) | contagem de alocações |
| `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/tests.rs) | razão de relógios |
| `the_mask_stroke_cost_does_not_follow_the_canvas` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/mask_tests.rs) | razão de relógios |
| `the_brush_snapshot_costs_the_same_on_a_canvas_sixteen_times_bigger` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/measure_window_premise.rs) | razão de relógios |
| `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/measure_input_cost.rs) | razão de relógios |
| `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/mask_gate_tests.rs) | razão de relógios |
| `glaze_layering_costs_a_ratio_not_an_order_of_magnitude` | [`ph2d-wet-paint`](../../crates/ph2d-wet-paint/tests/it/perf_experimental.rs) | razão de relógios |
| `only_the_lower_row_breathes_and_it_moves_with_the_playhead` | [`ph2d-app-motion`](../../crates/ph2d-app-motion/src/motion_state_conferencia_demos_audio_tests.rs) | relógio |
| `emitter_sim_ceiling_probe` | [`ph2d-gpu-cook`](../../crates/ph2d-gpu-cook/tests/it/gpu_cpu_parity_sim.rs) | relógio (`#[ignore]`) |
| `packing_a_dense_scribble_is_bounded` | [`ph2d-flip-render`](../../crates/ph2d-flip-render/tests/it/pack_perf.rs) | relógio |
| `the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke` | [`ph2d-app-flip`](../../crates/ph2d-app-flip/src/fit_cache_tests.rs) | razão de relógios |
| `a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget` | [`ph2d-app-flip`](../../crates/ph2d-app-flip/src/fit_budget_tests.rs) | relógio (família `orcamento`) |
| `the_shape_match_is_linear_in_the_mesh` | [`ph2d-node-motion-soft-body`](../../crates/ph2d-node-motion-soft-body/src/lib.rs) | razão de relógios |
| `the_cost_of_a_player_is_linear_in_their_number` | [`ph2d-physics-ecs`](../../crates/ph2d-physics-ecs/tests/it/measure_player_budget.rs) | razão de relógios |
| `sub_stepping_costs_what_it_says_it_costs` | [`ph2d-physics`](../../crates/ph2d-physics/tests/it/penetration.rs) | razão de relógios |
| `sizing_a_long_lossy_clip_is_capped_for_every_lossy_codec` | [`ph2d-audio-encode`](../../crates/ph2d-audio-encode/src/delivery.rs) | razão de relógios (OggVorbis `2,05×` contra o tecto `2×` na suíte do `ship.sh` de 04/10, crate sem diff; sozinho `8/8` a `load ~19`, `0,93–0,98×`) |
| `the_cost_of_a_range_edit_does_not_scale_with_the_clip` | [`ph2d-audio-edit`](../../crates/ph2d-audio-edit/tests/it/measure_range_edit.rs) | razão de relógios (`8,36×` contra o tecto `3×` no `ship.sh` seguinte de 04/10, crate sem diff; sozinho `8/8` a `load ~10`, `0,99–1,04×`) |
| `the_trusted_len_collect_allocates_once` | [`ph2d-audio-edit`](../../crates/ph2d-audio-edit/tests/measure_arc_build.rs) | contagem de alocações |
| `interaction_dispatch_no_alloc` | [`ph2d-editor-core`](../../crates/ph2d-editor-core/tests/interaction_no_alloc.rs) | contagem de alocações |
| `the_ui_clock_does_not_allocate_per_frame` | [`ph2d-editor-core`](../../crates/ph2d-editor-core/tests/ui_motion_no_alloc.rs) | contagem de alocações |
| `tres_bonecos_tres_amplitudes_e_o_rapido_acende_a_lampada` | [`ph2d-app-components`](../../crates/ph2d-app-components/src/script_smoke_tests.rs) | prazo de um quadro por gancho (Luau) |
| `o_console_cheio_deita_fora_a_mais_antiga_e_conta` | [`ph2d-script`](../../crates/ph2d-script/tests/it/a_recarga_e_o_console.rs) | prazo de um quadro (16,7 ms) por gancho (Luau), 256 `print` por quadro — candidato (CodeEditor 15/10): só estoura a `load 60–127` na suíte do fecho; sozinho `3/3` verde. 07/11: estourou no Windows do CI, na faixa isolada ⇒ `retries = 2` (ele e o irmão `um_print_longo…`) |
| `riding_the_path_costs_about_twice_the_straight_layout` | [`shells/desktop`](../../shells/desktop/src/text_path_smoke.rs) | razão de relógios |
| `crossing_the_reach_boundary_does_not_step_the_cost` | [`ph2d-gpu-cook`](../../crates/ph2d-gpu-cook/tests/it/gpu_collide.rs) | razão de relógios (vermelho na suíte da placa no fecho da `line/motion-value` de 07/10 a `load ~35`, zero linhas da onda no teste ou no nó; sozinho `2/2` a `load 9` e `35`) |
| `g4_o_custo_do_rc_nao_depende_do_numero_de_luzes` | [`ph2d-light2d-gpu`](../../crates/ph2d-light2d-gpu/tests/it/g4_custo_independe_das_luzes.rs) | razão de relógios da PLACA (K3 da luz 2D: variação ≤ 5 % entre 1 e 10 000 luzes; medido `0,42`–`1,37 %` a `load 25–42`, perfil smoke e dev; W5c: a causa da beira (3,9–5,1 %) era a entrada num emissor reler a emissão — curada, `0,9`–`1,1 %` a `load` 1) |
| `g4_o_custo_da_holografica_nao_depende_do_numero_de_luzes` | [`ph2d-light2d-gpu`](../../crates/ph2d-light2d-gpu/tests/it/holografica.rs) | razão de relógios da PLACA (o K3 na cascata holográfica, a do produto desde a W5c; medido `1,88`–`1,90 %` a `load 0,6`, perfil smoke). ⛔ 14/10: desde a W5h (`f9c8110287`, os buffers do miolo; bissecção sobre 37 commits) a régua de um solver por contagem media o SÍTIO dos buffers — o de 100 luzes ~3 % mais rápido, `4`–`6 %` mesmo sozinho; reprovou 2 de 3 portões de integração. Cura: as cinco cenas no MESMO solver (`medicao::rc_no_mesmo_solver`) — `2,3`–`2,6 %` a `load 6`; a prova é a `sonda_g4_holografica` (três réguas no mesmo processo) |
| `g4_o_interior_de_um_opaco_nao_custa` | [`ph2d-light2d-gpu`](../../crates/ph2d-light2d-gpu/tests/it/g4_custo_independe_das_luzes.rs) | razão de relógios da PLACA (disco opaco 100 px ÷ 25 px: `1,34` com a saída antecipada, `3,41` sem; barra `2`) |
| `o_escorrimento_do_s_nao_se_corta_em_rectangulos` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/achatamento_tests.rs) | prazo do nextest (180 s) sobre um portão de CENA do Wet Paint que já paraleliza as cenas — TIMEOUT duas vezes na suíte inteira de 17/10 (integração a `load 74`, `ship.sh` a `load ~60`), zero linhas da crate no diff do ship; sozinho `16,8 s` (`load 16–26`) e `20,3 s` (`load ~10`) |
| `n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/mistura_camadas_tests.rs) | prazo do nextest (a faixa pesada, 6 × 60 s) — o mesmo par de TIMEOUT de 17/10; sozinho `50,1 s` (`load 16–26`) e `71,6 s` (`load ~10`). Paraleliza por dentro: sob o fan-out disputa os núcleos que pede, e a faixa de prazo sozinha não basta |
| `o_glaze_do_pincel_escurece_o_cruzamento` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/tests/it/a_lei_da_tinta.rs) | prazo do nextest (a faixa pesada, 6 × 60 s) — TIMEOUT de 180 s em duas suítes seguidas do fecho dos Pincéis (22/10, load15 ~42); sozinho 17 · 8 · 8 s (`load 6,5–10`), 3/3 |
| `sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/tests/it/a_lei_da_tinta.rs) | prazo do nextest (a faixa pesada) — o mesmo par de TIMEOUT de 22/10; sozinho 10 · 9 · 9 s (`load 10–13`), 3/3; pinta a cena de cada preset de fábrica (45) |
| `as_bordas_nao_esbranquicam_depois_de_varios_cruzamentos` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/tests/it/a_lei_da_tinta.rs) | prazo do nextest (a faixa pesada, a dos irmãos de `a_lei_da_tinta`) — TIMEOUT de 180 s na suíte combinada da integração da `line/TrelloClone` (29/10, load15 ~80), zero linhas da crate no diff; sozinho PASS (load 18–29, ~140 s com a compilação) |
| `um_print_longo_corta_a_linha_e_o_console_tem_tecto_em_bytes` | [`ph2d-script`](../../crates/ph2d-script/tests/it/a_recarga_e_o_console.rs) | orçamento de RELÓGIO do quadro (16,667 ms, `ScriptDiagnostic::Budget`) sobre um `print` longo — falhou na 2.ª suíte combinada da integração da `line/TrelloClone` (29/10, load alto), zero linhas do teste/consola no diff; sozinho PASS |
| `o_deposito_de_um_pincel_pesado_de_fabrica_vai_pela_rota_que_paga` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/tests/it/o_custo_no_caminho_do_app.rs) | razão de relógios (rota nova × antiga, 2,25× calmo) — vermelho na suíte a `load ~30` (22/10), sozinho 2/2 verde |
| `a_tinta_nao_muda_com_o_spacing` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/tests/it/o_pigmento_por_comprimento.rs) | prazo do nextest (180 s → a faixa pesada, 6 × 60 s) — TIMEOUT na suíte inteira do ship de 23/10 (load ~40), zero linhas da crate no diff do ship; sozinho no `ci-test` 22,0 · 21,9 s (`load 3–14`), 2/2 |
| `opening_the_room_says_how_to_open_the_firewall` | [`ph2d-app-board`](../../crates/ph2d-app-board/src/live_firewall_tests.rs) | espera de ~100 ms (50 × `service` de 2 ms) pela thread da sonda do firewall — vermelho em 0,058 s na suíte do fecho W24 do MiroClone (23/10, load 70–78), zero linhas do teste nem do firewall no diff; sozinho 0,50 · 0,50 · 0,50 s (`load 9–10`), 3/3 |
| `macro_estado_inicial::cada_controlo_mexido_a_gravar_reproduz_se` · `macro_estado_inicial::o_estado_inicial_leva_cada_controlo_de_cada_meio` | `ph2d-panel-painter-layers` (`tests/it`) | painel inteiro do Painter por meio, em debug — TIMEOUT aos 180 s na suíte do fecho W25 do MiroClone (24/10, load 220 → 13) e na suíte da `line/PainterWatercolor` de 23/10 (load ~140; 143,4 · 145,0 s na suíte anterior do dia, 136,6 · 137,1 s sozinhos no `ci-test`), zero linhas da crate no diff; sozinhos 82 · 81 s com a compilação (load 12–33) — faixa de 6 × 60 s, a do irmão `seam_estado_escondido` |
| `macro_estado_inicial::o_estado_inicial_leva_as_ferramentas_do_trilho` | `ph2d-panel-painter-layers` (`tests/it`) | o irmão dos de cima — TIMEOUT aos 180 s na suíte da `line/TrelloClone` (28/10, load ~214–340), zero linhas da crate no diff; sozinho 179 s com a compilação, 142,6 s de teste (load 214 → 340) — faixa de 10 × 60 s dos irmãos |
| `a_agua_pelo_compositor_e_a_tela_de_casa_ao_byte` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/otica_no_tique_tests.rs) | contagem de imagens que o compositor da óptica (outro fio) aplica em 30 quadros — `4` contra o piso `5` na suíte da integração de 02/11c (load 35–70); sozinho `3/3` verde a `load 64–70` (8,6–8,8 s) |
| `a_separacao_das_pocas_e_exacta_na_fronteira` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/tests/watercolor_secagem_por_poca.rs) | prazo do nextest (180 s → a faixa pesada, 6 × 60 s) — TIMEOUT na suíte da `line/PainterWatercolor` de 23/10 (load ~140); 134,7 s na suíte anterior do dia |
| `o_censo_dos_controlos_so_encolhe` | [`ph2d-panel-painter-layers`](../../crates/ph2d-panel-painter-layers/tests/it/o_censo_so_encolhe.rs) | prazo global do nextest (180 s) sobre o censo dos quatro meios, que já paraleliza por dentro (`em_paralelo`, 8 fios × meio) — TIMEOUT aos 180 s na suíte combinada da integração 02/11c (load ~60); sozinho por nome 76,8 · 75,9 · 75,5 s (`load 26–37`), 3/3. Disputa os núcleos que pede: corre SOZINHO no fim |
| `com_pacotes_perdidos_chega_tudo_e_na_ordem` | [`ph2d-board-internet`](../../crates/ph2d-board-internet/src/node_tests.rs) | prazo de rede (`TimedOut` aos 33 s, QUIC com perdas simuladas) na suíte da integração de 03/11 (load ~100), zero linhas da crate no diff; sozinho 2,6 s |
| `cada_interruptor_que_neutraliza_pinta_como_o_pincel_limpo` | [`ph2d-panel-painter-layers`](../../crates/ph2d-panel-painter-layers/tests/it/seam_estado_escondido.rs) | prazo do nextest (faixa de 10 × 60 s) — TIMEOUT na suíte de 03/11 (load ~100); por nome 207 s (load ~57) |
| `cada_knob_do_material_muda_o_preview` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/material_do_pincel_tests.rs) | prazo global do nextest (180 s) — TIMEOUT na suíte de 03/11 (load ~100); por nome 127 s (load ~57) |
| `pular_a_pincelada_e_esperar_nao_cumpre_nenhuma` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/macro_track/tutorial_tests.rs) | prazo do nextest (faixa de 10 × 60 s) — TIMEOUT na suíte de 03/11 (load ~100); por nome 201 s (load ~57) |
| `o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/wetpaint_preset/portao_mae_f0.rs) | prazo global do nextest (180 s) — TIMEOUT na suíte de 03/11 (load ~100); por nome 112 s (load ~57) |
| `sorting_a_reversed_list_grows_n_log_n_not_n_squared` | [`ph2d-kanban-filter`](../../crates/ph2d-kanban-filter/src/sort_tests.rs) | razão de relógios: ×28,7 contra o tecto ×24 na suíte de 05/11 (load 11–45), zero diff na crate; sozinho 5/5 |
| `a_area_do_blend_cabe_no_circulo` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/tests/it/presets_de_fabrica/misturador.rs) | memória da fatia: «179 MiB + 9 de reserva > 186 livres» na suíte de 05/11; sozinho 5/5 |
| `ligar_o_relevo_mede_a_linha_da_camada_no_mesmo_quadro` | [`ph2d-panel-painter-layers`](../../crates/ph2d-panel-painter-layers/src/ui_que_mente_tests.rs) | barra do relevo pintada no MESMO quadro — candidato: vermelho só no B2 da integração de 13/11 (máquina carregada); verde na suíte completa anterior do dia e `2/2` sozinho (`load ~5`) |
| `com_a_agua_a_correr_o_clipe_nunca_mostra_a_imagem_final` | [`ph2d-app-painter`](../../crates/ph2d-app-painter/src/macro_track/palcos_marca_tests.rs) | palco da macro com a água a correr (`Divergiu(PerguntaInesperada)`) — candidato: o mesmo B2 de 13/11; verde na suíte anterior (32 s) e `2/2` sozinho |
| `as_faixas_fora_do_compositor_nao_abrem_fios_do_so` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/fios_do_so_tests.rs) | censo de fios do SO fora do pool (`("carimbo", 16, 15)`) — candidato: o mesmo B2 de 13/11; verde na suíte anterior e `2/2` sozinho. ⚠️ Se voltar, conte os fios com a máquina calma ANTES de chamar carga: um fio a mais é defeito, não relógio |
| `a_luz_do_traco_e_a_unica_luz_do_oleo` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/otica_no_tique_tests.rs) | a tela 30 quadros depois do pen-up (1 743 px diferentes) — candidato: vermelho só no 2.º B2 de 13/11 (máquina carregada); verde na 4.ª suíte completa do dia (18,7 s) e `2/2` sozinho (`load ~4`, 6,8 s) |
| `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/tests/it/sonda_custo_do_veu.rs) | razão de relógios: 1,26–1,29 contra o tecto 1,3 sob carga (10/11), sã ~0,9; a régua passou a aquecimento + 7 voltas rodadas + o mínimo (B07.1) e o tecto não subiu |
| `o_indicador_custa_o_que_o_quadro_ja_paga_pela_mesma_arte` | [`ph2d-skeleton-live`](../../crates/ph2d-skeleton-live/src/peso_a_mao_relogio_tests.rs) | razão de relógios — entrou pelo censo B07.7 (14/11), não por ter reprovado |
| `o_tecto_de_manchas_custa_uma_razao_e_nao_uma_ordem_de_grandeza` | [`ph2d-skeleton-live`](../../crates/ph2d-skeleton-live/src/peso_a_mao_relogio_tests.rs) | razão de relógios — entrou pelo censo B07.7 (14/11), não por ter reprovado |
| `o_tique_do_balde_custa_pela_regiao_nao_pela_tela` | [`ph2d-tool-painter`](../../crates/ph2d-tool-painter/src/tool/paint/fill_tique_tests.rs) | razão de relógios — entrou pelo censo B07.7 (14/11), não por ter reprovado |
| `pigment_mixing_costs_a_ratio_not_an_order_of_magnitude` | [`ph2d-wet-paint`](../../crates/ph2d-wet-paint/tests/it/perf_experimental.rs) | razão de relógios — entrou pelo censo B07.7 (14/11), não por ter reprovado |
| `the_route_cost_is_linear_in_the_wheel_count` | [`ph2d-physics`](../../crates/ph2d-physics/tests/it/pulley.rs) | razão de relógios — entrou pelo censo B07.7 (14/11), não por ter reprovado |
| `what_one_frame_of_a_knob_drag_costs` | [`ph2d-audio-edit`](../../crates/ph2d-audio-edit/tests/it/measure_preview.rs) | razão de relógios — entrou pelo censo B07.7 (14/11), não por ter reprovado |

As cenas 3, 4 e 5 do editor de scripts (`a_cena_*_conduzida_passo_a_passo`) SAÍRAM em 29/10: não eram carga, era a
árvore velha da gramática a esticar o comentário sobre o `ph2d.` escrito na linha nova (cura e gate em
`ph2d-app-code`: `Completer::settle` · `completar::depois_de_um_comentario_a_arvore_velha_nao_fecha_a_lista`; 30/30
verdes a `load` 35–48 contra ~40 % vermelhas a `load` 10 antes).

Uma família entra por padrão e não por nome: `test(/orcamento::the_fit_/)` (`ph2d-app-flip`).

⭐ **B07.7 (14/11): toda RAZÃO entre relógios é membro, sem esperar a primeira flake.** O censo
`architecture_toda_razao_de_relogios_corre_na_faixa_isolada` (`ph2d-editor-core`) classifica pelo texto: razão = um `let`
que divide dois relógios ou um `assert!` que junta dois; o PRAZO (`t.elapsed() < 5 s`, um tecto de segurança) fica
FORA da faixa — serializá-lo pararia a suíte por nada. Medido no dia: 15 razões (9 já na faixa, 6 entraram) e 40
testes com relógio que não o são.

## A espécie do estado do painel (04/11): nem era carga

Assinatura enganadora: passa sozinho, reprova em grupo, passa «em fila». Oito scrubs de `o_alfinete` reprovavam em
paralelo com o **mesmo** conjunto em 3 de 3 corridas (um flake de carga muda de conjunto). Não mediam recurso nenhum.

**A variável:** `InspectorState::last_entity`, dentro do `ErasedPanel`, um só no processo, ao lado de ~70 `thread_local`
que o painel lê. Duas mesas em paralelo trocavam-no a cada quadro; a «troca de objecto» (`sync.rs`) chama
`end_number_input_drag`, e o gesto morria. Medido, o mesmo binário, intercalado, `load` ~30:

| Estado do painel | `o_alfinete` a 8 fios | Arrastos largados pela troca |
|---|---|---|
| Por processo | 41 de 49, 3 de 3 corridas | 22, 22, 23 |
| Por fio | 49 de 49, 3 de 3 corridas | 0, 0, 0 |

**A cura está na origem:** `ErasedPanel` guarda estado e coluna do valor **por fio**. O libtest passa a ver o que o nextest vê
(um estado por teste); a app pinta num fio só e tem uma entrada. `a_vez_da_mesa` saiu. O binário inteiro de
`ph2d-panel-registry-init` a 8 fios ficou em 316 de 316, e três vermelhos que dependiam da companhia saíram com a cura:
`dentro_de_um_troco_o_valor_arranca_numa_coluna_so` e dois de `os_estilos_de_cor`.

**Produto:** sem defeito. Só há uma `HeroScreen` a pintar na app (`init.rs`); uma segunda, no mesmo fio, voltaria a partilhar o
estado. O mesmo acidente já tinha sido contornado duas vezes, com a mesma variável nomeada: `registo_privado`
(`quantas_entradas_tem_cada_painel`) e `InspectorPrivado` (`cartoes_mao`); saíram em 04/11 (fatia F4), com o censo a correr no registo do processo.

## A espécie da mola no relógio de parede (13/11): vermelho por PERFIL, não por carga

Assinatura: vermelho em 40 ms na faixa isolada, verde sozinho no `dev`, vermelho também em `cargo test --profile ci-test`.
`ligar_o_relevo_mede_a_linha_da_camada_no_mesmo_quadro` pintava 400 quadros para «assentar» as alturas dos cartões, que
andam por mola com o `dt` do relógio (`cards_lista.rs`). A opt-level 0 (~0,4 ms/quadro) assentavam; a opt-level 1 do
`ci-test` (~0,08 ms/quadro) a secção ficava com 4 px e a barra do relevo nem se pintava. **Cura:** os quadros de
assentar correm com `aquecimento::com_passo_fixo(0.1, …)` e o quadro medido com `PASSO_DE_QUADRO_S`. Um teste que
pinta N quadros para esperar uma mola sem fixar o passo é membro desta espécie.

## A espécie da memória da placa (W5e, 2026-10-11)

Um gate de GPU que ALOCA muito (a luz 2D: 0,3–4 GB por teste) reprova no fan-out quando a soma dos
testes em paralelo passa a memória da placa — o driver recusa (`OutOfMemory`; a luz devolve
`SemMemoria`) — e passa sozinho. O recurso é a VRAM, não o relógio: a cura é uma faixa que limita a
CONCORRÊNCIA desses testes (`[test-groups.luz2d-placa] max-threads = 2` no
[`.config/nextest.toml`](../../.config/nextest.toml), por pacote: `ph2d-app-light2d` e
`ph2d-light2d-gpu`), não a faixa isolada do relógio. Medido no portão W5e: 9 de 17 508 reprovaram a
load 8–34 com ~7 GB da placa já tomados por outros programas; os 9 passaram sozinhos.
⚠️ **Membro que a faixa de 2 não cura (12/10):** `os_tetos_da_placa::a_luz_recua_quando_a_placa_recusa_a_memoria`
pede DE PROPÓSITO mais do que a placa dá; com outro gate da luz ao lado a recusa vira falta de memória de
verdade para a placa inteira (o atlas do renderer inválido, pânico do wgpu). Sozinho 3/3 verde → corre
SOZINHO (`threads-required = "num-test-threads"` no `nextest.toml`) e **NO FIM** (`priority = -100`, 04/11): na
frente, pedia todas as vagas e deixava 2 de 16 ocupadas dos 30 s aos 210 s. ABAB, mesma compilação,
`package(ph2d-app-light2d) | package(ph2d-app-macros)` (201 testes): mínimo 285,7 → 251,7 s (−12 %).

## Parece carga e não é: a memória do processo com as arenas do glibc (03/11)

`a_memoria_do_dabber_cabe` reprovou na suíte a load ~100, parecia membro, e **não entrou na faixa**: a assinatura
falhou no passo 3 (reprovou por nome a load 12). O `VmHWM` de um processo com vários fios variava 1 673–2 053 MiB na
mesma lei, sem relação com a carga. Com `MALLOC_ARENA_MAX=1` no processo medido: 1 607–1 610 MiB em 17/17 (load 4–84).
⇒ Gate de RAM mede num filho com uma arena, e o tecto sai dessa medida
([`memoria.rs`](../../crates/ph2d-tool-painter/tests/it/presets_de_fabrica/memoria.rs)).

## A outra espécie: a flake de GPU só no Linux do CI

`atlas::tests::remove_of_missing_key_returns_none` ([`ph2d-render`](../../crates/ph2d-render/src/atlas/tests.rs))
saiu `SIGSEGV` uma vez em 1 300 no Linux do CI com macOS e Windows verdes, e a re-corrida do mesmo
commit passou: é o adaptador por software do runner (`lavapipe`), não a lei. Antes de investigar um
`SIGSEGV` de GPU no CI, pergunte **se os outros dois SO passaram** e **se o módulo tem diff**; com
as duas respostas «sim» e «não», a cura é `gh run rerun --failed`.

⚠️ **04/10: deixou de ser 1 em 1 300.** Com o Mesa do runner a passar de `25.2.8-0ubuntu0.24.04.3`
para `…04.4` (mesma imagem `ubuntu-24.04 20260927.320.1`), o mesmo teste caiu **2/2** (SIGABRT
«double free», depois SIGSEGV) — sempre DEPOIS de imprimir `ok`, no fecho do processo. Mecanismo: o
`TextureAtlas::new` submete a limpeza do nível 0, o teste não lê nada de volta e sai em ~0,05 s com
o `GpuContext` preso num `OnceLock` (nunca largado) e o lavapipe ainda a executar a fila. Os outros
20 testes GPU do atlas fazem readback (esperam a fila) e passaram. Cura: o teste espera a fila
(`device.poll(PollType::wait_indefinitely())`) antes de sair. ⇒ um teste GPU que **submete e não
lê** drena a fila no fim; o re-run deixou de ser a cura.

Os gates de GPU deste repo são `#[ignore]` e precisam de adaptador: *um skip gracioso não é verde*.
