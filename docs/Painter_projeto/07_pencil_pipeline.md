# 07 — Pipeline de input (Pencil, tablet, mouse, touch)

> Latência aqui é experiência. Tudo bem otimizado em outros pontos some pelo ralo se o stroke chega 80ms depois do dedo. O target é Procreate: **~9ms end-to-end em iPad ProMotion**, **≤16ms em desktop @ 120Hz**. Documentado aqui o que precisa pra atingir.

## 7.1 Modelo unificado de input

Tudo entra pelo [`ph2d-input`](../../crates/ph2d-input/) (já existe; abstração pure-data). Pencil/tablet/touch/mouse viram o mesmo `PointerEvent`:

```rust
pub struct PointerEvent {
    pub kind: PointerKind,           // Pencil | Touch | Mouse | Pen (tablet desktop)
    pub source_id: u32,              // id do device (multi-touch / multi-pen)
    pub action: PointerAction,       // Down | Move | Up | Cancel | Hover
    pub position: Vec2,              // x, y em logical pixels
    pub pressure: f32,               // [0, 1]; 1.0 para mouse/touch sem suporte
    pub tilt: f32,                   // [0, π/2]; 0 para devices sem suporte
    pub azimuth: f32,                // [0, 2π]; 0 para devices sem suporte
    pub barrel_roll: f32,            // [0, 2π]; 0 para devices sem suporte
    pub timestamp_ns: u64,           // alta resolução, monotônico
}
```

Shells (desktop winit, iPad SwiftUI, Android Kotlin, Web Pointer Events API) traduzem eventos nativos para este formato e enviam via `host_dispatch_pointer`.

## 7.2 Devices suportados

### 7.2.1 iPad / iOS

| Device | Capability |
|--------|-----------|
| **Apple Pencil 2** | Pressure, tilt, azimuth. Double-tap gesto via `UIPencilInteraction`. Latência 9ms @ ProMotion. |
| **Apple Pencil Pro** (2024+) | Tudo do Pencil 2 + **squeeze** + **barrel roll** (gyro) + **haptic feedback**. |
| **Apple Pencil 1** | Pressure, tilt. Sem azimuth confiável (estimado). Sem double-tap. |
| **Apple Pencil USB-C** (entry) | Pressure on/off (binary), tilt. Sem azimuth. |
| **iPad multi-touch** | 1–5 fingers tracked. Sem pressure (treated as 1.0 quando active). |

### 7.2.2 Desktop

| Device | Capability |
|--------|-----------|
| **Wacom** (Intuos / Cintiq / MobileStudio) | Pressure (1024–8192 levels), tilt, **rotation** (barrel roll para algumas pens — Wacom Art Pen). ExpressKeys + Touch Ring opcional. |
| **Huion** (Inspiroy / Kamvas) | Pressure (8192 levels), tilt. ExpressKeys. |
| **XP-Pen** | Pressure (8192 levels), tilt. ExpressKeys. |
| **Surface Pen** (MS Surface) | Pressure (4096), tilt. Hover preview (Surface displays). |
| **Mouse** | Position only. Pressure = 1.0 always. |
| **Trackpad** (MacBook / external Apple) | Multi-touch gestures. Sem pressure. |

### 7.2.3 Android

| Device | Capability |
|--------|-----------|
| **Samsung S Pen** (Galaxy Tab / Note) | Pressure (4096), tilt. Hover. Button. |
| **Wacom Android tablets** | Igual ao desktop Wacom. |
| **Touch** | Multi-touch (até 10 fingers). |

### 7.2.4 Web

Via [Pointer Events API](https://www.w3.org/TR/pointerevents3/): `pointerdown`, `pointermove`, `pointerup`. Pressure e tilt expostos quando browser+device suportam (Chromium-based + Wacom/Surface = OK; Safari iPad = OK; Firefox tem gaps).

Fallbacks:
- `pointerType === "mouse"` → pressure = 1.0 sempre.
- `pointerType === "touch"` → pressure do dedo (W3C-defined).
- `pointerType === "pen"` → pressure + tilt + tangentialPressure (barrel) + twist (rotation).

## 7.3 Pressure curve

Duas camadas de transformação:

### 7.3.1 Global curve (Painter Preferences)

Curva XY 8 pontos draggable em Painter Preferences → Apple Pencil & Tablet Pressure:

```
Output
  1.0 │ ─────────●●
      │       ●
      │     ●
      │   ●
      │ ●
   0  │●_________________
      0                1.0    Input
```

Aplicada a **todos os pen events** antes de chegar no brush. Defaults: curva identity (linear 0→1).

Use cases:
- Pen que requer pressão pesada → curva côncava (responder mais rápido em pressão baixa).
- Usuária com tendinite quer pintura leve → curva linear deslocada para cima.

### 7.3.2 Per-brush curve (Brush Studio)

Vide [01_brush_engine.md](01_brush_engine.md) §1.3.10. Aplicada **depois** da global curve. Modula targets configuráveis (Size, Opacity, Flow, Bleed).

Pipeline:

```
raw_pressure (device) → global_curve → per_brush_curve(target=Size) → final_size_multiplier
                                     ↓
                                     per_brush_curve(target=Opacity) → final_opacity_multiplier
                                     ↓
                                     ...
```

Cada target tem sua própria curva (compartilha o gráfico XY visual mas armazenado per-target).

## 7.4 Tilt curve

Similar. Global tilt curve em Painter Preferences (raro de usar, mas exposto). Per-brush curves modulam targets (Opacity, Gradation, Bleed, Size, SizeCompression).

Tilt é em radianos `[0, π/2]`:
- `0` = pencil vertical (90° em relação ao display).
- `π/2` = pencil horizontal (0° em relação ao display).

Mais frequente: tilt → grain depth (pencil-style shading), tilt → size compression (chisel brush effect).

## 7.5 Azimuth

Direção do tilt no plano XY do display. Usado para:
- Shape rotation (default em `shape_input_style = Azimuth`).
- Chisel brushes que dependem da direção do "ponta da pena".

Em devices sem azimuth real (Pencil 1, alguns Wacom), estimado da direção do stroke ou fixado em 0.

## 7.6 Barrel roll

Apple Pencil Pro + Wacom Art Pen + alguns S Pen. Detecta a rotação física do barrel via gyro.

Per-brush curve modula targets (Size, Opacity, Bleed). Útil para:
- Flat chisel brush rotacionando como pincel real.
- Calligraphy nib aplicando ângulo dinâmico.

UI: tooltip "Requires Apple Pencil Pro or Wacom Art Pen" greyed em devices sem.

## 7.7 Predictive touch e latency budget

### 7.7.1 Predição

iPadOS expõe `predictedTouches` em `UITouch` API — Apple's predictor extrapola posição N ms à frente. Painter usa:
- **Predictive ON** durante stroke (paint mode) — reduz latência percebida.
- **Predictive OFF** em UI/gizmo mode (precisão > latência).

Em Android, predição similar via Vulkan present timing. Em desktop, mais limitada (frame-pacing do compositor é dominante).

### 7.7.2 Latency budget end-to-end

| Etapa | iPad ProMotion 120Hz | Desktop 120Hz | Desktop 60Hz |
|-------|---------------------|---------------|--------------|
| Pencil/pen sense → OS event | ~1ms | 2–4ms | 2–4ms |
| OS event → shell callback | <1ms | 1–2ms | 1–2ms |
| Shell → core `dispatch_pointer` | <1ms | <1ms | <1ms |
| Core processing (stroke path, stamp scheduling) | ~1ms | 1–2ms | 1–2ms |
| Stamp encode + dispatch compute | ~1ms | 2–3ms | 2–3ms |
| GPU compute (Apple M / RDNA / Adreno) | ~2ms | 2–4ms | 4–8ms |
| Present (swapchain swap) | ~1ms | 1–3ms | 1–3ms |
| Display refresh | ~1ms (120Hz) | ~1ms (120Hz) | ~8ms (60Hz) |
| **Total estimado** | **~9ms** | **10–16ms** | **20–30ms** |

Procreate cita ~9ms (Apple specs). Painter alvo: paridade quando hardware idêntico, +1-3ms quando overhead PH2D (multi-platform abstraction).

### 7.7.3 Frame pacing — predictive resampling

Para chegar no target em 120Hz/240Hz, o stroke path **resampla pontos intermediários entre samples do device**. Devices reportam ~120Hz; canvas roda em 120Hz e o stroke path interpola/extrapola.

```rust
// Pseudo:
let sample_period_ms = device.report_period_ms();  // ~8ms para Pencil
let canvas_period_ms = display.refresh_period_ms(); // ~8.3ms para 120Hz

// Resample para canvas timing
for canvas_frame_t in last_t..now {
    let p = stroke.sample_at(canvas_frame_t);  // bezier interpolation
    emit_stamps_for(p);
}
```

Resampling preserva curvatura (Catmull-Rom spline entre device samples).

## 7.8 Palm rejection

### 7.8.1 Auto-rejection

Quando Pencil está em uso (touch ativo via Pencil tip), Painter automaticamente:
- Ignora touch events de finger no canvas para paint.
- Mantém touch events para multi-touch gestos (2-finger zoom, 4-finger zen).

Implementação: shell detecta o input type do touch event (iPadOS expõe `UITouch.type`), core recebe a info no `PointerEvent.kind`.

### 7.8.2 Pencil-only mode

Toggle global em Gesture Controls. Quando ativo:
- Touch nunca dispara paint (mesmo sem Pencil em uso).
- Touch só dispara gestos multi-touch.
- Útil para usuárias que descansam a mão no canvas com palmas grandes.

### 7.8.3 Desktop palm rejection

Tablets desktop (Wacom Cintiq) podem ter touch-screen capabilities. Painter detecta via input type (`tablet_pen` vs `tablet_touch`) e aplica mesma lógica.

## 7.9 Hover

iPad Pro M2+ + Apple Pencil 2/Pro suportam hover (pencil próximo do display antes do toque). Detalhes em [01_brush_engine.md](01_brush_engine.md) §1.3.10:

- Cursor outline mostra shape + grain + tilt + azimuth do que vai ser pintado.
- `hover_estimated_pressure` toggle: mostra preview de pressão estimada.
- `hover_fill` toggle: fill do shape outline.

Desktop com hover (Wacom hovering Cintiq, mouse hover): mesmas previews. Mouse hover = pressure exibido como 1.0 (mouse vai pintar com pressure 1.0).

## 7.10 Stroke history e undo

### 7.10.1 Modelo

`StrokeHistory` em `ph2d-painter-stroke`. Ring buffer de **250 strokes** (espelha Procreate).

Cada `StrokeRecord`:

```rust
#[derive(Serialize, Deserialize)]
pub struct StrokeRecord {
    pub uuid: Uuid,
    pub timestamp_ns: u64,
    pub brush_handle: BrushHandle,           // referência ao .ph2d-brush
    pub layer_target: LayerId,
    pub primary_color: OklchColor,
    pub secondary_color: OklchColor,
    pub points: Vec<RawPointerSample>,       // input samples antes de stabilization
    pub rng_seed: u64,                       // para replay determinístico
    pub tool_mode: ToolMode,                 // Paint | Smudge | Erase
}
```

### 7.10.2 Undo

Mantém **snapshot por stroke** apenas dos dirty rects + 250 stroke records. Custo:
- Dirty rect snapshot tipicamente 50-200 KB por stroke.
- 250 strokes × 100 KB médio = **~25 MB** RAM (cabe em budget).

Undo restaura snapshot na layer + remove o último stroke record. Redo aplica de volta.

### 7.10.3 Persistence

`StrokeHistory` é **per-document, salva no `.ph2d-painter`** (HR-14 versionado). Reabrir o documento mantém o history (até o limite de 250).

### 7.10.4 Replay determinístico (W11+)

Reaplicação ordenada da stroke list reproduz bit-identical o canvas final. Usa fixed-point coordinates + RNG seeded. Útil para:
- CI test de regressão.
- Replay time-lapse procedural (alternativa ao video capture).
- Compactação de save (stroke list pode ser muito menor que bitmap em alguns casos).

Trade-off: requer `--features det-painter` (vide [01 §1.8.4](01_brush_engine.md)). 3-5× mais lento que GPU.

## 7.11 Multi-pointer

Multi-touch durante stroke ativo:
- 1º Pencil/finger = paint primary.
- 2º finger durante paint = **abre 2-finger gesture mode**, pausa o stroke (sem cancelar) — finger releases reativam o stroke.

Multi-Pencil (cenário possível com >=2 iPads emparelhados, ou collaborative scenarios futuros): **out-of-scope v1.0**. Painter single-pointer no v1.0.

## 7.12 Gates de teste

| Gate | Crate | Valida |
|------|-------|--------|
| `input_pencil_pressure_normalized` | `ph2d-painter-stroke` | Raw pressure → curve → [0, 1] clamped |
| `input_tilt_radians_range` | idem | Tilt sempre em [0, π/2] |
| `input_azimuth_estimated_from_stroke` | idem | Devices sem azimuth real → estimated da direção |
| `input_predictive_resampling_curvature` | idem | Resample preserves curvature (Catmull-Rom golden test) |
| `palm_rejection_pencil_active` | `ph2d-tool-painter` | Pencil active + finger touch → finger ignorado pra paint, mantém pra gesture |
| `palm_rejection_pencil_only_mode` | idem | Pencil-only on + no Pencil + finger touch → ignored pra paint |
| `hover_pressure_estimated_visible` | `ph2d-tool-painter` | Hover ativo + estimated pressure toggle on → cursor preview reflete |
| `stroke_history_ring_250` | `ph2d-painter-stroke` | 251º stroke evicta o 1º |
| `stroke_history_undo_redo` | idem | Undo+Redo produz canvas idêntico ao pre-undo |
| `stroke_history_persist_roundtrip` | idem | Save .ph2d-painter → load → history preservada |
| `stroke_replay_determinism` (W11+) | idem | Replay produz bit-identical em 3 OS (HR-5) |
| `latency_pencil_120hz_under_16ms` | smoke (manual) | Latência E2E < 16ms em desktop @ 120Hz; documentar metodologia |

**Continua em:** [08_performance_memory.md](08_performance_memory.md) — performance e memory.
