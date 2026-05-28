# 05 — Procedural Fill (shader graph 2D, diffusion curves, topology vs UBO)

> Spec do **shader graph procedural** para fills do Vector Module. DAG de fill nodes (Solid / Linear / Radial / Mesh / Diffusion / Pattern / ProceduralShader / Image / Noise / Voronoi / Ramp / Mix / Bump / Coord / Math / Image-sample / Time). WGSL codegen on-the-fly. **Topology compile 1× + UBO update por frame** (resolve crítica B Antigravity — zero compile stutter on animate). Mesh gradient via **diffusion curve Poisson PDE** (substitui mesh-patches hand-author).
>
> **ADR ratificador:** ADR-0060 (Procedural fill shader graph).
> **Spec gêmeos:** [`02_geometry_graph.md`](02_geometry_graph.md) (FillRef referenciado em Regions) + [`03_renderer.md`](03_renderer.md) (compute shader pipeline integration).

## 5.1 Fill graph DAG model

### 5.1.1 Conceito

Cada `Region` de uma VectorNetwork pode receber um `FillStyle::ProceduralShader(graph_ref)`. O `graph_ref` aponta para um **fill graph DAG** — sequence de fill nodes que compõe um WGSL shader compilado.

```
Region.fill: ProceduralShader → FillGraph (DAG):
    Coord → Noise (frequency=4, octaves=3) → Ramp (gradient palette) → output color
    Coord → Voronoi (cells=32, jitter=0.5) → Mix (with above) → final output
```

### 5.1.2 Estrutura

```rust
pub struct FillGraph {
    pub nodes: SmallVec<[FillNode; 16]>,
    pub connections: SmallVec<[Connection; 32]>,
    pub output_node_id: NodeId,
}

pub enum FillNode {
    // Generators
    Solid { color: ColorOklch },
    LinearGradient { stops: Vec<GradientStop>, angle: f32 },
    RadialGradient { stops: Vec<GradientStop>, center: Vec2, radius: f32 },
    MeshGradient { gradient_id: GradientId },  // via diffusion curve (§5.6)
    Pattern { pattern_ref: PatternRef },  // Painter brush via §8.2 bridge
    ProceduralShader { shader_id: ShaderId },  // recursive (rare)
    Image { image_ref: ImageRef },
    
    // Procedural primitives
    Noise { kind: NoiseKind, frequency: f32, octaves: u32 },
    Voronoi { cells: u32, jitter: f32 },
    Ramp { palette: Palette },
    
    // Combinators
    Mix { mode: BlendMode, factor: f32 },
    Bump { strength: f32 },
    
    // Coords + math
    Coord { mode: CoordMode },  // local, world, screen, polar
    Math { op: MathOp },  // add, mul, abs, sin, etc.
    ImageSample { image_ref: ImageRef, uv_input: NodeId },
    
    // Time-based (Temporal effect)
    Time,
}

pub enum NoiseKind {
    Simplex,
    Perlin,
    Worley,
    Fbm { lacunarity: f32, persistence: f32 },
}
```

### 5.1.3 Connection

```rust
pub struct Connection {
    pub from_node: NodeId,
    pub from_output: OutputName,
    pub to_node: NodeId,
    pub to_input: InputName,
}
```

### 5.1.4 Validation

DAG must be acyclic. `output_node_id` reachable from all dependents. Type-check at edit time (Color → Color match; f32 → f32 match).

---

## 5.2 17 fill nodes (canon)

Lista canônica (§5.1.2 enum). Cada node:

| Node | Tipo | Params | Output |
|------|------|--------|--------|
| Solid | Generator | color: OKLCH | Color |
| LinearGradient | Generator | stops, angle | Color |
| RadialGradient | Generator | stops, center, radius | Color |
| MeshGradient | Generator | gradient_id (diffusion curve) | Color |
| Pattern | Generator | pattern_ref (brush) | Color (+alpha) |
| ProceduralShader | Generator (recursive) | shader_id | Color |
| Image | Generator | image_ref | Color |
| Noise | Procedural | kind, freq, oct | f32 |
| Voronoi | Procedural | cells, jitter | f32 |
| Ramp | Procedural | palette | f32 → Color |
| Mix | Combinator | mode, factor | Color |
| Bump | Combinator | strength | Vec3 normal |
| Coord | Coord | mode | Vec2 |
| Math | Math | op | f32/Vec2/Vec3 |
| ImageSample | Sampler | image_ref, uv | Color |
| Time | Temporal | — | f32 |
| Random | Procedural (deterministic w/ seed) | seed | f32 |

---

## 5.3 WGSL codegen (DAG → WGSL string)

### 5.3.1 Visitor pattern

```rust
fn codegen(graph: &FillGraph) -> WgslSource {
    let mut wgsl = String::new();
    
    // 1. Boilerplate
    wgsl.push_str(WGSL_HEADER);
    
    // 2. Per-node function emission (topological sort)
    let order = topological_sort(graph);
    for node_id in order {
        let node = &graph.nodes[node_id as usize];
        wgsl.push_str(&emit_node(node, graph, node_id));
    }
    
    // 3. Main function calling output node
    wgsl.push_str(&format!(
        "fn fill_main(coord: vec2<f32>) -> vec4<f32> {{ return node_{}(coord); }}",
        graph.output_node_id
    ));
    
    WgslSource(wgsl)
}

fn emit_node(node: &FillNode, graph: &FillGraph, id: NodeId) -> String {
    match node {
        FillNode::Solid { color } => format!(
            "fn node_{}(coord: vec2<f32>) -> vec4<f32> {{ return vec4<f32>({}, {}, {}, {}); }}",
            id, color.l, color.c, color.h, color.alpha
        ),
        FillNode::Noise { kind, frequency, octaves } => emit_noise(*kind, *frequency, *octaves, id),
        FillNode::Voronoi { cells, jitter } => emit_voronoi(*cells, *jitter, id),
        // ... other nodes
    }
}
```

### 5.3.2 Cache key

```
shader_hash = blake3(
    topological_layout(graph)  // structure only, no params
    + target_backend_id  // wgpu adapter
)
```

Params NÃO entram no hash (vão para UBO; vide §5.4).

### 5.3.3 Output

WGSL string → naga validate + parse → wgpu shader module + pipeline.

---

## 5.4 Topology vs Params split (resolve crítica B Antigravity)

### 5.4.1 O problema

Naïve approach: recompila WGSL toda vez que param escalar muda. Frame rate 60 Hz → ~60 recompilations/seg → 10-100 ms stalls cada → frame budget destruído.

### 5.4.2 Solução canônica

**Split topology de params:**

1. **Topology** = which nodes presentes + which connections.
   - Hash de structure (sem param values).
   - WGSL compile 1× por (topology hash, backend).
   - Cache em memória + on-disk.

2. **Params** = valores numéricos (cor, frequência, time, etc.).
   - Empacotados em `UniformBuffer` (UBO) struct.
   - Atualizado per frame com `wgpu::Queue::write_buffer` (zero alloc).
   - Shader lê UBO via `@group(0) @binding(0) var<uniform> params: FillParams`.

### 5.4.3 UBO layout

```rust
#[repr(C, align(16))]
pub struct FillParamsUbo {
    pub solid_colors: [Vec4; 16],   // max 16 solid colors
    pub noise_freqs: [f32; 16],
    pub voronoi_cells: [u32; 16],
    pub mix_factors: [f32; 16],
    pub time: f32,
    pub _padding: [f32; 3],
}
```

WGSL accesses:
```wgsl
@group(0) @binding(0) var<uniform> params: FillParams;

fn node_5(coord: vec2<f32>) -> vec4<f32> {
    // Uses params.solid_colors[3], params.noise_freqs[2], etc.
}
```

### 5.4.4 Animation hot loop — enum control indexing (revisado Antigravity L1F2 2ª iteração)

Quando user anima `noise.frequency` em curva 60 Hz:
- Cada frame: `queue.write_buffer(ubo, &updated_params)` → ≤ 100 µs.
- Compile shader: **zero** (topology unchanged).
- Result: smooth 60-120 Hz animation sem stutter.

**Crítica L1F2 absorvida — enum control DENTRO da UBO** (não codegen condicional estático):

Para params enum como `NoiseKind { Simplex, Perlin, Worley, Fbm }` que controlam **branching geometry**, **NÃO** gerar WGSL condicional estático (que requer recompile quando enum muda). Em vez disso:

1. **WGSL compilado** inclui **TODAS as variantes** numa switch interna:
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
2. **Enum value vai no UBO** como `u32`: `params.noise_kinds[node_id] = NoiseKind::Worley as u32`.
3. **Mudança do enum em runtime** = UBO update (~100 µs), **zero recompile**.

Trade-off: shader ligeiramente maior (todas variantes compiladas), mas WGSL optimizer normalmente dead-code-elimina branches não usados. Memory: ~2-5 KB extra per shader cached.

**Gate CI**: `procedural_fill_enum_change_no_recompile` — anima `kind` enum a cada 100ms por 5 segundos = 0 recompilations.

### 5.4.5 Topology change

Quando user pluga node novo no graph (e.g., adiciona Voronoi entre Noise e Ramp):
1. UI mostra spinner "compiling shader..." discreto no HUD.
2. Off-thread compile (rayon worker).
3. Durante compile, render usa template antigo (no glitch).
4. Compile done → swap atômico do pipeline.
5. Frame seguinte usa novo shader.

Compile time tipico: 50-300 ms (naga + wgpu pipeline creation).

### 5.4.6 Gate CI

`tests/budget/procedural_fill_no_recompile_on_animate.rs`:
- Fixture: graph com 5 nodes; anima Solid.color em curve 60 Hz por 5 segundos.
- Assertion: 0 recompilations (verifica via counter em wgpu device).
- Falha se contar ≥ 1.

---

## 5.5 Shader compile cache (on-disk)

### 5.5.1 Cache location — cross-platform via `directories` crate (revisado Antigravity 3ª iteração L2F1 2026-05-29)

Path resolution via [`directories`](https://crates.io/crates/directories) crate (`ProjectDirs::cache_dir()`) — handles Windows `%LOCALAPPDATA%`, macOS `~/Library/Caches`, Linux `~/.cache`, Web (OPFS).

```
# Resolved at runtime per OS:
# Linux: ~/.cache/ph2d/shaders/
# macOS: ~/Library/Caches/com.ph2d.engine/shaders/
# Windows: %LOCALAPPDATA%\ph2d\engine\cache\shaders\
# Web: OPFS (origin-private file system) /ph2d/shaders/
└── <topology_hash_1>_<backend_id>.wgsl
└── <topology_hash_1>_<backend_id>.spv
└── <topology_hash_2>_<backend_id>.msl
```

**Windows MAX_PATH 260-char limit** (L2F1 Antigravity):
- Hashes blake3 são 64 hex chars; backend_id ~16 chars; filename + dirs ~110 chars total.
- `%LOCALAPPDATA%` típico ~50 chars; user profile path variável (worst case ~100 chars).
- Total worst case ~210 chars — **safe sob 260**, mas adversarial paths (long user names + nested dirs) podem aproximar.
- **Mitigation**: usar UNC paths `\\?\` prefix em writes via `std::fs::canonicalize` → `\\?\C:\Users\...\cache\...`. UNC bypasses 260-char limit (suporta 32767 chars).
- Crate `directories` já handles internally; PH2D consume sem custom logic.

### 5.5.2 Cache hit-rate target

>95% em scenario realístico (user editing graph topology rarely; mostly animating params).

### 5.5.3 Eviction

LRU 1 GB cap. Eviction quando exceeds.

### 5.5.4 Versioning

Cache key inclui PH2D version + Vello version + naga version. Major bump invalida cache (re-compile).

### 5.5.5 Cross-machine cache (future)

Stretch: cache compartilhado via S3 ou local team server. Não v1.0.

---

## 5.6 Diffusion curve solver (Walk-on-Spheres / multigrid)

### 5.6.1 Modelo matemático

**Diffusion curves** ([Orzan SIGGRAPH 2008](https://dl.acm.org/doi/10.1145/1360612.1360691)): curva carrega cor em ambos os lados (opcionalmente + blur radius); cores se difundem no resto do canvas via solving Poisson PDE.

```
∇²u = 0  (interior, exceto na curve)
u(p) = c_left(p)  (em p sobre curve, lado esquerdo)
u(p) = c_right(p)  (em p sobre curve, lado direito)
```

### 5.6.2 Algoritmos disponíveis

| Algoritmo | Type | Performance | Determinismo |
|-----------|------|-------------|--------------|
| Multigrid | Iterativo | Boa | Bom |
| Walk-on-Spheres (WoS) Monte Carlo | Stochastic | Excellent (paraleliza trivially) | Opt-in (fixed seed) |
| Boundary Element Method | Direct | Excellent precisão; complexo implementation | Bom |

**Recomendação v1.0**: Walk-on-Spheres Monte Carlo (paraleliza melhor em GPU compute).

### 5.6.3 WoS algorithm

```
For each pixel p in canvas:
    1. Find largest sphere centered at p that doesn't intersect any curve.
    2. Pick random point on sphere boundary.
    3. Move to that point; if hit curve → return curve color at hit point.
    4. Else repeat from 1.
    5. Average over N samples for variance reduction.
```

GPU compute shader: 1 invocation per pixel, N samples per invocation. Highly parallel.

### 5.6.4 Shader skeleton

`crates/ph2d-vector-fill/shaders/diffusion.wgsl`:

```wgsl
@group(0) @binding(0) var<storage> curve_buffer: array<DiffusionCurve>;
@group(0) @binding(1) var<storage, read_write> output_color: texture_2d<f32>;
@group(0) @binding(2) var<uniform> params: DiffusionParams;

const MAX_SAMPLES: u32 = 64u;
const MIN_RADIUS: f32 = 0.5;

@compute @workgroup_size(8, 8)
fn diffusion_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let p = vec2<f32>(f32(gid.x), f32(gid.y));
    var color_accum = vec4<f32>(0.0);
    
    for (var s = 0u; s < MAX_SAMPLES; s++) {
        var pos = p;
        var bounce = 0u;
        loop {
            let r = nearest_curve_distance(pos, curve_buffer);
            if (r < MIN_RADIUS) {
                // hit curve — sample color
                color_accum += sample_curve_color(pos, curve_buffer);
                break;
            }
            // Random point on sphere
            let theta = hash_to_f32(p, s, bounce) * 2.0 * 3.14159;
            pos = pos + vec2<f32>(cos(theta), sin(theta)) * r;
            bounce++;
            if (bounce > 1000u) { break; }  // safety
        }
    }
    
    output_color[gid.xy] = color_accum / f32(MAX_SAMPLES);
}
```

### 5.6.5 Performance — adaptive resolution + bilateral filter (revisado pós-Antigravity L4F1 2ª iteração 2026-05-28)

**Naïve approach inviable em mobile:** 1080p × 64 spp × ~10 bounces avg = ~1.28G random walks. Em Apple A14 ~5 TFLOPS = ~256ms — quebra HR-4 budget brutalmente em mobile GPUs.

**Pipeline canônico revisado**:

1. **Tier-aware resolution scaling**:
   - Desktop (Heavy tier): full 1080p × 64 spp → ≤ 5ms em M-series.
   - iPad / Standard tier: **0.5× resolution** (540p) × 32 spp + bilateral filter upscale → ≤ 4ms.
   - Mobile Core tier: **0.25× resolution** (270p) × 16 spp + bilateral filter upscale → ≤ 3ms em A14 mobile.
   - Web (WebGPU): match Standard tier; fallback CPU multigrid se compute slow.

2. **Joint Bilateral Upsampling (JBU) multi-pass** (revisado Antigravity 3ª iteração L1F4 2026-05-29): single-pass 15×15 ou 21×21 kernel @ 1080p estouraria 0.3 ms budget em mobile GPU (~256 samples per pixel × 2M pixels). Pipeline JBU correto em **2 passes**:
   - **Pass 1 (low-res denoise)**: bilateral 3×3 ou 5×5 kernel no buffer 270p (Mobile Core) / 540p (Standard) — elimina ruído stocástico WoS high-frequency. Custo ~0.1 ms.
   - **Pass 2 (joint guided upscale)**: upscale guiado pelas curvas analíticas em high-res target (1080p). Kernel pequeno 3×3 mas guidance signal = curve distance field; preserves sharp boundaries from analytical curves. Custo ~0.2 ms.
   - **Total ≤ 0.3 ms** alcançado cleanly com 2 passes em vez de 1 mega-kernel.

3. **Adaptive sample count**: high-variance pixels (próximo a curves) ganham mais samples; low-variance pixels (interior) reduzem. Variance threshold dinâmico per frame; reduce total budget ~40% sem quality loss.

4. **CPU fallback multigrid** (Mobile Core entry-level): se compute shader budget exceeded, multigrid CPU SIMD (4 levels: 540p → 270p → 135p → 67p → solve → upscale). Slower (~15 ms) mas no GPU stall risk.

5. Cache result by curve hash; só re-compute em edit upstream.

**Gate CI**: `vector_diffusion_curve_tier_budget` valida ≤ tier budget per device class.

### 5.6.6 Determinismo opt-in

`hash_to_f32` é deterministic. Fixed sample count + ordered reductions → bit-identical cross-OS.

### 5.6.7 Edge cases

- Curve com self-intersection: undefined behavior — UI valida e previne.
- Curve com endpoint isolado (não fecha): canvas borda funciona como boundary "neutra" (transparent).

---

## 5.7 Gradient mesh as Poisson PDE (Unified Smooth Vector Graphics 2024)

### 5.7.1 Insight

[arXiv 2408.09211 paper](https://arxiv.org/pdf/2408.09211) demonstra que mesh gradients e diffusion curves são duas formas da **mesma Poisson PDE**, parameterizadas diferentemente.

### 5.7.2 Unificação

Vector Module trata ambos como `MeshGradient { gradient_id }`, com gradient_id apontando para:
- **Diffusion curve**: curva + cores → Poisson PDE (caso §5.6).
- **Patches mesh** (Illustrator-style): grid + corner colors → mesma Poisson PDE com boundary diferentemente expressa.

Solver unificado (Walk-on-Spheres OR multigrid) consume ambos via abstração `BoundaryCondition`.

### 5.7.3 Auto-conversion

User authoring com curve + colors → solver direct. User authoring com mesh patches (Illustrator import) → conversion to equivalent diffusion curve representation. Lossy (mesh patch grid mais flexível que curve), mas accepted trade-off.

---

## 5.8 Pattern brush integration (Painter bridge)

### 5.8.1 Bridge spec

Vide [`08_painter_bridge.md §8.2`](08_painter_bridge.md). Resumo:

`vector-pattern-along-path` node consume any brush from `ph2d-painter-brush` library:

```rust
FillNode::Pattern { pattern_ref: PatternRef::PainterBrush(BrushId("pencil_2b")) }
```

Distribui stamps Painter ao longo do path com spacing / jitter / scatter params (vide [02 §2.2.8](02_geometry_graph.md)).

### 5.8.2 Resultado visual

Path traçado parece pintado com brush real (Painter `pencil_2b`, `oil_round`, etc.) mas é vetor editável (mover vertex → re-renderiza brush stamps automaticamente).

---

## 5.9 Animation hook (UBO update)

### 5.9.1 Param animável

Qualquer `FillNode` param é animável via curve no timeline (vide [`06_animation.md`](06_animation.md)).

```
Animate: Noise.frequency from 1.0 → 5.0 over 2 seconds (cubic-ease-in-out)
```

### 5.9.2 Per-frame update

```rust
fn update_fill_params(
    fill_graph: &FillGraph,
    time: f32,
    ubo: &mut FillParamsUbo,
) {
    for (node_id, node) in fill_graph.nodes.iter().enumerate() {
        if let FillNode::Noise { frequency, .. } = node {
            let animated_freq = sample_animation_curve("frequency", time);
            ubo.noise_freqs[node_id] = animated_freq;
        }
        // ... other animable params
    }
}
```

### 5.9.3 Off-thread topology change

Se animação altera **topology** (raro — geralmente só values), off-thread compile (vide §5.4.5).

---

## 5.10 Determinismo cross-platform

### 5.10.1 Shader bit-identity

Same WGSL compila para SPIR-V / MSL / HLSL. Runtime escolhe backend. Risco: output diferente cross-backend (rounding, FMA, ordering).

### 5.10.2 Det-mode opt-in

Quando `deterministic=true`:
- Shader includes `#pragma fma_off` (handled by naga).
- Ordered reductions (avoid `subgroupBallot` non-deterministic).
- Fixed thread group size.

### 5.10.3 Gate CI

`tests/determinism/procedural_fill_cross_os.rs`:
- Fixture: 5 standard fills (solid / linear / noise / voronoi / diffusion).
- Compile + render em Linux + Mac + Win.
- Output image hash blake3 bit-identical.

---

## Fim do procedural fill spec

Shader graph DAG completo com 17 nodes canon. Topology compile 1× + UBO update per frame (resolve crítica B Antigravity). Diffusion curve Poisson solver. Painter brush integration. Animation hook canônico.

**Next:** [`06_animation.md`](06_animation.md) (timeline + state machine) + [`08_painter_bridge.md`](08_painter_bridge.md) (Painter ↔ Vector bridges).
