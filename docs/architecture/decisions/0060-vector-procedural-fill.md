# ADR-0060 — Procedural fill shader graph (topology vs UBO + diffusion curve Poisson)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0056 — Vector Network](0056-vector-network-data-model.md), [ADR-0058 — Geometry graph domain](0058-vector-geometry-graph.md), [ADR-0059 — Renderer pipeline](0059-vector-renderer-pipeline.md).
**Spec normativa:** [`docs/Vector Module/05_procedural_fill.md`](../../Vector%20Module/05_procedural_fill.md).
**Tags:** vector, wave-0, contract, shader-graph, diffusion-curve, ubo

---

## 1. Contexto

Regions de VectorNetwork aceitam fills procedurais via **shader graph DAG** compilável a WGSL on-the-fly (`Noise`, `Voronoi`, `Ramp`, `Mix`, `Bump`, `Coord`, `Math`, `Image-sample`, `Time`, `Diffusion`). Inovação #2 Mesh Gradient via Diffusion Curve substitui mesh-patches hand-author do Illustrator.

Crítica B Antigravity 1ª iter (compile stutter por frame) + L4F1 2ª iter (WoS Poisson 64 spp inviável mobile) + L2F1 3ª iter (Windows MAX_PATH) absorvidas integralmente.

---

## 2. Decisão

### 2.1 Crate foundational `ph2d-vector-fill`

```
crates/ph2d-vector-fill/
├── src/
│   ├── lib.rs                    FillGraph + Node enum
│   ├── wgsl_codegen.rs           DAG → WGSL string + topology hash
│   ├── ubo.rs                    FillParamsUbo (params dinâmicos por frame)
│   ├── poisson_cpu.rs            CPU multigrid baseline (Mobile Core fallback)
│   ├── diffusion_curve.rs        UI authoring + GPU dispatch
│   └── cache.rs                  On-disk shader cache via `directories` crate
├── shaders/
│   ├── diffusion.wgsl            Walk-on-Spheres Monte Carlo solver
│   └── bilateral_upsample.wgsl   JBU 2-pass (denoise + guided upscale)
└── tests/
```

### 2.2 17 fill nodes canônicos (FillNode enum)

`Solid` / `LinearGradient` / `RadialGradient` / `MeshGradient (diffusion curve)` / `Pattern` (Painter brush via `ph2d-brush-traits`) / `ProceduralShader` (recursive) / `Image` / `Noise` (Simplex/Perlin/Worley/Fbm) / `Voronoi` / `Ramp` / `Mix` / `Bump` / `Coord` / `Math` / `ImageSample` / `Time` / `Random` (deterministic w/ seed).

### 2.3 Topology vs Params split — resolve crítica B Antigravity 1ª iter

**Naïve approach**: recompilar WGSL toda vez que param escalar muda. 60 Hz animação = 10-100 ms stalls cada frame → quebra HR-4.

**Solução canônica**:

1. **Topology hash** (which nodes + connections) → compile WGSL **1×** via naga → cache on-disk + memory LRU.
2. **Params escalares** (cor, frequency, ramp position, time, coord) → empacotados em `FillParamsUbo` atualizado per frame com zero alloc (HR-3).
3. **Topology change** → spinner UI + compile off-thread + swap atômico ao terminar; durante compile, render usa template anterior.
4. **Enum control via UBO indexing** (L1F2 Antigravity 2ª iter): NoiseKind/etc. NÃO geram WGSL conditional estático; switch interno indexado por `u32` no UBO:

```wgsl
fn noise_kind_dispatch(coord: vec2<f32>, kind_idx: u32) -> f32 {
    switch kind_idx {
        case 0u: { return simplex_noise(coord); }
        case 1u: { return perlin_noise(coord); }
        case 2u: { return worley_noise(coord); }
        case 3u: { return fbm_noise(coord); }
        default: { return 0.0; }
    }
}
```

Mudança enum → UBO update (~100 µs), **zero recompile**.

Gate `procedural_fill_no_recompile_on_animate`: animate 60 frames param + enum = 0 recompilations.

### 2.4 Shader cache on-disk via `directories` crate (L2F1 Antigravity 3ª iter — cross-platform)

```rust
// Per OS via directories::ProjectDirs::cache_dir()
// Linux: ~/.cache/ph2d/shaders/
// macOS: ~/Library/Caches/com.ph2d.engine/shaders/
// Windows: %LOCALAPPDATA%\ph2d\engine\cache\shaders\ (UNC \\?\ if > 260 char)
// Web: OPFS /ph2d/shaders/
```

Cache key: `blake3(topology_layout + backend_id)`. LRU 1 GB cap. Cache hit-rate target >95%.

### 2.5 Diffusion curve Poisson PDE — tier-aware resolution + JBU multi-pass (L1F4 + L4F1 Antigravity 2ª+3ª iter)

**Naïve 64 spp @ 1080p**: 256 ms em A14 mobile (~5 TFLOPS). Inviable.

**Solução**:

| Tier | Resolution | Samples | Budget | Algorithm |
|------|-----------|---------|--------|-----------|
| Heavy (desktop) | 1080p | 64 spp | ≤ 5 ms | WoS GPU compute |
| Standard (iPad Pro) | 540p (0.5×) | 32 spp | ≤ 4 ms | WoS GPU compute + JBU upscale |
| Lite (Android top) | 270p (0.25×) | 16 spp | ≤ 3 ms | WoS GPU compute + JBU upscale |
| Mobile Core (entry) | CPU multigrid 4-level | — | ≤ 15 ms (off-thread) | Multigrid CPU SIMD fallback |
| Web | match Standard | — | ≤ 4 ms | WoS WebGPU OR CPU fallback se compute slow |

**JBU multi-pass (L1F4 Antigravity 3ª iter)** em 2 passes (não single-pass 21×21 que estouraria budget):
- **Pass 1**: bilateral 3×3 ou 5×5 kernel @ 270p low-res — elimina ruído WoS. ~0.1 ms.
- **Pass 2**: guided upscale 3×3 @ 1080p high-res — guidance signal = curve distance field; preserve sharp boundaries. ~0.2 ms.
- **Total ≤ 0.3 ms**.

Gate `vector_diffusion_curve_tier_budget`.

### 2.6 Determinismo opt-in (cross-platform shader bit-identity)

Quando `deterministic=true`:
- Shader includes `#pragma fma_off` (naga handles).
- Ordered reductions (avoid `subgroupBallot` non-deterministic).
- Fixed thread group size.
- WoS fixed seed (per-curve hash).

Gate `tests/determinism/procedural_fill_cross_os.rs`.

### 2.7 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| Total fill nodes | **17** | Adição requer ADR amendment |
| Topology hash WGSL cache LRU memory | **256 MB** | Balance disk vs memory |
| Topology hash WGSL cache on-disk | **1 GB** | LRU eviction |
| Cache hit-rate target | **>95%** | Gate `procedural_fill_no_recompile_on_animate` |
| Diffusion curve WoS samples (Heavy) | **64 spp** | Quality target Heavy tier |
| Diffusion curve WoS samples (Mobile Core) | **CPU multigrid only** | Compute unavailable |
| Shader compile timeout off-thread | **5 seconds** | UI shows "compiling…" e usa template anterior |

---

## 3. Consequências

### 3.1 Positivas

- **Zero compile stutter on animate** (crítica B Antigravity 1ª iter resolved fully via UBO + enum dispatch).
- **Diffusion curve mesh gradient** substitui hand-author mesh patches Illustrator — Inovação #2 ativa.
- **Tier-aware Poisson resolution** ativa Mobile Core sem crash + Heavy quality preservada.
- **JBU 2-pass upsample** cabe em 0.3 ms budget; single-pass mega-kernel não cabia.
- **Cross-platform cache via `directories`** resolve L2F1 (Windows MAX_PATH).

### 3.2 Negativas

- **Cache size 1 GB on-disk** pode incomodar usuários com SSD pequeno. Mitigação: LRU eviction + UI setting "Clear shader cache".
- **Diffusion curve solver complexity** (multigrid + WoS + bilateral upscale + tier matrix) — implementação research-grade. Wave 7 estimate aumentado 21d → 35d + CPU prototype baseline.
- **WGSL all-variants em switch interno** adds ~2-5 KB per shader cached. Aceito (trade-off vs recompile cost).

### 3.3 Neutras

- Naga compilation time per topology change ~50-300 ms (off-thread).

---

## 4. Alternativas consideradas

### 4.1 Mesh gradient via hand-authored patches Illustrator-style (rejeitada)

Replicar Illustrator's mesh gradient UX. **Por que rejeitada**: hand-author é painful (Illustrator's biggest UX pain); diffusion curve via Poisson PDE unifica mesh gradient + diffusion curve em mesma matemática ([Unified Smooth Vector Graphics 2024 arXiv 2408.09211](https://arxiv.org/pdf/2408.09211)).

### 4.2 Naga recompile per param change (rejeitada — crítica B)

Naïve approach. **Por que rejeitada**: 10-100 ms compile stalls quebram HR-4; vide §2.3 absorção L1F2.

### 4.3 Single-pass mega-kernel JBU (rejeitada — L1F4)

15×15 ou 21×21 single-pass. **Por que rejeitada**: ~256 samples/pixel × 2M pixels estoura 0.3 ms budget mobile. 2-pass JBU cabe.

### 4.4 Hardcoded `~/.cache` UNIX path (rejeitada — L2F1)

PH2D-only Linux path. **Por que rejeitada**: Windows MAX_PATH 260-char limit; macOS uses `~/Library/Caches`. `directories` crate handles cross-platform.

### 4.5 ML neural shader (Antigravity 3ª iter L7F2 — V2.0 stretch)

Mini-UNet em compute shader para diffusion curves AI-based. **Por que adiada**: ML model embed ~50 MB binary + training data overhead inviavel v1.0. Documentar future direction.

---

## 5. Implementação (Wave 6+)

- **W6**: `ph2d-vector-fill` skeleton + WGSL codegen + topology vs UBO split (this ADR).
- **W7**: Diffusion curve Poisson solver (Wave 7 35-day estimate per L8F2 Antigravity 3ª iter).

Gates ativos: `procedural_fill_no_recompile_on_animate` + `vector_diffusion_curve_tier_budget` + `procedural_fill_cross_os_bit_identity`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/05_procedural_fill.md`](../../Vector%20Module/05_procedural_fill.md) (510 linhas).
- Orzan SIGGRAPH 2008 diffusion curves: <https://dl.acm.org/doi/10.1145/1360612.1360691>
- Unified Smooth Vector Graphics 2024: <https://arxiv.org/pdf/2408.09211>
- `directories` crate: <https://crates.io/crates/directories>
- naga + wgpu compilation: <https://github.com/gfx-rs/naga>
