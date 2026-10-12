# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-12 — F12: O CARTÃO SAI, AS SETAS ⇕ NA LATERAL, O TRAÇO SEM SALTO, A ESPIRAL QUE SEGUE A MÃO

> Fecha **F9 + F10 + F11 + F12 juntas** (nenhuma está no `main`). O que a F9–F11 mudaram, mediram e
> deixaram aberto: [handoff da F11](HANDOFF_INTEGRACAO_line_UIUX_2026-10-11_F11_GIZMOS.md) (que cita o
> da [F10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-11_F10_PARAMETROS.md) e o da F9). Este cobre a F12.
> Pedido do dono palavra por palavra: [`18_prompt…`](../vetor_nova_ui/18_prompt_F12_sem_modais_e_o_traco.md)
> e [`21_prompt…`](../vetor_nova_ui/21_prompt_F12_espiral_cenas_e_fecho.md). O traço: doc 19 §5.
> Janelas intermédias: [F12](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F12.md) ·
> [F12b](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F12b.md) · [F12c](HANDOFF_CONTINUACAO_line_UIUX_2026-10-12_F12c.md).

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, rebaseada sobre o `main` `f1138408e1` em
  12/10 (55 commits, sem conflito; Mergiraf não interveio).
- **`PROJECT_SCHEMA` NÃO mudou (188)** — nenhuma migração.

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

A entrada UI/UX: «o cartão lateral (D10)» → «a fila de ⇕ na lateral (F12)», smoke `…6..19`,
«Último:» → este handoff.

## §1 — O que mudou (F12)

| commit | o quê |
|---|---|
| `dbde7296e5` · `a270b3d73e` | **o traço**: o bico CORTADO (`miter-clip` do SVG 2, `ph2d_vec_scene::miter_clip`) em vez do salto para o chanfro, em cinco leitores (`stroke_uniform`, `visual_bounds`, `ph2d-shape-gpu::contornos_dos_tracos`, *Outline Stroke*, `svg_export`); `PH2D_MITER_CLIP=0` = o chanfro de antes; `MITER_LIMIT` continua `4`. Polígono/estrela sem laço (`untangle`), o *Curve* a ⅓ da corda, o liso G1; o rabo dos balões; o censo `stroke_census_tests` + o gate `no_cooked_shape_loops_or_hooks` (todas as formas × campos × pares) |
| `da15d58283` · `e1542c9122` | a cena `=18` (as formas das fotos do dono, antes × depois com `PH2D_MITER_CLIP=0`); o rectângulo de raio minúsculo sob traço grosso (`miter_clip::tighter_than` → o contorno exacto) |
| `b0d629a800` | no modo Forma, um CLIQUE numa forma existente selecciona-a; arrastar desenha |
| `a2b76d01c4` | **a fila de ⇕** (`Spot::Rail`, `rail_slot`, `RAIL_PITCH` = 1,5 `park_at` = 27 px; `Law::Steps` contínuo = a faixa na altura da caixa, no passo do campo): todo campo do cartão vira ⇕ na borda direita, a ¼ do topo; o polígono recupera o ponto da aresta (`EdgeMid`); o *Smoothing* do rectângulo sai do Vetor (`shapes_units::NOT_SHOWN`); círculo/pizza: o furo de volta e o campo NOVO `Corner round` (`round_slice_corners`) |
| `a1059f2159` | **o cartão SAI** (`ToolBars::card`, `tool_bars_card.rs`, −865 linhas); a etiqueta da alça também sob o cursor, sem arrasto (`shape_handles_live::readout`) |
| `be9798018b` · `fc71a6bc03` | **a espiral**: o *Turns* passa a FRACCIONÁRIO (a ponta segue o ângulo da mão); a cena 16 renomeada (`vector_mode_card` → `vector_mode_estrela`); a cena NOVA `=19` (`vector_mode_setas`) |

## §2 — Medido e decidido (técnico, delegado)

- **A espiral «não funciona» (o dono, 11/10) — medido pelo press/arrasto da shell** (`on::press` /
  `on::drag` sobre uma espiral viva, a ponta de fora levada à volta do centro de 10° em 10°): de +10°
  a +170° o *Turns* ficava `3,000` e a ponta DESENHADA parada às 12h; a +180° saltava para `4`. Causa:
  a `Law::Winding` arredondava `wound/360` e o campo era `Count`. **Cura:** `TURNS` = `Ratio`
  (passo da caixa ¼), `spiral_ex(turns: f64)`, `Winding` = `at_press + wound/360` (Shift: a ponta a
  15°). Depois: +30° → 3,083, +90° → 3,250, a ponta no ângulo da mão a < 1e-12.
  **O oráculo** (Inkscape 1.4.4, `oraculo_alcas/fixtures/spiral.json`) agora sem `round`: as 6
  experiências da ponta de fora a < 1e-3 (o Ctrl dele prende a 15° — +50° → 3,125 — é o nosso Shift).
- **Save antigo coze igual:** voltas inteiras com expansão e interior a zero delegam em `spiral()`
  byte a byte; com expansão/interior, o número de passos de uma volta inteira não muda
  (`ceil(t·16·⌈q⌉)` = `t·16·⌈q⌉`). Gate novo `a_fractional_spiral_ends_at_its_angle_and_meets_the_whole_one`
  (a ponta a 3,25 às 9h; 3,0001 ≈ 3 a < 2e-3).
- **O que parecia um 2.º defeito não era:** na sonda, o 2.º press na MESMA alça ao mesmo instante
  repunha 3 voltas — é o duplo-clique (repõe o valor de fábrica, de propósito).
- **A cena 19 solta o ⇕ no fim:** a etiqueta mora junto do CURSOR (o `last_pointer` da shell); um ⇕
  deixado agarrado pela cena saltaria ao 1.º movimento do rato do dono (a lei mede desde o press da
  cena). O gate lê a etiqueta com o cursor SOBRE o ⇕ (`Expansion: 0.50`).
- **A 1.ª seta e a alça de canto numa caixa baixa (pedido do prompt: medir):** o quadrado de canto
  tem 12 px (`HANDLE_SIZE_PX`), o ⇕ ~22 px desenhado (24 px de acerto); o 1.º ⇕ fica a `h/4` abaixo do
  canto. **Tocam-se com a caixa abaixo de ~62 px de altura no ecrã** (≈ 0,6 m no zoom 100 %). Nas
  fotos (caixas de 160–185 px) ficam a 40–46 px. Não curado: decisão de desenho (ver §7).

## §3 — Foundational tocado e contratos

- **Campo novo APENSO** `Corner round` (Ellipse/Pie, slot 3, Px, neutro em zero).
- O slot 4 do `RoundRect` (*Smoothing*) **escondido mas vivo**: o Motion (`motion_shape_gen`) escreve-o
  e o `cook` honra-o; um save com squircle continua squircle, sem chip.
- `ToolBars::card` removido de `ph2d-editor-core` (`selection_gap()` mora em `float_bar.rs`).
- `ph2d_vec_scene::spiral_ex` mudou de assinatura (`turns: u32` → `f64`). Chamadores nesta árvore:
  `kind.rs` e os testes; varrido em 12/10 nas outras worktrees vivas (`line/Vector` `e66c250fc2`
  incluída): **zero** chamadores. Se aparecer um na árvore combinada, passa-se `f64`.
- `TURNS` passou de `Count` a `Ratio` (o `clamp` deixou de arredondar as voltas).
- Herdado da F12 A: `ph2d-vector` reexporta `dash`; `ph2d-shape-gpu` depende do `ph2d-vec-render`;
  `PathTess::agudo` (crate-privado). Contratos congelados (§6): nenhum tocado.

## §4 — O que só a árvore combinada pega

- O aviso do Miter que a `line/Vector` deixou no handoff dela (11/10) — o bico cortado é desta linha.
- Gates que leem a shell como texto (`the_shape_handles_are_drawn_and_dragged`) — re-correr.

## §5 — Provas de mutação

Agente `mutacao`, 1× ANTES do portão, sobre `fc71a6bc03`; controlos verdes: tool-vector lib 96 ·
vec-scene lib 529 · app-vec lib 889 · panel-vector 264 + 45 · vec-render lib 152; toda âncora casou
1×; nenhuma mutação deixou de compilar. **42 mutações: 26 sangraram, 16 sobreviveram** — cada
sobrevivente ganhou um teste provado VERMELHO sob ela (tabela abaixo, «curada»).

| id | cláusula | mutação | quem sangra |
|---|---|---|---|
| A1 | `Law::Winding` sem `round` | `(deg/360).round()` | `the_spiral_turns_follow_the_angle_of_the_hand` · `…_winding_like_inkscape` |
| A2 | o Shift do `Winding` | sem `snap` | os mesmos |
| A3a/b | `spiral_ex` delega só com voltas inteiras | sem `fract()` / nunca delega | `a_fractional_spiral_ends_at_its_angle_…` / `the_new_fields_at_zero_are_the_old_shapes_byte_for_byte` + 1 |
| A5 | o `cook` sem `round` | `.round()` em `f(v,0)` | `setas_tests::the_scene_moves_one_handle_of_each_shape_…` |
| B1 | `RAIL_PITCH` | 1,5 → 1,0 | `a_press_grabs_only_on_a_handle` |
| B2 | `rail_slot` | sempre 0 | 5 (f12, live, cenas 16 e 19) |
| B3 · B4a | o `Steps` contínuo | `int` forçado · passo ignorado | `a_continuous_arrow_runs_the_whole_range_in_the_box_height` + cena 19 |
| B5 | `NOT_SHOWN` | vazia | `the_fields_without_a_gesture_are_counted_not_listed` + 1 |
| B8 · B9 | `corner_at` · o `hot` desenhado | tirados | `the_hot_halo_counts_the_drawn_points` |
| B10 | `readout` sob o cursor | `None` sem grab | `the_readout_follows_the_grabbed_handle` + cena 19 |
| C1b · C4 · C5b · C6 · C7 · C9a/b | `untangle` 1,3 · G1 · `NEEDLE_COS` 0,2 · raiz · `TIP_FLOOR` · a régua do gancho | | `no_cooked_shape_loops_or_hooks` (C5c: `every_shape_declares_which_way_is_up_…`) |
| D1b · D2 · D3 · D4 · D5a | o limite · o lado · a recta de corte · `oriented` · `tighter_than` | | os gates do `miter_clip` / `every_sharp_join_of_every_shape_is_cut_not_bevelled` / `a_tiny_round_corner_…` |
| C3a | o período do `untangle` | procura tudo | **equivalente** (só custo: 91 s contra 50 s) |

Sobreviventes — cada um curado com um teste VERDE no código e VERMELHO sob a mutação (provado):

| id | cláusula | teste novo |
|---|---|---|
| A4 | `spiral_ex` passos com `ceil` | `polystar_tests::a_fractional_spiral_keeps_sixteen_cubics_per_turn` |
| A6 | `TURNS` = `Ratio` | `shapes_tests::clamp_keeps_the_fractional_spiral_turns` |
| B6 · B7 | `round_slice_corners`: só quinas `Corner` · `r ≤ 0` | `corner_tests::only_the_slice_corners_round_the_smooth_arc_vertices_stay` (numa elipse 2,5 × 13 os vértices lisos do arco têm alças não colineares no último bit: 199 fatias perdiam um vértice) · `a_negative_slice_radius_leaves_the_slice_intact` |
| C1 | `untangle` `MARGIN` 0,97 | `polystar_tests::the_untangled_shape_keeps_a_margin_below_the_crossing` |
| C2 | a quase-cúspide (`reverses`) | `polystar_tests::a_cusp_inside_one_segment_counts_as_a_crossing` |
| C3b | `segs % period == 0` | `polystar_tests::a_period_that_does_not_divide_the_contour_searches_all_of_it` |
| C5 | `NEEDLE_COS` 0,7 | `bubbles_tests::the_tail_tip_is_a_needle_only_above_the_needle_measure` |
| C8 | o losango da seta dupla sem haste | `arrows_tests::a_double_arrow_without_a_shaft_is_a_rhombus` |
| D1a | `half_w ≤ 0` | `miter_clip_tests::a_join_without_width_adds_nothing` |
| D5b | `agudo` por `any_sharp` | `stroke_uniform_tests::a_sharp_polygon_under_a_thin_pen_is_flagged_by_its_corners` |
| D6a · D6b | o tecto da quina · o ramo `closing` | `polystar_tests::the_curve_stops_at_the_cusp_of_the_corner_it_closes` |
| D7 | `chord` ⅓ | `polystar_tests::the_bent_edge_handles_sit_a_third_along_the_chord` |
| B4b | `step_world` × `px_to_world` | **equivalente hoje**: nenhum ⇕ move um campo `Px` (sondado em todas as formas) — o factor é sempre 1. Um ⇕ futuro em `Px` precisa do teste. |

Suítes depois: vec-scene lib 540 · vec-render lib 153 · tool-vector lib 97.

## §6 — Smoke (do dono)

O relatório ao dono (passos numerados) está na resposta final da janela. Cenas `=15` (alças), `=16`
(estrela e fila de ⇕), `=17` (a caixa por fora do traço), `=18` (o traço das fotos dele; com e sem
`PH2D_MITER_CLIP=0`) e `=19` (as setas lado a lado; `PH2D_CENA19_FICA=0..3` escolhe a forma
seleccionada). Fotos: `/var/tmp/ph2d_F12_cenas/`.

## §7 — O que fica ABERTO

1. **Perguntas ao dono (produto), sem resposta:** (a) o bico cortado (doc 19 §5, a/b/c; fotos
   `/var/tmp/ph2d_F12_cena18/`); (b) a *Resize Box* mantém a espessura do traço?; (c) o ⇕ do *Viewed*
   dos isométricos: subir dá «De baixo» — recomendado inverter só ele; (d) o rectângulo gravado antes
   com *Smoothing* continua suave sem controlo — recomendado aceitar.
2. A 1.ª seta encosta na alça de canto em caixas < ~62 px (§2). Cura possível: com a caixa baixa, a
   fila desce para o meio da borda (entre o canto e a alça do meio) — decidir com o dono.
3. A placa da GPU do Motion (`contorno_cpu.rs` + shaders) ainda salta para o chanfro — só depois de o
   dono aprovar o bico cortado.
4. Herdados da F9–F11: ver o §7 do handoff da F11.

## §8 — Portão, binário e `agent-loop-profile.sh`

(a preencher)
