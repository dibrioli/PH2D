# HANDOFF — Wave 8 Phase C (retomada após context refresh)

**Data:** 2026-05-18 (smoke B.6 OK confirmado pelo Enio).
**Estado:** Phase B.1–B.6 fechadas. Phase C–D pendentes.
**Lê isto se você é uma LLM nova chegando para retomar.**

---

## 1. Leia primeiro (ordem)

1. [`CLAUDE.md`](../CLAUDE.md) — workflow operacional (auto-load).
2. [`docs/DIRETRIZ_CODIFICACAO_RAPIDA.md`](DIRETRIZ_CODIFICACAO_RAPIDA.md)
   v1.2 — cadência: 1200 LOC threshold para `cargo check`, 1 commit por
   Phase, não duplicar pre-commit hook.
3. [`docs/architecture/decisions/0029-trait-driven-panel-host.md`](architecture/decisions/0029-trait-driven-panel-host.md)
   §5 Phase C — **o plano operacional do que fazer agora**. §4.3/4.4
   detalha a forma do `Panel` trait + `ErasedPanel`. §4.2 lista os ~25-30
   acessores que `PanelHostInternal` precisará crescer.
4. `git log --oneline -10` — vê os 10 commits empilhados em main.
5. Memória persistente: `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`
   — especialmente `project_post_phase_b_2026_05_18.md` (closeout) +
   `feedback_codificacao_rapida.md` v1.2.
6. [`docs/Migracao/2026-05-wave-8-phase-b-completed.md`](Migracao/2026-05-wave-8-phase-b-completed.md)
   — guia antigo de B.2; arquivado, útil só pra contexto histórico do
   que B.1–B.5 entregou.

---

## 2. Estado atual (verificar)

```bash
git log --oneline -1
# ee4f37f refactor(editor-core): ADR-0029 Phase B.2–B.5 — absorb ph2d-editor; shim left behind

git status -sb
# ## main...origin/main [ahead 10+]  (clean)

cargo check --workspace 2>&1 | tail -3
# Finished `dev` profile (verde, ~5s warm)
```

Se algo diverge, **pare e pergunte ao Enio** antes de mudar nada.

---

## 3. O que Phase B entregou (não toque)

**B.1 (`f46a6c4`)** — `ph2d-editor-core::panel/` infrastructure:
- `host.rs` — `PanelHost` (público, 2 métodos: theme, project) +
  `PanelHostInternal: PanelHost` (`#[doc(hidden)]`, 3 métodos: store,
  store_mut, hit_index_mut).
- `panel_trait.rs` — `pub trait Panel { type State; const ID, NODE_ID,
  DEFAULT_VISIBLE; fn paint/apply_event/populate }`.
- `paint_ctx.rs` — `PaintCtx { host: &mut dyn PanelHostInternal,
  layout, scene, … }`.
- `erased.rs` — `ErasedPanel` wrapper com `Box<dyn Any + Send>` state.
- `manifest.rs` — `PanelManifest` + `for_panel::<P>()` ctor.
- `registry.rs` — `PANEL_REGISTRY: OnceLock<Mutex<PanelRegistry>>`
  com `Vec<ErasedPanel>`.
- `event_outcome.rs` — `EventOutcome::{Consumed, Ignored, Observed}`.

**Foundation pronta. Nenhum painel está wired ainda.**

**B.2–B.5 (`ee4f37f`)** — absorção de `ph2d-editor` em
`ph2d-editor-core`:
- `ph2d-editor` é shim de 17 linhas (`pub use ph2d_editor_core::*;`).
  Targeted para deleção em ADR-0030 (~6 meses).
- Todo conteúdo substantivo (HeroScreen, panel_registry legado,
  action_bus, grid_snap, image_edit, screens/hero/*, test_support,
  tool, tools/) vive em `ph2d-editor-core::*` top-level.
- 9 testes integrados migraram para `ph2d-editor-core/tests/`.
- `HeroScreen` implementa `PanelHost` + `PanelHostInternal`
  (superfície mínima: 5 métodos no total).
- Orchestrator `paint_hero_screen` **preserva** iteração antiga via
  `panel_registry::panels()` — `panel/` trait infra fica em paralelo
  aguardando Phase C.

**B.6 smoke do Enio: PASSOU** (2026-05-18 noite). Phase B fechada.

---

## 4. O que Phase C precisa fazer (ADR-0029 §5 Phase C)

Migrar os 4 painéis in-tree da forma **fn-pointer antiga**
(`panel_registry::PANEL_MANIFEST` declarado em
`screens/hero/{inspector,hierarchy,widget_gallery}/mod.rs` +
`grid_snap/mod.rs`) para a forma **typed `Panel<State>`** (impl
`Panel for InspectorPanel { type State = InspectorState; … }`).

### 4.1 Ordem (1 painel por vez, todos no mesmo commit no fim)

1. **Inspector** (~3500 LOC, test bed).
2. **Hierarchy** (~1500 LOC, shape similar ao Inspector).
3. **Widget Gallery** (mostly migrated, finaliza isolamento de state).
4. **Grid Snap** (já tem state isolado em `crate::grid_snap::state`;
   o mais simples).

### 4.2 Por painel — workflow

Para cada painel (ex: Inspector):

a) **Mover state**: `InspectorState` (em
   `editor-core/src/screens/hero/state.rs`) → `crates/ph2d-panel-inspector/src/state.rs`.
   Renomeia `pub struct InspectorPanel;` zero-size + state factory.

b) **Mover paint logic**: `editor-core/src/screens/hero/inspector/*`
   → `ph2d-panel-inspector/src/{paint.rs,event.rs,...}`. Refatora as
   funções para `fn(state: &mut InspectorState, ctx: &mut PaintCtx)`
   (typed) em vez de `fn(ctx: &mut PaintCtx { hero: &mut HeroScreen })`
   (legacy).

c) **Impl `Panel`**: em `ph2d-panel-inspector/src/lib.rs`:
   ```rust
   pub struct InspectorPanel;
   impl Panel for InspectorPanel {
       type State = InspectorState;
       const ID: &str = "inspector";
       const NODE_ID: NodeId = ids::INSP_PANEL;
       const DEFAULT_VISIBLE: bool = true;
       fn paint(state: &mut InspectorState, ctx: &mut PaintCtx) { … }
       fn apply_event(state: &mut InspectorState, host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> EventOutcome { … }
       fn populate(store: &mut WidgetStore) { … }
   }
   ```

d) **Drop dep `ph2d-editor`**: em
   `crates/ph2d-panel-inspector/Cargo.toml`, remove `ph2d-editor`
   (já estava lá indireto via alias model). Mantém `ph2d-editor-core`.

e) **Crescer `PanelHostInternal`**: cada vez que o painel chama um
   field/método de HeroScreen que ainda não está no trait, ADICIONE o
   método em
   `editor-core/src/panel/host.rs` + impl em
   `editor-core/src/screens/hero.rs`. Lista provável (de §4.2 do ADR):
   `selection`, `selection_mut`, `bus_mut`, `view`, `view_mut`,
   `gizmo`, `gizmo_mut`, `grid`, `grid_mut`, `image_edit`,
   `live_hierarchy_entries`, `dragging_files`, `display_unit`,
   `pixels_per_meter`, `tool_registry`, `stats`,
   `import_requested`/`request_import`, `camera_reset_pending`/
   `request_camera_reset`, `panel_state<P>`/`panel_state_mut<P>`.
   **Não declare 25 métodos de uma vez** — só adicione quando o
   painel atual REALMENTE chamar. Tests `architecture_panel_host_surface`
   ainda não existe (criar em Phase D).

f) **Update `ph2d-panel-registry-init`**: trocar
   `&legacy_inspector::PANEL_MANIFEST` por `ErasedPanel::new::<InspectorPanel>()`
   no `register_all_panels()`.

g) **Cargo check** após cada painel
   (`cargo check -p ph2d-panel-inspector` + `cargo check --workspace`).
   **Não commit per painel** — acumula até os 4.

### 4.3 Cleanup parcial dentro de Phase C

Após cada painel migrado, **delete** o código legado correspondente
em `editor-core/src/screens/hero/<painel>/` e em
`editor-core/src/panel_registry.rs::default_panel_registry`. O
`paint_hero_screen` orchestrator transiciona da iteração via
`panel_registry::panels()` (legacy fn-pointer) para
`PANEL_REGISTRY.with(|r| r.panels_mut().iter_mut().for_each(|p| p.paint(&mut ctx)))`
(typed) à medida que os painéis migram. Pode dual-path durante
transição (orchestrator chama AMBAS as iterações enquanto sobra
algum legacy) ou migrar todos os 4 antes de switchar (mais limpo).
**Sugestão:** dual-path; vai virando typed um por um.

### 4.4 Checkpoint Phase C (Enio smoke)

Após os 4 painéis migrados + workspace verde:
```
cargo test --workspace --exclude ph2d-asset → ~1358 verde
cargo clippy --workspace --all-targets -- -D warnings → clean
./play.command → editor abre, 4 painéis renderizam, input funciona
```

**Smoke do Enio é gate** antes de Phase D (cleanup + arch tests + push).

---

## 5. Tensões técnicas observadas em B (cuidado em C)

- **`PanelHostInternal` cresce.** Cada painel adiciona ~5-10 métodos.
  Total esperado ~25-30 no final. Architecture test em Phase D vai
  cravar threshold de 35; até lá, sem gate. Se passar de 30, pause e
  considere refatoração de field grouping em HeroScreen.
- **Inspector é heaviest.** ~3500 LOC + `inspector_sync.rs`
  (cross-frame state sync). Faça primeiro como test bed; aprendizados
  ressaltam padrão pros outros 3.
- **`InspectorState` move pode quebrar tests.** `hero_sync_round_trip.rs`
  (testa Inspector ↔ ECS sync) depende de pode-construir
  `HeroScreen::new()` com Inspector visível. Após Phase C, `InspectorState`
  vive no panel crate — tests acessam via `panel_state<InspectorPanel>()`.
- **`grid_snap` é trickeiro** apesar de mais simples: panel + state +
  render + ids estão sub-divididos. Boa última peça depois dos outros 3.

---

## 6. Cadência v1.2 (mantenha)

- Edit burst até ~1200 LOC sem `cargo check`.
- `cargo check -p <panel-crate>` entre transformações por painel.
- `cargo check --workspace` ao fim de cada painel.
- Pre-commit hook é a matriz oficial — não duplicar `cargo test --workspace`
  antes do commit.
- **1 commit por Phase C** (acumula 4 painéis + cleanup).
- Smoke do Enio ao FIM (depois dos 4 painéis migrados).

---

## 7. Após Phase C pass

Phase D (nova sessão):
- Delete `ph2d-editor` crate completamente.
- Architecture tests: `panel_crates_depend_only_on_editor_core`
  sai de `#[ignore]`. `panel_host_surface_count` (novo) gates ≤35.
  `public_api_surface` (novo) gates ≤80 items pub.
- Push pro GitHub → PRCI babysit do CI.

---

## 8. Quando este doc é obsoleto

Quando Phase C fecha e Enio aprova smoke. Aí move para
`docs/Migracao/2026-05-wave-8-phase-c-completed.md` ou deleta.
