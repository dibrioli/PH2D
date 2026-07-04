---
name: project_watercolor_v2_gpu_first_refactor
description: Watercolor v2 — refatoração GPU-first; largar o twin CPU bit-a-bit; pintura-lenta é submit/copy-bound
metadata: 
  node_type: memory
  type: project
  originSessionId: b9f81918-0b5f-4e51-8ea6-7fc0f5b5e554
---

Em 2026-06-10, após um dia inteiro de band-aids de perf sem resultado, o Enio mandou **abandonar
a estratégia incremental** e refatorar todo o motor de aquarela GPU-first. Handoff mestre:
`docs/HANDOFF_watercolor_v2_refactor.md` (supersede `HANDOFF_painter_fluid_perf_block.md`).

**Why:** a queda de FPS ao pintar NÃO é compute (kernels somam <5ms reais pelo GPU-timestamp
profiler) — é **topologia de frame**: ~12-15 `queue.submit()`/frame (cada um sync no Metal) +
**cópia do canvas inteiro todo frame** (`copy_texture_into_individual`, não dirty-rect) + readback
no hot path do traço. O `present-stall` de 50ms é backpressure dessa fila profunda, não vsync. E o
**twin CPU bit-a-bit** (`diffusion.rs` 2808 LOC + ~740 gates `gpu_parity`/`composite_parity`) é o
imposto que travou reestruturar a topologia ontem — força algoritmos com forma-de-CPU.

**How to apply:** GPU-first tempo-real-only (decisão do Enio: "se não roda em tempo real na CPU,
não implementa na CPU"). Invariantes-alvo: single-submit/frame para sim+composite; renderizar a
textura de preview DIRETO como sprite (zero cópia full-canvas); bake `canvas_rgba` só no pen-up
(zero readback no stroke); sim+composite escopados ao bbox de água ATIVA (sparse). A GPU é
a fonte da verdade da física; o CPU vira oráculo de invariantes físicos (não bit-paridade) + bake offline.
1ª ação = **ADR-0085 (Coord-only)** que supersede a paridade bit-a-bit (ADR-0049/0080-0082) + a
promessa de fallback CPU em-tempo-real (ADR-0049/0053: low-tier sem compute = aquarela off, não
sim CPU lento). Preservar: modelo K–M espectral (24 bandas, ADR-0080) + o look ratificado
(blooms/edge-darkening/granulação/franja/sheen/lift) — re-derivar GPU-first, validar paridade
VISUAL (não bit) onda a onda. Pendências de realismo: a borda fininha de deposição (edge-darkening)
e o equilíbrio da poça (a água deve estabilizar por física sob Keep Wet, não por settle-freeze).
Os fixes do perf-block (crash water/water, undo, sparse-idle) ficam. Ver
[[project_painter_fluid_4k_perf_architecture]], [[feedback_measure_perf_symptom_scale]],
[[reference_watercolor_state_of_art]].
