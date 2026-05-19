# HANDOFF — Wave 8 Phase C.4 (final panel migration; closes Phase C)

**Data:** 2026-05-19.
**Estado:** Phase C.3 fechada — Widget Gallery migrado a typed Panel. Commit `4a8e361 refactor(panels): ADR-0029 Phase C.3 — Widget Gallery typed Panel migration`. Phase C.4 é a ÚLTIMA fase de Phase C — migra Grid Snap panel chrome para typed `Panel<State>` e fecha a wave inteira. Após C.4: PR + CI (autorizado pelo Enio em mensagem de 2026-05-19, padrão `feedback_phase_cascade_2026_05_19`).
**Lê isto se você é uma LLM nova chegando para retomar.**

## 0. Verificação rápida

```bash
git log --oneline -1
# 4a8e361 refactor(panels): ADR-0029 Phase C.3 — Widget Gallery typed Panel migration

git status -sb
# ## main...origin/main [ahead 16]  (clean — ou apenas docs/HANDOFF*.md pendente)

cargo check --workspace 2>&1 | tail -3
# Finished `dev` profile (verde, ~5s warm)

cargo test --workspace --exclude ph2d-asset 2>&1 | grep "^test result" | awk -F'[, ]+' '{p+=$4; f+=$6; i+=$8} END {print "passed:" p " failed:" f " ignored:" i}'
# passed:1299 failed:0 ignored:7
```

Se diverge, **pare e pergunte ao Enio** antes de mudar nada.

---

## 1. Leia primeiro (ordem)

1. [`CLAUDE.md`](../CLAUDE.md) — workflow operacional (auto-load).
2. [`docs/DIRETRIZ_CODIFICACAO_RAPIDA.md`](DIRETRIZ_CODIFICACAO_RAPIDA.md) v1.2.
3. [`docs/architecture/decisions/0029-trait-driven-panel-host.md`](architecture/decisions/0029-trait-driven-panel-host.md) §5 Phase C — plano operacional.
4. [`docs/HANDOFF_WAVE_8_PHASE_C3.md`](HANDOFF_WAVE_8_PHASE_C3.md) — antecessor.
5. [`docs/HANDOFF_WAVE_8_PHASE_C2.md`](HANDOFF_WAVE_8_PHASE_C2.md) — referência mais detalhada do padrão.
6. `git log --oneline -10`.
7. Memória persistente: `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`. Em particular `feedback_phase_cascade_2026_05_19.md`.

---

## 2. O que C.3 entregou (não toque)

Commit `4a8e361`. Resumo:
- `ph2d-panel-widget-gallery`: agora é um typed `Panel<WidgetGalleryState>` crate completo (state/event/paint/populate split).
- `view.widget_gallery_visible` + `view.widget_gallery_rect` removidos de `ViewState`. Visibilidade no `panel_visibility` map; rect no `WidgetGalleryState`.
- `screens/hero/widget_gallery.rs` em editor-core **DELETADO**.
- `panel-registry-init`: Widget Gallery em `build_typed_registry` (Inspector + Hierarchy + Widget Gallery aqui); só `grid_snap` resta no `build_legacy_registry`.
- `default_panel_registry()` em editor-core só tem `grid_snap`.

Tests: 1299 passed / 0 failed / 7 ignored.

---

## 3. O que C.4 precisa fazer

### 3.1 Migrar Grid Snap panel chrome (NÃO o canvas renderer)

**Importante:** `grid_snap` em editor-core mistura dois subsistemas:
1. **Canvas renderer** (`grid_snap/render/`, `grid_snap/state.rs::GridSnapState`, `grid_snap/inspect.rs`) — pinta a grade no canvas, snap algorithm. **FICA em editor-core** porque o painter de canvas (`paint_hero_screen`) usa `crate::grid_snap::render::paint(scene, &view, &state_for_paint)` e o `hero.grid.snap_state` é lido pelos toggles do view.
2. **Panel chrome** (`grid_snap/mod.rs::PANEL_MANIFEST`, `grid_snap/panel/{events,orchestrator,paint_helpers,paint_kinds,paint_rows,populate,mod}.rs`) — a janela flutuante "Grid Settings" com knobs. **MIGRA pra `ph2d-panel-grid-snap`**.

O `panel/` sub-mod já está bem isolado (~1944 LOC) — porte verbatim, igual C.2 fez com Hierarchy.

### 3.2 State

`GridSnapState` tem dois campos panel-chrome-específicos no meio dos campos do snap subsystem:
- `panel_visible: bool`
- `panel_rect: Option<Rect>`

**Decisão recomendada:**
- `panel_visible` → mover para `panel_visibility: BTreeMap<&'static str, bool>` map (já tem entrada `"grid_snap": false`).
- `panel_rect` → manter em `GridSnapState` por enquanto (não compensa criar `GridSnapPanelState` apenas para o rect; o canvas renderer usa o resto da `GridSnapState`). OU criar `GridSnapPanelState { rect: Option<Rect> }` se preferir paridade total com Widget Gallery — fica a critério do agente. **Recomendo manter em GridSnapState** para reduzir mudanças, com `panel_visible` migrado para o map.

### 3.3 Trabalho concreto

1. **`ph2d-panel-grid-snap` crate:**
   - Cargo.toml: flipar `ph2d-editor` → `ph2d-editor-core` + adicionar `ph2d-a11y`, `ph2d-tokens`, `ph2d-text`, `ph2d-vector` se necessário.
   - Criar `src/{state.rs,paint.rs,event.rs,populate.rs,lib.rs}` mirroring C.2/C.3 layout. `GridSnapPanelState` mínimo (talvez vazio, com `Default` impl).
   - Port `grid_snap/panel/*.rs` para o panel crate. Imports do tipo `crate::*` viram `ph2d_editor_core::*`.
   - `lib.rs` define `pub struct GridSnapPanel; impl Panel for GridSnapPanel { type State = GridSnapPanelState; const ID = "grid_snap"; const NODE_ID = ids::GS_PANEL; const DEFAULT_VISIBLE = false; ... }`.
2. **`ph2d-editor-core`:**
   - Remover `grid_snap/panel/` (1944 LOC) e `grid_snap/mod.rs::PANEL_MANIFEST`. Re-checar `mod.rs` — só deve sobrar export do canvas renderer + state.
   - `GridSnapState::panel_visible` → remover; substituir leituras por `host.panel_visible("grid_snap")` (no panel crate) e `hero.is_panel_visible("grid_snap")` (no orchestrator).
   - `apply_event` em `hero.rs` — verificar se há references diretas a `grid.snap_state.panel_visible` (toggle via TOPBAR_GRID_SETTINGS ou similar) e mover para `panel_visibility.insert("grid_snap", ...)`.
   - `panel_registry::default_panel_registry()` retorna `PanelRegistry::new(vec![])` (ou um wrapper de fato vazio). Considerar deletar `default_panel_registry()` inteiramente e ter o test_support usar `PanelRegistry::new_empty()`.
3. **`ph2d-panel-registry-init`:**
   - `build_legacy_registry` fica vazio. Considerar deletar `LegacyRegistry` install do `register_all_panels` — só `build_typed_registry` precisa instalar de fato. Cuidado: o trait `panels()` em editor-core ainda existe para iteração em `apply_event`/orchestrator. Se ficar com 0 panels, o walker just retorna nada. Reduzir codepath se possível.
   - `build_typed_registry` ganha `ErasedPanel::new::<GridSnapPanel>()` sob `panel-grid-snap` feature.
   - `EXPECTED_LEGACY = 0`, `EXPECTED_TYPED = 4` (todos painéis).
4. **`shells/desktop`:**
   - `Cargo.toml`: feature `panel-grid-snap = ["...", "dep:ph2d-panel-grid-snap"]` se necessário (atualmente não é dep direta).
   - Procurar refs a `grid.snap_state.panel_visible` no shell — se houver, ajustar.
5. **Tests gated `#[cfg(any())]`:** Procurar em `screens/hero/tests.rs` e qualquer `grid_snap*_tests.rs` por testes que tocam `grid_snap::panel`. Gate + recriar em `crates/ph2d-panel-grid-snap/tests/` usando `ensure_typed_registry()` helper (padrão C.2/C.3).
6. **Limpeza pós-C.4:**
   - Considerar deletar `crate::panel_registry::*` (todo o registry legacy) já que ninguém usa após C.4. **Cuidado:** o orchestrator (`hero.rs paint_hero_screen` + `apply_event`) ainda walks `crate::panel_registry::panels()`. Substituir por iteração só do typed registry. Esse cleanup pode ficar fora de C.4 (Phase D), mas se ficar trivial, faz.

### 3.4 Cadência

- 1 commit ao fim de C.4: `refactor(panels): ADR-0029 Phase C.4 — Grid Snap typed Panel migration (Phase C closes)`.
- Pre-commit hook T2 valida tudo automaticamente (fmt + typos + clippy + nextest + doc-tests).

---

## 4. APÓS C.4 fechar: PR + CI

**Esse é o procedimento explicitamente autorizado pelo Enio em 2026-05-19** (vide `feedback_phase_cascade_2026_05_19`):

1. **Verificar estado:** `git log --oneline -10`, commits C.1..C.4 + handoffs (~8 commits).
2. **Push:** `git push -u origin main`.
3. **Fornecer link da CI run:** `gh run list --workflow=spike.yml --limit=1` → fornecer link `https://github.com/dibrioli/PH2D/actions/runs/<run-id>` ao Enio.
4. **PR não é estritamente necessário** se o trabalho foi direto em `main` — verificar a política. Se a regra é PR-then-merge, abrir PR via `gh pr create` (a maioria do workflow PH2D commita direto em main per `feedback_commit_cadence`).

**Importante:** NÃO monitore CI em loop. Forneça o link e prossiga (per CLAUDE.md §CI). Se for fim de jornada/dia, o papel PRCI cuida do polling (vide `docs/IntegracaoMultiAgente/04-Agente-PRCI.md`).

**Memória pós-C.4:**
- Criar `project_phase_c_complete_2026_05_19.md` celebrando o fechamento de Phase C inteira (C.1-C.4 + PR).
- Substituir a entrada de `project_phase_c3_2026_05_19.md` no MEMORY.md por uma de Phase C complete.

---

## 5. Quando este doc é obsoleto

Quando Phase C.4 (Grid Snap) fecha + Phase C inteira é mergeada + CI verde. Aí escrever `docs/Migracao/2026-05-wave-8-phase-c-completed.md` (ou similar) com inventário do que mudou em C inteiro, e arquivar todos os `HANDOFF_WAVE_8_PHASE_C*.md`.

---

## 6. Resumo do estado de `ph2d-panel-registry-init` (referência cross-fase)

| Fase   | Legacy (fn-pointer)                        | Typed (Panel<State>)                  |
|--------|--------------------------------------------|---------------------------------------|
| pre-C  | inspector, hierarchy, gallery, grid_snap   | (vazio)                               |
| C.1    | hierarchy, gallery, grid_snap              | inspector                             |
| C.2    | gallery, grid_snap                         | inspector, hierarchy                  |
| C.3    | grid_snap                                  | inspector, hierarchy, gallery         |
| **C.4**| **(vazio — deletar?)**                     | **inspector, hierarchy, gallery, grid_snap** |

Após C.4, o legacy registry pode ser deletado inteiro (Phase D cleanup), mas mínimo viável de C.4 é apenas vaziar a lista; o tipo `LegacyRegistry` pode ficar.
