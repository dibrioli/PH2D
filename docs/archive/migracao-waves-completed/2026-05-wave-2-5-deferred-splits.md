# Wave 2.5 — Deferred file splits + Action Bus

> **Contexto:** Esta wave é uma fase de uma migração maior contra
> colisões multi-agente paralelas. Para a narrativa completa
> (problema → diagnóstico → 4 waves de solução), vide
> [`PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md`](PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md).


**Status:** Parcialmente entregue 2026-05-16. PR 11.7a + 11.11 +
11.8 foundation mergeadas em origin/main; PR 11.7d / 11.8b/c/d /
11.10 deferidas para Wave 3 com sessão dedicada (vide
[`2026-05-wave-3-deferred-state-decomp-and-golden-images.md`](2026-05-wave-3-deferred-state-decomp-and-golden-images.md)).

## Entregues (2026-05-16)

- **PR 11.7a** (`3f42972`) — `grid_snap/panel.rs` 2869 LOC → 7
  sibling files (`mod.rs`, `orchestrator.rs`, `populate.rs`,
  `paint_helpers.rs`, `paint_kinds.rs`, `paint_rows.rs`,
  `events.rs`). CI 10/10 verde.
- **PR 11.11** (`863a2ca`) — `lib.rs` widget/tools/image_edit
  re-exports removidos (eliminou ~84 zona-de-merge re-exports).
  Consumers usam paths longos (`ph2d_editor::widget::Button` etc.).
  ~12 sites no shell atualizados. CI 10/10 verde.
- **PR 11.8 foundation** (`8cbfe4e`) — `action_bus.rs` lança o
  EditorAction enum + ActionBus queue. Infrastructure only; 7
  unit tests pinning FIFO order + drain semantics. Consumers
  (pending_X migrations) deferidos para Wave 3 PR 11.8b/c/d.
  CI 10/10 verde.

## Deferidas para Wave 3
**Origem:** PRs deferidos do plano Wave 2 (vide
[2026-05-wave-2-eliminating-all-collisions.md](2026-05-wave-2-eliminating-all-collisions.md))
por demanda de tempo/contexto inviável em uma única sessão.

## Por que existe esta Wave

Wave 2 entregou todos os ganhos arquiteturais (convention-by-discovery operacional,
chrome derivado, design canonical, lints anti-regressão, HR-18 ativo). O que sobrou
são splits internos de arquivos god-files (LOC hygiene) e a Action Bus, que demandam
~2-3h cada por requerer:

1. Refactor profundo de estruturas internas
2. Smoke visual obrigatório após cada split
3. Análise cuidadosa de boundaries para evitar quebrar comportamento

## PRs

### PR 11.7a — `grid_snap/panel.rs` split

**Origem:** 2869 LOC monolítico → multi-file per kind.

**Plano:**

```
panel/mod.rs              (~400 LOC) — populate + paint orchestrator + state accessors
panel/paint_helpers.rs    (~500 LOC) — section_label, snap_toggle, segmented_button,
                                       kind_button_grid, target_button_stack,
                                       neighborhood_button_row, labeled_segmented_row,
                                       color_swatch_row, number_row variants
panel/paint_kinds.rs      (~550 LOC) — paint_kind_config dispatch + 8 per-kind painters
                                       (square, hex, iso, staggered_sq, tri, quadtree,
                                       voronoi, chunks)
panel/paint_universals.rs (~250 LOC) — paint_origin_rows, paint_aabb_rows,
                                       paint_show_overlay_row, paint_opacity_slider_row,
                                       paint_labeled_toggle
panel/events.rs           (~400 LOC) — apply_event, apply_toggle, apply_value_changed,
                                       write helpers, apply_click
panel/events_value.rs     (~400 LOC) — apply_value_changed (extracted if events.rs > 600)
```

`state.rs` (1250 LOC) split semelhante:
```
state/mod.rs              (~400 LOC) — GridSnapState root + active kind dispatch
state/per_kind.rs         (~600 LOC) — SquareCfg, HexCfg, IsoCfg, etc.
state/snap.rs             (~250 LOC) — SnapPolicy, SnapTarget, magnetism
```

### PR 11.7d — HeroScreen state decomp

**Origem:** `screens/hero.rs` 3300 LOC com 50+ campos em `HeroScreen` struct.

**Plano:**

Extrair sub-structs cohesivos:

```rust
pub struct HeroScreen {
    pub id: NodeId,
    pub theme: Theme,
    pub inspector: InspectorState,        // panel visibility, transform edit, render strategy
    pub hierarchy: HierarchyState,        // visibility, live entries, search, rename
    pub image_edit: ImageEditState,       // image_tools_mode, pending_trim/make_square/bgremoval
    pub view: ViewState,                  // grid_visible, ui_mirrored, stats_visible,
                                          // widget_gallery_visible
    pub store: WidgetStore,
    pub hit_index: HitIndex,
    // ... actions / pending_* drain into Action Bus (PR 11.8)
}
```

Cada sub-struct ganha `Default` + métodos próprios. `apply_event` divide em
`inspector_apply_event` / `hierarchy_apply_event` / etc.

**Risco:** ALTO. ~80 sites internos referenciam `hero.<field>` direto;
todos precisam atualizar para `hero.<group>.<field>`.

### PR 11.8 — Action Bus + drain residuals

**Origem:** `shells/desktop/src/main.rs` 2416 LOC, `hero_intents.rs` 696 LOC.
20 `pending_X` drains inline em `render_frame()`.

**Plano:**

Action Bus em `ph2d-editor::action_bus`:

```rust
pub enum EditorAction {
    TrimTransparency { entity: u64 },
    MakeSquare { entity: u64 },
    ActivateBgRemoval,
    SaveProject,
    OpenProject(PathBuf),
    SetPixelsPerMeter(u32),
    SetDisplayUnit(DisplayUnit),
    // ... 20 variants total
}

pub struct ActionBus { queue: Vec<EditorAction> }
impl ActionBus {
    pub fn push(&mut self, action: EditorAction);
    pub fn drain(&mut self) -> impl Iterator<Item = EditorAction>;
}
```

`HeroScreen` perde os campos `pending_X`; em vez, `apply_event` empurra
`EditorAction` no bus. Shell dreina uma vez por frame e dispatcha via
`apply_editor_action(action, gfx, host)` em lugar dos drains inline.

Resultado:
- `main.rs::render_frame` → < 400 LOC
- `hero_intents.rs` → colapsa (drains migram para `apply_editor_action`)
- Removable `// ph2d-loc-cap` exception markers em ambos

### PR 11.10 — Golden image tests por widget

**Origem:** validação visual contra baseline; designer pode re-emitir mockup e
CI detecta divergência.

**Plano:**

```
crates/ph2d-editor/tests/golden/
├── Cargo.toml-included via dev-deps
├── golden.rs                — helper: compare(scene, baseline_path) via SSIM
├── widget_button.rs         — 1 baseline PNG por estado
├── widget_slider.rs
├── ... (30 widgets)
└── baselines/
    ├── widget_button_normal.png
    ├── widget_button_hovered.png
    └── ... (~150 PNGs, 256×128, ~10KB cada → ~1.5 MB total committed)
```

Vello headless renderer rodando em CPU mode (sem GPU em CI). SSIM threshold
0.985.

**Risco:** MÉDIO. Setup headless Vello pode ter quirks (config CPU-only,
font fallbacks, MSAA).

### PR 11.11 — `lib.rs` trim aggressive

**Origem:** `crates/ph2d-editor/src/lib.rs` 135 LOC com ~50 `pub use widget::*`
re-exports criando zona de merge.

**Plano:**

Reduzir a `~30 LOC`. Manter apenas re-exports load-bearing:
`paint_hero_screen`, `HeroScreen`, `Layout`, `Zone`, `Theme`, `ZenMode`,
`ToastQueue`, `NodeId`.

Consumidores em `shells/desktop/src/*.rs` (~30-50 sites) atualizam paths:
`ph2d_editor::Button` → `ph2d_editor::widget::Button`.

**Quebra de API pública** — SKILL §12.3 marca PH2D pré-1.0 ("0.x.y aceita
quebras em x"). Pré-condição: PR 11.10 (golden tests) usam paths longos
desde início.

## Ordem recomendada

```
PR 11.7a  (grid_snap split)        ← ~2-3h
PR 11.7d  (hero state decomp)      ← ~3-4h, ALTO risco
PR 11.8   (Action Bus)             ← depends on 11.7d; ~2h
PR 11.10  (golden images)          ← independente; ~2h
PR 11.11  (lib.rs trim)            ← depends on 11.10; ~30min mecânico
```

Total estimado: **10-12 horas de sessão dedicada** (não cabe em uma janela
de contexto LLM padrão).

## Critério de fechamento Wave 2.5

- `cargo test --workspace` verde após cada PR.
- Smoke visual após cada split mecânico (Enio confirma "smoke OK").
- HR-18 inventário (`loc_cap_exceptions_inventory`) → NONE.
- ADR-0028 status atualizado para incluir Wave 2.5 conclusão (mesma seção
  "Status de migração").

## Não-prioridade

PR 11.10 (golden images) pode esperar Wave 3 sem prejuízo arquitetural — é
validação visual, não convention infrastructure.
