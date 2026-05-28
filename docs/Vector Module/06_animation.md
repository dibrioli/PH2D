# 06 — Animation (timeline + state machine + Houdini-style param animation)

> Spec do **sistema de animação** do Vector Module. **Toda param do graph é animável** (Houdini paradigm). State machine Rive-style (states + transitions + blending). Timeline panel + curve editor. Onion skin para frame-based animations. Export GIF / APNG / MP4 / Lottie subset.
>
> **ADRs ratificadores:** ADR-0063 (Vector runtime + state machine) + ADR-0066 (Variable fonts axes — also animable).

## 6.1 Toda param animável (Houdini paradigm)

### 6.1.1 Princípio

**Qualquer numeric param do graph é potencialmente animável**. User clica "Animate" button próximo ao param → cria timeline track para esse param.

Aplica a:
- Every node param em geometry graph (e.g., `vector-roughen.amplitude`).
- Every fill graph param (e.g., `noise.frequency`).
- Every tool param (e.g., `pen.tangent_magnitude` — raramente animado).
- Variable font axes (vide [`14 §14.7`](14_inovacoes_extraordinarias.md) §8.6).
- VectorNetwork vertex positions (raro; for ad-hoc keyframe animation).

### 6.1.2 Animation = curve per param

```rust
pub struct AnimatedParam {
    pub param_path: NodeParamPath,  // (node_id, param_name)
    pub curve: AnimationCurve,
    pub time_range: (f32, f32),     // start, end (seconds)
}

pub struct AnimationCurve {
    pub keyframes: SmallVec<[Keyframe; 8]>,
    pub interpolation: CurveInterpolation,
}

pub struct Keyframe {
    pub time: f32,
    pub value: ParamValue,
    pub in_tangent: Vec2,   // for Bézier interpolation
    pub out_tangent: Vec2,
    pub easing: EasingKind, // Linear | Cubic | Bounce | Spring
}
```

---

## 6.2 Curve editor (per param)

### 6.2.1 UI

Bottom panel (collapsible) mostra timeline. Tracks listed per animated param. Click track → opens curve editor inline.

Curve editor:
- Horizontal axis: time (seconds).
- Vertical axis: param value (range auto-fit OR user-set).
- Keyframes as nodes; tangent handles editable.
- Right-click → add keyframe at cursor time OR delete keyframe.

### 6.2.2 Tangent kinds

- **Linear**: straight interpolation between keyframes.
- **Bézier**: tangent handles per keyframe (Aligned / Free / Mirror — same kinds as VectorNetwork vertex).
- **Hermite**: tangent direction-only (magnitude derived from neighbors).
- **Step**: hold value (no interpolation).

### 6.2.3 Easing presets

Per-keyframe easing curves:
- Linear, Cubic-ease-in, Cubic-ease-out, Cubic-ease-in-out.
- Spring (stiffness + damping).
- Bounce (with bounce count + decay).
- Custom (Bézier curve).

### 6.2.4 Multi-select

Shift + click selects multiple keyframes. Drag moves all. Box select via marquee.

---

## 6.3 State machine (Rive-style)

### 6.3.1 Conceito

**Estado** = preset de params (per-node-param override). **Transition** entre estados com blend (linear / ease / spring). Triggers via ECS events ou Luau calls.

Detalhe em [`10_runtime_gameplay.md §10.2`](10_runtime_gameplay.md). Resumo:

```rust
pub struct StateMachine {
    states: HashMap<StateId, State>,
    transitions: Vec<Transition>,
    current_state: StateId,
}

pub struct State {
    id: StateId,
    params: HashMap<NodeParamPath, ParamValue>,
}

pub struct Transition {
    from: StateId,
    to: StateId,
    trigger: Option<TriggerId>,
    condition: Option<Condition>,
    blend_duration: Duration,
    blend_curve: BlendCurve,
}
```

### 6.3.2 Authoring UI

State Machine panel docado:
- Visual state diagram (nodes = states; arrows = transitions).
- Click state → set as "current" + edit params.
- Click transition → set blend curve + trigger.

### 6.3.3 Smoke W10

User cria 3 states (idle / hover / press) + 4 transitions. Click "Preview" → live state cycle no canvas com blend visible.

---

## 6.4 Timeline panel

### 6.4.1 Posição

Bottom panel (collapsible). Default height 200 px.

### 6.4.2 Conteúdo

- **Timeline ruler**: time markers (seconds / frames).
- **Playback controls**: Play / Pause / Stop / Loop toggle.
- **Frame rate selector**: 12 / 24 / 30 / 60 fps.
- **Tracks**: animated params listed; expandable to show curves.
- **Time cursor**: vertical line at current time; draggable.

### 6.4.3 Frame-based vs continuous

- **Continuous mode**: animation curves smooth interpolation; play em arbitrary fps.
- **Frame-based mode**: vector network discrete frames (paridade Painter Animation Assist); each frame = separate VectorNetwork. Used para hand-drawn animation style.

User toggles via panel header.

---

## 6.5 Onion skin (vector frame-by-frame)

### 6.5.1 Frame-based mode

Quando frame-based mode ativo:
- Each frame = own VectorNetwork in layer stack (sequenciais, e.g., `frame_0001.vector`, `frame_0002.vector`).
- Compositor mostra current frame opaque.

### 6.5.2 Onion skin behind

- Previous frame (frame-1): 30% opacity, blue tint.
- Frame-2: 15% opacity, blue tint.
- Frame-3: 7% opacity.
- (configurable count + opacity + color em panel).

### 6.5.3 Onion skin ahead

- Next frame (frame+1): 30% opacity, red tint.
- Frame+2: 15%, red tint.
- Configurable.

### 6.5.4 Performance

Onion skin re-renders apenas onion frames quando current frame changes. Cached per frame.

---

## 6.6 Export GIF / APNG / MP4 / Lottie subset

### 6.6.1 Format support

| Format | Wave | Quality | Use case |
|--------|------|---------|----------|
| GIF | W19 | Lossy, 256 colors | Web previews; small |
| APNG | W19 | Lossless | Web high-quality animated |
| MP4 (H.264) | W19 | Lossy compressed | Long animations |
| MP4 (H.265) | W19 | Lossy compressed efficient | Replace H.264 onde supported |
| Animated WebP | W19 | Lossy/lossless options | Modern web |
| Lottie subset | W19 | Vector animation | After Effects compatibility |
| `.ph2d-vector-anim` | W19 | Postcard binary native | PH2D ecosystem |

### 6.6.2 Lottie subset

Export to Lottie (JSON-based animation format). Subset:
- Paths + transforms + opacity + masks (covered).
- Keyframe interpolation curves (covered).
- Variable fonts axes (covered via Lottie "tn" track).
- **Not supported v1.0**: shape gradients animáveis (Lottie spec limitation; export bake gradient first frame OR rasterize).

Goal: Lottie output runs em After Effects básico (via Bodymovin plugin).

### 6.6.3 `.ph2d-vector-anim` native

Postcard binário versionado (HR-14). Schema:

```rust
pub struct Ph2dVectorAnim {
    pub version: u32,
    pub asset: Ph2dVectorAsset,  // base
    pub animated_params: Vec<AnimatedParam>,
    pub state_machine: Option<StateMachine>,
    pub duration: Duration,
    pub frame_rate: f32,
    pub frame_based: bool,  // true = frame-based mode (onion skin)
    pub frames: Option<Vec<Ph2dVectorAsset>>,  // present if frame_based
}
```

---

## 6.7 Variable fonts axes animation hook (§14.7 + ADR-0066)

### 6.7.1 Axis como AnimatedParam

```rust
let animated_axis = AnimatedParam {
    param_path: NodeParamPath::new("vector-text-on-path.axes.weight"),
    curve: AnimationCurve {
        keyframes: smallvec![
            Keyframe { time: 0.0, value: ParamValue::F32(100.0), ... },
            Keyframe { time: 2.0, value: ParamValue::F32(900.0), ... },
        ],
        interpolation: CurveInterpolation::Cubic,
    },
    time_range: (0.0, 2.0),
};
```

### 6.7.2 Result

Glyph deforma de weight 100 → 900 em 2 segundos, com smooth cubic interpolation. Renderiza via skrifa + Vello sem rasterizar fonte intermediária.

### 6.7.3 Use cases

- Logo intro animation (weight pulses with music).
- HUD que mostra urgency (numbers get bolder as urgency increases).
- Letterform morphs por proximidade do mouse (radial falloff driving slant).

---

## 6.7-bis Accessibility — semantic timeline readout (Antigravity 3ª iteração L5F1 2026-05-29)

Timeline + curve editor são visualmente dominantes. Screen reader users precisam **representação semântica** de animation behavior.

**AccessKit Node tree** para timeline:
- Root node: `role: Timeline`, `label: "Animation timeline, duration 2 seconds, 60 fps"`.
- Per track: `role: Track`, `label: "vector-roughen.amplitude"`, `description: "starts at 0.0 at 0s, peaks 1.0 at 1s via cubic ease-in-out, returns to 0.5 at 2s"`.
- Per keyframe: `role: Keyframe`, `value: f64 time + AnimValue`, `description: auto-gen "keyframe at 1.0 second, value 1.0"`.
- State machine: `role: StateGraph`, `current_state: "hover"`, `available_transitions: ["press", "release"]`.

**Auto-description generator** (`crates/ph2d-vector-a11y/src/timeline_describer.rs`):
- Curve shape detection: `cubic-ease-in-out`, `bounce`, `linear`, `step`, `spring(stiff=...)`.
- Peak detection: identifies max/min values, formats as "peaks at Xs with value Y".
- Anomaly detection: "discontinuous", "extremely steep slope", "constant".

Curve editor keyboard nav:
- `Tab` cycle through keyframes.
- `Arrow keys` adjust selected keyframe time (`Shift+Arrow` = value).
- `Enter` accept; `Esc` cancel.

Gate CI `vector_a11y_functional_traversal` (L3F3) inclui timeline traversal smoke.

---

## 6.8 Determinismo

### 6.8.1 Animation playback bit-identical

Quando `deterministic=true`:
- Time advance fixed-step (no wall-clock).
- Curve evaluation deterministic (no FMA in shader).
- State machine transitions sub-frame accuracy preserved.

### 6.8.2 Gate CI

`tests/determinism/vector_animation_replay.rs` — fixture animation com 5 sec replay produces same hash cross-OS.

---

## Fim do animation spec

Toda param animável (Houdini paradigm) + state machine Rive-style + timeline + onion skin + variable fonts axes + export to all major formats.

**Next:** [`07_motion_integration.md`](07_motion_integration.md) (motion nodes cross-domain integration).
