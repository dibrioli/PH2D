# `Painter` — handoffs (registro cronológico de sessão)

> **O que é esta pasta:** o registro de **como** o módulo foi construído — um arquivo por
> sessão de linha. O **pensamento** do módulo (planos, pesquisas, `BUGS_*`) fica **um nível
> acima**, em [`docs/Painter/`](..).
>
> ⚠️ **Isto NÃO é o estado atual do módulo.** O estado vivo é o **[`CLAUDE.md §5`](../../../CLAUDE.md)**;
> um handoff descreve o mundo **no dia em que foi escrito** e não é atualizado depois. Use-os
> para responder *"por que isto ficou assim?"* — nunca para decidir a próxima ação.

**68 handoffs** · **24** citados pelo CLAUDE.md §5 (marcados **◆** — são os que a
§5 aponta como o detalhe de mecanismo de uma integração).

| Data | | Arquivo | Papel | Assunto |
|---|---|---|---|---|
| — | ◆ | [HANDOFF_brush_jitter_color.md](HANDOFF_brush_jitter_color.md) | trabalho | Brush: Randomize Color + Jitter Scale + Jitter Rotate |
| — |  | [HANDOFF_line_Painter_INTEGRACAO.md](HANDOFF_line_Painter_INTEGRACAO.md) | integração | HANDOFF de INTEGRAÇÃO — line/Painter (2026-07-13) |
| — |  | [HANDOFF_line_painter_watercolor_integration.md](HANDOFF_line_painter_watercolor_integration.md) | trabalho | Handoff de integração — linha line/Painter (Watercolor, jornada 2026-07-10/11) |
| — | ◆ | [HANDOFF_painter_brush_engine.md](HANDOFF_painter_brush_engine.md) | trabalho | Painter / Brush Engine (tracker ÚNICO do módulo) |
| — |  | [HANDOFF_painter_falloff_curve.md](HANDOFF_painter_falloff_curve.md) | trabalho | Painter Custom Falloff curve + a dispatch regression + an FPS drop |
| — |  | [HANDOFF_painter_stroke_section.md](HANDOFF_painter_stroke_section.md) | trabalho | Painter "Stroke" section (Blender clean-room) — landed, but 4 behavioral gaps open |
| — |  | [HANDOFF_painter_texture_section.md](HANDOFF_painter_texture_section.md) | trabalho | Painter Texture section (Brush texture, Blender-parity, 2D-adapted) |
| — | ◆ | [HANDOFF_per_layer_color_perf_artifacts.md](HANDOFF_per_layer_color_perf_artifacts.md) | trabalho | Per-Layer Color (layers-as-brush): slowness + rectangular stripe artifacts |
| — |  | [HANDOFF_rake_rewrite.md](HANDOFF_rake_rewrite.md) | trabalho | Arrancar o Rake atual e reescrever do zero (sofisticado, moderno, funcional) |
| — |  | [HANDOFF_rendering_modes_wet_mix.md](HANDOFF_rendering_modes_wet_mix.md) | trabalho | Rendering Modes + Wet Mix (PH2D Painter) |
| — |  | [HANDOFF_selection.md](HANDOFF_selection.md) | trabalho | Painter Selection system (tracker vivo) |
| — |  | [HANDOFF_selection_curve_parity.md](HANDOFF_selection_curve_parity.md) | trabalho | Selection curve = IDENTICAL to the stroke Shape curve system |
| — |  | [HANDOFF_shape_grain_dual_texture.md](HANDOFF_shape_grain_dual_texture.md) | trabalho | Dois slots de textura no Painter: Shape + Grain (paridade Procreate) |
| — |  | [HANDOFF_stroke_multishape.md](HANDOFF_stroke_multishape.md) | trabalho | Stroke Multi-Shape (Enio 2026-07-04) |
| — |  | [HANDOFF_watercolor_tiling_shape_overlay.md](HANDOFF_watercolor_tiling_shape_overlay.md) | trabalho | Watercolor Tiling: shape overlay/edit na costura + fila pendente |
| 2026-07-09 |  | [HANDOFF_line_Painter_integracao_2026-07-09.md](HANDOFF_line_Painter_integracao_2026-07-09.md) | integração | HANDOFF de integração — linha line/Painter (aquarela: junções + sessão molhada) — 2026-07-09 |
| 2026-07-11 |  | [HANDOFF_line_Painter_continuacao_2026-07-11.md](HANDOFF_line_Painter_continuacao_2026-07-11.md) | continuação | continuação da linha line/Painter (pós-integração 2026-07-11) |
| 2026-07-11 |  | [HANDOFF_line_Painter_integracao_2026-07-11.md](HANDOFF_line_Painter_integracao_2026-07-11.md) | integração | HANDOFF de integração — linha line/Painter (Tiling seamless + Paper/Grain + Bug #11 ABERTO) — 20… |
| 2026-07-12 |  | [HANDOFF_line_Painter_impasto_2026-07-12.md](HANDOFF_line_Painter_impasto_2026-07-12.md) | trabalho | line/Painter · Impasto (#16) · 2026-07-12 |
| 2026-07-12 |  | [HANDOFF_line_Painter_integracao_2026-07-12.md](HANDOFF_line_Painter_integracao_2026-07-12.md) | integração | HANDOFF de integração — line/Painter (2026-07-12) |
| 2026-07-12 |  | [HANDOFF_line_Painter_integracao_2026-07-12_FECHAMENTO.md](HANDOFF_line_Painter_integracao_2026-07-12_FECHAMENTO.md) | integração | HANDOFF de INTEGRAÇÃO — line/Painter (fechamento, 2026-07-12) |
| 2026-07-13 |  | [HANDOFF_line_Painter_continuacao_2026-07-13.md](HANDOFF_line_Painter_continuacao_2026-07-13.md) | continuação | line/Painter: continuação do Impasto (2026-07-13) |
| 2026-07-13 |  | [HANDOFF_line_Painter_sculpt_2026-07-13.md](HANDOFF_line_Painter_sculpt_2026-07-13.md) | trabalho | line/Painter: o SCULPT do relevo (2026-07-13) |
| 2026-07-13 |  | [HANDOFF_line_Painter_sculpt_integracao_2026-07-13.md](HANDOFF_line_Painter_sculpt_integracao_2026-07-13.md) | integração | HANDOFF do SCULPT — detalhe técnico (W1 + W2 + W3, 2026-07-13) |
| 2026-07-14 |  | [HANDOFF_line_Painter_continuacao_2026-07-14.md](HANDOFF_line_Painter_continuacao_2026-07-14.md) | continuação | HANDOFF de CONTINUAÇÃO — line/Painter (2026-07-14) |
| 2026-07-15 |  | [HANDOFF_line_Painter_continuacao_2026-07-15.md](HANDOFF_line_Painter_continuacao_2026-07-15.md) | continuação | HANDOFF de CONTINUAÇÃO — line/Painter (2026-07-15) |
| 2026-07-15 |  | [HANDOFF_line_Painter_integracao_2026-07-15.md](HANDOFF_line_Painter_integracao_2026-07-15.md) | integração | HANDOFF de INTEGRAÇÃO — line/Painter (2026-07-15) |
| 2026-07-15 | ◆ | [HANDOFF_line_Painter_push_rim_anchor_2026-07-15.md](HANDOFF_line_Painter_push_rim_anchor_2026-07-15.md) | trabalho | o aro do Push ancora no CORPO da tinta, não no círculo do gizmo (line/Painter, 2026-07-15) |
| 2026-07-16 | ◆ | [HANDOFF_line_Painter_inflate_edges_2026-07-16.md](HANDOFF_line_Painter_inflate_edges_2026-07-16.md) | trabalho | a borda do Inflate (line/Painter, 2026-07-16) |
| 2026-07-17 |  | [HANDOFF_line_Painter_integracao_2026-07-17.md](HANDOFF_line_Painter_integracao_2026-07-17.md) | integração | HANDOFF de INTEGRAÇÃO — line/Painter (2026-07-17) |
| 2026-07-18 |  | [HANDOFF_INTEGRACAO_line_Painter_2026-07-18.md](HANDOFF_INTEGRACAO_line_Painter_2026-07-18.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter (2026-07-18, 2ª rodada) |
| 2026-07-18 |  | [HANDOFF_line_Painter_TAKEOVER_2026-07-18.md](HANDOFF_line_Painter_TAKEOVER_2026-07-18.md) | troca de agente | assumindo a line/Painter (2026-07-18) |
| 2026-07-18 | ◆ | [HANDOFF_line_Painter_gpu_light_2026-07-18.md](HANDOFF_line_Painter_gpu_light_2026-07-18.md) | trabalho | a LUZ do impasto roda na GPU (2026-07-18) |
| 2026-07-18 | ◆ | [HANDOFF_line_Painter_inflate_closing_2026-07-18.md](HANDOFF_line_Painter_inflate_closing_2026-07-18.md) | trabalho | Inflate: o footprint virou um FECHAMENTO MORFOLÓGICO (2026-07-18) |
| 2026-07-18 | ◆ | [HANDOFF_line_Painter_smear_field_2026-07-18.md](HANDOFF_line_Painter_smear_field_2026-07-18.md) | trabalho | o Smear virou um campo (2026-07-18) |
| 2026-07-19 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_rake_lag_random_angle_2026-07-19.md](HANDOFF_INTEGRACAO_line_Painter_rake_lag_random_angle_2026-07-19.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter: lag do Rake + remoção do Random Angle (2026-07-19) |
| 2026-07-19 |  | [HANDOFF_line_Painter_TAKEOVER_2026-07-19.md](HANDOFF_line_Painter_TAKEOVER_2026-07-19.md) | troca de agente | TAKEOVER da line/Painter (2026-07-19) |
| 2026-07-19 | ◆ | [HANDOFF_line_Painter_impasto_unified_tools_2026-07-19.md](HANDOFF_line_Painter_impasto_unified_tools_2026-07-19.md) | trabalho | Impasto: uma casa só para as dez ferramentas (line/Painter, 2026-07-19) |
| 2026-07-19 | ◆ | [HANDOFF_line_Painter_rotation_model_2026-07-19.md](HANDOFF_line_Painter_rotation_model_2026-07-19.md) | trabalho | line/Painter: o modelo de rotação (Blender × nosso) — 2026-07-19 |
| 2026-07-19 | ◆ | [HANDOFF_line_Painter_shape_flow_2026-07-19.md](HANDOFF_line_Painter_shape_flow_2026-07-19.md) | trabalho | line/Painter: Shape FLOW (o padrão segue o traço) — 2026-07-19 |
| 2026-07-20 | ◆ | [HANDOFF_line_Painter_wet_paint_2026-07-20.md](HANDOFF_line_Painter_wet_paint_2026-07-20.md) | trabalho | line/Painter: integração do módulo WET PAINT (física real, estilo Rebelle) |
| 2026-07-20 | ◆ | [HANDOFF_line_Painter_wet_paint_continuacao_2026-07-20.md](HANDOFF_line_Painter_wet_paint_continuacao_2026-07-20.md) | continuação | line/Painter: Wet Paint, W0 FECHADO → continuação (W1..W3) |
| 2026-07-21 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_wet_paint_2026-07-21.md](HANDOFF_INTEGRACAO_line_Painter_wet_paint_2026-07-21.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter: a jornada WET PAINT (2026-07-21) |
| 2026-07-22 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_wet_tuning_2026-07-22.md](HANDOFF_INTEGRACAO_line_Painter_wet_tuning_2026-07-22.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter · Wet Paint (doc 22/23) + reorg do Impasto + o modo de pint… |
| 2026-07-23 |  | [HANDOFF_INTEGRACAO_line_Painter_gpu_ondas_1_2_2026-07-23.md](HANDOFF_INTEGRACAO_line_Painter_gpu_ondas_1_2_2026-07-23.md) | integração | Handoff de integração — line/Painter · GPU Ondas 1 e 2 (o compositor para de recusar o documento… |
| 2026-07-23 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_wet_transfer_2026-07-23.md](HANDOFF_INTEGRACAO_line_Painter_wet_transfer_2026-07-23.md) | integração | Handoff de integração — line/Painter · transferência sRGB tabelada |
| 2026-07-24 |  | [HANDOFF_INTEGRACAO_line_Painter_gpu_onda5a_paint_nocopy_2026-07-24.md](HANDOFF_INTEGRACAO_line_Painter_gpu_onda5a_paint_nocopy_2026-07-24.md) | integração | Handoff de integração — line/Painter · Onda 5a (a pintura para de copiar o canvas por movimento) |
| 2026-07-24 |  | [HANDOFF_INTEGRACAO_line_Painter_gpu_onda5b_partial_layer_upload_2026-07-24.md](HANDOFF_INTEGRACAO_line_Painter_gpu_onda5b_partial_layer_upload_2026-07-24.md) | integração | Handoff de integração — line/Painter · Onda 5b (o compositor GPU re-envia só a região suja) |
| 2026-07-24 |  | [HANDOFF_INTEGRACAO_line_Painter_gpu_onda5c_mask_partial_lane_2026-07-24.md](HANDOFF_INTEGRACAO_line_Painter_gpu_onda5c_mask_partial_lane_2026-07-24.md) | integração | Handoff de integração — line/Painter · Onda 5c (o traço de máscara toma a via parcial) |
| 2026-07-25 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_2026-07-25.md](HANDOFF_INTEGRACAO_line_Painter_2026-07-25.md) | integração | Handoff de INTEGRAÇÃO — line/Painter, a jornada de 2026-07-23..25 |
| 2026-07-25 |  | [HANDOFF_INTEGRACAO_line_Painter_impasto_fold_2026-07-25.md](HANDOFF_INTEGRACAO_line_Painter_impasto_fold_2026-07-25.md) | integração | Handoff de integração — line/Painter: o fold do impasto anda o retângulo sujo |
| 2026-07-25 |  | [HANDOFF_line_Painter_gpu_continuacao_2026-07-25.md](HANDOFF_line_Painter_gpu_continuacao_2026-07-25.md) | continuação | line/Painter: levar o Painter para a GPU (continuação) |
| 2026-07-25 |  | [HANDOFF_line_Painter_mask_rewrite_2026-07-25.md](HANDOFF_line_Painter_mask_rewrite_2026-07-25.md) | trabalho | REESCREVER a máscara de proteção do zero (com referência de alta qualidade) |
| 2026-07-26 |  | [HANDOFF_INTEGRACAO_line_Painter_perf_2026-07-26.md](HANDOFF_INTEGRACAO_line_Painter_perf_2026-07-26.md) | integração | Handoff de integração — line/Painter (jornada de PERFORMANCE), 2026-07-26 |
| 2026-07-26 |  | [HANDOFF_INTEGRACAO_line_Painter_undo_delta_2026-07-26.md](HANDOFF_INTEGRACAO_line_Painter_undo_delta_2026-07-26.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter: o histórico de undo guarda a JANELA (U1) |
| 2026-07-26 |  | [HANDOFF_line_Painter_undo_delta_2026-07-26.md](HANDOFF_line_Painter_undo_delta_2026-07-26.md) | trabalho | line/Painter: o histórico de undo guarda um DOCUMENTO por passo |
| 2026-07-28 |  | [HANDOFF_INTEGRACAO_line_Painter_undo_journal_2026-07-28.md](HANDOFF_INTEGRACAO_line_Painter_undo_journal_2026-07-28.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter: o journal por tile (S3, degraus 1–3b) + o Wet Paint a 4 FPS |
| 2026-07-30 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_FECHAMENTO_2026-07-30.md](HANDOFF_INTEGRACAO_line_Painter_FECHAMENTO_2026-07-30.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter FECHADA (2026-07-30) |
| 2026-07-30 |  | [HANDOFF_INTEGRACAO_line_Painter_wet_perf_2026-07-30.md](HANDOFF_INTEGRACAO_line_Painter_wet_perf_2026-07-30.md) | integração | Handoff de integração — line/Painter, a frente de PERFORMANCE do Wet Paint (2026-07-30) |
| 2026-08-01 |  | [HANDOFF_INTEGRACAO_line_Painter_S3_2026-08-01.md](HANDOFF_INTEGRACAO_line_Painter_S3_2026-08-01.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter · 2026-08-01 |
| 2026-08-01 |  | [HANDOFF_INTEGRACAO_line_Painter_wet_flow_e_o_cap_do_pincel_2026-08-01.md](HANDOFF_INTEGRACAO_line_Painter_wet_flow_e_o_cap_do_pincel_2026-08-01.md) | integração | Handoff de integração — line/Painter · o campo de fluxo, os três instrumentos e o CAP DO PINCEL |
| 2026-08-01 |  | [HANDOFF_line_Painter_S3_journal_2026-08-01.md](HANDOFF_line_Painter_S3_journal_2026-08-01.md) | trabalho | line/Painter · S3: o journal vira a fonte do before do RELEVO |
| 2026-08-02 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_MESTRE_2026-08-02.md](HANDOFF_INTEGRACAO_line_Painter_MESTRE_2026-08-02.md) | integração | HANDOFF DE INTEGRAÇÃO MESTRE — line/Painter (2026-08-02) |
| 2026-08-02 |  | [HANDOFF_INTEGRACAO_line_Painter_watercolor_cadence_2026-08-02.md](HANDOFF_INTEGRACAO_line_Painter_watercolor_cadence_2026-08-02.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter, a cadência da aquarela (2026-08-02) |
| 2026-08-03 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_carimbo_2026-08-03.md](HANDOFF_INTEGRACAO_line_Painter_carimbo_2026-08-03.md) | integração | Handoff de integração — line/Painter, o CARIMBO (2026-08-03 · 2026-08-04) |
| 2026-08-06 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_bow_wave_2026-08-06.md](HANDOFF_INTEGRACAO_line_Painter_bow_wave_2026-08-06.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter, o bow wave gateado no knob + as bandas por trabalho (2026-… |
| 2026-08-08 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_MESTRE_2026-08-08.md](HANDOFF_INTEGRACAO_line_Painter_MESTRE_2026-08-08.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter, MESTRE (2026-08-08) |
| 2026-08-09 | ◆ | [HANDOFF_INTEGRACAO_line_Painter_grid_stamp_2026-08-09.md](HANDOFF_INTEGRACAO_line_Painter_grid_stamp_2026-08-09.md) | integração | HANDOFF DE INTEGRAÇÃO — line/Painter, 2026-08-09 |

---
*Índice gerado na arrumação de 2026-08-10 (DIRETRIZ §1.5.9). Handoff novo entra aqui, não na
raiz de `docs/`.*
