# DIRETRIZ — Operação Multi-Agente PH2D

**Versão:** 5.0 — 2026-05-17 (pós Wave 1-3.2; Wave 4 *source-of-truth UI* em planejamento)
**Substitui:** `01-Enio.md` / `02-Coordenador.md` / `03-Agente-Periferico.md` / `04-Agente-PRCI.md` (arquivados em `ARCHIVE-v4.0-pre-wave-1/`)
**Audiência:** Enio (humano) **e** qualquer sessão LLM que entra no projeto.

---

## TL;DR

> A arquitetura PH2D pós Wave 1-3.2 elimina colisões multi-agente
> por **construção**: cada tool é seu próprio crate; chrome é
> derivado do Registry; design canonical (tokens.json + SVGs +
> TOMLs) gera Rust via `build.rs`; 6 architecture tests + HR-18
> cap bloqueiam regressão.
>
> **Para adicionar uma tool nova:** criar `crates/ph2d-tool-<slug>/`
> + 1 linha em `ph2d-tool-registry-init::register_all`. **Zero
> edit em arquivos centrais.**
>
> Esta diretriz substitui 4 docs operacionais antigos (versão 4.0,
> 2026-05-13) que descreviam um modelo onde Coordenador editava
> manualmente `icons.rs` / `lib.rs` / `fixture.rs` / `ids.rs` /
> `ToolRegistry::new()` — arquitetura morta.

---

## 0. Antes de começar (leitura obrigatória)

Qualquer LLM que entra no projeto deve ler, **nesta ordem**:

1. **[`docs/Migracao/PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md`](../Migracao/PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md)**
   — narrativa do problema multi-agente paralelo + as 4 waves de
   solução. ~450 LOC. **Não pule.**
2. **[`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md)**
   — especialmente §HR-1..HR-18 (Hard Rules) e §1 (arquitetura geral).
3. **[`CLAUDE.md`](../../CLAUDE.md)** — operacional do dia-a-dia
   (CI, push, batching policy).
4. **Memory:**
   `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`
   (auto-loaded) — perfil do Enio, feedback acumulado, histórico
   entre sessões.
5. **[`docs/architecture/decisions/0027-convention-by-discovery.md`](../architecture/decisions/0027-convention-by-discovery.md)
   e [`0028-wave-2-codegen-design-canonical.md`](../architecture/decisions/0028-wave-2-codegen-design-canonical.md)**
   — as duas ADRs canônicas da arquitetura atual.

**Sanity check antes de tocar código:**

```bash
git log --oneline -3                                  # confirma onde HEAD está
git status -sb                                        # working tree clean?
cargo test --workspace --exclude ph2d-asset 2>&1 \
  | grep -E "test result:|FAILED" | head             # baseline verde?
find shells/desktop/src -name "*.rs" -exec wc -l {} \; | awk '$1 > 600'
                                                      # HR-18: zero violações
```

Se qualquer um diverge do esperado, **pare e pergunte ao Enio**.

---

## 1. Topologia

```
Enio (humano, relay)
 │
 ├─ Coordenador  (sessão Claude Code #1, sempre ativa)
 │   • Mantém STATE.md
 │   • Revisa PRs dos Periféricos
 │   • Edita arquivos compartilhados (raro, pós Wave 1-4)
 │
 ├─ Agentes Periféricos (até 4 sessões paralelas)
 │   • Cada uma numa sessão Claude Code separada, mesmo path
 │   • Cada uma trabalha em SEU crate exclusivo
 │   • Comunicam com Coordenador via Enio (copy/paste)
 │
 └─ Agente PRCI (ativado pelo Enio ao final do ciclo)
     • ÚNICO autorizado a `git push` + `gh`
     • Babysit CI até verde
```

Todas as sessões no mesmo diretório principal do projeto. Sem
worktrees. Sem `git checkout -b`. Tudo em main local até o ciclo
fechar.

**Limites operacionais:**
- Máximo **4 Periféricos simultâneos** (limite do STATE.md).
- **Um único Coordenador ativo** por vez. Duas sessões
  pretendendo ser Coordenador = colisão garantida.
- **Push só ao final** do ciclo (batching policy).

---

## 2. STATE.md é a fonte de verdade operacional

[`docs/IntegracaoMultiAgente/STATE.md`](STATE.md) registra
**sempre**:

- Slots ativos (slug, pastas reservadas, status, última atividade)
- Fila de integração (FIFO)
- Pedidos pendentes ao Coordenador
- Lock de integração (Coordenador idle | integrando X)
- Sha conhecido bom (rollback target)
- Histórico de operação (append-only, mais recente no topo)

**Quem escreve:** **só o Coordenador.** Periféricos e PRCI
**leem**. Conflitos = grave (vide §5).

Após cada operação significativa (atribuição slot, aprovação
pasta, status mudou, fila avançou, sha bom atualizado, integração
concluída), Coordenador commita STATE.md como `chore(coordenador):
<descrição>`.

---

## 3. Seu papel (selecione um)

### 3.1 Você é o Enio

Você é o **relay humano**: copia/cola mensagens entre Coordenador
e Periféricos. Decide:

- Quando iniciar nova feature
- Quando parar e fazer PR pro GitHub
- Resolver impasses quando Coordenador apresenta opções
- Smoke visual (`./play.command`)

**Fluxo padrão:**

1. **Setup uma vez por sessão de trabalho:**
   - Abre Sessão Claude Code #1. Cola: "Você é o Coordenador. Leia
     `docs/IntegracaoMultiAgente/DIRETRIZ.md`. Inicialize a operação."
   - Coordenador lê tudo + STATE.md + reporta pronto.

2. **Por feature nova:**
   - Diz ao Coordenador: "Quero feature X. Atribua slot."
   - Coordenador prepara briefing (cola desta DIRETRIZ + escopo +
     slot). Atualiza STATE.md.
   - Você abre nova Sessão Claude Code (mesmo path), cola o
     briefing.
   - Periférico lê briefing + SKILL + STATE.md + esta DIRETRIZ.
     Cria crate `ph2d-tool-<slug>/` (vide §4.1). **Não precisa
     pedir permissão de pasta** — a pasta é o crate dele.
   - Periférico desenvolve, comita local, reporta pronto.
   - Coordenador revisa, valida CI lints + smoke, marca `done` em
     STATE.md.

3. **Final do ciclo:**
   - Você decide: "manda PR pro GitHub".
   - Ou Coordenador assume PRCI, ou você abre sessão dedicada nova
     com `Você é o PRCI. Leia DIRETRIZ §3.4`.
   - Push + babysit CI per CLAUDE.md.

**Regras de ouro pra você:**

- Confia mais em STATE.md do que em memória sua ou de qualquer
  agente.
- Comunicação é assíncrona — não force agentes a "decidir agora".
- Smoke visual é SUA responsabilidade. Quando agente diz "pronto",
  abre `./play.command` e testa concretamente.
- **Push** é decisão sua — agente nunca push sem você autorizar
  explicitamente.

### 3.2 Você é o Coordenador

**Responsabilidades:**

1. **Atender pedidos do Enio** (relay):
   - "Quero feature X" → atribuir slot em STATE.md, preparar briefing.
   - "Agente Y reporta pasta Z" → validar (pasta livre? bate
     arquitetura?), atualizar STATE.md, devolver "aprovado" ou
     "use W em vez de Z".
   - "Agente Y reporta pronto" → adicionar à fila de integração.

2. **Revisar integrações:**
   - Quando vez do slug chega: ler o que o Periférico
     commitou em main local.
   - **Pós Wave 1-4:** geralmente NÃO precisa editar arquivos
     compartilhados — chrome aparece via Registry-derived;
     adicionar tool é só `register_all` (vide §4.1).
   - Rodar validação:
     ```bash
     cargo fmt --all -- --check
     cargo clippy --workspace --all-targets -- -D warnings
     cargo test --workspace --exclude ph2d-asset
     ```
   - **Smoke visual** (Enio confirma via `./play.command`).
   - Atualizar STATE.md: status `done`, sha bom novo.

3. **Manter STATE.md** sempre atualizado (a cada operação).

4. **Edição de arquivos compartilhados — RARO pós Wave 1-4:**
   - Adicionar tool: SÓ `register_all` em `ph2d-tool-registry-init`
     (1 linha). Coordenador faz.
   - Adicionar SVG novo: dropar em `docs/design/icons/<slug>.svg`.
     Coordenador faz.
   - Adicionar tool TOML: `docs/design/tools/<slug>.toml`.
     Coordenador faz.
   - Tokens.json updates (designer): Coordenador faz.

5. **Disciplina de commit** — anti-colisão com Periféricos:
   - **Antes de cada `git add`**, rode `git status`. Se houver
     arquivos `M`/`??` que não são seus, **PARE** — outro agente
     no meio de commit. Aguarde.
   - **Antes de cada `git commit`**, rode `git status --cached`.
     Se índice tem arquivo que você não estagiou, **PARE**.
     `git restore --staged <não-meus>`, então commit.
   - Stage→commit é UMA operação atômica.
   - Se vai trigar hook tier T2 (toca shared / shells), avise via
     Enio antes pra Periféricos segurarem commits 3-5min.

6. **Final do ciclo** — passar a PRCI:
   - Quando Enio diz "manda PR pro GitHub":
     - Verifica `STATE.md` (fila vazia, sha bom alinhado, working
       tree clean).
     - Comita STATE.md final.
     - Reporta a Enio: "Local pronto. Mude papel pra PRCI ou abra
       sessão nova com DIRETRIZ §3.4."

**O que você NÃO faz:**
- Codifica feature nova (Periféricos fazem).
- Pinta widget novo (Periféricos fazem).
- Push pro GitHub (PRCI faz).
- Decide o roadmap (Enio decide).

### 3.3 Você é um Agente Periférico

Você foi instanciado numa sessão Claude Code pra implementar UMA
feature. Sessão paralela com outras Periféricas (até 3 outras
ativas).

**Antes de qualquer código:**

1. Confirme estado (vide §0 sanity check).
2. Leia STATE.md — encontre seu slug + slot atribuído.
3. Leia o briefing que o Enio te colou.
4. Leia o cookbook relevante em §4 desta DIRETRIZ.
5. **NÃO precisa pedir aprovação de pasta** se vai criar
   `crates/ph2d-tool-<slug>/` (convenção canônica). Outras pastas
   exclusivas (e.g., novo crate stub) → pedir.

**O que você FAZ:**

- Cria seu crate em `crates/ph2d-tool-<slug>/` com:
  - `Cargo.toml`
  - `src/lib.rs` — `pub const MANIFEST: ToolManifest` + `pub fn register`
  - `src/icon.rs` (se action one-shot ou tool stateful, BezPath)
  - `src/algorithm.rs` (se aplicável, pure-Rust logic + tests)
- Adiciona seu crate como `workspace member` em `Cargo.toml` raiz.
  Isso é a ÚNICA edit fora do seu crate.
- Adiciona 1 linha em `crates/ph2d-tool-registry-init/src/lib.rs::register_all`:
  ```rust
  ph2d_tool_<slug>::register(reg);
  ```
- **Designer dropa** `docs/design/tools/<slug>.toml` +
  `docs/design/icons/<icon-slug>.svg`. Se você precisa que
  designer faça, reporte ao Enio com payload claro.

**O que você NÃO toca (regra clara):**

- **Tudo** fora do seu crate, exceto as 2 linhas exigidas:
  - `Cargo.toml` raiz (1 linha em `members`)
  - `ph2d-tool-registry-init/src/lib.rs::register_all` (1 linha)
- **NUNCA** edita:
  - `crates/ph2d-editor/src/icons.rs` (build.rs gera; designer
    edita SVG)
  - `crates/ph2d-editor/src/screens/hero/fixture.rs` (chrome derivado)
  - `crates/ph2d-editor/src/screens/hero/ids.rs` (hash-based)
  - `crates/ph2d-tokens/src/color.rs` (tokens.json source)
  - `lib.rs` de qualquer crate (path canônico, não re-export central)
  - SKILL, CLAUDE, ADRs, esta DIRETRIZ, `docs/Migracao/`
  - `shells/desktop/`, `shells/ipad/`, etc.
  - Outros tool crates (`ph2d-tool-*`)
  - Crates foundational: `ph2d-core`, `ph2d-ecs`, `ph2d-tokens`,
    `ph2d-host`, `ph2d-tool-registry`

**Antes de cada commit:** `git status` + `git diff --stat`.
Confirma que só sua pasta + as 2 linhas dos arquivos shared
aparecem.

**Commits locais:**

- Sem branches. Direto em main local quando atinge estado estável
  (cargo check do seu crate verde).
- Mensagem em inglês, imperativo, < 70 char.
- Pre-commit hook tiered (vide §5).
- **NUNCA `git push`** — PRCI faz no final.

**Reporta "pronto" pra Enio quando:**

- Todos os tests passam no seu crate.
- `cargo clippy -p ph2d-tool-<slug> -- -D warnings` clean.
- Architecture tests workspace-wide passam:
  - `cargo test --workspace --exclude ph2d-asset 2>&1 | grep "test result"`
- Smoke visual no `./play.command` foi confirmado por Enio (você
  pede; Enio testa).

**Quando precisa de algo fora:**

- Dep externa nova (Cargo.toml de outro crate) → reporta ao Enio.
- Mudança em foundational crate → reporta ao Enio.
- Coordenador atende (via Enio relay).

### 3.4 Você é o Agente PRCI

Você foi ativado pelo Enio ao final do ciclo. Pode ser:
- Sessão dedicada nova
- A mesma sessão Coordenador que trocou de papel

**Você é o ÚNICO autorizado a usar `git push` e `gh`.**

**Protocolo:**

1. **Verifica estado:**
   ```bash
   git log origin/main..HEAD --oneline   # quantos commits ahead
   git status -sb                        # working tree clean
   ```

2. **Push:**
   ```bash
   git push origin main
   ```

3. **Babysit CI** (CLAUDE.md §"CI/GitHub Actions"):
   ```bash
   sleep 5 && gh run list --workflow=spike.yml --limit=1 \
     --json databaseId,url
   gh run watch <run-id> --exit-status
   ```

4. **Se CI verde** (10/10 jobs):
   - Reporte ao Enio: link da run + "CI verde, sha bom = `<sha>`".
   - Coordenador atualiza STATE.md com novo sha bom.

5. **Se CI falha:**
   - Diagnostica via `gh run view <id> --log-failed | tail -80`.
   - Comita fix local (você assume papel Periférico temporariamente).
   - Push de novo.
   - Re-watch.
   - **Máximo 3 ciclos** consecutivos de falha do mesmo job antes
     de escalonar pro Enio.
   - PRCI loop policy completa em CLAUDE.md §"CI/GitHub Actions".

**Hard rules pra você:**

- **Never `git push --force` em main.** No exceptions.
- **Never `--no-verify` em commits.** Se hook falha, fix root cause.
- **Never amend.** Sempre new commit (CLAUDE.md "Git Safety Protocol").

---

## 4. Receitas canônicas

### 4.1 Adicionar tool nova (pós Wave 1-3.2)

**Receita 6-passos:**

1. **Designer** (humano ou Claude Design) cria
   `docs/design/tools/<slug>.toml` com a spec funcional:

   ```toml
   [tool]
   id          = "my_tool"
   cluster     = "image_tools"       # ou outro cluster
   zone        = "top_right"
   order       = 70                  # sorting within cluster
   a11y_role   = "Button"
   icon_slug   = "my-icon"           # → docs/design/icons/my-icon.svg
   touches_sim = false

   [label]
   fluent_key   = "tool.my_tool.label"
   pt_br_inline = "My Tool"
   en_us_inline = "My Tool"

   [memory_budget]
   vram_mb        = 0
   ram_mb         = 0
   heap_script_mb = 0
   ```

2. **Designer** dropa `docs/design/icons/my-icon.svg` (Lucide-style,
   24×24 viewBox, `stroke="currentColor"`).

3. **Dev (Agente Periférico)** cria `crates/ph2d-tool-my-tool/`:

   ```
   crates/ph2d-tool-my-tool/
   ├── Cargo.toml
   ├── src/
   │   ├── lib.rs           — pub const MANIFEST + pub fn register
   │   ├── icon.rs          — BezPath glyph (~30 LOC, hand-port do SVG)
   │   └── algorithm.rs     — pure-Rust logic + unit tests (se aplicável)
   ```

   `Cargo.toml` template:
   ```toml
   [package]
   name = "ph2d-tool-my-tool"
   version.workspace = true
   edition.workspace = true
   rust-version.workspace = true
   publish.workspace = true
   license.workspace = true
   authors.workspace = true

   [lib]

   [dependencies]
   ph2d-tool-registry = { path = "../ph2d-tool-registry" }
   ph2d-vector        = { path = "../ph2d-vector" }
   ph2d-a11y          = { path = "../ph2d-a11y" }
   ph2d-core          = { path = "../ph2d-core" }
   ```

   `src/lib.rs` template:
   ```rust
   #![forbid(unsafe_code)]
   //! ph2d-tool-my-tool — <description>.

   use ph2d_a11y::Role;
   use ph2d_core::MemoryBudget;
   use ph2d_tool_registry::{
       BezPath, HandlerFn, McpExposure, Registry, ToolHandler,
       ToolManifest, Zone,
   };

   pub mod icon;
   pub mod algorithm;   // se aplicável

   fn shadow_handler() {}

   pub const MANIFEST: ToolManifest = ToolManifest {
       id:           "my_tool",
       label_key:    "tool.my_tool.label",
       icon_fn:      icon::my_tool_bezpath,
       zone:         Zone::TopRight,
       cluster:      "image_tools",
       order:        70,
       a11y_role:    Role::Button,
       handler:      ToolHandler::OneShot {
                         on_click: shadow_handler as HandlerFn,
                     },
       memory_budget: MemoryBudget::new(0, 0, 0),
       touches_sim:  false,
       mcp:          McpExposure::reserved(),
   };

   pub fn register(reg: &mut Registry) {
       reg.register(&MANIFEST);
   }
   ```

4. **Dev** adiciona 1 linha em `Cargo.toml` raiz:
   ```toml
   members = [
       ...
       "crates/ph2d-tool-my-tool",
   ]
   ```

5. **Dev** adiciona 1 linha em
   `crates/ph2d-tool-registry-init/src/lib.rs::register_all`:
   ```rust
   ph2d_tool_my_tool::register(reg);
   ```
   E adiciona o crate como dep em
   `crates/ph2d-tool-registry-init/Cargo.toml`:
   ```toml
   ph2d-tool-my-tool = { path = "../ph2d-tool-my-tool" }
   ```

6. **Validação automática (CI):** 6 architecture tests rodam:
   - `tool_manifest_design_sync` — TOML ↔ MANIFEST cross-validation
   - `chrome_manifest_coverage` — chrome NodeId ↔ manifest hash
   - `node_id_collisions` — pairwise NodeId uniqueness
   - `no_literal_color` — nenhum hex em widget/screens
   - `file_loc_caps` — HR-18 cap (shells/desktop/src/)
   - cross-platform replay hash, MSRV, clippy

**Chrome aparece automaticamente.** O painter de
`image_action_pills` consome `Registry::cluster("image_tools")` e
renderiza a pill com o icon + label. Zero edit em chrome painters.

**NÃO toca:** `lib.rs` raiz, `icons.rs`, `tools/mod.rs` interno
do editor, `screens/hero/fixture.rs::topbar_clusters()`,
`screens/hero/ids.rs`, `Cargo.toml` raiz (além de adicionar o
member).

### 4.2 Adicionar widget novo (Wave 4 pendente)

> **Status:** Wave 4 (source-of-truth UI) ainda não fechou. Esta
> seção será expandida quando fechar. Por ora, siga:

1. Estuda widgets canônicos via **Widget Gallery** (TopBar →
   palette icon → painel flutuante com 10 seções).
2. Cria seu widget em `crates/ph2d-editor/src/widget/<slug>.rs`
   (após Wave 4: ou em um crate próprio `ph2d-widget-<slug>/`,
   se o subsistema for grande).
3. Implementa o pattern canônico:
   - struct com data
   - state enum (Normal / Hovered / Pressed / Focused / Disabled)
   - `pub fn paint_<slug>(widget, rect, scene, theme)` — consome
     `ColorToken::X.resolve(theme)`, `Spacing::X.px()`,
     `Radius::X.px()`, `StrokeToken::X.px()`, `TypeToken::X.px()`.
   - `pub fn build_a11y(&self, ...) -> Node` — HR-12.
4. Adiciona ao showcase em `screens/hero/inspector/showcase.rs`
   para aparecer no Widget Gallery.
5. **Lints pós Wave 4:** o `no_magic_numeric` lint + estendido
   `no_literal_color` vão bloquear `Color::rgba8(...)`,
   `Stroke::new(1.5)`, `8.0` literal — você é forçado a usar
   tokens canônicos.

### 4.3 Modificar tool existente

Mesma receita do §4.1 mas você edita o crate existente. Touch só
`crates/ph2d-tool-<slug>/src/`. Se mudou o TOML também, edite o
TOML correspondente. CI re-valida.

### 4.4 Adicionar painel novo (Wave 5 ✅ disponível)

> **Status:** Wave 5 entregou o pattern em 2026-05-17. Wave 6+7
> (Phase 1+2, 2026-05-17) extraiu `ph2d-editor-core` com primitives
> compartilhados — `widget/`, `interaction/`, `paint`, `gizmo/`,
> `floating_panel`, `ids` etc. Receita abaixo continua válida
> (paths via re-export). Wave 6+7 Phase 3 (panel-as-crate) + Phase 4
> (apply_event distribuído) deferidos — vide
> [ADR-0028 §Wave 6+7](../architecture/decisions/0028-wave-2-codegen-design-canonical.md#wave-67-2026-05-17--hotspot-decomp--editor-core-primitives-crate).

**Receita 4-passos** (simétrico ao §4.1 tool-as-crate):

1. **Dev** cria o módulo do painel em
   `crates/ph2d-editor/src/screens/hero/<slug>/mod.rs`
   (ou pasta exclusiva em outro local se faz mais sentido). Declara
   `pub static PANEL_MANIFEST: PanelManifest` com os 3 fn pointers:

   ```rust
   use crate::panel_registry::{PaintCtx, PanelManifest};
   use crate::interaction::{WidgetEvent, WidgetStore};
   use crate::screens::hero::HeroScreen;

   pub static PANEL_MANIFEST: PanelManifest = PanelManifest {
       id: "my_panel",
       panel_node_id: super::ids::MY_PANEL, // hash-derived NodeId
       default_visible: false,
       paint_fn: paint_thunk,
       apply_event_fn: apply_event_thunk,
       populate_fn: populate,
   };

   fn paint_thunk(ctx: &mut PaintCtx) {
       if !visibility_check(ctx.hero) { return; }
       // ... full per-frame logic:
       //   lazy default rect + drag/resize clamp via
       //   `style::clamp_panel_rect` + chrome publish +
       //   actual paint + content_h publish + scroll clamp.
   }

   fn apply_event_thunk(_hero: &mut HeroScreen, _ev: WidgetEvent) -> bool {
       false // stub OK por enquanto; god-match em `HeroScreen::apply_event`
             // ainda é o dispatcher canônico (canonicalization wave futura)
   }

   pub fn populate(store: &mut WidgetStore) {
       // register widget NodeIds (drag handle, resize handle, etc.)
   }
   ```

2. **Dev** adiciona 1 linha em
   `crates/ph2d-editor/src/panel_registry.rs::PANEL_REGISTRY`:

   ```rust
   pub static PANEL_REGISTRY: PanelRegistry = PanelRegistry::new(&[
       ...,
       &crate::screens::hero::<slug>::PANEL_MANIFEST,
   ]);
   ```

3. **Dev** adiciona o NodeId em `screens/hero/ids.rs` (hash-derived):

   ```rust
   pub const MY_PANEL: NodeId = hash_node_id("my_panel");
   ```

4. **Dev** chama `populate` em `HeroScreen::pre_populate_store` (se
   não estiver listado lá já — pós Wave 5 pode-se iterar
   `PANEL_REGISTRY.manifests()` chamando `populate_fn` automaticamente,
   mas a chamada hoje continua explícita).

**Chrome aparece automaticamente:** o `z_order` loop em
`paint_hero_screen` chama `(manifest.paint_fn)(&mut ctx)` para
todo NodeId no z_order que bate `find_by_panel_node_id`. Cross-panel
state vai em sub-struct do `HeroScreen` (vide §3 Stage B em
[`docs/architecture/decisions/0028-wave-2-codegen-design-canonical.md`](../architecture/decisions/0028-wave-2-codegen-design-canonical.md#wave-5-2026-05-17--chrome-canonical--heroscreen-state-decomp--panel-as-canonical-pattern)).

**Zero edits** em `paint_hero_screen` ou em match arms de chrome.
Multi-agente paralelo em painéis diferentes não colidem (cada um
toca só seu módulo + 1 linha em `PANEL_REGISTRY`).

---

## 5. Disciplina de commit (anti-colisão multi-sessão)

`git commit` é serializado pelo índice global do git. Múltiplas
sessões com arquivos staged + uma roda commit = a segunda **agarra
os arquivos da primeira junto** e a mensagem fica fundida. Isso
é a colisão a evitar.

### 5.1 Protocolo atômico stage→commit

```bash
# 1) Antes de stage: confere working tree.
git status
#    Se há M/?? que NÃO são seus → PARE. Outro agente em vôo.
#    Aguarde ou reporte ao Enio.

# 2) Stage só os seus. NUNCA `-A` / `-a` / `git add .`.
git add <arquivos específicos da sua pasta>

# 3) Antes de commit: confere índice.
git status --cached
#    Se índice tem arquivo que você não estaviou → vazamento.
#    git restore --staged <não-meus>  → devolve.

# 4) Commit. Hook tiered roda automaticamente.
git commit -m "feat(<slug>): <descrição curta em inglês>"
```

Stage→commit é **uma operação contínua**. Não pause entre eles.

### 5.2 Pre-commit hook tiered

| Tier | Ativa quando | Tempo | Quando esperar |
|------|--------------|-------|----------------|
| **T0** | docs / README / `.gitignore` / scripts | ~5s | Commits de doc |
| **T1** | arquivos do seu crate (cargo nextest + clippy local) | ~30s | Commit normal seu |
| **T2** | `Cargo.toml`, multi-crate, `shells/desktop`, foundational | ~3-5min | Coordenador edits compartilhados ou bumps de dep |

Se acidentalmente trigar T2 numa pasta isolada, provavelmente
está staged junto com algo de outro agente — confira
`git status --cached`.

**Coordenador, antes de T2:** avise via Enio pra Periféricos
segurarem commits 3-5min.

### 5.3 Bypass `--no-verify` — proibido

**Nunca** use `git commit --no-verify` ou
`git commit --no-gpg-sign`. Se hook falha, **fix root cause**.

Exceção: se o Enio explicitamente pedir bypass por razão
documentada. Documente no commit message.

### 5.4 Sintomas de colisão entre sessões

| Sintoma | Recuperação |
|---------|-------------|
| `fatal: cannot lock ref 'HEAD': is at X but expected Y` | Outra sessão commitou no meio do seu. `git status` → diagnose. |
| `git status` mostra M que você não tocou | Outro agente paralelo na mesma working tree. NÃO comite. Reporte. |
| `git log -1` mostra mensagem fundida (2 títulos, 2 Co-Authored-By, corpo truncado) | Colisão. Coordenador faz `git reset --soft HEAD~1` + split. |

---

## 6. Smoke + PR + CI

### 6.1 Smoke local — antes de "pronto"

```bash
./play.command
```

Abre o editor desktop. Confira:

- App abre sem panic.
- Feature nova aparece (chrome derivado do Registry — pill na
  TopBar Image Tools, entry no LeftRail, painel flutuante, etc.).
- Click → comportamento esperado.
- **Tools/Actions pré-existentes continuam funcionando** (não
  regrediu).
- Sem regressão visível em TopBar / LeftRail / Hierarchy /
  Inspector / Widget Gallery.

**Quem roda:** o **Enio**. Periférico/Coordenador pedem o smoke;
Enio confirma.

### 6.2 PR pro GitHub — só o PRCI

Vide §3.4. Push só ao final do ciclo (batching policy). Babysit
CI até verde. Update STATE.md com novo sha bom.

### 6.3 PRCI loop em falha

CLAUDE.md §"CI/GitHub Actions":

- Loop polling com 15min (`Monitor` com `sleep 900` ou
  `gh run watch`).
- Se falha: diagnose + fix local + push + re-watch.
- Loop fecha em `success` OU 3 ciclos consecutivos de falha do
  mesmo job → escalona ao Enio.

---

## 7. Hard Rules summary (cite ao comitar quando aplicável)

| HR | Conteúdo | Onde garantida |
|----|----------|----------------|
| HR-3 | Zero-alloc no dispatcher hot-path | `interaction_dispatch_no_alloc` test |
| HR-5 | Determinism cross-platform (replay hash) | CI replay-hash matrix (3 OS) |
| HR-12 | A11y obrigatória (Role + Action) | `hr12_widgets_a11y` test |
| HR-13 | Memory budget declarado por subsistema | Manifest `memory_budget` field |
| HR-15 | Zero hardcoded UI string + zero hex color | `hr15_no_hardcoded_ui_strings` + `no_literal_color` tests |
| HR-18 | Files em `shells/<plat>/src/` ≤ 600 LOC | `file_loc_caps` test |

Hard Rules completas em
[`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md)
§HR-1..HR-18.

---

## 8. Quando algo dá errado

| Sintoma | Resposta |
|---------|----------|
| Você não sabe o que fazer | Releia §0 + pergunte ao Enio |
| Arquivo que vc não tocou aparece em `git status` | §5.4 (colisão entre sessões) |
| Hook falha em fmt / clippy / nextest | Fix root cause; nunca `--no-verify` |
| Hook trigga T2 quando vc esperava T1 | §5.1 #3 — `git status --cached` pra ver vazamento |
| Smoke quebrou no `./play.command` | Reporte ao Enio com sintoma; agente Periférico diagnostica + fix local |
| CI failure cíclico (mesma job 3× seguidas) | PRCI escalona ao Enio |
| Você é Periférico e descobre bug fora da sua pasta | Reporte ao Enio com diagnose; Coordenador decide quem fix |
| Você é Coordenador e quer editar shared mas Periférico está working | Anuncie via Enio, espere Periférico chegar a stable, então edite |
| Você é Coordenador e tem que decidir entre 2 opções arquiteturais | Apresente as opções ao Enio com recomendação + tradeoff |
| Você é qualquer agente e a memória diz X mas o código diz Y | Confie no código. Memórias podem decair. Atualize memória depois. |

---

## 9. Referências canônicas

- **Narrativa do problema multi-agente:** [`docs/Migracao/PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md`](../Migracao/PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md)
- **Hard Rules + arquitetura:** [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md)
- **Operacional dia-a-dia + CI:** [`CLAUDE.md`](../../CLAUDE.md)
- **ADR convention-by-discovery (Wave 1):** [`docs/architecture/decisions/0027-convention-by-discovery.md`](../architecture/decisions/0027-convention-by-discovery.md)
- **ADR codegen + design canonical (Wave 2):** [`docs/architecture/decisions/0028-wave-2-codegen-design-canonical.md`](../architecture/decisions/0028-wave-2-codegen-design-canonical.md)
- **STATE operacional:** [`STATE.md`](STATE.md)
- **Memory (auto-loaded):** `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`
- **Wave 2.5 plan (parcialmente fechada):** [`docs/Migracao/2026-05-wave-2-5-deferred-splits.md`](../Migracao/2026-05-wave-2-5-deferred-splits.md)
- **Wave 3 plan (parcialmente fechada):** [`docs/Migracao/2026-05-wave-3-deferred-state-decomp-and-golden-images.md`](../Migracao/2026-05-wave-3-deferred-state-decomp-and-golden-images.md)

---

## 10. Versão + histórico

- **5.0 — 2026-05-17:** Diretriz unificada. Substitui 01-04 docs
  separados. Reflete arquitetura pós Wave 1-3.2. Wave 4 (source-
  of-truth UI) e Wave 5 (panel-as-canonical-source) referenciados
  como pendentes.
- **4.0 — 2026-05-13** (em `ARCHIVE-v4.0-pre-wave-1/`): modelo
  Coordenador edita manualmente icons.rs / fixture.rs / ids.rs /
  ToolRegistry::new. Arquitetura morta pós Wave 1.

---

## 11. Última nota — quando esta DIRETRIZ está obsoleta

Se a arquitetura mudar materialmente (e.g., panel-as-canonical-source
lança em Wave 5, mudando como painéis são adicionados), atualize
esta DIRETRIZ in-place e bump a versão. Não fragmente em múltiplos
docs — a lição dos 4 docs antigos (que ficaram dessincronizados)
é que um doc único é mais fácil de manter atualizado.

Se você é LLM lendo isso depois de uma mudança arquitetural maior
e a DIRETRIZ contradiz o código, **confie no código**, reporte ao
Enio com diagnose, e atualize esta DIRETRIZ quando autorizado.
