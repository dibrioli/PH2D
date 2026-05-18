# ADR-0029 — Trait-driven panel host (PanelHost + Panel<State>) — endgame post-Wave-7

**Status:** Accepted
**Data:** 2026-05-18 (Proposed) → 2026-05-18 (Accepted — Enio aprovou Phase A.5)
**Decisor(es):** Enio + LLM (review por outro agente incorporado)
**Substitui:** parcialmente ADR-0028 §Wave 6+7 (panel-as-crate alias model)
**Habilita:** Wave 8 Phase 2.B-F + fechamento definitivo de auditoria S1+A2.

---

## 1. Contexto

Wave 6+7 entregou painéis como crates (`ph2d-panel-*`) **em forma de alias** —
cada crate re-exporta `PANEL_MANIFEST` de `ph2d_editor::screens::hero::*`.
Wave 8 Phase 2.A.0 + 2.A descolou panel-chrome + showcase tree pra
`ph2d-editor-core`, mas a fronteira final permanece bloqueada:

**Cycle insolúvel** pelo modelo atual:

- `ph2d-panel-inspector::paint_fn` precisa `&mut HeroScreen` (god-struct vive em ph2d-editor).
- `ph2d-editor::hero.rs` precisa `ph2d_panel_inspector::PANEL_MANIFEST` (pra registrar).
- → cycle ph2d-editor ⇄ ph2d-panel-inspector.

A consequência: painéis **não conseguem dropar a dep em `ph2d-editor`**. O painel
físico permanece físico-em-aparência (crate separado) mas **estruturalmente
acoplado** ao orchestrator god-crate. 3rd-party panel continua exigindo fork.
Audit findings S1+A2 permanecem abertos.

ADR-0028 §Wave 6/7 reconheceu esse cycle como "resolvido por Wave 2 codegen
não-ainda-implementado" + adiou. ADR-0029 é a resolução estrutural.

---

## 2. Decisão

Adotar **trait-driven panel host** com dois primitives:

1. **`PanelHost`** — trait que descreve a interface pano-de-fundo que painéis
   consomem do host. Vive em `ph2d-editor-core`.

2. **`Panel`** — trait que cada panel crate implementa, com `type State`
   associada (não `dyn Any`). Manifest declarado via `const MANIFEST: PanelManifest`.

Consequências estruturais:

- Painel depende **só de `ph2d-editor-core`**. Zero `ph2d-editor` dep.
- `HeroScreen` impl `PanelHost`. Encolhe de god-struct (33 fields pré-Wave-5 →
  17 atual → ~8-10 post-refactor): per-panel state migra pros panel crates,
  cross-panel state fica.
- `ph2d-editor` é **deletado** ou vira shim de re-exports — todo conteúdo
  substantivo move pra `ph2d-editor-core`.
- Cycle ph2d-editor⇄panel-crate **deixa de existir** estruturalmente.

Adotamos **typed Panel<State>**, NÃO `dyn Any`-based registry (justificativa
em §4.3 + alternative-considered §9).

---

## 3. Arquitetura nova do workspace

```
ph2d-tokens, ph2d-a11y, ph2d-vector, ph2d-text, ph2d-host, ph2d-grid    ← primitives
        ↓
ph2d-editor-core
    │   widgets/, paint/, interaction/, zones/, ids/, project/, …       (atual)
    │   panel/                                                          (novo)
    │     host.rs            — pub trait PanelHost { … }
    │     internal_host.rs   — pub trait PanelHostInternal: PanelHost { … }
    │     panel_trait.rs     — pub trait Panel { type State; … }
    │     manifest.rs        — pub struct PanelManifest { fn pointers }
    │     registry.rs        — runtime registry (já existe; move pra cá)
    │     erased.rs          — ErasedPanel<T: Panel> wrapper (downcast 1x)
    │     event_outcome.rs   — EventOutcome enum (já existe; move pra cá)
    │   screen/
    │     hero.rs            — HeroScreen struct + impl PanelHostInternal
    │     layout.rs          — HeroLayout (4 zonas)
    │     orchestrator.rs    — paint_hero_screen
    │     fixture.rs, chrome (topbar/left_rail/bottom_hud), style hero
        ↓
ph2d-panel-inspector, -hierarchy, -grid-snap, -widget-gallery
    [dependencies] ph2d-editor-core   (ph2d-editor NÃO está listado)
    │   Cada crate:
    │     lib.rs               — pub static MANIFEST + impl Panel for ThisPanel
    │     state.rs             — per-panel state (ex InspectorState)
    │     paint.rs / event.rs  — paint/event logic
        ↓
ph2d-panel-registry-init   ← aggregator com cargo features per panel (atual)
        ↓
ph2d-host-desktop          ← cria HeroScreen, install registry, paint loop
```

**`ph2d-editor` desaparece.** Re-exports temporários durante migração
(arquivos `lib.rs` apontando pros novos paths em editor-core); depois deletado.

---

## 4. Detalhes técnicos

### 4.1 `PanelHost` — tier público (estável)

Trait mínimo, alvo de 3rd parties. Surface esperada ~8-12 métodos após
estabilização (NÃO desenhar de cara — carve out depois de 6 meses de uso
real do tier internal). Conteúdo provável:

- `theme(&self) -> Theme`
- `viewport(&self) -> Rect`
- `current_panel_state<S: 'static>(&self) -> Option<&S>` / `_mut`
- `panel_visible(&self, id: PanelId) -> bool`
- `toast(&mut self, t: Toast)` — cross-panel notification
- `project(&self) -> &ProjectSettings`

Decisão design: **NÃO** comprometer surface pública agora. ADR-0029 não fixa.
ADR-0030 (futuro, ~6 meses pós-migração) faz o carve-out.

### 4.2 `PanelHostInternal: PanelHost` — tier instável (in-tree)

Trait amplo, alvo dos 4 painéis in-tree. Surface esperada ~25-30 métodos.
Lista observada do uso atual de painéis em HeroScreen:

- `store(&self) -> &WidgetStore`
- `store_mut(&mut self) -> &mut WidgetStore`
- `hit_index_mut(&mut self) -> &mut HitIndex`
- `selection(&self) -> Option<&HeroSelection>`
- `selection_mut(&mut self) -> &mut Option<HeroSelection>`
- `bus_mut(&mut self) -> &mut ActionBus`
- `view(&self) -> &ViewState` (mirror, stats, grid_visible, gallery_visible)
- `view_mut(&mut self) -> &mut ViewState`
- `image_edit(&self) -> &ImageEditState`
- `gizmo(&self) -> &GizmoStateGroup` / `_mut`
- `grid(&self) -> &GridState` / `_mut`
- `dragging_files(&self) -> Option<&DraggingFiles>` / `set_dragging_files`
- `live_hierarchy_entries(&self) -> Option<&BTreeMap<NodeId, HierarchyEntity>>`
- `display_unit(&self) -> DisplayUnit`
- `pixels_per_meter(&self) -> f32`
- `tool_registry(&self) -> Option<&ToolRegistry>`
- `stats(&self) -> BottomHudStats`
- `import_requested(&self) -> bool` / `request_import`
- `camera_reset_pending(&self) -> bool` / `request_camera_reset`
- `panel_state<P: Panel>(&self) -> &P::State` / `_mut`   (downcast typed via Panel trait)

Tier internal `pub(crate)` ou `#[doc(hidden)]` — disponível pros 4 painéis
in-tree mas não pro mundo externo.

### 4.3 `Panel<State>` trait — typed, NÃO dyn Any

```rust
pub trait Panel: Sized + 'static {
    type State: Default + Send + 'static;

    const ID: &'static str;
    const NODE_ID: NodeId;
    const DEFAULT_VISIBLE: bool;

    fn paint(state: &mut Self::State, ctx: &mut PaintCtx);
    fn apply_event(
        state: &mut Self::State,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome;
    fn populate(store: &mut WidgetStore);
}
```

Justificativa typed (não dyn Any):

- **Compile-time check Panel↔State.** Rename de InspectorState pega no rustc,
  não no smoke do primeiro frame.
- **Refactor de state struct** (split, merge, rename, mover fields) é gate
  estática no compilador. Bug runtime-only inaceitável em foundation que dura
  anos.
- **Custo:** ~50 LOC de glue `ErasedPanel<P: Panel>` que faz downcast UMA vez
  na install do manifest, depois passa typed refs. Pattern bem entendido
  (Bevy System, axum Handler).
- **Não-custo:** 5ns de downcast por chamada é invisível mesmo em 60Hz × 4
  panels × N eventos.

### 4.4 `ErasedPanel` — wrapper

```rust
// editor-core::panel::erased
pub struct ErasedPanel {
    pub manifest: PanelManifest,
    pub state_factory: fn() -> Box<dyn Any + Send>,
    state: Box<dyn Any + Send>,
}

impl ErasedPanel {
    pub fn new<P: Panel>() -> Self {
        Self {
            manifest: PanelManifest::for_panel::<P>(),
            state_factory: || Box::new(<P::State>::default()),
            state: Box::new(<P::State>::default()),
        }
    }
    pub fn paint<P: Panel>(&mut self, ctx: &mut PaintCtx) {
        let state = self.state.downcast_mut::<P::State>()
            .expect("ErasedPanel state type mismatch");
        P::paint(state, ctx);
    }
    // similar para apply_event, populate
}
```

`PanelManifest` agora segura fn pointers que dispatcham pro typed Panel:

```rust
pub struct PanelManifest {
    pub id: &'static str,
    pub panel_node_id: NodeId,
    pub default_visible: bool,
    pub paint_fn: fn(&mut ErasedPanel, &mut PaintCtx),
    pub apply_event_fn: fn(&mut ErasedPanel, &mut dyn PanelHostInternal, WidgetEvent) -> EventOutcome,
    pub populate_fn: fn(&mut WidgetStore),
    pub state_factory: fn() -> Box<dyn Any + Send>,
}
```

Manifest constrói-se via `PanelManifest::for_panel::<P>()` que injeta os fn
pointers automaticamente. Painéis NÃO escrevem fn pointers manualmente.

### 4.5 `PaintCtx`

```rust
pub struct PaintCtx<'a> {
    pub host: &'a mut dyn PanelHostInternal,
    pub layout: &'a HeroLayout,
    pub viewport: Rect,
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
}
```

Painéis acessam state via `host.panel_state_mut::<MyPanel>()` (typed downcast
através do panel trait que provê o State type). Diferente de `dyn Any.downcast_mut`,
o compilador rejeita downcast inválido.

### 4.6 Registry + boot order

`PANEL_REGISTRY` continua sendo `OnceLock<PanelRegistry>`. Mudança: agora
guarda `Vec<ErasedPanel>` (não `&'static PanelManifest`). Cada Panel é
instanciado uma vez no `register_all_panels()`.

```rust
// ph2d_panel_registry_init
pub fn register_all_panels() -> bool {
    let mut reg = PanelRegistry::new_empty();
    #[cfg(feature = "panel-inspector")]
    reg.push(ErasedPanel::new::<ph2d_panel_inspector::InspectorPanel>());
    #[cfg(feature = "panel-hierarchy")]
    reg.push(ErasedPanel::new::<ph2d_panel_hierarchy::HierarchyPanel>());
    // …
    install_panel_registry(reg)
}
```

Boot order obrigatório (Phase 1 already enforces):

1. `register_all_panels()` — instala registry.
2. `HeroScreen::new(...)` — cria struct concreta + impl PanelHostInternal.

### 4.7 EventOutcome

Já existe (Wave 8 Phase 4, commit `269adb8`). Move de `ph2d-editor::panel_registry`
pra `ph2d-editor-core::panel::event_outcome` na migração.

---

## 5. Plano de migração

### Phase A — DESIGN (1-2 dias)

**Sem código de implementação. Sem mexer em painel.**

A.1. Definir tier internal completo (`PanelHostInternal`) com lista exata de
    métodos. ~25-30 acessores. Documentar cada um com (a) painel que usa,
    (b) razão.

A.2. Definir tier público (`PanelHost`) com subset mínimo. ~8-12 métodos.
    Marcar como "estável post-6-meses-uso-real" — não comprometer ainda.

A.3. Esboçar `Panel` trait + `ErasedPanel` wrapper. Validar com 1 painel
    (Inspector) em pseudocódigo: como InspectorState migra? quais métodos
    consumiria?

A.4. Architecture test: `panel_host_surface_count` que conta métodos do trait
    e falha se passa de N. Forçar disciplina ao adicionar.

A.5. ADR-0029 → status `Accepted`. **CHECKPOINT 1: Enio aprova design.**

### Phase B — MIGRATE INFRASTRUCTURE (~2 dias + 1 checkpoint)

B.1. Criar `ph2d-editor-core/src/panel/` com traits + ErasedPanel + manifest
    + registry + EventOutcome moved over. Sem painéis ainda.

B.2. `HeroScreen` move de `ph2d-editor/src/screens/hero.rs` pra
    `ph2d-editor-core/src/screen/hero.rs`. Junto com layout, orchestrator,
    chrome (topbar/left_rail/bottom_hud), fixture, style.

B.3. `HeroScreen` impl `PanelHostInternal`. Surface inteira preenchida.

B.4. `paint_hero_screen` orchestrator atualizado pra iterar `Vec<ErasedPanel>`.

B.5. `ph2d-editor` vira shim de re-exports apontando pra editor-core.

B.6. Workspace compila. **CHECKPOINT 2: cargo check + smoke "editor abre, painéis vazios renderizam"** (vão estar usando ainda os old manifests via shim — função reduzida).

### Phase C — MIGRATE PAINÉIS (~3-4 dias, 1 checkpoint por painel)

Por painel (4 painéis = 4 mini-checkpoints internos, 1 checkpoint Enio no fim):

- Inspector primeiro (heaviest, ~3500 LOC, é o test bed).
  - `InspectorState` move pro panel crate.
  - Impl `Panel` trait.
  - Drop dep `ph2d-editor`; adiciona deps narrow.
  - Update consumers (inspector_sync.rs, etc.).
- Hierarchy (similar shape, ~1500 LOC).
- Widget Gallery (already mostly migrated — finaliza state isolation).
- Grid Snap (já tem state isolado; mais simples).

Cada painel **isoladamente**: build + check. Não commit per panel — commit ao
final dos 4 com diff completo (cadência v1.2 — 1200 LOC threshold, commit no
fim do bloco lógico). **CHECKPOINT 3: workspace verde + smoke "4 painéis renderizam, input funciona".**

### Phase D — CLEANUP (~1 dia)

D.1. Delete `ph2d-editor` crate completamente (ou mantém como deprecated shim
    com `#[deprecated]` em cada re-export se downstream consumer for sensível).

D.2. Arquitecture test `panel_crates_depend_only_on_editor_core` sai do
    `#[ignore]`. Ativa permanentemente.

D.3. Architecture test surface count gates trait crescer.

D.4. ADR-0028 §Wave 8 amenda com closeout. SKILL bump. STATE.md sha bom.
    DIRETRIZ se aplicável.

D.5. `register_all_panels()` docstring atualiza.

**CHECKPOINT 4: cargo test workspace + smoke + push pra CI. PRCI babysit.**

---

## 6. Architecture invariants + tests

Implementar como tests permanentes em CI:

### 6.1 Cycle prevention (já existe, ativa)

`crates/ph2d-editor/tests/architecture_cycle_prevention.rs` —
`panel_crates_depend_only_on_editor_core` sai de `#[ignore]`. Falha se
qualquer `crates/ph2d-panel-*/Cargo.toml` tem `ph2d-editor` no
`[dependencies]`.

### 6.2 Surface area gate (novo)

`crates/ph2d-editor-core/tests/architecture_panel_host_surface.rs` —
parseia o source de `panel/host.rs` + `internal_host.rs`, conta métodos
`fn` em cada trait. Asserta:

- `PanelHost` (público) ≤ 12 métodos.
- `PanelHostInternal` (interno) ≤ 35 métodos.

Quando crescer além, force review explícito (subir N ou justificar com
comentário no método).

### 6.3 Editor-core invariants (já existe)

`editor_core_has_no_panel_or_editor_deps` permanece como gate.

### 6.4 Public API surface (novo)

`crates/ph2d-editor-core/tests/architecture_api_surface.rs` (mencionado em
Wave 8 brief Phase 3.C, agora real) — conta items pub sem `#[doc(hidden)]`.
Threshold inicial ~80 (HeroScreen + paint_hero_screen + Theme + Panel + etc.).
Falha se passa.

---

## 7. Trade-offs / Consequências

### Aceitos

- **Tempo:** ~2-3 semanas de migração focada. Não-trivial. Aceito como custo
  do endgame real (Wave 6+7 entregaram aparência sem estrutura; ADR-0029
  entrega estrutura).
- **API churn:** ~100+ call sites mudam (panel internals migrating). Tudo
  caught pelo rustc — não risco de regressão silenciosa.
- **Tier internal grande (~30 métodos):** aceito porque é doc-hidden + interno;
  carve out do tier estável é trabalho separado pós-6-meses de uso real.
- **PaintCtx dyn dispatch:** ~5ns per call de vtable. Invisível em 60Hz.
- **ph2d-editor crate apagado:** consumer crates de fora (shells) atualizam
  imports. Re-exports temporários cobrem migração; remove em D.1.

### Rejeitados

- **dyn Any state per painel:** rejeitado em favor de typed Panel<State>. R1
  do review do outro agente discordou; análise nossa em §4.3 mantém typed.
- **Hot-reload como benefício declarado:** R4 do review correto — pattern
  habilita, mas requer abi_stable + repr(C) boundary + toolchain match.
  iOS/Android proíbe. Documentar como "potencialmente viável em desktop",
  não como entrega.
- **Bevy-style dependency injection** (panel declara `needs: {Theme, &mut Store}`):
  over-engineering pra escala atual. Reconsider se chegar 20+ painéis.

### Riscos identificados

1. **Surface trait pode crescer descontrolado.** Mitigação: surface count
   architecture test (§6.2) + review explícito ao subir N.
2. **HeroScreen ainda god-struct (~8-10 fields restantes).** Mitigação:
   aceitar — cross-panel state genuinamente compartilhado é diferente de
   per-panel state. Não force decomp artificial.
3. **Migração de Inspector é grande (~3500 LOC).** Mitigação: fazer primeiro,
   é o test bed; aprendizados aplicam aos outros 3.
4. **CI pipeline tempo aumenta** durante migração (cada commit re-compila
   tudo). Mitigação: cadência v1.2 (1200 LOC threshold + 1 commit per Phase)
   minimiza isso.

---

## 8. Acceptance criteria — quando Phase D fecha

- [x] `cargo test --workspace --exclude ph2d-asset` verde.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [x] `cargo build -p ph2d-host-desktop --no-default-features --features lite`
      → binário com chrome only.
- [x] `cargo build -p ph2d-host-desktop --no-default-features --features panel-inspector`
      → binário só com Inspector.
- [x] `cargo tree -p ph2d-panel-inspector --depth=2 | grep ph2d-editor` → vazio
      (NÃO depende de ph2d-editor).
- [x] `cargo tree -p ph2d-panel-{hierarchy,grid-snap,widget-gallery} --depth=2 | grep ph2d-editor` → todos vazios.
- [x] `cargo test -p ph2d-editor-core --test architecture_cycle_prevention` →
      verde, INCLUSIVE `panel_crates_depend_only_on_editor_core` (sem `#[ignore]`).
- [x] `cargo test -p ph2d-editor-core --test architecture_panel_host_surface`
      → PanelHost ≤ 12, PanelHostInternal ≤ 35.
- [x] `ph2d-editor` crate **deletado** (ou shim deprecated minimal — Enio decide).
- [x] HeroScreen LOC reduz de ~1300 pra ~400-500.
- [x] CI 10/10 jobs verde.
- [x] Smoke do Enio passou (4 painéis renderizam, eventos roteiam, lite-build
      boota sem painéis, panel-inspector-only boota só com Inspector).

---

## 9. Alternativas consideradas + rejeitadas

### 9.1 dyn Any state (revisão R1 do outro agente)

Proposta: `host.panel_state_mut(PanelId).downcast_mut::<InspectorState>().unwrap()`.

Pros: 50 LOC a menos de glue. Bevy Resources pattern, battle-tested.

Cons:
- Refactor de State é runtime-only check.
- Cada panel paga 1 downcast por chamada (~5ns).
- Mudanças em State scape ao rustc.

**Veredito:** rejeitado. 50 LOC pra trocar runtime-panic por compile-error é
trade-off favorável em foundation que vai durar anos.

### 9.2 Bevy-style dependency injection

Painel declara `needs: { Theme, &mut WidgetStore }`. Framework injeta.

Pros: ergonômico, declarativo.

Cons: macro/derive pesado pra suportar. ~500 LOC de framework code. Escala
atual (4 painéis) não justifica.

**Veredito:** rejeitado para Wave 8. Reconsiderar se chegar 20+ painéis.

### 9.3 Inverse dependency (ph2d-editor depende dos painéis, NÃO o contrário)

Pros: cycle resolved trivialmente.

Cons: panel crate STILL precisa some interface pra ler/escrever HeroScreen
state. Ou seja, ainda precisa trait. Mesmo problema.

**Veredito:** rejeitado. Resolve apenas a sintaxe do cycle, não o problema
estrutural.

### 9.4 Mover HeroScreen pra `ph2d-editor-core` SEM trait (impl direto)

Pros: minimal refactor. Painéis dependem de editor-core, acessam HeroScreen direto.

Cons: 3rd-party panel precisa conhecer concreta HeroScreen. Sem abstration
para outras hosts (preview windows, headless test). Acopla painéis a uma
implementação concreta de host forever.

**Veredito:** rejeitado. Quase-trabalho do trait sem o benefício.

---

## 10. Open questions (deixar pra implementação)

- **Q1:** `PanelHostInternal` é `pub(crate)` em editor-core, ou `#[doc(hidden)] pub`?
  - `pub(crate)`: painéis in-tree precisam estar no mesmo crate → não fazem
    sentido como crates separados.
  - `#[doc(hidden)] pub`: painéis externos podem usar tier interno tecnicamente
    mas é marcado como "unstable".
  - **Decisão prévia:** `#[doc(hidden)] pub`, deixa painéis externos opt-in
    no tier interno por sua conta e risco.

- **Q2:** `Panel::State` é `Default`? Ou exige `state_factory: fn() -> Self::State`?
  - Default mais ergonômico. State_factory permite parametrização (raro).
  - **Decisão prévia:** Default; revisitar se algum painel pedir factory.

- **Q3:** Per-panel state como `Box<dyn Any>` em ErasedPanel, ou plain `dyn Any`
  via trait object?
  - Box<dyn Any> é simples + permite Sized state. Trait object exige Sized
    state também mas via mais ginástica.
  - **Decisão prévia:** Box<dyn Any> dentro de ErasedPanel.

- **Q4:** `paint_hero_screen` itera registry em z-order. Onde fica a tabela
  z-order? Em HeroScreen ou em registry?
  - Hoje vive em hero.rs com NodeId-based lookup. Manter no host (HeroScreen)
    por enquanto.

---

## 11. Sequence + checkpoints

| Phase | Duration | Checkpoint | Bloqueia até |
|-------|----------|------------|--------------|
| A — Design | 1-2 dias | **#1: Enio aprova ADR-0029** | sim |
| B — Infra (Host trait + HeroScreen move + registry) | 2 dias | **#2: smoke editor + painéis old-style funcionam** | sim |
| C — Migrate painéis (Inspector→Hierarchy→Gallery→GridSnap) | 3-4 dias | **#3: smoke 4 painéis + lite + per-feature builds** | sim |
| D — Cleanup (delete ph2d-editor + arch tests + docs + push) | 1 dia | **#4: CI 10/10 verde + Enio aprova** | — |

**Total:** ~7-9 dias úteis. Cadência v1.2 — 1200 LOC threshold por check, 1
commit por Phase, smoke 4× total. Enio aprova entre Phases.

---

## 12. Pós-ADR-0029 — futuro próximo

- **ADR-0030 (~6 meses pós-merge):** carve out `PanelHost` tier público estável
  baseado em uso real observado.
- **Wave 9:** se demanda concreta surgir — hot-reload via dylib, MCP-driven
  panel composition, lite-build em produção, embedded preview window.
- **Cookbook 3rd-party panel** em `docs/IntegracaoMultiAgente/DIRETRIZ.md` §4.4
  atualizado pós-migração.

---

**Decisor Enio:** sim/não no Phase A.5 design checkpoint. Sem aprovação,
Phase B não inicia.
