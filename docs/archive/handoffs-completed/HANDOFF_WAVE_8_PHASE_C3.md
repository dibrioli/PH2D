# HANDOFF — Wave 8 Phase C.3 (retomada após Phase C.2)

**Data:** 2026-05-19 madrugada (UTC-3).
**Estado:** Phase C.2 (Hierarchy migrado, dual-path orchestrator inalterado) **fechada e commitada** — `40a1091 refactor(panels): ADR-0029 Phase C.2 — Hierarchy typed Panel migration`. Smoke do Enio para C.2 pendente (cadência v1.2 permite agrupar com C.3/C.4). Phase C.3 (Widget Gallery) é a próxima migração.
**Lê isto se você é uma LLM nova chegando para retomar.**

## 0. Verificação rápida

```bash
git log --oneline -1
# 40a1091 refactor(panels): ADR-0029 Phase C.2 — Hierarchy typed Panel migration

git status -sb
# ## main...origin/main [ahead 14]  (clean — ou apenas docs/HANDOFF*.md pendente)

cargo check --workspace 2>&1 | tail -3
# Finished `dev` profile (verde, ~5s warm)

cargo test --workspace --exclude ph2d-asset 2>&1 | grep "^test result" | tail -3
# 1299 passed, 0 failed, 7 ignored
```

Se diverge, **pare e pergunte ao Enio** antes de mudar nada.

---

## 1. Leia primeiro (ordem)

1. [`CLAUDE.md`](../CLAUDE.md) — workflow operacional (auto-load).
2. [`docs/DIRETRIZ_CODIFICACAO_RAPIDA.md`](DIRETRIZ_CODIFICACAO_RAPIDA.md) v1.2.
3. [`docs/architecture/decisions/0029-trait-driven-panel-host.md`](architecture/decisions/0029-trait-driven-panel-host.md) §5 Phase C — plano operacional.
4. [`docs/HANDOFF_WAVE_8_PHASE_C2.md`](HANDOFF_WAVE_8_PHASE_C2.md) — antecessor; padrão C.1 + workarounds (#[cfg(any())] gating; integration tests em panel crate; canonical_panel_id; store_and_hit_index_mut).
5. `git log --oneline -10`.
6. Memória persistente: `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`.

---

## 2. Estado atual (verificar)

```bash
git log --oneline -1
# 40a1091 refactor(panels): ADR-0029 Phase C.2 — Hierarchy typed Panel migration

git status -sb
# ## main...origin/main [ahead 14] (clean)

cargo check --workspace 2>&1 | tail -3
# Finished `dev` profile (verde)

cargo test --workspace --exclude ph2d-asset 2>&1 | grep "^test result" | tail -3
# 1299 passed, 0 failed, 7 ignored
```

Se algo diverge, **pare e pergunte ao Enio** antes de mudar nada.

---

## 3. O que C.2 entregou (não toque)

**Decisão estrutural:** C.2 manteve a divisão C.1..C.4 (1 painel por commit, smoke agrupado no fim).

**C.2 entregou (commit `40a1091`):**
- `ph2d-panel-hierarchy` reescrito de alias → typed `Panel` crate completo:
  - `src/state.rs` — `HierarchyState` (apenas `rename_target_row`) + thread-locals (`CURRENT_LIVE_ENTRIES`, `CURRENT_COMPONENT_COUNT`, `LAST_HIER_CONTENT_H`) + setters `pub` (`set_live_entries`, `set_live_component_count`, `set_last_hierarchy_content_h`) + getters internos.
  - `src/populate.rs` — `populate` boot-time (`HIERARCHY_ADD` + `HIER_SEARCH` + `HIER_PLAYER` placeholder + `init_hierarchy_order`) e `repopulate` (per-frame via `sync_from_hierarchy`, usa `register_if_absent` + `set_hierarchy_order`).
  - `src/search.rs` — `compute_match_filter` + 5 unit tests (porte verbatim).
  - `src/row.rs` — `paint_hierarchy_row` (porte verbatim).
  - `src/paint.rs` — `paint()` thunk típado: visibility gate via `host.panel_visible("hierarchy")`; pinta chrome+rows; retorna `row_set: BTreeSet<NodeId>` para o thunk publicar via `host.store_mut().set_hierarchy_row_ids(row_set)` (separação por restrição do trait `store_and_hit_index_mut` que entrega `&WidgetStore` imutável).
  - `src/event.rs` — `apply_event_impl` refatorado pra `(state: &mut HierarchyState, host: &mut dyn PanelHostInternal, ev: WidgetEvent)`. `host.bus_mut()`, `host.store_mut()`, `host.selection_mut()`. Inline-rename mode mora em `state.rename_target_row` direto.
  - `src/lib.rs` — `pub struct HierarchyPanel; impl Panel for HierarchyPanel` + `pub fn sync_from_hierarchy`, `pub fn clear_live_hierarchy`.
  - `Cargo.toml` — dropou `ph2d-editor`; pegou `ph2d-editor-core` + `ph2d-a11y` + `ph2d-tokens` + `ph2d-text` + `ph2d-vector`.
- `ph2d-editor-core`:
  - `screens/hero.rs` — `HeroScreen` removeu `pub hierarchy: HierarchyState`. `RAIL_SHOW_HIERARCHY` usa `panel_visibility.insert("hierarchy", ...)`. Orchestrator (`paint_hero_screen`) deixou de publicar `hierarchy::set_selection_label/set_live_entries/set_rename_target` (panel lê via `host.selection()` + thread-local próprio + `state.rename_target_row`). `if hero.hierarchy.visible` → `if hero.is_panel_visible("hierarchy")`. `sync_from_hierarchy`/`clear_live_hierarchy` removidos (shell chama o panel crate direto).
  - `state.rs` — `HierarchyState` deletado (não há mais `pub use HierarchyState`).
  - `screens/mod.rs` + `lib.rs` — `set_live_component_count` removido do reexport.
  - `panel_registry::default_panel_registry()` removeu Hierarchy — só `widget_gallery` + `grid_snap` continuam fn-pointer.
  - Diretório `screens/hero/hierarchy/` (1290 LOC) **DELETADO**.
  - `tests/hero_sync_round_trip.rs` **DELETADO** (migrou pro panel crate).
- `ph2d-panel-registry-init`:
  - `build_legacy_registry()` perdeu Hierarchy; `build_typed_registry()` ganhou `ErasedPanel::new::<HierarchyPanel>()` sob `panel-hierarchy` feature.
- `shells/desktop`:
  - `Cargo.toml`: dep opcional `ph2d-panel-hierarchy = { ..., optional = true }`, feature `panel-hierarchy = [..., "dep:ph2d-panel-hierarchy"]`.
  - `render_loop/snapshots.rs`: `hero.sync_from_hierarchy(...)` → `ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, ...)`; `ph2d_editor::set_live_component_count` → `ph2d_panel_hierarchy::set_live_component_count`. Ambos cfg-gated por `panel-hierarchy`.
  - `input_dispatch.rs:428`: `hero.hierarchy.live_entries.as_ref()` → `resolve_live_entry(gfx.hero_live.as_ref(), picked)` helper. Helper extraído pra `forwarding.rs` (input_dispatch.rs estava em 606 LOC, ultrapassando HR-18 600-LOC; ficou em 592 LOC depois).

**Testes:** 1299 passed / 0 failed / 7 ignored (`cargo test --workspace --exclude ph2d-asset`). Pre-commit T2 full nextest: 1342 passed (78 slow), 4 skipped. Pré-baseline em C.1 era 1300 passed → -1 net porque `paint_hierarchy_smoke` (chrome paint) ficou `#[cfg(any())]` em editor-core e não foi recriado no panel crate (porte requer mock de `HeroLayout` + viewport).

**Tests gated `#[cfg(any())]` em `screens/hero/tests.rs`** (8 + helper):
- `hero_apply_event_hierarchy_click_changes_selection`
- `hier_menu_duplicate_sets_pending_duplicate`
- `hier_menu_add_child_sets_pending_add_child`
- `hier_menu_reset_transform_sets_pending`
- `hier_menu_delete_sets_pending_delete`
- `hier_menu_click_without_snapshot_consumes_but_no_pending`
- `hier_menu_one_action_per_drain`
- `paint_hierarchy_smoke`
- `hierarchy_row_click_raises_pending_for_live_entries` (já estava gated em C.1 cleanup)
- helper `stage_hierarchy_row_snapshot` (gated junto)

**Tests recriados no panel crate** (`crates/ph2d-panel-hierarchy/tests/`):
- `hierarchy_apply_event.rs` (1)
- `hierarchy_context_menu.rs` (6)
- `hierarchy_sync_round_trip.rs` (8 — porte de `hero_sync_round_trip.rs` + `hierarchy_row_click_raises_pending_for_live_entries`)

---

## 4. Tensão técnica resolvida (cuidado em C.3-C.4)

### 4.1 Dev-dep cycle bloqueia tests de panel inline em editor-core

Mesmo problema de C.1: tests de panel-crate que usam `HeroScreen::apply_event` precisam do typed registry instalado. `ph2d_editor_core::test_support::ensure_panel_registry()` só instala a registry legacy. Solução em C.2 (replicar em C.3): cada panel-crate test file define um helper local `ensure_typed_registry()` com `static INIT: Once` que instala `PanelRegistry::new_empty().push(ErasedPanel::new::<HierarchyPanel>())`. Pode haver overlap se múltiplos test files numa mesma binária instalarem registries diferentes — `OnceLock::set` é idempotente, então o primeiro vence; aceitável porque cada test file roda numa binária separada.

### 4.2 `store_and_hit_index_mut` retorna `(&WidgetStore, &mut HitIndex)`

O joint accessor do `PanelHostInternal` entrega o `WidgetStore` IMUTÁVEL. Painter que mutam o store (e.g. `set_hierarchy_row_ids`, `set_panel_scroll`) precisam de uma das duas opções:
- **(A)** Computar o set localmente, retornar do helper, mutar via `host.store_mut()` no thunk (foi o que C.2 fez).
- **(B)** Adicionar um novo joint accessor que devolve `(&mut WidgetStore, &mut HitIndex)`. Provavelmente inviável (split-borrow em trait dyn não funciona); manter padrão A.

### 4.3 HR-18 600-LOC cap em `shells/desktop/src/input_dispatch.rs`

O arquivo está em 592 LOC após C.2 — qualquer adição que ultrapasse 600 quebra o `file_loc_caps` test. Soluções em ordem de preferência:
- (A) extrair helper pra `forwarding.rs` (foi o que C.2 fez).
- (B) declarar exception comment `// ph2d-loc-cap: <razão>` nas primeiras 20 linhas — aceitável mid-refactor.
- (C) decompor `input_dispatch.rs` em submódulos (custoso; só se a pressão crescer).

### 4.4 Dual HeroLayout (legacy + canonical) e legacy widget_gallery shadow

`crate::screens::hero::HeroLayout` ≠ `crate::screens::layout::HeroLayout`. Mesma shape; orchestrator faz copy-by-field antes de passar pro typed `PaintCtx`. Phase D colapsa os dois. **Cuidado em C.3:** o legacy `screens/hero/widget_gallery.rs` ainda EXISTE em editor-core (re-exportado pra que o legacy registry tinha sua entrada). Após C.3, esse módulo pode ser deletado (similar ao deletado em C.2 com `screens/hero/hierarchy/`).

---

## 5. O que C.3 precisa fazer

### 5.1 Migrar Widget Gallery panel

A boa notícia: o crate `ph2d-panel-widget-gallery` já existe com a implementação real (não é mais alias). O escopo de C.3 é **converter o manifest legacy fn-pointer em typed `Panel<State>`** + absorver state.

**State cross-panel em HeroScreen (`view: ViewState`):**
- `view.widget_gallery_visible: bool` — flag de visibilidade do panel. Atualmente toggled em `apply_event_thunk` quando `Click(TOPBAR_WIDGET_GALLERY)` ou `Click(GAL_CLOSE)`.
- `view.widget_gallery_rect: Option<Rect>` — rect persistido (drag/resize). Lazy-init na primeira pintura.

**Decisão recomendada (replicar Hierarchy):**
- `widget_gallery_visible` move para `panel_visibility: BTreeMap<&'static str, bool>` map (já tem `"widget_gallery": false` em `default_panel_visibility`). Toggle em `apply_event` usa `self.panel_visibility.insert("widget_gallery", ...)`.
- `widget_gallery_rect` → `WidgetGalleryState { rect: Option<Rect> }`. Acesso via `state` típado no `paint`/`apply_event`.

**Trabalho concreto:**
1. `ph2d-panel-widget-gallery/src/lib.rs` — adicionar `pub struct WidgetGalleryPanel; impl Panel for WidgetGalleryPanel { type State = WidgetGalleryState; const ID = "widget_gallery"; const NODE_ID = ids::GAL_PANEL; ... }`. `Cargo.toml` flipa `ph2d-editor` → `ph2d-editor-core`.
2. Refatorar `paint_thunk` → `paint(state: &mut WidgetGalleryState, ctx: &mut PaintCtx)` usando trait acessors. Idem `apply_event_thunk` → `apply_event(state, host, ev)`.
3. `ph2d-editor-core`:
   - `screens/hero.rs` — remover `pub mod widget_gallery;` E `widget_gallery::populate(store)` em `pre_populate_store`. Remover quaisquer reads de `view.widget_gallery_visible` / `view.widget_gallery_rect`. Remover esses 2 campos de `ViewState` em `state.rs`.
   - `default_panel_registry()` removerá Widget Gallery — só sobra `grid_snap`.
   - Em `apply_event`, NÃO há código gallery-coupled fora do panel (verificar via grep) — o panel crate já cuida do `TOPBAR_WIDGET_GALLERY`/`GAL_CLOSE`. Mas o legacy `screens/hero/widget_gallery.rs` é só sombra (foi extraído via reexport); pode deletar.
4. `ph2d-panel-registry-init`:
   - `build_legacy_registry()` perde `ph2d_panel_widget_gallery::PANEL_MANIFEST` (já dropado de editor-core).
   - `build_typed_registry()` ganha `ErasedPanel::new::<WidgetGalleryPanel>()` sob `panel-widget-gallery` feature.
   - Atualizar `EXPECTED_LEGACY` / `EXPECTED_TYPED` em test_support.
5. `shells/desktop` — checar se algum site lê `hero.view.widget_gallery_visible` ou `widget_gallery_rect`. Mover para `host.panel_visible("widget_gallery")` ou via WidgetGalleryPanel state se necessário.

### 5.2 Tests

Procurar em `screens/hero/tests.rs` por testes que tocam `view.widget_gallery_visible` (achei 3: linhas 252, 537, 591). Avaliar: se passam pelo `hero.apply_event` precisarão de typed registry instalado, então gate com `#[cfg(any())]` + recriar em `crates/ph2d-panel-widget-gallery/tests/`.

### 5.3 Cadência v1.2

- 1 commit ao fim de C.3 (Widget Gallery).
- Sem smoke entre painéis a menos que o Enio peça.
- C.4 (Grid Snap) segue mesmo padrão.

---

## 6. Pontos de atenção para o smoke (no fim de Phase C, não C.3 isolado)

**O que validar manualmente no smoke (C.2 + C.3 agrupados):**

1. Editor abre (`./play.command`).
2. **Hierarchy** renderiza com Scene Root + entity rows live.
3. **Search filter** funciona; clique em rows seleciona.
4. **Drag-reparent** + **eye-toggle** + **chevron-expand**.
5. **Context menu** (right-click row) → Duplicate / Add child / Reset / Delete / Rename.
6. **Inline-rename** (long-press / "Rename") → Enter commit, Esc cancel, Blur implicit commit.
7. **Widget Gallery** abre via TOPBAR pill, X fecha, drag/resize handles funcionam.
8. **Toggle visibility left rail** (RAIL_SHOW_HIERARCHY) → Hierarchy aparece/some.

**Sinais de regressão:**
- Hierarchy não atualiza (live entries thread-local não publicado pelo shell).
- Toggle rail não some o painel (panel_visibility map fora de sync).
- Click em row não consome (registry typed não instalado).
- Widget Gallery não abre (visible flag migration quebrou; verificar typed state default).

---

## 7. Quando este doc é obsoleto

Quando Phase C.3 (Widget Gallery) fecha e Enio aprova smoke. Aí escrever `HANDOFF_WAVE_8_PHASE_C4.md` ou consolidar em `docs/Migracao/2026-05-wave-8-phase-c-completed.md`.
