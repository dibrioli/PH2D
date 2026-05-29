# Diretriz de Implementação — PH2D

**Versão:** 7.1 — 2026-05-28 (modelo de papéis consolidado: **1 Coordenador único + N Implementadores** — absorve os antigos Coord-A/Coord-B, após colisões git entre coordenadores/implementadores paralelos).
**Audiência:** **toda LLM que entra no projeto.** Leia inteiro antes de tocar em código.

> **Seu primeiro output sempre = TRIAGEM (§2).** Classifique a tarefa do Enio
> e diga **como proceder** antes de codar.

---

## TL;DR

- **Dois papéis:** **um Coordenador único** (absorve foundational + scaffolds + ship + arbitragem de posse) + **N Implementadores** (sempre vários, cada um numa pasta/módulo físicamente disjunto).
- **Três caminhos** (descobertos via Triagem §2):
  - **(A) Drop-crate (fan-out, §3.A)** — node ou tool nova. Implementador sozinho. Zero edit central. Paraleliza com outros (A).
  - **(B) Scaffold central (§3.B)** — painel/widget/chrome. O Coordenador faz scaffold + delega.
  - **(C) Coord-only (§3.C)** — foundational ou contrato congelado. Não paraleliza. ADR se for contrato.
- **Dois contratos congelados (§4)** com arch-gate ativo: nodes (ADR-0039) e tools (ADR-0040+0041). Mexer = (C).
- **Enio é relay mecânico**, não decisor.
- **Norte:** engine cresce por **duas famílias-irmãs** simétricas — `crates/ph2d-node-*` (declarativo, FBP) e `crates/ph2d-tool-*` (imperativo, manipulação direta). Ambas wireadas por codegen (`ph2d-{node,tool}-sync`). Adicionar conteúdo = drop-crate.

---

## 0. Antes de começar (sanity check obrigatório)

Independente do papel, **rode primeiro**:

```bash
git log --oneline -5             # confirma HEAD
git status -sb                   # working tree limpo?
cargo check --workspace 2>&1 | tail -5    # baseline compila?
```

Algo divergente (HEAD inesperado, working dirty, build quebrado) → **pare e reporte ao Enio.**

**Leitura mínima:**
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §HR-1..HR-18 (Hard Rules) e §1 (arquitetura).
- [`CLAUDE.md`](../../CLAUDE.md) (CI, push, batching).
- Memória persistente: `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`.

---

## 1. Papéis + infra multi-agente

**Coordenador (único).** Um só por jornada. Absorve o que antes eram Coord-A (foundational) e Coord-B (baldes). Autoridade **exclusiva** sobre: contratos congelados, arch-gates, foundational crates (`ph2d-render`, `ph2d-editor-core`, `ph2d-host`, `ph2d-tokens`, …), codegen tools, `shells/*` plumbing compartilhado, scaffolds de painel/widget/chrome, ADRs, `CLAUDE.md`/DIRETRIZ, `.github/workflows/`. É o **único** que toca arquivo foundational/compartilhado — isso serializa a superfície de colisão (causa-raiz dos incidentes que motivaram o modelo). Mexe nos 2 contratos congelados (§4) só via amendment ADR, nunca cap-bust ad-hoc. Responsabilidades do modelo multi-implementador:
- (a) escrever um **sub-handoff focado por implementador** (estado + pasta exclusiva + task + anti-colisão);
- (b) manter o **mapa de posse** em SESSION_ACTIVE (§1.1) — quem é dono de quê;
- (c) **arbitrar colisões** e **sequenciar dependências** entre implementadores (ex.: liberar `ph2d-render` ao módulo B só quando o módulo A soltar);
- (d) **ship-de-jornada** (ship.sh + commit + push + babysit CI — §8), incluindo limpar fmt-drift e ship-blockers cross-session no fim.

Não implementa feature de módulo — **coordena**.

**Implementador (sempre vários).** Sessão isolada, **uma por módulo físicamente disjunto** (uma crate-pasta ou um cluster de crates do mesmo módulo). Caminho **(A)**: cria pasta + roda sync + testa, sem Coordenador. Caminho **(D)**: edita dentro de pasta de módulo existente. Caminho **(B)**: recebe pasta já scaffoldada pelo Coordenador, edita **só** dentro dela. A não-colisão é garantida pela arquitetura física (glob `workspace.members` + codegen splice em marcadores) **somada** à regra de posse exclusiva arbitrada pelo Coordenador. **Precisou de QUALQUER coisa fora da sua pasta** (foundational, shell plumbing, contrato congelado, outro módulo)? **PARA e reporta ao Coordenador** — não edita, e **nunca renegocia direto com outro implementador**.

**Enio.** Humano que orquestra: abre sessões Claude Code, cola mensagens entre elas, roda smoke visual quando Coord pede. **Não decide nada operacional.**

### 1.1 Protocolo SESSION_ACTIVE (mapa de posse mantido pelo Coordenador)

[`docs/SESSION_ACTIVE.md`](../SESSION_ACTIVE.md) é o post-it vivo da orquestração. **Só o Coordenador escreve;** os implementadores **leem antes de cada burst** e não editam. O Coordenador mantém ali:

1. O **mapa de posse**: qual implementador é dono de qual pasta/módulo (escrita exclusiva) + seu slot.
2. Os **pontos compartilhados** e como estão resolvidos (ex.: crate X é escrita do Impl-N, leitura dos demais).
3. Os **itens que o Coordenador segura** (ship-blockers, foundational, sequenciamento de dependências).
4. **Pre-existing failures cross-session** a NÃO fixar (com owner identificado).

Implementador que precise tocar pasta fora da sua: **PARA e reporta ao Coordenador** — nunca renegocia direto com outro implementador. O Coordenador limpa os itens concluídos ao encerrar a jornada.

### 1.2 Isolamento físico — `scripts/slot-env.sh`

Cada sessão roda `source scripts/slot-env.sh <slot-id>` no início para isolar `CARGO_TARGET_DIR` por slot. Sem isso, dois agentes paralelos serializam no lock de `target/`. Slot IDs: `coord` + um por implementador nomeado pelo módulo (`impl-sprite`, `impl-painter`, `impl-vector`, …).

**RAM 8 GiB → máximo realista = 2-3 slots cargo-ativos simultâneos.** Com N implementadores, isso NÃO autoriza N cargos simultâneos: o Coordenador **escalona quem compila quando** (lê SESSION_ACTIVE). 4º cargo ativo causa swap thrashing.

### 1.3 Anti-colisão git — `scripts/git-stage-guard.sh`

Pre-commit roda o guard que **rejeita stage fora da pasta declarada** (env `PH2D_SLOT_FOLDER`). Coords legítimos exportam `COORD_OVERRIDE=1` na sessão pra bypass. Padroniza a disciplina §7 sem depender de memória humana.

### 1.4 As 3 obrigações do Implementador (sempre)

1. **ISOLAMENTO.** Edita **só** dentro da pasta exclusiva. Precisa algo fora? **Reporta** — não edita.
2. **UI canônica.** Toda cor/espaço/raio/tipografia/stroke passa por tokens. Zero hex, zero `f32` literal de UI (§5).
3. **Codificação rápida.** `cargo check -p <crate>` no editing burst. Sem `--workspace` em loop (§6).

Pra violar uma? **Pare e reporte.** Quase certo o Coord não fez scaffold direito.

---

## 2. TRIAGEM — seu PRIMEIRO output

Quando o Enio descreve uma tarefa, **antes de codar** responda exatamente neste formato:

```
TRIAGEM
- Tarefa: <1 linha do que o Enio pediu>
- Caminho: (A) drop-crate | (B) scaffold | (C) Coord-only
- Toca contrato congelado (nodegraph/expr OU Tool/RasterEditTool/PanelEvent)?
    <Não | Sim — exige ADR + bump de cap>
- Razão: <1-2 linhas>
- Se grande/ambíguo: <peças isoláveis vs. compartilhadas>
```

### Tabela de decisão

| Tarefa | Caminho | Razão |
|--------|---------|-------|
| **Nó novo** (domínio com avaliador existente) | **(A) §3.A** | Drop-crate `crates/ph2d-node-<dom>-<slug>/` + `cargo run -p ph2d-node-sync`. Wiring gerado. |
| **Tool nova** (any shape) | **(A) §3.A** | Drop-crate `crates/ph2d-tool-<slug>/` + `cargo run -p ph2d-tool-sync`. Sem variant novo em `EditorAction`. |
| **Modificar** nó/tool existente | **(A) §3.D** | A pasta já existe — edite dentro dela. |
| **Painel novo** (`ph2d-panel-<slug>`) | **(B) §3.B.1** | Coord plumba feature flag + `register_all_panels` ANTES. |
| **Widget primitive novo** | **(B) §3.B.2** | Coord adiciona em `widget/mod.rs` + showcase ANTES. |
| **Chrome handler novo** | **(B) §3.B.3** | Coord adiciona em `chrome/mod.rs::dispatch_all` ANTES. |
| **Avaliador novo (Wave-neck)** — Shader/Som/Gameplay | **(C)** durante neck → (A) depois | Trabalho "tipo W2" serial; abre fan-out só após o neck. Tracker em [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md). |
| **Mudar tokens / editor-core (não-contrato) / shells / arch tests** | **(C)** | Foundational, não paraleliza. |
| **Mudar contrato de nós** (porta, EvalCtx, motor) | **(C) + ADR** | Bump cap em `architecture_contract_surface.rs` + ADR estendendo 0039. |
| **Mudar contrato de tools** (método em `Tool`/`RasterEditTool`, variant em `PanelEvent`) | **(C) + ADR** | Bump cap em `architecture_tool_contract_surface.rs` + amendment de ADR-0040 §7. |

**Heurística de 1 frase:** conteúdo (nó) OU peça que manipula bitmap (tool) = **(A) drop-crate**. Chrome que renderiza tools/nós (painel/widget/chrome) = **(B) Coord scaffold**. Mudar regra do jogo (contrato congelado, foundational) = **(C) Coord-only + ADR**.

**Na dúvida A vs B:** "exige editar QUALQUER arquivo fora de UMA pasta nova?" Sim → (B). Único arquivo fora = wiring **gerado** (`ph2d-{node,tool}-sync`) → ainda **(A)**.

**Diff do sync é esperado** — não viola §1.4 ISOLAMENTO. O staleness gate em CI exige a regeneração.

---

## 3. Receitas

### 3.A Fan-out drop-crate (caminho (A)) — node OU tool

Receita simétrica única. Drop a crate, roda o sync, gates fecham. **Sem coordenação, sem edit central.**

#### 3.A.1 Mapa node ↔ tool

| Aspecto | **Node** (declarativo, pull / FBP) | **Tool** (imperativo, push) |
|---|---|---|
| Pasta exclusiva | `crates/ph2d-node-<dom>-<slug>/` | `crates/ph2d-tool-<slug>/` |
| Codegen | `cargo run -p ph2d-node-sync` | `cargo run -p ph2d-tool-sync` |
| Wiring gerado | `register_all_nodes` + deps (1 superfície) | `register_all` + `register_all_tools` + deps + 2 testes (5 superfícies) |
| Gate wiring | `cargo test -p ph2d-node-registry-init` | `cargo test -p ph2d-tool-registry-init` |
| Contrato | `NodeOp` + `NodeManifest` (`ph2d-nodegraph`) | `Tool` + opcional `RasterEditTool` + `ToolManifest` (`ph2d-editor-core` + `ph2d-tool-registry`) |
| 🔒 Cap arch-gate | `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` (ADR-0039) | `Tool=10` / `RasterEditTool=5` / `PanelEvent=4` (ADR-0040+0041) |
| Entry points | `pub fn register(reg: &mut NodeRegistry) -> Result<…>` | `pub fn register(reg: &mut Registry)` (manifest) E/OU `pub fn make() -> Box<dyn Tool>` (behavior); 3 sabores §3.A.3 |
| Vocab de canal | portas tipadas + effect + clock + params | `EditorAction::{ActivateTool, OneShotImageOp, ToolPanelEvent(PanelEvent), CancelActiveTool}` (4 genéricos — sem variant per-tool) |
| Templates | `ph2d-node-debug-const/` (Pure trivial) · `-debug-wave/` (Temporal + ph2d-expr + golden) · `-motion-{grid,clone,transform}/` (vertical Stateful-free) | `-make-square/` (sabor 1) · `-brush/` (sabor 2, `is_default=true`) · `-padding/` (sabor 3 leve) · `-bgremoval/` (sabor 3 completo) |
| Pegadinhas | `ctx.param("nome")` no eval (nunca `MANIFEST.params[..].default`); `param_as_count(v, max)` p/ alocação capada | `apply_ui_edit` = single-source-of-truth de clamps; ícone exige IconId variant alfabético em `editor-core/src/icons.rs` |

#### 3.A.2 Briefing pronto-pra-colar

Substitua `<family>` por `node`/`tool`, `<slug>` pelo seu, e (se node) `<domínio>`. Apague os blocos da família errada antes de mandar ao agente.

> **Variante 100% paste-ready** (zero placeholder, com algorithm.rs + icon.rs preenchidos): [`examples-fan-out.md`](examples-fan-out.md) instancia esse briefing fim-a-fim para `ph2d-node-shader-blur` e `ph2d-tool-grayscale`. Use a parametrizada abaixo pra flexibilidade; use os exemplos concretos quando o agente é novo e o objetivo é zero-substituição-mental.

```
═══════════════════════════════════════════════════════════════════
BRIEFING — <family>-crate · slug: <slug>  [node]  · domínio: <domínio>
═══════════════════════════════════════════════════════════════════

PASTA EXCLUSIVA:
  [node]  crates/ph2d-node-<domínio>-<slug>/
  [tool]  crates/ph2d-tool-<slug>/
Glob workspace.members cobre — NÃO edite Cargo.toml raiz.

ANTES DE CODAR: leia DIRETRIZ §3.A.1 (mapa) + copie o template do seu
sabor (vide §3.A.3 pra tool).

O QUE VOCÊ FAZ (só dentro da sua pasta):
0. **`src/lib.rs` PRIMEIRO** (mesmo com 1 linha — `#![forbid(unsafe_code)]`).
   Cargo recusa o manifest enquanto crate-novo não tem lib.rs; como o
   workspace usa glob `crates/*`, TODAS as outras sessões paralelas
   ficam bloqueadas com `can't find library X` até esse arquivo existir.
   Regra: lib.rs primeiro, depois Cargo.toml, depois módulos auxiliares.
1. Cargo.toml: deps mínimas.
   [node]  ph2d-nodegraph, ph2d-node-registry, ph2d-expr se usar math
           por-elemento.
   [tool]  ph2d-tool-registry, ph2d-editor-core (Tool / FloatingPanel
           se stateful), ph2d-a11y, ph2d-core, ph2d-vector p/ ícone.
2. src/lib.rs: implemente o contrato.
   [node]  pub const MANIFEST: NodeManifest { id (NodeTypeId::of(
           "<dom>.<slug>")), name, inputs/outputs, effect (Pure|
           Temporal|Stateful), clock, params, lowerings };
           impl NodeOp { manifest(); eval(ctx) — lê params via
           ctx.param("nome"); cape via param_as_count(v, max) se aloca };
           pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError>
   [tool]  Escolha o sabor (§3.A.3) e siga o template.
3. [node] Golden test: source→seu-nó, register, g.validate(&ops), cook,
         asserta saída.
   [tool] Tests: register attaches manifest / make builds / panel layout /
         handle_panel_event clamping.
4. ÍCONE (só tool — node não tem pill).
   [tool] src/icon.rs com BezPath (porte docs/design/icons/<slug>.svg,
         Lucide 24×24, stroke="currentColor"). Adicione IconId variant
         em ph2d-editor-core/src/icons.rs em ORDEM ALFABÉTICA — gate
         enum_order_matches_svgs falha se sair de ordem; NUNCA pule
         via --no-verify (quebra TODOS os ícones).

O QUE VOCÊ NÃO TOCA:
- Qualquer arquivo fora da sua pasta.
- 🔒 Contrato congelado (vide §4). Mudança = Coord-only + ADR.
  [node]  ph2d-nodegraph, ph2d-expr, ph2d-node-registry,
          ph2d-node-registry-init/ (GERADO).
  [tool]  editor-core/src/tool.rs (Tool/RasterEditTool/PanelEvent),
          action_bus.rs::EditorAction (use os 4 genéricos),
          ph2d-tool-registry, ph2d-tool-registry-init/ (GERADO),
          resto de editor-core (foundational).
- Cargo.toml raiz.

WIRING (sem colisão, sem edit central):
  cargo run -p ph2d-<family>-sync          # regenera tudo
  cargo test -p ph2d-<family>-registry-init  # staleness fecha

VALIDAÇÃO (§6):
  cargo check  -p ph2d-<family>-<slug>
  cargo test   -p ph2d-<family>-<slug>
  cargo clippy -p ... --all-targets -- -D warnings
  cargo fmt -p ...

NOMES (gates ativos):
  [node]  type name canônico = "<dom>.<slug>", único cross-crate
          (colisão pega em RegistryError::Collision).
  [tool]  manifest id = "<slug>" único; label_key = "tool.<slug>.label".

SE PRECISAR ALGO FORA (dep externa, contrato congelado, EditorAction
variant, domínio novo): PARE e reporte ao Enio. Provavelmente não era
fan-out puro — revise triagem §2.

QUANDO TERMINAR, reporte:
  "<Family> <slug> pronto. Commit local: <sha>. cargo test -p
   ph2d-<family>-<slug> e -p ph2d-<family>-registry-init verdes."
═══════════════════════════════════════════════════════════════════
```

#### 3.A.3 Sabores de tool

| Sabor | Expõe | Templates | Quando usar |
|---|---|---|---|
| **(1) One-shot stateless** | `pub fn register` (manifest) | `-make-square/` · `-trim-transparency/` · `-real-size/` · `-rasterize/` | Pill dispara algoritmo puro no Sprite ativo. Sem `impl Tool`. Shell drena via `EditorAction::OneShotImageOp`. |
| **(2) Palette modal** | `pub fn make` (`Box<dyn Tool>`) | `-brush/` (`is_default=true`) · `-move/` | Cursor de canvas, sem pill. `impl Tool` + `build_panel` Procreate-style. Sem `ToolManifest`. |
| **(3) Stateful + panel docado** | ambos `register` E `make` | `-padding/` (leve) · `-bgremoval/` (completo) · `-color-equalization/` · `-upscale/` | Pill + panel próprio (`ph2d-panel-<slug>/`) + preview/commit raster. (1)+(2)+opcional `impl RasterEditTool`. |

O `ph2d-tool-sync` é configurado pelas needles `"pub fn register("` (manifest) e `"pub fn make("` (behavior) — sabor (1) só em `register_all`, (2) só em `register_all_tools`, (3) nos dois.

#### 3.A.4 Trait `RasterEditTool` (heads-up importante)

Sub-trait com 5 métodos (`set_source` / `current_preview` / `take_pending_commit` / `run_full` / `deactivate`), congelado em ADR-0041. **3 tools de produção implementam** (BgRemoval, Color Equalization, Upscale). Padding e Equalize Sizes são exceção documentada (geométrico-only / multi-sprite-required).

**Padrão pra tool stateful que produz raster:**

1. **No tool crate:** `impl RasterEditTool for <Tool>` com os 5 métodos. Cache via `cached_canvas_preview: Option<(Vec<u8>, u32, u32)>`. **Critical:** `set_source` e `Tool::on_deactivate` DEVEM zerar o cache (audit Wave 10 §A1+A2: pular causa stale-frame).
2. **No shell:** `shells/desktop/src/render_loop/<slug>_bridge.rs` espelhando `bgremoval_preview.rs`. Use os 4 helpers de `ph2d-tool-runtime`: `drive_source_push`, `drive_preview_cache`, `drive_pending_commit`, `drive_deactivate_cleanup`.
3. **Bits tool-specific** (panel snapshot publish, brush ring, tint overlay) seguem via `as_any_mut().downcast_mut::<ConcreteTool>()` — **exceção documentada** (ADR-0040 §3), NÃO code smell.

**Template canônico:** [`shells/desktop/src/render_loop/bgremoval_preview.rs`](../../shells/desktop/src/render_loop/bgremoval_preview.rs).

#### 3.A.5 Garantia formal de não-colisão

Dois agentes adicionando duas features (mesma família ou não) **não tocam nenhum arquivo em comum**: cada um cria sua pasta; `workspace.members` é glob; superfícies centrais são geradas determinísticamente pelo sync entre marcadores codegen, e staleness gates pegam regen-esquecida. O contrato é o único acoplamento — e está congelado pelo arch-gate (§4). **Para tool especificamente**, `editor-core` está proibida de ganhar dep em qualquer `ph2d-tool-*` concreto (`editor_core_has_no_concrete_tool_deps`) — a única edge permitida é `tool-* → editor-core`.

#### 3.A.6 Checklist do revisor

**Comum:**
- [ ] `cargo run -p ph2d-<family>-sync` rodado; staleness verde.
- [ ] arch-gate do contrato congelado verde (sem cap-bust).
- [ ] clippy `--all-targets` + fmt limpos.
- [ ] Sem dep fora do contrato.

**Node:**
- [ ] `MANIFEST` completo (params + lowerings); nome canônico `"<dom>.<slug>"` único.
- [ ] `eval` puro (sem global, sem IO); effect declarado bate; params via `ctx.param`; alocação capada via `param_as_count`.
- [ ] Golden test verde.

**Tool:**
- [ ] `MANIFEST` completo OU `is_default` correto (sabor 2: só Brush é true).
- [ ] Se stateful: `handle_panel_event` cobre 1:1 os NodeIds; rota tudo via `apply_ui_edit`.
- [ ] Se `impl RasterEditTool`: `as_raster_edit_mut` retorna `Some(self)`; cache zerado em `set_source` + `on_deactivate`.
- [ ] Ícone: SVG em `docs/design/icons/` + IconId alfabético em `icons.rs`.
- [ ] **Painel docado segue Widget Gallery (§5.2)**: `link_slider_number`, `mark_chip_no_stepper`, storage `0..1`, bridge `<slug>_bridge.rs` se altera pixels.

### 3.B Scaffold central (caminho (B)) — Coordenador faz primeiro

Painel/widget/chrome ainda exigem edit central (não codegenado). O Coordenador cria pasta + plugues centrais + stubs verdes, entrega briefing pra Implementador preencher.

#### 3.B.1 Painel novo (`ph2d-panel-<slug>`)

Coord:
1. Decide `slug`, `DEFAULT_VISIBLE`, feature flag (`panel-<slug>`).
2. Cria `crates/ph2d-panel-<slug>/` com `Cargo.toml` (deps: `ph2d-editor-core`, `ph2d-a11y`, `ph2d-tokens`, `ph2d-text`, `ph2d-vector`, `ph2d-tool-registry`).
3. Cria `src/lib.rs` com stub `impl Panel` (template completo: [`ph2d-panel-inspector`](../../crates/ph2d-panel-inspector/src/lib.rs)). **Notas factuais:** `Panel::paint` tem 2 params (`state`, `ctx`); o host fica em `ctx.host` (campo de `PaintCtx`), não param separado; trait usado pelo host é `PanelHostInternal`; `hash_node_id` vive em `ph2d-tool-registry`.
4. Em [`ph2d-panel-registry-init/Cargo.toml`](../../crates/ph2d-panel-registry-init/Cargo.toml): adiciona feature `panel-<slug> = ["dep:ph2d-panel-<slug>"]` + entrada em `[dependencies]` `{ path = "...", optional = true }` + inclui em `default = [...]`.
5. Em `ph2d-panel-registry-init/src/lib.rs::build_typed_registry`: `#[cfg(feature = "panel-<slug>")] reg.push(ErasedPanel::new::<ph2d_panel_<slug>::Panel>());` (ordem não é alfabética — sem arch-gate, mantém ordem de migração ADR-0029).
6. Atualiza `EXPECTED_TYPED` no `#[cfg(test)] mod tests` (incrementa contador).
7. `cargo check -p ph2d-panel-<slug>` + `cargo test -p ph2d-panel-registry-init` verde.
8. Commit + briefing pro Implementador (§2.B).

Implementador: preenche `paint`, `apply_event`, `populate`, `State`.

#### 3.B.2 Widget primitive novo (em `editor-core/src/widget/`)

Coord:
1. Cria `crates/ph2d-editor-core/src/widget/<slug>.rs` (template: [`button.rs`](../../crates/ph2d-editor-core/src/widget/button.rs)).
2. Em `widget/mod.rs` (ordem alfabética): `mod <slug>; pub use <slug>::{...};`.
3. Cria seção no showcase em `widget/showcase/` (copia layout de `switches.rs`). Arch test `architecture_widget_showcase_coverage` enforça.
4. `cargo check -p ph2d-editor-core` + 4 arch-tests de widget verdes: `architecture_widget_loc_cap` (≤500 LOC), `architecture_widget_showcase_coverage`, `no_literal_color`, `hr12_widgets_a11y`.

Implementador: preenche paint usando **só tokens**, adiciona tests, ajusta showcase.

#### 3.B.3 Chrome handler novo

Coord:
1. Cria `editor-core/src/screens/hero/chrome/<slug>.rs` com stub: `pub fn apply(_hero, _event) -> bool { false }`.
2. Adiciona em `chrome/mod.rs`: `pub mod <slug>;` + `|| <slug>::apply(hero, event)` em `dispatch_all` (ordem alfabética = higiene, sem arch-gate).
3. Se precisa NodeIds: `screens/hero/ids.rs` via `hash_node_id`.
4. `cargo check -p ph2d-editor-core` verde.

Implementador: preenche corpo do handler.

### 3.C Foundational + contratos congelados (caminho (C)) — Coordenador só

Foundational = `ph2d-core`, `ph2d-tokens`, `ph2d-editor-core` (exceto widget/chrome scaffold de B), `ph2d-a11y`, `ph2d-host`, `ph2d-vector`, `ph2d-text`, `ph2d-tool-registry`, `ph2d-{tool,node,panel}-registry-init`, `tools/ph2d-{node,tool}-sync`, `shells/*`, arch tests, **+ os 2 contratos congelados** (§4).

**Não paralelizável. O Coordenador faz sozinho.** Não delega.

Exemplo (adicionar `ColorToken::AccentTeal`):
1. Edita `docs/design/tokens.json` em todos 4 temas.
2. `cargo check -p ph2d-tokens` (build.rs regenera).
3. Edita `crates/ph2d-tokens/src/color.rs` adicionando variant.
4. `cargo test --workspace --exclude ph2d-asset` (paranoia).
5. Commit: `feat(tokens): add ColorToken::AccentTeal`.

### 3.D Modificar feature existente

Sem scaffold. Pasta já existe. **Caminho (A) Implementador-só** — Enio abre sessão Implementador e cola:

```
Edite crates/ph2d-<family>-<slug>/src/<arquivo>.rs. Tudo da feature
vive no crate isolado (manifest + tool + algorithm + icon + params +
panel docado em ph2d-panel-<slug>/ quando aplicável). Não toque em nada
fora. Se exigir arquivo central (Cargo.toml raiz, EditorAction,
contrato congelado, foundational): PARE e reporte — quase certo a
tarefa estava mal triada.
```

Pasta canônica por feature:

| Feature | Pasta |
|---|---|
| Tool (algo / ícone / manifest / `impl Tool` / `handle_panel_event`) | `crates/ph2d-tool-<slug>/` |
| Vocab UI de um tool (`<Slug>UiEdit`, `…UiSnapshot`, `…Params`) | `crates/ph2d-tool-<slug>/src/params.rs` |
| Panel docado de um tool | `crates/ph2d-panel-<slug>/` |
| Nó | `crates/ph2d-node-<dom>-<slug>/` |
| Painel genérico (Inspector/Hierarchy/etc.) | `crates/ph2d-panel-<slug>/` |
| Widget primitive | `crates/ph2d-editor-core/src/widget/<slug>.rs` |
| Chrome handler | `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` |

**A pasta `crates/ph2d-editor-core/src/tools/` NÃO existe** desde ADR-0040 TG-D (`c4063b7`). Memória/doc antigo apontando lá = stale.

### 3.E Cross-cutting (perf audit, refactor cross-crate, sweep de lint)

Algumas tarefas não cabem em §3.A-D porque tocam múltiplos crates por natureza. **O Coordenador autoriza explicitamente a exceção ao ISOLAMENTO** no briefing:

> "Você toca tests em vários crates conforme os achados. Exceção autorizada à regra de uma pasta isolada (DIRETRIZ §1.4). Cada commit ainda fica T1 single-crate sempre que possível."

**Regras:**
1. Cada commit valida-se sozinho (`cargo test -p <crate>` verde antes).
2. Não tocar production code de foundational sem motivo claro — em audit de tests, só `tests/` + `#[cfg(test)]`.
3. Documentar risk surface no relatório final.

---

## 4. Contratos congelados — caps + arch-gates

**Dois contratos paralelos, mesma disciplina.** Mexer é Coordenador only + ADR.

| Contrato | Arquivos | Arch-gate (cap) | ADR | Mudar exige |
|---|---|---|---|---|
| **Sistema de nós** (W2.T4, 2026-05-22) | `crates/ph2d-nodegraph/src/{lib,node,port,effect,attr,cook,graph}.rs` + `crates/ph2d-expr/src/lib.rs` | [`architecture_contract_surface`](../../crates/ph2d-nodegraph/tests/architecture_contract_surface.rs) — `NodeOp ≤ 2` métodos, `OpResolver ≤ 1` método, `NodeManifest ≤ 8` campos | [ADR-0039](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md) | Bump cap + ADR estendendo 0039 + (se `ph2d-expr`) re-provar paridade CPU↔WGSL |
| **Sistema de tools** (TG-E + ADR-0041, 2026-05-22) | `crates/ph2d-editor-core/src/tool.rs` (`Tool`, `RasterEditTool`, `PanelEvent`) + canal genérico em `crates/ph2d-editor-core/src/action_bus.rs` (`EditorAction::{ActivateTool, OneShotImageOp, ToolPanelEvent, CancelActiveTool}`) | [`architecture_tool_contract_surface`](../../crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs) — `Tool ≤ 10` métodos, `RasterEditTool ≤ 5` métodos, `PanelEvent ≤ 4` variants | [ADR-0040](../architecture/decisions/0040-tool-as-isolated-feature-crate.md) + [ADR-0041](../architecture/decisions/0041-rasteredit-rename-and-deactivate.md) | Bump cap + amendment de ADR-0040 §7 |

**O que NÃO mexe nesses contratos** (vide §3.A, sem Coord):

- Nó novo num domínio com avaliador — `ph2d-node-<dom>-<slug>/` + sync.
- Tool nova (any shape) — `ph2d-tool-<slug>/` + sync.
- NodeId novo num panel docado — só edita o crate do tool/panel.
- Campo novo num `<Slug>UiEdit` — vive em `ph2d-tool-<slug>/src/params.rs`.

---

## 5. UI canônica — única fonte de verdade

Tudo de UI passa por **tokens**. Sem exceção.

```
docs/design/tokens.json    (designer edita; 4 temas; OKLCH p/ cores)
        │  (build.rs em ph2d-tokens regenera)
        ▼
crates/ph2d-tokens/src/    (5 enums: ColorToken, Spacing, Radius, TypeToken, StrokeToken)
        │
        ▼
let bg = ColorToken::Bg2.resolve(theme);
let pad = Spacing::Lg.px();
```

### 5.1 Gates ativos

Violação = build vermelho. Não há "vou abrir exceção".

| Gate | O que barra |
|---|---|
| [`no_literal_color`](../../crates/ph2d-editor-core/tests/no_literal_color.rs) | hex `0xRRGGBB`, `Color::rgba8(...)`, `Color::WHITE` em widget/screens |
| `no_magic_numeric` | `f32`/`f64` literais em UI fora do allowlist (`0.0`, `±0.5`, `±1.0`, `±2.0`) |
| [`hr12_widgets_a11y`](../../crates/ph2d-editor-core/tests/hr12_widgets_a11y.rs) | widget que não emite `Node` AccessKit |
| [`architecture_widget_loc_cap`](../../crates/ph2d-editor-core/tests/architecture_widget_loc_cap.rs) | widget primitive > 500 LOC |
| [`architecture_widget_showcase_coverage`](../../crates/ph2d-editor-core/tests/architecture_widget_showcase_coverage.rs) | widget que não aparece no Widget Gallery (nem em opt-out) |
| [`architecture_panel_chip_pill_no_stepper`](../../crates/ph2d-editor-core/tests/architecture_panel_chip_pill_no_stepper.rs) | chip pill sem `link_slider_number`/`mark_chip_no_stepper` (phantom stepper) |
| `mockup_tokens_exist` | `var(--X)` em mockup HTML não resolve em tokens.json |
| `architecture_register_all_alphabetical` | `register_all*` / Cargo deps fora de ordem |
| `staleness` (tool + node) | sync esquecido |
| [`architecture_cycle_prevention`](../../crates/ph2d-editor-core/tests/architecture_cycle_prevention.rs) | `editor-core` ⊥ `panel-*`/`ph2d-editor`; `editor-core` ⊥ `tool-*` (exceto `ph2d-tool-registry`); `panel-*` ⊥ outro `panel-*` |
| 🔒 `architecture_tool_contract_surface` | caps Tool/RasterEditTool/PanelEvent (§4) |
| 🔒 `architecture_contract_surface` (nodegraph) | caps NodeOp/OpResolver/NodeManifest (§4) |
| `tool_manifest_design_sync` | `docs/design/tools/<slug>.toml` divergente do MANIFEST |
| [`no_tofu_glyphs`](../../crates/ph2d-editor-core/tests/no_tofu_glyphs.rs) | glifos fora da fonte Inter bundled (setas, ⌘, ↵, ✕, ▸ etc.) viram tofu |

**Exceção declarada legítima:** comentário `// LITERAL-COLOR-OK: <razão>` ou `// LITERAL-PX-OK: <razão>` na mesma linha. Coord valida na revisão.

### 5.2 Widget Gallery é a fonte de verdade

[`ph2d-panel-widget-gallery`](../../crates/ph2d-panel-widget-gallery/) (showcase em [`editor-core/src/widget/showcase/`](../../crates/ph2d-editor-core/src/widget/showcase/) + seed em [`pre_populate.rs`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs)) é a **única fonte de verdade da UI**. Todo painel novo **DEVE** usar EXATAMENTE o mesmo padrão de cada widget que aparece no Gallery. Sem "minha variação compacta".

#### Regras herdadas do Gallery (cada uma já queimou ≥1×)

1. **Slider + chip pareados → SEMPRE `store.link_slider_number(slider_id, chip_id)`.** Engata mirror bidirecional automático + clamp `0..1`. Sem o link, painel escreve mirror manual que dessincroniza. Chip e slider compartilham espaço `0..1`; unidade natural ("2.00 clip", "+0.30 brightness") via `display_override` no `paint_slider_with_chip_layout` (paint-only). Veja [`pre_populate.rs:212-231`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs).
2. **Tempo real no canvas — todo slider que altera pixels publica preview por frame.** Tool expõe `take_params_dirty()` + `current_preview()` (ou implementa `RasterEditTool` §3.A.4); bridge em [`render_loop/<tool>_bridge.rs`](../../shells/desktop/src/render_loop/) (espelho de [`bgremoval_preview.rs`](../../shells/desktop/src/render_loop/bgremoval_preview.rs)) refaz cache `Arc<Vec<u8>>` quando dirty, pinta com `vector_scene.draw_image_rgba`, zera em Apply/deactivate. Sem isso = canvas congela = UX inaceitável.
3. **`paint_number_chip` (pill, sem setinhas) ≠ `paint_number_input_with_buffer` (boxed).** Dispatch carve coluna 16-22 px lado direito de TODO `NumberInput` como hit-zone de stepper. Pra chip pill = zona invisível: click direito arma `number_stepper_hold` → incrementa a cada 30ms com cursor parado. **Sempre chame `store.mark_chip_no_stepper(chip_id)` no populate.**
4. **Chip drag = incremental delta**, não absolute-from-Down. Dispatch usa `step_dx = event - last` + `advance_number_input_drag_anchor` por Move. Modelo absoluto pregava valor no bound até cursor voltar até `start_x` — bug invisível.

### 5.3 Anti-padrões UI que já queimaram (NÃO repita)

Bases de conhecimento: [`docs/UI_Bugs/README.md`](../UI_Bugs/README.md) e [`docs/Image Tools Bugs/README.md`](../Image%20Tools%20Bugs/README.md). **Leia antes de tocar em painter/dispatch/tool.**

1. **Estado de MODO e estado DERIVADO não podem viver desacoplados.** Toggle/modo (ex: `image_edit.mode_on`) que governa o que aparece (tool, painel, preview): quem desliga o modo é responsável por **desligar tudo** que ele expõe. Lugar certo = **reconciliação por frame** sobre estado derivado, **não** guard pontual no click.
2. **Enumere TODOS os caminhos de ativação.** Feature costuma ter >1 via de ligar (pill TopBar, tool palette, atalho, bus action). Gatear só uma deixa o bug vivo. Grep TODOS os `set_active`/push de action OU centralize.
3. **Hit-test e paint do MESMO widget têm que ser gateados pela MESMA condição.** Hit-test rodando onde o widget NÃO é pintado = zona de clique invisível. Sempre condicione paint E hit juntos.
4. **Pertencimento é data-driven, não lista de ids hardcoded.** "É image tool?" = está no cluster `"image_tools"` do manifest, resolvido por UM helper (`is_image_edit_tool`) — não `id == "x" || id == "y"` espalhado.
5. **Diagnostique medindo, não chutando.** Bug de UI/input com repro: instrumente (env-gate `PH2D_UIDBG`) e capture estado real antes de propor fix. Reverta a instrumentação no fim.

#### Checklist antes de mergear painel novo (Coord)

- [ ] Cada slider+chip tem `store.link_slider_number(slider, chip)` no populate.
- [ ] Cada chip pill tem `store.mark_chip_no_stepper(chip)` no populate.
- [ ] Storage chip + slider no MESMO espaço (`0..1`); unidade natural só em paint via `display_override`.
- [ ] Se altera pixels: existe `render_loop/<tool>_bridge.rs` espelhando `bgremoval_preview.rs`, refresh em `take_params_dirty()` + overlay via `draw_image_rgba`.
- [ ] `apply_event` é forwarder thin (sem mirror manual slider↔chip).

Faltou → **bounce pro Implementador antes de mergear.** Não "vou abrir exceção".

---

## 6. Codificação rápida

**Princípio:** não duplique o pre-commit hook durante editing burst. **Hook ≠ CI** em 2 pontos:

1. **clippy `--all-targets`:** o tier **T2 workspace** roda `cargo clippy --workspace -- -D warnings` **SEM `--all-targets`** (cortado no perf audit 2026-05-19 por velocidade). CI roda completo: `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings`.
2. **arch-gates:** mudança estrutural (widget novo, campo serializado) dispara arch-tests que só aparecem após ~3min de compile do hook. Rode o gate do crate antes (§6.1).

**Regra: antes do PUSH, rode `./scripts/ship.sh`** — paridade-CI completa (§8). E `git commit` SEMPRE em background (hook estoura timeout 2min em foreground).

### 6.1 Tabela de validação

| Situação | Comando | Tempo |
|---|---|---|
| Editou 1 arquivo, quer ver se compila | `cargo check -p <crate>` | 3-15s |
| Editou crate, quer rodar testes | `cargo test -p <crate>` | 5-30s |
| Quer rodar UM teste | `cargo test -p <crate> -- <pattern>` | 1-5s |
| Editou foundational, quer ver downstream | `cargo check --workspace` | 30-60s warm |
| Vai commitar (T2 hook vai rodar) | **nada** — deixa o hook validar | 0s |
| **Antes do push (obrigatório)** | `./scripts/ship.sh` | 3-8min warm |
| Só mudou `.md` | **nada** — hook é T0 (skip) | 0s |

### 6.2 LOC threshold (não interrompa o editing burst)

| LOC editados | Comando OK |
|---|---|
| 0-400 | nada, continue |
| 400-1200 | `cargo check -p <crate>` opcional |
| 1200+ ou módulo inteiro | `cargo check -p <crate>` — sane stop |
| Antes do commit | nada — hook valida |

**Não rode `cargo test` durante editing burst.** Só no hook ou em diagnóstico de falha específica.

### 6.3 O que NÃO fazer

- ❌ `cargo test --workspace` depois de cada edit
- ❌ `cargo clippy --workspace --all-targets` a cada COMMIT (SIM antes do PUSH via ship.sh)
- ❌ Re-rodar testes que já passaram pra "confirmar"
- ❌ Validar baseline no início da sessão se último commit já está verde
- ❌ `cargo build` antes de `cargo test` (test já compila)
- ❌ Re-`Read` arquivo que acabou de editar

### 6.4 Pre-commit hook tiered

| Tier | Ativa quando | Tempo |
|---|---|---|
| **T0** | só docs / `.md` / scripts | ~5s |
| **T1** | arquivos de UM crate isolado | ~30s |
| **T2 escopado** | multi-crate **sem** foundational/Cargo.toml/shells | ~30s-3min |
| **T2 workspace** | `Cargo.toml/lock`, foundational, `shells/desktop/` | ~5-15min |

Acidentalmente trigou T2 workspace numa pasta isolada? Provavelmente staged junto com algo de outro agente — confira `git status --cached`.

**Cortes A+B (2026-05-19):** hook NÃO roda `cargo test --doc --workspace` nem `clippy --all-targets`. Esses ficam pro CI. Implicações:
- Doctest novo só verificado em CI. Quem cria valida manual com `cargo test --doc -p <crate>`.
- Benches/examples só clippados em CI.

### 6.5 Como NÃO escrever test slow

**❌ NÃO faça:**
- `TextSystem::new()` — enumera fontes do sistema (25-77s × site). Use `TextSystem::without_system_fonts()`.
- Alloc gigante pra exercitar limit-check (`RgbaImage::new(16384, 16384)` = 1 GiB). Use dimensão 1 px acima do limite (8193×1 = 32 KiB).
- GPU init repetido por test. Use `OnceLock<Option<GpuContext>>` lazy module-level.
- Font shaping real quando só precisa shape de palavra fixa.

**✅ Faça:**
- Setup caro em `OnceLock` lazy, compartilhado entre tests do mesmo binário.
- Input minimal: 1 caso simples + 1 caso edge.
- IO real → `#[ignore]` + `cargo test -- --ignored` no CI separado.

---

### 6.6 Velocidade multi-agente — alta cadência (2026-05-28)

Com N implementadores numa máquina de 8 GiB, o build/teste **redundante** é o
gargalo. Regras para implementação de altíssima velocidade + validação pesada
**1× no fim**:

1. **Inner loop = SÓ `cargo check -p <crate>`** (ou `scripts/cargo-check-narrow.sh <crate>`
   para cortar payload de erro). **ZERO** `cargo test`, **ZERO** `clippy --all-targets`,
   **ZERO** auditor adversarial **por task** durante o burst de edição.
2. **A validação pesada é BATCHED no fim do módulo/wave**, não por task: a
   auditoria adversarial (≥2 lentes rotacionadas), `nextest`, `clippy --all-targets`
   e o smoke acontecem **uma vez** sobre o diff acumulado do módulo — não N× por
   micro-task. (O padrão-ouro é preservado **no gate**, não repetido a cada commit.)
3. **Build dedup via CoW**: a base warm fica em `target-slots/base`. Cada agente
   roda UMA vez `bash scripts/slot-seed.sh <slot>` (clone APFS `cp -c`, ~1s, 0 bytes)
   e depois **prefixa cada cargo** com o `CARGO_TARGET_DIR` impresso — o Bash-tool
   **não persiste env** entre chamadas, então `source slot-env.sh` não basta para
   agentes. Ex.: `CARGO_TARGET_DIR=<path> cargo check -p <crate>`. **Não use o
   `target/` default** (foi recuperado). Rebuild da base (Coordenador, SOZINHO,
   RAM-heavy) só quando `Cargo.lock`/toolchain muda.
4. **Teste de módulo rápido:** `scripts/nextest-impacted.sh` (roda só `rdeps()` do
   que mudou + força o golden de determinismo). O gate final continua sendo
   `./scripts/ship.sh` (`nextest run --workspace --cargo-profile ci-test` — paridade
   exata com o CI, deps em opt-level=3).
5. **Concorrência:** máx **2-3 `cargo` simultâneos** (RAM 8 GiB). O Coordenador
   escalona quem compila quando via SESSION_ACTIVE (§1.1); a CoW barateia criar
   slots mas **não** levanta esse teto.

**Anti-padrão que matou a velocidade (2026-05-28):** mandar cada implementador
rodar `cargo test` + `clippy --all-targets` + **spawnar 2 auditores adversariais
POR TASK**. Com 5 agentes isso é uma tempestade de builds redundantes. Auditoria
é **por módulo fechado**, não por micro-task.

---

## 7. Anti-colisão git

`git commit` é serializado pelo índice global do git. Duas sessões com arquivos staged ao mesmo tempo: uma roda commit e agarra os arquivos da outra junto.

### 7.1 Protocolo atômico stage→commit

```bash
# 1) Antes de stage: confira working tree
git status
#    Há M/?? que não são seus? PARE. Outro agente em vôo.

# 2) Stage só os seus. NUNCA -A / -a / git add .
git add <arquivos-específicos>

# 3) Antes de commit: confere índice
git status --cached
#    Arquivo que não estagiou? Vazamento.
#    git restore --staged <não-meus>

# 4) Commit. Hook tiered roda automaticamente.
git commit -m "<descrição em inglês, imperativo, <70 char>"
```

Stage→commit é **operação contínua**. Não pause entre os dois passos.

### 7.2 Proibições

- **Nunca** `git push --force` em main
- **Nunca** `--no-verify` (se hook falha, fix root cause)
- **Nunca** `git commit --amend` (sempre novo commit)
- **Nunca** `git config` mudando settings do repo
- **Nunca** `git restore --staged --worktree` em path fora da sua pasta sem coordenar

### 7.3 Sintomas de colisão

| Sintoma | Recuperação |
|---|---|
| `fatal: cannot lock ref 'HEAD'` no commit | Outra sessão commitou no meio. `git status` → diagnose |
| `git status` mostra M que você não tocou | Outro agente paralelo. NÃO comite. Reporte |
| `git log -1` mostra mensagem fundida (2 títulos) | Colisão. Se NÃO pushado: `git reset --soft HEAD~1` + split + recommit |
| Hook trigga T2 quando esperava T1 | `git status --cached` — vazamento de outro agente |

### 7.4 Armadilhas conhecidas

**Typos engine bloqueia palavras pt-BR ambíguas.** `erros` (typo de `errors`), `usso` (typo de `use`), `nao` sem acento (typo de `not`). Solução: prefira sinônimos ou use acento; se necessária, adicione exceção em `.typos.toml` `[default.extend-words]` **com justificativa no commit**, não esconda com `--no-verify`.

**Cargo lock entre sessões.** Se rodar `cargo` enquanto outra sessão Claude Code paralela está rodando, a 2ª **espera silenciosamente** pelo lock. Não é crash, só lentidão. Use `slot-env.sh` pra isolar (§1.2).

---

## 8. Ship + Push + CI (Coordenador absorve PRCI)

### 8.1 Fast mode (dia) vs Ship (fim do dia)

**Princípio: separe "implementar" de "entregar".** Validação completa + CI rodam **1× por jornada**, não 1× por commit.

**De dia — fast mode:**
- Checkpoints com `git commit --no-verify` → instantâneo, pula hook. Salva trabalho, permite reverter.
- `cargo check -p <crate>` só quando quiser confirmar. Sem `--workspace`/test em loop.
- **ZERO push, ZERO CI durante o dia.**

**Fim do dia — ship (Enio dispara: "commit"/"push"/"ship"/"fim do dia"):**
O Coordenador entra em **modo observa-e-corrige** e tem a OBRIGAÇÃO de entregar verde:

1. **`./scripts/ship.sh`** — job de lint+test do CI inteira, local, de uma vez (fmt, clippy `--all-targets --features ph2d-spike/bevy_ecs`, `cargo machete`, `cargo deny`, `cargo audit`, `nextest --workspace`, `typos`). Paridade EXATA com `spike.yml`.
2. Pra CADA `✗`: diagnostica + corrige + re-roda. **NÃO pusha enquanto não estiver 100% verde.**
3. Organiza os checkpoints `--no-verify` do dia em commits limpos (squash se preciso).
4. Push (§8.3) → babysit do CI (§8.4) até verde; em vermelho, fix + re-push até verde (escalona após 3 falhas do MESMO job).
5. Reporta link da run verde ao Enio.

### 8.2 Smoke local — antes do push

```bash
./play.command
```

Smoke é do **Enio**, sob comando do Coord. Coord escreve checklist concreta:

> "Enio, rode `./play.command` e verifica:
> 1. App abre sem panic.
> 2. Tool X aparece na TopBar Image Tools com ícone correto.
> 3. Clique → ação esperada.
> 4. Tools/Actions pré-existentes continuam funcionando.
> 5. Sem regressão visual em Hierarchy / Inspector / Widget Gallery."

### 8.3 Push (Coordenador faz)

Batching: **push UMA vez por jornada**. CI matrix (linux + macOS + windows + replay hash + bench) demora ~30min.

```bash
./scripts/ship.sh    # paridade-CI completa (§8.1)
# Só pusha se ✓
git push origin main
```

### 8.4 Babysit CI

```bash
gh run list --workflow=spike.yml --limit=1 --json databaseId,url
```

Polling **15min** (`gh run watch <id>` ou Monitor com `sleep 900`).

| Resultado | Resposta |
|---|---|
| Success 9/9 | Reporta link + sha bom ao Enio. Ciclo fechado |
| Falha de código | `gh run view --log-failed`, fix local, commit, push, re-watch |
| Falha de infra (cache/network/rustup flaky) | `gh run rerun --failed` + re-watch |
| 3 falhas consecutivas do mesmo job | Escala pro Enio com diagnose |

**Regra de ouro:** fora do babysit, ninguém polla CI. Push, link, próxima tarefa.

### 8.5 Comunicação pós-push

```
✓ Wave <N> pushed. CI run: https://github.com/dibrioli/PH2D/actions/runs/<id>
Entrei em babysit. Reporto quando concluir.
```

E ao terminar:

```
✓ CI verde 9/9 em <duração>. sha bom novo: <sha>.
Ciclo fechado. Disponível para próxima ordem.
```

---

## 9. Quando algo dá errado

| Sintoma | Resposta |
|---|---|
| Não sabe o que fazer | Releia §0 + §1 + pergunte ao Enio |
| Arquivo que não tocou em `git status` | §7.3 (colisão) |
| Hook falha em fmt/clippy/test | Fix root cause; nunca `--no-verify` |
| Hook trigga T2 quando esperava T1 | `git status --cached` — vazamento |
| Smoke quebrou em `./play.command` | Implementador diagnostica + fix local |
| CI failure cíclico (3× mesmo job) | Coord escalona pro Enio |
| Implementador descobre bug fora da pasta | Reporta ao Enio com diagnose; Coord faz |
| Coord quer editar shared mas Impl está working | Anuncie via Enio, espere Impl chegar a estado estável, edite |
| Coord tem dúvida arquitetural | Opções pro Enio com recomendação + tradeoff |
| Memória diz X mas código diz Y | Confie no código. Atualize memória depois |

---

## 10. Cheat-sheet

### 10.1 Hard Rules CI-gated

| HR | Conteúdo | Gate |
|---|---|---|
| HR-3 | Zero-alloc no dispatcher hot-path | `interaction_dispatch_no_alloc` |
| HR-5 | Determinism cross-platform | CI replay-hash matrix (3 OS) |
| HR-12 | A11y obrigatória | `hr12_widgets_a11y` |
| HR-13 | Memory budget declarado | manifest `memory_budget` |
| HR-15 | Zero hex + zero hardcoded UI string | `no_literal_color` + `hr15_no_hardcoded_ui_strings` |
| HR-18 | `shells/<plat>/src/` ≤ 600 LOC | `file_loc_caps` |
| (Wave 9) | Widget primitive ≤ 500 LOC | `architecture_widget_loc_cap` |
| (Wave 9) | Widget aparece no showcase | `architecture_widget_showcase_coverage` |

Completo em [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §HR-1..HR-18.

### 10.2 Caminhos canônicos

| O que | Onde |
|---|---|
| **Node crate** (fan-out (A)) | `crates/ph2d-node-<dom>-<slug>/` |
| **Tool crate** (fan-out (A)) | `crates/ph2d-tool-<slug>/` |
| Painel (caminho (B)) | `crates/ph2d-panel-<slug>/` |
| Widget primitive (caminho (B)) | `crates/ph2d-editor-core/src/widget/<slug>.rs` |
| Chrome handler (caminho (B)) | `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` |
| Vocab UI de um tool | `crates/ph2d-tool-<slug>/src/params.rs` |
| 🔒 Contrato de nós | `crates/ph2d-nodegraph/` + `crates/ph2d-expr/` |
| 🔒 Contrato de tools | `crates/ph2d-editor-core/src/tool.rs` + `action_bus.rs` |
| **Tool registry (GERADO)** | `crates/ph2d-tool-registry-init/` |
| **Node registry (GERADO)** | `crates/ph2d-node-registry-init/` |
| Panel registry (manual) | `crates/ph2d-panel-registry-init/src/lib.rs` |
| Codegens | `tools/ph2d-{node,tool,panel,chrome,widget}-sync/` |
| Widget showcase | `crates/ph2d-editor-core/src/widget/showcase/` |
| Tokens source | `docs/design/tokens.json` → build.rs gera `crates/ph2d-tokens/src/` |
| Tool design TOML | `docs/design/tools/<slug>.toml` |
| Icon SVG | `docs/design/icons/<slug>.svg` |
| Shell init | `shells/desktop/src/init.rs` |
| Arch tests editor | `crates/ph2d-editor-core/tests/` |
| Arch tests contrato tool 🔒 | `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs` |
| Arch tests contrato nodegraph 🔒 | `crates/ph2d-nodegraph/tests/architecture_contract_surface.rs` |

**Removido em ADR-0040 TG-D (`c4063b7`):** `crates/ph2d-editor-core/src/tools/` foi **deletado**. Foundation ⊥ tools gateado. Memória/doc apontando lá = stale.

### 10.3 Comandos mais usados

```bash
# Implementador — durante edição
cargo check -p ph2d-<family>-<slug>
cargo test  -p ph2d-<family>-<slug>
cargo test  -p ph2d-<family>-<slug> -- some_pattern

# Drop-crate fan-out — Implementador roda após criar a pasta
cargo run  -p ph2d-<family>-sync          # regenera wiring
cargo test -p ph2d-<family>-registry-init # staleness fecha

# Coordenador — antes do push (paridade-CI completa, obrigatório)
./scripts/ship.sh

# Coordenador — push + babysit
git push origin main
gh run list --workflow=spike.yml --limit=1
gh run watch <id> --exit-status
```

---

## 11. Referências canônicas

- **Stack + Hard Rules + "Adicionar uma tool":** [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md)
- **Operacional dia-a-dia + CI:** [`CLAUDE.md`](../../CLAUDE.md)
- **Exemplos fan-out 100% paste-ready:** [`examples-fan-out.md`](examples-fan-out.md)
- **Tracker vivo do fan-out de nodes:** [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md)
- **Plano de nodes (W1+W2 fechados, W3+ aberto):** [`docs/plans/2026-05-node-waves.md`](../plans/2026-05-node-waves.md)
- **Plano Wave 11 carry-overs:** [`docs/plans/2026-05-wave-11-carry-overs.md`](../plans/2026-05-wave-11-carry-overs.md)

**ADRs estruturais (leitura indispensável):**

- [ADR-0027 Convention-by-discovery](../architecture/decisions/0027-convention-by-discovery.md)
- [ADR-0029 Trait-driven panel host](../architecture/decisions/0029-trait-driven-panel-host.md)
- [ADR-0030 Multi-domain node engine](../architecture/decisions/0030-multi-domain-node-engine.md)
- [ADR-0031 Node E tool como unidade de feature](../architecture/decisions/0031-node-and-tool-as-feature-unit.md)
- [ADR-0032 `ph2d-nodegraph` substrato](../architecture/decisions/0032-nodegraph-substrate.md)
- [ADR-0033 `ph2d-expr` shared compute](../architecture/decisions/0033-shared-compute-expr.md)
- 🔒 [ADR-0039 Nodegraph contract FREEZE](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)
- 🔒 [ADR-0040 Tool as isolated feature crate](../architecture/decisions/0040-tool-as-isolated-feature-crate.md)
- 🔒 [ADR-0041 RasterEdit rename + deactivate](../architecture/decisions/0041-rasteredit-rename-and-deactivate.md)
- [ADR-0042 Wave 10 closure](../architecture/decisions/0042-wave-10-closure.md)

**Memória LLM (auto-loaded):** `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`

**Histórico anterior** (v6.0..v6.10): `git log docs/IntegracaoMultiAgente/DIRETRIZ.md`. Arquivados pre-v6.0: `docs/archive/multi-agente-pre-v6.0/`.

---

## 12. Quando esta diretriz fica obsoleta

Se a arquitetura mudar materialmente (3º papel surge, fluxo invertido vira lateral, contrato 3 surge), atualize **in-place** e bump versão. **Não fragmente em múltiplos docs** — lição dos 4 docs antigos que dessincronizaram é que doc único é mais fácil de manter.

LLM lendo isto depois de mudança arquitetural maior e diretriz contradiz código: **confie no código**, reporte ao Enio com diagnose, atualize quando autorizado.
