# ADR-0059 — Vector renderer pipeline (Vello + GPU stroke + draft+reconcile boolean)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [ADR-0020 — Surface lifecycle](0020-surface-lifecycle.md), [ADR-0056 — Vector Network data model](0056-vector-network-data-model.md), [ADR-0058 — Geometry graph domain](0058-vector-geometry-graph.md).
**Sub-contratos relacionados:** [ADR-0065 — SDF Hybrid GPU](0065-vector-sdf-hybrid-gpu.md) (boolean GPU compute compute pass).
**Spec normativa:** [`docs/Vector Module/03_renderer.md`](../../Vector%20Module/03_renderer.md).
**Tags:** vector, wave-0, contract, renderer, gpu-compute, draft-reconcile

---

## 1. Contexto

Vector Module deve renderizar em **frame budget 3.5 ms** (HR-4 sub-budget Render) com **paridade visual cross-platform** + **infinite zoom** + **integração runtime de jogo**. Decisões fundamentais:

1. Qual renderer? Vello (GPU compute) é o stack canônico PH2D (SKILL_Stack §5).
2. Como handle boolean ops em hot-path stylus (sub-9 ms ProMotion)? — pipeline **draft+reconcile** em 3 modos (Antigravity 1ª iter L1F5 + L1F1).
3. Como mitigar wgpu DeviceLost? — emergency edit_log save + Vello CPU fallback (Antigravity 3ª iter L7F1).
4. Como compartilhar renderer entre editor e runtime de jogo? — single crate `ph2d-vector` (encapsulação L6F1).

---

## 2. Decisão

### 2.1 Vello 0.8 pinado (paridade SKILL_Stack §5)

Pin deliberado **Vello 0.8** + **wgpu 28** (não upstream 0.9/29). Upgrade plano em W18 FREEZE event (vide §11.C Antigravity 2ª iter L2F2 — pin preserved).

### 2.2 Two-layer architecture (correção L2F1 Antigravity 2ª iter)

| Layer | Algorithm | Use case |
|-------|-----------|----------|
| **Vello GPU** | **Prefix-sum stage pipeline** (coarse → fine → ratification → fine rasterize) em compute shaders WGSL | Primary path em todos targets com WebGPU compute |
| **Vello CPU** (fallback) | **Sparse strips** ([Stampfl ETH 2025 thesis](https://ethz.ch/content/dam/ethz/special-interest/infk/inst-pls/plf-dam/documents/StudentProjects/MasterTheses/2025-Laurenz-Thesis.pdf)) — CPU SIMD (SSE2/AVX/AVX2/AVX512/NEON) | Fallback devices sem compute support OR DeviceLost recovery |
| **Vello Hybrid** | Combina GPU + CPU per-frame | Mobile thermal throttling adaptive |

**Crítico**: sparse strips é **arquitetura CPU-only** (L2F1 Antigravity 2ª iter — correção alucinação spec original que atribuía sparse strips ao pipeline GPU). Vello GPU = prefix-sum.

### 2.3 GPU stroke expansion (Levien+Uguray 2024)

Variable-width strokes via [paper ACM 10.1145/3675390](https://arxiv.org/pdf/2405.00127) — Euler spirals (clothoids) approximation paralela em compute pass único. Já integrado em Vello; PH2D consome via `vello::Scene::stroke()` API.

### 2.4 Pipeline boolean draft+reconcile (resolve crítica C Antigravity 1ª iter)

3 modos selectivos por contexto:

#### Modo 1: Draft preview (hot-path)
- **Budget**: ≤ 1 ms.
- **Algorithm**: CPU naive cubic Bézier clipping aproximado (não-exato).
- **Usado em**: slider drag, stylus motion contínua, real-time interactivity.

#### Modo 2: SDF Hybrid GPU (vide ADR-0065)
- **Budget**: ≤ 0.5 ms compute pass.
- **Algorithm**: VectorNetwork → SDF 2D rasterize compute pass; boolean via `min(d1, d2)` união, `max(d1, -d2)` corte.
- **Usado em**: 50+ paths boolean simultâneos a 120 FPS + gameplay morphing.

#### Modo 3: Linesweeper exato (async)
- **Budget**: 1-50 ms off-thread (background worker).
- **Algorithm**: [Joe Neeman Linesweeper](https://github.com/jneem/linesweeper) full robust.
- **Usado em**: commit (mouse-up / pencil-lift) — topology canônica.
- **Cache**: by hash (graph input) LRU 50 MB.

### 2.5 Linesweeper determinismo — app-layer responsibility (L2F3 Antigravity 2ª iter correção)

Linesweeper crate **não expõe** `deterministic_mode` flag nativamente. Determinismo bit-identical cross-platform é responsabilidade da camada de aplicação PH2D:

1. **Pré-ordenação canônica** segments antes de invocar linesweeper (sort por `(start.x, start.y, end.x, end.y)` Q16.16 fixed-point).
2. **Coordenadas em fixed-point Q16.16** quando `VectorNetwork::deterministic = true` (ADR-0056 §2.7).
3. **Ordered reductions** em post-process (regions ordenadas por `(area_signed, centroid.x, centroid.y)` fixed-point).
4. **FMA off** + sem `dpdx`/`dpdy` em pipelines determinísticos.

Test `tests/determinism/boolean_cross_os.rs`: 100 boolean ops bit-identical em Linux + Mac + Windows.

### 2.6 Linux multi-arch SIMD determinism (L2F2 Antigravity 3ª iter)

Vello CPU fallback usa SIMD auto-vectorization; compiler Rust gera diferentes opcodes per arch (AVX2 / SSE2 / NEON); ordering acumulação flutuante difere → divergent hashes Linux multi-arch CI.

**Mitigation em deterministic mode**:
- Disable auto-vec via `[profile.release-deterministic]` em `crates/ph2d-vector/Cargo.toml` (`-C target-feature=-avx,-avx2,-sse2,-sse3,-sse4.1,-sse4.2,-neon`).
- OR integer-only resolvers (Q16.16) em deterministic-critical paths.
- Gate `vector_linux_multiarch_determinism` (CI Linux x86_64 AVX2 + non-AVX + aarch64 NEON).

### 2.7 wgpu DeviceLost recovery (L7F1 Antigravity 3ª iter)

GPU pode falhar em runtime (driver crash, mobile thermal throttling, suspend/resume, Intel iGPU older bug). Pipeline canônico:

```rust
match surface.get_current_texture() {
    Ok(frame) => render_normal(frame),
    Err(wgpu::SurfaceError::Lost) => {
        emergency_save_edit_log()?;                  // edit_log dump ~/.ph2d/emergency/
        self.scene.invalidate_gpu_resources();
        match self.recreate_wgpu_context() {
            Ok(_) => log::info!("wgpu recovered"),
            Err(_) => {
                self.renderer = Renderer::VelloCpu;  // sparse strips SIMD fallback
                self.show_toast("Performance reduced — GPU unavailable");
            }
        }
    }
    Err(wgpu::SurfaceError::Outdated) => surface.configure(/* re-config */),
    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout, skip frame"),
    Err(e) => panic!("Unrecoverable surface error: {:?}", e),
}
```

Pattern paralelo a Painter ADR-0052 (tear-resistant stroke commit). Gate `vector_devicelost_emergency_save`.

### 2.8 Vello/kurbo/peniko encapsulação em single crate (L6F1 Antigravity 3ª iter long-tail maintenance)

Vello upgrade per quarter; spread em N crates = N-cost upgrade. Solução:
- **`ph2d-vector`** crate (existing, expanded em W1.T1.3) é o **único** com direct dep em `vello::*` / `kurbo::*` / `peniko::*`.
- Outros 31 crates Vector consomem PH2D-domain types via `ph2d_vector::{Pos2d, Network, Bez, ...}` re-exports.
- Arch-gate `vello_kurbo_only_in_ph2d_vector`: zero `use vello::*` ou `use kurbo::*` fora de `ph2d-vector` crate.

### 2.9 Editor + runtime sharing renderer (HR-7)

`ph2d-vector` é consumido por:
- Editor (`ph2d-editor-core` + chrome) — feature `editor` ativada.
- Runtime de jogo (`ph2d-vector-runtime`, ADR-0063) — feature `editor` OFF; tree-shake.

WYSIWYG absoluto: o que aparece no editor === o que renderiza no jogo.

### 2.10 Frame budget breakdown (3.5 ms HR-4)

| Stage | Budget |
|-------|--------|
| Network → BezPath conversion | 0.3 ms |
| Stroke style application | 0.2 ms |
| Fill style (cached procedural shader) | ~0 ms |
| Vello scene construction | 0.5 ms |
| Vello GPU compute pass | 1.5 ms |
| Composite + present | 0.5 ms |
| **Reserved buffer** | 0.5 ms |
| **Total** | **3.5 ms** |

Gate `tests/budget/vector_frame_budget_scenarios.rs`.

### 2.11 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| Vello version pin | **0.8** | Paridade SKILL_Stack §5; upgrade plano W18 FREEZE event |
| wgpu version pin | **28** | Paridade Vello 0.8 |
| Frame budget render sub-allocation | **3.5 ms** | HR-4 |
| Boolean cache LRU | **50 MB** | Balance speed vs memory |
| SDF resolution default | **2× canvas DPI** | ADR-0065 detalha |
| Linesweeper async max network size | **50k segments per side** | Acima disso, SDF mode only |

---

## 3. Consequências

### 3.1 Positivas

- **Infinite zoom + GPU compute** — paridade Linebender state-of-the-art; nenhum competitor mainstream entrega (Skia hybrid; Cairo CPU; Direct2D Windows-only).
- **Pipeline draft+reconcile** resolve L1F5 Antigravity 1ª iter (sub-9 ms ProMotion + 50+ paths boolean simultâneos).
- **DeviceLost recovery** previne data loss + graceful CPU fallback.
- **Vello encapsulation single-crate** mitiga long-tail upgrade cost (1 crate updated per Vello version vs 32).
- **Sparse strips correction** (L2F1) elimina alucinação técnica em spec.

### 3.2 Negativas

- **Vello 0.8 alpha churn per quarter** (ADR-0004) — risco upgrades destrutivos. Mitigação: encapsulation §2.8 + W18 FREEZE upgrade plan.
- **3 modos boolean** = complexidade implementação (CPU draft + SDF GPU + Linesweeper async). Justificada por performance demands.
- **Linesweeper beta** (Joe Neeman crate). Mitigação: fallback Clipper se emergência; PH2D contribute upstream se needed.
- **Determinismo opt-in custa ~3-5×** (Q16.16 + ordered reductions + FMA off). Aceito para use cases que beneficiam.

### 3.3 Neutras

- Memory budget 200 MB VRAM desktop / 80 MB mobile (vide ADR-0068 tier).

---

## 4. Alternativas consideradas

### 4.1 Skia (Google) (rejeitada — não Rust + CPU/GPU hybrid)

Skia é mature. **Por que rejeitada**: Skia é C++; FFI custo PH2D não justifica (toda engine Rust); GPU compute pipeline Vello é mais moderno (prefix-sum); paridade infinity zoom não confirmed.

### 4.2 Direct2D Windows-only (rejeitada — não cross-platform)

Direct2D é GPU. **Por que rejeitada**: HR-1 cross-platform obrigatório; Windows-only inviável.

### 4.3 wgpu native sem Vello (rejeitada — reinventar a roda)

Escrever próprio compute pipeline. **Por que rejeitada**: Vello 0.8 é state-of-the-art + paper-backed (Levien GPU stroke 2024). Reinventar = anos de research.

### 4.4 Clipper2 boolean library (rejeitada — não robust em degenerate cases)

Clipper2 é mature. **Por que rejeitada**: quebra em near-tangent / coincident edges (real-world vector art hits constantly). Linesweeper Joe Neeman aborda esses casos com ordering-first approach. Fallback Clipper considerado em emergência apenas.

### 4.5 Vello 0.9 / wgpu 29 upstream (rejeitada — pin deliberado)

Antigravity 2ª iter L2F2 sugeriu upgrade. **Por que rejeitada parcial**: SKILL_Stack §5 pin é deliberado para paridade Painter + MSRV. Upgrade W18 FREEZE event coordenado.

---

## 5. Implementação (Wave 1+)

- **T1.3**: `ph2d-vector` expansão (Vello pipeline + sparse strips reference).
- **T3.3**: Boolean foundation (draft+reconcile pipeline + Linesweeper integration).
- **T5.2**: SDF Hybrid full ativo (vide ADR-0065).
- **T7.0-T7.1**: Diffusion curve (ADR-0060).

Gates ativos a partir de W1: `vector_no_alloc_hot_path` + `vector_frame_budget_scenarios` + `vector_devicelost_emergency_save`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/03_renderer.md`](../../Vector%20Module/03_renderer.md) (390 linhas).
- Vello GitHub: <https://github.com/linebender/vello>
- Stampfl ETH 2025 thesis (sparse strips CPU): <https://ethz.ch/content/dam/ethz/special-interest/infk/inst-pls/plf-dam/documents/StudentProjects/MasterTheses/2025-Laurenz-Thesis.pdf>
- Levien+Uguray GPU-friendly stroke expansion 2024: <https://arxiv.org/pdf/2405.00127>
- Linesweeper (Joe Neeman): <https://joe.neeman.me/posts/linesweeper/>
- ADR-0020 surface lifecycle (pattern paralelo DeviceLost).
- ADR-0052 Painter tear-resistant commit (pattern paralelo emergency save).
