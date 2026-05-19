# HANDOFF — Wave 8 Phase C.2 (retomada após Phase C.1)

**Data:** 2026-05-18 noite.
**Estado:** Phase C.1 (Inspector migrado, dual-path orchestrator) **fechada e commitada** — `05dd935 refactor(panels): ADR-0029 Phase C.1 — Inspector typed Panel migration`. Smoke do Enio PASSOU. Phase C.2 (Hierarchy) é a próxima migração.
**Lê isto se você é uma LLM nova chegando para retomar.**

## 0. Verificação rápida

```bash
git log --oneline -1
# 05dd935 refactor(panels): ADR-0029 Phase C.1 — Inspector typed Panel migration

git status -sb
# ## main...origin/main [ahead 12]  (clean — ou apenas docs/HANDOFF*.md pendente)

cargo check --workspace 2>&1 | tail -3
# Finished `dev` profile (verde, ~5s warm)
```

Se diverge, **pare e pergunte ao Enio** antes de mudar nada.

---

## 1. Leia primeiro (ordem)

1. [`CLAUDE.md`](../CLAUDE.md) — workflow operacional (auto-load).
2. [`docs/DIRETRIZ_CODIFICACAO_RAPIDA.md`](DIRETRIZ_CODIFICACAO_RAPIDA.md) v1.2.
3. [`docs/architecture/decisions/0029-trait-driven-panel-host.md`](architecture/decisions/0029-trait-driven-panel-host.md) §5 Phase C — plano operacional.
4. [`docs/HANDOFF_WAVE_8_PHASE_C.md`](HANDOFF_WAVE_8_PHASE_C.md) — antecessor; contexto do Phase B.
5. `git log --oneline -10`.
6. Memória persistente: `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`.

---

## 2. Estado atual (verificar)

```bash
git log --oneline -1
# <hash> refactor(editor-core): ADR-0029 Phase C.1 — Inspector typed Panel migration

git status -sb
# ## main...origin/main [ahead 12+] (clean)

cargo check --workspace 2>&1 | tail -3
# Finished `dev` profile (verde, ~5s warm)

cargo test --workspace --exclude ph2d-asset 2>&1 | grep "^test result" | tail -3
# 1300 passed, 0 failed, 7 ignored
```

Se algo diverge, **pare e pergunte ao Enio** antes de mudar nada.

---

## 3. O que C.1 entregou (não toque)

**Decisão estrutural:** Phase C dividiu-se em C.1..C.4 (1 painel por commit) em vez de "1 commit para os 4". O handoff original (C) subestimou o blast radius: cross-panel coordination, populate.rs misturada, hero/tests.rs acoplada a `hero.inspector.*` fields, dev-dep cycle em testes.

**C.1 entregou:**
- `ph2d-panel-inspector` reescrito de alias → typed `Panel` crate completo:
  - `src/state.rs` — `InspectorState` + thread-locals (snapshots host-published) + setters/getters `pub`.
  - `src/sections.rs` — `paint_{entity_name,visibility,transform,render_source}` (portado verbatim do legacy, imports ajustados).
  - `src/populate.rs` — Inspector-only widget registrations (Name, Visibility, Transform, Render Strategy).
  - `src/paint.rs` — `paint()` + `paint_inspector()` refatorados pra `(state: &mut InspectorState, ctx: &mut PaintCtx)` com `host: &mut dyn PanelHostInternal`.
  - `src/event.rs` — `apply_event_impl` refatorado pra usar `host.bus_mut()`, `host.project()`, `host.store()` etc. (sem `&mut HeroScreen`).
  - `src/sync.rs` — `sync_inspector_from_snapshots` refatorado.
  - `src/lib.rs` — `pub struct InspectorPanel; impl Panel for InspectorPanel`.
  - `Cargo.toml` — dropou dep em `ph2d-editor`; pegou direto `ph2d-editor-core` + `ph2d-a11y` + `ph2d-tokens` + `ph2d-text` + `ph2d-vector`.
- `ph2d-editor-core`:
  - `panel/host.rs` — `PanelHostInternal` cresceu de 3 métodos pra 9: `store/_mut`, `hit_index_mut`, `store_and_hit_index_mut` (split-borrow joint accessor), `bus/_mut`, `selection/_mut`, `panel_visible/set_panel_visible`.
  - `panel/registry.rs` — adicionou `with_registry_opt` (lenient variant retornando `Option<R>`). Orchestrator usa esse pra tolerar typed registry ausente (editor-core tests).
  - `screens/hero.rs` — `HeroScreen` removeu `pub inspector: InspectorState`; adicionou `pub panel_visibility: BTreeMap<&'static str, bool>` (HR-5 — não HashMap). `is_panel_visible(id)` helper. Orchestrator `paint_hero_screen` agora dual-path: itera legacy registry + typed registry por z_order, escolhendo manifest baseado em `find_panel_by_node_id`. `apply_event` walks both registries + chama `widget::showcase::apply_showcase_event` como fallback host-level.
  - `screens/hero/pre_populate.rs` (NOVO) — registrações compartilhadas (showcase samples, BlenderColorPicker, ctx menus globais, scrollbars, Hierarchy drag/resize handles). Extraído da legacy `inspector/populate.rs`.
  - `widget/showcase/mod.rs` — `apply_showcase_event` (NOVA) — handler host-level dos cliques showcase-compartilhados (`CTX_MENU_OUTLINE_*`, `CTX_MENU_CREATE_NOTE`, `SECTION_IDS`, radio/tab/tree pin). Antes vivia em `inspector::apply_event` mas atendia BOTH Inspector + Gallery — agora roda no host.
  - Legacy `screens/hero/inspector/` (dir inteiro) **DELETADO**.
  - Legacy `screens/hero/inspector_sync.rs` **DELETADO**.
  - `panel_registry::default_panel_registry()` removeu entrada Inspector — só Hierarchy + Gallery + GridSnap continuam fn-pointer manifests.
- `ph2d-panel-registry-init`:
  - `register_all_panels()` agora instala AMBAS as registries (legacy + typed).
  - `build_legacy_registry()` + `build_typed_registry()` separadas.
  - `Cargo.toml` flipou de `ph2d-editor` → `ph2d-editor-core`.
- `shells/desktop/Cargo.toml` — adicionou `ph2d-panel-inspector` como dep opcional (gated por `panel-inspector` feature).
- `shells/desktop/src/render_loop/snapshots.rs` — atualizado: shell publica snapshots via thread-local setters `ph2d_panel_inspector::set_current_inspector_*` em vez de `hero.inspector.<field> = ...` (campo deletado).

**Testes:** 1300 passed, 0 failed, 7 ignored. Editor-core lib: 717 passed.

---

## 4. Tensão técnica resolvida (cuidado em C.2-C.4)

### 4.1 Dev-dep cycle bloqueia testes inline no editor-core

`crates/ph2d-editor-core/src/screens/hero/tests.rs` historicamente acessava `hero.inspector.<field> = Some(...)` para semear fixtures. Após C.1 esse campo não existe — substituir-iria por `ph2d_panel_inspector::set_current_inspector_<field>(Some(...))`.

**Problema:** Para usar `ph2d_panel_inspector::*` em testes do editor-core, precisa dev-dep. Mas isso cria ciclo (panel-inspector → editor-core → panel-inspector via dev-dep). Cargo trata o ciclo como **duas instâncias separadas** do `ph2d-editor-core`, quebrando type identity:

```
error[E0308]: expected `InspectorSpriteInfo`, found `hero::InspectorSpriteInfo`
note: there are multiple different versions of crate `ph2d_editor_core` in the dependency graph
```

**Workaround em C.1:** 15 testes Inspector-coupled marcados com `#[cfg(any())]` (não compilam) em `crates/ph2d-editor-core/src/screens/hero/tests.rs`. Plano: migrar pra `crates/ph2d-panel-inspector/tests/inspector_regression.rs` (integration tests não têm o ciclo).

**Testes desabilitados (15):**
- `inspector_position_value_displayed_in_pixels_round_trips_to_meters`
- `inspector_position_meters_mode_displays_raw_meters`
- `transform_field_commit_raises_pending_with_selection`
- `transform_reset_button_publishes_identity`
- `visibility_toggle_publishes_pending_with_selection`
- `visibility_toggle_no_pending_without_selection`
- `strategy_click_raises_pending_when_kind_differs`
- `strategy_click_no_pending_without_sprite_selection`
- `strategy_click_resets_button_state_to_normal`
- `entity_name_text_changed_raises_pending_with_selection`
- `name_text_changed_no_pending_without_selection`
- `name_text_changed_publishes_pending_with_current_text`
- `selection_switch_resets_entity_name_input_state_to_normal`
- `paint_inspector_smoke_with_selection`
- `paint_inspector_smoke_no_selection`

**Ação para C.2 (ou Phase D):** Criar `crates/ph2d-panel-inspector/tests/inspector_regression.rs`, mover esses 15 testes pra lá. Adicionar dev-deps em `ph2d-panel-inspector/Cargo.toml`: `bumpalo`, `ph2d-host`, `ph2d-vector`, `ph2d-text`, `ph2d-tokens`, `ph2d-panel-registry-init`. Tests usam `HeroScreen::new()` + `ph2d_panel_registry_init::register_all_panels()` + `paint_hero_screen()` para o setup.

### 4.2 Dual HeroLayout (legacy + canonical)

`crate::screens::hero::HeroLayout` (legacy) ≠ `crate::screens::layout::HeroLayout` (canonical, usada pela new PaintCtx). Mesma shape. Em `paint_hero_screen`, antes de chamar typed panel, fazemos copy-by-field. Phase D deve colapsar os dois (deletar o legacy).

### 4.3 `panel_visibility: BTreeMap<&'static str, bool>`

HR-5 proíbe `HashMap` (hashing não-determinístico). Usei `BTreeMap`. Keys são strings `&'static str` literal: `"inspector"`, `"hierarchy"`, `"widget_gallery"`, `"grid_snap"`. `PanelHostInternal::set_panel_visible(id: &str, ...)` tem `canonical_panel_id(id)` que mapeia strings conhecidas pra literais estáticas; ids desconhecidos vão por `Box::leak` (raro). Cuidado em C.2-C.4 ao adicionar novos panels: atualizar `canonical_panel_id` E `default_panel_visibility()` (ambos em `hero.rs`).

### 4.4 `store_and_hit_index_mut` joint accessor

Trait dyn-dispatch não consegue field-split como struct concreto. Painters precisam `&WidgetStore + &mut HitIndex` simultâneos. Solução: método único retornando o par. Padrão em uso só pelo Inspector (paint.rs); replicar para Hierarchy/Gallery se necessário.

---

## 5. O que C.2 precisa fazer

### 5.1 Migrar Hierarchy panel

`crates/ph2d-editor-core/src/screens/hero/hierarchy/` (~1290 LOC) → `crates/ph2d-panel-hierarchy/src/`.

**Cross-panel state em HeroScreen:** `hero.hierarchy.live_entries`, `hero.hierarchy.rename_target_row`. Decidir:
- (A) Mover pra panel state (HierarchyState moves to panel crate).
- (B) Manter em HeroScreen via `PanelHostInternal::live_hierarchy_entries()` + `rename_target_row()`.

Recomendo (A) — mantém o pattern do Inspector. Mas tem complicação: `hero.sync_from_hierarchy(...)` é API pública do shell que escreve em `hero.hierarchy.live_entries`. Após C.2 vira `ph2d_panel_hierarchy::set_live_entries(...)` (thread-local) ou um método em HeroScreen que reachs typed state via registry.

**Apply event:** apply_event.rs tem 192 LOC com muitos `hero.<field>` accesses (bus, selection, store, hierarchy.rename_target_row, hierarchy.live_entries). Padrão idêntico ao Inspector — refatorar usando PanelHostInternal trait.

**Orchestrator coordination:** Linhas 1239-1242 em hero.rs publicam HIER_PANEL rect baseado em `hero.is_panel_visible("hierarchy")`. Mantém após migração. Linhas 1261-1262 publicam `hero.hierarchy.live_entries` / `rename_target_row` via thread-local — esses thread-locals ou se movem pro panel crate, ou pode ser orquestrado dentro do paint do próprio Hierarchy.

**Testes:** mover os hierarchy-coupled tests pra `crates/ph2d-panel-hierarchy/tests/hierarchy_regression.rs`. Algumas linhas atingem `hero.hierarchy.*` — mesma cirurgia do Inspector.

### 5.2 Outra dependência conhecida

`screens/hero.rs` linha ~1257 chama `hierarchy::set_live_entries(hero.hierarchy.live_entries.clone())`. Esses thread-local setters vivem em `screens/hero/hierarchy/` que **migra inteiro pro panel crate**. Após C.2 essa linha vira `ph2d_panel_hierarchy::set_live_entries(...)` (gated por `feature = "panel-hierarchy"` em shells/desktop ou similar).

### 5.3 Cadência v1.2

- 1 commit ao fim de C.2 (Hierarchy).
- Sem smoke entre painéis a menos que o Enio pedir.
- C.3 (Widget Gallery), C.4 (Grid Snap) seguem mesmo padrão.

---

## 6. Pontos de atenção para o smoke do Enio

**O que validar manualmente no smoke C.1:**

1. Editor abre (`./play.command`).
2. **Inspector painel renderiza** — 4 zonas dock layout intacto.
3. Selecionar uma entidade — Inspector mostra Name + Visibility + Transform + Render Source.
4. **Toggle visibility** via left rail `RAIL_SHOW_INSPECTOR` — Inspector aparece/some.
5. **Editar Transform NumberInput** — commit via Enter publica `EditorAction::InspectorTransformEdit` no bus.
6. **Toggle visibility checkbox** — publica `InspectorVisibilityEdit`.
7. **Trocar Strategy buttons** — publica `InspectorSpriteSourceChange`.
8. **Editar Entity Name** — publica `InspectorNameEdit`.
9. **Context menu na seção** → outline color picker funciona (host-level `apply_showcase_event` herdou esse path).
10. **"Create Note"** → cria nota anexada à seção apropriada.
11. **Color circle clique** → abre BlenderColorPicker (`set_picker_target` via showcase event).
12. **Reimport button** → publica `Reimport` action.
13. **Hierarchy panel + Widget Gallery + Grid Snap** continuam funcionando (legacy registry; sem mudanças).

**Sinais de regressão a observar:**
- Inspector não atualiza com seleção (sync.rs quebrou).
- Transform commit converte unidade errada (display_unit propagation).
- Section outline color não persiste (apply_showcase_event não está rodando).

---

## 7. Quando este doc é obsoleto

Quando Phase C.2 (Hierarchy) fecha e Enio aprova smoke. Aí escrever `HANDOFF_WAVE_8_PHASE_C3.md` ou consolidar em `docs/Migracao/2026-05-wave-8-phase-c-completed.md`.
