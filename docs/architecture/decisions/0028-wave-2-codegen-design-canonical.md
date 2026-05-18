# ADR-0028 — Wave 2: build-time codegen + design canonical sources + lint guards

**Status:** Accepted
**Data:** 2026-05-16
**Decisor(es):** Enio + LLM Coordenador
**Antecessor:** [ADR-0027 — Convention-by-discovery](0027-convention-by-discovery.md)

## Contexto

Wave 1 (ADR-0027) entregou:

- `ph2d-tool-registry` + `ph2d-tool-registry-init` infraestrutura
- 4 tool-crates (`make_square`, `grid_snap`, `bgremoval`, `trim_transparency`) com
  `MANIFEST` const + `register(&mut Registry)`
- Shell decomposition (`init.rs` / `forwarding.rs` / `hero_intents.rs`)
- HR-18 declarada (mas inativa)

Auditoria pós-Wave-1 em 2026-05-16 identificou 10 pontos de colisão remanescentes
que continuam impedindo trabalho multi-agente paralelo:

| # | Ponto | Sintoma |
|---|-------|---------|
| 1 | `tokens.json` divergia dos valores hard-coded em `color.rs` | Designer (Claude Design) editava JSON; Rust ignorava |
| 2 | `icons.rs::cmds()` tinha 715 LOC de match-arms manuais portados de SVG | Cada ícone novo = edit central |
| 3 | NodeId chrome alocados à mão em ranges 100..1099 | 6 colisões numéricas silenciosas pré-Wave 2 |
| 4 | `topbar_clusters()` hard-coded; tool nova precisava editar fixture central | Source-of-truth fragmentada |
| 5 | Sem source canonical para tool functionality | TOML ↔ Rust drift inevitável |
| 6 | Sem lint anti-`0xRRGGBB` literal em widget/screens | Regressão de tema silenciosa |
| 7 | Arquivos > 600 LOC sem gate (HR-18 declarada mas inativa) | God-files crescendo |
| 8 | `panel.rs` 2869 LOC, `state.rs` 1250 LOC monolíticos | Multi-agente colidia em 4119 LOC |
| 9 | `screens/hero.rs` 3300 LOC com 50+ campos de estado | Toda interação tocava esta god-struct |
| 10 | `main.rs` 2416 LOC com 20 `pending_X` drains | Action Bus pendente |

## Decisão

PH2D adota **codegen-from-design-canonical + lint-as-spec** como mecanismo
permanente:

### 1. `docs/design/` é canonical

Toda aparência (tokens, icons) e declaração funcional (tools) origina aqui.

- `docs/design/tokens.json` — 4 themes × 33 OKLCH color tokens.
- `docs/design/icons/*.svg` — 100 Lucide-derived glyphs (24×24 design space).
- `docs/design/tools/*.toml` — per-tool functional spec (id / cluster / zone / order /
  icon_slug / a11y_role / label / memory_budget).

### 2. `build.rs` codegen elimina sync manual

- `crates/ph2d-tokens/build.rs` lê `tokens.json` → emite `tokens_generated.rs` com
  const arrays de 4 themes resolvidos (`$inherits` aplicado).
- `crates/ph2d-editor/build.rs` lê `docs/design/icons/*.svg` → emite
  `icons_generated.rs` com per-icon `IconCmd` arrays + `ICON_CMDS_BY_ID` + `lookup_cmds`
  + `ALL_ICON_SLUGS`.

Cargo `cargo:rerun-if-changed=` em ambos garante rebuild automático.

### 3. Hash-derived NodeIds

`hash_node_id(s: &'static str) -> NodeId` em `ph2d-tool-registry::node_id`
(FNV-1a 64-bit const-fn) substitui ranges manuais em `screens/hero/ids.rs`. 250
consts migradas; 12 fixture row ids (HIER_PLAYER..HIER_MAIN_CAMERA) ficam numeric
porque participam de bit-flag math (`EYE_TOGGLE_BIT` / `EXPAND_TOGGLE_BIT`).

`COMPANION_ROW_ID_MAX = 2^32` guard nos detection helpers garante que hash-output
chrome (que pode acidentalmente setar bits 61/62) não seja mis-detectado como row
companion.

### 4. Chrome derivado do Registry

`paint_image_action_row()` consome `Registry::cluster("image_tools")` em vez de
lista hardcoded. NodeIds dos chrome consts (`IMAGE_ACTION_TRIM` etc.) usam os
MESMOS slugs que os manifest ids — uma única hash, mesma `NodeId`.

`ph2d-editor::install_registry()` + `installed_registry()` via `OnceLock` evitam
propagar `&Registry` por todos os painters.

### 5. Design ↔ Manifest cross-validation

`crates/ph2d-tool-registry-init/tests/tool_manifest_design_sync.rs` enforça
parity field-by-field entre cada `docs/design/tools/<slug>.toml` e o `MANIFEST`
const correspondente. Drift falha CI com diff legível.

### 6. Lint guards

- `no_literal_color.rs` em `ph2d-editor/tests/` bloqueia novos hex literals em
  `widget/**` e `screens/**`. Allowlist `// LITERAL-COLOR-OK: <reason>`.
- `chrome_manifest_coverage.rs` em registry-init enforça que chrome consts ↔
  manifest ids hashes alinhados.
- `node_id_collisions.rs` em ph2d-editor enforça uniqueness pairwise + reserva
  `NodeId::ROOT` + companion-bit safety.

### 7. HR-18 file LOC cap ativo

`shells/desktop/tests/file_loc_caps.rs` gates `.rs` files ≤ 600 LOC.
Exceções via `// ph2d-loc-cap: <reason>` no top do arquivo (primeiros 20
linhas). Duas exceções ativas hoje, ambas pendentes Wave 2.5:

- `shells/desktop/src/main.rs` — 2421 LOC (Action Bus PR 11.8 reduz).
- `shells/desktop/src/hero_intents.rs` — 696 LOC (mesma Action Bus reduz / remove).

Inventário automático em `loc_cap_exceptions_inventory` test emite a lista de
exceções a cada `cargo test` para visibilidade contínua.

## Wave 4 (2026-05-17) — source-of-truth UI extends to spacing/radius/stroke/typography

Wave 4 closes the **decoration-layer** armadilhas that survived Waves
1-3. Same three pillars (canonical source → codegen → lint guard), now
applied to the dimensional + temporal axis of UI.

### Pillar 1: design canonical extends

`docs/design/tokens.json` gains 5 new top-level sections (moved out
of `themes.forge` where applicable; stripped from per-theme blocks):

- `spacing` — 9 values `xxs..4xl` (2..48 px)
- `radius` — 7 values `xs..full` (4..999 px)
- `stroke` — 5 values `hairline..heavy` (0.5..3.0 px) — **new
  dimension**, no prior Rust enum
- `density` — 3 values `compact..comfortable` (22..32 px)
- `chrome` — 3 values `row-h`, `icon-btn-size`, `section-gap`
  (28, 36, 14 px)

`typography.{size, weight, line, track}` (already present) becomes
codegen-driven (was a manual mirror).

### Pillar 2: codegen extends

`crates/ph2d-tokens/build.rs` extended with `parse_scalar_block` /
`parse_pairs` (handles both multi-line and single-line JSON bodies)
and emits new const tables: `SPACING_*`, `RADIUS_*`, `STROKE_*`,
`DENSITY_*`, `CHROME_*`, `TYPOGRAPHY_SIZE_*`, `TYPOGRAPHY_WEIGHT_*`,
`TYPOGRAPHY_LINE_*`, `TYPOGRAPHY_TRACK_*`. Still zero build-deps
(ad-hoc parser, comma-split for blocks, suffix-stripped values).

Consumers (`Spacing::px`, `Radius::px`, `Density::row_h_px`,
`StrokeToken::px`, `TypeToken::px`, `FontWeight::value`,
`LineHeight::ratio`, `LetterSpacing::em`, plus
`SECTION_GAP_PX`/`ICON_BTN_SIZE_PX`/`ROW_H_PX` consts) now read from
`crate::generated::*` instead of hardcoded match arms.

New `StrokeToken {Hairline/Thin/Default/Thick/Heavy}` enum
re-exported from `ph2d_tokens::*`.

### Pillar 3: cross-validation + lint extends

- `crates/ph2d-tokens/tests/design_token_sync.rs` (new) — 9 tests
  re-parse `tokens.json` with `serde_json` (dev-only dep,
  independent of build.rs's ad-hoc parser) and assert every public
  token API agrees with the JSON. Drift fails CI with inline diff.
- `crates/ph2d-editor/tests/no_literal_color.rs` (extended) —
  matcher now catches `Color::WHITE/BLACK/TRANSPARENT`,
  `Color::{rgba8, rgb8, from_rgba8, from_rgba, from_rgb}(`, all of
  the above prefixed with `VelloColor::`. Same allowlist mechanism
  (per-line comment + `blender_color_picker/` path). 21 sites in
  the existing tree annotated as legitimate (token-cast bridges,
  alpha-checker tiles, drop-overlay scrim theme-invariant, note
  text on highlighter background).
- `crates/ph2d-editor/tests/no_magic_numeric.rs` (new, warn mode) —
  bans bare `\d+\.\d+` literals in `widget/**` and `screens/**`
  outside structural ratios `{0.0, ±0.5, ±1.0, ±2.0}`. Allowlist
  via `// LITERAL-PX-OK: <reason>` per-line +
  `blender_color_picker/` path. Mode toggle const flips warn→deny
  when the migration sweep zeroes.

### Migration sweep status

The walker found 493 actionable hits across 35+ files. Wave 4 lands
the **infrastructure** + migrates 5 files as demonstration:
`style.rs`, `inspector/sections.rs`, `topbar/cluster_painter.rs`,
`hierarchy/panel_painter.rs`, `hierarchy/row_painter.rs` (~154
sites done, 31%).

The remaining 339 sites in ~30 files are deferred to **Wave 4.1
dedicated** (top remaining: `inspector/showcase.rs` 63,
`hero.rs` 20, `inspector/mod.rs` 19, `selection.rs` 18,
`color_picker.rs` 15; long tail of 22 files with 1-10 hits each).
Until Wave 4.1 lands, the lint stays in `LintMode::Warn` — CI
remains green, inventory is visible in stdout for any reviewer.

### Métricas Wave 4

| Métrica | Pré-Wave-4 | Pós-Wave-4 |
|---------|------------|-------------|
| `tokens.json` top-level sections (excl. `$meta`/themes/motion/z) | 1 (typography) | 6 (spacing, radius, stroke, density, chrome, typography) |
| Rust enums consuming codegen | 1 (`ColorToken`) | 9 (`+ Spacing, Radius, StrokeToken, Density, TypeToken, FontWeight, LineHeight, LetterSpacing`) |
| Color lint coverage | hex literals only | hex + `Color::{WHITE,BLACK,TRANSPARENT}` + `Color::{rgba8,from_rgba8,...}` + `VelloColor::*` |
| Magic-numeric lint | none | warn-mode in widget+screens; 154/493 sites migrated |
| Cross-validation tests | 1 (`tool_manifest_design_sync`) | 2 (`+ design_token_sync` with 9 sub-tests) |

When Wave 4.1 closes, an agent paralelo adicionando widget novo
**não pode**:
- Usar spacing/radius/stroke/font-size não-token (lint bloqueia)
- Usar `Color::WHITE` ou `from_rgba8` hardcoded (lint estendido)
- Drift JSON↔Rust em qualquer das 9 dimensions (sync test)

## Wave 2.5 — débito conhecido

Os splits internos a seguir foram deferidos por demandarem 2+ sessões cada e
não bloqueam o objetivo de "convention-by-discovery operacional":

- **PR 11.7a** — `grid_snap/panel.rs` (2869 LOC) → multi-file split per kind.
- **PR 11.7d** — `screens/hero.rs` (3300 LOC) → decompor `HeroScreen` state em
  `InspectorState` + `HierarchyState` + `ImageEditState` + `ViewState`.
- **PR 11.8** — Action Bus + drain residuals (main.rs < 400, hero_intents.rs
  colapsa via dispatcher).
- **PR 11.10** — Golden image tests (Vello headless rendering).
- **PR 11.11** — `lib.rs` trim aggressive (`pub use` cleanup com path
  migration de ~30-50 sites).

Plano em [docs/Migracao/2026-05-wave-2-eliminating-all-collisions.md](../../Migracao/2026-05-wave-2-eliminating-all-collisions.md)
(seções PR 11.7a/d/8/10/11).

## Consequências

### Positivas

- **Designer edita TOML/SVG/JSON; Rust replica automaticamente.** Zero sync
  manual entre design canonical e implementação.
- **Adicionar tool nova é 1 crate + 1 linha** em registry-init. Coordenador
  revisa, agente periférico executa, zero contato com chrome painters.
- **Colisões silenciosas eliminadas** — hash NodeIds + cross-validation tests
  fazem qualquer drift falhar build, não comportamento.
- **HR-18 ativo** previne god-file growth daqui pra frente.

### Negativas / a aceitar

- `build.rs` introduz dependência de `OUT_DIR` em todos os builds (compile time
  ligeiramente maior, mas codegen é simples + cached).
- Hash-derived NodeIds tornam debugging com `gdb` menos intuitivo (NodeId(0xAF63...)
  em vez de `NodeId(102)`); compensado pelos `tests/node_id_collisions.rs` com
  printable diagnostics.
- Wave 2.5 debt explícito — main.rs / hero.rs / panel.rs ainda god-files. HR-18
  excepts marker os mark, e a inventory test os surfaces a cada run.

## Métricas

| Métrica | Pré-Wave-2 (2026-05-15) | Pós-Wave-2 (2026-05-16) |
|---------|-------------------------|--------------------------|
| Manual NodeId consts | 253 | 0 (hash-derived) |
| Manual icon match-arms (icons.rs) | 715 LOC | 0 (codegen) |
| Manual color resolve fns (color.rs) | 4 × ~50 LOC = 200 LOC | 0 (codegen) |
| Sources of truth para tools | 4 fragmented (fixture + topbar + icons + color) | 1 canonical (`docs/design/tools/*.toml`) |
| Architecture tests | 1 (interaction_no_alloc) | 7 (collisions / coverage / design-sync / lint / cap × 2 / no-literal-color) |
| Files over 600 LOC (shells) | 2 ungated | 2 with explicit exceptions, cap active |
| Tests verde (workspace) | 1235 | 1296 |

## Status de migração

- ✅ PR 11.1.0 — tokens.json reset to Round 9 (c4f0da6)
- ✅ PR 11.1 — tokens build.rs (aa5331c)
- ✅ PR 11.2 — icons build.rs (e9577d7)
- ✅ PR 11.3 — NodeId hash universal (5e54638)
- ✅ PR 11.4 — Chrome derivado do Registry (8f82407)
- ✅ PR 11.5 — Design canonical TOMLs (7661a7d)
- ✅ PR 11.6 — no-literal-color lint (3d57906)
- ✅ PR 11.7c — topbar.rs split (bc5a456)
- ✅ PR 11.7b — hierarchy.rs split (8843d01)
- ✅ PR 11.9 — HR-18 file-LOC cap ativo (f2cbb20)
- 🔜 Wave 2.5: PR 11.7a, 11.7d, 11.8, 11.10, 11.11

### Wave 4 (2026-05-17, stage A+B+C + partial D)

- ✅ Stage A+B — spacing/radius/stroke/density/chrome top-level + typography codegen (`b84b74b`)
- ✅ Stage C — `no_literal_color` matcher extends to non-hex paths (`1dc8487`)
- ✅ Stage D.1 — `no_magic_numeric` lint infra (warn) + 2 demo files (`ccf1ff9`)
- ✅ Stage D.2 — 3 more painter files (`3463fc7`)
- ✅ Wave 4.1 — Stage D sweep completion: 493/493 sites migrated; `no_magic_numeric` flipped to `LintMode::Deny`; CRLF normalize in lint walkers (windows CI fix); section outline + user notes regressions fixed (`b8bf68c` + `dc978d1` + `1917860` + `890654b` + `4109a70`)

## Wave 5 (2026-05-17) — chrome canonical + HeroScreen state decomp + panel-as-canonical pattern

Wave 4.1 closed the **layer-of-decoration** colisões (every magic
numeric in widget/screens forced into a token). Wave 5 closes the
two architectural residuals that survived: chrome layout dimensions
still owned by Rust, and panels NOT being canonical-source units.

### Pillar 1: chrome layout to tokens.json (Stage A)

`docs/design/tokens.json::chrome` gains 17 new entries — every
fixed dimension that previously lived as a hardcoded `f32` literal
in `screens/hero/style.rs` (`HERO_VIEWPORT_W/H`, `EDGE_PAD`,
`TOPBAR_H/GAP`, `INSPECTOR_W`, `HIERARCHY_W`, `HUD_H/BOTTOM_PAD`,
`PANEL_RADIUS/HEAD_PAD`, `HIER_ROW_H`, `PANEL_RESIZE_HANDLE_SIZE`)
or in widget modules (`TOOL_CHIP_PX`, `DIVIDER_GAP_PX`,
`PILL_PADDING_PX`, `CHECKBOX_BOX_PX`). New `crates/ph2d-tokens/src/chrome.rs`
module re-exports 17 `pub const *_PX: f32` from `crate::generated::CHROME_*`.

`design_token_sync::chrome_consts_match_tokens_json` extended from
3 → 20 keys. Designer fully owns the dimensional side of UI.

### Pillar 2: HeroScreen state decomp (Stage B)

`HeroScreen`'s 21 flat state fields grouped into 6 cohesive
sub-state structs in new `screens/hero/state.rs`: `InspectorState`
(6 fields), `HierarchyState` (3), `ImageEditState` (2),
`ViewState` (5), `GizmoStateGroup` (3), `GridState` (3). Top-level
drops from 33 → 17 fields (3 identity + 3 UI machinery + 6 group
structs + 5 misc shell-publication channels).

~129 call sites migrated mechanically (`hero.inspector_sprite` →
`hero.inspector.sprite`, etc.). Pre-req for stage C — each panel
can now own its state group instead of poking flat HeroScreen
fields scattered across the god-struct.

### Pillar 3: PanelManifest infrastructure (Stage C)

New `crates/ph2d-editor/src/panel_registry.rs` mirrors
`ph2d-tool-registry`'s `ToolManifest` + `Registry`:

- `PaintCtx<'a>` — per-frame ref bundle (hero, layout, viewport,
  scene, text_system).
- `PanelManifest` — id, panel_node_id, default_visible, three fn
  pointers (paint / apply_event / populate).
- `PanelRegistry` + `PANEL_REGISTRY` static — append-only slice of
  manifests with `find_by_panel_node_id`.

Each of the 4 panel modules (`widget_gallery`, `hierarchy/`,
`inspector/`, `grid_snap`) exports `pub static PANEL_MANIFEST`.

### Pillar 4: paint_hero_screen collapse (Stage D)

Each panel's `paint_fn` thunk owns the full per-frame logic
(visibility early-return + lazy default rect + drag/resize clamp +
chrome publish + actual paint + content_h publish + scroll clamp +
stale-rect cleanup on hide). Shared `style::clamp_panel_rect`
helper extracted from a closure that lived inside `paint_hero_screen`.

`paint_hero_screen` collapses its 4 hardcoded per-panel paint blocks
(~280 LOC) into a single z-ordered loop over
`PANEL_REGISTRY.find_by_panel_node_id`. hero.rs: 3260 → 3027 LOC.

`apply_event_fn` thunks remain `false`-returning stubs — the
`HeroScreen::apply_event` god-match stays the per-event dispatcher.
Event canonicalization is a follow-up wave.

### Wave 5 status

- ✅ Stage A — chrome layout dims to tokens.json (`e26622b`)
- ✅ Stage B — HeroScreen state decomp (`4d8d6ad`)
- ✅ Stage C — PanelManifest + PanelRegistry infrastructure (`9d2d687`)
- ✅ Stage D — paint_hero_screen collapses to registry iteration (`9c93dce`)
- 🔜 Wave 6 (optional) — extract `crates/ph2d-panel-<slug>/` per panel; defer until concrete multi-agent demand on a panel.

### Métricas Wave 5

| Métrica | Pré-Wave-5 | Pós-Wave-5 |
|---------|------------|------------|
| Hardcoded chrome `f32` literals in style.rs + 3 widget files | 17 | 0 (token-driven) |
| `tokens.json::chrome` keys | 3 | 20 |
| Flat fields on `HeroScreen` | 33 | 17 (6 group structs + 11 misc) |
| Per-panel paint blocks in `paint_hero_screen` | 4 hardcoded (~280 LOC) | 1 registry iteration (~15 LOC) |
| Panel registration cost | edit hero.rs + ids.rs + populate-list-call | 1 `PANEL_MANIFEST` + 1 line in `PANEL_REGISTRY` |
| hero.rs LOC | 3260 | 3027 |
| Architecture tests | 9 | 9 (chrome_consts_match_tokens_json extends 3→20 keys) |

## Wave 6+7 (2026-05-17) — hotspot decomp + editor-core primitives crate

Wave 5 left 3 god-files intactos (dispatch/mod.rs 3392 LOC, gizmo.rs
1770 LOC, inspector/showcase.rs 1217 LOC) e o `apply_event` god-match
em `hero.rs` (798 LOC). Wave 6+7 ataca os hotspots E destranca o
caminho pra panel-as-crate criando a infraestrutura `ph2d-editor-core`
(primitives compartilhados sem orchestrator).

### Phase 1 — Hotspot decomp (3 commits)

| Hotspot | Antes | Depois | Δ |
|---|---|---|---|
| `interaction/dispatch/mod.rs` | 3392 | 279 | −92% |
| `gizmo.rs` (monolítico) | 1770 | 45 (mod.rs) + 6 sub-files | −97% mod.rs |
| `inspector/showcase.rs` (monolítico) | 1217 | 155 (mod.rs) + 11 per-section files | −87% mod.rs |

Padrões:

- **Dispatch split** (Phase 1.A) — extrai `dispatch_pointer_with_text`
  (god-function de 893 LOC) pra `pointer.rs`; helpers file-private
  vão junto. Tests inline movem pra `tests.rs` via `mod tests;`.
- **Gizmo split** (Phase 1.B) — `gizmo/` directory com per-domain
  files: `drag.rs` (state machine), `camera.rs` (config), `transform.rs`
  (math), `hit.rs` (id classification + ids module), `paint.rs`
  (GizmoView + paint_sprite_gizmo + helpers), `tests.rs`. mod.rs vira
  re-export hub.
- **Showcase split** (Phase 1.C) — per-section painters
  (`inputs.rs`, `slider.rs`, `switches.rs`, `lists.rs`, `vector.rs`,
  `status.rs`, `color.rs`, `actions.rs`, `identity.rs`, `card.rs`) +
  `body.rs` (orchestrator) + `mod.rs` (shared helpers + consts). Cada
  sub-file faz `use super::*;` pegando o shared scope da mod.rs.

### Phase 1 polish — 4 UX bug fixes (mesmo commit que Phase 2 foundation)

Bugs descobertos durante smoke do Phase 1:

1. **BlenderColorPicker drag bridge** — NumberInput drag em channel
   chip não propagava cor pro parent picker (commit path já tinha
   o bridge via `apply_blender_channel_value`; drag path faltava).
2. **Slider clamp** — NumberInput drag exibia valores fora do range
   0..1 do slider linkado. Clamp adicionado tanto pra linked_slider
   quanto BlenderPicker channel chip.
3. **Enter blur (TextInput single-line)** — Enter inseria `\n` literal
   em todos TextInputs. Agora: multi-line opt-in via
   `WidgetStore::mark_multiline_text(id)` (TextArea, note bodies);
   single-line default = Submit + Blur (convention de form-field).
4. **Enter blur (NumberInput)** — Enter commitava buffer mas mantinha
   foco. Agora: Submit → reset visual state → Blur (matching
   single-line TextInput).

### Phase 2 — `ph2d-editor-core` crate (engine primitives)

Novo crate `crates/ph2d-editor-core/` housing engine-level primitives
que panel crates (`ph2d-panel-*`) ou downstream shells podem
consumir sem depender do `ph2d-editor` orchestrator.

**Migrado em Phase 2 batch (1 commit):**

- `widget/` (~12000 LOC) — todos os primitives (Button, Slider, Toggle,
  RadioGroup, TextInput, TextArea, NumberInput, Combobox, Dropdown,
  Tabs, Tag, Avatar, Card, Checkbox, ColorSwatch, ColorPicker,
  BlenderColorPicker, ContextMenu, Divider, IconButton, ListItem,
  Modal, PillGroup, Popover, ProgressBar, ScrollBar, SectionHeader,
  SliderWithChip, Spinner, StatusBar, TreeView, ToolRail,
  Vector3Editor).
- `interaction/` (~10000 LOC) — dispatch + state + drag + event + hit
  + types + util. Refs `crate::screens::hero::ids` reescritas pra
  `crate::ids`.
- `ids` (629 LOC) — todos os NodeId constants hero/widget.
  `SECTION_IDS` + `LIVE_SECTION_IDS` (inspector) + `GS_PANEL`
  (grid_snap) consolidados aqui pra dispatch poder query sem
  depender back em ph2d-editor. Inspector + grid_snap re-export.
- `paint`, `grid`, `gizmo/`, `icons` + `build.rs` (SVG codegen),
  `floating_panel`, `project`, `toast`, `zen`, `zones` — primitives
  e helpers que widgets/dispatch consomem.
- Architecture tests `hr12_widgets_a11y` + `hr15_no_hardcoded_ui_strings`
  movem com `widget/` pra editor-core. `no_literal_color` +
  `no_magic_numeric` ficam em ph2d-editor mas scan_roots extendem
  pra `../ph2d-editor-core/src/widget`.

**Fica em ph2d-editor** (Phase 2 follow-up + Phase 3+):

- `screens/hero.rs` + `screens/hero/{state,fixture}.rs` — HeroScreen
  + sub-state + Info types. Mover pra editor-core exige refatorar
  `apply_event` (orphan rule: impl HeroScreen em editor-core não
  pode chamar chrome painters em ph2d-editor). Daria pra fazer com
  `apply_event` distribuído via PanelManifest thunks (Phase 4),
  mas é trabalho não-trivial.
- `panel_registry` — depende de HeroScreen + HeroLayout.
- `action_bus` — depende de Info types.
- `tool` + `tools/` — depende de floating_panel (já em core, daria
  pra mover; deferido pra reduzir batch size).
- `image_edit/`, `grid_snap/`, `screens/hero/{topbar,inspector,
  hierarchy,widget_gallery,…}/` — chrome + panels. Panel extraction
  é Phase 3 (deferido).

### Public API preserved

`pub use ph2d_editor_core::{floating_panel, gizmo, grid, icons,
interaction, paint, project, toast, widget, zen, zones, ids};` em
`ph2d_editor::lib.rs`. `screens::hero::ids` re-exporta editor-core::ids.
Inspector re-exporta `SECTION_IDS`/`LIVE_SECTION_IDS`. Grid_snap
re-exporta `GS_PANEL`. Zero downstream changes (shells, tool crates).

### Wave 6+7 status

- ✅ Phase 1.A — dispatch split (`326e18b`)
- ✅ Phase 1.B — gizmo split (`bbb8690`)
- ✅ Phase 1.C — showcase split (`316af75`)
- ✅ Phase 1 polish 4 bugs + Phase 2 foundation (4 leaf modules + bug
  fixes) (`cfad551`)
- ✅ Phase 2 batch — widget + interaction + paint + icons + grid +
  gizmo + floating_panel + ids consolidado (`53db463`)
- ✅ Phase 4 partial — `PANEL_REGISTRY.manifests()` iteration wired
  at top of `HeroScreen::apply_event` (`2440552`). 2 panel thunks
  filled (widget_gallery::GAL_CLOSE, grid_snap::GS_COLOR_PICKER).
- ✅ **Phase 4 completion** (`5ebcb22`) — all 4 panel thunks fully
  populated. New modules `hierarchy/apply_event.rs::apply_event_full`
  (HierReparent + companion bits + CTX_MENU_HIER_* + live row
  click/DoubleClick/LongPress + HIER_RENAME_INPUT) and
  `inspector/apply_event_full.rs::apply_event_full` (Reimport +
  Transform commits + Reset + Visibility + Render Strategy +
  entity-name). widget_gallery thunk extends to TOPBAR_WIDGET_GALLERY.
  grid_snap thunk extends to TOPBAR_GRID_SETTINGS + full delegation
  to `grid_snap::apply_event`. `HeroScreen::apply_event` god-match
  collapses 798 LOC → ~380 LOC (chrome-only: theme/radius menus,
  rail toggles, tool buttons, settings cascades, scene list,
  image-edit actions). `hero.rs` total: 3030 → 2620 LOC (−14%).
- 🔜 **Phase 2 follow-up** (deferido — HeroScreen + state + fixture +
  Info types + panel_registry + action_bus + tool migrations; not
  required for Phase 3 anymore since panel crates can depend on
  ph2d-editor for HeroScreen — `apply_event_fn` are free fns so
  no orphan-rule issue).

## Wave 7 Stage 1 (2026-05-18) — PANEL_REGISTRY runtime-init + cargo features

Lays the infrastructure for Phase 3 panel-as-crate extraction without
physically moving any panel yet. Cargo features per panel work TODAY
for the in-tree panels — lite-build distributions can drop panels
they don't need at compile time.

### Changes (`257740c`)

- `PANEL_REGISTRY`: `pub static PanelRegistry` (hardcoded 4 panels)
  → `pub static OnceLock<PanelRegistry>` (runtime-init).
- `PanelRegistry` field: `&'static [&'static PanelManifest]` → 
  `Vec<&'static PanelManifest>` (runtime aggregation).
- Public API: `install_panel_registry(reg)`, `panels()`,
  `find_panel_by_node_id(id)`, `default_panel_registry()`,
  `PanelRegistry::new_empty()`, `PanelRegistry::push(&MANIFEST)`.
- `HeroScreen::new` auto-installs the default registry if no
  registry installed — backward-compatible with every existing
  host/test.
- New crate `crates/ph2d-panel-registry-init/`:
  - `default = ["panel-widget-gallery", "panel-hierarchy",
    "panel-inspector", "panel-grid-snap"]` — bundles every panel.
  - `--no-default-features` — empty registry (lite-build).
  - Single-feature builds (`--no-default-features --features
    panel-inspector`) — feature-gated subset.
  - Self-test asserts panel count = sum of enabled features (passes
    for all 3 configurations tested locally).

### Wave 7 status

- ✅ Stage 1 — PANEL_REGISTRY runtime-init + ph2d-panel-registry-init
  crate w/ cargo features (`257740c`).
- ✅ Stage 2 — panel-as-crate extraction. 4 new crates:
  - `ph2d-panel-widget-gallery` — full physical extraction (real
    paint/apply_event/populate thunks in its `lib.rs`; delegates to
    `inspector::paint_showcase_body` which is now `pub`).
  - `ph2d-panel-grid-snap`, `ph2d-panel-inspector`,
    `ph2d-panel-hierarchy` — alias form (each `lib.rs` just
    `pub use ph2d_editor::<path>::PANEL_MANIFEST`). Keeps consumer
    dep paths stable so Wave 8 can physically migrate each body
    without touching downstream code.
  - 17 `screens/hero/style.rs` items bumped to `pub`;
    `paint_showcase_body` + 12 inspector::state helpers bumped to
    `pub` so external panel crates can reach them.
  - `ph2d-panel-registry-init` Cargo.toml gains 4 optional path-deps
    (one per panel); `build_registry()` pushes manifests from the
    panel crates (not in-tree modules). Default-all + lite-build +
    feature-subset all tested.
- 🔜 **Stage 3 — Hotspot decomp follow-up** (deferred — same Phase 1
  pattern, targets `inspector/mod.rs` 857 LOC + `grid_snap/state.rs`
  1250 LOC + `screens/hero.rs` 2620 LOC).
- 🔜 **Stage 4 — Wave 8 panel-body physical migration** (deferred —
  move `grid_snap/`, `inspector/`, `hierarchy/` implementation files
  from `crates/ph2d-editor/src/` into their `ph2d-panel-*` crates;
  alias `lib.rs` becomes the full module tree. ~10k LOC moved total;
  alias pattern means consumer dep paths stay constant).
- 🔜 **Stage 5 — Shells migration to editor-core direct paths**
  (deferred — low ROI, `pub use ph2d_editor_core::*;` re-export
  already works transparently).

### Métricas Wave 7 Stage 1+2

| Métrica | Pré-Wave-7 | Pós-Wave-7-S2 |
|---|---|---|
| Crates no workspace | 31 | 36 (+5: editor-core já tinha, panel-registry-init + 4 panel crates) |
| PANEL_REGISTRY init | hardcoded static | OnceLock runtime |
| Cargo features per panel | nenhum | 4 (default-all + per-panel) + alias-crate wiring |
| Panel crates extraídas | 0 | 4 (1 physical full + 3 alias) |
| `pub(crate)`/`pub(super)` items bumped pra `pub` (Stage 2) | — | 30+ (hero style + showcase + state helpers) |
| Tests panel-init crate | — | 1 (passes for default/empty/single-panel/feature-subset) |
| Public API breaks | — | 0 (downstream paths unchanged) |


- 🔜 **Phase 5** (deferido — ph2d-panel-registry-init + cargo features
  per panel + lite build + shells migration pra editor-core direto)

### Métricas Wave 6+7

| Métrica | Pré-Wave-6+7 | Pós-Wave-6+7 (Phase 1+2) |
|---|---|---|
| `interaction/dispatch/mod.rs` LOC | 3392 | 279 |
| `gizmo.rs` (mod.rs) LOC | 1770 | 45 |
| `inspector/showcase.rs` (mod.rs) LOC | 1217 | 155 |
| Crates em workspace | 30 | 31 (+`ph2d-editor-core`) |
| ph2d-editor lib tests | 732 | 275 (rest migraram) |
| ph2d-editor-core lib tests | — | 457 |
| Combined lib tests | 732 | 732 (unchanged) |
| HR-18 cap shells/desktop | enforced (600) | enforced (600) |
| Public API breaks | — | 0 (via re-exports) |

## Referências

- **[Narrativa completa: problema multi-agente paralelo + solução](../../Migracao/PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md)** ← *começar por aqui se for novo no projeto*
- [Plano Wave 2 canonical](../../Migracao/2026-05-wave-2-eliminating-all-collisions.md)
- [SKILL §HR-18](../../../SKILL_Stack_PH2D_Definitiva.md#hr-18--crescimento-bounded-em-shell-binaries)
- [ADR-0027 — Convention-by-discovery (Wave 1)](0027-convention-by-discovery.md)
