# ADR-0049 — Fluid Brushes Extension (`ph2d-painter-fluid`, opt-in W15)

**Status:** Accepted (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, sessão Painter W0).
**Pré-requisitos:** [ADR-0043 — Painter contract](0043-painter-contract.md), [ADR-0044 — Brush Engine GPU](0044-brush-engine-gpu.md).
**Spec normativa:** [`docs/Painter_projeto/14_inovacoes_extraordinarias.md`](../../Painter_projeto/14_inovacoes_extraordinarias.md) §14.6 (Proposta 4) + [`08_performance_memory.md`](../../Painter_projeto/08_performance_memory.md) §8.2.2.
**Tags:** painter, wave-0, contract, fluid-sim, shallow-water, gyroscope, opt-in, graceful-degrade

---

## 1. Contexto

Procreate / Photoshop / Fresco fazem aproximação visual de aquarela (smudge + blur + wet edges fake). Painter PH2D ambiciona **simulação física real**: tinta sangra pelas fibras do papel; wet edges são acumulação física de pigmento; tilt do device faz tinta escorrer fisicamente via giroscópio.

Diferenciação técnica genuína vs Procreate ([§14.8.1 spec](../../Painter_projeto/14_inovacoes_extraordinarias.md)). Mas:

1. **Risco técnico mais alto das 5 Propostas.** Shallow Water solver em GPU em devices heterogêneos (Apple silicon, RDNA1, Intel UHD, Web WGPU, mobile Adreno/Mali) é desafio empírico.
2. **Custo perf significativo.** 2.5-3.5 ms por frame **apenas quando ativo**. Budget total fica 6.7 ms — cabe 120Hz, aperta 60Hz.
3. **Budget memória adicional.** 20-30 MB VRAM apenas quando fluid ativo.
4. **Cross-platform sensor heterogeneity.** iPad / iPhone / Android têm giroscópios; desktop não tem; Web tem (via DeviceMotion API) mas com permissões.
5. **Det-mode trade-off.** Fluid sim GPU é não-determinístico; replay HR-5 exige CPU fallback ~10× mais lento.

Sem contrato congelado:

- **API `ph2d-painter-fluid` cresce orgânica em W15.** Wave dedicada gera "tudo que pareceu razoável".
- **Graceful degrade vira por-feature.** "Brush check fluid_capable" vs "platform check has_gyro" reimplementam.
- **`PlatformHost::gyroscope()` extension não-trivial.** Tocar `ph2d-host` mexe trait foundational que existe em todos os shells.

ADR-0043 §2.5 cedeu este território a esta ADR. Ela é **última na cascata W0** porque tem **maior risco técnico**, mas **NÃO é deferível** — regra de produto "perfeição desde início, sem adiamentos" (2026-05-26) obriga ship com fluid em W15 ou ADR de retração explícita (não silenciosa "wave estourou prazo").

---

## 2. Decisão

### 2.1 Crate `ph2d-painter-fluid` (W15)

```
crates/ph2d-painter-fluid/
  Cargo.toml         # deps: ph2d-gpu, ph2d-painter-brush (Brush types), ph2d-color, ph2d-host
                     # NÃO dep ph2d-tool-painter (decoupled — tool é um cliente entre outros)
  src/lib.rs         # #![forbid(unsafe_code)] PRIMEIRO; #[cfg(feature = "fluid")] gate todo o crate
  src/solver.rs      # Shallow Water solver (GPU compute + CPU det fallback)
  src/sim.rs         # FluidSim struct (texture handles + parameters + step driver)
  src/params.rs      # FluidParams (per-brush + per-canvas tuning)
  src/gravity.rs     # GravitySource enum + integração com PlatformHost
  src/budget.rs      # fluid_capable() check + headroom measurement
  tests/             # caps + det-mode bit-identical CPU + perf bench
```

**Feature flag canon:** `--features fluid`. Sem essa feature, crate compila como `pub mod stub` com no-op API. Mobile/Web/CI buildings que não querem fluid excluem deps:

```toml
# Cargo.toml de ph2d-tool-painter (consumer):
[features]
default = []
fluid = ["ph2d-painter-fluid/fluid"]
```

### 2.2 `PlatformHost::gyroscope()` — extension

ADR-0049 amenda a trait `ph2d-host::PlatformHost` adicionando **um slot** novo:

```rust
pub trait PlatformHost {
    // ... métodos existentes ...

    /// Returns the current gravity vector (m/s²) sensed by the device's
    /// gyroscope/accelerometer fusion, or `None` if the platform has no
    /// such sensor or the user denied permission.
    ///
    /// Convention: +Y = device-down (natural portrait gravity);
    /// magnitude ≈ 9.81 at rest. Caller (fluid sim) normalizes / clamps.
    ///
    /// Default impl returns `None` (desktop / headless / CI fallback).
    fn gyroscope(&self) -> Option<[f32; 3]> { None }
}
```

**Cap impact:** trait `PlatformHost` ganha **1 método** (default `None`). Pré-existing impls em shells (desktop, web, iOS, Android) não quebram — herdam o default. Shells que **querem** sensor implementam override.

**Coord-A only:** mexer em `ph2d-host` é fora-do-fan-out drop-crate (DIRETRIZ §4 contratos congelados). Esta ADR sanciona a extensão; W15 T-1 (criação do crate fluid) é o ponto onde o método entra em main.

### 2.3 `Brush.fluid_enabled` — opt-in per-brush

Adicionado ao `Brush` struct (ADR-0044) como campo NOVO em sub-struct `RenderingParams` (§1.3.6). **Bumpa o sub-cap específico** de `RenderingParams` em ADR-0044 §2.2.1 de **9 → 10** fields. Cap atualizado nesta cascata simultânea — ADR-0044 §2.2.1 tabela já reflete o 10 (com cap ≤ 14, 4 slots restantes).

Sem essa nota explícita o gate `brush_sub_caps_per_substruct` quebra silenciosamente quando ADR-0049 implementação ship. Audit C2 (2026-05-26) flagged.

```rust
// crates/ph2d-painter-brush/src/brush.rs
pub struct RenderingParams {
    // ... campos existentes §1.3.6 ...
    pub fluid_enabled: bool,                 // default false
    // ...
}
```

Brushes default que ativam fluid (W15):
- `watercolor_wash`, `watercolor_detail` → `fluid_enabled = true`
- `oil_round`, `oil_bristle` → `fluid_enabled = true`
- Todo resto → `fluid_enabled = false`

Usuária toggle em Brush Studio (W5 panel-painter-brush-studio expõe um checkbox quando o crate `ph2d-painter-fluid` está disponível no build; ausente o crate, checkbox não aparece).

### 2.4 `FluidSim` API — cap **≤ 12 fields**

```rust
#[derive(Debug)]
pub struct FluidSim {
    pub canvas_id: CanvasId,
    pub fluid_size: (u32, u32),              // 1/4 canvas res (texture handles abaixo)
    pub pigment_density_tex: TextureHandle,
    pub velocity_tex: TextureHandle,
    pub pressure_tex: TextureHandle,
    pub viscosity_tex: TextureHandle,
    pub gravity_source: GravitySource,
    pub params: FluidParams,
    pub steps_per_frame: u32,                // default 1; >1 = sub-step para estabilidade
    pub active: bool,                        // bool flag drained per-frame pelo solver
    pub version: u32,                        // HR-14 — v1 = 1
    // === 1 slot de headroom ===
}
```

### 2.5 `FluidParams` — cap **≤ 12 fields**

Parâmetros tuning do solver Shallow Water (per-brush ou per-canvas override):

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FluidParams {
    pub viscosity: f32,                      // [0, 1]; 0 = água, 1 = óleo espesso
    pub diffusion_rate: f32,                 // [0, 1]; capilaridade do papel
    pub gravity_strength: f32,               // [0, 2]; multiplica vetor gyro (1.0 = real-world)
    pub wet_edge_accumulation: f32,          // [0, 1]; quanto pigmento acumula nas bordas
    pub fiber_anisotropy: f32,               // [0, 1]; alinhamento de fibras (papel texturado)
    pub paper_absorbency: f32,               // [0, 1]; quanto papel "bebe" pigmento
    pub solver_iterations: u32,              // 1..=8 Jacobi iterations per step
    pub damping: f32,                        // [0, 1]; estabilidade numérica
    pub version: u32,                        // HR-14 v1 = 1
    // === 3 slots de headroom ===
}
```

### 2.6 `GravitySource` enum — cap **≤ 6 variants** (v1 usa 3)

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GravitySource {
    /// Sem gravidade — fluid se espalha igualmente em todas direções.
    None,
    /// Vetor fixo configurado em Painter Preferences. `(dir_x, dir_y)` normalizado, `magnitude`.
    Fixed { dir_x: f32, dir_y: f32, magnitude: f32 },
    /// Vetor lido do `PlatformHost::gyroscope()` (atualizado per-frame).
    /// Se host retorna `None`, solver tratará como `None` variant.
    DeviceGyroscope,
    // === 3 slots de headroom ===
}
```

### 2.7 Shallow Water solver — algoritmo (referência)

**Modelo:** Shallow Water Equations 2D ([Stam 1999](https://www.researchgate.net/publication/2486892_Stable_Fluids); [Crane 2007](https://www.cs.cmu.edu/~kmcrane/Projects/GPUFluid/paper.pdf) GPU adaptation). Variáveis em texturas R8/RG16F:

```
∂ρ/∂t + ∇·(ρv) = -k·ρ              (densidade pigmento — decay)
∂v/∂t + (v·∇)v = -∇p/ρ + g + ν∇²v  (velocidade)
∇·v = 0                              (incompressibilidade — projection step)
```

**Pipeline GPU compute (per frame, per step):**

```
1. Stamp pipeline (brush) deposita ρ em pigment_density_tex.
2. Advect v ← advection(v, dt).
3. Apply external forces: g (gravity) + brush velocity (drag).
4. Diffuse v ← diffusion(v, ν, iterations).
5. Compute pressure p via Jacobi iterations (8 default).
6. Project v ← v − ∇p (divergence-free).
7. Advect ρ ← advection(ρ, dt).
8. Composite ρ → layer texture (premultiplied alpha, OklchColor pigment).
```

**Texturas:** 1/4 canvas res (4K → 1024², ~16 MB total para 4 RG16F + 1 R8). Cabe em budget §2.10.

### 2.8 Graceful degrade — protocolo congelado

```rust
pub fn fluid_pass_eligible(
    memory_budget: &MemoryBudget,
    perf_budget: &PerfBudget,
    brush: &Brush,
) -> bool {
    brush.rendering.fluid_enabled
        && memory_budget.fluid_capable()             // see §2.9
        && perf_budget.fluid_headroom_ms() > 3.0     // see §2.10
}
```

**Quando `false`:** fallback para `Wet Mix` tradicional ([§1.3.7 da spec](../../Painter_projeto/01_brush_engine.md)) — usuária ainda vê comportamento "molhado", só não tem o "wow" da sim física. **Identidade do brush mantida** (UX honesto).

### 2.9 `MemoryBudget::fluid_capable()`

Vive em `ph2d-host::MemoryBudget`. Esta ADR fixa contrato:

```rust
impl MemoryBudget {
    /// Returns true if the platform has ≥ 32 MB free VRAM after baseline
    /// allocations (atlas + history + snapshots). Crisp/Heavy text rendering
    /// + Adjustment cache + Painter baseline must all fit before this returns true.
    pub fn fluid_capable(&self) -> bool {
        self.vram_free_mb >= 32 && self.tier >= MemoryTier::Mid
    }
}
```

`MemoryTier { Low, Mid, High }` é classificador device-side existente (ou criado em W15 T-1 junto com esta ADR).

### 2.10 `PerfBudget::fluid_headroom_ms()`

Vive em `ph2d-host::PerfBudget`. Esta ADR fixa contrato:

```rust
impl PerfBudget {
    /// Headroom no sub-budget Painter (08 §8.1.1: 4.5 ms / frame).
    /// Mede last_frame_painter_time_ms; retorna budget - measured.
    /// > 3.0 ms = espaço pro fluid pass (2.5-3.5 ms).
    pub fn fluid_headroom_ms(&self) -> f32 {
        (self.painter_budget_ms - self.last_frame_painter_ms).max(0.0)
    }
}
```

Cap: `PerfBudget` adiciona 1 método; `MemoryBudget` adiciona 1 método. **Sem amend de trait** (são impls inerentes em structs, não em traits).

### 2.11 Det-mode CPU fallback — perf realista + res 256² (audit A-7)

Quando `--features det-painter`:

```rust
#[cfg(feature = "det-painter")]
pub fn fluid_step_det(sim: &mut FluidSim, brush: &Brush, dt_q1616: i32) {
    // Tudo CPU. Sem GPU compute. Sem fast-math. Sem FMA.
    // Implementação reference: Stam 1999 Stable Fluids em f64 + compiler_fence.
    // RESOLUÇÃO REDUZIDA: det-mode usa fluid_size = 256² (não 1024² do GPU mode);
    // ~16× menos pixels, ~16× mais rápido vs det-mode @ full res.
    // Trade-off aceito: det-mode é CI / cross-OS verification, NÃO production paint.
}
```

**Perf realista (audit 2026-05-26 corrigiu otimismo prévio):**

| Modo | Res fluid | Tempo / step | Strokes/s det @ 4K canvas |
|---|---|---|---|
| GPU (production) | 1024² | ~0.25 ms | ~50 |
| CPU det @ 256² (corrigido) | 256² | **~10 ms** | **~5** |
| ~~CPU det @ 1024² (claim original "10× slower")~~ | ~~1024²~~ | ~~~150 ms~~ | ~~~0.3~~ |

**Decisão:** det-mode CPU sempre roda em 256² (não 1024²). CI replay test com 100 frames = ~1 segundo × N OSes — viável. Production paint usa GPU 1024² — não-determinístico mas estética OK (HR-5 não exige determinismo em PresentWorld).

**Re-medição obrigatória em W15 T-2 prototype:** Stam 1999 CPU 256² em Apple M2 single-threaded. Se medida real exceder 50 ms / step, reduzir res para 128².

**Gate:** `fluid_det_replay_cross_os` — replay de stroke molhado @ 256² em Linux/macOS/Windows/Web (WASM) devolve `blake3` idêntico. Soft em W15; hard em W16+.

### 2.12 Device matrix coverage — gate hard de ship (sem deferral)

Por regra "perfeição desde início, sem adiamentos" (2026-05-26): fluid sim **ship em v1.0** ou ADR de retração explícita com aprovação Enio. Crate `ph2d-painter-fluid` não é "deletável silenciosamente" — `--features fluid` é canon, e a probabilidade de cobrir mid-tier ≥ 80% é o blocker concreto.

#### 2.12.1 Device matrix obrigatória (W15 T-2 prototype)

Antes de W15 T-3+ prosseguir, fluid sim deve passar **device matrix test** com cobertura mínima:

| Device tier | Devices alvo | Cobertura mínima |
|---|---|---:|
| **High** (must-pass 100%) | Apple M1+/M2/M3, RDNA1+ desktop, Apple Pencil 2+ iPad Pro M1+ | **100%** |
| **Mid** (must-pass ≥80%) | Apple A14+ iPad Air, Adreno 650+, Mali G77+, Intel Iris Xe, RDNA1 mobile | **≥80%** |
| **Low** (graceful — sem hard requirement) | Adreno 6xx older, Mali G7x older, Intel UHD, iPad 2018, Web WGPU baseline | **≥30%** com graceful degrade explícito |

**Pass criteria por device:**
1. Crash-free em 1000 frames de stroke contínuo com `fluid_enabled` brush.
2. Frame budget ≤ 3.5 ms p99 para fluid pass.
3. Memory delta ≤ 35 MB (cap +5MB sobre §2.10 budget).
4. Driver compute compaction não trava (Adreno 650 known issue).
5. Jacobi iterations 1..=8 stable sem timeout (Mali G77 known issue).

#### 2.12.2 Abort criteria + decisão de produto

Se device matrix em W15 T-2 retornar:

- **Mid coverage < 80%:** STOP W15. Coord-A escreve ADR-0049-amendment-1 com decisão explícita:
  - (a) **Retract fluid de v1.0:** flag default-off em produção; ADR-0053 (Cross-platform tier policy) registra como "Painter Pro feature, post-1.0 GA".
  - (b) **Reduzir escopo fluid:** apenas Apple/RDNA1 desktop; remover Android/Intel/Web do scope; ADR-0053 documenta gap.
  - Decisão precisa **aprovação Enio explícita**. Sem aprovação, default = (a) retract.
- **Mid coverage ≥ 80%:** prossegue W15 T-3+. Devices que falham em runtime caem para wet_mix matemático (graceful degrade §2.8) com **telemetry de incidência** (não só `fluid_capable() = false`).

#### 2.12.3 Telemetry de produção (W15 T-5 obrigatório)

ADR-0049 spec exige (não opcional): cada fluid_pass que aborta em runtime via crash-detector / timeout / driver-error registra evento no audit log. Após 30 dias de produção, % de fluid_capable() devices que efetivamente ativaram fluid sim ≥ 90% (calibração via real telemetry, não probing CPU at boot).

### 2.13 Caps numéricos

| Tipo | Cap |
|---|---|
| `FluidSim` | ≤ 12 fields |
| `FluidParams` | ≤ 12 fields |
| `GravitySource` | ≤ 6 variants |
| `PlatformHost::gyroscope()` | +1 método (default `None`) |
| `Brush.rendering.fluid_enabled: bool` | +1 campo em `RenderingParams` (sub-cap 9 → 10; cap ≤ 14 ADR-0044 §2.2.1) |
| `MemoryBudget::fluid_capable()` | +1 método inerente |
| `PerfBudget::fluid_headroom_ms()` | +1 método inerente |
| Fluid texture res | = 1/4 canvas (frozen) |
| Solver Jacobi iterations | 1..=8 |

### 2.14 Arch-gate `painter_contract_surface::fluid`

Adicionado ao homestead `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`:

```rust
#[cfg(feature = "fluid")]
mod fluid {
    #[test] fn fluid_sim_field_count_is_capped()        { /* ≤ 12 */ }
    #[test] fn fluid_params_field_count_is_capped()     { /* ≤ 12 */ }
    #[test] fn gravity_source_variant_count_is_capped() { /* ≤ 6 */ }
    #[test] fn fluid_pass_eligible_decision_table()     { /* truth table de §2.8 */ }
    #[test] fn fluid_det_mode_uses_256_res()            { /* §2.11 — res 256² em det builds */ }
}
```

### 2.15 Gates de comportamento

| Gate | Crate | Valida |
|---|---|---|
| `fluid_pass_activates_only_when_eligible` | ph2d-painter-fluid | Decision table §2.8: fluid_enabled × fluid_capable × headroom_ms → bool. Não ativa quando algum false. |
| `fluid_det_replay_cross_os` | idem (det-painter) | Replay CPU bit-identical Linux/macOS/Windows/WASM. |
| `fluid_perf_budget_under_3p5_ms_4k` | idem | Solver pass @ 4K canvas, 1/4 res = 1024², 8 Jacobi iterations ≤ 3.5 ms p99. Soft em W15; hard em W16+. |
| `fluid_graceful_degrade_to_wet_mix` | idem | Brush fluid_enabled=true em device low-end → wet mix matemático ativo; identidade visual preservada. |
| `gyroscope_permission_denied_falls_back_to_fixed` | ph2d-host | Web/iOS sem permission → `gyroscope() == None` → solver usa `GravitySource::Fixed` default. |

---

## 3. Consequências

### Positivas

- **Diferenciação técnica genuína vs Procreate** ([§14.8.1](../../Painter_projeto/14_inovacoes_extraordinarias.md)). Fluid sim real é "wow factor" defensável.
- **Opt-in feature flag isolado.** Mobile/Web buildings sem fluid não pagam custo binário nem VRAM.
- **Graceful degrade documentado.** Mid/Low devices recebem wet-mix matemático; identidade do brush mantida.
- **`PlatformHost::gyroscope()` é extensão mínima** (1 método com default `None`). Shells existentes não quebram.
- **Det-mode CPU fallback** preserva HR-5 replay cross-OS (ainda que 10× mais lento).
- **W15 deferral NÃO é aceitável** (regra 2026-05-26). Fluid ship em v1.0 ou ADR-0049-amendment-1 retrata explicitamente com decisão Enio. Telemetry pós-ship calibra continuamente.

### Negativas / Custos

- **Risco técnico mais alto das 5 Propostas.** Solver Shallow Water em GPU heterogêneo é desafio empírico. Mitigação: prototipagem cedo em W15 T-2 com canvas sample; abort cedo se device coverage abaixo de 80% mid-tier.
- **`ph2d-host::PlatformHost` é trait foundational tocada.** Não é fan-out drop-crate; exige Coord-A. Mitigação: 1 método default-`None`, fan-out shells absorve sem refactor.
- **20-30 MB VRAM adicional quando fluid ativo.** Cap incluso no budget §2.10 ([`08 §8.2`](../../Painter_projeto/08_performance_memory.md)).
- **2.5-3.5 ms perf hit quando fluid ativo.** Cabe 120Hz com folga; 60Hz com warning UX explícito (HUD "Fluid mode active").
- **`fluid_capable()` decision pode "flippar" frame-a-frame** (memory pressure varia). Mitigação: hysteresis 60 frames antes de degradar/reativar (não definido nesta ADR — registrar em W15 T-N implementation).

### Neutras

- **Crate `ph2d-painter-fluid` é folha.** Sem outro crate dep dele exceto consumer opcional `ph2d-tool-painter` feature-gated.
- **Det-mode CPU é não-aceitável para production paint** (10× slowdown), só para CI replay tests. Documentado.

---

## 4. Alternativas consideradas

### 4.1 Fluid sim sempre ativo (sem opt-in per-brush)

**Rejeitada.** 2.5-3.5 ms perf hit + 20-30 MB VRAM para brushes secos (pencil, ink, marker) é desperdício total. Opt-in é canon.

### 4.2 PlatformHost::gyroscope() retorna f32 escalar (em vez de Vec3)

**Rejeitada.** Vec3 dá direction + magnitude — suporta gravity sideways (device tilted). Escalar mata UX honesta (tinta escorre lateral quando device inclina).

### 4.3 Sem feature flag (crate sempre incluído)

**Rejeitada.** Web build sem fluid quer 0 bytes do crate. Mobile entry sem fluid quer 0 deps do solver. Feature flag canon evita pagar quando não usa.

### 4.4 Fluid solver no `ph2d-painter-brush` (mesmo crate)

**Rejeitada.** Acopla risco técnico maior (fluid) ao crate foundational da brush engine. Crate separado isola — se W15 falha, é só não compilar a feature. Brush engine intacta.

### 4.5 Det-mode fluid sim em GPU (não CPU fallback)

**Rejeitada.** GPU reductions + atomic ops são non-deterministic cross-platform. HR-5 exige bit-identical; CPU fallback é a saída honesta. 10× slowdown aceito para CI tests.

### 4.6 Skip ADR-0049 (deixar W15 sem contrato)

**Rejeitada.** W15 é Wave dedicada + a mais arriscada. Sem ADR, "Coord decide na hora" — exatamente o cenário que ADR-0040 (e a cascata W0 toda) sancionou contra.

---

## 5. Verificação

```sh
cargo test -p ph2d-painter-fluid --features fluid
# 4 caps + 5 behavior gates.

cargo test -p ph2d-painter-fluid --features fluid,det-painter
# Det replay cross-OS.

cargo build --workspace --no-default-features
# Build sem fluid — todos os shells / tool-painter funcionam.
```

### 5.1 Definição de "Accepted"

Esta ADR transita `Proposed → Accepted` no mesmo evento T0.9. Como é última na cascata + opt-in + deferível, sua reprovação não bloqueia W0 — apenas remove fluid do v1.0 roadmap.

---

## 6. Tracking

- Plano operacional: [§15 do plano §3 (T0.7) + §16 (W15)](../../Painter_projeto/15_plano_de_implementacao.md).
- Spec normativa: [§14.6 (Proposta 4)](../../Painter_projeto/14_inovacoes_extraordinarias.md) + [§8.2.2 budget](../../Painter_projeto/08_performance_memory.md).
- Risk policy: §2.12 desta ADR + [§14.6.9](../../Painter_projeto/14_inovacoes_extraordinarias.md).
- Próxima ação na cascata W0: **T0.9 — Aprovação Enio dos 7 ADRs**.
