# 06 — Seleção, Transform e Adjustments

## 6.1 Seleção (Selection)

Acesso: tool `S` (em ph2d sigla pra Select; igual ao Procreate "S").

### 6.1.1 4 tipos de seleção

| Tipo | Comportamento | Atalho |
|------|---------------|--------|
| **Automatic** | Magic wand. Tap em pixel → expande área de cor similar via flood fill 8-conectado. | `W` |
| **Freehand** | Lasso. Desenhe path (polilinha com nodes editáveis). Tap-tap-tap forma uma polyline, retorno ao primeiro = fechar. | `L` |
| **Rectangle** | Drag de canto a canto. | `M` |
| **Ellipse** | Drag de canto a canto (oval inscrito no bounding box). | `Shift+M` (alternativa ao Rectangle) |

### 6.1.2 Threshold da Automatic

Após tap inicial, **drag horizontal** ajusta threshold em tempo real (mesmo gesto do ColorDrop §03):
- Esquerda = menor (mais conservador, área menor).
- Direita = maior (mais inclusivo, área maior).
- Barra fina no topo do canvas mostra valor.

ΔE OKLab como métrica de similaridade.

### 6.1.3 Modificadores (Add / Subtract / Intersect / Invert)

Após selection ativa, modificadores via popover ou modifiers de teclado:

| Modificador | Touch (popover) | Atalho |
|-------------|-----------------|--------|
| **Add** | Botão `+` | `Shift+drag` |
| **Subtract** | Botão `−` | `Alt+drag` |
| **Intersect** | Botão `∩` | `Shift+Alt+drag` |
| **Invert** | Botão `⇆` | `Ctrl+Shift+I` |

### 6.1.4 Outras operações na selection

| Operação | Descrição | Atalho |
|----------|-----------|--------|
| **Feather** | Suaviza bordas. Slider 0–32 px. | `Ctrl+Alt+D` opens dialog |
| **Smooth** | Reduz ruído da borda (algoritmo). | — |
| **Expand / Contract** | Cresce/encolhe N px. | — |
| **Copy** | Copia conteúdo selecionado para clipboard. | `Ctrl+C` |
| **Cut** | Copia + apaga. | `Ctrl+X` |
| **Paste** | Cola como nova layer. | `Ctrl+V` |
| **Copy All** | Copia todas as layers visíveis flattenadas. | `Ctrl+Shift+C` |
| **Color Fill** | Fill com active color. | `Alt+Backspace` |
| **Clear** | Apaga area selecionada na layer ativa. | `Delete` |
| **Deselect** | Limpa selection. | `Ctrl+D` |
| **Reselect** | Restaura última selection. | `Ctrl+Shift+D` |
| **Save** | Memoriza selection (slot 1–4). | (via menu) |
| **Load** | Restaura selection memorizada. | (via menu) |
| **Select All** | Selecionar todo o canvas. | `Ctrl+A` |
| **Select Layer Content** | Gera selection do conteúdo opaco da layer ativa. | `Alt+Click layer thumb` |

### 6.1.5 Visual da selection

"Marching ants" (linha tracejada animada em duas cores complementares). Render via Vello em layer overlay (não no canvas commit). Animação: 60Hz, 8px de step.

Toggle visibility: `Ctrl+H` (hide selection marquee, mantém selection ativa — útil para preview).

### 6.1.6 Selection persistence

Per-canvas, gravada no `.ph2d-painter`. **4 slots de save/load** disponíveis.

## 6.2 Transform

Acesso: tool `T` ou Transform pill no top bar.

### 6.2.1 4 modos canônicos

| Modo | Atalho dentro do Transform | Descrição |
|------|---------------------------|-----------|
| **Freeform** | (padrão) | Stretch livre. 8 handles (4 cantos + 4 meios). Não preserva ratio. |
| **Uniform** | `U` ou shift modifier | Preserva aspect ratio (drag any corner = uniform scale). |
| **Distort** | `D` | Corner handles independentes (corner pinning, perspectiva manual). |
| **Warp** | `W` | Mesh deformation. 4×4 grid de handles (interno + externo). |

### 6.2.2 Sub-controles

Visíveis no popover top:

| Control | Descrição |
|---------|-----------|
| **Snapping** | Toggle. Edges + center + content bbox alinham com canvas/outros objetos. |
| **Magnetics** | Toggle. Trava eixos retos (H/V) e ângulos (15°/45°/90°). |
| **Bilinear / Nearest Neighbor** | Interpolation toggle. NN para pixel art (preserva pixels nítidos); Bilinear para imagens lisas. |
| **Flip Horizontal** | `H` (dentro do transform) |
| **Flip Vertical** | `V` |
| **Rotate 45°** | preset button |
| **Rotate 90° CW / CCW** | preset buttons |
| **Fit to Screen** | Auto-resize para caber no canvas |
| **Reset** | Volta ao estado pré-Transform |

### 6.2.3 Warp mode — mesh

Default mesh = 4×4 (16 internal handles + 16 outer edge handles). Toggle "Dense Mesh" → 8×8 (mais controle, mais lento de render).

Algoritmo: piecewise bilinear interpolation por quadrilateral entre os handles. Edge handles dragáveis. Internal handles dragáveis.

### 6.2.4 Confirmation

Transform fica em modo "preview" até confirmation:
- **Enter** (keyboard) ou tap em "Done" no popover = commit.
- **Esc** ou tap em "Cancel" = revert.
- Trocar de tool = auto-commit (warning popup explicativo).

Transform sem selection ativa = transforma a layer ativa inteira.

### 6.2.5 Atalhos durante transform

| Atalho | Ação |
|--------|------|
| `Enter` | Commit |
| `Esc` | Cancel |
| `Shift+drag` | Constrain proportions (mesmo de Uniform) |
| `Alt+drag corner` | Anchor opposite corner |
| `Ctrl+drag corner` | Distort mode local |
| Arrows ↑↓←→ | Move 1px |
| Shift+arrows | Move 10px |

## 6.3 Adjustments

Acesso: tool Adjustments (wand icon) no top bar.

### 6.3.1 Modo Layer vs Modo Pencil (Adjustment Brush)

Ao escolher uma adjustment, popover top mostra toggle:

```
┌──────────────────────────────────┐
│ Hue, Saturation, Brightness      │
├──────────────────────────────────┤
│  [ Layer ]   [ Pencil ]          │
└──────────────────────────────────┘
```

- **Layer mode**: aplica em toda a layer (ou selection). Intensidade via **drag horizontal no canvas** (slider invisível, gestual — esq = sem efeito, dir = max).
- **Pencil mode (Adjustment Brush)**: pinta o efeito com o brush ativo. Stroke acumula intensidade. O cursor mostra sparkle icon indicando modo.

### 6.3.2 12 Adjustments canônicos (v1.0)

Lista enxuta vs Procreate (~16+). Tirados (cortados para v1.0; reavaliar pós-v1.0):
- Glitch (4 sub-modes) — efeito específico, baixa frequência.
- Halftone — caso específico print.
- Chromatic Aberration — efeito stylístico.
- Perspective Blur — uso raro, custo alto.
- Bloom (mantido — comum).

#### Adjustments incluídos

| # | Adjustment | Layer-mode | Pencil-mode | Sub-params |
|---|-----------|-----------|-------------|------------|
| 1 | **Hue, Saturation, Brightness** | ✓ | ✓ | H/S/B sliders |
| 2 | **Color Balance** | ✓ | ✓ | Highlights/Midtones/Shadows sliders por canal |
| 3 | **Curves** | ✓ | ~ (limitado) | RGB + per-channel; 8 control points |
| 4 | **Gradient Map** | ✓ | ✓ | Maps luminance → gradient (custom palette) |
| 5 | **Gaussian Blur** | ✓ | ✓ | Radius |
| 6 | **Motion Blur** | ✓ | ✓ | Angle + distance |
| 7 | **Noise** | ✓ | ✓ | Type (Clouds/Billows/Ridges) + Scale/Octave/Turbulence |
| 8 | **Sharpen** | ✓ | ✓ | Amount + radius |
| 9 | **Bloom** | ✓ | ✓ | Threshold + intensity + size |
| 10 | **Liquify** | — | ✓ | Push/Twirl L/R/Pinch/Expand/Crystals/Edge/Reconstruct |
| 11 | **Clone** | — | ✓ | Source point (Alt+click define) |
| 12 | **Recolor (Replace Color)** | ✓ | ✓ | Source color picker + target color + tolerance ΔE |

> `~` para Pencil mode Curves significa "available but limited" — curves brush não é trivial; Procreate-style permite mas é estranho. Manter por paridade. Default-off na UI primária.

### 6.3.3 Liquify (detalhe)

Modo Pencil-only. Brushes (sub-modes):
- **Push** — empurra pixels na direção do stroke.
- **Twirl Right / Twirl Left** — rotaciona pixels no entorno.
- **Pinch** — comprime pixels para o centro do brush.
- **Expand** — empurra pixels para fora.
- **Crystals** — fragment-like distorção (artístico).
- **Edge** — distorção limitada a bordas detectadas.
- **Reconstruct** — pinta de volta para estado original (eraser de Liquify).

Sub-params: brush size, brush pressure (Apple Pencil), brush momentum (continua agindo brevemente após release).

### 6.3.4 Clone (detalhe)

Pencil-only. Workflow:
1. `Alt+Click` (ou tap-and-hold + "Set Source") define o ponto-fonte na layer ativa.
2. Pinta com o brush ativo — cor "clonada" do source point com offset constante.
3. Source point é destrutivo (não-rastreado; Photoshop Cloning behavior, não Smart Object).

### 6.3.5 Atalho `Ctrl+M` para Curves (Photoshop muscle memory)

Curves é a adjustment mais frequente em workflow profissional. Atalho dedicado.

### 6.3.6 Adjustments aplicáveis a Selection

Quando há selection ativa, adjustments em Layer mode aplicam **apenas dentro da selection**. UI mostra o area "iluminada" com selection marquee visível.

### 6.3.7 Non-destructive considerations

**Não há adjustment layers** (§02 §2.1). Adjustments são destrutivas. Workflow alternativo:
- Duplicate layer (`Ctrl+J`) antes da adjustment como "preview".
- Undo (`Ctrl+Z`) reverte normalmente.
- Snapshot do canvas no stroke history se quiser revert para state pré-adjustment depois.

Justificativa em [12_fora_de_escopo.md](12_fora_de_escopo.md) §12.2.

## 6.4 Move tool

Atalho `G` ou `V`. Comportamento:
- 1-finger drag (Pencil ou touch) = move conteúdo da layer/selection.
- 2-finger drag = continua sendo pan do canvas (não conflita).
- Shift+drag = constrain to axis (H or V whichever has larger initial delta).

Move sem selection = move layer inteira. Move com selection ativa = move só o conteúdo da selection (recorta + cria floating selection até next deselect).

## 6.5 Tool palette no top bar (resumo dos tools)

Ordem canônica do top bar (esquerda → direita), tirado [README.md](README.md) §7:

**Editing Tools (left):**
1. Gallery
2. Actions (wrench)
3. Adjustments (wand)
4. Selection (S)
5. Transform (arrow)

**Painting Tools (right):**
6. Brushes (paint brush)
7. Smudge (finger)
8. Eraser (eraser)
9. Layers (square stack)
10. Color (filled circle, active color thumb)

> Move tool não tem pill — acessado via atalho `G`/`V` ou via Transform tool (single-handle drag dentro do transform popover = move). Decisão deliberada para minimalismo.

## 6.6 Gates de teste

| Gate | Crate | Valida |
|------|-------|--------|
| `selection_rect_drag_correctness` | `ph2d-tool-painter` | Drag corner-to-corner cria rect com pixels esperados |
| `selection_automatic_threshold_drag` | idem | Drag horizontal após tap muda threshold em real-time |
| `selection_add_subtract_intersect` | idem | Cada operação combina selections corretamente |
| `selection_feather_smoothness` | idem | Feather 16px produz transição suave (sem step artifacts) |
| `transform_freeform_8_handles` | idem | Cada um dos 8 handles move corretamente |
| `transform_uniform_constrain_ratio` | idem | Drag corner em Uniform mantém aspect |
| `transform_warp_4x4_mesh_dragging` | idem | Mesh handles dragáveis; piecewise bilinear interpolation correta |
| `transform_nearest_neighbor_pixel_art` | idem | NN scale produz pixels nítidos (sem AA blur) |
| `adjustments_hsb_layer_mode_drag` | idem | Drag horizontal em layer mode altera HSB; commit no release |
| `adjustments_curves_per_channel` | idem | RGB curves independente de per-channel |
| `adjustments_liquify_push_brush` | idem | Push brush stroke desloca pixels corretamente |
| `adjustments_clone_source_offset` | idem | Source set + paint = clone com offset constante |
| `move_with_selection_floating` | idem | Move com selection ativa cria floating selection |

**Continua em:** [07_pencil_pipeline.md](07_pencil_pipeline.md) — pipeline de input Pencil/tablet/mouse.
