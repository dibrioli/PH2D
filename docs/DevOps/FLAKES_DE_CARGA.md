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
| `riding_the_path_costs_about_twice_the_straight_layout` | [`shells/desktop`](../../shells/desktop/src/text_path_smoke.rs) | razão de relógios |

Uma família entra por padrão e não por nome: `test(/orcamento::the_fit_/)` (`ph2d-app-flip`).

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
