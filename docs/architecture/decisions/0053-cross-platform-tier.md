# ADR-0053 — Cross-platform tier policy (universal feature parity + graceful degrade)

**Status:** Accepted (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, cascata W0 rework regra perfeição).
**Pré-requisitos:** [ADR-0043 — Painter contract](0043-painter-contract.md), [ADR-0049 — Fluid Brushes](0049-fluid-brushes.md), [ADR-0050 — Device heterogeneity](0050-device-heterogeneity-layer.md), [ADR-0051 — Color profile pipeline](0051-color-profile-pipeline.md).
**Motivação:** audit padrão-ouro 2026-05-26 (C5 "features marquise só rodam em desktop / iPad Pro M1+"). Sense-check veterano (cross-platform claims sem device coverage).
**Tags:** painter, wave-0, contract, cross-platform, graceful-degrade, tier-policy, universal-parity

---

## 1. Contexto

PH2D Painter posiciona-se como "**sucessor multiplataforma** do Procreate". Procreate hoje é iPad-only — PH2D promete iPad + iPhone + Android + Web + desktop (Mac/Windows/Linux). Mas auditoria padrão-ouro flagged:

> "As duas features mais marquise (Fluid + Inspector retroativo profundo) só rodam plenamente em desktop / iPad Pro M1+ / Android top-tier. O 'sucessor multiplataforma' é cross-platform na superfície, não no padrão-ouro."

E sense-check veterano:

> "Cross-platform claims sem device coverage. Fluid em Android entry? Web WGPU?"

Sem ADR-0053:

1. **Tier classification implícito.** Cada feature decide individualmente "mid-tier acima funciona, abaixo não" — fragmentação UX.
2. **"Padrão-ouro multiplataforma" é claim sem contrato.** Marketing vs realidade técnica divergem.
3. **Graceful degrade não é uniforme.** Fluid cai para wet-mix (matemático). Inspector cai para... nada? Mixbox cai para Linear lerp? Cada feature decide em silos.
4. **Sem policy de comunicação ao user.** "Você está em tier baixo" = como dizer? Onde? Toast? Settings page?

Regra "perfeição desde início, sem adiamentos" obriga decisão **explícita**: ou universal feature parity (caro), ou tier-tagged honest UX.

---

## 2. Decisão

### 2.1 Universal feature parity como goal — graceful degrade como fallback honesto

**Decisão principal:** **toda feature roda em toda plataforma**, **MAS** com graceful degrade explícito + UX honesto sobre o tier ativo.

Não há "Painter Pro" vs "Painter Lite" SKU — é o **mesmo Painter em todos os devices**, com adaptive quality conforme `DeviceCapability` (§2.2). Filosofia: **competir com Procreate inclui ser MELHOR em cross-platform** (Procreate não tem cross-platform).

### 2.2 `DeviceCapability` tier classification — congelado

Vive em `ph2d-host::DeviceCapability` (extension da trait):

```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DeviceTier {
    /// Top: M1+/M2+/M3+ iPad Pro/Mac, RDNA3 desktop, RDNA2 high-end mobile,
    /// Apple A17 Pro+. 8+ GB RAM. All features full quality.
    Top = 4,
    /// High: M1 iPad Air, A14+ iPad, RDNA2 desktop, RTX 30+, Snapdragon 8 Gen 2+,
    /// Apple A15+. 6-8 GB RAM. All features; some perf cap (e.g., fluid 512²).
    High = 3,
    /// Mid: iPad 2020-2022, A12-A13, Adreno 660+, Mali G77+, Intel Iris Xe,
    /// RDNA1 mobile. 4-6 GB RAM. All features ativas; quality reduzida (e.g., fluid 256²).
    Mid = 2,
    /// Low: iPad 2018, A10-A11, Adreno 6xx older, Mali G7x older, Intel UHD,
    /// Snapdragon 7xx. 3-4 GB RAM. Features ATIVAS mas com matemática fallback
    /// (e.g., Fluid → wet_mix matemático; Procedural Grain → bitmap atlas).
    Low = 1,
    /// Web baseline: Chrome/Firefox/Safari WGPU em laptop entry, Chromebook.
    /// Web tem cap RAM dura (200-500 MB), storage limits, sem fsync garantido.
    /// Features ativas; alguns gates suavizados (e.g., WAL flush 250ms vs 100ms).
    Web = 0,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeviceCapability {
    pub tier: DeviceTier,
    pub gpu_id: GpuId,                       // ADR-0050 §2.5 — para quirks lookup
    pub ram_total_mb: u32,
    pub vram_estimate_mb: u32,
    pub display_p3_capable: bool,            // ADR-0051 §2.2
    pub hdr_capable: bool,                   // Rec2100Pq display
    pub has_gyroscope: bool,                 // ADR-0049 §2.2
    pub stylus_pressure_levels: u16,         // 0=touch only, 8192=Wacom Cintiq, 1024=Apple Pencil
    pub version: u32,                        // HR-14 v1 = 1
    // === 3 slots de headroom ===
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GpuId {
    Unknown = 0,
    AppleSiliconM1 = 10, AppleSiliconM2 = 11, AppleSiliconM3 = 12, AppleSiliconM4 = 13,
    AppleA14 = 20, AppleA15 = 21, AppleA16 = 22, AppleA17 = 23,
    AppleM_iPadProSeries = 30,                // generic legacy iPad Pro chips
    Rdna1 = 40, Rdna2 = 41, Rdna3 = 42,
    Nvidia20xx = 50, Nvidia30xx = 51, Nvidia40xx = 52,
    Adreno6xx = 60, Adreno7xx = 61,
    MaliG7x = 70, MaliG77 = 71, MaliG78 = 72, MaliImmortalis = 73,
    IntelUhd = 80, IntelIrisXe = 81, IntelArc = 82,
    WebGpuBaseline = 90,
    AndroidGenericVulkan = 100,
    // === slots de headroom ===
}
```

**Caps:** `DeviceTier = 5 variants FROZEN`, `DeviceCapability ≤ 16 fields` (v1 = 9), `GpuId ≤ 64 variants` (v1 ≈ 20; cresce conforme telemetry pos-ship).

### 2.3 Per-feature degradation table (CONGELADA)

| Feature | Top | High | Mid | Low | Web |
|---|---|---|---|---|---|
| **Stroke history full** (ADR-0046) | ✓ Full | ✓ Full | ✓ Full | ✓ Ring 5000 | ✓ Ring 1000 |
| **Inspector dirty-rect propagation** (ADR-0048) | ✓ Full | ✓ Full | ✓ Full | ✓ Full | ✓ Full (lazy R-tree rebuild ok) |
| **Mixbox pigment mixing** (ADR-0044 §2.5) | ✓ Mixbox | ✓ Mixbox | ✓ Mixbox | ✓ Mixbox (CPU fallback se GPU lento) | ✓ Mixbox |
| **Procedural Grain** (ADR-0044 §2.7) | ✓ Procedural compute | ✓ Procedural compute | ✓ Procedural compute | ⚠ Bitmap atlas fallback (4 grãos procedural → bitmap) | ⚠ Bitmap atlas fallback |
| **Fluid Brushes** (ADR-0049) | ✓ 1024² GPU | ✓ 512² GPU | ✓ 256² GPU | ⚠ wet_mix matemático (no fluid sim) | ⚠ wet_mix matemático |
| **Adjustment Layers** (ADR-0045) | ✓ All | ✓ All | ✓ All | ✓ All | ✓ All (cap layers via MemoryBudget) |
| **MCP Stroke Engine** (ADR-0047) | ✓ Full | ✓ Full | ✓ Full | ✓ Full | ✓ Full (sem mudança) |
| **HDR canvas** (ADR-0051) | ✓ Rec2100Pq | ✓ Se display capable | ✗ SDR only | ✗ SDR only | ✗ SDR only |
| **Wide-gamut Display P3** (ADR-0051) | ✓ | ✓ | ✓ | ✓ Se display capable | ✓ Se browser supports |
| **Auto-save WAL flush** (ADR-0052) | 100ms | 100ms | 100ms | 100ms | 250ms (Web fsync limits) |
| **R-tree spatial index** (ADR-0048) | ✓ Persisted | ✓ Persisted | ✓ Persisted | ✓ Persisted | ⚠ In-memory only (Web storage limits) |
| **Crash recovery** (ADR-0052) | ✓ Full | ✓ Full | ✓ Full | ✓ Full | ⚠ IndexedDB-based (Web) |

**Princípio de design:**
- ✓ = feature ativa na full quality / standard quality / degradation menor.
- ⚠ = feature ativa em **forma alternativa** (não removida) que preserva identidade visual. Ex: "Fluid Brushes" em Low/Web ainda existe — usa `wet_mix` matemático (não fluid sim física). Usuária pinta com `watercolor_wash` e ainda VÊ aquarela; só não tem o "wow" da física.
- ✗ = feature requer hardware capability ausente; UX skip explícito.

**Crucialmente:** **TODA capability brush** (stroke history, layers, adjustments, Mixbox, color picker, MCP, retroactive edit) **roda em todo tier**. Diferenças são em qualidade de simulation física / atlas resolution / HDR display, NÃO em capability funcional.

### 2.4 UX honesto — `TierIndicator`

Settings → About mostra:

```
┌──────────────────────────────────────────┐
│ Painter capability                        │
│                                            │
│  Tier: Mid                                 │
│  GPU: Adreno 660                           │
│  RAM: 6 GB available                       │
│                                            │
│  ✓ All features available                  │
│  ⚠ 2 features use adaptive quality:        │
│     • Fluid Brushes (256² simulation)      │
│     • Procedural Grain (bitmap fallback)   │
│                                            │
│  [Why? See platform tiers]                 │
└──────────────────────────────────────────┘
```

**Princípios UX:**
- Tier é visível mas **não bloqueia UX normal** (no popup/dialog forçado).
- Linguagem: "adaptive quality", NÃO "lite version" / "feature missing".
- Toast só aparece quando feature ativada em runtime hitting degrade (first time per session).
- `[Why?]` link abre doc explicando tiers + razão técnica.

### 2.5 `PlatformHost::device_capability()` — trait extension

```rust
pub trait PlatformHost {
    // ... existentes ...

    /// Returns the device capability assessment. Cached after first call.
    /// Default impl: probes via GPU adapter info + RAM detection + display caps.
    fn device_capability(&self) -> DeviceCapability { /* fallback heuristics */ }
}
```

Coord-A only (foundational); autorizado por ADR-0043 §2.5 ceded list. Adicionado nesta cascata rework.

### 2.6 Tier upgrade post-detection

Telemetry pode descobrir que detection inicial estava errada (e.g., usuário em Low actually runs fluid sim fine no seu device específico). Painter Preferences tem opção **manual override**:

```
┌──────────────────────────────────────────┐
│ Painter capability tier                   │
│                                            │
│  Auto-detected: Mid                        │
│  Override: [Top ▼]                         │
│   ⚠ Manual override. App may stutter or    │
│     crash if device cannot sustain Top     │
│     tier. We log crashes for telemetry.    │
└──────────────────────────────────────────┘
```

Telemetry de manual override + crashes → next release update auto-detection heuristics.

### 2.7 Tier downgrade automático sob thermal pressure

`PlatformHost` em iOS/Android reporta thermal state (ThermalState enum). Quando `thermal_state >= Critical`:

```rust
if platform.thermal_state() >= ThermalState::Critical {
    // Reduzir efetivo tier dinamicamente
    runtime_tier = current_tier.downgrade_one_step();
    // Fluid sim de 1024² → 256²; Procedural Grain → bitmap; etc.
    // UX toast (subtle): "Reducing simulation quality due to device temperature."
}
```

Auto-restore quando thermal volta a Normal. Hysteresis 30s para evitar oscilação.

**Cap:** `ThermalState ≤ 6 variants` (Normal, Fair, Serious, Critical, ShutdownImminent, Unknown). Padrão indústria.

### 2.8 Caps numéricos

| Tipo | Cap |
|---|---|
| `DeviceTier` | **= 5 variants FROZEN** |
| `DeviceCapability` | ≤ 16 fields (v1 = 9) |
| `GpuId` | ≤ 64 variants (v1 ≈ 20; cresce telemetry) |
| `ThermalState` | ≤ 6 variants (v1 = 5) |
| Degradation table rows | = 12 features categorias (FROZEN — adicionar feature requer ADR-amend) |

### 2.9 Arch-gate `painter_contract_surface::tier_policy`

```rust
mod tier_policy {
    #[test] fn device_tier_variant_count_is_exact_5()           { /* FROZEN */ }
    #[test] fn device_capability_field_count_is_capped()        { /* ≤ 16 */ }
    #[test] fn gpu_id_variant_count_is_capped()                 { /* ≤ 64 */ }
    #[test] fn thermal_state_variant_count_is_capped()          { /* ≤ 6 */ }
    #[test] fn all_features_have_tier_entry()                   { /* table §2.3 completa para 12 features */ }
    #[test] fn no_feature_is_disabled_in_low_or_web()           { /* ✗ apenas em HDR; resto é ✓ ou ⚠ */ }
}
```

### 2.10 Gates de comportamento

| Gate | Crate | Valida |
|---|---|---|
| `fluid_degrades_to_wet_mix_in_low_tier` | ph2d-painter-fluid (cross ADR-0049) | Tier=Low + brush fluid_enabled → wet_mix matemático ativo; identity visual preservada. |
| `procedural_grain_degrades_to_bitmap_in_low_tier` | ph2d-painter-brush | Tier=Low → 4 procedural grains carregam bitmap atlas equivalent. |
| `stroke_history_caps_at_ring_in_low_tier` | ph2d-painter-stroke | Tier=Low → StrokeHistory::Ring{cap: 5000}; Web → Ring{cap: 1000}. |
| `wal_flush_interval_relaxes_in_web` | ph2d-painter-stroke | Tier=Web → FlushPolicy 250ms (não 100ms). |
| `tier_indicator_visible_in_settings` | shells/* | Settings → About mostra tier + features adaptive. |
| `thermal_critical_downgrades_runtime` | ph2d-host + ph2d-painter-* | ThermalState=Critical → runtime tier downgrade applied. |
| `manual_tier_override_persisted` | ph2d-host | User manual override survive restart. |
| `tier_downgrade_hysteresis_30s` | ph2d-host | Auto downgrade/upgrade não oscila < 30s. |

---

## 3. Consequências

### Positivas

- **"Sucessor multiplataforma" é contratualmente honesto.** Toda feature em todo tier; degradação é qualidade de simulation, não capability removal.
- **Procreate cross-platform comparison:** Procreate é iPad-only. PH2D ship em 5+ plataformas com feature parity. **Esta é uma 6ª/7ª dimensão de superioridade** sobre Procreate (audit C5 problema → audit dimensão+).
- **Thermal management.** Mobile em sessões longas não trava — auto-downgrade gracioso.
- **Manual override + telemetry.** Power users podem forçar Top tier; telemetry calibra heuristics.
- **UX honesto.** Não esconde tier; explica adaptive quality.

### Negativas / Custos

- **Per-feature degradation table cresce com features futuras.** Cada nova feature precisa ADR-amend adicionando linha. Aceito vs caos.
- **Telemetry infra adicional.** Manual override + thermal events + crash logs. Aceito; OS já fornece APIs.
- **Bitmap atlas fallback para Procedural Grain em Low/Web** exige asset duplication (4 grãos × bitmap 1024² R8 = 4 MB extra storage). Aceito; ship com bitmap baseline.

### Neutras

- **Universal feature parity philosophy é decisão de produto.** Alternativa (tier-gated SKU) considerada e rejected.

---

## 4. Alternativas consideradas

### 4.1 Tier-gated SKU (Painter Pro vs Painter Lite)

**Rejeitada.** Fragmenta UX. Usuário em iPad 2018 ainda pode pintar profissionalmente; tier-gating diz "você é cidadão de 2ª classe". Mudaria a competição: vs Procreate é "mesmo nível". Vs Adobe Lightroom Mobile (que TEM tier gates) é "diferente". Decisão: PH2D não vai por aí.

### 4.2 Feature parity total sem degradação (universal Top quality)

**Rejeitada.** Fluid sim em Mali G77 com timeout em Jacobi >4 = crash. Procedural Grain em Web WGPU baseline = OOM em canvas 4K. Universal Top quality é tecnicamente impossível em low-end.

### 4.3 Degradação implícita (cada feature decide silosly)

**Rejeitada.** Fragmenta UX (Fluid degrada de uma forma, Procedural de outra, Inspector talvez não, sem padrão). Tabela central + DeviceTier canônico evita inconsistência.

### 4.4 Sem manual override

**Rejeitada.** Power users querem control. Procreate não tem (porque iPad-only homogeneo); PH2D tem multiplataforma → override é UX honesto.

### 4.5 Sem thermal awareness

**Rejeitada.** iPad em sessão 4h aquece; sem auto-downgrade, app trava. Procreate aborda; PH2D obrigado a abordar.

---

## 5. Verificação

```sh
cargo test -p ph2d-host
# DeviceCapability detection + thermal state handling.

cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# Caps cumulativos.
```

**Manual smoke obrigatório em W1 T-tier-policy:**
1. Boot em Mac M2 → tier=Top, Fluid 1024².
2. Boot em iPad 2020 → tier=Mid, Fluid 256².
3. Boot em iPad 2018 → tier=Low, wet_mix matemático.
4. Boot em Chrome WGPU → tier=Web, ring 1000.
5. Manual override Mid → Top em iPad 2020; pinta 5 min; thermal Critical → auto-downgrade Mid.

---

## 6. Tracking

- Plano operacional: integra em W1 T-tier-policy (foundational; precede T-input/T-color/T-durability).
- Spec normativa: integra com `08_performance_memory.md` (budgets per tier).
- Telemetry pós-ship: calibração continuous de heuristics + GpuId catálogo expansion.
- Cascata W0 rework completa após esta ADR (próximas ações são rework ADR-0047 e ADR-0045 ja existentes).
