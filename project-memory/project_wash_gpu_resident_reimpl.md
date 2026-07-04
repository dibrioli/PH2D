---
name: project-wash-gpu-resident-reimpl
description: "Wash watercolor reimplementation — GPU-first/real-time-only, port the solved backup physics onto the resident canvas, don't re-fight B1-B9"
metadata: 
  node_type: memory
  type: project
  originSessionId: 4e51f187-9840-4a3b-9378-185be66e06bf
---

Reimplementação do Wash (aquarela) iniciada 2026-06-14 sob plano `docs/plans/2026-06-14-wash-gpu-resident.md`.

**Princípio durável (Enio):** o Painter é ferramenta de **runtime tempo-real com params animáveis** (game engine 2D). Logo **GPU-first, tempo-real-only — ZERO fallback CPU**: se a CPU não sustenta o recurso em tempo real, o recurso não existe nessa forma. Isso **supersede** a recomendação "gate pro CPU" da avaliação de gaps GPU.

**Tese:** o backup `backups/wash_2026-06-14/crates/ph2d-painter-wash/` é o solver **já depurado** (B1–B9 vencidos — ver `docs/Painter_projeto/wash_solucao_de_erros.md`). O que falhou foi **topologia/perf** (storage-buffers, submit/copy-bound), não física. Reimplementação = casar a física resolvida com a topologia **GPU-residente single-submit** do [[project-watercolor-v2-gpu-first-refactor]] / ADR-0093 (canvas residente Fase 1/2 já construído: `painter_canvas_gpu.rs` + `wash_pipeline.rs`). **Portar B1–B9, não re-lutar.**

**Decisões travadas:** campos (water/pig/dye/res/paper) em **textura** (não buffer — foge do limite de 8 storage-buffers, dá bilinear, alinha com o canvas-GPU); porto com banda ULP vs o backup como oráculo. Execução = **eu dirijo fase a fase**, parando no checkpoint visual de cada fase (aquarela é perceptual — bench-verde ≠ vivo, lição 7 do postmortem).

**Fases:** W0 ADR+scaffold → W1 núcleo seco (B1/B5b/B6/CFL) → W2 água+bordas (B3/B4/B5, evap-0 caso primário) → W3 cor Mixbox residual (B9) → W4 undo de campo no histórico transacional (B7/B8) → W5 revisão de params → W6 tempo-real 4K. Loop por fase: impl → 2 lentes de auditoria → gate headless → checkpoint Enio.

Undo de layers já virou transacional (enum `Stroke`/`Structural` em `crates/ph2d-tool-painter/src/undo.rs`); o undo de campo do wash (W4) integra nesse enum.
