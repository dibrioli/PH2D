# 08 — Performance e memória

> HR-3 (zero-alloc hot path), HR-4 (frame budget), HR-13 (memory budget declarado por plataforma) governam tudo aqui. Decisões de design devem caber nos budgets ou expor explicitamente o estouro (`#[allow(budget_overrun)]` proibido em release).

## 8.1 Frame budget

Painter compartilha o budget global do subsistema **Render** (3.5ms @ 60Hz no [SKILL_Stack §HR-4](../../SKILL_Stack_PH2D_Definitiva.md)) mas adiciona budgets internos próprios.

### 8.1.1 Sub-budget Painter (within Render)

| Etapa | 60 Hz target | 120 Hz target | Como |
|-------|--------------|---------------|------|
| Input dispatch + stroke path advance | 0.2ms | 0.1ms | CPU, zero-alloc |
| Stamp scheduling + cor dynamics | 0.2ms | 0.1ms | CPU, bumpalo arena per-stroke |
| Stamp pipeline encode (CPU side prep) | 0.3ms | 0.2ms | upload `Stamp[]` buffer |
| GPU compute (stamps) | 1.5ms | 1.0ms | workgroup dispatch |
| Layer compositor (dirty rect) | 0.8ms | 0.5ms | depende do número de layers + dirty rect size |
| UI overlay (sidebar HUD, marching ants, brush cursor) | 0.5ms | 0.4ms | Vello + parley |
| **Total Painter / frame** | **3.5ms** | **2.3ms** | dentro do budget global Render |

### 8.1.2 Worst-case dimensioning

- **Stamps/frame max**: 4096 (pool size). Em 60Hz, 4096 stamps com brush size = 32px → ~1M pixels processados; bem dentro do throughput de Apple M2 / RDNA1.
- **Brush size max**: 2048px (`max_size_px` cap). Stamp grande dispatches mais workgroups; pior caso é 16 stamps/frame com brush 2048px (256² workgroups cada × 16 = 4M workgroups). Apple M2 ~5ms; RDNA1 ~6ms — borderline. **Solução:** se brush size > 512, stamp scheduler reduz spacing temporariamente (10% spacing min em vez de 1%) — preserva visual sem estourar budget.
- **Layers**: compositor escala linearmente. 50 layers @ 4K = 5ms (acima do budget). **Solução:** layer cache + dirty rect (compose só o necessário).

### 8.1.3 Frame budget no Brush Studio

Brush Studio fora do hot path principal. Live preview repaint em debounce 16ms — não preempta paint:
- Preview cabe em 0.5ms (stroke fixed S-curve com 32 stamps).
- Triggered apenas em mudança de slider; idle = zero custo.

## 8.2 Memory budget (HR-13)

Declarado em `PainterPlugin::init() -> MemoryBudget`. Valores por plataforma:

| Componente | iPad/iOS | Android | Desktop | Web |
|------------|----------|---------|---------|-----|
| **VRAM** | | | | |
| Layer textures (working set) | 200 MB | 200 MB | 800 MB | 150 MB |
| Shape atlas | 4 MB | 4 MB | 4 MB | 4 MB |
| Grain atlas | 32 MB | 32 MB | 64 MB | 16 MB |
| Compositor cache | 50 MB | 50 MB | 200 MB | 30 MB |
| UI overlay (Vello buffers) | 32 MB | 32 MB | 64 MB | 24 MB |
| Stamp buffer + uniforms | 1 MB | 1 MB | 1 MB | 1 MB |
| **VRAM total Painter** | **319 MB** | **319 MB** | **1133 MB** | **225 MB** |
| **RAM** | | | | |
| Stroke history (250 strokes × ~100 KB) | 25 MB | 25 MB | 25 MB | 25 MB |
| Undo dirty-rect snapshots | 25 MB | 25 MB | 50 MB | 20 MB |
| Brush library (loaded brushes) | 8 MB | 8 MB | 16 MB | 8 MB |
| Color palettes + history | 1 MB | 1 MB | 1 MB | 1 MB |
| Painter UI state (PainterPanel + WidgetStore) | 4 MB | 4 MB | 8 MB | 4 MB |
| **RAM total Painter** | **63 MB** | **63 MB** | **100 MB** | **58 MB** |
| **Heap script (Luau, se brush actions usadas)** | 8 MB | 8 MB | 16 MB | 8 MB |

Total fits within platform memory budgets do SKILL_Stack §12.1:

| Platform | Total Painter (VRAM+RAM+Lua) | Platform total app | Folga |
|----------|------------------------------|--------------------|-------|
| iPad/iOS | 390 MB | 1000 MB | 610 MB para Engine core + ECS + Editor UI etc |
| Android | 390 MB | 1000 MB | 610 MB |
| Desktop | 1249 MB | 3500 MB | 2251 MB |
| Web | 291 MB | 700 MB | 409 MB |

## 8.3 GPU stamp pipeline performance

### 8.3.1 Pipeline detalhe

```
Frame N:
  T=0.0ms  CPU collects pointer events (since last frame)
  T=0.2ms  StrokePath.advance() — produce sample points
  T=0.4ms  StampScheduler.emit() — produce stamps[]
  T=0.5ms  Upload stamps[] storage buffer (BAR memory; Metal/Vk shared)
  T=0.6ms  Dispatch compute(stamp_count)
  T=2.1ms  Compute done; layer texture dirty
  T=2.1ms  Compositor: collect dirty rects from all dirty layers
  T=2.4ms  Composite top-down with dirty rect crop
  T=2.9ms  Encode UI overlay (Vello)
  T=3.4ms  Encode present command
  T=3.5ms  Submit + wait swap
```

### 8.3.2 Stamps per workgroup

Workgroup 8×8 = 64 threads. Cada workgroup processa **1 stamp em 1 8×8 tile**. Stamps maiores fan-out em múltiplos workgroups:

```
stamp_tiles = ceil(stamp_size / 8)²
total_workgroups = sum(stamp_tiles for each stamp)
```

Para stamps 32px → 16 workgroups/stamp. 100 stamps de 32px = 1600 workgroups. Apple M2 dispatch ~100µs.

### 8.3.3 Optimization passes (W5+)

Considerar se medições justificarem:
- **Stamp culling**: stamps fora do canvas viewport descartados.
- **Stamp coalescing**: stamps muito próximos coalescidos num único quando overlap > 95%.
- **Specialized shaders**: 6 shaders por Rendering mode em vez de um com switch (vide [01 §1.8.3](01_brush_engine.md)).
- **Multi-frame stamp commits**: para brushes muito grandes, dividir stamps em batches por frame.

## 8.4 Compositor performance

### 8.4.1 Dirty rect

Tracking de dirty rects por layer:

```rust
pub struct DirtyRect {
    pub layer_id: LayerId,
    pub bounding_box: Rect,    // em world coords
    pub blend_mode_dirty: bool, // mudou blend mode -> precisa recompose
    pub mask_dirty: bool,
}
```

Compositor combina dirty rects e calcula **mínimo rectangle** que precisa de recompose. Tipicamente:
- 1 stroke = dirty rect ~10–500 px² (depende do tamanho do brush).
- Pan/zoom = 0 dirty rects (presentation transform).
- Layer reorder = full canvas dirty.
- Visibility toggle = full canvas dirty.

### 8.4.2 Layer cache

`LayerCache::HashMap<LayerId, CachedLayer>` mantém texture composta. Invalidação por dirty rect — apenas re-compose o crop.

Quando layer cache está full (VRAM budget atingido), evicta LRU. Layers evictadas re-renderizam on-demand do source raster (mais rápido que loaded; cache é só pra evitar O(N_layers) recomposição).

### 8.4.3 Layer compositing throughput

Bench em macOS M2 / 4K canvas:
- 10 layers cache-hot: 0.8ms.
- 50 layers cache-hot: 4ms.
- 50 layers cache-cold (full re-render): 12ms.

Gate `layers_composite_50_4k_under_5ms` (HR-4 budget) garante o caso hot. Cold path é raramente atingido (canvas open + first frame).

## 8.5 Hot path zero-alloc (HR-3)

**Caminhos críticos** marcados `#[no_alloc]`:
- `StrokePath::advance(input) -> SamplePoint` — usa bumpalo pre-allocated.
- `StampScheduler::emit(point) -> &[Stamp]` — pool de 4096 stamps pré-alocados.
- `StampPipeline::encode(stamps)` — reusa `BindGroup` cache; storage buffer pré-criado com capacity = 4096 × 96 = 384 KB.
- `LayerCompositor::composite_dirty(rects)` — usa staging textures pre-allocated.
- `BrushCursor::paint(p)` — bumpalo arena reset per-frame.

Bench: `tests/budget/painter_no_alloc.rs` corre 100 frames sintéticos com 1k stamps/frame e dhat-rs conta allocations. Falha se > 0 em hot path.

## 8.6 Async morre na shell

Painter NÃO usa async. Tudo síncrono dentro de game thread (Vide SKILL_Stack §12.2):

Exceções (tolerated):
- **`ph2d-asset::loader`** — load de brushes do disco off-thread, callback no game thread quando pronto.
- **Time-lapse encoder** — frame capture pushed para encoder thread; encoder roda em thread separada, finaliza arquivo MP4 ao fim.
- **Save** — `Save` action serializa stroke history + canvas state em thread separada (heavy IO).

`tokio` proibido no workspace; `pollster` aceitável para casos pontuais (raros no Painter).

## 8.7 Thermal e battery (mobile)

iPad/Android sob carga sustentada throttle CPU/GPU. Painter detecta thermal state:
- `PlatformHost::thermal_state() -> ThermalState { Nominal | Fair | Serious | Critical }` (iOS expõe via `ProcessInfo.thermalState`; Android via `PowerManager.getCurrentThermalStatus()`).
- Em `Serious`/`Critical`: stamp scheduler reduz spacing min para 5%, reduz brush size cap temporariamente, frame target cai para 60Hz mesmo em ProMotion devices.
- Notificação discreta na UI (badge sutíl no top-bar): *"Thermal throttle ativo"*.

## 8.8 GPU device lost / surface lifecycle

Compartilha protocolo do [ADR-0020 surface-lifecycle.md](../architecture/decisions/0020-surface-lifecycle.md). Cenários:

- **Surface Lost** (driver crash, sleep): Painter `pause_render()`. Mantém stroke history em RAM. Re-cria swapchain on wake.
- **Surface Outdated** (resize): re-create swapchain, manter VRAM resources.
- **Surface Suboptimal**: ignore (Apple sometimes returns suboptimal during mode transitions).
- **Surface OOM**: PainterPlugin libera caches (layer cache LRU, compositor cache) e tenta novamente. Se ainda OOM: notifica usuária e congela paint (com snapshot do canvas atual seguro em RAM).

## 8.9 Save / load performance

### 8.9.1 Save

Async, off-thread. Sequence:
1. Game thread: faz snapshot read-only da `PainterState` (layers, stroke history, brush refs, etc.).
2. IO thread: serializa via postcard.
3. IO thread: compress (zstd, level 3 default — balance speed/size).
4. IO thread: write atomic (write to `.ph2d-painter.tmp` then rename).
5. Game thread: recebe notify "save complete".

Tipicamente: 4K canvas, 20 layers, 100 strokes = ~30-80 MB postcard → ~15-40 MB after zstd → save 200-500ms off-thread.

Auto-save: a cada 5 min (configurável) save background; usuária nunca espera.

### 8.9.2 Load

Reverse:
1. IO thread: read file, decompress.
2. IO thread: deserialize postcard.
3. IO thread: upload layer textures para VRAM (off-thread upload via wgpu staging buffer).
4. Game thread: switch active canvas.

Loading screen com progress bar (chunks de 10%).

## 8.10 Time-lapse capture model

### 8.10.1 Frame capture

Capture **off-thread** (não bloqueia paint). A cada N ms (N depende de Quality preset — Studio = 100ms, Good = 200ms, Low = 500ms):
1. Compositor produces full-canvas texture.
2. Render thread captures texture to a `staging buffer` via `wgpu::CommandEncoder::copy_texture_to_buffer`.
3. Staging buffer mapped; bytes pushed para encoder thread queue.
4. Encoder thread (separate): encode frame para MP4/HEVC via ffmpeg-like library.

### 8.10.2 Encoder library (decisão pendente)

Candidatos:
- **`ffmpeg-next`** (wrapper ffmpeg) — completo mas pesado deps.
- **`gstreamer-rs`** — alternativa; ainda pesado.
- **Native platform APIs**:
  - macOS/iOS: `AVAssetWriter` (Swift, expõe via FFI).
  - Android: `MediaCodec`.
  - Windows: `Media Foundation`.
  - Linux: ffmpeg fallback.

**Decisão preliminar (W11+):** native APIs por plataforma. Encoder roda na shell, core Painter envia bytes via channel. Razão: evita 50MB+ de ffmpeg shipping com cada Painter.

Quando native indisponível (Web — sem AVAssetWriter equivalente): fallback `webcodecs` API quando suportado, caso contrário time-lapse desabilitado.

### 8.10.3 Custo do capture

Per frame capture: 5–15ms (copy_texture_to_buffer + push to encoder queue). **Off-thread**, não afeta paint hot path.

Disco: ~1080p 12 Mbps = ~1.5 MB/s. Hora de pintura = ~5 GB. Capture interval relaxado (200ms = 5 fps capture) reduz para ~30 MB/min mesmo em Studio quality.

## 8.11 Reprodutibilidade de build (HR + reproducible builds)

Painter contribui para reproducible builds workspace-wide:
- Brush textures bundled = stable bytes em `.ph2d-brush` resources/.
- Default brushes shipados com hash blake3 fixado em build script.
- WGSL shaders embedded via `include_str!`; sem dynamic compilation.

## 8.12 Profiling

Profiling hooks via `tracing` spans:
- `painter.input` span per pointer event batch.
- `painter.stamp_pipeline` span per encode + dispatch.
- `painter.compositor` span per composite call (dirty rects logged).
- `painter.ui` span per UI repaint.

Em feature `tracy`: spans aparecem em Tracy timeline para profiling external.
Em feature `puffin` (editor default): in-app puffin overlay (Painter Preferences → Dev → Show profile overlay).

## 8.13 Gates de teste (sumarizado)

| Gate | Crate | Valida |
|------|-------|--------|
| `painter_frame_budget_60hz` | `ph2d-painter-brush` | Sintetic stroke 1k stamps/frame cabe em 3.5ms em CI baseline |
| `painter_no_alloc_hot_path` | idem | 0 allocs em 100 frames sintéticos com 1k stamps/frame (dhat-rs) |
| `painter_layer_composite_50_4k` | idem | 50 layers @ 4K composita em ≤ 5ms cache-hot |
| `painter_dirty_rect_min` | idem | Pintar 1 pixel produz dirty rect ≤ stamp_size |
| `painter_memory_budget_within_platform` | idem | Calculated MemoryBudget < platform max (HR-13) |
| `painter_stamp_buffer_capacity` | idem | 4097th stamp causa flush (não realloc) |
| `painter_thermal_throttle_kick_in` | idem | Mock thermal state Serious → spacing min sobe para 5% |
| `painter_save_load_roundtrip` | `ph2d-tool-painter` | Save → load → state idêntico (incluindo stroke history) |
| `painter_save_load_perf_300ms_p99` | smoke | Save de canvas 4K + 20 layers + 100 strokes < 300ms p99 |
| `painter_timelapse_encoder_off_thread` | idem | Capture não causa stutter no paint (CPU stamping vs encoder thread separados) |
| `painter_gpu_device_lost_recovery` | idem | Mock surface lost → recreated → strokes pendentes preservados |

**Continua em:** [09_export_interop.md](09_export_interop.md) — formatos de export, PSD interop.
