# ADR-0065 — Vector-SDF Hybrid GPU Pipeline (boolean 120 FPS via min/max compute)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0056 — Vector Network](0056-vector-network-data-model.md), [ADR-0059 — Renderer pipeline](0059-vector-renderer-pipeline.md), [ADR-0063 — Runtime + Physics](0063-vector-runtime-physics-dormant-fractures.md).
**Spec normativa:** [`docs/Vector Module/03_renderer.md §3.4`](../../Vector%20Module/03_renderer.md) + [`docs/Vector Module/14_inovacoes_extraordinarias.md §14.8`](../../Vector%20Module/14_inovacoes_extraordinarias.md).
**Tags:** vector, wave-0, contract, gpu-compute, sdf, boolean

---

## 1. Contexto

Inovação #7 (Proposta 1 Antigravity 1ª iter): **boolean ops em compute shader via SDF 2D** com `min/max` trivial. Resolve simultaneamente:
- Crítica C 1ª iter (Linesweeper síncrono inviável em hot-path sub-9ms ProMotion).
- Gameplay morphing 120 FPS em runtime de jogo (sword-cut shape morph).

Pipeline coordena com Linesweeper exato (ADR-0059 §2.4 draft+reconcile) — SDF é **draft real-time**; Linesweeper async é **commit canônico**.

---

## 2. Decisão

### 2.1 Modelo matemático

VectorNetwork rasterizada para SDF 2D em compute pass; boolean ops triviais em shader:

```wgsl
@compute @workgroup_size(8, 8)
fn boolean_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let pos = vec2<f32>(gid.xy);
    let d_a = sample_sdf(input_a, pos);
    let d_b = sample_sdf(input_b, pos);

    let result = switch params.op_kind {
        case 0u: min(d_a, d_b);                            // Union
        case 1u: max(d_a, -d_b);                           // Subtract
        case 2u: max(d_a, d_b);                            // Intersect
        case 3u: max(-min(d_a, d_b), min(d_a, -d_b));      // Exclude (XOR)
        case 4u: abs(d_a) - params.round_radius;           // Outline (round corner)
        default: d_a;
    };
    output_sdf[gid.xy] = result;
}
```

Shader em `crates/ph2d-vector/shaders/boolean_sdf.wgsl`.

### 2.2 Resolution policy

- **Default 2× canvas DPI** (anti-alias quality vs cost balance).
- **Per-asset override** em `.ph2d-vector` metadata.
- **Adaptive em zoom**: zoom out → lower SDF res; zoom in → higher res.
- **Per-tier** (cross-ref ADR-0068):
  - Heavy: 4× canvas DPI.
  - Standard: 2× canvas DPI (default).
  - Lite: 1.5× canvas DPI.
  - Mobile Core: 1× canvas DPI.

### 2.3 Limites documentados

- **SDF produz silhueta, NÃO topology editável**. `vector-boolean` em SDF-only mode não permite downstream `vector-roughen` no boolean result se topology precisa preservar — Linesweeper async (Tier 2 ADR-0059) reconcile em commit.
- **Shapes sub-pixel detail** podem perder em SDF mode (resolvido no commit via exact).
- **Compute shader required** — WebGPU sem compute = fallback Linesweeper síncrono com warning UI "real-time preview unavailable".

### 2.4 Determinismo opt-in

Quando `deterministic = true`:
- Fixed SDF resolution (no adaptive zoom).
- Ordered reductions (no `subgroupBallot`).
- WGSL shader includes `#pragma fma_off` (naga handles).
- Atomic ops avoided (use storage barriers).

Gate `tests/determinism/boolean_sdf_cross_os.rs`.

### 2.5 Frame budget breakdown

| Stage | Budget |
|-------|--------|
| VectorNetwork → SDF rasterize compute pass | ≤ 0.2 ms |
| Boolean op compute (`min`/`max`) | ≤ 0.3 ms |
| **Total SDF boolean** | **≤ 0.5 ms** |

50+ paths boolean simultâneos cabem em 25 ms (way under HR-4 frame budget; main path stays ≤ 3.5 ms).

### 2.6 Fallback graceful

```rust
fn boolean_pipeline_select(asset: &Asset, ctx: &RenderCtx) -> BooleanPipeline {
    if ctx.compute_shader_available() && ctx.tier >= DeviceTier::Lite {
        BooleanPipeline::SdfHybrid { resolution: ctx.tier.sdf_multiplier() }
    } else {
        log::warn!("compute shader unavailable; falling back to Linesweeper sync");
        ctx.show_toast("real-time preview unavailable; commit will be exact");
        BooleanPipeline::LinesweeperSync
    }
}
```

### 2.7 Integration com Tier 0 Dormant Fractures (cross-ref ADR-0063)

Runtime cut em gameplay:
1. SDF GPU silhueta imediata (this ADR, ≤ 0.5 ms).
2. Dormant Fracture activate (ADR-0063 §2.6, sub-µs) se applicable.
3. Linesweeper async para topology exata (ADR-0059 §3.3).

### 2.8 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| Boolean ops supported (SDF mode) | **5** (Union/Subtract/Intersect/Exclude/Outline-round) | Cobre Pathfinder Illustrator essentials |
| SDF compute pass budget | **≤ 0.5 ms** | Sub-frame target 120 FPS |
| SDF rasterization budget | **≤ 0.2 ms** | Sub-frame target |
| SDF resolution default | **2× canvas DPI** | Anti-alias quality |
| SDF tier multiplier matrix | Heavy 4× / Standard 2× / Lite 1.5× / Mobile Core 1× | Quality vs cost |
| WGSL workgroup size | **8×8** | Match typical mobile GPU subgroup |

---

## 3. Consequências

### 3.1 Positivas

- **120 FPS morphing/cutting** em editor + runtime — vide spec §8.7 example.
- **Resolve crítica C 1ª iter** (Linesweeper síncrono inviável hot-path).
- **Gameplay diferencial** sword-cut visual feedback instant.
- **Cross-platform graceful fallback** (Linesweeper sync se compute unavailable).
- **Determinismo opt-in** preserve replay cross-OS.

### 3.2 Negativas

- **SDF não preserva topology editável** — Linesweeper async obrigatório em commit. Documented.
- **VRAM SDF buffer** ~ 4× canvas area memory (default 2× DPI = 4× pixels). Mitigation: tier-aware resolution.
- **Compute shader dependency** — WebGPU older versions fallback (raro em 2026 Chrome 121+/Safari 18+/Firefox 141+).

### 3.3 Neutras

- 5 boolean ops cobre Pathfinder essentials; 4 mais (Divide, Trim, Merge, Crop) ficam Linesweeper-only.

---

## 4. Alternativas consideradas

### 4.1 Linesweeper síncrono no hot-path (rejeitada — crítica C)

Sem SDF. **Por que rejeitada**: 100+ segments em sub-ms na CPU móvel inviável. Vide ADR-0059 §3.3 absorção.

### 4.2 SDF como único renderer (sem Linesweeper) (rejeitada — topology lost)

Apenas SDF. **Por que rejeitada**: topology lost; downstream modifiers impossíveis. Linesweeper async é canônico em commit.

### 4.3 Single high-res SDF (sem tier-aware) (rejeitada — Mobile Core)

Sempre 4× DPI. **Por que rejeitada**: estoura Mobile Core VRAM budget.

---

## 5. Implementação (Wave 3 + Wave 5)

- **T3.3** (W3): Boolean foundation com SDF draft mode ativo.
- **T5.2** (W5): SDF Hybrid full ativo (50+ paths simultaneos 120 FPS).
- **T16.4** (W16): Runtime gameplay integration (cross-ref ADR-0063).

Gates: `vector_sdf_real_time` + `boolean_sdf_cross_os` + `vector_sdf_fallback_graceful`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/03_renderer.md §3.4`](../../Vector%20Module/03_renderer.md) + [§14.8 Inovação #7](../../Vector%20Module/14_inovacoes_extraordinarias.md).
- SDF boolean ops math (Inigo Quilez canon): <https://iquilezles.org/articles/distfunctions2d/>
- msdfgen (multi-channel SDF reference): <https://github.com/Chlumsky/msdfgen>
- ADR-0059 renderer pipeline (parent context).
- ADR-0063 runtime physics (Tier 0 Dormant integration).

---

## Amendment 1 — implementation (2026-06-04, Coord)

This ADR decided the *draft+reconcile* shape (§1–§5); this amendment records the
**implementation placement + phasing** as the draft layer lands.

- **Placement = satellite crate `ph2d-vector-sdf`** (not inside `ph2d-vector`).
  `ph2d-vector` has no GPU layer (it renders via vello); coupling it to wgpu for
  the SDF would be the wrong cut. A drop-crate mirrors the `ph2d-vector-kurbo`
  precedent (the reconcile half) and confines the SDF compute. Deps: `ph2d-
  vector-doc` + `glam` (the GPU port adds `ph2d-gpu` + `wgpu` + `bytemuck`).
- **Algorithm (analytic, not jump-flood):** flatten each region's cubic boundary
  to a fixed-subdivision polyline (renderer's `c1 = start+out_at_start` /
  `c2 = end+in_at_end` convention), then per grid cell take `min` unsigned
  distance to the boundary edges × the inside sign (NonZero winding-number /
  EvenOdd ray-crossing). `boolean_sdf` is the §2.1 `min/max` combine of two
  co-located grids (Union/Subtract/Intersect/Exclude/Outline). The other 4 ops
  are topology → Linesweeper only. Determinism (§2.4): fixed `SUBDIV_PER_SEGMENT`
  + fixed grid + ordered per-pixel reductions + `BTreeMap` (no random hasher).
- **Phasing = CPU-core-first → GPU-parity** (the layer_compositor discipline):
  **Phase 1 (LANDED):** the pure-Rust core (`network_sdf` / `boolean_sdf`,
  5 tests) — the source of truth, a CPU silhouette fallback (usable at draft
  res), AND the GPU parity oracle. **Phase 2 (next):** `boolean_sdf.wgsl` (the
  same per-pixel SDF + `min/max`) + a wgpu compute pipeline mirroring
  `ph2d-render/src/layer_compositor/`, gated by a Metal parity test vs Phase 1.
  **Phase 3:** wire the draft into `vector_graph_bridge` (silhouette during drag)
  alongside the exact-engine reconcile.
