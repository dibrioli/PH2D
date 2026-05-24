# 04 — Canvas, Drawing Guides, QuickShape, Reference Companion

## 4.1 Canvas creation

Dialog ao criar um canvas (acesso: Gallery → "+ New canvas", ou atalho `Ctrl+N`).

### 4.1.1 Layout do dialog

```
┌──────────────────────────────────────────────────┐
│ New Canvas                                       │
├──────────────────────────────────────────────────┤
│ Presets:                                         │
│   ◯ Screen (device-resolution match)             │
│   ◯ Square 2048                                  │
│   ◯ Square 4096                                  │
│   ◯ 4K UHD (3840×2160)                           │
│   ◯ A4 portrait (300 DPI print)                  │
│   ◯ A3 landscape                                 │
│   ● Custom...  ←                                 │
├──────────────────────────────────────────────────┤
│ Custom:                                          │
│   Width:  [ 2048 ]  px / mm / cm / in            │
│   Height: [ 2048 ]                               │
│   DPI:    [ 300 ]                                │
│                                                  │
│ Color profile:                                   │
│   ● sRGB (web-safe, universal)                   │
│   ◯ Display P3 (wide gamut, RGBA16F)             │
│   ◯ CMYK ⚠ (limited; see docs/Painter §3.7)     │
│                                                  │
│ Max layers: 75  (calculated dynamically)         │
│                                                  │
│ Time-lapse:                                      │
│   Quality: Low / Good / Studio / Lossless       │
│   ☑ Enable time-lapse capture                    │
│                                                  │
│ Background:                                      │
│   ☑ White        (otherwise transparent)         │
├──────────────────────────────────────────────────┤
│              [ Cancel ]    [ Create ]            │
└──────────────────────────────────────────────────┘
```

### 4.1.2 Constraints

- **Dimensions cap**: depende de plataforma + memory budget.
  - Desktop: até 16384×16384 (subject a memory).
  - iPad Pro M2/M4: 16384×8192 (paridade Procreate) ou 11585² (max square).
  - Android (top tier S25+/Pixel 9): 8192×8192.
  - Web: 4096×4096 (subject a WASM heap budget).
- **DPI**: 72–600. Não afeta visualização — só metadata para print.
- **Max layers**: calculado conforme [02_layers.md](02_layers.md) §2.5.

### 4.1.3 Restrição de mutação

Color profile e dimensions **não-mutáveis** após criação (espelha Procreate). Para alterar: export → import novo canvas. Razão: mudar profile mid-stroke requereria re-conversão de todas as layers (custosa + lossy); mudar dimensions sem perder dados é UX trap.

Operação "Crop & Resize" oferece **resize destrutivo** (Canvas → Crop) — pixels fora dos novos bounds são deletados.

## 4.2 Drawing Guides

Acesso: Actions → Canvas → Drawing Guide → Edit Drawing Guide.

### 4.2.1 4 tipos canônicos

#### 2D Grid

Grid retangular regular. Params:
- `spacing_px` — distância entre linhas (default 64).
- `thickness_px` — espessura da linha (default 1).
- `color` — token ColorToken (default Bg3 com 30% opacity).
- `subdivisions` — linhas auxiliares finas entre principais (default 0 = off).
- `opacity` — 0..1.

Drawing Assist com 2D Grid: stroke snap a múltiplos de 90° + paralelo às linhas mais próximas. Toggleável.

#### Isometric

Grid 30°/30°/30°. Útil para pixel art e arquitetura técnica.
- `angle_deg` — 30 default; configurável 20°-60° (Iso, dimétrico, trimétrico).
- `spacing_px` — distância entre eixos.
- Demais params iguais ao 2D Grid.

Drawing Assist com Iso: snap aos 3 eixos.

#### Perspective

1, 2, ou 3 vanishing points. Cada ponto draggable diretamente no canvas (modo edit).

- `points` — `Vec<Vec2>` com 1, 2, ou 3 pontos. Defaults: pontos próximos das bordas do canvas.
- `horizon_show` — toggle exibir horizon line entre vps.
- `lines_density` — quantas guidelines por vp (default 24, range 8–96).

Drawing Assist com Perspective: stroke snap em projeções para os vanishing points mais próximos do trajeto.

#### Symmetry

4 sub-tipos + rotational toggle:

- **Vertical** — espelhamento horizontal sobre eixo vertical central.
- **Horizontal** — espelhamento vertical sobre eixo horizontal.
- **Quadrant** — 4 quadrantes (espelhamento V + H).
- **Radial** — N segmentos radiais (default 8; range 2–32). Útil para mandalas.

Toggle adicional: `Rotational Symmetry` — em Vertical/Horizontal, o conteúdo replicado é também rotacionado (não só espelhado).

Drawing Assist com Symmetry: cada stroke é replicado em real-time nos espelhos/segmentos correspondentes. O usuário vê seu único stroke ao desenhar; os outros aparecem automaticamente.

### 4.2.2 Drawing Assist toggle

Independente do guide ativo, **Drawing Assist** é toggle separado (Actions → Canvas → Drawing Assist on/off, atalho `Shift+G`):

- **Off**: guide aparece como overlay visual (referência), mas strokes não snap nem replicam.
- **On**: strokes snap/replicam conforme o guide ativo.

Permite "draw freely with guide visible" vs "draw assisted".

### 4.2.3 Estado do guide

- Apenas **1 guide ativo por canvas** simultaneamente (espelha Procreate).
- Guide params persistem por canvas — salvos em `.ph2d-painter`.
- Toggle visibility separado do toggle Drawing Assist (Canvas → Drawing Guide visible on/off, atalho `Ctrl+'`).

## 4.3 QuickShape

Gesto: desenhe + **hold no fim do stroke** (~0.5s) → o stroke snap em forma "perfeita" detectada automaticamente.

### 4.3.1 Detecção

Algoritmo classifica o stroke desenhado em uma de 6 categorias:

| Detectado | Critério |
|-----------|----------|
| **Line** | Path quase-reto (variação de direção < 5° ao longo) |
| **Arc** | Curvatura aproximadamente constante; path < 180° |
| **Polygon** (polyline) | Path com cantos detectados (curvatura súbita > 60° pontual) sem fechar |
| **Ellipse** | Path fechado; ratio width:height variável; ajuste por mínimos quadrados |
| **Triangle** | 3 cantos detectados em path fechado |
| **Quadrilateral** | 4 cantos detectados em path fechado |

Após snap, aparece **Edit Shape** button no canto top-right do canvas: toca → entra em modo Edit Shape com nodes draggable nas extremidades/vértices.

### 4.3.2 Modificadores durante hold

Com o stroke "hold" ativo (forma snap-ada na tela, dedo ainda no canvas):

- **Segundo dedo no canvas** → fixa proporção (square, circle, equilateral triangle).
- **Terceiro dedo no canvas** (line apenas) → snap a incrementos de 15°.
- **Solte** → commit shape.
- **Tap outro lugar** → cancel, mantém stroke original.

### 4.3.3 Edit Shape mode

Após commit (release), o stroke é commitado mas o **Edit Shape** floating button permanece visível por ~3 segundos. Tocá-lo:
- Mostra nodes draggable (verde) nas extremidades e vértices.
- Drag node = altera shape (re-rasteriza on commit).
- Para ellipse: 4 nodes (4 axes).
- Para polygon: 1 node por vértice.
- Tap fora = sai do Edit mode.

### 4.3.4 QuickShape em desktop

Em desktop, o gesto "hold" funciona com mouse/tablet (sustentar o último ponto antes de soltar). Atalho alternativo: `Shift+drag` força a detecção (Blender-like) — útil quando "hold" sente menos natural com mouse.

## 4.4 Reference Companion

Window flutuante para referência. Acesso: Actions → Canvas → Reference (toggle); atalho `R`.

### 4.4.1 Layout

Window arrastável + redimensionável:

```
┌─────[ ═══ handle ═══ ]─────[ ✕ ]┐
│                                  │
│        ╔═══════════════╗         │
│        ║               ║         │
│        ║  CONTENT      ║         │
│        ║               ║         │
│        ╚═══════════════╝         │
│                                  │
├──────────────────────────────────┤
│   [ Canvas ] [ Image ]           │
└──────────────────────────────────┘
```

- Handle no top = drag para mover.
- Cantos = resize.
- 2 abas: Canvas / Image (sem Face mode no v1.0 — Face é iOS-only via ARKit, mancharia multi-plataforma).

### 4.4.2 Canvas mode

Mini-mirror do canvas principal. Útil para zoom-in trabalhando enquanto vê o whole.
- Auto-updates em real-time conforme pinta.
- Pinch/pan dentro da window funciona independentemente do canvas principal (zoom-in pra detalhe, mantendo o ref window mostrando o whole).

### 4.4.3 Image mode

Importa imagem do Files / Photos / drag-and-drop / Camera.
- Pinch/pan dentro funcionam.
- Tap-and-hold no conteúdo da window com eyedropper ativo = samplea cor da ref image.
- Suportes: PNG, JPEG, WebP, HEIC, TIFF.

### 4.4.4 Z-order

Reference Companion fica **above-canvas, below-popovers**:
- Sempre visível enquanto pinta.
- Color picker popover sobrepõe (quando aberto).
- Brush library sobrepõe.

### 4.4.5 Múltiplas Reference Companion?

**Não no v1.0** (espelha Procreate). 1 window por canvas. Para múltiplas referências, recomendação na docs UX: usar OS-level split-view (iPadOS) ou window managing (desktop).

## 4.5 Snapping & Magnetics (no contexto de Transform — referência)

Detalhe completo em [06_selection_transform_adjustments.md](06_selection_transform_adjustments.md) §6.2. Aqui, breve nota:

Durante Transform (não durante stroke):
- **Snapping**: edges, center, content bbox alinham com edges/centro do canvas e outros objetos.
- **Magnetics**: trava eixos retos (H/V) e incrementos angulares (15°/45°/90°).

Toggle disponível dentro do Transform popover. Configurável em Painter Preferences.

## 4.6 Time-lapse capture

Subsistema separado mas configurado na canvas creation (§4.1.1).

### 4.6.1 Modelo

Capture **frame-based** (video, não procedural). Cada N segundos (ou N ms de pintura ativa), `host` recebe um frame e o encoder MP4/HEVC o adiciona à stream.

**Importante:** capture é off-thread. Encoding feito em thread separada (não bloqueia paint hot path).

### 4.6.2 Qualidades

| Quality | Resolution | Codec | Bitrate alvo | Para |
|---------|------------|-------|---------------|-----|
| **Low** | 720p | H.264 | 4 Mbps | Sharing rápido |
| **Good** | 1080p | H.264 | 12 Mbps | Default |
| **Studio** | 4K (se canvas >= 4K) | HEVC | 30 Mbps | Broadcast |
| **Lossless** | Canvas resolution | HEVC HQ | 60 Mbps | Archival |

Settings configuráveis **apenas na criação do canvas** (espelha Procreate; alterar mid-canvas exigiria re-encode).

### 4.6.3 Export

Actions → Video → Export Time-lapse:
- **Full length** — vídeo completo.
- **30 Seconds** — auto-edit com highlights (algoritmo identifica momentos de pintura ativa, descarta pausas longas, monta clip de ~30s).

## 4.7 Canvas operations (Canvas menu)

Acesso: Actions → Canvas.

| Op | Atalho | Descrição |
|----|--------|-----------|
| Crop & Resize | — | Destrutivo. Crop por seleção retangular + opcionalmente resize. |
| Flip Horizontal | `Shift+H` | Espelha todo o canvas horizontalmente. |
| Flip Vertical | `Shift+V` | Idem vertical. |
| Rotate 90° CW | `Shift+R` | Rotaciona todo o canvas. |
| Rotate 90° CCW | `Shift+L` | — |
| Animation Assist | — | Toggle timeline (§10). |
| Reference | `R` | Toggle ref window (§4.4). |
| Drawing Guide | — | Open guide editor (§4.2). |
| Information | — | Mostra canvas info (size, layers, profile, file size). |
| Canvas Properties | — | Background color, transparency. |

## 4.8 Gates de teste

| Gate | Crate | Valida |
|------|-------|--------|
| `canvas_dimensions_cap_per_platform` | `ph2d-tool-painter` | iPad config caps a 16384×8192; desktop a 16384×16384; web a 4096×4096 |
| `canvas_max_layers_calculation` | idem | Cálculo `max_layers = vram_budget / (w*h*bpp)` matches §2.5 |
| `guide_perspective_vps_draggable` | idem | Mover vp em canvas atualiza guide; persiste no `.ph2d-painter` |
| `guide_symmetry_radial_replication` | idem | Radial N segmentos: stroke aparece em N posições rotacionadas |
| `drawing_assist_snap_to_grid` | idem | Com 2D Grid + assist on, stroke quase-paralelo snap a paralelo perfeito |
| `quickshape_line_detection` | idem | Stroke desenhado quase-reto + hold → snap to line perfeita |
| `quickshape_ellipse_proportion_lock` | idem | Ellipse + 2-finger hold → snap to circle (1:1 ratio) |
| `reference_companion_image_loads` | idem | Importar PNG/JPEG → renderiza na window sem panic |
| `reference_companion_eyedropper` | idem | Sample em ref window → primary color = pixel sampleado |
| `timelapse_studio_quality_4k` | idem | Canvas 4K + Studio quality → encode HEVC válido |
| `canvas_color_profile_immutable` | idem | Tentar alterar profile post-creation falha com erro claro |

**Continua em:** [05_gestos_input.md](05_gestos_input.md) — gestos e input.
