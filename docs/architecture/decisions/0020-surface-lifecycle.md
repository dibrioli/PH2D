# ADR-0020: Surface lifecycle e device-lost recovery

**Status:** Accepted
**Data:** 2026-05-08
**Decisor:** Enio Oliveira Dias Brito
**Implementador:** Claude Opus 4.7 (1M context)
**Origem:** auditoria pré-plano operacional pós-spike (sugestão LLM1).

## Contexto

§12.5 do SKILL menciona "se falhar 2× consecutivas, panic graceful" sem definir o protocolo de recovery. Em mobile (especialmente iOS), o ciclo background→foreground destrói GPU contexts agressivamente. wgpu reporta `SurfaceError::Lost` e `SurfaceError::Outdated` que precisam de tratamento distinto:

- `Lost` — surface destruída, requer `surface.configure()` + recriação de transients (depth buffers, intermediate textures, swapchain).
- `Outdated` — surface ainda existe mas configuração ficou stale (resize que perdeu corrida com presente). Reconfigure imediato sem recriar.
- `Suboptimal` — render ainda funciona mas estatísticas degradadas; reconfigure no próximo frame ocioso.
- `Timeout` — driver demorou demais; retry ou skip frame.
- `OutOfMemory` — fatal, panic graceful (per HR-13 budget).

Sem protocolo formal, cada caller de `get_current_texture()` reinventa o tratamento e corre risco de deadlock no recovery (especialmente quando assets transients precisam ser recooked do AssetDb).

## Decisão

`ph2d-gpu::SurfaceContext` expõe método único `acquire_frame() -> Result<FrameTarget, AcquireError>` que encapsula toda a lógica de retry/reconfigure/recovery. Callers (subsistemas de render) só recebem `FrameTarget` válido ou erro terminal.

**Protocolo de recovery (por variant):**

| `SurfaceError` | Ação imediata | Side-effect | Próximo frame |
|---|---|---|---|
| `Outdated` | `surface.configure()` com config atual; retry `get_current_texture()` 1× | Nenhum | Normal |
| `Suboptimal` | Renderiza no frame com warning logado | Marca flag `needs_reconfigure` | `surface.configure()` antes de acquire |
| `Lost` | Drop transients via `TransientPool::clear()`; `surface.configure()`; reload de textures depth/MSAA via `AssetDb::request_recook(handle)` | Emite `SurfaceLostEvent` (subsistemas reagem se necessário) | Aguarda transients prontos antes de acquire |
| `Timeout` | Skip frame; incrementa counter `consecutive_timeouts` | Se `consecutive_timeouts ≥ 3`, escala para `Lost` recovery | — |
| `OutOfMemory` | Panic graceful via `host.report_fatal("GPU OOM")`; HR-13 violado | Captura snapshot do MemoryBudget aggregator | — |

**Background→foreground (mobile-specific):**
- Shell (Swift/Kotlin) emite `host.event_lifecycle(Background)` → `ph2d-gpu` faz `TransientPool::clear()` proativo, libera VRAM (não-essencial cache de assets).
- Shell emite `host.event_lifecycle(Foreground)` → `ph2d-gpu` aguarda `surface.configure()` antes do primeiro acquire.

**Determinismo:** se modo lockstep/rollback ativo, `Lost` pausa a simulação até recovery completar (não pula ticks; cliente reconecta com replay). Sem isso, divergência cross-client.

## Consequências

**Aceitas:**
- `SurfaceContext::acquire_frame()` é o único caminho público para obter texture. Acesso direto à `wgpu::Surface` proibido fora de `ph2d-gpu`.
- `TransientPool` em `ph2d-gpu` mantém handle→texture mapping; `clear()` libera tudo, `request(spec)` recoze sob demanda.
- `SurfaceLostEvent` é evento ECS (per ph2d-ecs convenção); subsistemas registram observer se precisam reagir (ex: ph2d-render limpa render graph cache).

**Negadas:**
- Não vamos fazer recovery silencioso de `OutOfMemory` (HR-13 manda OOM ser falha visível para diagnóstico).
- Não vamos pular frames silenciosamente em modo determinístico (HR-5).

## Alternativas consideradas

- **Tratamento per-caller:** descartado — cada subsistema reinventaria + bug surface enorme.
- **Auto-retry infinito:** descartado — encobre divergência de drivers ou GPU OOM real.
- **Reconfigure síncrono em todo `acquire`:** descartado — custo desnecessário no caso comum (90% dos frames são `Ok`).

## Próximos passos

1. Implementar `SurfaceContext::acquire_frame()` em `ph2d-gpu` (M3 do plano operacional).
2. Adicionar fixture `tests/budget/surface_recovery.rs` que simula cada variant via mock surface; valida protocolo.
3. Documentar em §12.5 do SKILL substituindo o bullet "se falhar 2× consecutivas, panic graceful" por link para este ADR.
