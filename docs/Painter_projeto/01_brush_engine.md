# 01 — Motor de pincel (Brush Engine)

> **Ponto crucial.** O motor de pincel é o que faz o Painter ter "sabor Procreate" ou não. Tudo nesta seção é must-have antes de declarar W1 fechado.

## 1.1 Modelo conceitual: dual-texture Shape × Grain

O motor é **stamp-based dual-texture**. Cada pincel é definido por duas texturas:

- **Shape** — silhueta/envelope do stamp (e.g., círculo macio, oval com taper, traço de pena, splash de tinta). Tipicamente PNG grayscale 256×256 ou 512×512, com alpha contribuindo para o "feathering" da borda.
- **Grain** — textura interna que preenche a Shape (e.g., granulação de papel, carvão poroso, ruído de spray). Tipicamente PNG grayscale 1024×1024 ou maior, tilável.

Visualmente, o stamp final é:

```
stamp_alpha(uv) = shape_alpha(uv) * grain_value(grain_uv) * brush_opacity * pressure_curve(p)
stamp_color(uv) = paint_color * color_dynamics(stamp_idx, pressure, tilt)
```

O `grain_uv` é derivado do `uv` do shape modulado pelo **Grain Behavior** (Moving vs Texturized — §1.3.7).

O stroke é uma sequência de stamps carimbados ao longo do path do input, com espaçamento, jitter e count controlados pelos parâmetros de §1.3.

**Por que Shape×Grain importa:** este modelo separa **silhueta** de **textura**, permitindo combinatória enorme com poucas texturas. 30 Shapes × 30 Grains = 900 brushes únicos, e cada um pode ser ajustado independentemente sem afetar os outros. É o que dá o estilo Procreate (mídia tradicional simulada) com tamanho de asset razoável.

## 1.2 Stroke pipeline (stamp-based, não ribbon-based)

```
Input event (PointerEvent {x, y, p, tilt})
    │
    ▼
StrokePath.advance() — calcula próximo ponto interpolado
    │  • aplica Streamline (suavização)
    │  • aplica Stabilization (média móvel)
    │  • aplica Motion Filtering (remove jitter de extremidade)
    │  • aplica Taper (size/opacity curve nas extremidades)
    │  • interpola N pontos entre samples se necessário
    │
    ▼
StampScheduler.emit_stamps(point, pressure, tilt) → Vec<Stamp>
    │  • aplica Spacing (distância entre stamps)
    │  • aplica Jitter (deslocamento perpendicular)
    │  • aplica Scatter (rotação aleatória por stamp)
    │  • aplica Count (multi-stamp por ponto)
    │  • aplica Color Dynamics (jitter de cor por stamp)
    │
    ▼
StampPipeline.encode(stamps) — compute shader em ph2d-gpu
    │  • upload de stamps[] como storage buffer (struct Stamp em §1.4)
    │  • compute shader lê Shape texture + Grain texture
    │  • blend dentro do layer texture conforme Rendering mode (§1.3.10)
    │
    ▼
Layer texture atualizada
    │
    ▼
Composição final (presentation): layer texture + outros layers → swapchain
```

**Stamp = unidade atômica** do brush engine. Hot path (HR-3): nenhuma alocação dentro de `emit_stamps` nem dentro do encode do compute. Pool de stamps pré-alocado (capacity = `max_stamps_per_frame = 4096` no v1.0); ring buffer; overflow vira flush forçado + warning.

## 1.3 Parâmetros canônicos do brush (Brush Studio)

A organização canônica do Painter espelha a do Brush Studio do Procreate — porque é boa, e porque facilita import. Cada seção abaixo é um sub-panel do Brush Studio (§1.7).

### 1.3.1 Stroke Path

Distribui stamps ao longo do path.

| Param | Tipo | Range | Default | Descrição |
|-------|------|-------|---------|-----------|
| `spacing` | f32 | 0.01..=1.0 | 0.10 | Distância entre stamps em frações do diameter do brush. 0.10 = 10 stamps por diameter. |
| `spacing_jitter` | f32 | 0.0..=1.0 | 0.0 | Variação aleatória do espaçamento (uniform em `[-jitter, +jitter]`). |
| `jitter_lateral` | f32 | 0.0..=1.0 | 0.0 | Desloca stamps perpendicular ao stroke direction (em frações do diameter). |
| `falloff` | f32 | 0.0..=1.0 | 0.0 | Fade-out de opacity ao longo do stroke (1.0 = stroke desvanece completamente até o fim). |

### 1.3.2 Stabilization

Suaviza o input.

| Param | Tipo | Range | Default | Descrição |
|-------|------|-------|---------|-----------|
| `streamline_amount` | f32 | 0.0..=1.0 | 0.0 | "Lazy mouse" — desloca cursor de painting atrás do input real (similar a Clip Studio Stabilizer). |
| `streamline_pressure` | f32 | 0.0..=1.0 | 0.0 | Prolonga aplicação suave de pressão (taper interno). |
| `stabilization` | f32 | 0.0..=1.0 | 0.0 | Média móvel sobre últimos N pontos (N = `1 + stabilization * 16`). |
| `motion_filtering_amount` | f32 | 0.0..=1.0 | 0.0 | Remove extremidades de oscilação (algoritmo dedicado). |
| `motion_filtering_expression` | f32 | 0.0..=1.0 | 0.5 | Reinjeta expressividade depois do filtering (compensa over-smoothing). |

### 1.3.3 Taper

Estreitamento de início/fim do stroke. Dividido em **Pressure Taper** (Apple Pencil / tablet pen) e **Touch Taper** (finger).

| Param | Tipo | Range | Default | Descrição |
|-------|------|-------|---------|-----------|
| `taper_size_start` | f32 | 0.0..=1.0 | 0.0 | Tip size no início (fração do max size). |
| `taper_size_end` | f32 | 0.0..=1.0 | 0.0 | Tip size no fim. |
| `taper_length_start` | f32 | 0.0..=0.5 | 0.0 | Comprimento (frações do stroke total) com taper aplicado no início. |
| `taper_length_end` | f32 | 0.0..=0.5 | 0.0 | Idem no fim. |
| `taper_opacity_start` | f32 | 0.0..=1.0 | 0.0 | Opacity multiplier no tip start. |
| `taper_opacity_end` | f32 | 0.0..=1.0 | 0.0 | Idem fim. |
| `taper_pressure_link` | bool | — | true | Se true, taper usa pressure curve em vez dos valores literais (sinal natural de início/fim). |
| `taper_link_sizes` | bool | — | true | Se true, `taper_size_end` espelha `taper_size_start` (e idem opacity). |

### 1.3.4 Shape

Fonte e comportamento da Shape.

| Param | Tipo | Range / Valores | Default | Descrição |
|-------|------|------------|---------|-----------|
| `shape_source` | `ShapeSource` | `Builtin(name)` ou `Imported(handle)` | `Builtin("round_hard")` | Identifica a textura Shape. Built-ins em §1.6.1. |
| `shape_input_style` | enum | `TouchOnly` / `Azimuth` / `AzimuthBarrelRoll` | `TouchOnly` | Como Pencil tilt e barrel-roll afetam a Shape. |
| `shape_rotation_follow` | bool | — | false | Shape rotaciona seguindo a direção do stroke. |
| `shape_scatter` | f32 | 0.0..=360.0 | 0.0 | Range de rotação aleatória por stamp em graus. |
| `shape_count` | u32 | 1..=16 | 1 | Múltiplos stamps por ponto (espalhados conforme scatter). |
| `shape_count_jitter` | f32 | 0.0..=1.0 | 0.0 | Variação aleatória do count. |
| `shape_randomized` | bool | — | false | Rotação aleatória ao iniciar stroke (não por stamp). |
| `shape_flip_x` | bool | — | false | Flip horizontal da Shape. |
| `shape_flip_y` | bool | — | false | Flip vertical. |
| `shape_roundness` | f32 | 0.1..=1.0 | 1.0 | Compressão horizontal (1.0 = circle, 0.5 = oval 2:1). |
| `shape_pressure_roundness` | f32 | 0.0..=1.0 | 0.0 | Pressão comprime/expande a Shape. |
| `shape_tilt_roundness` | f32 | 0.0..=1.0 | 0.0 | Tilt comprime conforme ângulo. |
| `shape_vertical_jitter` | f32 | 0.0..=1.0 | 0.0 | Jitter na roundness vertical. |
| `shape_horizontal_jitter` | f32 | 0.0..=1.0 | 0.0 | Idem horizontal. |
| `shape_filtering` | enum | `None` / `Classic` / `Improved` | `Improved` | Antialiasing do stamp (Improved é bilinear high-quality em compute). |

### 1.3.5 Grain

Fonte e comportamento da Grain.

| Param | Tipo | Range / Valores | Default | Descrição |
|-------|------|------------|---------|-----------|
| `grain_source` | `GrainSource` | `Builtin(name)` ou `Imported(handle)` ou `None` | `None` | Identifica a textura Grain. Built-ins em §1.6.2. |
| `grain_behavior` | enum | `Moving` / `Texturized` | `Texturized` | Moving = grain segue stroke (smeary); Texturized = grain estática "atrás" do stroke (papel). |
| `grain_movement` | f32 | 0.0..=1.0 | 0.0 | Mistura entre os dois extremos. |
| `grain_scale` | f32 | 0.05..=4.0 | 1.0 | Tamanho da Grain dentro da Shape. |
| `grain_zoom` | enum | `Cropped` / `FollowSize` | `FollowSize` | Cropped = scale fixo independente do brush size; FollowSize = escala com brush size. |
| `grain_rotation` | f32 | 0.0..=360.0 | 0.0 | Rotação da Grain em graus. |
| `grain_depth` | f32 | 0.0..=1.0 | 1.0 | Contraste/força da Grain (0.0 = sem grain, 1.0 = full). |
| `grain_depth_min` | f32 | 0.0..=1.0 | 0.0 | Pisos mínimo da grain depth quando pressure baixa. |
| `grain_depth_jitter` | f32 | 0.0..=1.0 | 0.0 | Jitter de depth por stamp. |
| `grain_offset_jitter` | f32 | 0.0..=1.0 | 0.0 | Jitter de posição da Grain por stamp. |
| `grain_blend_mode` | enum | `Multiply` / `Linear Burn` / `Overlay` / ... | `Multiply` | Blend entre Grain e Shape (lista canônica de 8 modes em §1.5). |
| `grain_brightness` | f32 | -1.0..=1.0 | 0.0 | Ajuste pré-blend. |
| `grain_contrast` | f32 | 0.0..=2.0 | 1.0 | Idem. |
| `grain_filtering` | enum | `None` / `Classic` / `Improved` | `Improved` | Filter da Grain texture (idem Shape). |

### 1.3.6 Rendering

Modos de blending intrínsecos do brush (independentes do blend mode da layer). Lista canônica abaixo. Para detalhe técnico de cada modo, ver §1.5.2.

| Param | Tipo | Range / Valores | Default | Descrição |
|-------|------|------------|---------|-----------|
| `rendering_mode` | enum | `LightGlaze` / `UniformGlaze` / `IntenseGlaze` / `HeavyGlaze` / `UniformBlending` / `IntenseBlending` | `LightGlaze` | Modo de composição do stamp na layer (lookup table em §1.5.2). |
| `flow` | f32 | 0.0..=1.0 | 1.0 | Multiplica o stamp alpha antes de blend. Diferente de opacity (que multiplica o stroke completo). |
| `wet_edges` | bool | — | false | Acumula tinta nas bordas (efeito aquarela). |
| `burnt_edges` | bool | — | false | Escurece bordas (efeito de carvão queimado / sumi-e seco). |
| `burnt_edges_mode` | enum | `Subtle` / `Normal` / `Heavy` | `Normal` | — |
| `stroke_blend_mode` | enum | (lista §02 layers) | `Normal` | Blend mode do stroke inteiro contra a layer existente. |
| `luminance_blending` | bool | — | false | Habilita blending baseado em luminance (útil para layers de luz). |
| `alpha_threshold` | f32 | 0.0..=1.0 | 0.0 | Cap mínimo de alpha — abaixo disso o pixel não escreve. |

### 1.3.7 Wet Mix (simulação de mídia úmida)

Subsistema próprio. Ativo se `wet_mix_enabled = true`; caso contrário todos os params abaixo são ignorados (zero custo no shader).

| Param | Tipo | Range | Default | Descrição |
|-------|------|-------|---------|-----------|
| `wet_mix_enabled` | bool | — | false | Master toggle. |
| `dilution` | f32 | 0.0..=1.0 | 0.0 | Quanto de água mistura com a tinta. Alta = transparente. |
| `load` | f32 | 0.0..=1.0 | 1.0 | Tinta carregada no início do stroke. |
| `attack` | f32 | 0.0..=1.0 | 0.5 | Taxa de deposição (alta = deposita mais rápido). |
| `pull` | f32 | 0.0..=1.0 | 0.0 | Força com que o brush puxa pintura já no canvas. |
| `grade` | f32 | 0.0..=1.0 | 0.5 | Espessura/contraste de textura úmida. |
| `blur` | f32 | 0.0..=1.0 | 0.0 | Gaussian blur aplicado ao stamp antes de blend. |
| `blur_jitter` | f32 | 0.0..=1.0 | 0.0 | Jitter de blur por stamp. |
| `wetness_jitter` | f32 | 0.0..=1.0 | 0.0 | Jitter de dilution por stamp. |

### 1.3.8 Color Dynamics

Variação de cor dentro do pincel. Aplicada **antes** do stamp ir pro compute shader (em CPU side).

**Stamp-level jitter** (desvio aleatório por carimbo):

| Param | Tipo | Range | Default | Descrição |
|-------|------|-------|---------|-----------|
| `stamp_hue_jitter` | f32 | 0.0..=1.0 | 0.0 | Desvio aleatório de hue em frações de 360°. |
| `stamp_saturation_jitter` | f32 | 0.0..=1.0 | 0.0 | Idem saturation. |
| `stamp_lightness_jitter` | f32 | 0.0..=1.0 | 0.0 | Idem lightness. |
| `stamp_darkness_jitter` | f32 | 0.0..=1.0 | 0.0 | Desvio apenas para baixo. |
| `stamp_secondary_color` | bool | — | false | Mistura aleatória com `secondary_color` (slot 2 do color picker). |
| `stamp_secondary_amount` | f32 | 0.0..=1.0 | 0.0 | Quantidade da mistura. |

**Stroke-level jitter** (desvio aleatório por stroke inteiro, fixado quando começa):

Mesmos parâmetros prefixados `stroke_*`.

**Pressure modulation**:

| Param | Tipo | Range | Default | Descrição |
|-------|------|-------|---------|-----------|
| `color_pressure_hue` | f32 | -1.0..=1.0 | 0.0 | Pressão modula hue. |
| `color_pressure_saturation` | f32 | -1.0..=1.0 | 0.0 | — |
| `color_pressure_brightness` | f32 | -1.0..=1.0 | 0.0 | — |
| `color_pressure_secondary` | f32 | 0.0..=1.0 | 0.0 | Pressão modula mistura com secondary. |

**Tilt modulation**: idem prefixed `color_tilt_*`. **Barrel roll modulation**: idem `color_barrel_*` (Pencil Pro / Wacom barrel sensors apenas).

### 1.3.9 Dynamics (modulação por velocidade do stroke)

| Param | Tipo | Range | Default | Descrição |
|-------|------|-------|---------|-----------|
| `speed_size` | f32 | -1.0..=1.0 | 0.0 | Velocidade modula size. -1.0 = stroke rápido = menor; +1.0 = stroke rápido = maior. |
| `speed_opacity` | f32 | -1.0..=1.0 | 0.0 | Idem opacity. |
| `speed_spacing` | f32 | -1.0..=1.0 | 0.0 | Idem spacing (útil para spray brushes). |
| `jitter_size` | f32 | 0.0..=1.0 | 0.0 | Jitter puramente aleatório de size por stamp. |
| `jitter_opacity` | f32 | 0.0..=1.0 | 0.0 | Idem opacity. |

### 1.3.10 Apple Pencil / Tablet Pen

Curves de input **per-brush** (global curve fica em Painter Preferences — §07).

| Param | Tipo | Range | Default | Descrição |
|-------|------|-------|---------|-----------|
| `pressure_curve` | `Curve` | 8 control points | identity | Curva XY pressão input → output normalizado [0,1]. |
| `tilt_curve` | `Curve` | 8 control points | identity | Idem tilt (em radianos, 0 = pencil vertical, π/2 = horizontal). |
| `barrel_roll_curve` | `Curve` | 8 control points | identity | Idem barrel roll (radianos). |
| `pressure_targets` | bitmask | { Size, Opacity, Flow, Bleed } | { Size, Opacity } | Quais atributos a pressão modula. |
| `tilt_targets` | bitmask | { Size, Opacity, Gradation, Bleed, SizeCompression } | { } | — |
| `barrel_targets` | bitmask | { Size, Opacity, Bleed } | { } | — |
| `cursor_outline` | enum | `None` / `Contrast` / `ActiveColor` | `Contrast` | Estilo do cursor. |
| `hover_estimated_pressure` | bool | — | true | Mostra preview de pressão estimada no hover (M2+, Wacom hover). |
| `hover_fill` | bool | — | false | Fill do shape no hover (vs apenas outline). |

### 1.3.11 Properties

Meta-config do brush.

| Param | Tipo | Range | Default | Descrição |
|-------|------|-------|---------|-----------|
| `max_size_px` | f32 | 1.0..=2048.0 | 256.0 | Cap superior do brush size slider. |
| `min_size_px` | f32 | 0.5..=1024.0 | 1.0 | Cap inferior. |
| `max_opacity` | f32 | 0.0..=1.0 | 1.0 | Cap superior do opacity slider. |
| `min_opacity` | f32 | 0.0..=1.0 | 0.0 | — |
| `smudge_pull` | f32 | 0.0..=1.0 | 0.5 | Quanto puxa quando usado em modo Smudge. |
| `orient_to_screen` | bool | — | false | Shape rotation independe do device rotation. |

### 1.3.12 About (metadata)

| Param | Tipo | Default | Descrição |
|-------|------|---------|-----------|
| `name` | String | "Untitled Brush" | Nome do brush. |
| `author` | String | "" | Author. |
| `signature_uri` | Option\<String\> | None | Asset handle de signature image. |
| `created_at` | DateTime\<Utc\> | now | — |
| `reset_points` | Vec\<BrushSnapshot\> | empty | Save points para revert no Brush Studio. |

## 1.4 Stamp struct (canônico para GPU)

A `Stamp` é a struct enviada do CPU para o compute shader. Layout binário fixo (HR-3 zero-alloc; HR-5 determinismo opt-in onde aplicável).

```rust
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Stamp {
    // Posição e geometria
    pub position_world: Vec2,       // x, y in world space (pixels do canvas)
    pub size_px: f32,               // diameter em pixels
    pub rotation_rad: f32,          // rotação do stamp

    // Pressão / tilt resolvidos
    pub pressure: f32,              // [0, 1] após pressure_curve
    pub tilt: f32,                  // [0, π/2] após tilt_curve
    pub azimuth: f32,               // [0, 2π]
    pub barrel_roll: f32,           // [0, 2π], 0 se device não suporta

    // Cor resolvida (com color dynamics aplicado)
    pub color_oklab: [f32; 4],      // OKLab (não OKLCH — Lab é o linear que compute pode misturar)

    // Modulação
    pub opacity: f32,               // [0, 1] após flow + taper opacity
    pub flow: f32,                  // [0, 1] flow do brush
    pub wet_amount: f32,            // [0, 1] dilution * wet_load (se wet_mix_enabled)

    // Shape/Grain UVs (do atlas — texture arrays do brush atlas em §1.8)
    pub shape_layer: u32,           // índice no shape atlas
    pub grain_layer: u32,           // 0xFFFFFFFF se sem grain
    pub grain_offset_uv: Vec2,      // offset da grain dentro do shape
    pub grain_scale: f32,           // scale da grain

    // Behavior flags (bitmask)
    pub flags: u32,                 // bit 0: shape_flip_x, bit 1: shape_flip_y,
                                    // bit 2: grain_behavior_moving, bit 3: burnt_edges,
                                    // bit 4: wet_edges, bit 5: luminance_blending,
                                    // ...

    // Rendering mode (enum como u32)
    pub rendering_mode: u32,        // 0..=5 conforme §1.5.2

    // Padding para alinhar 16
    pub _pad: [f32; 1],
}
```

**Tamanho:** 96 bytes alinhados em 16. Pool default: 4096 stamps × 96 B = **384 KB** por frame de stamp buffer. Cabe em VRAM com folga (orçamento §08).

## 1.5 Modos de blend e Rendering

### 1.5.1 Grain blend modes (entre Shape e Grain, antes do stamp)

8 modos canônicos:

1. **Multiply** (default) — `grain * shape`. Resultado escurece onde grain escura.
2. **Linear Burn** — `grain + shape - 1`. Forte escurecimento.
3. **Overlay** — combina multiply (escuros) com screen (claros).
4. **Soft Light** — Photoshop soft light, gentle.
5. **Hard Light** — overlay invertido.
6. **Screen** — `1 - (1-grain)(1-shape)`. Claro.
7. **Add** — `grain + shape`. Brilha.
8. **Subtract** — `shape - grain`. Pode usar para "carvão raspado".

### 1.5.2 Rendering modes (composição do stamp na layer)

Os 6 modos canônicos, em ordem de "leve → pesado". Cada um é uma função `(stamp, layer) → new_layer` no compute shader.

Notação: `s` = stamp color (premultiplied), `α_s` = stamp alpha (após shape × grain × flow × pressure × taper), `L` = layer color (premultiplied), `α_L` = layer alpha.

| Modo | Fórmula (Porter-Duff style + tweaks) | Caracter |
|------|--------------------------------------|----------|
| **Light Glaze** | `new = s * α_s + L * (1 - α_s * 0.6)` — partial preserva layer abaixo | Aquarela leve, sobreposições mantêm transparência |
| **Uniform Glaze** | `new = s * α_s + L * (1 - α_s)` — Porter-Duff "over" puro | Photoshop "Normal" tradicional |
| **Intense Glaze** | `new = s * α_s^0.5 + L * (1 - α_s^0.5)` — alpha curve agressiva | Cobre mais rápido, ainda permite sobreposição |
| **Heavy Glaze** | `new = clamp(s * α_s + L * (1 - α_s * 0.85), 0, 1)` + saturate | Cor pura e sólida, traços marcantes |
| **Uniform Blending** | mistura linear contínua: `new = mix(L, s, α_s)` + accumulator | Mídia úmida: óleo/acrílico, ainda misturando |
| **Intense Blending** | `Uniform Blending` + smudge factor (pull = wet_mix.pull) | Wet Mix forte; tinta puxa o que tem embaixo |

**Detalhe técnico:** os modos não são meros blend modes — eles têm comportamento de **alpha accumulation** diferente. Glaze modes acumulam alpha de forma "harmonious" (sobrepor 2× não duplica), Blending modes mixam continuamente. Implementação fina no shader em §1.8.3.

### 1.5.3 Stroke blend modes (do stroke contra a layer, configurável)

Lista canônica de 22 modos em [`02_layers.md`](02_layers.md) §2.2. O Painter brush expõe os mesmos via `stroke_blend_mode`. Default `Normal`.

## 1.6 Default brush library

12 brushes em 6 categorias. Filtro Blender-minimalismo: cada categoria tem **exatamente 2 brushes** (um "core" workhorse + um "expressive" com sabor). Strings em i18n (chave `painter.brush.<id>`).

### 1.6.1 Pencils

1. **`pencil_2b`** — Shape `round_hard`, Grain `paper_subtle`, low spacing, tilt → grain depth alto, pressure → opacity. Workhorse de sketch.
2. **`pencil_charcoal`** — Shape `oval_soft`, Grain `charcoal_heavy`, tilt → size compression, scatter pequeno. Expressivo, gestual.

### 1.6.2 Inks

3. **`ink_studio_pen`** — Shape `round_hard`, no grain, full opacity, taper pressure-linked agressivo. Para line art.
4. **`ink_brush_pen`** — Shape `tapered_oval`, no grain, pressure → size dramático (range 0.1x→1.0x), streamline 0.4. Caligráfico.

### 1.6.3 Markers

5. **`marker_fine`** — Shape `round_hard`, Grain `marker_streak`, low flow, intense blending. Acumula em sobreposição.
6. **`marker_chisel`** — Shape `flat_chisel`, shape_rotation_follow=false, orient_to_screen=true. Mantém ângulo do traço.

### 1.6.4 Paints (mídia opaca, óleo/acrílico)

7. **`oil_round`** — Shape `round_hard`, Grain `canvas_weave`, wet_mix_enabled, dilution 0.3, pull 0.6. Mistura no canvas.
8. **`oil_bristle`** — Shape `bristle_spread`, Grain `canvas_weave`, count 4, scatter 30°, jitter_lateral 0.15. Vai como pincel chato espalhando.

### 1.6.5 Watercolors

9. **`watercolor_wash`** — Shape `round_soft`, Grain `watercolor_paper`, wet_edges=true, dilution 0.8, light_glaze. Lavagem clássica.
10. **`watercolor_detail`** — Shape `round_soft_small`, Grain `watercolor_paper`, wet_edges=true, dilution 0.4. Detalhe wet-on-dry.

### 1.6.6 Airbrushes

11. **`airbrush_soft`** — Shape `round_gradient_soft`, no grain, low flow, speed_opacity=-0.3 (rápido = menos tinta). Render rápido.
12. **`airbrush_textured`** — Shape `round_gradient_soft`, Grain `spray_grain`, low flow, grain_offset_jitter 0.5. Spray paint look.

### 1.6.7 Shape source assets (built-in)

10 shapes shipados:

`round_hard`, `round_soft`, `round_gradient_soft`, `round_soft_small`, `oval_soft`, `tapered_oval`, `flat_chisel`, `bristle_spread`, `splatter_spread`, `square_hard`.

### 1.6.8 Grain source assets (built-in)

8 grains shipados:

`paper_subtle`, `charcoal_heavy`, `marker_streak`, `canvas_weave`, `watercolor_paper`, `spray_grain`, `noise_white`, `noise_pink`.

## 1.7 Brush Studio (UI de edição)

Painel docado, abertura via:
- Tap-and-hold em um brush thumb na Brush Library
- Atalho `Ctrl+B` (desktop) / Actions → Brushes → Edit (mobile)
- MCP `painter_brush_edit(brush_id, token)`

Layout (vertical scroll, agrupado por seções §1.3.1-1.3.12):

```
┌───────────────────────────────────┐
│ < Stroke preview                  │  ← live preview com brush atual
├───────────────────────────────────┤
│ [Stroke Path]      ▼              │  ← collapsible sections
│   spacing       ────●─────        │
│   spacing_jit   ─●───────         │
│   jitter        ─●───────         │
│   falloff       ●─────────        │
├───────────────────────────────────┤
│ [Stabilization]    ▼              │
│ [Taper]            ▶              │  ← collapsed
│ [Shape]            ▶              │
│ [Grain]            ▶              │
│ [Rendering]        ▶              │
│ [Wet Mix]          ▶              │
│ [Color Dynamics]   ▶              │
│ [Dynamics]         ▶              │
│ [Apple Pencil]     ▶              │
│ [Properties]       ▶              │
│ [About]            ▶              │
├───────────────────────────────────┤
│ [Reset to defaults]  [Save as...] │
└───────────────────────────────────┘
```

**HR-18 enforcement:** Brush Studio decomposto em sub-módulos. `crates/ph2d-panel-painter/src/brush_studio/{stroke_path, stabilization, taper, shape, grain, rendering, wet_mix, color_dynamics, dynamics, pencil, properties, about}.rs` — cada arquivo ≤ 600 LOC, cada `paint_*` fn ≤ 200 LOC.

**Live preview:** o stroke preview no topo re-renderiza a cada mudança de slider (debounce 16ms). Usa um stroke fixed (S-curve) com pressure ramp 0→1→0 e tilt steady 45°. Cabe no budget — ~0.5ms para repaint do preview.

## 1.8 Pipeline GPU (compute shader)

### 1.8.1 Atlas de Shape e Grain

Para evitar bind/unbind de texture per-stamp, todas as Shape textures vão num **texture array 2D** (`wgpu::TextureViewDimension::D2Array`), 256×256 layers de R8Unorm. Idem Grain (1024×1024 layers).

- Shape atlas: 64 layers × 256×256 × R8 = **4 MB**.
- Grain atlas: 64 layers × 1024×1024 × R8 = **64 MB**.

Brushes built-in usam layers fixed (`round_hard = 0`, etc.). Imported brushes upload em layer livre; se atlas cheio, evicta LRU.

### 1.8.2 Workgroup model

Compute shader `stamp.wgsl`:

```wgsl
// Workgroup: 8×8 threads (64 threads/stamp).
// Cada workgroup processa 1 stamp.
// Dispatch: dispatch(stamp_count, 1, 1) — 1 workgroup por stamp.

@compute @workgroup_size(8, 8, 1)
fn cs_stamp(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let stamp = stamps[wid.x];
    let pixel = compute_pixel_for_stamp(stamp, lid.xy);
    if (pixel.in_bounds) {
        let s = sample_shape_grain(stamp, pixel.local_uv);
        let layer_color = textureLoad(layer_in, pixel.coord, 0);
        let new_color = apply_rendering_mode(stamp.rendering_mode, s, layer_color);
        textureStore(layer_out, pixel.coord, new_color);
    }
}
```

Stamps que cobrem mais de 8×8 px (tipicamente brush size > 8) precisam de múltiplos workgroups — dispatch escala com `ceil(stamp_size / 8)²`. Para brush size 256, isso é 32×32 = 1024 workgroups por stamp. Ainda dentro do budget para ~16 stamps/frame de brushes grandes (~16k workgroups/frame, OK para Apple M1+ / RDNA1+).

### 1.8.3 Rendering mode dispatch

**Decisão estrutural (2026-05-23, fechada em [README §11](README.md) #2):** W1 implementa **unified**; padrão ouro final é **especializado**. ADR-0042 ratificará a transição.

#### W1 — unified (rampa)

Um único compute shader com `switch (stamp.rendering_mode)` selecionando função. Branching **uniforme** (todas as 64 threads de um workgroup processam o mesmo stamp = mesmo modo) — GPU lida bem; ~5-10% perf hit vs especializado.

```wgsl
@compute @workgroup_size(8, 8, 1)
fn cs_stamp(@builtin(workgroup_id) wid: vec3<u32>,
            @builtin(local_invocation_id) lid: vec3<u32>) {
    let stamp = stamps[wid.x];
    let pixel = compute_pixel_for_stamp(stamp, lid.xy);
    if (!pixel.in_bounds) { return; }
    let s = sample_shape_grain(stamp, pixel.local_uv);
    let layer_color = textureLoad(layer_in, pixel.coord, 0);
    var new_color: vec4<f32>;
    switch (stamp.rendering_mode) {
        case 0u: { new_color = light_glaze(s, layer_color); }
        case 1u: { new_color = uniform_glaze(s, layer_color); }
        case 2u: { new_color = intense_glaze(s, layer_color); }
        case 3u: { new_color = heavy_glaze(s, layer_color); }
        case 4u: { new_color = uniform_blending(s, layer_color); }
        case 5u: { new_color = intense_blending(s, layer_color); }
        default: { new_color = s; }
    }
    textureStore(layer_out, pixel.coord, new_color);
}
```

#### W5+ — especializado (padrão ouro final)

6 shaders, um por Rendering mode. Stamps pré-agrupados por modo no CPU side. Dispatch um shader por modo. Sem branching no shader; mais rápido em entry-level. Procreate Valkyrie faz isso.

#### Caminho do refactor (não-destrutivo)

1. CPU `StampScheduler` ganha pre-grouping: `stamps_by_mode: [Vec<Stamp>; 6]` (vetores reutilizados frame a frame; HR-3 mantido).
2. `StampPipeline::encode()` itera modos não-vazios e despacha o pipeline correspondente.
3. API pública `StampPipeline` inalterada.
4. Test golden por modo continua o mesmo (output bit-identical se fórmula mantida).
5. ADR-0042 amendment registra a transição.

#### Gate `painter_stamp_specialize_when_budget_pressure`

Roda em CI baseline (Apple M2 / RDNA1). Mede headroom no sub-budget Painter ([08 §8.1.1](08_performance_memory.md)). Se headroom < 15% em 3 runs consecutivos, falha — sinal para o ciclo de especialização entrar no roadmap.

### 1.8.4 Determinismo (opt-in, W11+)

Para replay determinístico, o compute shader é trocado por implementação CPU (fallback `--features det-painter`) que processa stamps em ordem fixa. Resultado bit-identical cross-platform. Custo: ~3-5× lento. Usado em CI replay test + em jogos que querem grava replays do Painter (raro, mas spec sustenta).

Stamps determinísticos requerem:
- `position_world` em fixed-point Q16.16 (não f32) durante replay.
- `pressure`, `tilt`, etc. em Q8.8.
- `rng_seed` por stroke (gravada no `.ph2d-painter`).
- Sem `fast-math`. Sem FMA. Sem GPU compute (no det mode).

## 1.9 Brush format (`.ph2d-brush`)

Native postcard binário (HR-6, blake3-addressed). Schema versionado (HR-14).

```rust
#[derive(Serialize, Deserialize)]
pub struct PainterBrushFileV1 {
    pub version: u32,                       // = 1
    pub uuid: Uuid,                         // identity estável
    pub blake3_self: [u8; 32],              // self-hash do payload (excluding este campo)
    pub brush: Brush,                       // todos os params §1.3
    pub shape_source_data: Option<Bytes>,   // PNG bytes se Imported
    pub grain_source_data: Option<Bytes>,   // PNG bytes se Imported
    pub metadata: BrushMetadata,            // §1.3.12
}
```

Tamanho típico: 4-12 KB sem shape/grain importadas; 200-500 KB com texturas inline.

### 1.9.1 Brushsets

`.ph2d-brushset` = `Vec<PainterBrushFileV1>` + manifest:

```rust
#[derive(Serialize, Deserialize)]
pub struct BrushsetFileV1 {
    pub version: u32,                       // = 1
    pub uuid: Uuid,
    pub name: String,                       // e.g., "My Inks Collection"
    pub category_hint: Option<String>,      // hint pra UI agrupar
    pub brushes: Vec<PainterBrushFileV1>,
    pub cover_thumbnail: Option<Bytes>,     // PNG preview
}
```

### 1.9.2 Procreate `.brush` importer (lossy)

Implementado em `ph2d-painter-brush::import_procreate`. Lê o ZIP do `.brush` Procreate, extrai o plist/JSON de parâmetros + as PNGs de Shape e Grain. Mapeia 1:1 onde possível, registra warnings onde Procreate-specific (e.g., 3D Materials) é ignorado.

Output: log estruturado por brush listando parâmetros perdidos.

```
[WARN] painter.import.procreate: brush "My Wet Inkr" — `materials_metallic` ignored (no 3D)
[WARN] painter.import.procreate: brush "Pencil Studio" — `apple_pencil_squeeze_action` ignored (PH2D usa Gesture Controls global)
```

Importar Procreate `.brushset` produz `.ph2d-brushset` 1:1 (subject às mesmas perdas).

## 1.10 Smudge tool e Eraser

Smudge e Eraser são **modos** do brush engine, não brushes separados. O brush ativo continua sendo o atual; o tool define o modo:

- **Paint mode** (default): stamp deposita cor.
- **Smudge mode**: stamp puxa cor existente no canvas, modulada por `smudge_pull`.
- **Erase mode**: stamp subtrai alpha da layer, com mesma Shape×Grain (eraser texturizado).

Implementação: o Rendering mode no compute shader é overridden conforme `tool_mode`. Mantém zero-alloc, mantém o sabor (eraser herda Shape do brush ativo = "eraser do mesmo formato").

UI: tool switching via top-bar (Brushes / Smudge / Eraser pills) ou atalhos B / S / E.

## 1.11 Gates de teste

| Gate | Crate | O que valida |
|------|-------|--------------|
| `painter_no_alloc_hot_path` | `ph2d-painter-brush` | HR-3: 0 allocs em `StampPipeline::emit_and_encode()` durante 100 frames sintéticos com 1k stamps/frame. dhat-rs. |
| `painter_stamp_golden_round_hard` | `ph2d-painter-brush` | Renderiza S-curve fixed com `round_hard` em headless GPU, compara contra baseline `tests/golden/painter/round_hard.png`. SSIM ≥ 0.995. |
| `painter_rendering_modes_distinct` | `ph2d-painter-brush` | Cada um dos 6 Rendering modes produz output **distinto** (anti-regressão a single-mode bug). |
| `painter_brush_format_roundtrip` | `ph2d-painter-brush` | `.ph2d-brush` save → load → idêntico (postcard determinístico). |
| `painter_brush_format_version_migration` | `ph2d-painter-brush` | HR-14: v1 brush carrega; v2 (future) tem migrator. |
| `painter_contract_surface` 🔒 | `ph2d-painter-brush` | Cap arch-gate: `Brush ≤ N campos`, `Stamp ≤ 96 bytes`, `RenderingMode ≤ 6 variants`. Mudar exige ADR-0042 amendment. |
| `painter_pressure_curve_monotonic` | `ph2d-painter-brush` | Pressure curve com defaults é monotônica não-decrescente (sanidade). |
| `painter_procreate_importer_smoke` | `ph2d-painter-brush` | Fixture `.brush` Procreate importa sem panic e produz `Brush` válido. |
| `painter_determinism_replay` (W11+) | `ph2d-painter-stroke` | HR-5: replay de stroke list reproduz bit-identical em 3 OS. |

## 1.12 Não-objetivos do brush engine (resumo)

- 3D Materials (Procreate Metallic + Roughness) — não-objetivo PH2D 2D.
- Brush packaging visual-only (sem schema typed) — Painter exige schema postcard versionado.
- Stroke ribbon-based (não-stamp) — incompatible com sabor Procreate.
- "Smart" brushes (ML-generated) — fora do escopo v1.0.

**Continua em:** [02_layers.md](02_layers.md) — camadas e blend modes.
