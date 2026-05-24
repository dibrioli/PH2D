---
name: painter-projeto
description: Especificação completa do Painter (Paint stack completo dentro de Image Tools), sabor Procreate, filtro minimalista-Blender. Doc canônico vivo; ratificar via ADR-0041.
status: Proposed (W0 spec)
data: 2026-05-23
decisor: Enio + LLM
---

# Painter — Paint stack do PH2D (sabor Procreate, filtro Blender)

## 0. Localização canônica e status

- **Doc canônico vivo:** este diretório. Spec evolui in-place até ser **congelada por ADR-0041**.
- **Estado em 2026-05-23:** W0 — **spec inicial revisada, 4 decisões estruturais fechadas (§11)**. Aguardando **mudanças estruturais no app** (decisão Enio 2026-05-23) antes da implementação começar. Não há crate `ph2d-tool-painter` ainda.
- **Família arquitetural:** Image Tools (irmã de [`ph2d-tool-bgremoval`](../../crates/ph2d-tool-bgremoval/) e [`ph2d-tool-padding`](../../crates/ph2d-tool-padding/)). Caminho fan-out (A) do [DIRETRIZ §3.8](../IntegracaoMultiAgente/DIRETRIZ.md), sabor **(3) stateful + panel docado**. Sem variant novo em `EditorAction` (canal genérico `OneShotImageOp` / `ToolPanelEvent` / `ActivateTool` cobre).
- **Pré-requisitos de implementação (ordem):** (1) mudanças estruturais pré-Painter no app concluídas; (2) ADR-0041 (Painter contract) aprovado; (3) ADR-0042 (Brush Engine GPU contract) aprovado. Sem ADRs, W1 fica bloqueada.

## 1. Visão em uma frase

Um Paint raster completo (Painter) integrado às Image Tools do PH2D, com **motor de pincel dual-texture Shape × Grain** ao estilo Procreate, **gestos canvas-first** que somem quando a usuária desenha, e **minimalismo Blender** — uma única forma canônica por feature, defaults muito bons, power-user controls escondidos atrás de um Brush Studio. Funciona em **desktop (mouse + tablet pen) e iPad/iOS (Apple Pencil) desde o W1**, não é port de feature mobile-only.

## 2. Os três eixos

1. **Sabor Procreate visível:** dual-texture brush engine (Shape × Grain), 6+ Rendering modes (Light/Uniform/Intense Glaze + Uniform/Intense Blending), ColorDrop com threshold gestual, QuickShape (hold-at-end → shape perfeita), gestos 2/3-finger para undo/redo/menu, sidebar Procreate-style (size + opacity + modifier + undo/redo), HUD que esmaece durante o stroke, 4-finger zen mode. As decisões de UX que tornam Procreate fluido são copiadas com intenção, não imitadas por inércia.

2. **Filtro Blender-minimalismo:** uma feature, um caminho canônico. Sem 4 jeitos de fazer a mesma coisa. Defaults excelentes. Sub-painéis para power users (Brush Studio). **Atalhos de teclado em desktop são primeira-classe** (lição Blender — ignorado por Procreate porque iPad). Sem wizards, sem onboarding modal, sem tooltip pop-up persistente.

3. **PH2D-native:** integra-se ao node-graph como gerador raster cookável (W12+ futuro), cores via `ph2d-tokens` (OKLCH, P3 quando disponível), brush stamp rendering em compute via `ph2d-gpu` (HR-3 zero-alloc no hot path), strokes persistem no asset content-addressed (HR-6), determinismo opt-in para replay/snapshot (HR-5; off por default — Painter vive no `PresentWorld`, não em `SimWorld`). LLM-first: brush studio inteiro exposto a MCP via `#[lua_export]` (HR-10).

## 3. Escopo — IN (versão 1.0 do Painter)

Numerado por arquivo do spec. Tudo aqui é **must-have antes do Painter ser considerado v1.0**.

### Brush engine ([01_brush_engine.md](01_brush_engine.md))
- Dual-texture stamp pipeline (Shape × Grain) com 7 parâmetros canônicos: **Spacing, Jitter, Scatter, Count, Streamline, Taper, Wet Mix**.
- 6 Rendering modes: **Light Glaze, Uniform Glaze, Intense Glaze, Heavy Glaze, Uniform Blending, Intense Blending**.
- 12 default brushes em 6 categorias: **Pencils, Inks, Markers, Paints, Watercolors, Airbrushes**. Minimal vs Procreate (18 cat); cada categoria com 2 brushes (1 "core", 1 "expressive").
- Pressure curve + tilt curve **per-brush** (modulam size, opacity, flow, color).
- Color Dynamics: stamp jitter (H/S/L) + stroke jitter + pressure modulation.
- Brush Studio: editor completo de brush expondo todos os parâmetros num panel docado (sabor "power-user", escondido por default).
- Brush format: `.ph2d-brush` (postcard binário, HR-6, blake3-addressed, importável de Procreate `.brush` com lossy mapping documentado).

### Layers ([02_layers.md](02_layers.md))
- Layer types: **raster, mask, clipping mask, reference, group, alpha-locked**.
- 22 blend modes (subset essencial de Photoshop/Procreate; lista canônica em §02).
- Layer panel gestures (swipe, pinch merge, 2-finger opacity, hold-multi-select).
- Layer limit dinâmico função do canvas × DPI × `MemoryBudget` (HR-13).
- Sem "adjustment layers" Photoshop-style — mantida a limitação Procreate por design (anti-pattern §12).

### Cor ([03_color.md](03_color.md))
- 5 modos no Color Panel: **Disc, Classic, Harmony, Value, Palettes**.
- ColorDrop com threshold gestual.
- Eyedropper tap-and-hold (1s default, configurable).
- Color management: **OKLCH interno** (via `ph2d-tokens`), display **sRGB / Display P3** detectado por device. CMYK fora de escopo (§12).
- Palettes importáveis: `.swatches` (Procreate format) + `.ph2d-palette` (postcard).
- Reference Companion samplável.

### Canvas e guides ([04_canvas_guides.md](04_canvas_guides.md))
- Canvas creation: presets + custom + DPI + color profile + max layers display.
- Drawing Guides: **2D Grid, Isometric, Perspective (1/2/3-point), Symmetry (V/H/Quadrant/Radial)**.
- **QuickShape** unificado (hold-at-end detecta line, arc, polygon, ellipse, triangle, quadrilateral; edit-shape mode com nodes).
- Reference Companion: **Canvas mode + Image mode** (sem Face mode no v1.0 — ARKit é iOS-only e mancha o multi-plataforma).
- Symmetry com Drawing Assist (stroke espelhado em real-time).

### Gestos e input ([05_gestos_input.md](05_gestos_input.md))
- Gesture model: 2-finger undo, 3-finger redo, 4-finger zen, pinch zoom+rotate, 3-finger swipe down → cut/copy/paste, tap-and-hold → eyedropper, draw-and-hold → QuickShape.
- **QuickMenu radial** 6 slots × 4 menus salváveis.
- **Gesture Controls panel** completo (reassignment de toda ação principal).
- **Atalhos de teclado em desktop** (Blender-style): B (brush), E (eraser), S (smudge), G (move), R (rotate), Z (zoom), Tab (zen), Ctrl+Z/Y, [/] (size), Shift+[/] (opacity), 0-9 (opacity %). Lista canônica em §05.
- Apple Pencil 2 double-tap + Apple Pencil Pro squeeze/barrel-roll (quando device suportar; gracefully ignored em desktop/Android/Pencil 1).

### Seleção, Transform, Adjustments ([06_selection_transform_adjustments.md](06_selection_transform_adjustments.md))
- Seleção: **Automatic, Freehand, Rectangle, Ellipse** + add/sub/intersect/feather/invert/save-load.
- Transform: **Freeform, Uniform, Distort, Warp** + Bilinear/Nearest Neighbor + Snapping + Magnetics + flip + 45° preset.
- Adjustments com modo **Layer/Pencil** (Procreate Adjustment Brush): HSB, Color Balance, Curves, Gradient Map, Gaussian Blur, Motion Blur, Noise, Sharpen, Bloom, Liquify, Clone. Halftone/Chromatic Aberration/Glitch como follow-up pós-v1.0.

### Input pipeline ([07_pencil_pipeline.md](07_pencil_pipeline.md))
- Apple Pencil (iPad), Wacom/Huion/XP-Pen (desktop), S Pen (Android), mouse fallback.
- Pressure + tilt + azimuth + (Pencil Pro) barrel-roll.
- Pressure curve global (em `Painter Preferences`) + per-brush (em Brush Studio).
- Palm rejection automática quando stylus em uso.
- Hover preview (iPad M2+, Wacom hover-enabled devices).
- Latency target: **≤16ms end-to-end** em desktop @ 120Hz; **≤9ms** em iPad ProMotion (paridade Procreate).

### Performance e memória ([08_performance_memory.md](08_performance_memory.md))
- Frame budget: stamp pipeline cabe em **3.5ms** do sub-budget Render (HR-4 §SKILL_Stack). Brush stamps em compute shader, sem alocação em hot path (HR-3, gateado por `tests/budget/painter_no_alloc.rs`).
- `MemoryBudget`: 200 MB VRAM (textures + layer cache) + 50 MB RAM (stroke history + undo stack) em targets desktop; 100 MB + 30 MB em mobile (HR-13).
- Stroke history como ring buffer (250 frames undo, paridade Procreate).

### Export e interop ([09_export_interop.md](09_export_interop.md))
- Native: `.ph2d-painter` (postcard, layers + brush refs + time-lapse opcional).
- Raster: PNG, JPEG, TIFF, WebP.
- Interop: **PSD export** com layer/blend/mask mapping (lossy onde Procreate-specific; tabela em §09).
- Animation: GIF, APNG, MP4, animated WebP (via Animation Assist).
- Time-lapse: MP4/HEVC frame capture (Studio quality default).

### Animation Assist ([10_animation_assist.md](10_animation_assist.md))
- Frame-based timeline (layer = frame OU group = frame).
- Onion skin (slider de frames antes/depois + colors + secondary).
- Loop / Ping-Pong / One Shot + frame rate 1-60.
- Background / Foreground locked layers.
- Hold duration por frame.
- **Não** track-based timeline (isso é Procreate Dreams; out-of-scope §12).

### UX chrome ([11_ux_chrome.md](11_ux_chrome.md))
- Layout 4-zonas existente em [`ph2d-editor-core`](../../crates/ph2d-editor-core/) (ADR-0023): Painter ocupa o canvas (Center 100%) + sidebar esquerda (size/opacity/modifier/undo/redo) + top bar (Gallery/Actions/Adjustments/Selection/Transform/Brushes/Smudge/Eraser/Layers/Color).
- HUD durante stroke: chrome esmaece, número de size/opacity flutua perto do slider.
- Zen Mode (4-finger tap ou keyboard Tab).
- Sidebar flippable left/right (canhotos).
- Procreate sidebar: a barra vertical existente do PH2D vira a Painter sidebar quando o Painter está ativo.

## 4. Escopo — OUT (não-objetivos explícitos)

Tudo abaixo é **deliberadamente** fora de v1.0. Detalhe em [12_fora_de_escopo.md](12_fora_de_escopo.md):

- Vetorial real (Painter é 100% raster; vetor entra via outras tools sobre Vello).
- Text avançado nativo no Painter (text é ferramenta separada com parley).
- 3D painting / Materials 3D (não-objetivo da engine 2D).
- Procreate Dreams timeline (motion tracks, keyframes, audio) — animação fica em Animation Assist frame-based.
- Face Reference / ARKit FacePaint (iOS-only, mancha multi-plataforma).
- CMYK first-class (P3/sRGB internos; CMYK só em export-side tool dedicado se demanda surgir).
- Cloud sync nativo (arquivos no FS, sync por Files.app/Dropbox/Drive externos).
- Macros/Actions/Scripts no estilo Photoshop (substituído por Luau quando o usuário precisar de automação; HR-10).
- Smart objects, layer linking, instâncias clone (Clone é destrutivo, como Procreate).
- Adjustment layers Photoshop-stack (mantém limitação Procreate por design — workflow lean).

## 5. Filtro minimalista-Blender — princípios operacionais

Quatro regras que se aplicam **em toda decisão de design** do Painter. Quando duas alternativas competem, vence a que satisfaz mais regras.

### 5.1 Um caminho canônico por feature
Se Procreate tem 3 jeitos de samplear cor (tap-and-hold + modifier square + Apple Pencil tap), o Painter expõe **1 default + 1 customizável em Gesture Controls**. Sem três botões na UI primária fazendo a mesma coisa.

### 5.2 Defaults excelentes; preferences escondidas
A Brush Studio existe, mas o usuário típico nunca precisa abrir. Os 12 default brushes funcionam tão bem que o usuário só vai à Brush Studio quando quer um efeito específico. Sem "first-time user wizard". Sem tooltip que aparece sempre.

### 5.3 Atalhos de teclado em desktop são primeira-classe
A grande lacuna do Procreate é desktop. PH2D roda em desktop desde W1; atalhos Blender-style são canônicos. Listados em §05. Em iPad, gestos cobrem a mesma função. **Não criar UI button só porque desktop precisa de hover** — atalho de teclado vence.

### 5.4 Power escondido atrás de Brush Studio / Gesture Controls / Preferences
Profundidade existe (curves, Wet Mix completo, todos os jitter axes), mas em sub-painéis especializados. A UI primária permanece minúscula: 5 botões top-bar + sidebar de 4 elementos + color thumb.

## 6. Multi-plataforma desde o W1 (vs Procreate iPad-only)

Procreate é deliberadamente iPad-only — uma decisão filosófica deles. **Painter PH2D é desktop + iPad + iOS + Android desde o primeiro pixel pintado**, herdando o modelo de [`ph2d-host`](../../crates/ph2d-host/) e shells PH2D.

| Plataforma | Input principal | Input secundário | Atalhos | Gestos multi-touch |
|---|---|---|---|---|
| Desktop (Mac/Win/Linux) | Wacom/Huion/XP-Pen tablet | Mouse | **Primeira classe (Blender-style)** | Trackpad multi-touch |
| iPad / iOS | Apple Pencil (2/Pro) | Finger touch | Hardware keyboard quando presente | Primeira classe |
| Android | S Pen / Wacom Android | Finger touch | Hardware keyboard quando presente | Primeira classe |
| Web | Pointer Events API (touch/pen/mouse) | — | Browser-limited | Pointer Events |

**Implicações concretas:**
- Apple Pencil Pro features (squeeze, barrel-roll) detectadas em runtime; **não há build conditional** que esconda UI em outros devices (HR-1 + HR-15). A UI "Configure squeeze action" simplesmente fica greyed com tooltip "requer Apple Pencil Pro" em devices sem.
- Atalhos de teclado configuráveis num panel "Painter Shortcuts" (foundational, vive em `ph2d-editor-core`, não no Painter crate).
- Hover preview funciona em qualquer device que suporte pointer hover (iPad M2+, Wacom hover-enabled, mouse, alguns Surface Pens).
- Pressure normalizada em `[0.0, 1.0]` no `ph2d-input` (HR-1); curvas per-brush trabalham no espaço normalizado.

## 7. Mapping arquitetural ao PH2D

### 7.1 Crates novos (W1+)

```
crates/
├── ph2d-tool-painter/         ✨ tool stateful + manifest (sabor 3, DIRETRIZ §3.8.3)
│   ├── Cargo.toml             ph2d-tool-registry, ph2d-editor-core,
│   │                          ph2d-painter-brush, ph2d-a11y, ph2d-core,
│   │                          ph2d-vector (ícone)
│   ├── src/lib.rs             pub const MANIFEST + pub fn register + pub fn make
│   ├── src/tool.rs            impl Tool (id="painter", build_panel,
│   │                          handle_panel_event, as_any_mut, set_source_snapshot,
│   │                          commit_to_sprite)
│   ├── src/algorithm.rs       stroke compositing (alpha onto layer texture)
│   ├── src/params.rs          PainterUiEdit / PainterUiSnapshot / PainterParams
│   ├── src/icon.rs            BezPath (Lucide brush 24×24)
│   └── tests/                 golden tests, no-alloc tests
│
├── ph2d-panel-painter/        ✨ panel docado (Brush Library + sidebar gigante)
│   ├── Cargo.toml             ph2d-editor-core, ph2d-a11y, ph2d-tokens,
│   │                          ph2d-text, ph2d-vector
│   ├── src/lib.rs             impl Panel for PainterPanel (ID="painter",
│   │                          State = PainterPanelState)
│   └── tests/
│
├── ph2d-painter-brush/        ✨ brush engine compartilhável (Shape×Grain,
│   ├── Cargo.toml             stamp pipeline, default brush library)
│   ├── src/lib.rs             pub use brush::* + library::*
│   ├── src/brush.rs           Brush struct + serde
│   ├── src/stamp.rs           StampPipeline (CPU side; GPU encode em ph2d-gpu)
│   ├── src/library.rs         DEFAULT_BRUSHES const + load/save .ph2d-brush
│   ├── src/import_procreate.rs    Procreate .brush importer (lossy)
│   ├── shaders/stamp.wgsl     compute shader principal
│   └── tests/                 golden brush stamps
│
├── ph2d-painter-color/        ✨ Color modes (Disc/Classic/Harmony/Value/Palettes)
│   ├── src/lib.rs             pub use {disc, classic, harmony, value, palettes}::*
│   ├── src/colordrop.rs       flood fill + threshold + reference-layer aware
│   └── tests/
│
├── ph2d-painter-stroke/       ✨ stroke history + undo + time-lapse capture
│   ├── src/lib.rs             StrokeHistory (ring 250) + UndoStack
│   ├── src/timelapse.rs       frame capture infra (off-thread encode)
│   └── tests/
│
└── ph2d-painter-anim/         ✨ Animation Assist (frame-based timeline)
    ├── src/lib.rs             FrameTimeline + OnionSkin + Loop modes
    └── tests/
```

**Wiring:** `cargo run -p ph2d-tool-sync` regenera `ph2d-tool-registry-init` (DIRETRIZ §3.8). Painel é caminho (B) — Coord adiciona feature em `ph2d-panel-registry-init` (uma vez, quando W3 fechar a infra de multi-layer).

### 7.2 Contratos congelados que o Painter respeita

- **`Tool` / `RasterEditTool` / `PanelEvent`** (ADR-0040 §7 + ADR-0041 rename, gate [`architecture_tool_contract_surface`](../../crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs); caps `Tool=10` / `RasterEditTool=5` / `PanelEvent=4`). O Painter implementa `Tool` certamente; `RasterEditTool` opcional — o padrão atual (BgRemoval/Padding) é downcast via `as_any_mut`, e o Painter segue esse padrão na v1.0. Migração para `RasterEditTool` é vertical separada futura.
- **`EditorAction`** (4 variants genéricos). Painter usa:
  - `ActivateTool("painter")` quando o usuário troca pra Painter
  - `OneShotImageOp(...)` zero — Painter é stateful, não one-shot
  - `ToolPanelEvent(PanelEvent::SetValue|Click(id, ...))` para todos os widgets do panel
  - `CancelActiveTool` ao trocar de ferramenta com stroke em vôo

### 7.3 Hard Rules aplicadas

| HR | Aplicação no Painter |
|----|---------------------|
| **HR-1** | Painter é platform-agnostic; toda input vem via `ph2d-input` (pointer/pencil/touch); arquivos via `PlatformHost`. |
| **HR-3** | Hot path do stamp pipeline: zero `Box::new` / `Vec::push`-que-realoca / `String::from`. Pool de stamps pré-alocado (capacidade = max_stamps_per_frame), bump arena por stroke, ring buffer pra undo. Gateado em `tests/budget/painter_no_alloc.rs`. |
| **HR-4** | Stamp pipeline cabe no sub-budget de **3.5ms** (Render). Brush Studio editing fora de hot path. Time-lapse encode em thread separada. |
| **HR-5** | Painter vive em `PresentWorld` (ADR-0021), **não** em `SimWorld`. Determinismo NÃO é prometido por default. Replay determinístico (stroke list → reaplica) é feature opt-in (W11), usa fixed-point para coordenadas e ordena reduções. |
| **HR-6** | Brushes content-addressed (blake3 do `.ph2d-brush`). Strokes do canvas viram parte do asset; mudanças geram novo hash. |
| **HR-7** | Painter compilado só com feature `editor` (release de jogo distribuído não tem Painter). |
| **HR-10** | Brush Studio params expostos via `#[lua_export]` → MCP toolset `painter_brush_*`. |
| **HR-11** | Operação destrutiva (`painter_flatten`, `painter_clear_all_layers`, `painter_delete_layer`) exige confirmation token MCP. |
| **HR-12** | Todo widget Painter (slider de size, color disc, layer thumb, brush thumb) emite `accesskit::Node`. Brush Library com role `Tree`, color disc com role `Slider2D` (custom + label "color picker, hue X, saturation Y"). |
| **HR-13** | `Plugin::init` declara `MemoryBudget { vram_mb, ram_mb, heap_script_mb }` — números em §08. |
| **HR-15** | Strings via Fluent (`t!`) em `crates/ph2d-tool-painter/locales/pt-BR.ftl` + `en-US.ftl`. Nomes de brush e modes em chave i18n (`painter.brush.round_hard`, `painter.rendering.light_glaze`). |
| **HR-16** | Brush params salvos no stroke history são POD-like; sem closures, sem userdata. |
| **HR-18** | Nenhum arquivo do Painter ≥ 600 LOC; funções ≤ 200 LOC. Brush Studio decomposto em sub-módulos por seção (`brush_studio/{stroke_path, taper, shape, grain, rendering, wet_mix, color_dynamics, dynamics, pencil}.rs`). |

### 7.4 Dependências externas (além do stack PH2D já pinado)

Nada novo precisa entrar no workspace. Reutiliza:
- **Vello 0.8** — UI do Painter (chrome, widgets do Brush Studio, color disc).
- **wgpu 28** — stamp pipeline em compute shader (WGSL) via `ph2d-gpu`.
- **parley 0.6** — labels, brush names, layer names.
- **kurbo 0.12** — Bézier paths do QuickShape, transform mesh warp.
- **glam 0.30** — math do input pipeline.
- **`image 0.25`** — import/export PNG (já no stack).
- **`postcard 1`** — serialização do `.ph2d-painter`, `.ph2d-brush`, stroke history.
- **`blake3 1`** — content-address de brushes e strokes.

Possível dep nova candidata (justificar em PR):
- **`png 0.18`** ou seguir usando `image::png` — escolher quando W12 (export) entrar.

## 8. Roadmap por waves

Sequência de fan-out, cada wave entregável e auditada (loop §1 do [HANDOFF_node_system.md](../HANDOFF_node_system.md) aplica). Numeração propositalmente paralela ao sistema de nós (W1=neck, W2=vertical, W3+=fan-out).

| Wave | Conteúdo | Critério de fechamento |
|------|----------|------------------------|
| **W0** | **Spec freeze** — este doc + ADR-0041 (Painter contract) + ADR-0042 (Brush Engine GPU contract). | ADR aprovado + arch test `painter_contract_surface` ativo (caps congelados de `Brush` / `Stroke` / `PainterUiEdit`). |
| **W1** | **Neck — brush engine core**: `ph2d-painter-brush` com Shape×Grain mínimo (1 brush "Round Hard"), stamp compute shader, integração com Sprite ativo via `as_any_mut` downcast. Smoke "stroke deixa pixels na sprite". | Smoke do Enio: clica no botão Brush no chrome do PH2D, pinta na sprite, sai pixel certo. Cross-platform (Mac desktop pelo menos). |
| **W2** | **Vertical Painter MVP**: 1 brush, 1 layer, undo/redo (ring 250), sidebar (size + opacity slider), basic color picker (Classic), commit-to-sprite via Tool contract. | Painter aparece como pill no chrome de Image Tools; clica → palette modal abre painter sidebar; pinta; Ctrl+Z desfaz; troca de tool → commit no asset. |
| **W3** | **Multi-layer + masks + clipping + alpha lock + blend modes** (22 modes). Layer panel completo. | Layer panel docado mostra layers reordenáveis; pinch merge funciona; blend modes pickable via popover. |
| **W4** | **Full brush library** (12 default brushes, 6 categorias). Importer Procreate `.brush` (lossy, log de gaps). | 12 brushes, todos com golden test. Importação de uma .brushset Procreate gera lista de brushes utilizável. |
| **W5** | **Brush Studio** (full Shape/Grain/Stroke Path/Taper/Rendering/Wet Mix/Color Dynamics/Dynamics/Apple Pencil UI). | Brush Studio panel docado; usuária edita brush; preview live; save como `.ph2d-brush`. |
| **W6** | **Color modes completos**: Disc, Classic, Harmony, Value, Palettes. ColorDrop + threshold. Palettes import/export. Reference Companion (Canvas + Image). | 5 modos funcionam; ColorDrop com threshold gestual no-axis ative; palettes `.ph2d-palette` + import `.swatches`. |
| **W7** | **Selection + Transform + Adjustments**. Adjustment Brush mode. | 4 tipos de selection; 4 modos de transform; ~12 adjustments com modos Layer/Pencil. |
| **W8** | **Drawing Guides + QuickShape + Symmetry com Drawing Assist**. | 4 tipos de guide; QuickShape detecta line/arc/poly/ellipse/triangle/quad; Symmetry stroke espelha em real-time. |
| **W9** | **Gestures + QuickMenu + Gesture Controls panel + desktop shortcuts**. | Todos os 10+ gestos canvas + 6-slot QuickMenu × 4 menus salváveis + shortcut config + Apple Pencil double-tap/squeeze quando device disponível. |
| **W10** | **Animation Assist** (frame-based timeline, onion skin, loop modes, BG/FG layers, hold duration). | Timeline aparece via Actions; frame-by-frame funciona; export GIF/MP4. |
| **W11** | **Reference Companion enhancements + Time-lapse capture/export + replay determinístico opcional**. | Time-lapse MP4 export 1080p/4K; replay reaplica stroke list e produz bit-identical image. |
| **W12** | **Export interop**: PSD com layer/blend/mask mapping; native `.ph2d-painter`; PSD import (lossy). | Export PSD abre no Photoshop preservando layers/blend; import PSD trazer layers. |

**Frequência de FREEZE:**
- W0 congela contrato (ADR-0041/0042).
- W2 congela API pública do `ph2d-tool-painter` (sem variant novo em `EditorAction`).
- W5 congela schema de `.ph2d-brush` (HR-14: brush format ganha campo `version: u32`).
- W12 congela schema de `.ph2d-painter` (idem).

## 9. Índice dos arquivos do spec

| Arquivo | Conteúdo |
|---------|----------|
| [01_brush_engine.md](01_brush_engine.md) | Motor de pincel — Shape×Grain, parâmetros, Rendering modes, Brush Studio, default library, format `.ph2d-brush`, GPU stamp pipeline. |
| [02_layers.md](02_layers.md) | Camadas — tipos, blend modes (lista canônica 22), operações, panel gestures, layer limit dinâmico. |
| [03_color.md](03_color.md) | Cor — 5 modos, palettes, ColorDrop, eyedropper, color management OKLCH/P3/sRGB, Reference Companion samplável. |
| [04_canvas_guides.md](04_canvas_guides.md) | Canvas creation, Drawing Guides (2D/Iso/Perspective/Symmetry), QuickShape unificado, Reference Companion. |
| [05_gestos_input.md](05_gestos_input.md) | Gestos canvas + layer + sidebar + QuickMenu + Gesture Controls + atalhos de teclado desktop. |
| [06_selection_transform_adjustments.md](06_selection_transform_adjustments.md) | Selection (4 tipos), Transform (4 modos), Adjustments (12 com modo Layer/Pencil). |
| [07_pencil_pipeline.md](07_pencil_pipeline.md) | Pipeline de input: Pencil/tablet/mouse, pressure/tilt curves, palm rejection, hover, predictive touch, latency target. |
| [08_performance_memory.md](08_performance_memory.md) | Frame budget, memory budget, GPU stamp pipeline, time-lapse capture model, perf gates. |
| [09_export_interop.md](09_export_interop.md) | Formatos (`.ph2d-painter`/PSD/PNG/JPEG/TIFF/WebP/GIF/APNG/MP4), PSD mapping table, animation export. |
| [10_animation_assist.md](10_animation_assist.md) | Frame-based timeline, onion skin, loops, BG/FG layers, hold duration. |
| [11_ux_chrome.md](11_ux_chrome.md) | Layout 4-zonas, sidebar, HUD durante stroke, Zen Mode, integração com chrome PH2D, multi-plataforma. |
| [12_fora_de_escopo.md](12_fora_de_escopo.md) | Não-objetivos explícitos com razões + roadmap de "quando talvez" para cada um. |
| [13_referencias.md](13_referencias.md) | Fontes oficiais Procreate + community + papers + referências PH2D. |

## 10. Como contribuir com este spec

Este doc é **vivo** até o ADR-0041 ser aprovado. Sugestões / correções:

1. Edite o arquivo específico (`NN_*.md`) e abra PR com link para esta README.
2. Spec contradiz código: spec ganha (estamos pré-implementação). Após W1, código ganha e spec atualiza.
3. Decisão grande (mudar contrato Tool, mudar formato `.ph2d-brush` após W5): **exige ADR novo** estendendo 0041/0042.
4. LLM editando: leia [DIRETRIZ.md](../IntegracaoMultiAgente/DIRETRIZ.md) §0..§1.4 antes de tocar em código (essa fase ainda é só doc — sem triagem aplicável).

## 11. Decisões estruturais — fechadas (2026-05-23)

Os 4 pontos abertos na primeira versão deste spec foram resolvidos. **Não reabrir sem ADR amendment.**

| # | Item | Decisão | Razão |
|---|------|---------|-------|
| 1 | Brush format storage | **postcard binário + CLI** (`ph2d-painter-brush-cli inspect / edit / compile / diff / validate / convert`) | Texturas inline ficam binárias nativas (vs TOML+base64 inflando ~4×); hash determinístico (HR-6) grátis via postcard canonical serialization; boot rápido com zero-copy deserialize; alinha com Procreate `.brush` para importer simétrico. CLI provê camada textual derivada para auditoria, edit à mão, e diff em PR review — sem armadilhas de texturas inline em text. Detalhe em [01 §1.9](01_brush_engine.md) + [09 §9.7](09_export_interop.md). |
| 2 | Stamp shader | **W1 = unified (rampa). Padrão ouro final = especializado (6 shaders).** ADR-0042 ratificará a transição. | Especializado é o que Procreate Valkyrie faz e ganha 5-10% de perf — diferença visível em devices entry-level (iPad Air, Android mid-tier, Apple M1 8-core GPU) ProMotion 120Hz. Mas começar especializado em W1 congela a matemática dos 6 Rendering modes **antes** do affinamento visual em W4-W5 — cada ajuste vira refactor coordenado em 6 shaders. Unified em W1 prova o pipeline inteiro (input → stamp scheduler → GPU compute → composite → present) e isola bugs com 1 implementação. Refactor para especializado é mecânico, não-destrutivo (CPU `StampScheduler` ganha pre-grouping `stamps_by_mode: [Vec<Stamp>; 6]`; encode despacha 1 pipeline por non-empty group; API pública `StampPipeline` inalterada; golden tests bit-identical). Gate `painter_stamp_specialize_when_budget_pressure` em CI monitora budget headroom — falha se < 15% em 3 runs consecutivos, sinalizando momento do refactor. Detalhe em [01 §1.8.3](01_brush_engine.md). |
| 3 | Chrome quando Painter ativo | **Takeover do canvas** (Procreate-style). Painter substitui chrome PH2D do canvas enquanto ativo. | É o que dá o sabor — sidebar + top bar Procreate-style com sliders verticais de size/opacity + 10 pills é o que faz o Painter "respirar" como Procreate respira. Trade-off aceito: inconsistente com BgRemoval/Padding (que são panel-docado-style) — mas Painter é workhorse de horas vs eles que são one-shot/lean; nível de ferramenta diferente justifica chrome diferente. Mecânica: `ActivateTool("painter")` registra `painter_active=true` no `HeroScreen`; chrome `dispatch_all` (ADR-0040 handlers) lê o estado e suprime pills/sidebar fora do escopo Painter; reverter em `CancelActiveTool` ou `ActivateTool(other)`. Esc com stroke em vôo → confirm popup (UX safety). Detalhe em [11 §11.2](11_ux_chrome.md). |
| 4 | Multi-document Gallery | **Só W11+.** | Gallery exige scene management infra que ainda não foi triada no PH2D editor; investigar estado pós-W10 quando todos os tools internos do Painter estiverem maduros (brushes/layers/color/transform/anim). Single-canvas em W1-W10 simplifica o pipeline de save/load/Painter lifecycle e remove uma surface de bug. |

---

**Próximos passos concretos (ordem):**
1. **Mudanças estruturais no app** (decisão Enio 2026-05-23) — pré-requisito antes de W0 fechar.
2. **ADR-0041** (Painter contract) referenciando este spec + congelando o `Tool` impl e contratos públicos do `ph2d-tool-painter`.
3. **ADR-0042** (Brush Engine GPU contract) ratificando decisão #2 e definindo o caminho W1-unified → W5+-especializado.
4. **W1 começa** (vide §8 roadmap): neck brush engine + 1 brush "Round Hard" + smoke "pixel sai na sprite".
