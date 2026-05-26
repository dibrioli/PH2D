# ADR-0050 — Device heterogeneity layer (pointer source + per-device curves + palm rejection + driver quirks)

**Status:** Proposed (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, cascata W0 rework regra perfeição).
**Pré-requisitos:** [ADR-0043 — Painter contract](0043-painter-contract.md), [ADR-0046 — Stroke Vector History](0046-stroke-vector-history.md).
**Spec normativa:** [`docs/Painter_projeto/07_pencil_pipeline.md`](../../Painter_projeto/07_pencil_pipeline.md) (pencil pipeline) + [`05_gestos_input.md`](../../Painter_projeto/05_gestos_input.md) (gestos).
**Motivação:** sense-check veterano paint stack 2026-05-26 (gaps "veteranos sempre perdem" 1, 2, 5).
**Tags:** painter, wave-0, contract, device-heterogeneity, pointer-input, palm-rejection, driver-quirks

---

## 1. Contexto

Pintar profissionalmente exige que o app **se adapte ao hardware**, não o contrário. O ecossistema hoje:

| Device family | Latency | Pressure curve | Tilt | Barrel | Hover | Quirks conhecidos |
|---|---:|---|---|---|---|---|
| **Apple Pencil 2 (iPad Pro)** | ~9 ms | linear hardware | ✓ | ✗ | ✓ | predictive 4ms (iOS API) |
| **Apple Pencil Pro (M4)** | ~9 ms | linear hardware | ✓ | ✓ (roll) | ✓ | haptic + squeeze gesture |
| **Wacom Cintiq / Intuos Pro** | ~10-15 ms | per-driver curve | ✓ | ✓ | ✓ (hover Cintiq) | driver firmware variations |
| **Wacom Bamboo (entry)** | ~15-25 ms | hard-cap binary | ✗ | ✗ | ✗ | sem tilt — `tilt=0` constante |
| **XP-Pen v2** | ~12-18 ms | jitter ~1-2% no centro do range | ✓ | ✗ | ✗ | known: false positives no edge |
| **Huion** | ~12-20 ms | barrel poll 200Hz (sobrecarrega CPU) | ✓ | ✓ | ✗ | barrel polling tem que ser throttled |
| **MS Surface Pen** | ~15 ms | non-linear curve | ✓ | ✗ | ✓ | hover events com gaps |
| **Mouse (genérico)** | ~5-10 ms | sem pressure | ✗ | ✗ | ✗ | precisa pressure curva simulada |
| **Touch (finger)** | ~25 ms | sem pressure | ✗ | ✗ | ✗ | palm rejection crítico |

Sem device heterogeneity layer:

1. **`pressure: f32` opaco** — ADRs 0043/0046 tratam pressure como número uniforme. Wacom curve ≠ Apple Pencil curve; user com 2 devices recebe pressões diferentes pelo mesmo gesto físico.
2. **Palm rejection ausente** — Procreate gastou ~3 anos polindo (2014-2017). PH2D não tem nem schema.
3. **Driver quirks viram bug-report** — XP-Pen jitter, Huion barrel polling, Mali compute crashes em fluid sim, todos viram tickets em produção.
4. **Hover state perdido em produção** — Wacom Cintiq + Surface Pen têm; ADR não trata.
5. **Sem device identification em StrokeRecord** — replay determinístico cross-device é impossível sem saber qual device gerou o stroke.

Regra "perfeição desde início, sem adiamentos" (2026-05-26): este ADR endereça os gaps **agora**, não como bug-fix pós-ship.

---

## 2. Decisão

### 2.1 Crate `ph2d-painter-input` (W1 T-input, novo)

```
crates/ph2d-painter-input/
  Cargo.toml         # deps: serde, ph2d-host (PlatformHost), ph2d-color
  src/lib.rs         # #![forbid(unsafe_code)] PRIMEIRO
  src/source.rs      # PointerSource enum + identification logic
  src/curve.rs       # PressureCurve + TiltCurve + BarrelCurve presets per-device
  src/palm.rs        # PalmRejection algoritmo + state machine
  src/quirks.rs      # DriverQuirks per-source workarounds
  src/prediction.rs  # Apple Pencil 4ms prediction integration; smoothing fallback
  src/hover.rs       # HoverState tracking (Wacom Cintiq + Surface Pen + Apple Pencil)
  tests/             # cross-device + palm rejection scenarios + quirks
```

**Dep direction:** `ph2d-painter-input` é folha (depende só de `ph2d-host` e `ph2d-color`). `ph2d-tool-painter` (ADR-0043) consome.

### 2.2 `PointerSource` enum — cap **≤ 16 variants** (v1 usa 11)

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PointerSource {
    Unknown = 0,                             // fallback
    Mouse = 1,
    Touch = 2,                               // finger, sem stylus
    ApplePencil1 = 10,
    ApplePencil2 = 11,
    ApplePencilPro = 12,                     // M4 iPad — squeeze + roll + haptic
    WacomIntuos = 20,
    WacomCintiq = 21,
    WacomBamboo = 22,                        // entry — sem tilt
    XpPenV2 = 30,
    Huion = 31,
    SurfacePen = 40,
    AndroidStylus = 50,                      // generic Adreno/Mali tablet
    WebPointer = 60,                         // PointerEvent.pointerType — capabilities discovery
    // === 2 slots de headroom (Wacom Cintiq Pro 27, Apple Pencil 3 quando aparecer) ===
}

impl PointerSource {
    pub fn has_tilt(&self) -> bool { /* tabela hard-coded */ }
    pub fn has_barrel(&self) -> bool { /* idem */ }
    pub fn has_hover(&self) -> bool { /* idem */ }
    pub fn supports_prediction(&self) -> bool { /* só ApplePencil* */ }
    pub fn default_pressure_curve(&self) -> PressureCurve { /* presets §2.3 */ }
}
```

### 2.3 `PressureCurve` + `TiltCurve` + `BarrelCurve` — cap **≤ 12 fields each**

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PressureCurve {
    pub points: [(f32, f32); 8],             // 8 control points (input, output) ∈ [0,1]²
    pub interpolation: CurveInterpolation,   // Linear | CatmullRom | Bezier
    pub clamp_min: f32,                      // hardware minimum pressure (e.g., Bamboo binary 0.5)
    pub clamp_max: f32,                      // hardware maximum (e.g., Cintiq 8192-level → normalize)
    pub jitter_filter_window: u32,           // XP-Pen jitter: median of N samples; default 1 (off)
    pub deadzone_low: f32,                   // pressure below = treated as 0 (touchstart noise)
    pub deadzone_high: f32,                  // pressure above = saturate (avoid clip artifacts)
    pub version: u32,                        // HR-14 v1 = 1
    // === 4 slots de headroom ===
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TiltCurve {
    pub max_angle_rad: f32,                  // π/2 default; alguns devices reportam menos
    pub remap_to_size: bool,                 // tilt afeta size compression
    pub remap_to_opacity: bool,
    pub points: [(f32, f32); 8],
    pub interpolation: CurveInterpolation,
    pub absent_fallback: TiltAbsentFallback, // {Zero, ConstantPi4, SimulateFromVelocity}
    pub version: u32,
    // === 5 slots de headroom ===
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BarrelCurve {
    pub max_angle_rad: f32,                  // 2π default
    pub poll_throttle_hz: u32,               // Huion 200Hz → throttle para 60Hz; default 60
    pub points: [(f32, f32); 8],
    pub interpolation: CurveInterpolation,
    pub version: u32,
    // === 8 slots de headroom ===
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CurveInterpolation { Linear = 0, CatmullRom = 1, Bezier = 2 }

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TiltAbsentFallback { Zero = 0, ConstantPi4 = 1, SimulateFromVelocity = 2 }
```

**Presets per-device:** `PressureCurve::default_for(PointerSource::WacomCintiq)` etc., baseado em medições reais (cabeçalho doc-comment cita fonte e ano). Usuária override em Painter Preferences → Pencil/Device.

### 2.4 `PalmRejection` — algoritmo congelado

**Modelo:** state machine que classifica touch events em `Stylus | Palm | Pending`, com latência de classification ≤ 100 ms (UX target — palm não vira stroke visível).

```rust
#[derive(Clone, Debug)]
pub struct PalmRejection {
    pub state: PalmState,
    pub config: PalmRejectionConfig,
    pub active_strokes: BTreeMap<TouchId, PalmClassification>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PalmRejectionConfig {
    pub min_contact_area: f32,               // touch area > este = candidate palm; default 80 mm²
    pub stylus_priority_window_ms: u32,      // stylus chega + 100ms = qualquer touch nesse range = palm
    pub thumb_zone_disabled: bool,           // borda inferior 5cm = palm by default em landscape
    pub thumb_zone_height_px: u32,           // default 200 px
    pub require_stylus_first: bool,          // false (start sem stylus = first touch = stroke; default true em iPad)
    pub palm_release_grace_ms: u32,          // palm levantando deixa 200 ms grace antes de aceitar new stroke
    pub version: u32,
    // === 4 slots de headroom ===
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PalmClassification {
    Stylus,                                  // confirmed stylus event
    Palm,                                    // confirmed palm — DESCARTA
    Pending(u32),                            // ms aguardando classificação; durante isso buffer events
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PalmState { Idle, ActiveStylus, PalmGrace }
```

**Critérios de classificação:**
1. **Stylus event explícito** (PointerEvent.pointerType = "pen" / iOS UITouch.type = .stylus) → `Stylus`.
2. **Touch após stylus dentro de `stylus_priority_window_ms`** → `Palm`.
3. **Touch com contact area > `min_contact_area`** → `Palm` (palma deita maior).
4. **Touch em `thumb_zone` (landscape inferior 200 px)** com `thumb_zone_disabled=true` → `Palm`.
5. **Touch first (sem stylus prévio)** + `require_stylus_first=false` → `Stylus` (mouse/touch-only mode).
6. **Caso ambíguo:** `Pending(ms)`, buffer events; reclassifica em 100 ms.

**Gate `palm_rejection_p99_under_100ms`:** classification para 99% dos cases em ≤ 100ms.

### 2.5 `DriverQuirks` — workarounds catalogados

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u32)]
pub enum DriverQuirk {
    /// XP-Pen v2: jitter ~1-2% no centro do range. Workaround: median filter window=3.
    XpPenCenterJitter = 1,
    /// Huion: barrel polling a 200Hz sobrecarrega CPU. Workaround: throttle a 60Hz.
    HuionBarrelOverpoll = 2,
    /// Wacom Bamboo: tilt sempre 0. Workaround: TiltAbsentFallback::SimulateFromVelocity.
    BambooNoTilt = 3,
    /// Surface Pen: hover events com gaps de até 100ms. Workaround: hold last hover state 200ms.
    SurfacePenHoverGaps = 4,
    /// Mali G77: timeout em Jacobi >4 iterations (fluid sim). Workaround: cap iter=4 em Mali.
    MaliJacobiTimeout = 5,
    /// Adreno 650: compute compaction crash em batch >512 stamps. Workaround: cap batch=512.
    Adreno650ComputeCompactionCrash = 6,
    /// Intel UHD: fluid sim 1024² OOM. Workaround: cap fluid_size=512².
    IntelUhdFluidOom = 7,
    // === reserved slots para quirks futuros descobertos via telemetry ===
}

impl DriverQuirk {
    pub fn applies_to(&self, source: PointerSource, gpu: GpuId) -> bool { /* tabela */ }
    pub fn workaround_action(&self) -> QuirkAction { /* enum: FilterJitter, ThrottleHz, CapIter, etc. */ }
}
```

**Cap:** `DriverQuirk ≤ 32 variants` (catálogo cresce com telemetry de produção; cada quirk descoberto entra como amendment + bump).

### 2.6 `HoverState`

```rust
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HoverState {
    pub position: [f32; 2],                  // canvas coords
    pub pressure_estimated: Option<f32>,     // some devices report estimated pressure on hover
    pub tilt_estimated: Option<f32>,
    pub source: PointerSource,
    pub timestamp_ms: u64,
    pub last_event_age_ms: u32,              // útil para detect gap (Surface Pen)
    // 6 fields fixos; cap = 8 com 2 slots de headroom
}
```

`HoverState::is_stale(threshold_ms=200)` → device com hover gaps degrada para `false` após 200ms; UI esmaece cursor preview.

### 2.7 `RawPointerSample` (ADR-0046) ganha `device_source: PointerSource`

Amendment de ADR-0046 §2.3 (sancionado pelo bump de cap headroom): adicionar campo `device_source` ao `RawPointerSample`. Conta no cap **≤ 12 fields** existente — v1 usa 8 + device_source = 9; 3 slots de headroom restantes.

```rust
// ADR-0046 §2.3 amended:
pub struct RawPointerSample {
    pub x_q1616: i32,
    pub y_q1616: i32,
    pub pressure_q88: u16,
    pub tilt_q88: u16,
    pub azimuth_q88: u16,
    pub barrel_roll_q88: u16,
    pub timestamp_delta_us: u32,
    pub flags: u32,
    pub device_source: PointerSource,        // NEW ADR-0050 §2.7
    // 3 slots de headroom
}
```

`StrokeRecord` herda capability indirectly via samples — replay determinístico cross-device é viável (mesma curve aplicada em replay).

### 2.8 Apple Pencil prediction integration

iOS oferece **4ms prediction** via `UITouch.estimatedProperties`. Painter consome:

```rust
pub struct PredictionState {
    pub predicted_points: Vec<RawPointerSample>,
    pub confirmed_seq: u64,                  // last seq não-predicted
    pub max_lookahead_ms: u32,               // default 4; iOS hardware cap
}
```

`StrokeRecord.points` grava só **confirmed** samples; preview render usa `predicted_points` (vivos só durante stroke ativo, descartados no commit).

**Other devices:** prediction implementada via smoothing fallback (~2 samples ahead extrapolation linear) com cap `max_lookahead_ms = 4`.

### 2.9 Caps numéricos

| Tipo | Cap |
|---|---|
| `PointerSource` | ≤ 16 variants (v1 = 14) |
| `PressureCurve` | ≤ 12 fields (v1 = 8) |
| `TiltCurve` | ≤ 12 fields (v1 = 7) |
| `BarrelCurve` | ≤ 12 fields (v1 = 4) |
| `CurveInterpolation` | ≤ 4 variants (v1 = 3) |
| `TiltAbsentFallback` | ≤ 4 variants (v1 = 3) |
| `PalmRejectionConfig` | ≤ 12 fields (v1 = 7) |
| `PalmClassification` | ≤ 4 variants (v1 = 3) |
| `DriverQuirk` | ≤ 32 variants (v1 = 7; catálogo cresce com telemetry) |
| `HoverState` | ≤ 8 fields (v1 = 6) |
| `PredictionState` | ≤ 6 fields (v1 = 3) |

### 2.10 Arch-gate `painter_contract_surface::input` (`ph2d-painter-contracts/tests/`)

```rust
mod input {
    #[test] fn pointer_source_variant_count_is_capped()      { /* ≤ 16 */ }
    #[test] fn pressure_curve_field_count_is_capped()        { /* ≤ 12 */ }
    #[test] fn tilt_curve_field_count_is_capped()            { /* ≤ 12 */ }
    #[test] fn barrel_curve_field_count_is_capped()          { /* ≤ 12 */ }
    #[test] fn palm_rejection_config_field_count_is_capped() { /* ≤ 12 */ }
    #[test] fn driver_quirk_variant_count_is_capped()        { /* ≤ 32 */ }
    #[test] fn hover_state_field_count_is_capped()           { /* ≤ 8 */ }
    #[test] fn raw_pointer_sample_has_device_source()        { /* ADR-0046 §2.3 amended */ }
    #[test] fn all_devices_have_default_curves()             { /* PointerSource::* tem default_pressure_curve() */ }
    #[test] fn all_devices_have_quirks_mapped()              { /* PointerSource × GpuId → known quirks */ }
}
```

### 2.11 Gates de comportamento

| Gate | Crate | Valida |
|---|---|---|
| `palm_rejection_p99_under_100ms` | ph2d-painter-input | Classification 99% em ≤ 100ms cross-device. |
| `apple_pencil_prediction_drops_at_commit` | idem | Predicted points não entram em StrokeRecord.points. |
| `xppen_jitter_filter_engages` | idem | Touch com `device_source=XpPenV2` ativa median filter window=3. |
| `huion_barrel_throttle_engages` | idem | Touch com `device_source=Huion` cap barrel poll a 60Hz. |
| `bamboo_tilt_simulates_from_velocity` | idem | `device_source=WacomBamboo` + missing tilt → simulate from velocity. |
| `hover_gap_200ms_releases_state` | idem | Surface Pen hover gap > 200ms → HoverState::is_stale() = true. |
| `mali_jacobi_iter_cap_4` | idem (cross com ADR-0049) | GPU Mali G77 → fluid solver cap iter=4. |
| `adreno650_stamp_batch_cap_512` | idem | GPU Adreno 650 → stamp batch cap=512. |
| `intel_uhd_fluid_size_cap_512` | idem (cross com ADR-0049) | GPU Intel UHD → fluid_size=512² (não 1024²). |

---

## 3. Consequências

### Positivas

- **Stroke determinístico cross-device.** `device_source` no `RawPointerSample` permite replay aplicar a mesma curve usada na captura, não a curve default do device atual.
- **Palm rejection profissional desde v1.** Não há gap para Procreate ou Adobe Fresco.
- **Driver quirks catalogados, não bug-de-produção.** Telemetry de campo identifica quirk novo → ADR-amend adiciona variant → fix mecânico.
- **Apple Pencil prediction nativo.** UX premium em iPad Pro de saída.
- **Heterogeneity layer isolado** em crate folha — sem acoplamento com brush engine.

### Negativas / Custos

- **Curve presets exigem medições reais** (não chute). W1 T-input inclui sessão de medição com devices reais (Apple Pencil 2, Wacom Intuos, XP-Pen) — custo: ~1 semana de coleta + tuning. Aceito vs gap "device heterogeneity".
- **Palm rejection state machine cresce** com casos de uso reais. v1 cobre 6 critérios (§2.4); telemetry pode trazer 5-10 quirks novos pós-ship. Mitigação: cap ≤ 12 deixa headroom; ADR-amend para novos critérios.
- **`DriverQuirk` cap 32 parece grande** mas a indústria conhece >20 quirks documentados em paint stacks. Cap honesto.

### Neutras

- **`device_source` adiciona 1 byte ao Stamp/Sample.** Total Sample 24 → 25 bytes; cabe em 256-points-per-stroke budget. Negligible.

---

## 4. Alternativas consideradas

### 4.1 Single `PressureCurve` global (sem per-device)

**Rejeitada.** Wacom curve ≠ Pencil curve fisicamente. Usuária com 2 devices ganharia pressões diferentes pelo mesmo gesto. Mata UX profissional.

### 4.2 Palm rejection adiada para "pós-ship" (estilo Procreate 2014)

**Rejeitada.** Regra "perfeição desde início" (2026-05-26). Procreate gastou 3 anos polindo isso; PH2D não vai sangrar isso em produção quando o ADR pode entregar v1 honesto.

### 4.3 `DriverQuirks` como hardcoded `if/else` (não enum catalogado)

**Rejeitada.** Quirks futuros adicionados ad-hoc viram débito invisível. Enum catalogado + tabela `applies_to` torna catalog auditável + amendable.

### 4.4 Prediction como detalhe interno do shell

**Rejeitada.** Prediction é parte do **contrato de input** — afeta UX visual diretamente. ADR-level decision.

---

## 5. Verificação

```sh
cargo test -p ph2d-painter-input
# 10+ caps + 9+ behavior gates.

cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# Caps cumulativos (cascata W0 + ADR-0050).
```

---

## 6. Tracking

- Plano operacional: integra em W1 T-input (novo) — precedência sobre T1.5 brush implementation.
- Spec normativa: integra com `07_pencil_pipeline.md` + `05_gestos_input.md`.
- Próxima ADR na cascata W0 rework: [ADR-0051 — Color profile pipeline](0051-color-profile-pipeline.md).
