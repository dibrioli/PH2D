---
name: project-painter-w4-spatial-gpu-bloom-sh
description: "Bloom + Shadows/Highlights GPU no pass-graph do ph2d-render: reconciliar bit-a-bit contra as fns CPU canônicas do impl (dev-dep), não duplicar a math; render PRONTO, impl liga via gpu_spatial_code"
metadata: 
  node_type: memory
  type: project
  originSessionId: 00710036-c40d-4ddb-8905-3f77e41f6f0c
---

**Bloom + S/H GPU (Painter W4, 2026-06-06)** — acelerei os 2 kinds no pass-graph segmentado do `ph2d-render` (commits `ff70ad2` Bloom, `37a06d4` S/H, handoff `docs/HANDOFF_painter_w4_bloom_sh_gpu_done_coord.md`).

**Padrão durável (3 lições):**
1. **Reconciliar contra a fn CPU canônica DIRETO**, não contra um mirror hand-rolled: o teste materializa o below-composite, chama `apply_bloom`/`apply_shadows_highlights` (do `ph2d-painter-brush`, **dev-dep** do render — sem acoplar libs) com blend=Normal+opacity=1, e compara. S/H bateu **byte-exato (diff 0)** no Metal; Bloom ≤5B. Decoupling preservado.
2. **Reuso > reinventar**: Bloom = bright-pass nova + blur separável COMPARTILHADO (`premul_read=0`) + branch `COMBINE_BLOOM`. S/H = luma-extract + 2 blurs escalares (luma em `.r`, reusa `cs_blur`) + combine própria (`cs_combine_sh`, cobertura preservada, pula a `run_combine` compartilhada).
3. **Contrato sem gate = dono decide**: S/H precisa de 8 params → `LayerOp::SpatialAdjustment.params` alargou `[f32;4]→[f32;8]`. Só os testes do render + o flatten do shell (`painter_gpu_flatten.rs`, Coord) constroem `SpatialAdjustment`; a crate do impl tem só `gpu_spatial_code()`/`spatial_params()`. Sem arch-gate → alarguei sem tocar a crate do impl (os 4 kinds atuais zero-padam).

**Pyramid p/ Bloom NÃO byte-bate** Gaussiana separável real (`apply_bloom`) → usei a separável exata; pyramid = otimização CONJUNTA futura (CPU ref teria de virar pyramid também).

**Noise + Halftone (commit `84f559e`)** fecharam a mesma onda W4-GPU mas por caminho DIFERENTE: não são spatial (sem vizinhança), são per-pixel COORDENADA-dependentes (lêem gx,gy absoluto) → vão no `cs_flat`/`apply_adjustment`, que passou a receber `coord` (threaded em cs_flat/cs_grouped/cs_segment; kinds coord-independentes ignoram → 11 gates existentes byte-idênticos). `ADJ_NOISE=9`/`ADJ_HALFTONE=10`. Noise: hash inteiro (`hash_u32`/`rand01`) **bit-idêntico** CPU↔GPU (u32 wrapping, sem transcendental) → diff 0. Halftone: threshold duro + sin/cos/fract → gate por FRAÇÃO (observei 0/9216 flips). Sem pipeline/textura nova. Sem Noise/Halftone-GPU, uma layer dessas força o preview INTEIRO pro CPU fallback.

**Falta (impl):** flipar `gpu_spatial_code(Bloom)→Some(4)`, `(ShadowsHighlights)→Some(5)` + `spatial_params` (S/H alarga pra 8); e `gpu_code(Noise)→Some(9)`, `(Halftone)→Some(10)` + `gpu_params` (cast do discriminante `NoiseKind`/`HalftoneShape`) — idêntico aos kinds já wirados. `feathers_coverage()` já espelhado por construção no GPU. Relacionado: [[feedback_pipeline_inject_dont_cap]] [[feedback_app_ui_english_only]] [[project_painter_composite_perf_2026_06_03]].
