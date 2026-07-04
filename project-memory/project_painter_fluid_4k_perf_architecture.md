---
name: project_painter_fluid_4k_perf_architecture
description: "Painter fluid sim's 4K perf ceiling is structural (per-frame O(grid) CPU + uploads), not micro-opt — the GPU-resident architecture that fixes it"
metadata: 
  node_type: memory
  type: project
  originSessionId: 17ce2a9d-e337-4c27-8c97-12e1c154443e
---

W15.3 GPU watercolor (ADR-0049) está **correto e bonito** (smoke OK Enio), mas NÃO escala pra 4K.
Os fixes de micro-opt (SS=1 full-res, buffers persistentes, pre-warm, GPU-clear) deram só **~10%**.

**Por quê (medido @1408×768, Metal M-series, brush 32px, --release):** o hot loop por-frame é
`O(grid)` em CPU + transfer CPU↔GPU, não micro-custo:
- `fluid_frame_step_inputs` ~2ms = alloc Vec água+depósito + scan wet-bbox + evaporate + clear (CPU O(grid))
- `step_resident` ~2ms = **upload água+depósito full-grid** + diffuse/advect GPU
- `composite_frame` ~1.5ms = composite GPU (já SS=1) + **`device.poll(wait)` readback síncrono**

Em 4K (×~16 área) cada um vira ~32/32/24ms → estoura 16ms (60Hz) **com UMA camada**. Multi-camada multiplica.

**Root cause:** água e pigmento NÃO são GPU-residentes — todo frame a CPU aloca/varre o grid inteiro e
faz upload full-grid; o composite serializa GPU↔CPU com 1 readback síncrono.

**Why:** Enio quer pintar até 4K, pinturas animadas grandes multi-camada, tudo real-time ("melhor que o
Procreate"). O custo estrutural mata isso; nenhuma micro-otimização resolve.

**How to apply:** arquitetura-alvo (mata os 3 custos O(grid) de uma vez), plano E1–E5 detalhado em
[`docs/HANDOFF_painter_fluid_gpu_composite.md`](docs/HANDOFF_painter_fluid_gpu_composite.md) §4:
(1) água GPU-residente (já há `pig_a`+`step_resident`); (2) `cs_splat` por lista-de-dabs (⚠️ paridade de
forma bit-a-bit vs splat CPU senão o traço muda); (3) evaporate GPU + dry-check por massa-escalar O(1)
(não scan); (4) wet-bbox por redução GPU; (5) composite→textura de preview, readback 1× só no pen-up.
Depois o custo vira `O(dabs)`+passes GPU; o `O(grid)` da CPU some do hot loop. É reescrita fundacional do
hot-path (solver/tool/render-loop) → executar em contexto fresco, validação visual estágio-a-estágio,
commit local, push só após Enio validar. Relacionado: [[project_painter_composite_perf_2026_06_03]]
(GPU compositor do slider-drag, problema irmão mas distinto: preview de ajuste, não o sim live).
