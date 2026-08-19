# `Motion Nodes` — índice do módulo

> **Gerado por `bash scripts/doc-index.sh` — não edite à mão.** Uma lista mantida à
> mão envelhece na primeira semana; esta é derivada do primeiro `# ` de cada arquivo.
>
> O **pensamento** do módulo Motion Nodes: o plano, as pesquisas de referência, e uma *nota-ADR* por família de nós — o registro de por que cada nó tem os parâmetros que tem. O registro de **como** foi construído (um arquivo por sessão de linha) fica em [`handoffs/`](handoffs/README.md); a conferência nó-a-nó, em [`89_conferencia/`](89_conferencia/README.md).
>
> ⚠️ **Isto NÃO é o estado atual do módulo.** O estado vivo é o **[`CLAUDE.md §5`](../../CLAUDE.md)**;
> um doc descreve o mundo **no dia em que foi escrito** e não é atualizado depois. Use-os
> para responder *"por que isto ficou assim?"* — nunca para decidir a próxima ação.

**99 arquivos** · **2** citados pelo `CLAUDE.md` (marcados **◆**) · **2** são handoffs (registro **morto**).

| # | | Arquivo | Papel | Assunto |
|---|---|---|---|---|
| 00 |   | [00_estudo_estado_da_arte.md](00_estudo_estado_da_arte.md) | pesquisa | Motion Nodes — Estudo de estado da arte e proposta para a PH2D |
| 01 |   | [01_plano_modulo_motion_nodes.md](01_plano_modulo_motion_nodes.md) | plano | Plano — Módulo Motion Nodes da PH2D |
| 02 |   | [02_dinamica_m2_pesquisa_decisoes.md](02_dinamica_m2_pesquisa_decisoes.md) | pesquisa | M2-Dynamics — pesquisa e decisões (forças + integrador + spring, SEM timeline) |
| 03 |   | [03_reentrada_integrate_estudo_padrao_ouro.md](03_reentrada_integrate_estudo_padrao_ouro.md) | pesquisa | 03 — Por que o Integrate tem reentrada? Estudo e padrão-ouro |
| 04 |   | [04_evidencias_industria_feedback_sim.md](04_evidencias_industria_feedback_sim.md) | — | 04 — Evidências da indústria: estado de simulação em grafos de nós (pesquisa primária) |
| 05 |   | [05_time_scopes_nota_adr.md](05_time_scopes_nota_adr.md) | nota-ADR | 05 — Nota-ADR: escopos de tempo no Cook (`cook_scoped`) — M2.N1/N4 |
| 06 |   | [06_pulse_gatilho_primeira_classe.md](06_pulse_gatilho_primeira_classe.md) | — | 06 — Pulse: o gatilho de 1ª classe (decisão + evidências) |
| 07 |   | [07_noise_perlin_gradient_field.md](07_noise_perlin_gradient_field.md) | — | 07 — `motion.noise`: campo de ruído (decisão + evidências) |
| 08 |   | [08_pulse_counter_reducer_bridge.md](08_pulse_counter_reducer_bridge.md) | — | 08 — `pulse.counter`: o redutor de pulsos (a ponte Evento→Valor) |
| 09 |   | [09_handoff_pulse_signal_source_and_naming.md](09_handoff_pulse_signal_source_and_naming.md) | ⚠️ handoff (morto) | HANDOFF 09 — Família pulse: matar o "clock hack", criar uma fonte de sinal de verdade, e arrumar os nomes |
| 10 |   | [10_auditoria_30_nos_correcoes.md](10_auditoria_30_nos_correcoes.md) | auditoria | 10 — Auditoria dos 30 nós (2026-07-10): achados, correções aplicadas e o que fica |
| 11 |   | [11_checkpoint_restore_scrub_nota_adr.md](11_checkpoint_restore_scrub_nota_adr.md) | nota-ADR | 11 — Nota-ADR: `Cook::checkpoint/restore` + scrub para trás — M2.N2/N3 |
| 12 |   | [12_dominio_de_valor_nota_adr.md](12_dominio_de_valor_nota_adr.md) | nota-ADR | 12 — Nota-ADR: o domínio de VALOR (pulse.counter puro + motion.drive) — P2 do doc 09 |
| 13 |   | [13_lfo_map_range_nota_adr.md](13_lfo_map_range_nota_adr.md) | nota-ADR | 13 — Nota-ADR: LFO + Map Range (fatia 2 do domínio de VALOR) — follow-up do doc 12 |
| 14 |   | [14_sample_hold_instance_field_nota_adr.md](14_sample_hold_instance_field_nota_adr.md) | nota-ADR | 14 — Nota-ADR: Sample & Hold + Instance Field (fatia 3 do domínio de VALOR) |
| 15 |   | [15_handoff_continuacao_dominio_valor.md](15_handoff_continuacao_dominio_valor.md) | ⚠️ handoff (morto) | 15 — Handoff de CONTINUAÇÃO: próxima linha do Motion (domínio de valor + resto do M2) |
| 16 |   | [16_math_compare_nota_adr.md](16_math_compare_nota_adr.md) | nota-ADR | 16 — Nota-ADR: Math + Compare (fatia 4 do domínio de VALOR) — follow-up dos docs 12–14 |
| 17 |   | [17_switch_on_change_nota_adr.md](17_switch_on_change_nota_adr.md) | nota-ADR | 17 — Nota-ADR: Switch + On Change (fatia 5 do domínio de VALOR) — fecha o vocabulário |
| 18 |   | [18_fibonacci_twist_nota_adr.md](18_fibonacci_twist_nota_adr.md) | nota-ADR | 18 — Nota-ADR: Fibonacci + Twist (abertura do M3 — distribuições + deformers) |
| 19 |   | [19_scatter_morph_nota_adr.md](19_scatter_morph_nota_adr.md) | nota-ADR | 19 — Nota-ADR: Scatter + Morph (M3.2 — blue-noise + crossfade) |
| 20 |   | [20_bend_look_at_nota_adr.md](20_bend_look_at_nota_adr.md) | nota-ADR | 20 — Nota-ADR: Bend + Look At (M3.3 — dois deformers) |
| 21 |   | [21_verlet_rope_boids_nota_adr.md](21_verlet_rope_boids_nota_adr.md) | nota-ADR | 21 — Nota-ADR: Verlet-Rope + Boids (M4.1 — abre a família de SIMULAÇÃO) |
| 22 |   | [22_soft_body_wave_nota_adr.md](22_soft_body_wave_nota_adr.md) | nota-ADR | 22 — Nota-ADR: Soft-Body + Wave (M4.2 — mídia contínua na família de simulação) |
| 23 |   | [23_lattice_voronoi_nota_adr.md](23_lattice_voronoi_nota_adr.md) | nota-ADR | 23 — Nota-ADR: Lattice + Voronoi (M3 — as distribuições que faltavam) |
| 24 |   | [24_four_point_warp_spherize_nota_adr.md](24_four_point_warp_spherize_nota_adr.md) | nota-ADR | 24 — Nota-ADR: Four Point Warp + Spherize (M3 — os deformers que faltavam) |
| 25 |   | [25_radial_mirror_nota_adr.md](25_radial_mirror_nota_adr.md) | nota-ADR | 25 — Nota-ADR: Radial Array + Mirror (M3 — array polar + simetria) |
| 26 |   | [26_kaleidoscope_collide_nota_adr.md](26_kaleidoscope_collide_nota_adr.md) | nota-ADR | 26 — Nota-ADR: Kaleidoscope + Collide (M3 — simetria N-fold + empacotamento) |
| 27 |   | [27_sort_cull_nota_adr.md](27_sort_cull_nota_adr.md) | nota-ADR | 27 — Nota-ADR: Sort + Cull (M3 — os operadores ESTRUTURAIS do stream) |
| 28 |   | [28_distribute_curve_spline_wrap_nota_adr.md](28_distribute_curve_spline_wrap_nota_adr.md) | nota-ADR | 28 — Nota-ADR: Distribute Curve + Spline Wrap (M3 — a família CURVA, self-contained) |
| 29 |   | [29_color_ramp_color_array_nota_adr.md](29_color_ramp_color_array_nota_adr.md) | nota-ADR | 29 — Nota-ADR: Color Ramp + Color Array (M1 — a família COR, cauda self-contained) |
| 30 |   | [30_combine_mixer_nota_adr.md](30_combine_mixer_nota_adr.md) | nota-ADR | 30 — Nota-ADR: Combine + Mixer (M1 — os operadores de STREAM, branch-and-merge) |
| 31 |   | [31_make_point_luminance_nota_adr.md](31_make_point_luminance_nota_adr.md) | nota-ADR | 31 — Nota-ADR: Make Point + Luminance (M1 — adapters valor↔geometria↔cor) |
| 32 |   | [32_expression_text_param_channel_nota_adr.md](32_expression_text_param_channel_nota_adr.md) | nota-ADR | 32 — Nota-ADR: Expression + o canal de TEXT PARAM (M1 — o escape-hatch de fórmula) |
| 33 |   | [33_expression_text_field_ui_nota_adr.md](33_expression_text_field_ui_nota_adr.md) | nota-ADR | 33 — Nota-ADR: UI de texto no painel de params (editar a fórmula da expression) |
| 34 |   | [34_pin_constraint_e_slit_scan_nota_adr.md](34_pin_constraint_e_slit_scan_nota_adr.md) | nota-ADR | 34 — Pin Constraint (massa inversa) + Slit Scan — nota-ADR |
| 35 |   | [35_backdrops_nota_adr.md](35_backdrops_nota_adr.md) | nota-ADR | 35 — Backdrops (grupos no editor de nós) — nota-ADR |
| 36 |   | [36_duplicate_knife_botoes_nota_adr.md](36_duplicate_knife_botoes_nota_adr.md) | nota-ADR | 36 — Editor F2: botões (pan/seleção), Ctrl+D e a faca — nota-ADR |
| 37 |   | [37_probe_sparkline_smart_connect_nota_adr.md](37_probe_sparkline_smart_connect_nota_adr.md) | nota-ADR | 37 — Editor F2: as teclas que nunca chegavam · probe + sparkline · smart-connect — nota-ADR |
| 38 |   | [38_fx_ghost_copias_rgb_split_drop_shadow_nota_adr.md](38_fx_ghost_copias_rgb_split_drop_shadow_nota_adr.md) | nota-ADR | 38 — M4 FX por-instância: `fx.rgb_split` + `fx.drop_shadow` (e por que `fx.mirror` foi CANCELADO) |
| 39 |   | [39_size_identity_nota_adr.md](39_size_identity_nota_adr.md) | nota-ADR | 39 — `SIZE_IDENTITY`: um nó no seu default não pode redimensionar a cena — nota-ADR |
| 40 |   | [40_rig_esqueleto_e_fk_nota_adr.md](40_rig_esqueleto_e_fk_nota_adr.md) | nota-ADR | 40 — Rig: `rig.skeleton` + `rig.fk` — e a **decisão M4.N3** (sem `Domain::Rig`, sem ADR) |
| 41 |   | [41_rig_ik_2bone_e_fabrik_nota_adr.md](41_rig_ik_2bone_e_fabrik_nota_adr.md) | nota-ADR | 41 — Rig: `rig.ik_2bone` (lei dos cossenos) + `rig.fabrik` — nota-ADR |
| 42 |   | [42_rig_rubber_hose_e_skin_nota_adr.md](42_rig_rubber_hose_e_skin_nota_adr.md) | nota-ADR | 42 — Rig: `rig.rubber_hose` + `rig.skin_deformer` — **M4 Rig FECHADO** — nota-ADR |
| 43 |   | [43_readouts_inline_e_cards_inertes_nota_adr.md](43_readouts_inline_e_cards_inertes_nota_adr.md) | nota-ADR | 43 — Editor F2: **readouts inline** + **cards inertes** — o grafo diz o que está fazendo — nota-ADR |
| 44 |   | [44_waypoints_nota_adr.md](44_waypoints_nota_adr.md) | nota-ADR | 44 — Editor F2: waypoints (roteamento de fios) — **REVOGADO** — nota-ADR |
| 45 |   | [45_reroute_e_socket_de_entrada_nota_adr.md](45_reroute_e_socket_de_entrada_nota_adr.md) | nota-ADR | 45 — O ponto no fio é um **NÓ** (reroute) · e o socket de ENTRADA vira um plugue — nota-ADR |
| 46 |   | [46_o_grafo_mostra_o_que_esta_vivo_nota_adr.md](46_o_grafo_mostra_o_que_esta_vivo_nota_adr.md) | nota-ADR | 46 — O grafo mostra o que está **vivo** (F3.1: inerte · marcha · massa) — nota-ADR |
| 47 |   | [47_influencia_e_postage_stamp_nota_adr.md](47_influencia_e_postage_stamp_nota_adr.md) | nota-ADR | 47 — **Influência** (o que o nó afeta) e o **postage stamp** (o que o nó faz) — nota-ADR |
| 48 |   | [48_zona_de_simulacao_nota_adr.md](48_zona_de_simulacao_nota_adr.md) | nota-ADR | 48 — **Zona de Simulação** (O4) — nota-ADR |
| 49 |   | [49_nascimento_na_zona_nota_adr.md](49_nascimento_na_zona_nota_adr.md) | nota-ADR | 49 — **Nascimento** na zona (`sim.spawn`) — nota-ADR |
| 50 |   | [50_idade_vida_e_atributo_nota_adr.md](50_idade_vida_e_atributo_nota_adr.md) | nota-ADR | 50 — **Idade, vida e o nó `value.attribute`** — nota-ADR |
| 51 |   | [51_fade_de_verdade_opacity_nota_adr.md](51_fade_de_verdade_opacity_nota_adr.md) | nota-ADR | 51 — **Desvanecer de verdade**: o canal Opacity — nota-ADR |
| 52 |   | [52_colisao_com_o_mundo_nota_adr.md](52_colisao_com_o_mundo_nota_adr.md) | nota-ADR | 52 — **Colisão com o mundo** (`sim.collide`) — nota-ADR |
| 53 |   | [53_o_fps_e_o_numero_de_draw_objects_nota_adr.md](53_o_fps_e_o_numero_de_draw_objects_nota_adr.md) | nota-ADR | 53 — **O FPS é o número de DRAW OBJECTS** (e o doc de boot enxugou) — nota-ADR |
| 54 |   | [54_scroll_no_add_menu_nota_adr.md](54_scroll_no_add_menu_nota_adr.md) | nota-ADR | 54 — **Scroll (e barra arrastável) no add-menu** — nota-ADR |
| 55 |   | [55_uma_regua_nao_pode_ser_funcao_do_que_ela_mede_nota_adr.md](55_uma_regua_nao_pode_ser_funcao_do_que_ela_mede_nota_adr.md) | nota-ADR | 55 — Uma régua não pode ser função do que ela mede (sliders em bilhões) |
| 56 |   | [56_o_grafo_entra_no_projeto_nota_adr.md](56_o_grafo_entra_no_projeto_nota_adr.md) | nota-ADR | 56 — O grafo entra no projeto (Ctrl+S / Ctrl+O) |
| 57 |   | [57_subgrafos_nota_adr.md](57_subgrafos_nota_adr.md) | nota-ADR | 57 — Subgrafos: nesting é uma DOBRA DA VISTA, sobre um grafo que continua PLANO |
| 58 |   | [58_params_dirigidos_nota_adr.md](58_params_dirigidos_nota_adr.md) | nota-ADR | 58 — Params dirigidos por fio (nota-ADR) |
| 59 |   | [59_busca_no_add_menu_nota_adr.md](59_busca_no_add_menu_nota_adr.md) | nota-ADR | 59 — A busca no add-menu (nota-ADR) |
| 60 |   | [60_poisson_e_buoyancy_nota_adr.md](60_poisson_e_buoyancy_nota_adr.md) | nota-ADR | 60 — Poisson-disc e Bóia (nota-ADR) |
| 61 |   | [61_nomes_no_grafo_nota_adr.md](61_nomes_no_grafo_nota_adr.md) | nota-ADR | 61 — Nomes no grafo (nota-ADR) |
| 62 |   | [62_paleta_do_backdrop_nota_adr.md](62_paleta_do_backdrop_nota_adr.md) | nota-ADR | 62 — A paleta do backdrop (nota-ADR) |
| 63 |   | [63_delay_nota_adr.md](63_delay_nota_adr.md) | nota-ADR | 63 — `motion.delay` (nota-ADR) |
| 63 |   | [63_pesquisa_industria_2026_e_plano_estado_da_arte.md](63_pesquisa_industria_2026_e_plano_estado_da_arte.md) | plano | 63 — Pesquisa profunda da indústria + PLANO: Motion Nodes ao estado da arte |
| 64 |   | [64_dock_da_timeline_nota_adr.md](64_dock_da_timeline_nota_adr.md) | nota-ADR | 64 — O dock da timeline (W4.T4) (nota-ADR) |
| 66 |   | [66_fx_de_passe_a_premissa_do_plano_e_FALSA.md](66_fx_de_passe_a_premissa_do_plano_e_FALSA.md) | plano | 66 — FX de passe: **a premissa do plano é falsa** (documento de DECISÃO) |
| 67 |   | [67_fx_de_passe_glow_opcao_B_nota_adr.md](67_fx_de_passe_glow_opcao_B_nota_adr.md) | nota-ADR | 67 — FX de passe: o **glow** é do módulo (Opção B), nó `fx.glow` (nota-ADR) |
| 68 |   | [68_value_curve_nota_adr.md](68_value_curve_nota_adr.md) | nota-ADR | Doc 68 — `value.curve`: o shaper do domínio de VALOR (nota-ADR) |
| 69 |   | [69_value_noise_nota_adr.md](69_value_noise_nota_adr.md) | nota-ADR | Doc 69 — `value.noise`: o driver COERENTE do domínio de VALOR (nota-ADR) |
| 70 |   | [70_value_mix_nota_adr.md](70_value_mix_nota_adr.md) | nota-ADR | Doc 70 — `value.mix`: o crossfader do domínio de VALOR (nota-ADR) |
| 71 |   | [71_value_quantize_nota_adr.md](71_value_quantize_nota_adr.md) | nota-ADR | Doc 71 — `value.quantize`: a escada do domínio de VALOR (nota-ADR) |
| 72 |   | [72_value_gain_nota_adr.md](72_value_gain_nota_adr.md) | nota-ADR | Doc 72 — `value.gain`: o shaper de CONTRASTE / GAMMA do domínio de valor (nota-ADR) |
| 73 |   | [73_value_step_nota_adr.md](73_value_step_nota_adr.md) | nota-ADR | Doc 73 — `value.step`: o GATE / COMPARADOR do domínio de valor (nota-ADR) |
| 74 |   | [74_value_normalize_nota_adr.md](74_value_normalize_nota_adr.md) | nota-ADR | Doc 74 — `value.normalize`: o FIT-TO-RANGE (o 1º reducer do domínio de valor) (nota-ADR) |
| 75 |   | [75_value_unary_nota_adr.md](75_value_unary_nota_adr.md) | nota-ADR | Doc 75 — `value.unary`: o operador de UM argumento do domínio de valor (nota-ADR) |
| 76 |   | [76_value_reduce_nota_adr.md](76_value_reduce_nota_adr.md) | nota-ADR | Doc 76 — `value.reduce`: o reducer GERAL (reduce → broadcast) do domínio de valor (nota-ADR) |
| 77 |   | [77_value_smooth_nota_adr.md](77_value_smooth_nota_adr.md) | nota-ADR | Doc 77 — `value.smooth`: o FILTRO (box blur sobre o índice) do domínio de valor (nota-ADR) |
| 78 |   | [78_value_pattern_nota_adr.md](78_value_pattern_nota_adr.md) | nota-ADR | Doc 78 — `value.pattern`: o STEP SEQUENCER (lista explícita por índice) do domínio de valor (nota-ADR) |
| 79 |   | [79_value_wrap_nota_adr.md](79_value_wrap_nota_adr.md) | nota-ADR | Doc 79 — `value.wrap`: o MODO DE ENDEREÇAMENTO (Clamp/Repeat/Mirror) do domínio de valor (nota-ADR) |
| 80 |   | [80_value_time_nota_adr.md](80_value_time_nota_adr.md) | nota-ADR | Doc 80 — `value.time`: o RELÓGIO animado como produtor do domínio de valor (nota-ADR) |
| 81 |   | [81_value_slope_nota_adr.md](81_value_slope_nota_adr.md) | nota-ADR | Doc 81 — `value.slope`: a DERIVADA (Slope CHOP) do domínio de valor (nota-ADR) |
| 82 |   | [82_value_median_nota_adr.md](82_value_median_nota_adr.md) | nota-ADR | Doc 82 — `value.median`: o filtro de MEDIANA (não-linear) do domínio de valor (nota-ADR) |
| 83 |   | [83_value_percentile_nota_adr.md](83_value_percentile_nota_adr.md) | nota-ADR | Doc 83 — `value.percentile`: o filtro MORFOLÓGICO / de rank do domínio de valor (nota-ADR) |
| 84 |   | [84_value_wave_nota_adr.md](84_value_wave_nota_adr.md) | nota-ADR | Doc 84 — `value.wave`: o SHAPER de forma de onda (o dual do lfo) do domínio de valor (nota-ADR) |
| 85 |   | [85_gradient_editor_nota_adr.md](85_gradient_editor_nota_adr.md) | nota-ADR | Doc 85 — `motion.color_ramp` Custom: o editor de GRADIENTE (nota-ADR) |
| 86 |   | [86_plano_objetos_engine_render_e_preview.md](86_plano_objetos_engine_render_e_preview.md) | plano | 86 — Plano: objetos da engine no grafo, o Duplicator, a ponte de render, e o preview em moldura própria |
| 87 |   | [87_plano_correcao_automatica_setup.md](87_plano_correcao_automatica_setup.md) | plano | 87 — Correção automática de setup do grafo (o app conserta quando o artista erra o lugar do nó) |
| 88 |   | [88_plano_parametros_nos_unidades_e_slider.md](88_plano_parametros_nos_unidades_e_slider.md) | plano | Doc 88 — PLANO: os parâmetros dos nós ganham unidades, slider dual e o conjunto PRO |
| 89 | ◆ | [89_plano_conferencia_dos_nos.md](89_plano_conferencia_dos_nos.md) | plano | 89 — PLANO DE CONFERÊNCIA DOS NÓS (o super-upgrade, nó a nó) |
| — | ◆ | [BUGS_motion_nodes.md](BUGS_motion_nodes.md) | bugs | Bugs do módulo Motion Nodes — registro + soluções |
| — |   | [referencia_catalogo_nodes_minicavalry.md](referencia_catalogo_nodes_minicavalry.md) | referência | Mini Cavalry — Referência de Nós (autor) |
| — |   | [referencia_design_node_graph_ph2d_v1.md](referencia_design_node_graph_ph2d_v1.md) | referência | PH2D — Sistema de Nós — Design Canônico |
| — |   | [referencia_pesquisa_blender_gn.md](referencia_pesquisa_blender_gn.md) | pesquisa | Blender Geometry Nodes 4.x — pesquisa de referência para PH2D Motion Nodes |
| — |   | [referencia_pesquisa_c4d_fields.md](referencia_pesquisa_c4d_fields.md) | pesquisa | Cinema 4D MoGraph + Fields — pesquisa profunda (help.maxon.net) |
| — |   | [referencia_pesquisa_cavalry.md](referencia_pesquisa_cavalry.md) | pesquisa | CAVALRY (Scene Group / cavalry.studio) — Pesquisa profunda v2.7.2 |
| — |   | [referencia_pesquisa_houdini_mops.md](referencia_pesquisa_houdini_mops.md) | pesquisa | PESQUISA: SideFX Houdini + MOPs → referência para Motion Nodes PH2D |
| — |   | [referencia_pesquisa_niagara_stardust.md](referencia_pesquisa_niagara_stardust.md) | pesquisa | Motion-por-nós fora do canvas puro — pesquisa para PH2D (2026-07-24) |
| — |   | [referencia_pesquisa_ui_editores.md](referencia_pesquisa_ui_editores.md) | pesquisa | Mineração UI/UX — TouchDesigner · Nuke · Fusion · Substance Designer · Notch |

**Subpastas:** [`89_conferencia/`](89_conferencia/README.md) · [`handoffs/`](handoffs/README.md)

---

⚠️ Um `Papel` `—` é um **achado**, não um defeito deste índice: é um doc cujo próprio
nome não diz o que ele é. Um arquivo **sem** ◆ não é lixo — é um doc que o roteador
(`CLAUDE.md`) não alcança, e essa era exactamente a medição que criou este índice.

