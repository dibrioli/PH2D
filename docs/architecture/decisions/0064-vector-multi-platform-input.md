# ADR-0064 — Vector multi-platform input (Pencil/Wacom/S Pen + predict+reconcile + Metal overlay)

**Status:** Accepted (2026-05-29)
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Pré-requisitos:** [HR-1 — Platform-agnostic core](../../SKILL_Stack_PH2D_Definitiva.md), [ADR-0020 — Surface lifecycle](0020-surface-lifecycle.md), [ADR-0024 — Editor input + widget state](0024-editor-input-and-widget-state.md), [ADR-0050 — Device heterogeneity layer](0050-device-heterogeneity-layer.md) (Painter precedente).
**Spec normativa:** [`docs/Vector Module/11_pencil_pipeline.md`](../../Vector%20Module/11_pencil_pipeline.md).
**Amendment relacionado:** [`0020-amendment-1.md`](0020-amendment-1.md) — Metal Direct Overlay PlatformHost extension.
**Tags:** vector, wave-0, contract, input, pencil, multi-platform

---

## 1. Contexto

Vector Module promete **multi-plataforma desde W1**: Desktop (Mac/Win/Linux) + iPad (Apple Pencil) + Android (S Pen) + Web (Pointer Events). Sub-9 ms ProMotion latency target em iPad M-series.

L4F2 Antigravity 2ª iter absorvido: Bevy ECS + wgpu standard buffer queue = 12-16ms latency real. Sub-9ms exige Metal Direct Overlay shell-level. L1F5 3ª iter absorvido: PlatformHost extension formal + Metal Shared Events sync.

---

## 2. Decisão

### 2.1 PointerEvent unified model (via `ph2d-input` HR-1)

```rust
pub struct PointerEvent {
    pub kind: PointerKind,          // Pen | PenHover | Touch | Mouse
    pub source: PointerSource,      // ApplePencil2, WacomPro, SPen, etc.
    pub pos: Vec2,
    pub pressure: f32,              // [0..1]
    pub tilt: Vec2,                 // [-π/2, π/2] per axis
    pub azimuth: f32,               // [0, 2π)
    pub barrel: f32,                // [0, 1] Pencil Pro
    pub timestamp: Instant,
    pub predicted: bool,            // predict+reconcile flag
}
```

Zero `cfg(target_os)` em vector code (HR-1). Shells (`shells/desktop/`, `shells/ipad/`) injectam events via FFI.

### 2.2 Device capability matrix

| Device | Pressure | Tilt | Azimuth | Barrel | Squeeze | Hover | Latency target |
|--------|----------|------|---------|--------|---------|-------|----------------|
| Apple Pencil 1 | ✓ | ✓ | ✓ | — | — | — | ≤ 12 ms |
| Apple Pencil 2 | ✓ | ✓ | ✓ | — | — | — | ≤ 9 ms ✨ Modo B |
| Apple Pencil Pro | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ M-series | ≤ 9 ms ✨ Modo B |
| Wacom Cintiq Pro | ✓ | ✓ | ✓ | — | — | ✓ | ≤ 16 ms (60Hz) / ≤ 8 ms (120Hz) |
| Wacom Intuos Pro | ✓ | ✓ | ✓ | — | — | ❌ | ≤ 16 ms |
| Huion Kamvas | ✓ | ✓ | ✓ | — | — | partial | ≤ 16 ms |
| Samsung S Pen | ✓ | ✓ | ✓ | — | ✓ button | ✓ Air | ≤ 12 ms |
| Microsoft Surface Pen | ✓ | ✓ | partial | — | — | ✓ Studio | ≤ 12 ms |
| Mouse | ❌ const | ❌ | ❌ | — | — | ✓ | ≤ 1 frame |

Runtime detection via `PlatformHost::pointer_source_caps()`.

### 2.3 Predict+reconcile loop (sub-9 ms ProMotion)

```rust
struct PredictReconcile {
    history: VecDeque<PointerSample>,            // last N
    predicted_samples: SmallVec<[PointerSample; 4]>,
}

fn predict_next(&mut self, dt: f32) -> PointerSample {
    // Linear extrapolation OR cubic spline projection from history
    // OR Kalman-filter quality em complex cases
}

fn reconcile(&mut self, actual: PointerSample) {
    let discrepancy = (actual.pos - self.predicted_at(actual.timestamp).pos).length();
    if discrepancy > RECONCILE_THRESHOLD_PX {
        scene_mark_dirty();  // re-render speculative path corrected
    }
    self.history.push_front(actual);
}
```

ProMotion frame pacing iPad 120 Hz (8.3 ms/frame):
- Sensor: ~2 ms.
- OS → app: ~1 ms.
- Predict + scene update: ~0.5 ms.
- Render (Modo B Metal Direct OR Modo A Vello compute): ~1-1.5 ms.
- Compositor iOS: ~2 ms.
- **Total Modo B: ~6.5-9 ms** ✓ meets target.

### 2.4 Two-mode pipeline (L4F2 + L1F5 Antigravity 2ª+3ª iter)

| Modo | Backend | Latência | Devices |
|------|---------|----------|---------|
| **Modo A — Bevy/wgpu standard** | Bevy ECS scheduler + wgpu compositor padrão | 12-15 ms | Default desktop + Wacom + Web; iPad std non-ProMotion |
| **Modo B — Metal Direct Overlay** | Shell-level Metal CAMetalLayer overlay bypassing Bevy queue | **7-9 ms** ✓ | iPad Pro M-series + Mac M-series, opt-in tools Pencil-critical |

**Modo B PlatformHost extension formal (L1F5 Antigravity 3ª iter)** — vide [`0020-amendment-1.md`](0020-amendment-1.md):

```rust
#[cfg(any(target_os = "ios", target_os = "macos"))]
fn register_metal_overlay(
    &mut self,
    config: MetalOverlayConfig,
) -> Result<OverlayHandle>;

pub struct MetalOverlayConfig {
    pub pixel_format: PixelFormat,
    pub framebuffer_only: bool,
    pub sync_strategy: SyncStrategy,  // SharedEvent OR Independent
}
```

- **SharedEvent sync** (default): overlay e wgpu compartilham `MTLSharedEvent`; sem flickering.
- **Independent** (advanced): mais rápido mas pode glitch ocasional em loadshift.

### 2.5 Pressure / Tilt / Azimuth / Barrel curves per-tool

Default curves em Vector Preferences (global). Per-tool override em Tool Studio panel (W15):

```rust
pub struct PressureCurve {
    pub control_points: SmallVec<[Vec2; 8]>,    // (input, output) pairs em [0..1]²
    pub interpolation: CurveInterpolation,      // Linear | Cubic | Hermite
}
```

### 2.6 Palm rejection automática

- iPad: UIPencilInteractionDelegate.
- Wacom desktop: driver expõe stylus separate de mouse.
- Android: `MotionEvent::TOOL_TYPE_STYLUS` filter.
- Generic fallback: pressure > 0 → stylus; large contact area + low pressure → palm reject.

### 2.7 Hover preview enriquecido (L5F3 Antigravity 3ª iter)

Quando Pencil hovers sem touch:
- **Pen tool**: preview vertex + tangent cubic (em Spiro Assist, clothoid preview).
- **Pencil tool**: **elipse oriented por tilt+azimuth+pressure** (não simple width circle):
  - Major axis: predicted width via `width-profile.pressure_weight × predicted_pressure`.
  - Minor axis: asymmetria de tilt/azimuth.
  - Cor: stroke color current 40% opacity.
- **Eyedropper**: color swatch flotante + RGB/HSL numeric.
- **Shape tool**: shape outline preview.
- **Direct Select**: vertex highlight + tangent handles visible.

### 2.8 Pencil Pro features (graceful degrade non-Pro)

- **Squeeze**: open QuickMenu radial (configurable em Gesture Controls).
- **Barrel-roll** (Pen tool): rotate tangent magnitude.
- **Haptic feedback** (Pencil Pro): vibration em snap to grid/guide, close-path, vertex add, boolean cut.

Detection runtime via `PointerEvent::source` enum; UI cinza + tooltip "requires Apple Pencil Pro" em devices sem features.

### 2.9 Caps congelados

| Cap | Valor | Razão |
|---|---|---|
| Latency target Modo B M-series | **≤ 9 ms** | Apple ProMotion paridade |
| Latency target Modo A standard | **≤ 12-15 ms** | Bevy/wgpu queue real |
| Predict ahead frames | **1-4 frames** | Balance accuracy vs lookahead |
| Reconcile discrepancy threshold | **3 px** | Sub-pixel visual indistinguível |
| Palm rejection contact area threshold | **30 px²** | Empirical mid-finger contact |
| Hover detection range (Pencil M2+) | **~12 mm** | Apple-documented sensor range |

---

## 3. Consequências

### 3.1 Positivas

- **Sub-9 ms ProMotion alcançado** via Modo B Metal Direct Overlay (L4F2 absorbed properly).
- **PlatformHost extension formal** (L1F5) elimina flickering via Metal Shared Events.
- **Multi-plataforma desde W1** — Pencil + Wacom + S Pen + Mouse abstracted.
- **Hover preview enriquecido** dá UX competitive vs Linearity Curve.
- **Graceful degrade** Pencil Pro features sem build conditional (HR-1).

### 3.2 Negativas

- **Modo B exige shell iPad** (T0.14 pre-W1 task crítico — L2F3 Antigravity 3ª iter).
- **Two-mode pipeline** complexity em rendering layer. Justificado por latency targets.
- **Metal-only Modo B** — Vulkan/D3D12 paths não têm direct overlay equivalente. Aceito (Apple ProMotion único entre devices target sub-9ms).

### 3.3 Neutras

- Hover preview overlay ~0.3 ms render cost; cabe em budget.

---

## 4. Alternativas consideradas

### 4.1 Apenas Modo A standard (rejeitada — L4F2 sub-9ms inalcançável)

Bevy ECS + wgpu queue padrão. **Por que rejeitada**: 12-15 ms latency typical; Apple ProMotion target sub-9ms inviável.

### 4.2 Sem hover preview (rejeitada — UX gap Linearity Curve)

Skip hover. **Por que rejeitada**: Linearity Curve / Procreate hover é UX feature crítica. Stylus M2+ tem proximity sensor; usar.

### 4.3 Sem palm rejection (rejeitada — basic UX requirement)

Skip. **Por que rejeitada**: iPad multi-touch surface; sem rejection = unusable.

### 4.4 Vulkan Direct Overlay (rejeitada — não existe equivalent)

Espelhar Modo B em Linux/Android. **Por que rejeitada**: Vulkan não tem CAMetalLayer-equivalent direct overlay; Modo A é canon non-Apple.

---

## 5. Implementação (Wave 1 + Wave 17)

- **T0.14** (W1 pre-task CRITICAL L2F3): shell iPad scaffold (5-7d) destranca tudo.
- **T1.5/T1.6** (W1): Pen tool / Pencil tool com basic input.
- **T17.1** (W17): predict+reconcile loop sub-9 ms.
- **T17.2** (W17): hover preview enriquecido.
- **T17.3** (W17): Pencil Pro features (squeeze + barrel-roll).
- **T17.4** (W17): Wacom + S Pen + Mouse fallback.

Amendment ADR-0020 (`0020-amendment-1.md`) registra Metal overlay PlatformHost extension.

Gates: `vector_metal_overlay_no_flicker` + `vector_predict_reconcile_latency_sub_9ms` + `vector_palm_rejection_correctness`.

---

## 6. Referências

- Spec normativa: [`docs/Vector Module/11_pencil_pipeline.md`](../../Vector%20Module/11_pencil_pipeline.md) (381 linhas).
- ADR-0050 Device heterogeneity (Painter pattern paralelo).
- ADR-0020 Surface lifecycle (extension).
- Apple Pencil sub-9ms HN: <https://news.ycombinator.com/item?id=20091610>
- Antigravity 2ª iter L4F2 + 3ª iter L1F5 + L5F3 absorvidos.
