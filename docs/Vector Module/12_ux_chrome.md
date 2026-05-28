# 12 — UX Chrome (Layout 4-zonas + atalhos + QuickMenu + multi-plat)

> Spec da **interface visual** do Vector Module. Layout 4-zonas (ADR-0023) — Vector Module ocupa Center 100% canvas + sidebar esquerda + top bar + bottom HUD. Geometry Graph panel docado, Inspector panel, Tool Studio. HUD durante edit (chrome fade). Zen Mode. Atalhos Blender-style desktop primeira-classe. Gestos canvas iPad. QuickMenu radial. Multi-platform chrome behavior.
>
> **ADR ratificador:** ADR-0023 (UI/UX baseline; Vector Module conforma).

## 12.1 Layout 4-zonas (existing em ph2d-editor-core)

### 12.1.1 Zonas

Layout 4-zonas Procreate-inspired ([ADR-0023](../architecture/decisions/0023-ui-ux-baseline.md)) já implementado em `crates/ph2d-editor-core/src/screens/hero/`:
- **TopLeft EDIT zone**: tools + actions.
- **TopRight CREATE zone**: layers + adjustments.
- **Sidebar modulators**: brush size / opacity / undo / redo.
- **Center**: 100% canvas.

Vector Module ocupa o canvas e adiciona pills + panéis docados.

### 12.1.2 Chrome substituição quando Vector Module ativo

Padrão Painter (decisão Enio 2026-05-23 §11 README): "takeover" do canvas chrome. Quando `ActivateTool("vector-*")`:
- TopBar: vector tool pills (Pen / Pencil / Shape / Select / Direct / Knife / Bucket / Symbol / Text-on-Path / Eyedropper) + ações (File / Edit / View / Object / Path / Effect).
- Sidebar esquerda: tool size + tool opacity + tool color + undo / redo (Procreate-style).
- Bottom HUD: zoom + coord + status.

Reverter ao trocar para outra tool (`ActivateTool("painter")` ou `CancelActiveTool`).

---

## 12.1.3 Pathfinder Studio — UX layer Illustrator-style (NEW Antigravity L5F1 2ª iteração 2026-05-28)

**Crítica L5F1**: artistas vetoriais profissionais ex-Illustrator rejeitarão Geometry Graph como UX primária para boolean básico. Décadas de muscle memory esperam **botões clássicos** (Pathfinder palette Illustrator).

**Solução**: **Pathfinder Studio** — painel UX clássico que **silenciosamente insere nodes** no Geometry Graph do layer ativo.

### Layout

```
┌─ Pathfinder Studio ───────────────────────┐
│  Shape Modes:                             │
│  [Union] [Subtract] [Intersect] [Exclude] │
│                                            │
│  Pathfinders:                             │
│  [Divide] [Trim] [Merge] [Crop] [Outline] │
│                                            │
│  Offset:                                  │
│  [Inset]  [Outset]  Size: [____10____]    │
│                                            │
│  ▷ Show Geometry Graph (advanced)         │
└────────────────────────────────────────────┘
```

### Comportamento

1. User seleciona 2 paths no canvas.
2. Click "Union" → Pathfinder Studio insere `vector-boolean(op="Union")` node no Geometry Graph; user vê resultado live no canvas (mesma rendering pipeline).
3. **Geometry Graph permanece oculto** (collapsed) por default; user vê apenas o resultado.
4. Click "▷ Show Geometry Graph" expand panel para edit avançado.

### Benefício

- Persona A (Illustrator pro): UX familiar; **zero friction migration**.
- Persona D (game dev): também pode usar; Pathfinder Studio é layer cosmético sobre o mesmo Geometry Graph poderoso.
- Power user (motion designer): direto pro Geometry Graph para non-destructive node graph.

**Resolução de §5.1 (Filtro minimalista-Blender — um caminho canônico)**: ambos UI surfaces apontam para mesma underlying Geometry Graph. Não é "two paths" — é "one path com UX progressive disclosure".

### 12.1.4 Bidirectional validation (Antigravity 3ª iteração L8F1 2026-05-29)

**Risco**: user clica "Union" no Pathfinder Studio → node inserido no graph. User abre Geometry Graph e edita o node manualmente (e.g., adiciona modifier downstream). Pathfinder Studio UI **não reflete** o estado real do graph. State divergence silent.

**Solução v1.0**: validation observable + visual hint.
- Pathfinder Studio observa o graph; se detecta modifications fora do esperado (e.g., manual vertex edit em path operand, OU downstream modifier afecta result), mostra hint inline:

```
┌─ Pathfinder Studio ──────────────────────────────┐
│  Shape Modes: [Union] [Subtract] [Intersect]    │
│                                                  │
│  ⚠ This boolean has manual modifications.       │
│     Switch to Geometry Graph to edit fully.     │
│     [Open Geometry Graph]                       │
└──────────────────────────────────────────────────┘
```

- Click "Open Geometry Graph" expande panel + focus no node related.
- Pathfinder Studio buttons remain functional para basic ops; user advised but not forced.

**V2.0 stretch**: full bidirectional validation com `Pathfinder Studio abstraction proof` — round-trip Pathfinder operation → graph → Pathfinder rendering deve ser idempotent. Se quebrar (e.g., user adds incompatible modifier), automatic mark "Pathfinder semantics broken; using raw graph".

Gate CI v1.0 `vector_pathfinder_studio_divergence_warn` valida hint surface em divergent scenario.

---

## 12.2 Geometry Graph panel docado

### 12.2.1 Posição default

Painel docado à **direita** do canvas. Flotável via `FloatingPanel` (Procreate-style draggable).

### 12.2.2 Conteúdo

- **Graph visualization**: nodes como rectangles; connections como Bézier curves entre input/output ports.
- **Node placement**: drag-and-drop from sidebar palette OR right-click context menu.
- **Edge drag**: click + drag from output port → input port para connect.
- **Param sliders**: cada node mostra params inline OR click para abrir Inspector panel.

### 12.2.3 Zoom + pan

Graph panel suporta zoom (mouse wheel) e pan (drag empty area).

### 12.2.4 Layout algoritmo

Auto-layout (Sugiyama-style hierarchical layout) quando user inserts node sem position hint. Manual drag preserva position user-defined.

### 12.2.5 Keyboard navigation completo (Antigravity 3ª iteração L5F3 2026-05-29)

Designer sem mouse/stylus precisa **navegação total via teclado** no Geometry Graph + Studios.

**Geometry Graph panel**:
- `Tab` → cycle through nodes em order topológica (source → modifiers → output).
- `Shift+Tab` → reverse.
- `Enter` em selected node → open Inspector (focus moves para Inspector panel).
- `Ctrl/Cmd+Arrow` → move selected node visualmente (10 px increment; `Shift+Arrow` = 1 px fine).
- `Ctrl/Cmd+E` → enter "edge creation mode" (focus on output port of selected node).
- In edge creation mode: `Arrow keys` move target indicator entre input ports de outros nodes; `Enter` confirma connection; `Esc` cancela.
- `Ctrl/Cmd+X` cut node; `Ctrl/Cmd+V` paste; `Ctrl/Cmd+Z` undo.
- `Delete` remove selected node + warning toast "Connection broken" se downstream depended.

**Inspector panel** (per node):
- `Tab` cycle through params (sliders, dropdowns, color pickers).
- Para slider: `Arrow keys` adjust value; `Page Up/Down` = 10× increment; `Home/End` = min/max.
- Para color picker: `Enter` opens picker; `Tab` cycle hue/sat/lightness; `Arrow keys` adjust.

**Visual focus indicator**: 2px ring + glow em focused element (theme-aware OKLCH accent). Conform WCAG 2.2 §2.4.7 focus visible.

**Screen reader hints**: cada keyboard nav action emite AccessKit `Action::Custom` events + descriptive announcement ("Selected node: vector-roughen, amplitude 5.0, frequency 1.0").

Gate CI `vector_a11y_functional_traversal` (L3F3) navega Graph completo só com keyboard; valida 100% nodes reachable.

---

## 12.3 Inspector panel

### 12.3.1 Posição

Docado à **direita** abaixo (ou em tab) do Geometry Graph panel. Largura padrão 320 px.

### 12.3.2 Conteúdo

Mostra params do node OR vertex OR segment selected:

#### Node selected
- Param sliders + checkboxes + dropdowns (typed per `Param` enum).
- Description tooltip.
- "Animate this param" button (cria timeline track).

#### Vertex selected (Direct Select tool)
- `pos` (Vec2 numeric inputs).
- `kind` (Mirror / Aligned / Free / Auto dropdown).
- `tangents` (numeric inputs).

#### Region selected
- `winding` (EvenOdd / NonZero).
- `fill` (FillStyle picker: solid / gradient / mesh / procedural).
- `z_order` (i32).

### 12.3.3 Cor consistente com Painter

Color picker reusado `ph2d-painter-color` (vide [08 §8.6](08_painter_bridge.md)).

---

## 12.4 Tool Studio panel (W15)

### 12.4.1 Conteúdo

Per-tool customization:
- **Pen tool**: tangent magnitude default, click-vs-drag threshold, Spiro tension default, Hyperbezier tension default.
- **Pencil tool**: pressure curve, tilt curve, Hobby weight, sample interval.
- **Shape tool**: defaults per sub-mode (polygon sides, star inner radius, etc.).

Power user customiza; save preset como `.ph2d-vector-tool` (share-able).

---

## 12.5 HUD durante edit

### 12.5.1 Chrome fade

Quando user ativa tool e toca canvas (stylus down OR click), chrome **esmaece para 30% opacity** durante stroke. Result: foco no canvas.

Chrome volta para 100% opacity ao stylus-up + 200 ms delay.

### 12.5.2 Slider floaters

Durante stroke, number do slider mais recente (e.g., width) flutua perto do stylus cursor — não precisa olhar para sidebar.

### 12.5.3 Animação fade

Curve cubic-ease-in-out, duration 150 ms. Sem snap-to-zero (smooth blend).

---

## 12.6 Zen Mode (4-finger / Tab)

### 12.6.1 Trigger

- iPad: 4-finger tap on canvas.
- Desktop: `Tab` key.

### 12.6.2 Behavior

Chrome inteiro hide (TopBar + Sidebar + Bottom HUD + panels). Só canvas + cursor + minimal toast notifications.

### 12.6.3 Exit Zen

- iPad: 4-finger tap novamente.
- Desktop: `Tab` again OR `Esc`.

---

## 12.7 Atalhos Blender-style desktop (FULL TABLE)

Atalhos **primeira-classe** em desktop — paridade Blender + lacuna Illustrator que o doc original criticou.

### 12.7.1 Tool activation

| Key | Tool |
|-----|------|
| `P` | Pen |
| `B` | Pencil (Brush em outras tools) |
| `R` | Shape (Rectangle; Tab cycles shapes) |
| `V` | Select |
| `A` | Direct Select |
| `N` | Knife |
| `K` | Bucket |
| `Y` | Symbol |
| `T` | Text on Path |
| `I` | Eyedropper |

### 12.7.2 Edit operations

| Key | Action |
|-----|--------|
| `Ctrl/Cmd + Z` | Undo |
| `Ctrl/Cmd + Y` (or `Ctrl/Cmd + Shift + Z`) | Redo |
| `Ctrl/Cmd + A` | Select all |
| `Esc` | Deselect all / cancel current |
| `Ctrl/Cmd + C` | Copy |
| `Ctrl/Cmd + V` | Paste |
| `Ctrl/Cmd + X` | Cut |
| `Delete` / `Backspace` | Delete selected |
| `Ctrl/Cmd + D` | Duplicate |
| `Ctrl/Cmd + J` | Join paths |
| `Ctrl/Cmd + G` | Group |
| `Ctrl/Cmd + Shift + G` | Ungroup |

### 12.7.3 View operations

| Key | Action |
|-----|--------|
| `Tab` | Toggle Zen Mode |
| `Space` (hold) | Pan canvas |
| `Z` (hold) | Zoom + drag |
| `1`-`9` | Switch active layer / scene |
| `Ctrl/Cmd + +/-` | Zoom in/out |
| `Ctrl/Cmd + 0` | Fit canvas to view |
| `Ctrl/Cmd + 1` | 100% zoom |

### 12.7.4 Tool-specific

| Key | Tool | Action |
|-----|------|--------|
| `S` | Pen | Toggle Spiro Assist |
| `H` | Pen | Toggle Hyperbezier Assist |
| `[` / `]` | Pencil | Decrement / increment width |
| `Shift + [` / `Shift + ]` | Pencil | Decrement / increment opacity |
| `Enter` | Pen | Close path |
| `Backspace` | Pen | Undo last vertex (mid-path) |
| `Tab` | Direct Select | Cycle through vertices |

### 12.7.5 Panels

| Key | Action |
|-----|--------|
| `F` | Open Fill panel |
| `L` | Open Stroke (Line) panel |
| `Cmd/Ctrl + Shift + G` (taken — adjust to Cmd/Ctrl + ;) | Open Geometry Graph panel |
| `Cmd/Ctrl + Shift + I` | Open Inspector panel |
| `Cmd/Ctrl + Shift + Y` | Open Symbol Library |

### 12.7.6 Customization

Atalhos editáveis em `crates/ph2d-editor-core/src/screens/hero/keybindings.toml` (TOML format). Reset to default available.

---

## 12.8 Gestos canvas iPad (FULL TABLE)

### 12.8.1 Multi-touch gestures

| Gesto | Ação |
|-------|------|
| 1-finger drag | Tool action (depende do tool ativo) |
| 2-finger tap | Undo |
| 3-finger tap | Redo |
| 4-finger tap | Toggle Zen Mode |
| Pinch (2-finger) | Zoom + rotate |
| 2-finger drag | Pan canvas |
| 3-finger swipe down | Cut |
| 3-finger swipe up | Paste |
| 3-finger swipe left | Copy |
| Tap and hold | Eyedropper (sample color from canvas) |
| Draw and hold (1s) | QuickShape detection (Pencil tool) |

### 12.8.2 Pencil-specific

| Gesto | Ação |
|-------|------|
| Pencil 2 double-tap | Toggle Pen ↔ Direct Select (default; configurable) |
| Pencil Pro squeeze | Open QuickMenu |
| Pencil Pro barrel-roll (Pen tool) | Rotate tangent magnitude |

### 12.8.3 Gesture Controls panel

Path: Edit → Preferences → Gesture Controls. User pode remap todos gestos.

---

## 12.9 QuickMenu radial

### 12.9.1 Trigger

- Desktop: `Cmd/Ctrl + Space` (configurável).
- iPad: 2-finger tap-and-hold OR Pencil Pro squeeze.

### 12.9.2 Estrutura

6 slots × 4 menus salváveis = 24 actions accessíveis sem tocar UI.

Default Menu 1 (Tools): Pen, Pencil, Shape, Select, Direct Select, Knife.

Default Menu 2 (Boolean): Union, Subtract, Intersect, Exclude, Divide, Outline.

Default Menu 3 (Modifiers): Roughen, Twist, Mirror, Corner Round, Scatter, Recolor.

Default Menu 4 (Custom): user-defined.

### 12.9.3 Configuração

Via Gesture Controls panel. Drag-and-drop actions para slots.

---

## 12.10 Multi-platform chrome behavior

### 12.10.1 Desktop (Mac / Win / Linux)

- Keyboard shortcuts primeira-classe.
- Tablet input via Wacom driver (hover preview + ExpressKeys).
- Mouse fallback.
- Window chrome native OS (minimize / maximize / close buttons).
- Toolbar floating (Procreate-style) — configurável.

### 12.10.2 iPad / iOS

- Touch + Pencil gestures primeira-classe.
- Hardware keyboard quando attached (Smart Keyboard / Magic Keyboard).
- Chrome fade more aggressive em iPad (canvas é centerpiece).
- Sidebar flippable left/right (canhotos via UI gesture).
- iPadOS Stage Manager integration (window resize fluid).

### 12.10.3 Android

- S Pen gestures + Android system back button.
- Hardware keyboard quando attached.
- Android system bar (navigation buttons).

### 12.10.4 Web

- Pointer Events API (touch / pen / mouse).
- Browser limited (no `Cmd/Ctrl + N` — conflito com browser).
- Use `Alt + N`-style alternatives where conflict.

---

## 12.11 Single source of truth = Widget Gallery (DIRETRIZ §4.2)

Vector Module reusa Widget Gallery canon (`ph2d-panel-widget-gallery`) — TODA UI vector copy pattern from Widget Gallery sem inventar:
- Slider + chip pareados via `link_slider_number`.
- Chip pill via `mark_chip_no_stepper`.
- Real-time preview via `take_params_dirty()` + bridge no shell.
- `apply_event` é forwarder thin (não escrever mirror manual).

Memoria feedback `feedback-hier-companion-dispatch-allowlist` + `feedback-panel-populate-register` aplicam.

---

## 12.12 Painel Painter precedent

Vector Module espelha Painter UX:
- **Painter sidebar Procreate-style** → Vector tools sidebar Procreate-style.
- **Painter Brush Studio** → Vector Tool Studio (W15).
- **Painter Color Panel** → reuso direto (color picker shared).
- **Painter Animation Assist** → Vector Animation panel (W10).

Consistency UX cruzada entre os 2 modules.

---

## Fim do UX chrome spec

Layout 4-zonas + Geometry Graph + Inspector + Tool Studio panels + HUD fade + Zen + atalhos Blender-style + gestos iPad + QuickMenu radial + multi-platform consistency.

**Next:** [`06_animation.md`](06_animation.md) (timeline + state machine details).
