---
name: project-speed-study-2026-10-12-gate-critical-path
description: Estudo de velocidade 12–13/10 — gate de placa lento = cache de shaders NVIDIA CHEIO (1 GiB); gabarito Luz2D 3× ao bit; seletor de impacto preciso; selo do verde; main só verde
metadata:
  node_type: memory
  type: project
  originSessionId: d00af5bf-c31f-4689-9602-ebc38343104d
  modified: 2026-10-13T11:15:31.436Z
---

Estudo pedido pelo dono (12/10) após «Batched closing gate F13» 39 min e «Fix the 13 Luz2D red tests» 2 h 50.
Linha `line/Velocidade`; detalhe e tabelas em `docs/DevOps/VELOCIDADE_DOS_PORTOES.md`.

- ⭐⭐⭐ **Gate de placa lento ⇒ olhe primeiro o `~/.cache/nvidia/GLCache`.** Estava em 1 023 MB = tecto de fábrica
  1 GiB: o driver recompilava as mesmas pipelines a cada processo. Mesmo gate: 52,7 s cheio → 0,6 s com
  `__GL_SHADER_DISK_CACHE_SIZE` 16 GiB (agora no `[env]` do `.cargo/config.toml`). `o_oclusor_fino` ~640 s
  estava todo em `_nv002nvvm` (amostras de pilha por `gdb` com o teste como FILHO — `ptrace_scope=1`; o `samply`
  pede `perf_event_paranoid ≤ 1`, sem sudo).
- O caminho crítico do gate era o grupo `luz2d-placa` (2 por vez); a máquina ficava a ~5 de 32 núcleos.
- Gabarito Luz 2D (CG do miolo): canais em paralelo + vizinhança compacta + passagens fundidas, MESMO bit
  (impressão `9bdb5b2ed184a704`): 61 → 20 s num gate.
- `rdeps()` do nextest segue dev-arestas EM CADEIA (`ph2d-render` tem pintura/editor-core como dev-deps) →
  `scripts/impacto-preciso.py` + corte `ph2d-ecs → ph2d-i18n`: F13 151 → 127 pacotes, sai a Luz 2D na placa.
- Vermelho aceito no main pagou-se 3× ⇒ regra «main só verde» (DIRETRIZ §1.5.3).
- Laços de espera dos agentes (38 h/14 dias) eram espera pelos testes — proibi-los não poupa relógio.

**Why:** o dono quer implementação/integração mais rápidas sem perder qualidade.
**How to apply:** régua de velocidade de gate de placa nomeia o estado do cache (quente/frio); antes de dar
prazo a um teste lento, cronometre as fases ou amostre a pilha. Ver [[feedback_plan_every_measurement_upfront_and_batch_it]].
