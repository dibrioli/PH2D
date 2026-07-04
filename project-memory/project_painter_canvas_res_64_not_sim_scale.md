---
name: project-painter-canvas-res-64-not-sim-scale
description: "Painter edits sprites at native res; demo sprites are 64×64 — \"low-res\" render quality is gated by canvas size, not the fluid sim/shader scale"
metadata: 
  node_type: memory
  type: project
  originSessionId: 17ce2a9d-e337-4c27-8c97-12e1c154443e
---

O painter PH2D edita o sprite selecionado na resolução **nativa** dele (`read_sprite_source` usa `img.width/height`, **sem upscale**; `shells/desktop/src/hero_intents/texture_edit.rs`). Os sprites de demo do atlas são **64×64** (`ATLAS_SPRITE_PX=64`, `shells/desktop/src/integration.rs`). Então a aquarela GPU (W15.3 / [ADR-0049](docs/architecture/decisions/0049-fluid-brushes.md)), mesmo em "full-res" (`scale=1`), roda num grid 64×64 — minúsculo quando o canvas é exibido a ~800px (~12× zoom) → bordas macias viram borrão ("baixa resolução nas bordas").

**Why:** custou ~5 rounds caçando o pipeline da aquarela (sub-rect apply, 2×2 AA, full-res, envelope monotônico) quando o "low res edges" era simplesmente o canvas de 64px. Brush DURO parece nítido em 64px (blocos com transição seca); aquarela MACIA vira mush. A escala do sim (1 vs 2) é irrelevante aí (64 vs 32, ambos minúsculos). O Enio ratificou ("smoke ok. Corrigiu!") só depois de entender que era o canvas.

**How to apply:** quando um render parecer "baixa resolução", cheque PRIMEIRO a resolução real do canvas/source (`source_size` / sprite nativo), ANTES de mexer na escala do sim/shader. Pra testar alta-res no painter: **arraste um PNG grande** (importa via `DroppedFile` → `import_image_at_camera` na res nativa). Gap em aberto: não há "novo canvas" grande dedicado — o painter ("sucessor do Procreate") só edita sprites do atlas (64×64) ou imagens importadas; um canvas 1024²/2048² é trabalho futuro. Lição irmã do W15.3: `water_bbox ≠ extensão do pigmento` sob evaporação (o composite GPU usa o **envelope molhado monotônico**, não a bbox da água que recua). Relacionado: [[feedback_visual_bug_debug]], [[project_painter_composite_perf_2026_06_03]].
