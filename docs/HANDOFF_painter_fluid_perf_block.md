# HANDOFF — Painter Fluid PERF block (A): the fixed per-frame GPU stall (2026-06-10)

> **Bloco fundacional dedicado** (ADR-0083 tiling/sparse + `project_painter_fluid_4k_perf_architecture`).
> Executar em **contexto fresco e focado** — toca o caminho vivo do motor de aquarela (4000+ gates de
> paridade). Leia inteiro antes de mexer; a §2 (a pista) é a mais cara.

## §0 — O QUE JÁ LANDOU nesta sessão (commit local, sem push — fast-mode `--no-verify`)

Tudo compila (`-p ph2d-render --tests`, `-p ph2d-painter-fluid --features fluid --tests`,
`-p ph2d-host-desktop` default **e** `--features fluid`). Testes não-GPU verdes (picking, layer_compositor
ABI/naga, undo). Gates GPU `--ignored` (Metal) compilam — **rodar 1× pra confirmar**.

| Fix | O quê | Arquivos | Estado |
|---|---|---|---|
| **CRASH water/water** | `composite_frame` (sync) re-mapeava o `staging` ainda mapeado pelo readback PIPELINED → erro wgpu → processo aborta. Agora drena o `pending` antes de re-mapear (espelha `begin_stroke`). | `composite.rs` | ✅ corrigido na raiz; **Enio re-testar** |
| **stats poll(wait) crescente** | O dry-check `read_field_stats` fazia `poll(wait_indefinitely)` (drena a fila GPU inteira) → crescia 0.5→6ms num runaway. Novo `read_field_stats_pipelined` (async, staging persistente, lê o disparo anterior; dry-check tolera 1 ciclo). Síncrono mantido p/ os gates. | `solver.rs`, `painter_fluid_bridge.rs` | ✅ **PROVADO no log do Enio: 6ms → 0.008ms** |
| **Bug 2: sprite sempre paintável** | Gate `[0,1)` do quad derrubava samples na borda → `break_stroke_segment` → sim estancava. Novo `sprite_world_to_uv_unclamped`; clamp a jusante (envelope + cutoff de splat). | `picking.rs`, `lib.rs`, `painter_input.rs` | ✅ + unit test |
| **Bug 3: undo multi-step** | `canvas_rgba` é assado com atraso (catch-up pós pen-up); o snapshot do próximo traço pegava estado stale → 1 undo apagava vários. `flush_pending_bake` assa o pendente sincronamente no pointer-down, antes do `begin_stroke`. | `painter_fluid_bridge.rs`, `painter_input.rs` | ✅ **visual pendente** |
| **Bug 1: cadeia E5 region-scoped (multi-camada 4K)** | A cadeia E5 refazia 3 ops full-canvas/frame (`Region::full`). Novo `composite_region_into_canvas` (escreve em coords-canvas num `out` persistente) + inject/slot-copy sub-rect + seed-full no 1º frame. Premul fica full (barato). | `compositor.rs`, `mod.rs` (layer_compositor), `layer_composite.wgsl`, `individual.rs`, `renderer.rs`, `painter_gpu_preview.rs`, `painter_fluid_bridge.rs`, `painter_fluid_support.rs` | ✅ compila + gate Metal `composite_region_into_canvas_refreshes_region_and_preserves_outside`; **NÃO validado visual** (o teste do Enio foi 1 camada → E5 nem rodou). Vale p/ o objetivo multi-camada 4K. |
| **Instrumentação `[frame]`** | Profiler de frame (total / cpu-encode-raw / present-acquire-stall / painter-dispatch), gated no `PH2D_FLUID_PROFILE`. É o ponto de partida deste bloco. | `mod.rs` (render_loop) | ✅ ligado |

## §1 — O PROBLEMA (não resolvido, é este bloco)

FPS despenca pintando **aquarela com água**, e **fica baixo mesmo parado depois de pintar**. Medido
(Enio, Metal, Mac **8 GB**, present **Immediate**, canvas **1408×768**):

```
[fluid] step=1.0ms composite=0.16ms stats=0.008ms total≈1.2ms     ← submit de CPU do drive (enganoso)
[frame] total=134ms (~7fps) | cpu-encode(raw)=2.81ms | present/acquire-stall=131.67ms | painter-dispatch=0.01ms
```

**Lido corretamente:** a CPU encoda em **2.8ms**; os **131ms estão na execução da GPU** (o `acquire_frame`
espera a GPU drenar — em Immediate, logo NÃO é vsync; é backpressure de trabalho real de GPU). O profiler
`[fluid]` mede o **submit de CPU** (~1ms), **não** a execução na GPU — por isso o fluido "parecia" inocente.

## §2 — A PISTA QUE MUDA TUDO (Enio, 2026-06-10) — leia com atenção

> **"Uma única pequena mancha de água em Immediate já era suficiente para a queda de FPS."**

Isso **refuta** a hipótese de envelope-crescente / pressão de RAM (que eu persegui). Uma mancha PEQUENA já
dá 131ms → o custo é **FIXO por frame, INDEPENDENTE do tamanho do molhado**, disparado por ter **qualquer
campo vivo**. Não é O(área), não é 4K-scaling, não é memória. É **uma operação de GPU pesada de custo
fixo que roda todo frame quando um campo de fluido está vivo** — bug muito mais específico e tratável que
"tiling pra 4K". **Keep Wet** só piora porque mantém o campo vivo pra sempre (o custo fixo nunca para).

## §3 — DESCARTADOS (com evidência) vs SUSPEITOS (a investigar)

**Descartado:**
- CPU encode (raw=2.8ms), painter-dispatch CPU (0.01ms), o preview CPU.
- present-mode/vsync (Immediate também trava — é backpressure de GPU).
- envelope-crescente / RAM (mancha pequena já trava — §2).
- o `stats` (pipelinado, 0.008ms).
- caminho CPU `on_tick`/`tick_wet_field` (gated por `fluid_hires`; o `[fluid]` imprime ⇒ drive GPU ativo).

**Suspeitos (precisam de GPU-timestamp pra cravar) — algo full-canvas/fixo POR FRAME quando há campo vivo:**
1. **`copy_preview_into_slot`** (`painter_fluid_support.rs`): `copy_texture_into_individual(id, tex, cw, ch)`
   — copia o canvas **inteiro** (cw×ch) GPU→GPU **todo frame**, ignorando a região molhada. Candidato #1.
2. **Shallow-water S3d sempre-ligado** (`WATERCOLOR_VELOCITY=1.3` etc.): ~11 passes/substep
   (add_forces+divergence+2 clear+6 Jacobi+project). Region-scoped, mas verificar se algum pass/`project`
   tem custo fixo full-canvas ou convergência cara. Foi adicionado recente e "perf live nunca medida".
3. **Sheen** (`publish_wet_sheen_between_strokes`): composite EXTRA por frame ocioso enquanto molhado+ShowWet.
4. **`cs_reduce`** (stats) full-canvas com atomics — sporadic (20 frames), provavelmente não, mas medir.
5. **Múltiplos `queue.submit()` por frame** (step, composite, copy, inject, stats) → splits de encoder /
   serialização no Metal. Medir se a soma de submits estoura.
6. **map_async/poll(Poll)** dos readbacks pipelinados (composite + stats) interagindo com o `acquire` do
   present sob Immediate.

## §4 — PLANO

**Passo 1 (medir — NÃO pular):** GPU-timestamp profiling. Adicionar um `wgpu::QuerySet` (timestamp) com
`timestamp_writes` em cada compute/render pass do fluido (step/composite/copy/stats) **e** dos 4 passes
principais (sprite/tonemap/vello/compositor); `resolve_query_set` → readback → deltas em ms. Pintar UMA
mancha pequena e ver **qual pass come os 131ms**. (O `[frame]` de CPU já provou que NÃO é CPU; agora é
achar o pass de GPU.) Suspeito #1 = `copy_preview_into_slot` full-canvas.

**Passo 2 (corrigir a causa):** depende do passo 1. Se for o copy full-canvas → copiar só o sub-rect
(como já fiz no slot-copy do E5: `copy_texture_region_into_individual`). Se for o shallow-water → revisar
custo/convergência ou escopo. Etc.

**Passo 3 (o objetivo maior, depois do custo-fixo):** sim+composite escopados à **frente molhada ativa**
(não ao envelope all-time) — tiling/sparse GPU-residente (ADR-0078 S1b / ADR-0083). Respeitar a cerca de
Chesterton do envelope monotônico (foi o fix das "quinas retangulares"): o envelope monotônico só
dimensiona/semeia a textura persistente; o trabalho POR FRAME escopa ao ativo. Validação visual estágio-
a-estágio com o Enio (blooms, edge-darkening, granulação, sheen intactos).

## §5 — COMO RODAR / MEDIR
```bash
PH2D_FLUID_PROFILE=1 ./play.command   # imprime [fluid] (drive) + [frame] (total/raw/stall/dispatch)
# Repro: pincel água, Keep Wet ON, UMA mancha pequena → já cai pra ~7fps (custo fixo, §2).
cargo test -p ph2d-painter-fluid --features fluid --test gpu_parity --test composite_parity -- --ignored
cargo test -p ph2d-render --test layer_compositor_gpu -- --ignored   # inclui o novo gate region-into-canvas
```

— deixado por Claude (sessão 2026-06-10): bugs de input/undo/crash/stats fechados; FPS isolado como
GPU-execution-bound de **custo fixo por frame** (§2). Próximo: GPU-timestamp → achar o pass → corrigir.
