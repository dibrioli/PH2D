# ADR-0043 — Painter contract (`ph2d-tool-painter` surface + caps)

**Status:** Proposed (2026-05-26)
**Decisor(es):** Enio + Claude (Coord-A, sessão Painter W0).
**Pré-requisitos:** [ADR-0040 — tool-as-isolated-feature-crate](0040-tool-as-isolated-feature-crate.md), [ADR-0041 — RasterEditTool rename + deactivate](0041-rasteredit-rename-and-deactivate.md), [ADR-0042 — Wave 10 closure](0042-wave-10-closure.md).
**Sub-contratos congelados por ADRs irmãs:** 0044 (Brush Engine GPU + Mixbox + Procedural Grain), 0045 (Adjustment Layers), 0046 (Stroke Vector History), 0047 (MCP Stroke Engine), 0048 (Stroke Inspector), 0049 (Fluid Brushes Extension).
**Spec normativa:** [`docs/Painter_projeto/`](../../Painter_projeto/) (16 docs + plano §15) — esta ADR fixa o **contrato de superfície** do `PainterTool`; a spec detalha o **comportamento**.
**Tags:** painter, wave-0, contract, drop-crate, padrão-ouro

---

## 1. Contexto

O Painter (sucessor do Procreate, mandato §0 [HANDOFF_painter.md](../../HANDOFF_painter.md)) é a **maior feature single-tool do PH2D**: 17 waves planejadas, ~140 parâmetros de brush em 12 sub-painéis (Brush Studio), 22 blend modes, 12 adjustment kinds, stroke history vetorial, MCP integration, fluid brushes opcionais. Antes de escrever uma linha de código (W1+) o contrato precisa estar **congelado** porque:

1. **Drop-crate fan-out depende disso.** [DIRETRIZ v7.0 §3.A](../../IntegracaoMultiAgente/DIRETRIZ.md) define o caminho (A): para uma feature ser pura adição de crate satélite, o que ela importa de `editor-core` precisa estar fixo. Painter é o primeiro tool não-bgremoval-shape a esticar a moldura — se a moldura ceder por feature, viramos um god-tool em `editor-core` outra vez (precisamente o que [ADR-0040 §1](0040-tool-as-isolated-feature-crate.md) sancionou contra).

2. **Múltiplas waves paralelas vão consumir o mesmo `PainterTool`.** W1 cria o esqueleto; W2 adiciona sidebar; W3 adiciona panel-painter-layers; W5 adiciona panel-painter-brush-studio; W7 adiciona panel-painter-color; W14 adiciona panel-painter-inspector. Sem contrato fixo, cada wave reinterpreta a API e o trabalho de duas sessões paralelas colide.

3. **A spec do Painter já está madura** (16 docs, 2201-linha plano de implementação). O risco "spec do papel" se materializa se o ADR só repetir prosa sem caps numéricos — vide DIRETRIZ §3.A.3 "caps numéricos são o teste de cheiro do contrato".

4. **O contrato `Tool`/`RasterEditTool`/`PanelEvent` está congelado por ADR-0040 + ADR-0041** (caps 10/5/4). Esta ADR **não os amenda** — só fixa **o que o `PainterTool` impl expõe** dentro desses caps.

### 1.1 O que diferencia o Painter dos outros `RasterEditTool`s

Os 5 raster-edit tools de produção (bgremoval, color-equalization, padding, upscale, equalize-sizes) seguem o mesmo padrão "lean": panel docado, slider→preview rerun, Apply→commit. O Painter quebra esse padrão em três dimensões — todas internas ao seu próprio crate, sem amendment de contrato central:

| Eixo | RasterEdit lean (bgremoval & cia) | PainterTool |
|---|---|---|
| **Tempo de sessão** | One-shot (segundos a minutos) | Workhorse (horas) |
| **Chrome** | Panel docado coexiste com chrome PH2D | Takeover — substitui topbar + sidebar PH2D ([11_ux_chrome.md §11.2](../../Painter_projeto/11_ux_chrome.md)) |
| **Canvas input** | Click+drag arma feature (eyedropper, protect brush) | Stream contínuo de stamps via stroke pipeline (HR-3 hot path, [01_brush_engine.md §1.2](../../Painter_projeto/01_brush_engine.md)) |

**Crucial:** nenhum desses três eixos exige amendment ao trait `Tool` ou `RasterEditTool`. O takeover é estado do `HeroScreen` (chrome lê `painter_active` e suprime pills/sidebar PH2D). O stream de stamps reusa a canvas-event path que `BrushTool` (proto-tool já registrado em `editor-core`) já consome — o Painter substitui o `BrushTool` proto-tool por uma implementação satélite plena, **não** introduz um novo eixo de input no trait.

### 1.2 Por que sub-contratos separados (0044..0049)

Esta ADR fixa **só a superfície de `PainterTool`** (impl Tool + impl RasterEditTool + tipos UI). A engine real do brush (`Brush` struct, `Stamp` struct, atlas, compute pipeline) vive em `ph2d-painter-brush` (W1) e é congelada por ADR-0044. Sub-sistemas adjacentes (adjustment layers W4, stroke history W1/W12, MCP W13, inspector W14, fluid W15) viram crates próprios e ADRs próprias.

Razão da separação: cada sub-contrato tem evolução distinta. Adicionar uma nova categoria de adjustment (ADR-0045) não pode forçar bump no `PainterUiEdit` cap. Adicionar um modo de `ProceduralGrain` (ADR-0044) não pode tocar contrato MCP. A modularidade do contrato espelha a modularidade dos crates.

---

## 2. Decisão

### 2.1 Crate satélite `ph2d-tool-painter`

O Painter é uma feature satélite drop-in conforme [ADR-0040 §2](0040-tool-as-isolated-feature-crate.md). Estrutura canônica (espelha [`crates/ph2d-tool-bgremoval/`](../../../crates/ph2d-tool-bgremoval/), o template-mestre Wave 10):

```
crates/ph2d-tool-painter/
  Cargo.toml             # deps: ph2d-editor-core, ph2d-tool-registry, ph2d-painter-brush,
                         #       ph2d-painter-stroke, ph2d-color, ph2d-tokens, ph2d-a11y
  src/lib.rs             # #![forbid(unsafe_code)] PRIMEIRO; pub fn make() / pub fn register / pub const MANIFEST
  src/manifest.rs        # ToolManifest (id="painter", cluster="image_tools", kind=ImageEdit, is_default=false)
  src/tool.rs            # struct PainterTool + impl Tool + impl RasterEditTool
  src/params.rs          # PainterParams + PainterUiEdit + PainterUiSnapshot
  src/icon.rs            # BezPath placeholder paint-brush Lucide-style 24×24 (W1 polish substitui)
```

Crates irmãos criados em waves seguintes (cada um com sua ADR de contrato; esta ADR só os nomeia):

| Crate | Wave | Sub-contrato |
|---|---|---|
| `ph2d-painter-brush` | W1 | ADR-0044 (Brush + Stamp + atlas) |
| `ph2d-painter-stroke` | W1 | ADR-0046 (Stroke Vector History) |
| `ph2d-panel-painter` (sidebar) | W2 | Conforme padrão ADR-0029 trait-driven panel host |
| `ph2d-painter-color` | W7 | Sem ADR — usa `ph2d-color` (ADR-0042) + 5 modos via panel |
| `ph2d-panel-painter-layers` | W3 | Conforme ADR-0029 |
| `ph2d-panel-painter-brush-studio` | W5 | Conforme ADR-0029; consome ADR-0044 |
| `ph2d-panel-painter-inspector` | W14 | ADR-0048 |
| `ph2d-painter-anim` | W11 | Sem ADR — extensão pura de stroke history |
| `ph2d-painter-fluid` | W15 | ADR-0049 (opt-in) |

### 2.2 Superfície de `PainterTool` (caps congelados)

```rust
// crates/ph2d-tool-painter/src/tool.rs
pub struct PainterTool {
    params: PainterParams,
    active_brush: BrushHandle,       // -> ph2d-painter-brush::Library
    stroke_state: StrokeBuilder,     // -> ph2d-painter-stroke
    source_rgba: Vec<u8>,            // active layer pixels (RasterEditTool source)
    source_size: (u32, u32),
    preview_dirty: bool,
    pending_commit: bool,
    // ... privado
}

impl Tool for PainterTool { /* 10 métodos canônicos, sem add */ }
impl RasterEditTool for PainterTool { /* 5 métodos canônicos, sem add */ }
```

**`PainterTool` NÃO bump caps de `Tool` nem `RasterEditTool`.** Toda funcionalidade Painter-específica (sidebar event handling, takeover state push, stroke ingestion, history binding) opera dentro dos 10+5 slots existentes via:

- `Tool::handle_panel_event(PanelEvent)` — sidebar slider/toggle/button → `PainterUiEdit` interno.
- `Tool::on_activate` / `Tool::on_deactivate` — pushes/clears `painter_active = true` flag no `HeroScreen` (chrome lê esse flag e faz takeover; mecanismo é estado, não contrato).
- `RasterEditTool::set_source` / `current_preview` / `run_full` / `deactivate` — fluxo de raster idêntico ao bgremoval.
- Canvas pointer events (stamp ingestion) viajam pela canvas-event path existente (a mesma que `BrushTool` proto consome hoje) — **NÃO** via `PanelEvent`. Esta ADR registra que esse path continua válido; sua surface (se virar trait) é trabalho de wave futura, fora do escopo Painter v1.

### 2.3 Tipos UI — caps numéricos

#### `PainterUiEdit` — **≤ 24 variants**

Eventos da sidebar Painter + topbar tool-switching, traduzidos do `PanelEvent` genérico para semântica Painter dentro de `handle_panel_event`. Baseline calibração: bgremoval = 15 variants (canônico Wave 10); CEQ = 41 (outlier, segmentado em slider+chip pares). Painter sidebar tem **escopo menor que CEQ** (chips advanced vivem no Brush Studio panel separado, não no sidebar), mas **maior que bgremoval** (workhorse com mais controles, evolução multi-wave).

**Cap = 24** — bump original de 20 → 24 (audit adversarial 2026-05-26): contagem realista de variants exige headroom para W9 Drawing Assist (~3 variants — symmetry axis, radial N, mirror offset), W11 Animation Assist (~2 variants — onion-skin toggle, fps), W14 Inspector entry (~1 variant — OpenInspector). v1 ~15 + W9 3 + W11 2 + W14 1 = ~21; cap 24 deixa 3 slots residuais.

Variants esperados v1 (~15, com slots de headroom para waves futuras):

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum PainterUiEdit {
    // Sidebar sliders (normalized 0..=1, projetados via FULL_SCALE consts)
    Size(f32),
    Opacity(f32),
    // Color (delegado via slot; ColorPanel emite OklchColor completo)
    SetColor(OklchColor),
    // Brush selection (BrushHandle = atlas-resident id; Library é detalhe de ph2d-painter-brush)
    SelectBrush(BrushHandle),
    // Topbar tool-switching (10 chips conforme 11_ux_chrome §11.3; só painting tools 6..10 fluem por aqui;
    // editing tools 1..5 são EditorAction genéricos)
    ToggleBrushMode,
    ToggleSmudgeMode,
    ToggleEraserMode,
    OpenLayersPopover,
    OpenColorPopover,
    // Eyedropper (sidebar tap-and-hold equivalent; arm flag)
    ToggleEyedropper,
    // Undo/Redo (sidebar buttons; ALSO disparáveis via gesture/keyboard fora deste enum)
    Undo,
    Redo,
    // Brush Studio (opens modal — emits OpenBrushStudio; modal events viajam por panel-painter-brush-studio,
    // não por aqui)
    OpenBrushStudio,
    // Reset (sidebar long-press; análogo ao ResetAll dos outros tools)
    ResetSidebar,
    // Symmetry quick-toggle (W9 Drawing Assist)
    ToggleSymmetry,
    // === ~9 slots de headroom para waves futuras ===
    // W9: SetSymmetryAxis(SymmetryAxis), SetRadialN(u8), SetMirrorOffset(f32)
    // W11: ToggleOnionSkin, SetAnimFps(f32)
    // W14: OpenInspector
    // 3 residual
}
```

**Não cabe aqui:** edits de adjustment layers (ADR-0045 / panel-painter-layers), edits de brush studio sliders (ADR-0044 / panel-painter-brush-studio), MCP tool calls (ADR-0047), stroke retroactive edits (ADR-0048). Esses fluem via seus próprios `*UiEdit` enums nos seus crates.

#### `PainterUiSnapshot` — **≤ 18 fields**

Projeção read-only do `PainterTool` que o `ph2d-panel-painter` (sidebar) pinta a cada frame. Baseline bgremoval = 13 fields; +20% = 15.6; **cap = 18** (sidebar tem ~13 elementos visuais + 2-3 flags de affordance).

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct PainterUiSnapshot {
    // Sliders (normalized 0..=1)
    pub size01: f32,
    pub opacity01: f32,
    // Color (display)
    pub active_color: OklchColor,
    pub secondary_color: OklchColor,    // long-press slot
    // Brush (display)
    pub active_brush_thumb: ThumbHandle, // -> atlas; render é responsabilidade do painel
    pub active_brush_name: String,
    // Topbar mode (display)
    pub mode: PainterMode,               // Brush / Smudge / Eraser
    // Affordance flags (display state)
    pub eyedropper_armed: bool,
    pub symmetry_enabled: bool,
    pub undo_enabled: bool,
    pub redo_enabled: bool,
    pub stroke_in_flight: bool,          // chrome fade-out (11_ux_chrome §11.1 #2)
    // Selection / takeover state (display)
    pub takeover_active: bool,
    pub active_layer_name: String,       // sidebar layer chip
    pub active_layer_locked: bool,
    // === 3 slots de headroom ===
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PainterMode { Brush, Smudge, Eraser }
```

#### `PainterParams` — **≤ 12 fields**

Estado serializável do Painter sidebar (não inclui Brush struct nem stroke history — esses vivem nos crates dedicados via `BrushHandle` + `StrokeHistoryRef`). Persistido por sessão via mecanismo existente do `ph2d-editor-core`.

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct PainterParams {
    pub size_px: f32,                    // resolvido (não normalizado) — sidebar normaliza p/ slider
    pub opacity: f32,                    // [0, 1]
    pub active_color: OklchColor,
    pub secondary_color: OklchColor,
    pub active_brush: BrushHandle,
    pub mode: PainterMode,
    pub symmetry: Option<SymmetryAxis>,  // None / Vertical / Horizontal / Radial(n)
    pub eyedropper_armed: bool,
    pub takeover_active: bool,
    pub version: u32,                    // HR-14 serialization version (=1 v1)
    // === 2 slots de headroom ===
}
```

**Não cabe aqui:** parâmetros do brush ativo (vivem em `Brush` ADR-0044), layer state (panel-painter-layers), stroke history estado (`ph2d-painter-stroke`), color picker mode (panel-painter-color).

### 2.4 Arch-gate `painter_contract_surface` — homestead `ph2d-painter-contracts` (T0.8, NÃO diferido)

**Localização canônica congelada:** `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`. Crate dedicado, **criado em T0.8 da cascata W0** (não em W1).

**Conteúdo do crate `ph2d-painter-contracts/`:**

```
crates/ph2d-painter-contracts/
  Cargo.toml         # SEM deps de runtime — só std + walkdir (audita texto)
  src/lib.rs         # #![forbid(unsafe_code)] — mod stub vazio
  tests/architecture_painter_contract_surface.rs
                     # Audita o TEXTO dos arquivos crates/ph2d-{tool-painter,
                     # painter-brush, painter-stroke, painter-mcp,
                     # panel-painter-inspector, painter-fluid}/src/**/*.rs
                     # via grep estrutural + walkdir.
                     # Em W0/T0.8 esses crates não existem ainda — testes
                     # passam vacuamente (no files found = ok).
                     # Em W1+ conforme crates filhos nascem, testes ativam
                     # automaticamente sem nova edição.
```

**Razão da revisão (audit 2026-05-26):** o deferral original "T0.8 desliza pra W1 T1.3" criava janela ~14 dias onde caps congelados em W0 não tinham enforcement. Audit A-6 sinalizou corretamente como "crate-fantasma mal-disfarçado". Crate dedicado de contratos sem deps de runtime resolve: zero acoplamento aos crates filhos, gate ativa lazy conforme eles nascem.

**Conteúdo enforcement (cobertura cumulativa ADR-0043..0049):** vide ADR-0044 §2.11 (tabela cumulativa de tests).

**T0.8 do plano §3 atualizado:** "Criar crate `ph2d-painter-contracts/` com arch-gate textual; sem deps de runtime; testes ativam lazy quando crates filhos nascem em W1+."

### 2.5 O que esta ADR **NÃO** decide (delegação congelada às ADRs irmãs)

- **Brush struct cap** (`Brush ≤ N campos`) — **ADR-0044** (modelo de cap duplo top-level + sub-structs, §2.2).
- **Stamp layout / alinhamento** — **ADR-0044** (96 bytes, repr(C, align(16)), ABI freeze).
- **`BrushHandle` tipo** — **ADR-0044 §2.8** (fonte única canônica).
- **`RenderingMode` / `PigmentMode` / `GrainSource` / `ProceduralGrain` enums** — **ADR-0044**.
- **`AdjustmentKind` + `AdjustmentParams` + `AdjustmentLayer`** — **ADR-0045**.
- **`StrokeRecord` schema + `StrokeHistory` enum + `.ph2d-painter` v1** — **ADR-0046**.
- **`StrokeId = Uuid` type alias** — **ADR-0046 §2.2** (fonte única).
- **MCP tool schemas (`StrokeSpec`, `StrokeMods`, `StrokeFilter`, `StrokeRef`)** — **ADR-0047**.
- **Inspector `InspectorState` + lasso geometric matching + recompose slice** — **ADR-0048**.
- **Fluid sim parameters + `FluidSim` + `FluidParams` + `GravitySource`** — **ADR-0049**.
- **`PlatformHost::gyroscope()` trait extension em `ph2d-host`** — **ADR-0049 §2.2** (sancionado: amend foundational autorizado por esta ADR-0043 §2.5 — único bump de trait foundational da cascata).
- **`MemoryBudget::fluid_capable()` + `PerfBudget::fluid_headroom_ms()` métodos inerentes em `ph2d-host`** — **ADR-0049 §2.9, §2.10**.
- **`MemoryBudget::stroke_history_mb` campo em `ph2d-host`** — **ADR-0046 §2.5** (sancionado: amend foundational autorizado por esta ADR-0043 §2.5).
- **Bump de `Tool`/`RasterEditTool`/`PanelEvent` caps** — frozen por ADR-0040+0041, intocados aqui.
- **Canvas-event surface trait** (substituto pro path `BrushTool` proto). v1 reusa o path existente; eventual trait-extraction é wave futura.

### 2.6 Tipos importados de fora — fonte canônica

Esta ADR usa tipos definidos em outros crates / ADRs. Para evitar deriva, cada tipo tem **fonte única**:

| Tipo | Definido em | Crate |
|---|---|---|
| `BrushHandle` | ADR-0044 §2.8 | `ph2d-painter-brush` |
| `OklchColor` | ADR-0042 | `ph2d-color` |
| `BlendMode` | (compositor) — TBD W4 home | `ph2d-painter-brush` ou `ph2d-painter-canvas` |
| `LayerId` | LayerStack data model — TBD W3 home | `ph2d-painter-brush` (ou crate futuro) |
| `CanvasId` | TBD W1 — `ph2d-editor-core` ou novo | `ph2d-editor-core` |
| `MaskData` | LayerStack — TBD W3 home | idem |
| `OklchColor`, `SrgbRgba`, etc. | ADR-0042 | `ph2d-color` |
| `ThumbHandle` | TBD W2 (sidebar) | `ph2d-painter-brush` (atlas-resident) |
| `SymmetryAxis` | W9 — TBD | `ph2d-tool-painter` (interno) |
| `NodeId` | ADR-0023 (a11y) | `ph2d-a11y` |
| `Token` (HR-11) | ADR-0047 §2.10 | `ph2d-painter-mcp` |
| `StrokeId = Uuid` | ADR-0046 §2.2 | `ph2d-painter-stroke` |
| `BrushParamsHash = [u8; 32]` | ADR-0044 §2.2 (blake3) | `ph2d-painter-brush` (re-export) |
| `ToolMode { Paint, Smudge, Erase }` | ADR-0046 §2.4 | `ph2d-painter-stroke` |
| `LayerStack` | TBD W3 (data model raster + adjustment) — provisório `ph2d-painter-brush` | TBD W3 ADR ratifica |

**Tipos "TBD W-N home"** são placeholders. Cada W-N que materializar o tipo registra em sua ADR de wave + commit que migra esta tabela. ADRs da cascata W0 **não pre-decidem** o crate-home; apenas registram a dep.

#### 2.6.1 `PainterMode` (UI) vs `ToolMode` (history) — mapping congelado

Audit 2026-05-26 flagged naming drift cross-ADR. Decisão: **dois enums separados** intencionalmente, com mapping fixo:

| `PainterMode` (UI sidebar — ADR-0043 §2.3) | ↔ | `ToolMode` (stroke history — ADR-0046 §2.4) |
|---|---|---|
| `Brush` | ↔ | `Paint` |
| `Smudge` | ↔ | `Smudge` |
| `Eraser` | ↔ | `Erase` |

**Razão:** UI usa nomes do usuário ("Brush", "Eraser" — botões da sidebar); history usa nomes do schema serializável ("Paint" é o ato; "Erase" é o tempo verbal canônico para serialização postcard). Conversão acontece em `PainterTool::handle_panel_event` (one-line `match`).

**Risco de divergência:** se variant nova for adicionada em um sem o par, gate `painter_mode_tool_mode_mapping_total` (em `painter_contract_surface` ADR-0044 §2.11 ou aditivo W1 T-N) falha.

---

## 3. Consequências

### Positivas

- **Drop-crate fan-out vale pro Painter.** Toda task W1..W15 que adiciona uma feature ao Painter cai em um destes baldes:
  - (A) nova variant em `PainterUiEdit` (dentro do cap 24) — fan-out drop-in;
  - (A) novo field em `PainterUiSnapshot` ou `PainterParams` (dentro do cap) — fan-out drop-in;
  - (A) novo crate satélite `ph2d-panel-painter-*` ou `ph2d-painter-*` — fan-out drop-in via DIRETRIZ §3.A;
  - (C) Coord-A ADR-amend (raríssimo).
- **Múltiplos implementadores paralelos.** W1 (brush engine), W2 (sidebar), W3 (layers panel) podem rodar em sessions paralelas sem colidir — cada crate-balde é isolamento FBP.
- **Surface mínima força disciplina.** Brush params (140 campos!) ficam ESCONDIDOS em `Brush` (ADR-0044); só o `BrushHandle` cruza pra `PainterParams`. Isso bloqueia o vazamento "brush studio invade sidebar" antes que possa acontecer.
- **Caps numéricos, não prosa.** ≤ 20 / ≤ 18 / ≤ 12 são testáveis. Quando bater o cap, o gate vermelho força ADR-amend (review explícita) — não silenciosa expansão.

### Negativas / Custos

- **PainterTool é o primeiro Tool 'workhorse'** — se a moldura `RasterEditTool` apertar (e.g., precisar de canvas-event hook trait-level), exige ADR-amend bumpando caps 10/5/4. Mitigação: spec já é v1 madura; risco identificado e watch-listed.
- **`BrushHandle` cross-crate** — `PainterParams.active_brush: BrushHandle` cria dep `ph2d-tool-painter → ph2d-painter-brush`. Já está no design (Cargo.toml §2.1), apenas registro o custo.
- **Caps são opinion-shaped.** 24/18/12 são chutes calibrados (audit 2026-05-26 bumpou 20→24 após contagem realista de variants por wave). Se W9+W11+W14 combinadas pedirem mais que 9 variants além do v1, vira ADR-amend (T-W-N.X). Aceito — é o trade-off do design "freeze cedo, amend explicitamente".

### Neutras

- **T0.8 do plano (gate vazio) desliza pra W1 T1.3.** Esta ADR documenta a motivação (sem crate-fantasma); plano §3 atualizado por aceitação T0.9.
- **Brush proto-tool de `editor-core` continua.** Esta ADR não força remoção; ela acontece na W1 T1.1 (PainterTool substitui o proto na registry via codegen tool-sync). Trade-off: o proto fica como fallback no caso (improvável) de o crate Painter falhar em W1; remove-se ao fim da wave.

---

## 4. Alternativas consideradas

### 4.1 Caps maiores (UiEdit ≤ 40, Snapshot ≤ 25)

**Rejeitada.** CEQ chegou a 41 variants por design defeituoso (slider+chip duplicados — vide ADR-0042 §5.2 split recipe). Imitar esse outlier no Painter perpetuaria o padrão. Caps apertados forçam disciplina (variants compostos com payload `enum SliderTarget { Size, Opacity, … }` em vez de `Size(f32)` + `Opacity(f32)`).

### 4.2 Caps menores (UiEdit ≤ 12, Snapshot ≤ 10)

**Rejeitada.** Painter sidebar TEM 10 pills na topbar + ~7 sidebar controls + ~3 quick-action affordances. Cap 12 quebra antes do W2 day 1.

### 4.3 Fundir `PainterTool` em `BrushTool` (extender o proto existente)

**Rejeitada.** Proto `BrushTool` vive em `editor-core/src/tools/brush.rs` — é exatamente a inércia que ADR-0040 sancionou contra. Painter é **a feature** que justifica satélite isolado: 17 waves, 10+ crates filhos, 7 ADRs. Estender proto seria reabrir god-crate.

### 4.4 `PainterParams` carrega `Brush` inteiro (não `BrushHandle`)

**Rejeitada.** `Brush` tem ~140 campos (ADR-0044). Fundi-lo em `PainterParams` mata o cap de 12 e viola §2.5 (separação contratos modulares). Handle indirection é canon em raster pipelines (atlas-resident).

### 4.5 Adiar o ADR (escrever caps "depois de W1")

**Rejeitada.** É exatamente o cenário "spec do papel sem caps efetivos" que DIRETRIZ §3.A.3 alerta. O valor de um contrato congelado é congelar **antes** do código tê-lo flexionado para si.

---

## 5. Verificação

Após esta ADR ser ratificada (T0.9) e W1 T1.3 criar o crate `ph2d-painter-brush`:

```sh
# Gate de contrato Painter (W1 T1.3+)
cargo test -p ph2d-painter-contracts --test architecture_painter_contract_surface
# Deve passar com caps: PainterUiEdit ≤ 24, PainterUiSnapshot ≤ 18, PainterParams ≤ 12.

# Drop-crate fan-out OK
cargo run -p ph2d-tool-sync
cargo test -p ph2d-tool-registry-init  # staleness gate verde

# Contratos congelados não foram tocados
cargo test -p ph2d-editor-core --test architecture_tool_contract_surface
# Tool=10 / RasterEditTool=5 / PanelEvent=4 intactos.
```

### 5.1 Definição de "Accepted"

Esta ADR transita `Proposed → Accepted` quando:

1. ✅ Enio leu e aprovou em T0.9 (smoke W0).
2. ✅ ADRs irmãs 0044-0049 escritas (T0.2..T0.7) — esta ADR depende textualmente delas para auto-coerência (§2.5 lista as fronteiras).
3. ✅ Commit do conjunto de 7 ADRs em main (T0.9).

Esta ADR fica `Proposed` (não bloqueia leitura/discussão) entre T0.1 e T0.9.

---

## 6. Tracking

- Plano operacional: [`docs/Painter_projeto/15_plano_de_implementacao.md`](../../Painter_projeto/15_plano_de_implementacao.md) §3 (W0) + §4 (W1).
- Handoff de continuidade: [`docs/HANDOFF_painter.md`](../../HANDOFF_painter.md).
- Spec normativa (16 docs): [`docs/Painter_projeto/`](../../Painter_projeto/).
- Quando esta ADR amendar (futuro): bump menor (caps) = nova ADR `0043-amendment-N.md`; rewrite estrutural = nova ADR principal + esta marca `Superseded by 00XX`.
- Próxima ADR na cascata W0: [ADR-0044 — Brush Engine GPU contract](0044-brush-engine-gpu.md) (T0.2).
