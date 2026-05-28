# 11 — Pencil pipeline (multi-platform input)

> Spec do **input pipeline** do Vector Module. Apple Pencil (iPad), Wacom / Huion / XP-Pen (desktop), S Pen (Android), mouse fallback. **Predict+reconcile sub-9 ms** em ProMotion. Pressure / tilt / azimuth / barrel curves per-tool. Palm rejection automática. Hover preview onde supported.
>
> **ADR ratificador:** ADR-0064 (Vector multi-platform input).
> **Espelha Painter [07_pencil_pipeline.md](../Painter_projeto/07_pencil_pipeline.md) + Painter ADR-0050 (Device heterogeneity layer).**

## 11.1 PointerEvent unified model

### 11.1.1 Source

`ph2d-input` crate (já existing, expanded). Single source of truth — Vector Module consume via standard API.

### 11.1.2 Estrutura

```rust
pub struct PointerEvent {
    pub kind: PointerKind,        // Pen | Touch | Mouse
    pub source: PointerSource,    // ApplePencil2, WacomPro, SPen, etc.
    pub pos: Vec2,                // canvas coords
    pub pressure: f32,            // [0..1] normalized
    pub tilt: Vec2,               // [-π/2, π/2] per axis
    pub azimuth: f32,             // [0, 2π) compass rotation
    pub barrel: f32,              // [0, 1] for Pencil Pro
    pub timestamp: Instant,
    pub predicted: bool,          // true if predict+reconcile predicted sample
}
```

### 11.1.3 Cross-platform via HR-1

`ph2d-input` é platform-agnostic. Shell desktop usa `winit` adapter; iPad shell usa `UIPencilInteraction` adapter; Android usa `MotionEvent` adapter. **Zero `cfg(target_os)` em vector code.**

---

## 11.2 Device capability matrix

| Device | Pressure | Tilt | Azimuth | Barrel | Squeeze | Double-tap | Hover | Latency target |
|--------|----------|------|---------|--------|---------|------------|-------|----------------|
| Apple Pencil 1 | ✓ | ✓ | ✓ | — | — | — | — | ≤ 12 ms |
| Apple Pencil 2 | ✓ | ✓ | ✓ | — | — | ✓ | — | ≤ 9 ms |
| Apple Pencil Pro | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (M-series) | ≤ 9 ms |
| Wacom Cintiq Pro | ✓ | ✓ | ✓ | — | — | — | ✓ | ≤ 16 ms (60Hz) / ≤ 8 ms (120Hz) |
| Wacom Intuos Pro | ✓ | ✓ | ✓ | — | — | — | ❌ | ≤ 16 ms |
| Huion Kamvas | ✓ | ✓ | ✓ | — | — | — | partial | ≤ 16 ms |
| XP-Pen Innovator | ✓ | ✓ | ✓ | — | — | — | ❌ | ≤ 16 ms |
| Samsung S Pen (Note/Tab S series) | ✓ | ✓ | ✓ | — | ✓ (button) | — | ✓ (Air) | ≤ 12 ms |
| Microsoft Surface Pen | ✓ | ✓ | partial | — | — | — | ✓ (Surface Studio) | ≤ 12 ms |
| Mouse | ❌ (constant) | ❌ | ❌ | — | — | — | ✓ | ≤ 1 frame |

### 11.2.1 Runtime detection

`PointerSource` enum detectado em runtime via:
- iOS: `UIPencilInteractionDelegate` + device model.
- Android: `MotionEvent::getToolType()` + tool device queries.
- Windows: WinAPI device enumeration.
- Linux: `evdev` + udev.
- Web: `PointerEvent` API + heuristics.

### 11.2.2 Graceful degrade

Features not supported (e.g., barrel-roll em devices não-Pencil-Pro) → UI cinza com tooltip "requires Apple Pencil Pro / Wacom Pro 2024+". Sem build conditional (HR-1).

---

## 11.3 Predict+reconcile sub-9 ms loop (ProMotion)

### 11.3.1 Objetivo

ProMotion (iPad 120 Hz adaptive) target latência **≤ 9 ms** end-to-end (Apple documented since iPadOS 13). Para isso:

1. **Prediction**: estimate next sample position from current trajectory.
2. **Speculative render**: render path com predicted samples.
3. **Reconcile**: when actual sample arrives, replace predicted; re-render se discrepância > threshold.

### 11.3.2 Algoritmo

```rust
struct PredictReconcile {
    history: VecDeque<PointerSample>,  // last N samples
    predicted_samples: SmallVec<[PointerSample; 4]>,  // 1-4 frames ahead
}

impl PredictReconcile {
    fn predict_next(&mut self, dt: f32) -> PointerSample {
        // Linear extrapolation OR cubic spline projection
        let recent = &self.history.iter().rev().take(3).collect::<Vec<_>>();
        if recent.len() < 2 {
            return recent[0].clone();  // can't predict
        }
        
        let velocity = (recent[0].pos - recent[1].pos) / (recent[0].timestamp - recent[1].timestamp).as_secs_f32();
        let predicted_pos = recent[0].pos + velocity * dt;
        
        PointerSample {
            pos: predicted_pos,
            pressure: recent[0].pressure,  // hold previous
            tilt: recent[0].tilt,
            // ...
            predicted: true,
        }
    }
    
    fn reconcile(&mut self, actual: PointerSample) {
        // Pop predicted matching actual timestamp.
        // If discrepancy > threshold, mark scene dirty for re-render.
        let discrepancy = (actual.pos - self.predicted_at(actual.timestamp).pos).length();
        if discrepancy > RECONCILE_THRESHOLD_PX {
            scene_mark_dirty();  // re-render speculative path with corrected samples
        }
        
        self.history.push_front(actual);
        if self.history.len() > 10 {
            self.history.pop_back();
        }
    }
}
```

### 11.3.3 ProMotion frame pacing — realista pós-Antigravity L4F2 2ª iteração

**Caveat crítico**: sub-9ms é **alvo ambicioso**. Bevy ECS scheduler + wgpu compositor padrão típico produz 12-16ms latência subjetiva devido a 1-2 frame queue buffer (HN discussions + Procreate / Linearity Curve real measurements confirm). Para fechar o gap:

**Pipeline canônico em dois modos**:

**Modo A — Bevy/wgpu standard (default, sub-12ms)**:
- Frame queue padrão Bevy ECS.
- Latência típica 12-15ms — aceptable mas NOT sub-9ms target.
- Usado em editor mode + non-pencil-critical tools.

**Modo B — Metal Direct Overlay Pass (iPad/Mac M-series, opt-in para tools Pencil-critical)**:
- **Shell-level fast-path** contornando Bevy/wgpu frame queue.
- Pencil preview rendered direto via Metal CAMetalLayer overlay sobre o Vello canvas.
- Predict + render speculative em overlay (sub-2ms latência); Vello compositor full pass async per regular frame.
- Reconcile on actual sample arrival; visual snap to corrected position se discrepância > threshold.
- **Latência típica 7-9ms** — meets target em M-series ProMotion.
- Trade-off: overlay layer adiciona alpha compositing cost (~0.3ms) mas atinge target.

**PlatformHost extension formal — Metal Shared Events sync (Antigravity 3ª iteração L1F5 2026-05-29)**:

Sem coordenação explícita, wgpu surface + CAMetalLayer overlay competem pelo mesmo display sem synchronization → flickering, tearing, race conditions. ADR-0020 (Surface lifecycle) precisa amendment. Solução formalizada:

```rust
// Extension em ph2d-host::PlatformHost
pub trait PlatformHost {
    // ... existing methods ...

    /// Register Metal overlay layer attached to main wgpu surface,
    /// sharing GPU semaphore (Metal Shared Events) for sync.
    /// Returns OverlayHandle to drive from app.
    #[cfg(target_os = "ios")]
    fn register_metal_overlay(
        &mut self,
        config: MetalOverlayConfig,
    ) -> Result<OverlayHandle>;

    #[cfg(target_os = "macos")]
    fn register_metal_overlay_macos(
        &mut self,
        config: MetalOverlayConfig,
    ) -> Result<OverlayHandle>;
}

pub struct MetalOverlayConfig {
    pub pixel_format: PixelFormat,
    pub framebuffer_only: bool,
    pub sync_strategy: SyncStrategy,  // SharedEvent OR Independent
}
```

- **SharedEvent sync** (default): overlay e wgpu compartilham `MTLSharedEvent`; overlay submit aguarda wgpu compositor pass; sem flickering.
- **Independent** (advanced, lower latency risk de tearing): overlay submit independente; mais rápido mas pode glitch ocasional em loadshift.

Documentado em **ADR-0020 amendment** (criar `0020-amendment-1.md` per ADR amendments policy L6F2).

**Decisão policy**:
- Desktop Mac (M-series) + iPad Pro (M-series): Modo B ativo por default em Pen/Pencil tools.
- iPad standard + Wacom desktop: Modo A com predict (sub-12ms).
- Android / Linux / Windows non-touch: Modo A standard (mouse latency limited).
- Web: Modo A only (no direct overlay access in browser).

**Vello GPU pipeline budget original (Mac M-series ideal scenario)**:
- Sensor scan: ~2 ms (hardware).
- OS → app: ~1 ms.
- Predict + scene update: ~0.5 ms (within our budget).
- Metal Direct Overlay render: ~1 ms (Modo B) OR Vello compute compositor ~1.5 ms (Modo A).
- Compositor (iOS): ~2 ms.
- **Total Modo B**: ~6.5-9 ms → meets target.
- **Total Modo A**: ~12-15 ms → aceptable mas NOT sub-9ms.

**Documentado em ADR-0064.**

### 11.3.4 Desktop 60-240 Hz

Wacom + winit:
- 60 Hz: 16 ms budget.
- 144 Hz: 7 ms budget.
- 240 Hz: 4 ms budget.

Predict+reconcile applies em qualquer rate. Em 60 Hz desktop, predict 1 frame ahead suffice; em 240 Hz, predict 2-3 frames ahead para spare margin.

### 11.3.5 Mouse

Mouse não tem prediction necessária (cursor já é current; latency dominated by OS/display). Sample diretamente sem predict.

---

## 11.4 Pressure / Tilt / Azimuth / Barrel curves

### 11.4.1 Per-tool override

Default curves em Vector Preferences (global). Per-tool override em Tool Studio panel (W15):

```rust
pub struct PressureCurve {
    pub control_points: SmallVec<[Vec2; 8]>,  // (input, output) pairs em [0..1]²
    pub interpolation: CurveInterpolation,    // Linear | Cubic | Hermite
}

pub struct TiltCurve {
    pub control_points: SmallVec<[Vec2; 8]>,
    pub axis: TiltAxis,  // X (perpendicular to pen) | Y (parallel)
    pub scale: f32,
}
```

### 11.4.2 Curve editing UI

Tool Studio panel (W15):
- 2D curve editor (X = raw input, Y = mapped output).
- Drag control points; add/remove.
- Default presets (Linear, Soft, Hard, Procreate-Compatible).

### 11.4.3 Application no Hobby fitter / GPU stroke expansion

Curves applied per-sample antes do fitter. GPU stroke expansion consume mapped values.

---

## 11.5 Palm rejection automática

### 11.5.1 iPad

`UIPencilInteractionDelegate` separa Pencil de touch. Quando Pencil em uso, finger touches são rejeitados unless são gestos canvas-level (2-finger undo, etc.).

### 11.5.2 Wacom desktop

Wacom driver expõe stylus events separados de mouse. Vector Module subscribe APENAS a stylus events quando tool é Pen/Pencil.

### 11.5.3 Android S Pen

`MotionEvent::TOOL_TYPE_STYLUS` filter. Finger touch rejected during stylus active.

### 11.5.4 Generic fallback

Quando driver não distingue, heurística:
- Pressure > 0 → stylus.
- Pressure == 0 + small contact area → finger.
- Large contact area (>30 px²) + pressure < 0.1 → palm; reject.

---

## 11.6 Hover preview

### 11.6.1 Devices supported

- iPad M2+ com Apple Pencil 2/Pro: hover detected até ~12 mm.
- Wacom Cintiq Pro: hover detected.
- Surface Studio: hover detected.
- Outros: no hover (graceful degrade — só funciona quando stylus contact canvas).

### 11.6.2 Visual feedback — enriquecido com tilt/pressure envelope (Antigravity L5F3 2ª iteração 2026-05-28)

Quando Pencil hovers sem touch:
- **Pen tool**: preview next vertex position + tangent line preview cubic ANTES do click. Em Spiro Assist mode, mostra clothoid preview.
- **Pencil tool**: **elipse oriented por tilt + azimuth + pressure** (revisado L5F3). Não mais simple width circle — desenha:
  - Eixo maior: predicted width baseado em pressure axis curve atual (`width-profile.pressure_weight × predicted_pressure`).
  - Eixo menor: assimetria baseada em tilt + azimuth (tilt_x → squash X axis, tilt_y → rotate ellipse).
  - Cor: stroke color current com 40% opacity (não 100% — sinaliza "preview").
  - Inner shadow indicativo se hover detecta pencil VERY close to touch (< 3mm) — visual cue "ready to stroke".
- **Eyedropper**: preview color sample under cursor (color swatch flotante + RGB/HSL numeric).
- **Shape tool**: preview shape outline a partir do hover OR start position se mid-drag.
- **Direct Select**: hover sobre vertex highlight + tangent handles visible.

**Render path hover**: dedicated overlay layer (não main scene) para zero impact em frame budget. Atualiza apenas em pointermove event, não per frame.

### 11.6.3 Hover state em PointerEvent

```rust
pub enum PointerKind {
    Pen,
    PenHover,  // proximity sensor active, no contact
    Touch,
    Mouse,
}
```

---

## 11.7 Apple Pencil Pro specific

### 11.7.1 Squeeze

Hardware squeeze button. Default action: open QuickMenu radial. Configurável em Gesture Controls panel.

### 11.7.2 Barrel-roll

Pencil Pro detecta rotation around pen axis (barrel value [0..1]). Default action em Pen tool: rotate tangent magnitude (more roll = longer tangent).

Outros tool usage:
- Pencil tool: barrel doesn't drive width (already pressure-driven); barrel could drive tilt-direction-asymmetry override.
- Shape tool: barrel could rotate shape preview.

### 11.7.3 Haptic feedback

Pencil Pro tem haptic feedback. Vector Module triggers haptic em:
- Snap to grid / guide.
- Close-path detection.
- Vertex add (light tap).
- Boolean cut applied (medium tap).

---

## 11.8 Pencil 2 double-tap

### 11.8.1 Default action

Toggle current tool ↔ Eraser (or last-used opposite tool). Configurável em Gesture Controls.

### 11.8.2 Em Vector Module

- Pen tool ↔ Direct Select (alternativa rápida).
- Pencil tool ↔ Eraser (vector eraser = delete vertices em proximity).
- Configurável per-user.

---

## 11.9 Wacom hover-enabled devices

### 11.9.1 Detection

Wacom Cintiq Pro line + Wacom Intuos Pro Pro have hover. Detect via Wacom driver API.

### 11.9.2 Tablet shortcuts

Wacom ExpressKeys remappable. Default mappings em Vector Module:
- Key 1: Tool toggle (e.g., Pen ↔ Direct Select).
- Key 2: Undo.
- Key 3: Pan (hold).
- Key 4: Zoom (hold).
- Key 5-8: User defined (Tool Studio).

### 11.9.3 Wacom radial menu

Wacom Driver tem radial menu nativo. Não conflito com nosso QuickMenu (vide [12 §12.9](12_ux_chrome.md)).

---

## 11.10 Mouse fallback

### 11.10.1 Pressure simulation

Mouse não tem pressure. Defaults:
- `pressure = 1.0` (max constant) — feels "ink pen", consistent.
- OR `pressure` modulated by speed (slow = high; fast = low) — feels "real pencil". User option.

### 11.10.2 No tilt / azimuth / barrel

Defaulted to zero. Width-profile uses only `pressure_weight` axis; tilt-asymmetric envelope disabled.

### 11.10.3 Hover preview via cursor

Mouse cursor inherently hovers. Hover preview ativo (vide §11.6).

---

## Fim do pencil pipeline

Multi-platform input com Pencil/Wacom/S Pen/Mouse abstracted via `ph2d-input`. Predict+reconcile sub-9 ms ProMotion. Per-tool curves customizable. Palm rejection + hover preview onde supported.

**Next:** [`12_ux_chrome.md`](12_ux_chrome.md) (layout + atalhos + QuickMenu).
