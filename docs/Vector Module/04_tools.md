# 04 — Tools (Pen / Pencil / Shape / Select / Direct / Knife / Bucket / Symbol / Text-on-Path / Eyedropper)

> Spec dos **10 tools canônicos** do Vector Module. Cada tool = crate drop-crate fan-out (DIRETRIZ §3.A) com `Tool` impl + manifest + icon + bridge no shell. **Bézier cúbico = default** em Pen tool (decisão D Antigravity); Spiro/Hyperbezier como Assist Modes.
>
> **ADR ratificador:** ADR-0056 (Vector Network data model, includes Pen authoring decision).

## 4.0 Lista canon

| # | Tool | Crate | Wave | Sabor (DIRETRIZ §3.A.3) |
|---|------|-------|------|--------------------------|
| 1 | **Pen** | `ph2d-tool-vector-pen` | W1 | 3 (stateful + panel docado) |
| 2 | **Pencil** (freehand Hobby) | `ph2d-tool-vector-pencil` | W2 | 3 (stateful + panel docado) |
| 3 | **Shape** (rect/ellipse/poly/star/spiral) | `ph2d-tool-vector-shape` | W2 | 2 (stateful) |
| 4 | **Select** (marquee + click) | `ph2d-tool-vector-select` | W2 | 2 (stateful) |
| 5 | **Direct Select** (vertex/tangent) | `ph2d-tool-vector-direct` | W2 | 3 (stateful + Inspector panel) |
| 6 | **Knife** (segment cut) | `ph2d-tool-vector-knife` | W14 | 2 (stateful) |
| 7 | **Bucket** (region fill) | `ph2d-tool-vector-bucket` | W14 | 1 (one-shot stateless) |
| 8 | **Symbol** (Cuttle-parametric) | `ph2d-tool-vector-symbol` | W9 | 3 (stateful + Symbol Library panel) |
| 9 | **Text on Path** | `ph2d-tool-vector-text-on-path` | W14 | 3 (stateful + Text panel) |
| 10 | **Eyedropper** (sample style) | `ph2d-tool-vector-eyedropper` | W2 ou W4 | 1 (one-shot) |

---

## 4.1 Pen tool (`ph2d-tool-vector-pen`)

### 4.1.1 Default behavior — Bézier cúbico

**Resolve crítica D Antigravity.** Comportamento idêntico ao Illustrator / Affinity Pen tool:
- **Click** adiciona vertex com tangentes Zero (corner kind).
- **Click + drag** adiciona vertex com tangentes esticadas (smooth kind).
- **Click vertex existente** ao iniciar = close-path detection.
- **Alt + click on tangent** = break tangent symmetry (corner from smooth).
- **Cmd/Ctrl + click on vertex** = delete vertex.

### 4.1.2 Assist Modes (opt-in toggle HUD)

`S` toggle = Spiro mode. `H` toggle = Hyperbezier mode. Quando ativado:
- Pen tool desenha com 2 control points per segment (Spiro) ou single tension param (Hyper).
- UI mostra preview do model (clothoid spiral / elastica).
- Data model interno guarda dual-representation (cubic + spiro/hyper).
- Toggle de volta para Cubic → keep authoring representation (UI mostra cubic tangentes regenerated).

### 4.1.3 Visual feedback

- Vertex sob mouse: glow azul (PH2D theme accent).
- Tangent handles: linhas tracejadas + endpoints clicáveis.
- Open path: indicador "open" no canvas; close-path hint quando approaching start vertex.

### 4.1.4 Interaction states

```rust
pub enum PenState {
    Idle,                          // sem path em construção
    AwaitingFirstClick,            // ativou tool, esperando primeiro click
    AddingVertex { current_path: VectorNetwork },  // mid-path
    DraggingTangent { vertex: VertexId, anchor: Vec2 },  // click+drag em progresso
    DraggingExistingVertex { vertex: VertexId },  // edit vertex existente
    PreviewClosePath { proximity_to_start: f32 },
}
```

### 4.1.5 Tool Studio panel

Power user customiza:
- Tangent magnitude default (0..2 range).
- Click vs drag threshold (px).
- Auto-smooth strength (0..1).
- Spiro tension default.
- Hyperbezier tension default.

### 4.1.6 Keyboard shortcuts (Blender-style)

- `P` activate Pen tool.
- `S` toggle Spiro Assist.
- `H` toggle Hyperbezier Assist.
- `Esc` finish path (open).
- `Enter` close path.
- `Backspace` undo last vertex.

---

## 4.2 Pencil tool (`ph2d-tool-vector-pencil`)

### 4.2.1 Freehand com Hobby fitter

**Algoritmo**: Hobby's algorithm (minimum curvature variation, John Hobby, MetaPost reference) — fits cubic Bézier through point sequence minimizing curvature variation. Vastly superior to Catmull-Rom or Schneider's (Inkscape default).

### 4.2.2 Pipeline

1. Stylus input: sample N times per second (60 Hz minimum; 240+ em ProMotion).
2. Per-sample: `PointerEvent { pos, pressure, tilt, azimuth, timestamp }`.
3. Buffer samples até stylus-up.
4. Aplicar Hobby fitter para gerar cubic Bézier chain.
5. Apply WidthProfile per sample (pressure → width).
6. Emit `vector.pencil` path no edit log.

### 4.2.3 Predict + reconcile (sub-9 ms ProMotion)

Vide [`11_pencil_pipeline.md §11.3`](11_pencil_pipeline.md). Resumo: predict next sample using linear extrapolation; render speculative; reconcile on actual input arrival.

### 4.2.4 Tool Studio panel

- Pressure curve (per axis): default smooth-ease; user editable.
- Tilt curve: default flat (no effect); user editable.
- Hobby weight (smoothness): default 0.5.
- Sample interval (px between samples): default 2.

### 4.2.5 Shortcuts

- `B` activate Pencil tool.
- `[` `]` decrement / increment width.
- `Shift + [` `Shift + ]` decrement / increment opacity (translates to stroke color alpha).

---

## 4.3 Shape tool (`ph2d-tool-vector-shape`)

### 4.3.1 5 sub-modes

Toggle no panel docado: `Rect` / `Ellipse` / `Polygon` / `Star` / `Spiral`.

### 4.3.2 Interaction

Click + drag → preview shape. Release → commit. Holding `Shift` constrains aspect ratio (square / circle).

Cada shape commit emite `vector-source` primitive (consolidated crate, vide [02 §2.2.1](02_geometry_graph.md)).

### 4.3.3 Params no panel (per shape mode)

- **Polygon**: `sides` (3..64).
- **Star**: `sides` (3..64) + `inner_radius` (0..1).
- **Spiral**: `turns` (1..20).

### 4.3.4 Shortcuts

- `R` activate Shape tool (R por "Rectangle"; Tab cycles modes).

---

## 4.4 Select tool (`ph2d-tool-vector-select`)

### 4.4.1 Selection modes

- **Marquee** (rect drag): seleciona networks fully contained.
- **Lasso** (freehand drag): seleciona networks contained.
- **Click**: seleciona network sob mouse.
- **Shift + click**: add to selection.
- **Cmd/Ctrl + click**: toggle in selection.
- **Cmd/Ctrl + A**: select all.
- **Esc**: deselect all.

### 4.4.2 Selection level

Operates em **network** level (whole VectorNetwork = 1 selection unit). Para vertex-level seleção, use **Direct Select** (§4.5).

### 4.4.3 Shortcuts

- `V` activate Select tool.
- `Cmd/Ctrl + A` select all.
- `Esc` deselect.

---

## 4.5 Direct Select tool (`ph2d-tool-vector-direct`)

### 4.5.1 Vertex/tangent manipulation

- **Click vertex**: seleciona individual vertex.
- **Click tangent handle**: seleciona tangent.
- **Click + drag**: move selecionado.
- **Alt + drag tangent**: break tangent symmetry.
- **Cmd/Ctrl + click + drag tangent**: smooth tangent (mirror restore).

### 4.5.2 Multi-select

Shift + click adiciona; drag rectangle selection multi-vertex.

### 4.5.3 Inspector panel docado

Mostra params do vertex/segment selected:
- `pos` (editable numeric).
- `kind` (Mirror / Aligned / Free / Auto dropdown).
- `tangents` (numeric edit).
- `style` (stroke + fill refs).

### 4.5.4 Shortcuts

- `A` activate Direct Select.
- `Tab` cycle through vertices (next).
- `Shift + Tab` previous vertex.

---

## 4.6 Knife tool (`ph2d-tool-vector-knife`)

### 4.6.1 Algoritmo

Click + drag desenha straight line OR freehand path; segments intersected são cortados em 2 sub-segments (cria intersection vertex). Tangent continuity preserved.

### 4.6.2 Interaction

- Click start point.
- Drag (mouse held).
- Release end point → execute cut.

### 4.6.3 Edge cases

- Cut path não intersecta nenhum segment → no-op.
- Cut path passa por vertex existente → snap to vertex.

### 4.6.4 Shortcuts

- `N` activate Knife tool.

---

## 4.7 Bucket tool (`ph2d-tool-vector-bucket`)

### 4.7.1 Fill region

Click dentro de region → applies current Fill Style (selected no panel) to region.

### 4.7.2 Flood fill

Quando network tem holes (regions disconnected), bucket usa **flood fill em raster proxy** — rasteriza network em low-res mask, flood fill em CPU, vectorize back.

### 4.7.3 Edge case: click fora de region

Cria nova region usando boundary detection automática (closes the smallest enclosing region from click point).

### 4.7.4 Shortcuts

- `K` activate Bucket tool.

---

## 4.8 Symbol tool (`ph2d-tool-vector-symbol`)

### 4.8.1 Cuttle-style parametric symbols

Symbol = parametric VectorNetwork template. Instances bind params via graph.

```rust
pub struct Symbol {
    pub id: SymbolId,
    pub template_network: VectorNetwork,
    pub params: Vec<SymbolParam>,  // typed sliders
    pub bindings: Vec<ParamBinding>,  // param → network mutation
}

pub enum SymbolParam {
    Number { name: String, min: f32, max: f32, default: f32 },
    Color { name: String, default: ColorOklch },
    Enum { name: String, options: Vec<String>, default: String },
    Vector { name: String, default: Vec2 },
}
```

### 4.8.2 Authoring workflow

1. User cria VectorNetwork comum.
2. Click "Convert to Symbol" → opens Symbol authoring mode.
3. User declares params (sliders typed).
4. User binds params para parts of network (e.g., `arms` slider drives `vector-scatter.count`).
5. Save → `.ph2d-vector-symbol` postcard.

### 4.8.3 Instance behavior

Symbol instance no canvas mostra params como sliders (Inspector panel). Edit slider → live update of ALL instances of same symbol.

### 4.8.4 Symbol Library panel

`ph2d-panel-vector-symbol-lib` (W9 T9.2):
- Browse symbols em library (per-project + shared).
- Drag-and-drop pra canvas.
- Search by name / tags.

### 4.8.5 Example: snowflake symbol

```
Params:
- arms: u32 (3..16, default 6)
- roughness: f32 (0..1, default 0.3)
- color: ColorOklch (default white)

Bindings:
- arms → vector-scatter.count
- roughness → vector-roughen.amplitude
- color → fill_color
```

User cria 1 snowflake symbol; spawns 100 instances no canvas; muda `arms` master = todas 100 atualizam.

### 4.8.6 Shortcuts

- `Y` activate Symbol tool.
- `Cmd/Ctrl + Shift + Y` open Symbol Library.

---

## 4.9 Text on Path tool (`ph2d-tool-vector-text-on-path`)

### 4.9.1 Parley + kurbo integration

Reusa `ph2d-text` (parley wrapper) + kurbo `ParamCurveNearest::nearest()` per segment.

### 4.9.2 Workflow

1. Click on existing vector path.
2. Type text.
3. Glyphs distribute along path (parley shapes; kurbo positions per-segment).
4. Variable font axes (vide §8.6 + ADR-0066) editable per-glyph OR per-text-block.

### 4.9.3 Edge cases

- Text longer than path: overflow indicator + option to wrap or truncate.
- Path with sharp corners: glyph spacing adjusts; complex corners may distort glyph.

### 4.9.4 Tool Studio panel

- Font picker.
- Font size.
- Variable font axes sliders (weight / width / slant / optical / GRAD).
- Glyph spacing.
- Vertical alignment to path (above / on / below).

### 4.9.5 Shortcuts

- `T` activate Text on Path tool.

---

## 4.10 Eyedropper tool (`ph2d-tool-vector-eyedropper`)

### 4.10.1 Sample style from path

Click on path → copies its style (stroke + fill + width-profile + etc.) to active style.

### 4.10.2 Tap-and-hold

Para Apple Pencil / Wacom: tap-and-hold (1s default) ativa eyedropper temporariamente (sem trocar tool).

### 4.10.3 Shortcuts

- `I` activate Eyedropper.
- `Alt + click` (em other tools) → temporary eyedropper.

---

## 4.11 Crate consolidation strategy

### 4.11.1 Per-tool crate (10 crates) — decisão

Cada tool merece próprio crate (fan-out drop-crate DIRETRIZ §3.A) **porque**:
- Cada tool tem state complexa + Inspector panel + tool studio params (não-trivial).
- Implementadores paralelos sem colisão (memory `feedback_phase_cascade_2026_05_19`).
- Painter precedent: tools são individuais crates.

### 4.11.2 Exceções de consolidação

- **Shape tool**: 5 sub-modes (rect/ellipse/poly/star/spiral) em **1 crate** com mode-switching. Decisão: tools com sub-modes triviais ficam consolidados; tools com state distintos ficam separados.
- **Select + Direct Select**: 2 crates separados (state semantics distintas; Direct Select tem Inspector panel próprio).

### 4.11.3 Total

10 tool crates (consolidando primitives no `vector-source` node = 1 crate, vide [02 §2.2.1](02_geometry_graph.md)).

---

## 4.12 Bridges no shell — Padrão Wave 10

Cada tool stateful tem bridge no `shells/desktop/src/render_loop/` espelhando `bgremoval_preview.rs`:

```
shells/desktop/src/render_loop/
├── vector_pen_bridge.rs ✨
├── vector_pencil_bridge.rs ✨
├── vector_shape_bridge.rs ✨ (single bridge para 5 sub-modes)
├── vector_select_bridge.rs ✨
├── vector_direct_bridge.rs ✨
├── vector_knife_bridge.rs ✨
├── vector_bucket_bridge.rs ✨
├── vector_symbol_bridge.rs ✨
├── vector_text_on_path_bridge.rs ✨
└── vector_eyedropper_bridge.rs ✨
```

Bridges chamados from frame loop: `refresh_<tool>_preview()` + `commit_<tool>()`.

---

## Fim do tools spec

10 tools canon com Bézier default + Assist Modes opcionais. Shortcuts Blender-style. Inspector + Tool Studio panels docados. Bridges no shell espelhando padrão Wave 10 do Painter.

**Next:** [`11_pencil_pipeline.md`](11_pencil_pipeline.md) (input pipeline detalhe) + [`12_ux_chrome.md`](12_ux_chrome.md) (chrome layout).
