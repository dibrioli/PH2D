# ADR-0044 — Brush Engine GPU contract (Brush + Stamp + Mixbox + Procedural Grain)

**Status:** Accepted (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, sessão Painter W0).
**Pré-requisitos:** [ADR-0040](0040-tool-as-isolated-feature-crate.md), [ADR-0041](0041-rasteredit-rename-and-deactivate.md), [ADR-0043 — Painter contract](0043-painter-contract.md).
**Spec normativa:** [`docs/Painter_projeto/01_brush_engine.md`](../../Painter_projeto/01_brush_engine.md) (§1.1..§1.12) + [`14_inovacoes_extraordinarias.md`](../../Painter_projeto/14_inovacoes_extraordinarias.md) (Propostas 2 e 3 absorvidas).
**Tags:** painter, wave-0, contract, brush-engine, gpu, mixbox, procedural-grain

---

## 1. Contexto

O motor de pincel é o coração do Painter — o que faz ter "sabor Procreate" ou não (§0 [HANDOFF_painter.md](../../HANDOFF_painter.md)). Antes de W1 escrever o stamp pipeline GPU, precisamos congelar:

1. **`Brush` struct** — ~140 parâmetros em 12 sub-painéis (§1.3.1..§1.3.12 de 01_brush_engine.md). Sem cap, vira god-struct mutável que cada wave atira mais campos dentro até o `.ph2d-brush` (HR-14) virar incompatível entre versões.
2. **`Stamp` struct** — unidade atômica enviada CPU→GPU. Hot path HR-3. Layout binário (96 bytes, `repr(C, align(16))`) é ABI; mudar quebra shader + golden tests + replay determinístico.
3. **Eixos ortogonais (Propostas 2 e 3 absorvidas):**
   - **Mixbox** — `pigment_mode: PigmentMode` axis novo ao `rendering_mode`. Sochorová+Jamriška SIGGRAPH 2021 ("Practical Pigment Mixing for Digital Painting"). Necessário pra "azul + amarelo → verde vibrante" (vs cinza lamacento do lerp linear). Existe open-source ([Scrtwpns/mixbox](https://github.com/scrtwpns/mixbox)).
   - **Procedural Grain** — `GrainSource::Procedural(ProceduralGrain)` axis novo. Geração matemática no compute, tiling zero, resolução infinita. Reduz atlas VRAM 64 MB → 32 MB (§1.6.8). Variants v1: Simplex / Gabor / PaperWeave / SprayDot.
4. **Caminho W1-unified → W5+-especializado.** §1.8.3 fechou que W1 ship unified (1 shader com `switch`); padrão-ouro é especializado (6 shaders, stamps pré-agrupados). Esta ADR ratifica o protocolo de transição.
5. **Gate `painter_stamp_specialize_when_budget_pressure`** mede headroom no sub-budget Painter ([08 §8.1.1](../../Painter_projeto/08_performance_memory.md)) e força entrada do ciclo de especialização quando headroom < 15%.

ADR-0043 §2.5 explicitamente cedeu esses cinco itens a esta ADR. Caps são numéricos (não "será definido depois") — DIRETRIZ §3.A.3.

---

## 2. Decisão

### 2.1 Crate `ph2d-painter-brush` (W1 T1.3)

```
crates/ph2d-painter-brush/
  Cargo.toml         # deps: serde, postcard, bytemuck, ph2d-tokens, ph2d-color, ph2d-gpu
  src/lib.rs         # #![forbid(unsafe_code)] PRIMEIRO
  src/brush.rs       # struct Brush + sub-structs por seção §1.3.x
  src/stamp.rs       # #[repr(C, align(16))] struct Stamp + tests Pod + size=96
  src/library.rs     # 12 brushes built-in (§1.6) como `pub const`
  src/atlas.rs       # ShapeAtlas + GrainAtlas (texture arrays)
  src/mixbox.rs      # impl Mixbox (paper Sochorová+Jamriška 2021)
  src/procedural.rs  # ProceduralGrain + 4 variants generators
  src/shader/        # WGSL: stamp.wgsl (W1 unified) + 6 specialized (W5+)
  tests/architecture_painter_contract_surface.rs  # caps de ADR-0043 + 0044
```

Dep `ph2d-gpu` (wgpu 28 wrapper) carrega o pipeline GPU; `bytemuck` garante Pod+Zeroable de `Stamp`. Sem dep direta de `ph2d-editor-core` — o crate `ph2d-tool-painter` (ADR-0043) consome `Brush`/`BrushHandle`/`StampPipeline` e cuida do binding com Tool/RasterEditTool.

### 2.2 `Brush` struct — modelo de cap duplo (top-level + sub-structs)

Baseline §1.3.1..§1.3.12: **~140 campos enumerados** na spec. Cap recursivo é **soma dos sub-caps + cap top-level**, NÃO um número único:

- **Top-level `Brush` struct ≤ 14 fields** (12 sub-structs + `version` + 1 slot de headroom para sub-struct nova futura).
- **Cada sub-struct tem seu próprio sub-cap** (tabela §2.2.1). Soma máxima recursiva = **180 fields** (com headroom; estimado-real v1 = ~131; ver §2.2.2).
- **NO sub-sub-structs.** Sub-structs são folha — só primitivos / enums / `Option<T>` / `Vec<T>` onde T é primitivo. Recursão proibida pelo gate `brush_no_sub_sub_structs` (§2.11). Razão: cap recursivo arbitrário não é auditável por grep textual.

#### 2.2.1 Tabela de sub-caps (frozen)

| Sub-struct | Spec § | v1 fields | Sub-cap | Headroom slots |
|---|---:|---:|---:|---:|
| `StrokePathParams` | §1.3.1 | 4 | **≤ 6** | 2 |
| `StabilizationParams` | §1.3.2 | 5 | **≤ 8** | 3 |
| `TaperParams` | §1.3.3 | 8 | **≤ 12** | 4 |
| `ShapeParams` | §1.3.4 | 15 | **≤ 20** | 5 |
| `GrainParams` | §1.3.5 | 14 | **≤ 20** | 6 |
| `RenderingParams` | §1.3.6 | **10** (9 v1 + `fluid_enabled` de ADR-0049) | **≤ 14** | 4 |
| `WetMixParams` | §1.3.7 | 9 | **≤ 12** | 3 |
| `ColorDynamicsParams` | §1.3.8 | ~28 (stamp+stroke+pressure+tilt+barrel) | **≤ 36** | 8 |
| `DynamicsParams` | §1.3.9 | 5 | **≤ 8** | 3 |
| `PencilParams` | §1.3.10 | 9 | **≤ 14** | 5 |
| `PropertiesParams` | §1.3.11 | 6 | **≤ 10** | 4 |
| `AboutParams` | §1.3.12 | 5 | **≤ 8** | 3 |
| **Totais** | | **~118** | **≤ 168** | **50 slots distribuídos** |

Plus top-level: `version: u32` (1) + 1 slot de headroom = **2 fields top-level fixos** + 12 sub-struct slots = **≤ 14 top-level**.

**Soma total possível: ≤ 168 sub-fields + 2 top-level = 170**. Gate textual audita cada cap individualmente (tabela acima) + top-level + sub-sub-struct ausência.

#### 2.2.2 Estrutura canônica

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Brush {
    pub stroke_path: StrokePathParams,        // §1.3.1
    pub stabilization: StabilizationParams,   // §1.3.2
    pub taper: TaperParams,                   // §1.3.3
    pub shape: ShapeParams,                   // §1.3.4
    pub grain: GrainParams,                   // §1.3.5
    pub rendering: RenderingParams,           // §1.3.6 (inclui pigment_mode + fluid_enabled ADR-0049)
    pub wet_mix: WetMixParams,                // §1.3.7
    pub color_dynamics: ColorDynamicsParams,  // §1.3.8
    pub dynamics: DynamicsParams,             // §1.3.9
    pub pencil: PencilParams,                 // §1.3.10
    pub properties: PropertiesParams,         // §1.3.11
    pub about: AboutParams,                   // §1.3.12
    pub version: u32,                         // HR-14 — v1 = 1
    // === 1 slot top-level (sub-struct nova futura) ===
}
```

**Não fundir sub-structs em flat Brush.** Razão: serialização postcard (HR-14) usa boundary de sub-struct pra versionamento parcial — uma seção pode evoluir sem invalidar `.ph2d-brush` antigos via `Option<NewField>` em sub-struct.

**Não permitir sub-sub-structs.** Razão: gate por grep textual fica auditável; pesos arquiteturais visíveis (deep nesting esconde complexidade). Se W5+ identificar necessidade real (e.g., `WetMixParams.advanced: WetMixAdvancedParams`), exige ADR-amend explícita.

### 2.3 `Stamp` struct — layout binário **96 bytes, `repr(C, align(16))`** (ABI freeze)

```rust
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Stamp {
    // ↓ ordem é ABI — não reordene sem amendment.
    pub position_world: [f32; 2],   //  0..8
    pub size_px: f32,               //  8..12
    pub rotation_rad: f32,          // 12..16
    pub pressure: f32,              // 16..20
    pub tilt: f32,                  // 20..24
    pub azimuth: f32,               // 24..28
    pub barrel_roll: f32,           // 28..32
    pub color_oklab: [f32; 4],      // 32..48
    pub opacity: f32,               // 48..52
    pub flow: f32,                  // 52..56
    pub wet_amount: f32,            // 56..60
    pub shape_layer: u32,           // 60..64
    pub grain_layer: u32,           // 64..68
    pub grain_offset_uv: [f32; 2],  // 68..76
    pub grain_scale: f32,           // 76..80
    pub flags: u32,                 // 80..84
    pub rendering_mode: u32,        // 84..88
    pub pigment_mode: u32,          // 88..92  (Linear=0, Mixbox=1)
    pub _pad: u32,                  // 92..96  alinhamento 16
}
const _: () = assert!(core::mem::size_of::<Stamp>() == 96);
const _: () = assert!(core::mem::align_of::<Stamp>() == 16);
```

**Caps ABI:**
- **`size_of::<Stamp>() == 96`** — verificado por `const _: () = assert!(…)` (compile-time) + golden test runtime.
- **`align_of::<Stamp>() == 16`** — idem.
- **`Stamp: bytemuck::Pod + bytemuck::Zeroable`** — derivado; falha de derive (e.g., adicionar field não-Pod) quebra build.

**Pool de stamps:** 4096 stamps × 96 B = 384 KB / frame (§1.4). Sem `Box::new` no hot path (HR-3). Ring buffer reusado.

**Diferença vs spec §1.4:** spec listou `pigment_mode` como campo "TBD" (Proposta 2 absorvida). Esta ADR aloca o slot agora (`u32` em offset 88) para que W1 unified shader não precise reordenar layout em W5+. Trade-off: 4 bytes "desperdiçados" em W1 (todos os brushes built-in default ainda como Linear até W5 ativar Mixbox per-brush §1.6.9). Aceito.

### 2.4 `RenderingMode` enum — cap **= 6 variants FROZEN** (sem headroom)

```rust
#[repr(u32)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RenderingMode {
    LightGlaze       = 0,
    UniformGlaze     = 1,
    IntenseGlaze     = 2,
    HeavyGlaze       = 3,
    UniformBlending  = 4,
    IntenseBlending  = 5,
    // === ZERO slots de headroom — frozen. ===
}

pub const MAX_RENDERING_MODES: usize = 6;
```

**Razão do freeze sem headroom:** o scheduler W5+ pre-grouped (§2.9) tem `stamps_by_mode: [Vec<Stamp>; MAX_RENDERING_MODES]` — **array statically-sized**. Headroom no enum sem bump correspondente no const cria out-of-bounds silencioso. Acoplamento intencional: bumpar `RenderingMode` exige amendment desta ADR + bump de `MAX_RENDERING_MODES` + rework do scheduler. Não é cap softável.

`#[repr(u32)]` é ABI — `Stamp::rendering_mode` é `u32` lido direto pelo shader (`case 0u..=5u:`). Variants ilegais > 5 são impossíveis pelo type system. Gate `rendering_mode_variant_count_is_exact_6` (não `is_capped`) confere.

### 2.5 `PigmentMode` enum — cap **≤ 4 variants** (v1 usa 2)

```rust
#[repr(u32)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PigmentMode {
    /// Lerp em OKLab (Photoshop default). Funciona em qualquer brush.
    /// **Único modo det-mode disponível.**
    Linear = 0,
    /// Mixbox — Kubelka-Munk simplificado, pigment mixing real-world.
    /// Per-brush default em paints/watercolors (§1.6.9).
    /// **NÃO funciona em --features det-painter** (vide §2.5.1).
    Mixbox = 1,
    // === 2 slots de headroom ===
}
```

**Mixbox refs:**
- **Paper:** Sochorová, Š., & Jamriška, O. (2021). *Practical Pigment Mixing for Digital Painting.* ACM Transactions on Graphics (SIGGRAPH Asia) 40(6).
- **Open-source impl:** [github.com/scrtwpns/mixbox](https://github.com/scrtwpns/mixbox) — CC0/MIT dual-licensed; ~700 LOC C. Port para WGSL é trabalho W1 T1.X (escopo ADR-0044).
- **API expectation:** funções `mixbox_lerp_srgb8(a, b, t) -> rgb8` e `mixbox_lerp_linear(a, b, t) -> rgb_linear`. Trabalho equivalente: ~3KB LUT + 12 SH coefficients + 7 mat ops por sample.

#### 2.5.1 Mixbox **excluído do det-mode** (decisão explícita)

Em `--features det-painter` (replay determinístico cross-OS — vide [ADR-0046 §2.8](0046-stroke-vector-history.md)):

```rust
#[cfg(feature = "det-painter")]
fn resolve_pigment_mode(brush: &Brush) -> PigmentMode {
    // Mixbox usa LUT 3D float + 12 SH coeficientes + 7 mat ops f32.
    // f32 SIMD em ARM vs x86 diverge (FMA fusion + libm sin/cos + denormals);
    // Q-fixed-point port do Mixbox é trabalho não-trivial (estimado ~2 sessões;
    // não escopo W1; registrado como follow-up §6).
    // Comportamento det-mode: força Linear independente do brush default.
    PigmentMode::Linear
}
```

**Consequência:** replay det-mode de stroke `oil_round` (default `Mixbox`) produz cor **diferente** (mais "Photoshop-like") do GPU mode. Aceito como limitação documentada — det-mode é CI / cross-OS verification, não production paint.

**Follow-up registrado:** porting Mixbox para Q-fixed-point (Q16.16 nos KS coefs, Q24.8 nas mat ops) habilita Mixbox em det-mode. Estimativa: 2 sessões dedicadas. Não bloqueia W0/W1; entra em follow-up plan quando alguma feature exigir.

### 2.6 `GrainSource` enum — cap **≤ 6 variants** (v1 usa 4)

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrainSource {
    None,
    /// Bitmap escaneado/desenhado, atlas-resident.
    Bitmap { atlas_layer: u32, blake3: [u8; 32] },
    /// Procedural — generated in compute shader (§2.7).
    Procedural(ProceduralGrain),
    /// User-imported PNG → atlas-resident.
    Imported { atlas_layer: u32, blake3: [u8; 32] },
    // === 2 slots de headroom ===
}
```

`blake3` é content hash para de-duplicar atlas + cache de pré-processo.

### 2.7 `ProceduralGrain` enum — cap **≤ 8 variants** (v1 usa 4)

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProceduralGrain {
    /// Simplex Noise (Perlin moderno). Para noise-like genérico.
    /// Ref: Perlin, Ken (2002) "Improving Noise."
    SimplexNoise   { scale: f32, octaves: u32, persistence: f32, seed: u64 },
    /// Gabor Noise. Para textura anisotrópica (fibras orientadas).
    /// Ref: Lagae, Lefebvre, Drettakis, Dutré (2009) "Procedural Noise using Sparse Gabor Convolution."
    GaborNoise     { frequency: f32, orientation: f32, anisotropy: f32, seed: u64 },
    /// Paper Weave — trama de papel/tela.
    /// Composição própria PH2D (sem ref externa direta — cross-hatched Gabor + threshold).
    PaperWeave     { fiber_density: f32, fiber_anisotropy: f32, crossweave: bool, seed: u64 },
    /// Spray Dot — Poisson-disk pseudo-aleatório para spray.
    /// Composição própria PH2D (Poisson-disk via Bridson 2007 § Fast Poisson Disk Sampling).
    SprayDot       { dot_density: f32, dot_size: f32, dot_jitter: f32, seed: u64 },
    // === 4 slots de headroom (e.g., Worley, ReactionDiffusion, …) ===
}
```

**Determinismo:** todos os 4 variants são determinísticos se `seed` é estável. HR-5 mantido. Validação via golden tests cross-OS (W11 det-mode).

**Custo GPU por sample:**
| Variant | Ops estimadas | Dentro budget? |
|---|---|---|
| SimplexNoise | ~50 | Sim (3.5 ms / frame) |
| GaborNoise | ~80 | Sim |
| PaperWeave | ~150 | Sim, headroom modesto |
| SprayDot | ~30 | Sim |

Medir cross-platform baseline (Apple M2, RDNA1, Intel UHD) em W5 T-procedural-bench.

### 2.8 `BrushHandle` (opaco; consumido por ADR-0043, 0046, 0047, 0048)

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BrushHandle(u32);

impl BrushHandle {
    /// Bit 31 = 1 → imported (atlas layer livre, dynamic blake3-keyed).
    /// Bit 31 = 0 → built-in (Library fixed slot, índice 0..63 reservado).
    pub const IMPORTED_FLAG: u32 = 1 << 31;
    pub fn new_builtin(slot: u32) -> Self { assert!(slot < 64); Self(slot) }
    pub fn new_imported(atlas_layer: u32) -> Self { Self(atlas_layer | Self::IMPORTED_FLAG) }
    pub fn is_imported(&self) -> bool { self.0 & Self::IMPORTED_FLAG != 0 }
    pub fn slot(&self) -> u32 { self.0 & !Self::IMPORTED_FLAG }
}
```

ADR-0043 §2.3 alocou `BrushHandle` em `PainterParams.active_brush`. ADR-0046 usa em `StrokeRecord.brush_handle`. ADR-0047 usa em `StrokeRef.brush_handle` + `StrokeMods.new_brush`. Esta ADR é a **única fonte canônica** do tipo + bit layout.

**Gate `brush_handle_bit31_layout`** (§2.11) verifica que `new_imported(0).is_imported() == true` + `new_builtin(0).is_imported() == false` + slot extraction round-trip.

### 2.9 Shader transition W1-unified → W5+-especializado

**W1 (rampa):** 1 shader `stamp.wgsl` com `switch (stamp.rendering_mode)` (§1.8.3 W1). Branching uniforme; ~5-10% perf hit aceitável.

**W5+ (padrão ouro):** 6 shaders especializados, `StampScheduler` pre-agrupa stamps por modo (`stamps_by_mode: [Vec<Stamp>; MAX_RENDERING_MODES]` — `MAX_RENDERING_MODES = 6` const-driven; §2.4). Vetores reusados frame-a-frame; HR-3 mantido. `StampPipeline::encode()` itera modos não-vazios e despacha pipeline correspondente. Coupling explícito ao const garante que bump do enum força bump do array (sem out-of-bounds silencioso).

**Protocolo de transição (não-destrutivo):**

1. API pública `StampPipeline` inalterada entre W1 e W5+.
2. Test golden por modo (W1) continua bit-identical em W5+ (mesmas fórmulas; só layout do shader muda).
3. Pre-grouping é otimização interna; transparente para `PainterTool`.
4. Amendment desta ADR (`0044-amendment-1.md`) **só** se W5 descobrir que pre-grouping força mudança de surface (e.g., novo `StampMode` no scheduler) — improvável.

### 2.10 Gate `painter_stamp_specialize_when_budget_pressure`

**Localização:** `crates/ph2d-painter-brush/tests/perf_budget_pressure.rs` (gate runtime, executado em CI baseline runners — Apple M2 / RDNA1 / Intel UHD).

**Spec:**
```rust
// Roda stamp pipeline com 1024 stamps, mede tempo CPU + GPU.
// Compara contra sub-budget Painter (08 §8.1.1: 4.5 ms / frame).
// Headroom = 1.0 - (measured_ms / budget_ms).
//
// Falha se headroom < 0.15 em 3 runs consecutivos no mesmo runner.
// Falha = sinal pro ciclo de especialização entrar no roadmap; NÃO bloqueia merges
// imediatamente (warning escala via ph2d-loc-trend memo).
```

**Importante:** este gate é **soft** em W1-W4 (warning logged, exit 0); transita pra **hard** (exit 1) em W5+ quando o scheduler especializado já existe e a expectativa é headroom > 15% sustentável.

### 2.11 `painter_contract_surface` arch-gate (cumulativo ADR-0043 + 0044)

**Localização canônica congelada:** `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`. O crate `ph2d-painter-contracts` é criado em **T0.8 da cascata W0** (não diferido pra W1 — vide ADR-0043 §2.4 amended) e é o homestead único dos gates desta cascata. Dep de tipos resolvida lazily (audita texto das ADRs e código quando os crates filhos existem em W1+).

```rust
mod painter_ui {
    // De ADR-0043
    #[test] fn painter_ui_edit_variant_count_is_capped()    { /* ≤ 24 */ }
    #[test] fn painter_ui_snapshot_field_count_is_capped()  { /* ≤ 18 */ }
    #[test] fn painter_params_field_count_is_capped()       { /* ≤ 12 */ }
}

mod brush_engine {
    // De ADR-0044
    #[test] fn brush_top_level_field_count_is_capped()      { /* ≤ 14 (12 sub + version + 1 headroom) */ }
    #[test] fn brush_sub_caps_per_substruct()               { /* tabela §2.2.1 — 12 caps individuais */ }
    #[test] fn brush_no_sub_sub_structs()                   { /* recursão proibida — sub-structs são folha */ }
    #[test] fn brush_total_recursive_field_count_is_capped() { /* soma sub-caps ≤ 168 + top-level ≤ 14 */ }
    #[test] fn stamp_size_is_96_bytes_aligned_16()          { /* compile-time + runtime */ }
    #[test] fn stamp_is_pod_zeroable()                      { /* bytemuck::Pod */ }
    #[test] fn rendering_mode_variant_count_is_exact_6()    { /* = 6 FROZEN, não ≤ */ }
    #[test] fn max_rendering_modes_matches_enum()           { /* const MAX_RENDERING_MODES == 6 */ }
    #[test] fn pigment_mode_variant_count_is_capped()       { /* ≤ 4 */ }
    #[test] fn grain_source_variant_count_is_capped()       { /* ≤ 6 */ }
    #[test] fn procedural_grain_variant_count_is_capped()   { /* ≤ 8 */ }
    #[test] fn brush_handle_bit31_layout()                  { /* §2.8 — imported/builtin flag + slot round-trip */ }
}
```

---

## 3. Consequências

### Positivas

- **Brush é fixed-shape pra v1.** ~140 campos + 14% folga = cap 160 deixa W5 ampliar wet mix / color dynamics sem amend desta ADR. Quando bater 160, ADR-amend explícita.
- **Stamp é ABI freeze.** 96 bytes é o número que vai pra shader, replay determinístico (W11+), `.ph2d-painter` (HR-14 versionado). Mudar = ADR-amend + recook golden tests + bump version.
- **Mixbox + Procedural Grain são axis ortogonais, não bolt-ons.** Cada axis é enum cap-ado; brushes built-in (§1.6.9) já são data-driven na escolha; usuária custom override em Brush Studio.
- **Shader transition W1→W5+ é não-destrutiva.** API estável; só layout do shader muda. Risco de "refactor escapa do escopo" minimizado.
- **Gate de specialize pressure é early-warning.** Soft em W1-W4, hard em W5+. Encadeia naturalmente o trabalho de otimização sem forçar especialização prematura (HR-3 zero-alloc é o blocker, não branching).

### Negativas / Custos

- **Mixbox precisa LUT 3KB + 12 SH coefs em VRAM.** Custo memória ínfimo (~3 KB const buffer); custo perf ~7 mat ops por sample (~0.05 ms em 1024 stamps). Aceito.
- **Procedural Grain `PaperWeave` no limite do budget.** ~150 ops/sample; cabe em desktop M2/RDNA1; medir em Intel UHD / mobile baseline em W5. Se quebrar, mitigação: fallback bitmap (já temos `canvas_weave`).
- **Slot `_pad: u32` em Stamp.** 4 bytes "desperdiçados" por stamp em W1 (4096 × 4 = 16 KB / frame). Trade-off: alinhamento 16 + estabilidade ABI. Aceito vs reordenação em W5+.
- **`Stamp::pigment_mode = 1u32` (Mixbox) ativa branching em W1 unified.** GPU lida bem (uniforme dentro de workgroup), mas custo +~5% no shader. Mitigação: brushes default Linear (§1.6.9 — 8/12 brushes); só paints+watercolors disparam Mixbox em W1.
- **Cap recursivo `Brush ≤ 168 sub-fields + 14 top-level` pode apertar em W5.** Se Brush Studio drill-down expor 25+ campos novos em wet mix (improvável; spec lista 9 + cap 12 = headroom 3), ADR-amend sobe sub-cap específico. Aceito como risco baixo.

### Neutras

- **`ph2d-painter-brush` cresce grande.** Estimativa LOC v1.0: ~3500 (Brush + Stamp + Library 12 brushes + Atlas + Mixbox port + Procedural × 4 + WGSL 1 unified shader). Acima do gate panel-loc-cap (600 LOC/file), mas o gate aplica em `crates/ph2d-panel-*/src/**` — `ph2d-painter-brush` não é panel; sem restrição. Tracker de god-file: `ph2d-loc-trend` watch-list em W2.

---

## 4. Alternativas consideradas

### 4.1 Brush flat (não dividir em sub-structs)

**Rejeitada.** ~140 campos flat = legibilidade negativa + serialização não-evolutiva (HR-14 versionado precisa de boundary de sub-struct para `Option<NewField>` evoluir sem invalidar `.ph2d-brush` antigos).

### 4.2 Stamp menor (e.g., 64 bytes)

**Rejeitada.** Procreate Valkyrie usa stamp ~80 bytes; PH2D adiciona `pigment_mode` (Mixbox axis) + 2 slots de headroom. 96 bytes alinhado 16 é o sweet spot — cabe em VRAM (384 KB / frame ring buffer), permite alinhamento natural pra GPU load, dá ABI headroom.

### 4.3 Mixbox como sub-shader (em vez de axis no Stamp)

**Rejeitada.** Sub-shader exigiria pre-grouping no scheduler (sem split por axis pigment_mode = duplicate dispatch). Axis no Stamp permite W1 unified com 1 shader; W5+ pre-grouping faz por `(rendering_mode, pigment_mode)` tupla (12 pipelines em vez de 6, ainda OK).

### 4.4 Procedural Grain como atlas dinâmico (não compute-time)

**Rejeitada.** Atlas dinâmico exige pre-render por mudança de seed/scale; tiling visível em scale baixo; perde o ganho de "resolução infinita". Compute-time gera o sample direto no workgroup do stamp — sem atlas overhead.

### 4.5 Adiar Mixbox / Procedural pra W5+

**Rejeitada.** Adiar o **slot ABI** (`pigment_mode` no Stamp) força reorganização do layout em W5 e invalida golden tests do W1. Slot reservado em W1 com `default = Linear` evita o re-layout. Implementação real do Mixbox compute pode adiar pra W5; o **slot** entra agora.

### 4.6 Especializar shader já em W1

**Rejeitada.** Pre-grouping no scheduler + 6 pipelines em W1 dobra superfície de teste antes de termos primeira pintura (smoke W1 day 7). Unified em W1 entrega pintura mais rápido; especialização vira otimização honesta com headroom medido.

---

## 5. Verificação

Após esta ADR ratificada (T0.9) e W1 T1.3+T1.4 implementarem:

```sh
cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# 10 sub-tests: caps ADR-0043 (3) + ADR-0044 (7). Todos verdes.

cargo test -p ph2d-painter-brush
# Stamp Pod/Zeroable + size_of==96 + align_of==16 + Mixbox round-trip vs reference + Procedural deterministic.

cargo test -p ph2d-painter-brush --features det-painter
# Determinismo opt-in: CPU fallback bit-identical cross-OS para procedural grain.

cargo test -p ph2d-painter-brush --test perf_budget_pressure -- --ignored
# Soft em W1-W4 (warning); hard em W5+.
```

### 5.1 Definição de "Accepted"

Esta ADR transita `Proposed → Accepted` no mesmo evento T0.9 que ADR-0043 (cascata W0 ratifica em bloco). Pré-condição extra desta ADR: ADRs irmãs 0043, 0045..0049 também escritas (texto coerente).

---

## 6. Tracking

- Plano operacional: [`docs/Painter_projeto/15_plano_de_implementacao.md`](../../Painter_projeto/15_plano_de_implementacao.md) §3 (T0.2) + §4 (W1 T1.3..T1.6 implementam).
- Spec normativa: [`01_brush_engine.md`](../../Painter_projeto/01_brush_engine.md) §1.1..§1.8.
- Decisão estrutural de transição W1→W5+: [README §11](../../Painter_projeto/README.md) #2.
- Mixbox impl ref: [github.com/scrtwpns/mixbox](https://github.com/scrtwpns/mixbox).
- Próxima ADR na cascata W0: [ADR-0045 — Adjustment Layers contract](0045-adjustment-layers.md) (T0.3).
