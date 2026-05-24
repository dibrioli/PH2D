# 03 — Cor

## 3.1 Modelo interno: OKLCH

Cor canônica internamente: **OKLCH** (`L` lightness 0..1, `C` chroma 0..0.4, `H` hue 0..2π). Justificativas:

- O resto do PH2D já usa OKLCH em [`ph2d-tokens`](../../crates/ph2d-tokens/) para themes (ADR-0028); cor de tokens e cor de Painter falam o mesmo idioma.
- Perceptualmente uniform — sliders de brightness sentem-se naturais.
- Hue rotation não desatura ao passar pelo azul (problema clássico HSL).
- Conversão direta para OKLab para o stamp pipeline (compute shader recebe OKLab — linear na axis L/a/b, mistura corretamente).

**Compute shader recebe OKLab**, não OKLCH (interpolação correta exige Lab). Painter UI mostra OKLCH; conversão a OKLab é constante e barata.

## 3.2 Color Panel — 5 modos canônicos

Acesso: ícone circular (active color thumb) no canto direito do top bar. Tap abre o Color Panel popover. As 5 abas espelham Procreate na ordem oficial.

### 3.2.1 Disc

Layout:
- **Anel externo**: Hue (gradient 360°, 18px de espessura).
- **Disco interno**: Saturation (radial: centro = 0, borda = max) × Chroma (não-uniform; segue OKLCH chroma cap por hue).
- **Reticle**: ponto pequeno indicando seleção atual.
- Tap em qualquer ponto = nova seleção. Drag dentro do disco = adjustment fino.
- **Mira secundária** mini-comparação: half-circle dividida (esquerda = cor anterior, direita = nova) abaixo do reticle. Permite comparação 1-click revert.

Gestures inline:
- 1-finger drag dentro do disco = ajuste contínuo.
- 2-finger pinch dentro = zoom no disco (zoom 1×–4×).
- 2-finger tap = reset zoom.

### 3.2.2 Classic

Layout tradicional iPad-mobile:
- **Square color picker** (S × V no espaço HSB) — chroma e lightness combinados.
- **Hue slider** vertical à direita (gradient).
- Abaixo, **3 sliders**: H, S, B (numéricos com valor exibido).

Default para usuárias vindas de Photoshop / Procreate Classic.

### 3.2.3 Harmony

Gera sugestões automáticas a partir da cor atual:
- **Complementary** — cor oposta (180° hue).
- **Split Complementary** — 2 cores ao lado da oposta (±30°).
- **Analogous** — 2 cores adjacentes (±30°).
- **Triadic** — 3 cores em triângulo (120°).
- **Tetradic** — 4 cores em quadrado (90°).

UI: disco de hue + dots para a cor atual + os pares/trios derivados. Tap em qualquer dot = troca para essa cor.

Pickerveis adicionais:
- Toggle "Lock harmony" — ao mover a cor atual, as derivadas se movem juntas mantendo a relação.

### 3.2.4 Value

Sliders numéricos para precisão. Entrada por digitação suportada (keyboard quando disponível). 7 sliders:

- **L** (OKLCH Lightness) 0..1.0
- **C** (Chroma) 0..0.4 (clamp depende do hue/lightness — exibe value efetivo se exceder gamut)
- **H** (Hue) 0..360°
- **R** (sRGB Red) 0..255 — read-only display + edit
- **G** (sRGB Green) 0..255
- **B** (sRGB Blue) 0..255
- **Hex** input: `#RRGGBB` ou `#RRGGBBAA` (parser tolerante: `#abc` expande para `#aabbcc`).

Out-of-gamut warning: quando OKLCH escolhido está fora do display gamut (sRGB ou P3 dependendo de profile), badge "out of gamut" + botão "snap to gamut" (clamp para o mais próximo válido).

### 3.2.5 Palettes

Manage swatches.

**Layouts disponíveis:**
- **Cards** (apresentação visual; swatch maior + label opcional).
- **Compact** (grid 8 cols, swatches pequenos sem label).

**Operações:**
- Tap swatch = select cor.
- Long-press swatch = menu (Replace with current, Delete, Rename, Get info).
- Drag swatch = reorder.
- Drag swatch para fora da palette = remove (com confirmação).

**Criar palette:**
- "+ New" → empty palette.
- "From image" → file picker → algoritmo k-means extrai N cores dominantes (default 10).
- "From camera" → live capture (iPad/iPhone; greyed em desktop sem webcam).
- "Import" → `.swatches` (Procreate) ou `.ph2d-palette` (native) ou `.ase` (Adobe).

**Set as Default:** uma palette designada "default" aparece na base de todas as outras 4 abas (Disc/Classic/Harmony/Value) como faixa horizontal de swatches para 1-tap access.

**Sharing/export:** `.ph2d-palette` (postcard) e `.swatches` (Procreate-compatible).

## 3.3 Active color + secondary color

Painter mantém **dois slots** de cor ativa visualmente:

- **Primary color** — usada por pincéis, fill, ColorDrop (visible no thumb do top bar).
- **Secondary color** — usada por Color Dynamics (stamp_secondary_color, color_pressure_secondary). Acessível via gesto ou shortcut `X` (Blender-style).

Quando o usuário samplea via eyedropper, **primary** é trocada. Para alterar secondary: long-press no color thumb → "Set as secondary".

## 3.4 Color History

Lista das **últimas 10 cores usadas** (FIFO; cores duplicadas movem pro topo em vez de duplicar). Visível na base de cada um dos 5 modos do Color Panel.

Cores commitadas a Color History quando:
- Usuária pinta um stroke completo (não durante drag).
- Eyedropper sampleia uma cor distinta.

Não commitadas durante:
- Drag em color picker (uma cor por gesto, commitada no release).

## 3.5 ColorDrop

Gesto: drag do **color thumb** (top right) **para dentro do canvas** → flood fill na região delimitada.

### 3.5.1 Algoritmo

1. Detecta pixel sob o release: `seed_pixel = canvas.read(release_pos)`.
2. Determina **layer alvo**:
   - Se há **reference layer ativa** → usa a reference para geometria; pinta na layer ativa.
   - Caso contrário → usa a layer ativa para geometria E pinta nela.
3. Flood fill 8-conectado com `threshold` ajustável.

### 3.5.2 Threshold gestual

Após o release no canvas, enquanto a usuária **mantém o dedo/Pencil pressionado** (não solta), o gesto entra em modo "ajuste":
- **Arrastar horizontalmente** ajusta threshold (esq = menor = mais conservador; dir = maior = mais inclusivo).
- **Barra fina no topo** do canvas (4px de altura) mostra valor (0% a 100%) em tempo real.
- Preview do fill atualiza por frame.
- Solte = commit fill com o threshold atual.
- Tap fora do canvas durante o gesto = cancel.

Threshold é tolerância de cor em ΔE OKLab (não em RGB). Default opening = 5 (suave-strict).

### 3.5.3 Pencil-aware: continuous color drop

Continuous tap-drag de várias regiões: gesto canvas só vale para o release inicial. Para flood múltiplas regiões em sequência sem reabrir o color picker, alternativa é ativar Fill tool dedicada (futuro pós-v1.0; v1.0 mantém Procreate-style 1-shot).

## 3.6 Eyedropper

### 3.6.1 Gesto canônico (default)

**Tap-and-hold no canvas** por `eyedropper_delay_ms` (default 800ms; configurável em Gesture Controls 0..1500ms). Aparece **loupe magnificada** sobre o cursor mostrando:
- Half superior: nova cor (sob o cursor).
- Half inferior: cor atual (antes do sample).

Drag mantido pelo Pencil/dedo move o sample para explorar. Solte = commit.

### 3.6.2 Outros caminhos (configurable em Gesture Controls — §05)

- **Apple Pencil tap** (se assigned em Gesture Controls)
- **Apple Pencil double-tap** (se assigned em Painter Preferences > Apple Pencil)
- **Apple Pencil Pro squeeze** (se assigned)
- **Modifier square hold** (sidebar) + touch
- **QuickMenu slot** dedicado
- **Keyboard `I`** (desktop) ou `Alt`-modifier durante stroke (Photoshop-style)

### 3.6.3 Eyedropper scope

Por default, samplea a **composição final** (todas as layers visíveis blended). Toggle opcional: "Sample current layer only" (similar PS).

Pode samplear:
- Dentro do canvas principal.
- Dentro da **Reference Companion window** (§04) — explicitamente suportado.

## 3.7 Color management

### 3.7.1 Color profile no canvas

Escolhido na criação do canvas. Não-mutável após criação (espelha Procreate; conversão exige re-export → re-import).

Profiles suportados:
- **sRGB** — default; web-safe, universal.
- **Display P3** — wide gamut (~25% mais cores que sRGB); recomendado em devices com display P3 (iPad Pro 2017+, MacBook Pro 2016+, iPhone 7+, Pixel 5+, modern Samsung). Requer formato RGBA16F (8-bit é insuficiente para P3 perceptually-lossless).
- **CMYK** — listado mas com aviso: *"Print workflow recomendado: trabalhar em sRGB/P3 e converter no fim via ferramenta dedicada (Photoshop, Affinity, etc.)"* — vide §12.3.

### 3.7.2 Detecção de display

`PlatformHost::display_color_profile()` retorna `DisplayProfile { gamut: Srgb | DisplayP3 | Rec2020, bit_depth: 8 | 10 | 12 }`. Painter adapta:
- Out-of-gamut markers nos color pickers.
- Render path com sRGB conversion no presentation pass (não no compute — compute trabalha em linear).

### 3.7.3 Texturas de canvas: RGBA8 vs RGBA16F

Em `sRGB profile`: layers em `Rgba8UnormSrgb` (linear na semantic, gamma na storage).
Em `Display P3 profile`: layers em `Rgba16Float` (linear, wide gamut).

Memory cost diferença: 2× em VRAM para P3. `MemoryBudget` declarado considera o pior caso (P3).

## 3.8 Reference Companion samplável

Window flutuante (§04.5) tem o conteúdo (Canvas mirror ou imagem importada) **samplável** pelo eyedropper. Implementação:
- Janela tem hit-test próprio.
- Eyedropper gesture detecta se cursor está dentro da window e samplea seu conteúdo em vez do canvas.
- Cor sampleada vai para primary slot (mesmo fluxo).

UX importante: o ícone do loupe muda sutilmente (badge "Ref") quando samplando da window, deixando claro pra usuária de onde a cor vem.

## 3.9 Persistência

Color state que persiste entre sessões (gravado em `Painter Preferences`):
- Última primary + secondary color.
- Color History (10 últimas).
- Palette ativa (default + custom).
- Modo do Color Panel (Disc/Classic/etc.) escolhido por último.
- Eyedropper delay configurado.

Por documento (gravado em `.ph2d-painter`):
- Palette docked à esquerda do canvas (se usuária habilitou a Procreate-style "active palette swatches strip").
- Color History local do projeto (separada da global; pode mergedar opcionalmente).

## 3.10 Cores especiais

| Cor | Quando aparece |
|-----|----------------|
| **Transparent (alpha=0)** | "Cor" especial para erase de partes específicas via paint — não exposed como swatch, mas Behind blend mode + alpha=0 dá o efeito. Eraser tool é a forma canônica. |
| **Mask white** | Active color when painting in mask (sempre L=1, C=0, alpha=1). |
| **Mask black** | Active color when painting in mask if Brush is in "invert" mode (shortcut `X` switching primary/secondary). |

## 3.11 Gates de teste

| Gate | Crate | Valida |
|------|-------|--------|
| `color_disc_pick_roundtrip` | `ph2d-painter-color` | Pick → OKLCH → render → sample → match com tolerância ΔE < 1.0 |
| `color_harmony_relationships` | idem | Complementary = +180°; Triadic = 120° × 2; Tetradic = 90° × 3 (em OKLCH hue) |
| `color_palette_format_roundtrip` | idem | Save .ph2d-palette → load → idêntico |
| `color_swatches_import_procreate` | idem | Fixture .swatches Procreate importa sem panic |
| `color_ase_import` | idem | Fixture .ase Adobe importa sem panic |
| `colordrop_threshold_gestual_correctness` | idem | Threshold = 0 → só fill cor exata; threshold = 100 → fill canvas inteiro |
| `colordrop_reference_layer` | idem | Reference layer ativa = geometry vem dela, paint vai pra ativa |
| `eyedropper_in_reference_companion` | idem | Sample em ref window produz cor da imagem ref, não do canvas |
| `gamut_warning_visible_when_out` | idem | OKLCH escolhido fora de sRGB com sRGB profile → badge "out of gamut" |

**Continua em:** [04_canvas_guides.md](04_canvas_guides.md) — canvas creation, drawing guides, QuickShape.
