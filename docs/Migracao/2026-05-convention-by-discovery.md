# Plano de Migração — Convention-by-Discovery + Shell Decomposition

**Versão:** 1.0 — 2026-05-16
**Status:** Proposto, aguardando aprovação para PR-1.
**Audiência:** LLMs (Coordenador, Periféricos, PRCI) + Enio.
**Escopo:** Refatorar PH2D de *convention-by-edit* (registries centrais editados por feature) para *convention-by-discovery* (tools como ilhas autônomas), e desinflar `shells/desktop/src/main.rs` (3463 LOC) num conjunto de módulos `growth-bounded`.

> **Princípio raiz:** este plano não substitui nem revoga decisões do SKILL.
> Toda invariante das Hard Rules HR-1..HR-17 e dos ADRs aceitos (0003-rev2,
> 0019, 0020, 0021, 0022, 0023, 0024) **continua valendo bit-a-bit**. A
> migração é estrutural — muda como o código é organizado, não como o
> código se comporta. A hierarquia §18 do SKILL é o desempate em qualquer
> dúvida.

---

## 1. Sumário executivo

### 1.1 O que está quebrado hoje

1. **Multi-agente serializa em 1 ponto.** Coordenador é único autorizado a
   editar arquivos compartilhados (lib.rs, tools/mod.rs, widget.rs,
   icons.rs, screens/hero/fixture.rs, screens/hero/ids.rs, Cargo.toml,
   shells/desktop/src/main.rs). Periféricos paralelos convergem todos
   nele. STATE.md hoje tem 3 slots `working` competindo pela mesma fila.

2. **`INTEGRATION.md` é cola humana para abstração faltante.** Cada handoff
   de Periférico → Coordenador descreve 30-80 LOC de edições mecânicas em
   N arquivos centrais. Exemplo: `tools/make_square/INTEGRATION.md` tem
   181 linhas. Isso é trabalho que deveria ser inexistente, não
   documentado.

3. **`main.rs` cresce sem teto.** 3463 LOC, com `render_frame()` sozinha
   ocupando 1825 LOC (19 blocos `pending_X.take()` inline) e
   `window_event()` ocupando 747 LOC (13 arms inflados). Cada feature nova
   adiciona 30-100 LOC em main.rs. Linear no número de features. Hostil a
   LLM por excesso de contexto, hostil a merge por superfície de conflito.

4. **Registries enumerados manualmente colidem.** `enum IconId` (89
   variants hoje), ranges de NodeId alocados à mão em `screens/hero/ids.rs`
   (100..199 TopBar, 200..299 LeftRail, etc.), `topbar_clusters()` Vec
   editado por feature. Convention-by-edit em todos esses pontos.

### 1.2 O que este plano entrega

Após os 13 PRs:

- **Cada tool é um crate isolado** (`crates/ph2d-tool-<slug>/`) com
  Cargo.toml/deps/testes próprios. Periférico trabalha 100% no seu crate.
- **Coordenador edita 1 linha por integração** (em `registry_init.rs`),
  contra 6-8 arquivos hoje. Resolução de merge 3-way trivial.
- **`main.rs` encolhe de 3463 → ~250 LOC** e fica `growth-bounded` —
  futuras features não engordam mais.
- **Chrome (TopBar, LeftRail, icons) é derivação pura** do registry.
  fixture.rs/ids.rs/icons.rs deixam de ser editados.
- **`pending_X` proliferation morre** — substituído por dispatcher
  genérico + módulo `hero_intents.rs` bounded por número de tipos de
  intent (não por número de tools).
- **Hard Rule HR-18 formaliza caps de LOC** (600/200/400) com CI gate.

### 1.3 Custo do plano

13 PRs, ~2-3 jornadas de trabalho. Cada PR é reversível. Nenhum PR
requer "big bang". Slots ativos (grid-snap, bgremoval) operam sob
modelo antigo sem interferência até concluírem; migram só pós-`done`.

### 1.4 Decisões fechadas

- **Sem `linkme`/`inventory`.** Riscos cross-platform (wasm32, iOS
  bitcode, MSVC ThinLTO) e ordem de iteração não-determinística matam
  a proposta de auto-registration via distributed_slice. Adotado
  **híbrido conservador**: registro explícito via `register_all()` em
  arquivo append-only.
- **PRs 9a/9b/9c inseridos** para refatorar `main.rs`.
- **HR-18 formalizada** com CI gate. Caps: 600 LOC por arquivo, 200
  LOC por função, 400 LOC para `main.rs` de shells.
- **MCP exposure reservada no manifest mas não wireada** nesta migração
  (HR-8/HR-11 são sub-projeto separado).
- **Shadow mode durante piloto** (PR 4-7): dispatcher novo e
  `pending_X` antigo coexistem até migração de uma tool completa.

---

## 2. Contexto detalhado

### 2.1 Diagnóstico canônico (auditado em 2026-05-16)

**Arquivos hoje editados por integração de feature nova:**

| Arquivo | LOC | O que adiciona por tool |
|---|---:|---|
| `shells/desktop/src/main.rs` | 3463 | `pending_X` field + drain block (30-80 LOC) + click handler + keymap |
| `crates/ph2d-editor/src/lib.rs` | 96 (50 `pub use`) | re-export |
| `crates/ph2d-editor/src/widget.rs` | 98 | `mod X` + `pub use` |
| `crates/ph2d-editor/src/tools/mod.rs` | 17 | `pub mod X` + `pub use` |
| `crates/ph2d-editor/src/icons.rs` | enum 89 variants | `IconId::X` + match arm + entry em `ALL_ICONS` |
| `crates/ph2d-editor/src/screens/hero/fixture.rs` | `topbar_clusters()` Vec | item novo |
| `crates/ph2d-editor/src/screens/hero/ids.rs` | NodeId ranges | const novo |
| `crates/ph2d-editor/Cargo.toml` | — | dep nova (Coordenador only) |

**Decomposição de `main.rs` (3463 LOC):**

| Faixa | O que é | LOC | Bounded? |
|---:|---|---:|:---:|
| 1-110 | imports / `mod` | 110 | quase |
| 110-360 | structs (`App`, `HeroLive`) | 250 | sim |
| 362-855 | helpers `impl App` | 493 | sim |
| 855-2680 | **`render_frame()` SOZINHA** | **1825** | **NÃO** — cresce com features |
| 2682-2941 | `resumed()` (init) | 260 | quase |
| 2941-3688 | **`window_event()`** | **747** | **NÃO** — arms inflam |
| 3689-fim | `fn main()` + tests | ~70 | sim |

Dois métodos concentram 75% do arquivo. Ambos crescem com features.

### 2.2 19 drenos de `hero.pending_*` em `render_frame`

Categorização (essencial para o plano):

**Drenos de Tool Action** (~4 blocos, ~400 LOC) — alvo do dispatcher
genérico:
- `pending_trim_transparency`
- `pending_make_square`
- `pending_reimport`
- `pending_sprite_source_change`

**Drenos de Inspector Intent** (~13 blocos, ~900 LOC) — bounded por
tipo de operação, NÃO por tool. Alvo de `hero_intents.rs`:
- `pending_transform_edit`, `pending_name_edit`, `pending_visibility_edit`
- `pending_visibility_toggle`, `pending_rename_seed`, `pending_rename_commit`
- `pending_reparent`, `pending_duplicate`, `pending_add_child`
- `pending_delete`, `pending_reset_transform`
- `pending_hierarchy_row_click`, `pending_view_focus`

**Drenos de Input/Lifecycle** (~2 blocos) — permanecem em `App`:
- `pending_drops`, `pending_resize`

### 2.3 Modelo multi-agente em curso

Ver `docs/IntegracaoMultiAgente/{01-Enio.md, 02-Coordenador.md,
03-Agente-Periferico.md, 04-Agente-PRCI.md, STATE.md}`. Resumo:
- 1 Enio (relay humano).
- 1 Coordenador (única sessão autorizada a tocar compartilhados +
  STATE.md).
- Até 4 Periféricos paralelos no mesmo path (sem worktrees, sem
  branches feature/).
- 1 PRCI (push pro GitHub + babysit CI no fim de jornada).

STATE.md em 2026-05-16: slot 1 grid-snap `working`, slot 2 bgremoval
`working`, slot 3 make-square `done`, slot 4 vago.

---

## 3. Princípios e invariantes preservados

Esta seção é o contrato com o SKILL. Cada item aqui é **obrigação** do
plano. Se algum PR pretende violar, o PR é inválido e o plano precisa
revisão antes de prosseguir.

### 3.1 Hard Rules preservadas

| HR | Implicação para o plano |
|---|---|
| **HR-1** core platform-agnostic | Refatoração de `shells/desktop/` NÃO toca crates `ph2d-*` exceto onde explicitamente migra módulos antigos. `ph2d-tool-*` crates são platform-agnostic (não usam winit, não usam `target_os`). |
| **HR-2** unsafe documentado | Nenhum `unsafe` novo introduzido pelo plano. |
| **HR-3** zero alloc hot path | Dispatcher genérico (PR 9) usa `VecDeque<ActionInvocation>` pré-alocada + payload em arena `bumpalo` per-frame. Validação via `dhat-rs` em `tests/budget/no_alloc_hot_path.rs`. `editor_layout` continua zero-alloc. |
| **HR-4** frame budget | Sub-budgets inalterados. Dispatcher entra no slot "Input + ECS scheduler" (0.5ms) — folga histórica suficiente. |
| **HR-5** determinismo | Registry ordena tools por `(cluster, order, id)` em build time. Nenhuma iteração consome ordem bruta. `NodeId::from_str_hash` usa FxHash 64-bit determinístico. Lint `touches_sim: bool` no manifest exige ordenação se afeta SimWorld. |
| **HR-6** asset blake3 | Inalterado — tool-crates não tocam asset DB exceto via APIs existentes. |
| **HR-7** editor=off corta 100% | Tool-crates são `optional = true` no Cargo.toml de `ph2d-editor`, gateadas pela feature `editor`. Build de jogo com `--no-default-features` omite todos `ph2d-tool-*`. CI job `nm + grep` valida ausência de símbolos `ph2d_tool_*` em binário de release. |
| **HR-8** handles opacos | Manifest expõe `id: &'static str` e `Entity`/`Handle<T>` em handlers, nunca pointers. Reforçado em revisão de PR de novo tool-crate. |
| **HR-9** GC Luau | Inalterado. |
| **HR-10** MCP first-class | Manifest reserva campo `mcp: McpExposure` com default `exposed: false`. Wiring real fica para sub-projeto MCP separado (não-bundle nesta migração). |
| **HR-11** destructive token | Idem — wiring real fora de escopo, mas campo reservado. |
| **HR-12** a11y tree | Manifest declara `a11y_role: Role`. Registry exige presença. Lint customizada já valida widgets — extensão para validar manifests é trivial. |
| **HR-13** memory budget | Manifest declara `memory_budget: MemoryBudget`. Registry agrega em init; recusa boot se soma > `MemoryBudget::platform_max`. |
| **HR-14** save versionado | Inalterado — plano não toca formato de save. |
| **HR-15** i18n | `label_key: &'static str` no manifest. CI lint coleta todos os keys declarados e valida presença em bundle Fluent `pt-BR.ftl` + `en-US.ftl`. |
| **HR-16** storage lateral POD | Inalterado. |
| **HR-17** examples Luau compilam | Inalterado. Se manifest declara API Luau, exemplo correspondente entra no test set. |

### 3.2 ADRs preservados

| ADR | Implicação |
|---|---|
| **ADR-0003-rev2** bevy_ecs | `bevy_ecs::App::add_plugins` NÃO é mecanismo de registro de tools (esse caminho foi explicitamente rejeitado por forçar API). Mantemos bevy_ecs como ECS, registry de tools é estrutura paralela. |
| **ADR-0019** Luau scripting | Inalterado. Tools podem expor APIs Luau via `#[lua_export]`; manifest pode declarar isso em campo opcional para validação cruzada (HR-10). |
| **ADR-0020** surface lifecycle | Inalterado. `SurfaceContext::acquire_frame()` continua único caminho público. `render_loop.rs` chama-a no orquestrador. |
| **ADR-0021** SimWorld/PresentWorld | Handlers recebem contexto encapsulando acesso aos worlds. Manifest declara `touches_sim: bool` para lint. Trait `SimComponent`/`PresentComponent` continua enforce em compile-time. |
| **ADR-0022** sem HashMap em sim | Registry de tools vive no editor (PresentWorld scope), pode usar `BTreeMap`/`HashMap` (não-sim). Lint workspace-wide continua. |
| **ADR-0023** UI 4-zonas | Manifest declara `zone: Zone` (enum, compile-time válido) + `cluster: &'static str` (validado em registry init contra lista de clusters válidos por zona — panic early). |
| **ADR-0024** input pipeline | `panel_builder: Option<fn(&mut WidgetStore)>` no manifest. Pre-população acontece no registry init, antes do primeiro frame. HR-3 zero-alloc preservado. Bench `tests/interaction_no_alloc.rs` continua. |

### 3.3 Modelo operacional multi-agente preservado

- Coordenador continua existindo, mas seu escopo encolhe drasticamente
  (de "edita 6-8 arquivos por integração" para "revisa manifest + edita
  1 linha em `registry_init.rs`").
- Periférico continua trabalhando em pasta exclusiva — mudança: a pasta
  vira `crates/ph2d-tool-<slug>/` (crate inteiro) em vez de subdir em
  `tools/`. Periférico ganha autonomia sobre Cargo.toml do seu crate.
- STATE.md continua sendo o ledger. Histórico append-only preservado.
- PRCI inalterado.

### 3.4 Stack inalterada

Nenhuma versão de dependência muda neste plano. Refatoração é puramente
estrutural sobre stack já pinada em 2026-05-09 (vide SKILL §5).

---

## 4. Design alvo

### 4.1 Arquitetura pós-migração

```
crates/
  ph2d-core/                 (inalterado)
  ph2d-host/                 (inalterado)
  ph2d-ecs/                  (inalterado)
  ph2d-gpu/                  (inalterado)
  ph2d-render/               (inalterado)
  ph2d-vector/               (inalterado)
  ph2d-text/                 (inalterado)
  ph2d-physics/              (inalterado)
  ph2d-asset/                (inalterado)
  ph2d-script/               (inalterado)
  ph2d-input/                (inalterado)
  ph2d-tokens/               (inalterado)
  ph2d-editor/               (registry + chrome derivado)
    src/
      lib.rs                 (re-exports MÍNIMOS após cleanup)
      registry/              (novo módulo)
        mod.rs
        manifest.rs          (struct ToolManifest)
        registry_init.rs     (append-only — único ponto compartilhado)
        node_id.rs           (NodeId::from_str_hash + collision detect)
        icon_handle.rs       (IconHandle wrapper)
        action.rs            (ActionInvocation + Dispatcher)
      tools/
        mod.rs               (pub mod brush, pub mod move_tool — encolhe)
        registry_init.rs     (← arquivo append-only do plano)
        brush.rs             (inalterado por enquanto)
        move_tool.rs         (inalterado por enquanto)
      screens/hero/
        fixture.rs           (topbar_clusters() derivado do registry)
        ids.rs               (encolhe — só IDs de chrome fixo)
  ph2d-mcp/                  (inalterado neste plano)
  ph2d-a11y/                 (inalterado)

  ph2d-tool-brush/           ⬅ NOVOS — extraídos um a um
  ph2d-tool-move/
  ph2d-tool-make-square/     (primeiro piloto)
  ph2d-tool-trim-transparency/
  ph2d-tool-grid-snap/       (migrado pós-done)
  ph2d-tool-bgremoval/       (migrado pós-done)
  ph2d-tool-<slug>/          (futuros)

shells/desktop/src/
  main.rs                    (← 3463 LOC → ~250 LOC, bounded)
  app.rs                     (← struct App + ApplicationHandler dispatch)
  render_loop.rs             (← orquestrador, ~200 LOC bounded)
  hero_intents.rs            (← drenos de Inspector intent, ~900 LOC, bounded em N intents)
  input_dispatch.rs          (← arms de WindowEvent decompostos, ~750 LOC)
  init.rs                    (← resumed() decomposto, ~260 LOC)
  tool_actions.rs            (← drena ActionInvocation via Registry, ~80 LOC)
  (módulos existentes: cursor_pos, forwarding, hero_bridge, image_import,
   input_log, integration, keymap, theme, winit_host, gilrs_adapter)
```

### 4.2 Fluxo de dados pós-migração

```
Frame N:
  ┌─ App.window_event(ev)
  │    └→ input_dispatch::on_<event_kind>(app, ev)
  │         └→ ph2d_editor::interaction::dispatch_pointer/key/...
  │            (existing, unchanged — ADR-0024)
  │
  ├─ App.about_to_wait()
  │    └→ window.request_redraw()
  │
  └─ App.window_event(RedrawRequested)
       └→ render_loop::render_frame(app)
            ├→ tick_inputs(app)
            ├→ hero_intents::drain_all(app)            # 13 funções pequenas
            ├→ tool_actions::drain(app, &app.registry) # dispatcher genérico
            ├→ extract_to_present(app)                 # ADR-0021 boundary
            ├→ paint(app)
            └→ present(app)                            # ADR-0020 acquire_frame
```

### 4.3 Anatomia de um tool-crate

```
crates/ph2d-tool-make-square/
  Cargo.toml                 (deps próprias do tool)
  src/
    lib.rs                   (pub fn register(reg: &mut Registry) + pub const MANIFEST)
    manifest.rs              (definição de MANIFEST)
    algorithm.rs             (lógica pura, já existe hoje)
    icon.rs                  (fn icon() -> BezPath)
    handler.rs               (fn on_clicked(ctx: &mut ToolCtx) p/ Action one-shot
                              OU fn build_panel/handle_event p/ Tool stateful)
  tests/
    algorithm.rs             (testes unitários do algoritmo, já existem)
    manifest.rs              (smoke: ToolManifest válido, label_key Fluent OK)
```

### 4.4 Manifesto canônico

```rust
pub struct ToolManifest {
    pub id: &'static str,
    pub label_key: &'static str,
    pub icon_fn: fn() -> BezPath,
    pub zone: Zone,
    pub cluster: &'static str,
    pub order: u32,
    pub a11y_role: Role,
    pub handler: ToolHandler,
    pub memory_budget: MemoryBudget,
    pub touches_sim: bool,
    pub mcp: McpExposure,
}

pub enum ToolHandler {
    OneShot(fn(&mut ToolCtx)),
    Stateful {
        panel_builder: fn(&mut WidgetStore),
        on_event: fn(&mut ToolCtx, PanelEvent),
        on_activate: fn(&mut ToolCtx),
        on_deactivate: fn(&mut ToolCtx),
    },
}

pub struct McpExposure {
    pub exposed: bool,        // default: false (reservado, não wireado nesta migração)
    pub destructive: bool,    // default: false
    pub handle_only: bool,    // default: true
}
```

### 4.5 Ponto único de contato compartilhado

```rust
// crates/ph2d-editor/src/tools/registry_init.rs   (append-only, ~30 LOC alvo)
pub fn register_all(reg: &mut Registry) {
    ph2d_tool_brush::register(reg);
    ph2d_tool_move::register(reg);
    ph2d_tool_make_square::register(reg);
    ph2d_tool_trim_transparency::register(reg);
    ph2d_tool_grid_snap::register(reg);
    ph2d_tool_bgremoval::register(reg);
    // Periférico novo: adiciona UMA linha. Merge 3-way trivial.
}
```

### 4.6 Obrigações do registry init

Em ordem, durante `Registry::build()`:

1. Coletar todos `manifest.id` → hash via `FxHash::const_hash` → detectar
   colisão → **panic com mensagem `"NodeId collision: ids X and Y both
   hash to Z. Rename one."`**.
2. Validar que `manifest.cluster` ∈ clusters válidos da `manifest.zone`
   (lookup em tabela canônica de ADR-0023) → panic se inválido.
3. Somar `manifest.memory_budget` → comparar com
   `MemoryBudget::platform_max()` → panic se estoura (HR-13).
4. Ordenar tools por `(cluster, order, id)` → ordem determinística
   cross-platform (HR-5).
5. Construir índices: `BTreeMap<&str, &ToolManifest>` por id,
   `BTreeMap<NodeId, &ToolManifest>` por hash, vec ordenado por
   `(zone, cluster)`.

---

## 5. Plano de execução — 13 PRs

> **Restrições de execução globais:**
> - Nenhum PR quebra os 1098 testes hoje passantes.
> - Nenhum PR força slots ativos (grid-snap, bgremoval) a parar.
> - Cada PR é reversível por `git revert <sha>`.
> - PR é mergeado em `main` local; push pro GitHub continua sendo papel
>   do PRCI no fim da jornada.
> - Cada PR cita HR aplicável no commit message (convenção SKILL §20).

### PR 1 — Foundation: Registry + ToolManifest + Dispatcher (vazios)

**Objetivo:** Criar a infraestrutura nova **sem migrar nada**. Sistema
antigo continua intacto.

**Arquivos novos:**
- `crates/ph2d-editor/src/registry/mod.rs`
- `crates/ph2d-editor/src/registry/manifest.rs` (struct + enums)
- `crates/ph2d-editor/src/registry/action.rs` (ActionInvocation,
  Dispatcher trait, payload arena)
- `crates/ph2d-editor/src/registry/registry.rs` (struct Registry,
  fn build, índices)
- `crates/ph2d-editor/src/tools/registry_init.rs` (vazio: `pub fn
  register_all(_reg: &mut Registry) {}`)

**Arquivos editados:**
- `crates/ph2d-editor/src/lib.rs` (+1 linha: `pub mod registry;`)

**Critério de aceite:**
- `cargo check -p ph2d-editor` passa.
- `cargo test -p ph2d-editor` passa (testes existentes intactos).
- `Registry::default().build()` retorna registry vazio sem panic.
- `dhat` bench: `Registry::build()` aloca apenas no heap esperado
  (alocação fora de hot path, OK).

**Risco:** ~zero. Apenas adiciona código novo.

**Rollback:** `git revert`.

**Cita HR:** HR-3 (dispatcher arena pré-alocada), HR-5 (ordenação
determinística), HR-13 (campo budget no manifest).

---

### PR 2 — Helpers: NodeId hash + collision + IconHandle wrapper

**Objetivo:** Implementar primitivos de identificação estável.

**Arquivos novos:**
- `crates/ph2d-editor/src/registry/node_id.rs`
  - `pub const fn hash_node_id(s: &'static str) -> NodeId` (FxHash 64-bit)
  - `pub fn detect_collisions(manifests: &[ToolManifest]) -> Result<(),
    CollisionError>`
- `crates/ph2d-editor/src/registry/icon_handle.rs`
  - `pub struct IconHandle(&'static str)` newtype
  - `pub fn resolve(handle: IconHandle, reg: &Registry) -> Option<fn() ->
    BezPath>` — lookup direto no manifest da tool dona

**Arquivos editados:**
- `crates/ph2d-editor/src/registry/mod.rs` (+2 re-exports).

**Testes novos:**
- `crates/ph2d-editor/tests/registry_node_id.rs` — propriedade: hash
  estável cross-platform via fixture com 100 strings; colisão detectada
  e retorna `Err`.

**Critério de aceite:**
- Hash de `"tool.make_square.button"` é o mesmo em Linux/Mac/Win (CI
  matrix existente cobre).
- Colisão sintética força panic com mensagem clara.
- `enum IconId` antigo permanece — coexistência.

**Risco:** baixo. Hash determinístico é fácil de testar.

**Cita HR:** HR-5 (cross-platform stable hash).

---

### PR 3 — CI lint stack (HR-13, HR-15, HR-7)

**Objetivo:** Ter os gates HR antes de qualquer tool migrar — sem isso,
gates escapam pela arquitetura nova.

**Arquivos novos:**
- `tests/architecture/manifest_budget_aggregate.rs` — itera
  `Registry::default()` + `register_all` + agrega budget, valida contra
  `MemoryBudget::platform_max` para cada plataforma da matriz §4 do
  SKILL.
- `tests/architecture/manifest_i18n_keys.rs` — coleta todos
  `label_key` declarados, parseia bundles Fluent existentes
  (`crates/ph2d-editor/locales/*.ftl` quando existir; até lá, valida
  formato de chave `tool.<slug>.label`).
- `tests/architecture/no_tool_symbols_in_release.rs` — builda
  `shells/desktop` com `--no-default-features --features release-game`
  (mesmo flag que HR-7 já usa); roda `nm` no binário; falha se grep
  encontra `ph2d_tool_`.

**Arquivos editados:**
- `.github/workflows/spike.yml` — adicionar 3 jobs (rodam em paralelo
  no matrix existente).

**Critério de aceite:**
- 3 jobs verdes em CI com registry vazio (caso base).
- Smoke: injetar manifest sintético com budget acima de iOS max →
  job falha com mensagem clara.

**Risco:** médio — CI matrix pode ficar mais lenta. Mitigação: jobs
rodam em paralelo, não em série.

**Cita HR:** HR-13, HR-15, HR-7.

**Nota:** este PR é INTENCIONALMENTE antes do piloto (PR 4) — sem os
gates, a primeira tool migrada poderia furar HR-15 ou HR-13 sem
detecção.

---

### PR 4 — Piloto: extrair `crates/ph2d-tool-make-square/` (shadow mode)

**Objetivo:** Provar end-to-end com tool já `done` no STATE.md.
make-square é alvo seguro: algoritmo estável, testes passando, sem
dependência viva de Periférico.

**Arquivos novos:**
- `crates/ph2d-tool-make-square/Cargo.toml`
  - `edition = "2024"`, MSRV 1.92
  - `[dependencies]` apenas: `kurbo` (via vello re-export),
    `ph2d-editor` (path), `ph2d-a11y` (path)
- `crates/ph2d-tool-make-square/src/lib.rs`
  - `pub const MANIFEST: ToolManifest = ToolManifest { ... };`
  - `pub fn register(reg: &mut Registry) { reg.register(&MANIFEST); }`
- `crates/ph2d-tool-make-square/src/algorithm.rs` — copy de
  `crates/ph2d-editor/src/tools/make_square/algorithm.rs`
- `crates/ph2d-tool-make-square/src/icon.rs` — copy de
  `crates/ph2d-editor/src/tools/make_square/icon.rs`
- `crates/ph2d-tool-make-square/src/handler.rs` — extrai
  `on_make_square_clicked` de `shells/desktop/src/main.rs`
- `crates/ph2d-tool-make-square/tests/algorithm.rs` — copy de teste
  existente

**Arquivos editados:**
- `Cargo.toml` raiz — `members = ["crates/*", "shells/desktop", ...]`
  (glob já cobre ou adicionar explícito)
- `crates/ph2d-editor/Cargo.toml` — `ph2d-tool-make-square = { path =
  "../ph2d-tool-make-square", optional = true }` sob feature `editor`
- `crates/ph2d-editor/src/tools/registry_init.rs` — `+1 linha:
  ph2d_tool_make_square::register(reg);`
- `shells/desktop/src/main.rs` — drena `pending_make_square` via
  dispatcher genérico (em vez de inline). **Outras 18 pending_* permanecem
  inline** (shadow mode).

**Arquivos NÃO removidos ainda:**
- `crates/ph2d-editor/src/tools/make_square/` — permanece, mas vira
  re-export thin de `ph2d_tool_make_square::*` para não quebrar consumers
  intermediários. Removido em PR 10.

**Critério de aceite:**
- `cargo check --workspace` verde.
- `cargo test --workspace` verde (1098 testes + novos do
  manifest_budget_aggregate).
- Smoke visual: `PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop` →
  clicar em [▢ Make Square] na TopBar → sprite vira quadrado → undo
  funciona. **Comportamento idêntico ao atual**.
- `dhat` bench `tests/budget/no_alloc_hot_path.rs` continua verde.

**Risco:** médio. Primeiro tool migrado; testa toda a infraestrutura.

**Rollback:** `git revert PR-4` deixa make-square voltar a ser
sub-folder de `tools/`.

**Cita HR:** HR-3, HR-7 (tool-crate optional), HR-13 (budget agregado).

---

### PR 5 — Validação operacional (sem código novo)

**Objetivo:** medir tempo real de integração de tool nova no shape novo
antes de comprometer escala.

**Ação:** quando slot 4 do STATE.md receber próxima feature do Enio,
Coordenador instrui Periférico a usar shape novo (criar
`crates/ph2d-tool-<slug>/`). Coordenador cronometra:
- Tempo Periférico → "pronto".
- Tempo Coordenador → "integrado" (deve ser <10min — só adiciona 1
  linha em `registry_init.rs` + edita `Cargo.toml` de ph2d-editor).
- Conflitos de merge com outros slots: zero esperado.

**Critério de go/no-go:**
- Se Coordenador gasta >30min para integrar OU surge conflito com slot
  paralelo: pausar plano, revisar shape antes do PR 6.
- Se ≤10min e zero conflito: prosseguir para PR 6.

**Entregável:** entrada em STATE.md histórico documentando métricas
reais; decisão registrada.

**Risco:** zero. É um experimento controlado.

---

### PR 6 — Migrar grid-snap (pós-`done`)

**Pré-condição:** slot 1 do STATE.md em status `done` para grid-snap.
**Não tocar enquanto `working`.**

**Análise específica:** grid-snap é mais complexo (já tem 8 arquivos +
subdir `render/`, vide diagnóstico). Vai virar
`crates/ph2d-tool-grid-snap/` com mesma estrutura interna preservada.

**Arquivos novos:**
- `crates/ph2d-tool-grid-snap/Cargo.toml` — deps: `ph2d-grid` (path),
  `ph2d-editor`, `kurbo`, `ph2d-a11y`.
- `crates/ph2d-tool-grid-snap/src/lib.rs` (register + MANIFEST,
  Stateful porque tem painel)
- `crates/ph2d-tool-grid-snap/src/{state,panel,inspect,ids,render}.rs`
  — move de `ph2d-editor/src/grid_snap/`

**Arquivos editados:**
- `crates/ph2d-editor/Cargo.toml` — `ph2d-tool-grid-snap = { path,
  optional }`
- `crates/ph2d-editor/src/tools/registry_init.rs` — +1 linha
- `crates/ph2d-editor/src/lib.rs` — remover `pub mod grid_snap` (estava
  lá durante slot working)

**Critério de aceite:** mesmos de PR 4. Smoke visual: painel de Grid
Settings continua abrindo, todos kinds renderizando.

**Risco:** médio-alto (módulo maior). Mitigação: tool foi `done`,
testes existem.

---

### PR 7 — Migrar bgremoval (pós-`done`)

Idêntico em forma a PR 6. Pré-condição: slot 2 do STATE.md em `done`.

**Particularidade:** bgremoval tem deps externas (`image`, `rayon`) que
estavam em `ph2d-editor/Cargo.toml`. Move essas deps para
`ph2d-tool-bgremoval/Cargo.toml`; remove de `ph2d-editor/Cargo.toml` se
nenhum outro consumer.

**Cita HR:** HR-7 (deps específicas de tool não poluem editor).

---

### PR 8 — Chrome derivado (TopBar / LeftRail / icons)

**Objetivo:** parar de editar `screens/hero/fixture.rs`, `ids.rs`,
`icons.rs` por feature nova.

**Arquivos editados:**
- `crates/ph2d-editor/src/screens/hero/fixture.rs` —
  `topbar_clusters()` vira:
  ```rust
  pub fn topbar_clusters(reg: &Registry) -> Vec<(NodeId, TopBarCluster)> {
      let mut out = Vec::new();
      // Itens hard-coded de chrome fixo (Theme, Save, Open, Settings, etc.)
      out.push((ids::TOPBAR_THEME, TopBarCluster::theme(...)));
      // ... outros hard-coded
      // Itens derivados:
      for m in reg.manifests_for_zone(Zone::TopBar) {
          out.push((m.node_id(), m.into_topbar_cluster()));
      }
      out
  }
  ```
- `crates/ph2d-editor/src/screens/hero/left_rail.rs` — análogo.
- `crates/ph2d-editor/src/icons.rs` — itens migrados perdem variants
  do enum. Variants restantes (ícones de chrome fixo: Save, Open,
  Settings, Play, etc.) permanecem.

**Critério de aceite:**
- Render visual idêntico (golden image teste já cobre).
- Ordem dos itens preservada (registry ordena por `(cluster, order, id)`
  + chrome fixo vem antes/depois conforme posição no array).

**Risco:** médio. Mudança visual mais óbvia.

**Cita HR:** HR-5 (ordenação determinística), ADR-0023.

---

### PR 9a — Extrair `render_loop.rs` + `hero_intents.rs` (main.rs -900 LOC)

**Objetivo:** mover os 13 drenos de Inspector intent para módulo
bounded. Reduzir main.rs significativamente antes do dispatcher full.

**Arquivos novos:**
- `shells/desktop/src/render_loop.rs`
  - `pub fn render_frame(app: &mut App)` — orquestrador
  - Funções privadas: `tick_inputs`, `extract_to_present`, `paint`,
    `present` (extraindo blocos atuais).
- `shells/desktop/src/hero_intents.rs` — 13 funções pequenas:
  - `pub fn drain_view_focus(app: &mut App)`
  - `pub fn drain_visibility_toggle(app: &mut App)`
  - `pub fn drain_reparent(app: &mut App)`
  - `pub fn drain_duplicate(app: &mut App)`
  - `pub fn drain_add_child(app: &mut App)`
  - `pub fn drain_reset_transform(app: &mut App)`
  - `pub fn drain_delete(app: &mut App)`
  - `pub fn drain_hierarchy_row_click(app: &mut App)`
  - `pub fn drain_rename_seed(app: &mut App)`
  - `pub fn drain_rename_commit(app: &mut App)`
  - `pub fn drain_transform_edit(app: &mut App)`
  - `pub fn drain_visibility_edit(app: &mut App)`
  - `pub fn drain_name_edit(app: &mut App)`
  - `pub fn drain_all(app: &mut App) { ... 13 chamadas ... }`

**Arquivos editados:**
- `shells/desktop/src/main.rs`:
  - `+mod render_loop; +mod hero_intents;`
  - `impl App { fn render_frame(&mut self) { render_loop::render_frame(self); } }`
  - Remove os 1825 LOC de `render_frame()` inline.

**Critério de aceite:**
- `cargo test -p ph2d-host-desktop` verde.
- Smoke visual: editor abre, Inspector edita, Hierarchy reparenta,
  delete funciona — comportamento idêntico.
- `wc -l shells/desktop/src/main.rs` < 2700.
- HR-3 bench verde.

**Risco:** alto. Movimentação de 900+ LOC entre arquivos.
**Mitigação:** funções pequenas, uma por intent, fácil de auditar 1:1
com original. Recomendo PR ser feito em commits sequenciais (1 commit
por dreno extraído) para revisão granular.

**Cita HR:** HR-3, anti-pattern §15 (god-file).

---

### PR 9 — Dispatcher genérico full (substitui pending_X de actions)

**Objetivo:** absorver os 4 drenos restantes de Tool Action via
`tool_actions.rs`.

**Arquivos novos:**
- `shells/desktop/src/tool_actions.rs`
  - `pub fn drain(app: &mut App, reg: &Registry)` — itera
    `app.actions: VecDeque<ActionInvocation>`, lookup no registry,
    chama handler com `ToolCtx` montado.

**Arquivos editados:**
- `shells/desktop/src/main.rs`:
  - `App` struct: remove `pending_trim_transparency`,
    `pending_make_square`, `pending_reimport`,
    `pending_sprite_source_change`.
  - Adiciona `actions: VecDeque<ActionInvocation>` pré-alocada
    capacidade 256.
  - Adiciona `action_arena: Bump` per-frame, reset no início do
    `render_frame`.
- `shells/desktop/src/render_loop.rs`:
  - Após `hero_intents::drain_all`, chama `tool_actions::drain(app,
    &app.registry)`.
- Handlers das 4 tools (make-square já migrada em PR 4 — só
  trim_transparency, reimport, sprite_source_change ainda usavam
  `pending_X`). Esses 3 viram tools migradas se ainda não estavam
  (escopo expandido pra incluir migração mínima ou ações genéricas
  no shell).

**Critério de aceite:**
- `dhat` bench valida zero-alloc no enqueue de action (payload em
  arena bumpalo).
- `cargo test --workspace` verde.
- Smoke visual: Trim Transparency, Make Square, Re-import, Sprite
  Source change todos funcionam.

**Risco:** médio. Migração pontual de 4 fluxos.

**Cita HR:** HR-3 (dhat-validated), HR-7.

---

### PR 9b — Decompor `window_event()` em `input_dispatch.rs`

**Objetivo:** dissolver os 747 LOC de `window_event()` em módulo
bounded.

**Arquivos novos:**
- `shells/desktop/src/input_dispatch.rs` — uma função por arm:
  - `pub fn on_close_request(app: &mut App, event_loop: &ActiveEventLoop)`
  - `pub fn on_resized(app: &mut App, size: PhysicalSize<u32>)`
  - `pub fn on_hovered_file(app: &mut App, path: PathBuf)`
  - `pub fn on_dropped_file(app: &mut App, path: PathBuf)`
  - `pub fn on_scale_factor_changed(app: &mut App, scale: f64)`
  - `pub fn on_modifiers_changed(app: &mut App, mods: ModifiersState)`
  - `pub fn on_ime_commit(app: &mut App, text: String)`
  - `pub fn on_cursor_moved(app: &mut App, position: PhysicalPosition<f64>)`
  - `pub fn on_mouse_wheel(app: &mut App, delta: MouseScrollDelta)`
  - `pub fn on_mouse_input(app: &mut App, state: ElementState, button: MouseButton)`
  - `pub fn on_keyboard_input(app: &mut App, event: WinitKeyEvent, is_synthetic: bool)`
  - `pub fn on_redraw_requested(app: &mut App)`

**Arquivos editados:**
- `shells/desktop/src/main.rs`:
  - `window_event` reduzido a match de uma linha por arm chamando
    `input_dispatch::on_*`.

**Critério de aceite:**
- Comportamento idêntico (todos eventos roteados).
- `wc -l shells/desktop/src/main.rs` < 2000 (target).
- HR-3 bench verde.

**Risco:** médio. Same playbook que PR 9a.

**Cita HR:** anti-pattern §15.

---

### PR 9c — Decompor `resumed()` em `init.rs`

**Objetivo:** `resumed()` perde 260 LOC.

**Arquivos novos:**
- `shells/desktop/src/init.rs`:
  - `pub fn init_gpu(app: &mut App, event_loop: &ActiveEventLoop) -> Gfx`
  - `pub fn init_atlas(gfx: &mut Gfx)`
  - `pub fn init_script(app: &mut App)`
  - `pub fn init_hero(app: &mut App, gfx: &mut Gfx)`
  - `pub fn init_mcp(app: &mut App)`
  - `pub fn init_registry(app: &mut App)` — chama `register_all`
  - `pub fn resume(app: &mut App, event_loop: &ActiveEventLoop)` —
    orquestrador

**Arquivos editados:**
- `shells/desktop/src/main.rs`:
  - `resumed(&mut self, el)` reduzido a `init::resume(self, el)`.

**Critério de aceite:**
- App inicializa idêntico — golden image inicial preservado.
- `wc -l shells/desktop/src/main.rs` < 800 (target intermediário).

**Cita HR:** anti-pattern §15.

---

### PR 10 — Cleanup + SKILL update + HR-18 formalização

**Objetivo:** purgar código obsoleto, atualizar canônicos.

**Arquivos editados:**
- `crates/ph2d-editor/src/icons.rs`: remover variants do `enum IconId`
  que não têm consumer em chrome fixo (todos os ícones de tool).
- `crates/ph2d-editor/src/screens/hero/ids.rs`: remover consts de
  NodeId que ficaram só em chrome fixo (TopBar/LeftRail derivados ganham
  IDs via hash).
- `crates/ph2d-editor/src/tools/mod.rs`: remove `pub mod` para tools
  migradas; mantém só re-exports de tipos canônicos
  (`Bounds`, `MakeSquareResult`, etc. — esses ficam como facade
  thin redirecionando para `ph2d_tool_*::*`).
- `crates/ph2d-editor/src/lib.rs`: encolhe re-exports (mata facades de
  tools que não precisam mais ser visíveis no editor crate).
- `shells/desktop/src/main.rs`: remove `pending_*` fields restantes
  que já foram absorvidos.

**Arquivos novos:**
- `tests/architecture/file_loc_caps.rs` — implementa HR-18 (vide §6
  abaixo).
- `docs/architecture/decisions/0025-convention-by-discovery.md` —
  ADR registrando a migração. Status `Accepted`.

**Atualizações canônicas (SKILL_Stack_PH2D_Definitiva.md):**
- §5: nenhuma mudança de versão.
- §6: diagrama de arquitetura atualizado com `ph2d-tool-*` crates.
- §7: layout do repositório atualizado.
- §9: adicionar **HR-18** após HR-17.
- §11.9 (Editor UI): nota sobre Registry como source-of-truth pra TopBar/
  LeftRail.
- §14: "Adicionar uma tool" — nova receita (criar `crates/ph2d-tool-X/`,
  declarar MANIFEST, anexar linha em `registry_init.rs`).
- §15: anti-patterns — registrar god-file como anti-pattern.
- §19: adicionar ADR-0025 à tabela.

**Atualizações operacionais:**
- `docs/IntegracaoMultiAgente/03-Agente-Periferico.md`: §6 "Decida
  pasta exclusiva" — receita nova é "crie crate `ph2d-tool-<slug>/`".
- `docs/IntegracaoMultiAgente/02-Coordenador.md`: nova lista do que
  Coordenador edita por integração (1 linha em `registry_init.rs` +
  1 dep em `ph2d-editor/Cargo.toml`).

**Critério de aceite:**
- `cargo test --workspace` verde (1098+ novos).
- `wc -l shells/desktop/src/main.rs` < 400 (HR-18 cap).
- HR-18 CI gate verde.
- SKILL atualizado, ADR-0025 mergeado.

**Risco:** baixo. É cleanup; comportamento já validado por PRs anteriores.

---

## 6. Hard Rule HR-18 — Crescimento bounded em shell binaries

### 6.1 Texto canônico (a ser inserido em SKILL §9, após HR-17)

```markdown
### HR-18 — Crescimento bounded em shell binaries
**Rule:** Arquivos em `shells/<plataforma>/src/` respeitam caps de
tamanho:
- Qualquer arquivo `.rs`: **≤ 600 LOC** (excluindo `tests/` e arquivos
  declarados como tabelas em comentário `// ph2d-loc-cap: table`).
- Qualquer função: **≤ 200 LOC** (corpo entre `{` e `}` do top-level
  fn).
- `main.rs` de qualquer shell: **≤ 400 LOC** — contém apenas struct
  App, impl ApplicationHandler, fn main, e tests inline.

Crescimento de funcionalidade acontece por adição de módulo `mod X;`
(arquivo novo abaixo do cap), nunca por inflação de função ou arquivo
existente.

**Rationale:** god-files são hostis a multi-agente (superfície de
conflito), a LLM (excesso de contexto por janela), e a auditoria
(complexidade ciclomática inauditável). Bound estrito força
decomposição contínua por responsabilidade.

**Enforced by:** `tests/architecture/file_loc_caps.rs` em CI. Falha
se qualquer arquivo/função excede o cap. Exceções por
`// ph2d-loc-cap: <razão>` no topo do arquivo (uso raro, requer
justificativa em PR).
```

### 6.2 Implementação do CI gate

`tests/architecture/file_loc_caps.rs`:
- Walk `shells/*/src/**/*.rs`.
- Conta linhas de cada arquivo (exclui linhas só de whitespace/comment
  se quiser ser estrito; v1 conta todas).
- Detecta cada `fn` top-level + linhas até fechamento de bloco.
- Falha com mensagem específica: `"shells/desktop/src/main.rs: 412 LOC
  excede HR-18 cap (400). Decompor em módulo novo."`.

Edge cases:
- Tabelas declarativas grandes (ex.: keymap, theme tokens) podem usar
  exception comment `// ph2d-loc-cap: table` na primeira linha. Lint
  permite.
- Generated code (build.rs output) não está em `shells/*/src/` por
  convenção (vive em `OUT_DIR`), não afetado.

### 6.3 Caps confirmados (decisão do Enio, 2026-05-16)

- **600 LOC** por arquivo (frouxo — permite módulos de domínio médio).
- **200 LOC** por função (frouxo — permite handlers complexos como
  `on_mouse_input` se for 1 fn coesa).
- **400 LOC** para `main.rs` (frouxo — permite struct App + dispatch
  + tests sem forçar split).

Esses caps são propositalmente menos apertados que o ideal pessoal de
LLMs, para reduzir falsos positivos em CI. Reapertar é mudança de
configuração trivial; afrouxar nunca foi pedido.

---

## 7. Coordenação com STATE.md atual

### 7.1 Slots ativos durante a migração

Hoje (2026-05-16):

| Slot | Slug | Status | Tratamento |
|---|---|---|---|
| 1 | grid-snap | working | NÃO TOCAR. Migra em PR 6 pós-`done`. |
| 2 | bgremoval | working | NÃO TOCAR. Migra em PR 7 pós-`done`. |
| 3 | make-square | done | Migra em PR 4 (alvo seguro). |
| 4 | vago | — | Reservado para PR 5 (validação operacional). |

### 7.2 Janelas de execução

- **Janela A (PRs 1-3):** Coordenador trabalha em isolamento. Sem
  interferência em slots Periféricos (foundation, helpers, CI lint só
  tocam arquivos novos ou CI).
- **Janela B (PR 4 + PR 5):** Coordenador migra make-square + observa
  slot 4. Periféricos 1 e 2 podem continuar trabalhando.
- **Janela C (PRs 6 e 7):** Migrações executadas serial, **somente
  após** slot correspondente reportar `done`. Pode levar semanas — OK,
  plano não tem prazo apertado.
- **Janela D (PRs 8, 9a, 9, 9b, 9c, 10):** Coordenador trabalha em
  isolamento; novos slots Periféricos já usam shape novo.

### 7.3 Comunicação durante a migração

- STATE.md ganha seção nova **"Plano migração ativo"** com link para
  este documento + status da fase atual.
- Periféricos novos (a partir de janela B) recebem briefing com
  instrução: "use shape `crates/ph2d-tool-<slug>/`; consulte
  `docs/Migracao/2026-05-convention-by-discovery.md` apêndice A".
- Coordenador faz commit de progresso após cada PR mergeado:
  `chore(coordenador): plano migração — PR N/13 done`.

### 7.4 Critério de pausa

Coordenador pausa migração imediatamente se:
- Algum PR de migração quebrar smoke visual.
- HR-18 lint causar falsos positivos > 5 (revisar caps).
- Periférico reportar bloqueio causado pelo shape novo (ex.: API faltando).

Pausa = registrar em STATE.md, escalar para Enio, não prosseguir até
decisão.

---

## 8. Critérios de aceite globais (Definition of Done do plano)

Plano todo está `done` quando:

- [ ] 13 PRs mergeados em `main` local.
- [ ] `cargo test --workspace` verde (1098+ testes novos).
- [ ] `cargo clippy --workspace -- -D warnings` verde.
- [ ] `cargo fmt --check` verde.
- [ ] HR-18 CI gate verde.
- [ ] Todos os 3 jobs de CI lint stack (HR-13/HR-15/HR-7) verdes.
- [ ] `wc -l shells/desktop/src/main.rs` < 400.
- [ ] `wc -l crates/ph2d-editor/src/icons.rs` < 600 (cap explícito;
      hoje é o maior risco).
- [ ] Smoke visual: PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop
      abre editor, todas tools migradas funcionam, Inspector edita,
      Hierarchy reparenta, Trim/MakeSquare/Reimport funcionam.
- [ ] ADR-0025 mergeado e linkado em SKILL §19.
- [ ] SKILL §5/§6/§7/§9/§11.9/§14/§15/§19 atualizados.
- [ ] `docs/IntegracaoMultiAgente/{02,03}.md` atualizados com receita
      nova.
- [ ] Próximo Periférico chegando consegue integrar tool nova em ≤30min
      (incluindo Coordenador wiring).

---

## 9. Riscos, mitigação, rollback

### 9.1 Risco: PR de migração quebra comportamento sutil

**Probabilidade:** média (especialmente PR 6, PR 7, PR 9a — movimentação
grande de código).

**Mitigação:**
1. Cada migração mantém código antigo em sub-folder como facade thin
   por 1 PR antes de purgar.
2. Smoke visual obrigatório antes de `done`.
3. Golden image tests (já existem em `tests/golden/`) cobrem render
   visual.
4. Periférico/Enio validam manualmente cada smoke antes de marcar PR.

**Rollback:** `git revert <sha>`. Cada PR é commit limpo e reversível.
PR 9a é split em commits granulares (1 dreno por commit) — rollback
fino possível.

### 9.2 Risco: HR-18 lint causa falsos positivos massivos no PR 10

**Probabilidade:** baixa (caps são frouxos: 600/200/400).

**Mitigação:**
1. PR 10 inclui exceções `// ph2d-loc-cap: table` onde justificado
   (keymap, theme).
2. Caps revisitados se >5 exceções aparecerem.

**Rollback:** caps são configuração no test; ajuste é 1 linha.

### 9.3 Risco: deps específicas de tool conflitam entre crates

**Probabilidade:** baixa (cargo lida bem com versões transitivas).

**Mitigação:** `cargo-deny` continua catching conflitos. PR 7
(bgremoval) é o primeiro caso real; resolve no momento.

### 9.4 Risco: shadow mode causa confusão durante janela B-C

**Probabilidade:** média (sistema temporariamente híbrido).

**Mitigação:**
1. STATE.md documenta explicitamente quais tools estão no shape novo
   vs antigo.
2. Documento `docs/Migracao/STATUS.md` (a criar no PR 1) lista status
   por tool.
3. Periféricos novos só usam shape novo (sem ambiguidade para eles).

### 9.5 Risco: Periféricos em curso resistirem ao plano

**Probabilidade:** baixa (LLMs leem briefing; protocolo respeita
"não tocar slot working").

**Mitigação:** este documento é canônico; conflitos resolvem-se por
SKILL §18 + ADR-0025 (a criar).

---

## 10. Atualizações canônicas a serem feitas

### 10.1 SKILL_Stack_PH2D_Definitiva.md

Versão sobe para **2.4** ao final do PR 10. Mudanças:

- §1.4 versão: `Versão deste documento: 2.4 — 2026-05-<dia> (migração
  convention-by-discovery + HR-18 + shells decompostas)`.
- §5: nenhuma versão de crate muda; adicionar nota sobre tool-crates
  serem optional deps gateadas por feature `editor`.
- §6: diagrama atualizado mostrando `ph2d-tool-*` como ilhas.
- §7: layout atualizado com novos crates + shells/desktop módulos.
- §9: HR-18 inserida após HR-17.
- §11.9: §11.9 "Editor UI" ganha parágrafo sobre Registry como source
  of truth para chrome (TopBar/LeftRail).
- §14: receita "Adicionar uma tool" reescrita — criar
  `crates/ph2d-tool-X/`, escrever MANIFEST, anexar linha em
  `registry_init.rs`. INTEGRATION.md antigo morre.
- §15: novo anti-pattern "God-file em shells/" + "Manual NodeId range
  allocation".
- §19: tabela ADR ganha **ADR-0025 — Convention-by-discovery + HR-18
  (Accepted)**.

### 10.2 ADR-0025 (a criar no PR 10)

```markdown
# ADR-0025: Convention-by-discovery + Shell decomposition + HR-18

**Status:** Accepted
**Data:** 2026-05-<dia>
**Decisor(es):** Enio + LLM Coordenador + 2 pareceres independentes

## Contexto
Convention-by-edit em registries centrais (lib.rs, tools/mod.rs,
icons.rs, fixture.rs, ids.rs) serializa multi-agente no Coordenador e
infla shells/desktop/src/main.rs a 3463 LOC. Hostil a 4+ Periféricos
paralelos e a janelas de contexto LLM.

## Decisão
1. Cada tool vira crate isolado `crates/ph2d-tool-<slug>/`.
2. Auto-registration via `ToolManifest` + `register_all()` em arquivo
   append-only `crates/ph2d-editor/src/tools/registry_init.rs`. Linkme/
   inventory rejeitados por fragilidade cross-platform (wasm32, iOS).
3. Chrome (TopBar/LeftRail/icons) derivado puramente do Registry.
4. `shells/desktop/src/main.rs` decomposto em
   `render_loop.rs`/`hero_intents.rs`/`input_dispatch.rs`/`init.rs`/
   `tool_actions.rs`.
5. HR-18 formaliza caps 600/200/400 com CI gate.

## Consequências
Positivas: 4 Periféricos paralelos sem colisão; Coordenador deixa de
ser gargalo; main.rs growth-bounded; INTEGRATION.md mecânico extingue.
Negativas: workspace cresce em N crates (favorável a paralelismo de
compile, neutro em CI); migração de 13 PRs precisa coordenação;
shadow mode temporário durante janelas B-C.
Neutras: dispatch dinâmico em click path (não hot — HR-3 preservado
via dhat bench).

## Alternativas consideradas
- `linkme` distributed_slice: rejeitado (wasm32 + iOS bitcode +
  MSVC LTO fragility, ordem não-determinística cross-linker).
- build.rs codegen: viável mas perde transparência para LLM (§18#8
  do SKILL).
- `bevy_ecs::App::add_plugins` piggyback: rejeitado (força ECS plugin
  pra papel de UI registry, viola separação ADR-0021).
- Pasta-por-tool dentro de `crates/ph2d-editor/src/tools/`: parcial,
  mantém Cargo.toml + icons.rs como pontos de colisão.

## Referências
- `docs/Migracao/2026-05-convention-by-discovery.md` (este plano).
- 2 pareceres independentes registrados no histórico STATE.md.
```

### 10.3 Diretrizes Multi-Agente

`docs/IntegracaoMultiAgente/03-Agente-Periferico.md` (versão 1.1):
- §6: "Decida pasta exclusiva" — receita primária é
  `crates/ph2d-tool-<slug>/` (crate inteiro). Subpasta em
  `crates/ph2d-editor/src/tools/<slug>/` permanece como exceção para
  casos triviais.
- §7.2: lista de "NÃO pode tocar" perde
  `crates/ph2d-editor/src/{tools/mod.rs, icons.rs, widget.rs}`
  parcialmente — apenas para tools migrando. Mantém para chrome fixo.
- §12: receita de relatório "pronto" encolhe — não precisa mais listar
  "wiring pendente" item por item; MANIFEST cobre tudo.

`docs/IntegracaoMultiAgente/02-Coordenador.md` (versão 1.1):
- §X: receita de integração nova — `+ ph2d_tool_<slug>::register(reg);`
  em registry_init.rs + `ph2d-tool-<slug> = { path, optional }` em
  ph2d-editor/Cargo.toml. **2 edições por integração**, deletando
  receita antiga de 6-8 edições.

---

## Apêndice A — Shape canônico de tool-crate

Periférico criando feature nova segue este template **literalmente**:

```
crates/ph2d-tool-<slug>/
├── Cargo.toml
├── README.md                    (opcional, descreve o tool)
└── src/
    ├── lib.rs                   (pub MANIFEST + pub fn register)
    ├── manifest.rs              (definição do const MANIFEST)
    ├── algorithm.rs             (lógica pura, testável sem editor)
    ├── icon.rs                  (fn icon() -> BezPath)
    ├── handler.rs               (fn(s) handler — OneShot ou Stateful)
    └── panel.rs                 (apenas se Stateful — build_panel)
└── tests/
    ├── algorithm.rs             (testes unitários puros)
    └── manifest.rs              (smoke: MANIFEST válido)
```

### A.1 `Cargo.toml` mínimo

```toml
[package]
name = "ph2d-tool-<slug>"
version = "0.1.0"
edition = "2024"
rust-version = "1.92"

[lib]
path = "src/lib.rs"

[dependencies]
# Deps específicas DESTE tool, NUNCA poluem ph2d-editor.
ph2d-editor = { path = "../ph2d-editor" }
ph2d-a11y   = { path = "../ph2d-a11y" }
kurbo       = { workspace = true }

# Adicione aqui APENAS o que esta tool precisa.
# Ex.: image, rayon, imageproc — restritos ao escopo da tool.
```

### A.2 `src/lib.rs` mínimo

```rust
#![forbid(unsafe_code)]
//! ph2d-tool-<slug> — <descrição curta>.

mod algorithm;
mod handler;
mod icon;
mod manifest;
#[cfg(<feature = "stateful">)]
mod panel;

pub use algorithm::*;
pub use manifest::MANIFEST;

pub fn register(reg: &mut ph2d_editor::registry::Registry) {
    reg.register(&MANIFEST);
}
```

### A.3 `src/manifest.rs` exemplo (Action one-shot)

```rust
use ph2d_editor::registry::{
    ToolManifest, ToolHandler, McpExposure, Zone, Role, MemoryBudget,
};

pub const MANIFEST: ToolManifest = ToolManifest {
    id: "make_square",
    label_key: "tool.make_square.label",
    icon_fn: crate::icon::icon,
    zone: Zone::TopBar,
    cluster: "image_tools",
    order: 50,
    a11y_role: Role::Button,
    handler: ToolHandler::OneShot(crate::handler::on_clicked),
    memory_budget: MemoryBudget {
        vram_mb: 0,
        ram_mb: 0,    // pure-CPU, dimensão depende de imagem em runtime
        heap_script_mb: 0,
    },
    touches_sim: false,  // só PresentWorld (sprite swap)
    mcp: McpExposure {
        exposed: false,        // reservado, não wireado nesta migração
        destructive: false,
        handle_only: true,
    },
};
```

---

## Apêndice B — Shape de `render_loop.rs`

```rust
//! Render frame orchestrator. Each step is one call into a bounded
//! module. This file stays under 250 LOC indefinitely.

use crate::App;

pub fn render_frame(app: &mut App) {
    // 1. Bookkeeping (frame counter, EWMA dt).
    tick_frame_counter(app);

    // 2. Drain pending lifecycle (resize, file drop).
    drain_lifecycle(app);

    // 3. Drain Inspector intents (13 functions, bounded).
    crate::hero_intents::drain_all(app);

    // 4. Drain tool actions via Registry (O(1) in main.rs growth).
    crate::tool_actions::drain(app, &app.registry);

    // 5. ECS step (sim → present extract, ADR-0021).
    extract_to_present(app);

    // 6. Paint (chrome + canvas + overlay).
    paint(app);

    // 7. Present (ADR-0020 acquire_frame).
    present(app);
}

fn tick_frame_counter(app: &mut App) { /* ~20 LOC */ }
fn drain_lifecycle(app: &mut App) { /* ~30 LOC: pending_resize + pending_drops */ }
fn extract_to_present(app: &mut App) { /* ~40 LOC */ }
fn paint(app: &mut App) { /* ~80 LOC */ }
fn present(app: &mut App) { /* ~30 LOC */ }
```

---

## Apêndice C — Shape de `hero_intents.rs`

```rust
//! Inspector intent drains. ONE function per intent kind. Bounded by
//! the kinds of operations the Inspector exposes (~13 today, stable).
//! Adding a new intent kind = adding a new function + 1 line in
//! drain_all. Never grows existing functions.

use crate::App;

pub fn drain_view_focus(app: &mut App) { /* ~40 LOC */ }
pub fn drain_visibility_toggle(app: &mut App) { /* ~30 LOC */ }
pub fn drain_reparent(app: &mut App) { /* ~120 LOC — pode precisar split */ }
pub fn drain_duplicate(app: &mut App) { /* ~40 LOC */ }
pub fn drain_add_child(app: &mut App) { /* ~30 LOC */ }
pub fn drain_reset_transform(app: &mut App) { /* ~20 LOC */ }
pub fn drain_delete(app: &mut App) { /* ~50 LOC */ }
pub fn drain_hierarchy_row_click(app: &mut App) { /* ~30 LOC */ }
pub fn drain_rename_seed(app: &mut App) { /* ~30 LOC */ }
pub fn drain_rename_commit(app: &mut App) { /* ~50 LOC */ }
pub fn drain_transform_edit(app: &mut App) { /* ~80 LOC */ }
pub fn drain_visibility_edit(app: &mut App) { /* ~40 LOC */ }
pub fn drain_name_edit(app: &mut App) { /* ~50 LOC */ }

pub fn drain_all(app: &mut App) {
    drain_view_focus(app);
    drain_visibility_toggle(app);
    drain_reparent(app);
    drain_duplicate(app);
    drain_add_child(app);
    drain_reset_transform(app);
    drain_delete(app);
    drain_hierarchy_row_click(app);
    drain_rename_seed(app);
    drain_rename_commit(app);
    drain_transform_edit(app);
    drain_visibility_edit(app);
    drain_name_edit(app);
}
```

Se `drain_reparent` ou similar exceder cap de 200 LOC/função, split
local: `drain_reparent` chama `reparent_with_before_node`,
`reparent_with_after_node`, `reparent_root`. HR-18 enforced.

---

## Apêndice D — Shape de `input_dispatch.rs`

```rust
//! Window event arms decomposed. ONE function per WindowEvent variant.
//! Bounded by winit's API (~13 arms, stable).

use crate::App;
use winit::event::*;

pub fn on_close_request(app: &mut App, el: &winit::event_loop::ActiveEventLoop) { /* */ }
pub fn on_resized(app: &mut App, size: winit::dpi::PhysicalSize<u32>) { /* */ }
pub fn on_hovered_file(app: &mut App, path: std::path::PathBuf) { /* */ }
pub fn on_hovered_file_cancelled(app: &mut App) { /* */ }
pub fn on_dropped_file(app: &mut App, path: std::path::PathBuf) { /* */ }
pub fn on_scale_factor_changed(app: &mut App, scale: f64) { /* */ }
pub fn on_modifiers_changed(app: &mut App, mods: ModifiersState) { /* */ }
pub fn on_ime_commit(app: &mut App, text: String) { /* */ }
pub fn on_cursor_moved(app: &mut App, pos: winit::dpi::PhysicalPosition<f64>) { /* */ }
pub fn on_mouse_wheel(app: &mut App, delta: MouseScrollDelta) { /* */ }
pub fn on_mouse_input(app: &mut App, state: ElementState, button: MouseButton) {
    // Se esta função passar de 200 LOC: split em on_mouse_press,
    // on_mouse_release, e helpers privados. HR-18 enforced.
}
pub fn on_keyboard_input(app: &mut App, event: KeyEvent, is_synthetic: bool) {
    // Idem.
}
pub fn on_redraw_requested(app: &mut App) {
    crate::render_loop::render_frame(app);
}
```

`main.rs` reduz `window_event` a:

```rust
fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, ev: WindowEvent) {
    match ev {
        WindowEvent::CloseRequested            => input_dispatch::on_close_request(self, el),
        WindowEvent::Resized(size)             => input_dispatch::on_resized(self, size),
        WindowEvent::HoveredFile(p)            => input_dispatch::on_hovered_file(self, p),
        WindowEvent::HoveredFileCancelled      => input_dispatch::on_hovered_file_cancelled(self),
        WindowEvent::DroppedFile(p)            => input_dispatch::on_dropped_file(self, p),
        WindowEvent::ScaleFactorChanged{ scale_factor, .. } => input_dispatch::on_scale_factor_changed(self, scale_factor),
        WindowEvent::ModifiersChanged(m)       => input_dispatch::on_modifiers_changed(self, m.state()),
        WindowEvent::Ime(Ime::Commit(t))       => input_dispatch::on_ime_commit(self, t),
        WindowEvent::CursorMoved{ position, .. }=> input_dispatch::on_cursor_moved(self, position),
        WindowEvent::MouseWheel{ delta, .. }   => input_dispatch::on_mouse_wheel(self, delta),
        WindowEvent::MouseInput{ state, button, .. } => input_dispatch::on_mouse_input(self, state, button),
        WindowEvent::KeyboardInput{ event, is_synthetic, .. } => input_dispatch::on_keyboard_input(self, event, is_synthetic),
        WindowEvent::RedrawRequested           => input_dispatch::on_redraw_requested(self),
        _ => {}
    }
}
```

13 linhas + fechamento. Stable forever.

---

## Apêndice E — Shape de `init.rs`

```rust
//! Subsystem initialization. ONE function per subsystem boot.

use crate::App;
use winit::event_loop::ActiveEventLoop;

pub fn resume(app: &mut App, el: &ActiveEventLoop) {
    init_registry(app);            // ~10 LOC
    let mut gfx = init_gpu(app, el);  // ~80 LOC
    init_atlas(&mut gfx);          // ~30 LOC
    init_hero(app, &mut gfx);      // ~40 LOC
    init_script(app);              // ~30 LOC
    init_mcp(app);                 // ~20 LOC
    app.gfx = Some(gfx);
}

fn init_registry(app: &mut App) {
    let mut reg = ph2d_editor::registry::Registry::default();
    ph2d_editor::tools::registry_init::register_all(&mut reg);
    reg.build().expect("registry build failed (collisions or budget overrun)");
    app.registry = reg;
}

fn init_gpu(app: &mut App, el: &ActiveEventLoop) -> Gfx { /* ~80 LOC */ }
fn init_atlas(gfx: &mut Gfx) { /* ~30 LOC */ }
fn init_hero(app: &mut App, gfx: &mut Gfx) { /* ~40 LOC */ }
fn init_script(app: &mut App) { /* ~30 LOC */ }
fn init_mcp(app: &mut App) { /* ~20 LOC */ }
```

---

## Apêndice F — Shape de `tool_actions.rs`

```rust
//! Generic action drain. Reads ActionInvocation queue, dispatches via
//! Registry. Size is O(1) in number of tools — never grows.

use crate::App;
use ph2d_editor::registry::Registry;

pub fn drain(app: &mut App, registry: &Registry) {
    while let Some(invocation) = app.actions.pop_front() {
        match registry.handler_for(&invocation.action_id) {
            Some(handler) => handler(&mut build_ctx(app, &invocation)),
            None => tracing::warn!(action = %invocation.action_id, "no handler"),
        }
    }
    app.action_arena.reset();  // HR-3 — payload arena resets per-frame.
}

fn build_ctx<'a>(app: &'a mut App, inv: &'a ActionInvocation) -> ToolCtx<'a> {
    ToolCtx {
        sim: &mut app.sim,
        present: &mut app.present,
        history: &mut app.history,
        asset_db: &app.asset_db,
        payload: inv.payload,
    }
}
```

Tamanho: ~30 LOC. Cresce zero quando tools migram — só o conteúdo dos
handlers (em cada tool-crate) cresce.

---

## Apêndice G — Shape de `registry_init.rs` (estado pós PR 10)

```rust
//! APPEND-ONLY. Single point of contact for tool registration.
//!
//! Adding a new tool: add ONE line below, in alphabetical order.
//! Removing a tool: remove its line (and its dep in Cargo.toml).
//!
//! Coordenador edits this file. Periféricos NEVER touch it.

use ph2d_editor::registry::Registry;

pub fn register_all(reg: &mut Registry) {
    ph2d_tool_bgremoval::register(reg);
    ph2d_tool_brush::register(reg);
    ph2d_tool_grid_snap::register(reg);
    ph2d_tool_make_square::register(reg);
    ph2d_tool_move::register(reg);
    ph2d_tool_trim_transparency::register(reg);
    // Adicione tools em ordem alfabética. Merge 3-way trivial.
}
```

Tamanho alvo: ~30 LOC para os primeiros 20 tools, ~80 LOC para 50+.
Crescimento linear mas trivial — uma linha por tool, ordem alfabética,
zero lógica. Conflito de merge: ~impossível.

---

## Apêndice H — HR-18 lint test skeleton

`tests/architecture/file_loc_caps.rs`:

```rust
//! HR-18 enforcement: caps de LOC em shells/.

use std::path::Path;

const FILE_CAP: usize = 600;
const FN_CAP: usize = 200;
const MAIN_RS_CAP: usize = 400;

#[test]
fn shells_respect_loc_caps() {
    let mut errors = Vec::new();
    for entry in walkdir::WalkDir::new("shells")
        .into_iter().filter_map(Result::ok)
        .filter(|e| e.path().extension().map(|x| x == "rs").unwrap_or(false))
        .filter(|e| !e.path().to_string_lossy().contains("/tests/"))
    {
        let path = entry.path();
        let content = std::fs::read_to_string(path).unwrap();

        if has_exception(&content) {
            continue;
        }

        let loc = content.lines().count();
        let cap = if path.file_name().unwrap() == "main.rs" {
            MAIN_RS_CAP
        } else {
            FILE_CAP
        };
        if loc > cap {
            errors.push(format!(
                "{}: {} LOC excede HR-18 cap ({}). Decompor.",
                path.display(), loc, cap
            ));
        }

        // Detecta fn top-level e mede tamanho.
        for (fn_name, fn_loc) in find_top_level_fns(&content) {
            if fn_loc > FN_CAP {
                errors.push(format!(
                    "{}::{}: {} LOC excede HR-18 fn cap ({}). Split.",
                    path.display(), fn_name, fn_loc, FN_CAP
                ));
            }
        }
    }
    assert!(errors.is_empty(), "HR-18 violations:\n{}", errors.join("\n"));
}

fn has_exception(content: &str) -> bool {
    content.lines().take(5).any(|l| l.contains("// ph2d-loc-cap:"))
}

fn find_top_level_fns(content: &str) -> Vec<(String, usize)> {
    // Implementação via regex simples — fn-body delimitada por { } balanced
    // em indent zero. Detalhe em tests/architecture/loc_helpers.rs.
    todo!()
}
```

(Implementação real do parser de fn-body fica em helper module separado;
v1 pode ser regex grosseiro; v2 usar `syn` se necessário.)

---

## Fim do plano

**Próxima ação imediata (a aguardar aprovação do Enio):**
Coordenador inicia **PR 1 — Foundation**. Antes disso, registrar
operação em STATE.md:

```
## Plano migração ativo
Documento: docs/Migracao/2026-05-convention-by-discovery.md
Fase atual: aguardando aprovação PR 1
Próxima ação: Coordenador inicia foundation (PR 1)
```
