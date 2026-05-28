# 03 — Renderer (Vello + GPU stroke expansion + Linesweeper + SDF Hybrid)

> Spec do **pipeline de renderização** do Vector Module. Vello 0.8 como renderer único (GPU compute, prefix-sum, sparse strips). GPU stroke expansion (Levien+Uguray 2024). Pipeline boolean **draft+reconcile** em 3 modos (resolve crítica C Antigravity). Editor + runtime sharing renderer. Frame budget 3.5 ms (HR-4).
>
> **ADR ratificador:** ADR-0059 (Vector renderer pipeline) + ADR-0065 (SDF Hybrid Pipeline).
> **Spec gêmeos:** [`01_data_model.md`](01_data_model.md) (data fonte) + [`05_procedural_fill.md`](05_procedural_fill.md) (shader graph fill) + [`10_runtime_gameplay.md`](10_runtime_gameplay.md) (renderer shared with game runtime).

## 3.1 Vello pipeline overview

### 3.1.1 Vello 0.8 — escolhas chave

- **Vello GPU**: **prefix-sum based pipeline** (coarse → fine → ratification → fine rasterize) em compute shaders WGSL. Backbone para todos targets com WebGPU compute (Mac/Win/Linux/iPad/Android/Web).
- **Vello CPU** (fallback): **sparse strips arch** (Laurenz Stampfl ETH 2025 thesis "High-performance 2D graphics rendering on the **CPU** using sparse strips" — run-length-compressed antialiased boundaries + sparsely represented solid interiors via Rust SIMD: SSE2/AVX/AVX2/AVX512/NEON). **Correção crítica L2F1 Antigravity 2ª iteração 2026-05-28** — sparse strips é literalmente arquitetura CPU-only do Vello, NÃO o pipeline GPU.
- **WebGPU compute requerido** para Vello GPU. Para devices sem compute (rare em 2026 — older WebGPU implementations): cai para Vello CPU multi-threaded SIMD com sparse strips. Cross-platform invariante via mesma WGSL → naga backend.
- **Vello Hybrid**: combina GPU + CPU em casos específicos (e.g., GPU stalls em mobile thermal throttling); fallback automático per-frame.

### 3.1.2 Por que Vello vs alternatives

| Renderer | Type | Cross-plat | Infinite zoom | Compute pipeline | Status |
|----------|------|------------|----------------|------------------|--------|
| Skia | CPU + GPU hybrid | ✓ | partial | mixed | mature (Chrome/Flutter) |
| Cairo | CPU only | ✓ | ✓ | ❌ | mature (GTK) |
| Direct2D | GPU | Windows | partial | ❌ | mature |
| **Vello** | **GPU compute** | **✓** | **✓ infinite** | **✓ prefix-sum** | **alpha (0.8, churns per quarter)** |

Vello é o **único renderer GPU compute open-source** com infinite zoom + cross-platform via WebGPU. Risco churn aceito (ADR-0004 Painter precedente).

### 3.1.3 Pipeline stages (per frame)

```
[VectorNetwork] (W1 doc data model)
     ↓
[Network → BezPath conversion] (kurbo cubics)
     ↓
[Apply Stroke Style] (constant width via vello stroke;
                      variable width via §3.2 GPU expansion)
     ↓
[Apply Fill Style] (solid / gradient / procedural — §05)
     ↓
[Vello Scene::push()]
     ↓
[Vello GPU rasterize (compute)] (coarse pass → fine pass → output)
     ↓
[Composite with editor chrome / overlay]
     ↓
[Present to surface]
```

### 3.1.4 Sub-budget breakdown (3.5 ms)

| Stage | Budget |
|-------|--------|
| Network → BezPath conversion | 0.3 ms |
| Stroke style application | 0.2 ms |
| Fill style (procedural shader topology compile) | cached → 0 ms |
| Vello scene construction | 0.5 ms |
| Vello GPU compute pass | 1.5 ms |
| Composite + present | 0.5 ms |
| **Reserved buffer** | 0.5 ms |
| **Total** | **3.5 ms** |

---

## 3.2 GPU stroke expansion (Levien+Uguray 2024)

### 3.2.1 Paper reference

[GPU-friendly Stroke Expansion (ACM 2024, arXiv 2405.00127)](https://arxiv.org/pdf/2405.00127) — Levien & Uguray.

### 3.2.2 Algoritmo

- Approximate parallel-curve offset de cubic Bézier com **Euler spirals** (clothoids), depois flatten.
- Fully parallel: roda em compute shader pass único.
- Robust em zoom extremo, handles variable-width strokes, miter/bevel/round joins, dashes.

### 3.2.3 Integração

Vello já integra (Linebender consorcia). Vector Module consome via `vello::Scene::stroke()` API.

### 3.2.4 Variable width via WidthProfile

```rust
// WidthProfile (vide 01_data_model.md §1.11.1)
impl WidthProfile {
    fn sample_along(&self, t: f32, pressure: f32) -> f32 {
        let base = self.base_width;
        let pressure_contrib = pressure * self.pressure_weight;
        let taper = lerp(
            1.0 - self.taper_start,
            1.0 - self.taper_end,
            t,
        );
        base * (pressure_contrib + 1.0 - self.pressure_weight) * taper
    }
}
```

GPU shader recebe N samples de width along stroke; expansion paper handles smooth interpolation.

### 3.2.5 Performance

- 1000-segment stroke com variable width: ≤ 0.5 ms na ProMotion (M-series).
- Sub-9 ms total stylus latency target alcançado.

---

## 3.3 Linesweeper integration (boolean ops)

### 3.3.1 Linesweeper — papel

`linesweeper` crate (Joe Neeman, Linebender ecosystem) — boolean ops 2D robust em degenerate cases.

### 3.3.2 Por que Linesweeper vs Clipper

| Lib | Approach | Robustness em degenerate | Performance | Determinismo |
|-----|----------|--------------------------|-------------|--------------|
| Clipper2 | Bentley-Ottmann classical | Quebra em near-tangent / coincident | Boa | Não-deterministic em float catastrophe |
| Boost.Polygon | Voronoi-based | Robust mas slow | Lenta | Razoável |
| **Linesweeper** | **Ordering-first sweep** | **Robust em real-world cases** | **Boa** | **Determinístico opt-in** |

Joe Neeman's insight: priorizar orderings sobre intersections (vide [joe.neeman.me/posts/linesweeper](https://joe.neeman.me/posts/linesweeper)).

### 3.3.3 Pipeline boolean draft+reconcile (resolve crítica C Antigravity)

3 modos selectivos por contexto (vide [`02_geometry_graph.md §2.2.2`](02_geometry_graph.md)):

#### Modo 1: Draft preview (hot-path)

- **Budget**: ≤ 1 ms.
- **Algorithm**: CPU naive cubic Bézier clipping aproximado (não-exato; pode haver imprecisão em near-tangent).
- **Usado em**: slider drag, stylus motion contínua.

#### Modo 2: SDF Hybrid GPU

- **Budget**: ≤ 0.5 ms compute pass.
- **Algorithm**: vide §3.4 + ADR-0065.
- **Usado em**: real-time interactivity, gameplay morphing (W16).

#### Modo 3: Linesweeper exato (async)

- **Budget**: 1-50 ms (off-thread).
- **Algorithm**: Linesweeper full robust.
- **Usado em**: commit (mouse-up / pencil-lift / após N ms inatividade).
- **Cache**: result by hash, LRU 50 MB.

UI mostra indicador discreto "boolean em commit…" quando worker está computando.

### 3.3.4 Determinismo — app-layer responsibility (revisado Antigravity L2F3 2ª iteração 2026-05-28)

**Correção crítica**: o crate `linesweeper` (Joe Neeman) **não expõe** `deterministic_mode` flag nativamente em sua API. Determinismo bit-identical cross-platform é responsabilidade da **camada de aplicação PH2D**:

1. **Pré-ordenação canônica** de segments antes de invocar linesweeper: sort por `(start.x, start.y, end.x, end.y)` Q16.16 fixed-point ascending.
2. **Coordenadas em fixed-point Q16.16** quando `VectorNetwork::deterministic=true` (vide [01 §1.9.2](01_data_model.md)).
3. **Ordered reductions** no post-processing: result regions ordenadas por `(area_signed, centroid.x, centroid.y)` fixed-point.
4. **FMA off** em qualquer post-process matemático (`#[cfg(target_feature="fma")]` guarded).
5. **Linesweeper roda determinístico-localmente** (mesma input → mesmo output em mesma CPU); pré-ordenação + Q16.16 garante "mesma input" cross-platform.

Test: `tests/determinism/boolean_cross_os.rs` valida bit-identical em Linux + Mac + Windows com fixture de 100 boolean ops.

Pattern alinhado com Painter ADR-0050+0051 (color/device determinism layer).

**Linux multi-arch SIMD determinism (Antigravity 3ª iteração L2F2 2026-05-29)**: Vello CPU fallback usa SIMD auto-vectorization. Compiler Rust gera diferentes opcodes per arch (AVX2 / SSE2 / NEON), e ordering de acumulação flutuante difere → hashes Vello scene divergent em Linux multi-arch CI.

**Pipeline canônico em deterministic mode**:
- **Disable auto-vectorization** em `crates/ph2d-vector/Cargo.toml` via `[profile.release-deterministic]` com `codegen-units = 1` + `panic = "abort"` + `rustflags = ["-C", "target-feature=-avx,-avx2,-neon,-sse2,-sse3,-sse4.1,-sse4.2"]` para CPU fallback path.
- OR **integer-only resolvers** em deterministic-critical paths: Q16.16 fixed-point arithmetic instead of f32. Disponível em `ph2d-vector-doc` quando `deterministic: true`.
- **CI gate** `vector_linux_multiarch_determinism` roda fixture replay em Linux x86_64 AVX2, Linux x86_64 SSE2 (no AVX), Linux aarch64 NEON — todos produzem mesmo blake3 hash.

---

## 3.4 SDF Hybrid Pipeline (§8.7, ADR-0065)

### 3.4.1 Modelo

VectorNetwork rasterizada para SDF 2D (compute pass), boolean ops via `min/max`:

- `min(d1, d2)` → união.
- `max(d1, -d2)` → corte (subtract).
- `max(d1, d2)` → intersect.
- `abs(d) - r` → arredondamento (corner round).

### 3.4.2 Shader

`crates/ph2d-vector/shaders/boolean_sdf.wgsl`:

```wgsl
@group(0) @binding(0) var<storage> input_a: SdfTexture;
@group(0) @binding(1) var<storage> input_b: SdfTexture;
@group(0) @binding(2) var<storage, read_write> output: SdfTexture;
@group(0) @binding(3) var<uniform> params: BooleanParams;  // op type

@compute @workgroup_size(8, 8)
fn boolean_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let pos = gid.xy;
    let d_a = sample_sdf(input_a, pos);
    let d_b = sample_sdf(input_b, pos);

    let result = switch params.op_kind {
        case 0: min(d_a, d_b);          // Union
        case 1: max(d_a, -d_b);         // Subtract
        case 2: max(d_a, d_b);          // Intersect
        case 3: max(-min(d_a, d_b), min(d_a, -d_b));  // Exclude (XOR)
        default: d_a;
    };

    output[pos] = result;
}
```

### 3.4.3 Resolution policy

- Default 2× canvas DPI.
- Per-asset override em `.ph2d-vector` metadata.
- Adaptive em zoom (zoom out → lower SDF res).

### 3.4.4 Determinismo opt-in

Quando `deterministic=true`:
- Fixed SDF resolution (no adaptive).
- Ordered reductions (no FMA, no atomic non-deterministic).
- WGSL shader includes `#pragma fma_off`.

### 3.4.5 Limites

- **SDF produz silhueta, não topology editável** — `vector-boolean` em SDF-only mode não permite downstream `vector-roughen` no boolean result se topology necessária. Pipeline reconcile chama Linesweeper async para refresh em commit.
- Resolução SDF determina precisão de bordas: shapes sub-pixel detail podem perder em SDF mode (resolved no commit).

### 3.4.6 Fallback graceful

Se compute shader unavailable (rare: WebGPU older versions), cai para Linesweeper síncrono com warning UI "real-time preview unavailable; performance may suffer".

---

## 3.5 Editor + runtime sharing renderer (HR-7)

### 3.5.1 Single renderer codebase

`ph2d-vector` crate (já existing, expandido em W1.T1.3) tem o renderer. Editor (`ph2d-editor-core` + Painter chrome) E `ph2d-vector-runtime` (W16 crate ship-em-jogo) consomem mesmo renderer.

**Resultado**: WYSIWYG absoluto. O que aparece no editor === o que renderiza no jogo.

### 3.5.2 Feature flags

- `feature = "editor"` — habilita editor chrome, Studios, LLM bridges. Ativado em editor build.
- Sem feature `editor` (default em jogo distribuído) — apenas runtime renderer + state machine. Tree-shaken em release build (HR-7).

---

## 3.6 Procedural fill sub-pipeline

Vide [`05_procedural_fill.md`](05_procedural_fill.md). Resumo: shader graph DAG → WGSL codegen → cache por topology hash + UBO update per frame (resolve crítica B Antigravity).

---

## 3.7 Frame budget breakdown (3.5 ms render sub-budget HR-4)

### 3.7.1 Per-stage budget

Vide §3.1.4 acima.

### 3.7.2 Per-scenario verification

`tests/budget/vector_frame_budget_scenarios.rs`:

| Scenario | Budget alvo | Method |
|----------|-------------|--------|
| Single rect static | 0.3 ms | smoke W1 baseline |
| 50 paths boolean union (Linesweeper) | < 3.5 ms (async) | offload to worker |
| 50 paths boolean union (SDF Hybrid) | ≤ 0.5 ms | inline GPU compute |
| 1000-segment stroke variable width | ≤ 1.0 ms | GPU stroke expansion |
| 50 elements vector runtime + LOD | ≤ 3.5 ms | W16 verification |
| Procedural fill (Noise + Voronoi + Ramp) | ≤ 1.0 ms shader (cached) | W6 verification |
| Diffusion curve Poisson PDE | ≤ 5 ms (off-thread cache) | W7 verification |

---

## 3.8 Cache strategy

### 3.8.1 Path data hash → BezPath cache

Network change rare → BezPath conversion cached.

### 3.8.2 Shader hash → compile cache

Vide [`05_procedural_fill.md §5.4`](05_procedural_fill.md). On-disk cache `~/.cache/ph2d/shaders/<hash>.{wgsl,spv,msl}`.

### 3.8.3 Boolean result hash → cached network

LRU 50 MB. Hash = `(input_a_hash, input_b_hash, op_type)`. Invalidate on edit upstream.

### 3.8.4 SDF texture cache

VectorNetwork → SDF texture cached por hash. Re-render only if network changed OR resolution changed.

---

## 3.9 Cross-platform target matrix

| Platform | Backend | Vector renderer | Notes |
|----------|---------|-----------------|-------|
| Mac (M-series) | Metal (via wgpu) | Vello compute | Ideal: Mac Pro M3, 120 Hz ProMotion |
| Windows | D3D12 (via wgpu) | Vello compute | RTX 30+/RDNA2+ ideal |
| Linux | Vulkan (via wgpu) | Vello compute | Mesa 24+ required |
| iPad Pro (M-series) | Metal | Vello compute | 120 Hz ProMotion |
| iPad standard | Metal | Vello compute (light LOD) | 60 Hz |
| Android top-tier | Vulkan 1.3 (Adreno 660+) | Vello compute | |
| Android entry | Vulkan 1.3 fallback | Vello CPU SIMD | LOD aggressive |
| Web | WebGPU | Vello compute | Chrome 121+, Safari 18+, Firefox 141+ |

---

## 3.9-bis wgpu DeviceLost recovery (Antigravity 3ª iteração L7F1 2026-05-29)

GPU pode falhar em runtime (driver crash, mobile thermal throttling, suspend/resume, Intel iGPU older bug). wgpu surface error `Lost` propaga; sem handling = editor crash + data loss.

**Pipeline canônico**:

```rust
match surface.get_current_texture() {
    Ok(frame) => render_normal(frame),
    Err(wgpu::SurfaceError::Lost) => {
        // 1. Emergency save edit_log to disk imediato
        emergency_save_edit_log()?;
        
        // 2. Mark scene state as needing recreation
        self.scene.invalidate_gpu_resources();
        
        // 3. Try recreate surface + device
        match self.recreate_wgpu_context() {
            Ok(_) => {
                log::info!("wgpu recovered after DeviceLost");
                // Continue rendering normally
            }
            Err(_) => {
                // 4. Fallback to Vello CPU mode
                log::warn!("wgpu unrecoverable; falling back to Vello CPU SIMD");
                self.renderer = Renderer::VelloCpu;
                self.show_toast("Performance reduced — GPU unavailable");
            }
        }
    }
    Err(wgpu::SurfaceError::Outdated) => surface.configure(/* re-config */),
    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout, skip frame"),
    Err(e) => panic!("Unrecoverable surface error: {:?}", e),
}
```

**Emergency save**:
- `edit_log` (event-sourced — full session) salvo em `~/.ph2d/emergency/<session_id>/edit_log.postcard`.
- Asset state hash + last successful render frame metadata.
- Recovery on next launch: detecta emergency saves, prompts user "Recover unsaved work?".

**Vello CPU fallback**:
- Same crate `ph2d-vector` exposes Vello CPU via feature flag.
- Sparse strips SIMD path; ~3-5× slower mas funcional.
- Notification UI persistente "Editor running in fallback mode; restart recommended after work saved".

Gate CI `vector_devicelost_emergency_save` simula DeviceLost + valida edit_log emergency dump intact.

Pattern paralelo ao Painter ADR-0052 (tear-resistant stroke commit).

## 3.10 Anti-patterns (DO NOT)

### 3.10.1 `PipelineLayout::Auto`

Proibido (SKILL_Stack §10.5). Sempre `PipelineLayoutDescriptor` explícito.

### 3.10.2 Compile WGSL por frame

**Resolve crítica B Antigravity.** Topology compile = 1×; params via UBO. Vide [`05_procedural_fill.md §5.4`](05_procedural_fill.md).

### 3.10.3 Linesweeper síncrono no hot-path

**Resolve crítica C Antigravity.** Pipeline draft+reconcile (§3.3.3). Linesweeper exato APENAS em commit, NUNCA em slider drag.

### 3.10.4 Layout reflection / hot-path alloc

HR-3 enforced. Sem `Vec::push` que realloca, sem `Box::new`. Bump arena por frame.

### 3.10.5 Re-render entire scene em mouse move

Dirty rect propagation — só re-render afetado.

---

## Fim do renderer spec

Vello pipeline + GPU stroke + Linesweeper + SDF Hybrid integrados. Editor + runtime sharing single renderer. Frame budget 3.5 ms verified per-scenario.

**Next:** [`04_tools.md`](04_tools.md) (Pen / Pencil / Shape / Select / etc. — interaction com renderer) + [`05_procedural_fill.md`](05_procedural_fill.md) (shader graph fill).
