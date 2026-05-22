# Diretriz de Implementação Universal — PH2D

**Versão:** 6.5 — 2026-05-22 (arquitetura node-centric: §3.8 novo balde "Node crate (fan-out)" apontando pro `briefing-node-crate.md` — o fan-out mais isolado que existe, pois o wiring central é GERADO por `ph2d-node-sync`, sem edit central; §3.6 foundational ganha `ph2d-nodegraph`+`ph2d-expr` congelados (ADR-0039, W2.T4); TL;DR nota o sistema de nós como caminho principal de crescimento. Os baldes de editor — Tool/Painel/Widget/Chrome — seguem válidos: são o chrome do editor, que edita os grafos de nós.) · 6.4 — 2026-05-21 (§4.1 novo: regras de UI sem gate automático que já queimaram >1× — glifos fora da fonte bundled viram tofu; estado de modo ↔ estado derivado não pode desacoplar (reconciliação por frame, não guard pontual); enumere TODOS os caminhos de ativação (pill/palette/atalho/bus); pertencimento data-driven via cluster do manifest, não lista de ids; diagnostique medindo com env-probe. Bases: `docs/UI_Bugs/` + `docs/Image Tools Bugs/`.) · 6.3 — 2026-05-20 (§7.0 novo: fluxo fast-mode/ship — de dia `git commit --no-verify` sem push/CI; no fim do dia `./scripts/ship.sh` (paridade-CI completa) → fix loop → commit → push → babysit, modo observa-e-corrige, entrega sem falta. §7.2 troca a matriz manual incompleta pelo ship.sh. v6.2: §5 corrigida — o perf audit cortou `--all-targets` do hook T2 workspace, mas o CI ainda exige — documentado o gap hook≠CI + regra "rode o comando exato do CI antes do push". v6.1: perf audit acrescentou §3.7 cross-cutting, §5.6 test slow, §6.4 armadilhas; tabela T2 cortes A+B)
**Substitui:** `01-Enio.md` · `02-Coordenador.md` · `03-Agente-Periferico.md` · `04-Agente-PRCI.md` · DIRETRIZ v5.0 · `STATE.md` · `DIRETRIZ_CODIFICACAO_RAPIDA.md` · `Migracao/PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md`
**Audiência:** **toda LLM que entra no projeto.** Você lê este doc inteiro antes de tocar em código. Depois, Enio te diz "Você é o Coordenador" ou "Você é o Implementador" e este doc te diz o resto.

---

## TL;DR

> **Dois papéis. Fluxo invertido. Zero colisão por construção.**
>
> 1. Enio diz "vamos criar X" ao **Coordenador**.
> 2. Coordenador **faz primeiro todos os links centrais** (pasta nova, Cargo.toml, register_all, TOML de design, SVG, stubs prontos). Valida com `cargo check -p <crate-novo>`. Pasta já está plugada na árvore.
> 3. Coordenador entrega briefing pro **Implementador**: "tua pasta é `crates/ph2d-tool-<slug>/`. Stubs prontos. Não saia daí."
> 4. Implementador preenche **apenas dentro da pasta isolada**. Nunca toca arquivo fora. Usa tokens canônicos. Obedece codificação rápida.
> 5. Coordenador revisa, faz smoke com Enio, commita, faz push, babysit CI.
>
> **Enio não decide nada operacional.** Coordenador instrui Enio passo a passo. Enio é relay mecânico entre as duas sessões Claude Code.
>
> **Caminho principal de crescimento = sistema de nós (fan-out, §3.8).** A engine virou node-centric (ADR-0030..0039): adicionar feature de conteúdo = largar um node-crate isolado, e até o wiring central é gerado (`ph2d-node-sync`) — o fan-out mais leve do projeto. Os baldes de editor (Tool/Painel/Widget/Chrome, §3.1-3.4) seguem para o chrome do editor que edita esses grafos.

---

## 0. Antes de começar (sanity check obrigatório)

Independentemente do papel que vai assumir, rode primeiro:

```bash
git log --oneline -5                              # confirma HEAD
git status -sb                                    # working tree clean?
cargo check --workspace 2>&1 | tail -5            # baseline compila?
```

Se algo divergir do esperado (HEAD inesperado, working tree suja, build quebrado), **pare e reporte ao Enio** antes de qualquer ação.

Leitura mínima de contexto técnico:
- [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §HR-1..HR-18 (Hard Rules) e §1 (arquitetura).
- [`CLAUDE.md`](../../CLAUDE.md) (CI, push, batching).
- Memória persistente do LLM em `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md` (perfil do Enio, feedback acumulado).

---

## 1. O modelo

### 1.1 Os dois papéis

**Coordenador** — única autoridade global. Centraliza tudo que é "arquivo compartilhado". Faz scaffold de feature nova *antes* do Implementador começar. Revisa entrega. Commita. Faz push. Babysit CI. Absorve o papel que antes era PRCI.

**Implementador** — sessão isolada, uma por feature. Recebe pasta já plugada na árvore. Edita **somente** dentro dela. Reporta pronto. Pode rodar em paralelo com outros Implementadores sem coordenação direta — a arquitetura física garante que eles não colidem.

Enio não é papel. Enio é o humano que orquestra: abre sessões Claude Code, cola mensagens entre elas, roda smoke visual quando o Coordenador pede.

### 1.2 Fluxo invertido (a mudança fundamental)

O modelo antigo era: Implementador cria a tool → Coordenador depois faz os links centrais. **Errado.** O Implementador editava arquivos fora da pasta dele pra plugar a tool, e dois Implementadores em paralelo colidiam.

Modelo novo: **scaffolding primeiro, conteúdo depois.**

```
Enio: "Coordenador, vamos criar a tool Transform."
   │
   ▼
Coordenador:
  1. Decide: é uma tool? painel? widget? (vide §3)
  2. Cria crates/ph2d-tool-transform/ inteira com stubs
  3. Adiciona em Cargo.toml raiz (members)
  4. Adiciona linha em ph2d-tool-registry-init::register_all (alfabético)
  5. Cria docs/design/tools/transform.toml
  6. Dropa docs/design/icons/transform.svg (ou pede ao Enio)
  7. cargo check -p ph2d-tool-transform  →  verde
  8. Briefing pro Implementador (vide §2)
   │
   ▼  (Enio abre nova sessão Claude Code, cola briefing)
   │
Implementador:
  9. Lê briefing + esta DIRETRIZ + sanity check
 10. Edita SÓ dentro de crates/ph2d-tool-transform/
 11. Preenche algorithm.rs, icon.rs, manifest, register
 12. cargo test -p ph2d-tool-transform  →  verde
 13. Commit local, reporta pronto
   │
   ▼
Coordenador:
 14. Revisa o que foi commitado
 15. Pede smoke pro Enio (./play.command)
 16. Se OK, push origin main + babysit CI
 17. CI verde → Enio livre pra próxima feature
```

A partir do passo 8 o Implementador **nunca** toca arquivo fora da pasta dele. Zero risco de colisão com outro Implementador que esteja trabalhando em outra feature em paralelo.

### 1.3 As 3 obrigações do Implementador (sempre, sem exceção)

1. **ISOLAMENTO.** Edita **só** arquivos dentro da pasta exclusiva atribuída pelo Coordenador. Se precisa de algo fora (dep nova, mudança em foundational, novo NodeId), **reporta** ao Enio — não edita por conta própria.
2. **UI canônica.** Toda cor, espaçamento, raio, tipografia, stroke **passa por tokens** (`ColorToken::X.resolve(theme)`, `Spacing::Lg.px()`, etc.). Zero hex, zero `f32` literal de UI. Vide §4.
3. **Codificação rápida.** Usa `cargo check -p <crate>` durante editing burst. Não duplica trabalho do pre-commit hook. Não roda `--workspace` em loop. Vide §5.

Se você é o Implementador e está pra violar uma das três, **pare e reporte**. Quase certamente significa que o Coordenador não fez o scaffold direito.

---

## 2. Como Coordenador e Implementador se comunicam

Enio é **relay mecânico**, não decisor.

### 2.1 Coordenador → Implementador

Quando o Coordenador precisa que o Implementador comece, ele entrega ao Enio um briefing pronto-pra-colar, com este formato exato:

```
═══════════════════════════════════════════════════════════════════
BRIEFING — IMPLEMENTADOR · slot <N> · feature: <slug>
═══════════════════════════════════════════════════════════════════

PASTA EXCLUSIVA: crates/ph2d-tool-<slug>/

ESTADO INICIAL (já plugado na árvore pelo Coordenador):
- Cargo.toml      (workspace deps prontos)
- src/lib.rs      (MANIFEST + register stubs)
- src/icon.rs     (BezPath placeholder)
- src/algorithm.rs (assinatura pronta, corpo vazio)
- docs/design/tools/<slug>.toml (spec funcional)
- docs/design/icons/<slug>.svg (ícone source)

O QUE VOCÊ FAZ:
- Preenche src/algorithm.rs com a lógica pura + unit tests
- Preenche src/icon.rs com o BezPath final (porte do SVG)
- Ajusta o MANIFEST se a spec do TOML pedir
- cargo test -p ph2d-tool-<slug> verde

O QUE VOCÊ NÃO TOCA (em hipótese alguma):
- Qualquer arquivo fora de crates/ph2d-tool-<slug>/
- Cargo.toml raiz (já tem você como member)
- ph2d-tool-registry-init/* (já tem sua linha em register_all)
- crates/ph2d-tokens/* (foundational)
- crates/ph2d-editor-core/* (foundational)
- shells/*

LEIA ANTES DE CODAR:
- docs/IntegracaoMultiAgente/DIRETRIZ.md §1.3 (3 obrigações)
- docs/IntegracaoMultiAgente/DIRETRIZ.md §4 (UI canônica)
- docs/IntegracaoMultiAgente/DIRETRIZ.md §5 (codificação rápida)

QUANDO TERMINAR:
Reporte ao Enio nesta forma EXATA:
  "Implementador slot <N> pronto. Commit local: <sha>. Aguardando revisão."
═══════════════════════════════════════════════════════════════════
```

Enio copia esse bloco e cola numa sessão Claude Code nova. Implementador lê + executa.

### 2.2 Implementador → Coordenador

Quando Implementador termina, reporta ao Enio com a frase ritual:

> "Implementador slot N pronto. Commit local: `<sha>`. Aguardando revisão."

Enio cola essa frase na sessão do Coordenador. Coordenador faz `git log --oneline -3`, lê o diff, faz seus checks.

### 2.3 Coordenador → Enio (instruções operacionais)

Coordenador instrui Enio mecanicamente:

- "Enio, rode `./play.command` e me diga se a tool Transform aparece na TopBar com o ícone correto."
- "Enio, abra uma sessão Claude Code nova e cole o briefing abaixo."
- "Enio, confirma que posso pushar? Estou pronto."

Enio executa sem decidir. Se Enio precisa decidir algo (escolher entre 2 abordagens, definir escopo), o Coordenador apresenta opções com recomendação primeiro, conforme preferência registrada em memória.

### 2.4 Implementador → Coordenador (pedidos de fora-da-pasta)

Se Implementador descobre que precisa:

- Adicionar dep externa em `Cargo.toml` workspace
- Mudar algo em `ph2d-core`, `ph2d-tokens`, `ph2d-editor-core` (foundational)
- Criar um NodeId novo, alocar um cluster id, etc.
- Adicionar widget primitive novo que outras tools vão consumir

**Para imediatamente.** Reporta ao Enio:

> "Implementador slot N bloqueado. Preciso que o Coordenador faça: <descrição precisa>. Razão: <por quê>. Continuo após scaffold extra."

Enio cola pro Coordenador. Coordenador faz o scaffold adicional, comita, libera o Implementador.

---

## 3. Receitas canônicas — o que o Coordenador faz para cada tipo de pedido

Cinco buckets. O Coordenador classifica o pedido antes de fazer qualquer coisa.

### 3.1 Tool nova ("vamos criar a tool Transform")

**Coordenador (scaffold):**

1. Decide `slug` (`transform`), `cluster` (`image_tools` / `selection` / etc.), `zone` (`top_right` / `left_rail` / etc.), `order` numérico.
2. Cria `crates/ph2d-tool-transform/Cargo.toml`:
   ```toml
   [package]
   name = "ph2d-tool-transform"
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
3. Cria `src/lib.rs` com stubs:
   ```rust
   #![forbid(unsafe_code)]
   //! ph2d-tool-transform — placeholder

   use ph2d_a11y::Role;
   use ph2d_core::MemoryBudget;
   use ph2d_tool_registry::{
       BezPath, HandlerFn, McpExposure, Registry, ToolHandler,
       ToolManifest, Zone,
   };

   pub mod icon;
   pub mod algorithm;

   fn shadow_handler() {}

   pub const MANIFEST: ToolManifest = ToolManifest {
       id:           "transform",
       label_key:    "tool.transform.label",
       icon_fn:      icon::transform_bezpath,
       zone:         Zone::TopRight,
       cluster:      "image_tools",
       order:        50,
       a11y_role:    Role::Button,
       handler:      ToolHandler::OneShot { on_click: shadow_handler as HandlerFn },
       memory_budget: MemoryBudget::new(0, 0, 0),
       touches_sim:  false,
       mcp:          McpExposure::reserved(),
   };

   pub fn register(reg: &mut Registry) { reg.register(&MANIFEST); }
   ```
4. Cria `src/icon.rs` com BezPath placeholder (1 retângulo, será trocado pelo Implementador):
   ```rust
   use ph2d_tool_registry::BezPath;
   pub fn transform_bezpath() -> BezPath {
       let mut p = BezPath::new();
       p.move_to((4.0, 4.0)); p.line_to((20.0, 4.0));
       p.line_to((20.0, 20.0)); p.line_to((4.0, 20.0)); p.close_path();
       p
   }
   ```
5. Cria `src/algorithm.rs` com assinatura vazia.
6. Adiciona em `Cargo.toml` raiz (`members = [..., "crates/ph2d-tool-transform"]`).
7. Adiciona em `crates/ph2d-tool-registry-init/Cargo.toml` deps:
   ```toml
   ph2d-tool-transform = { path = "../ph2d-tool-transform" }
   ```
8. Adiciona em `crates/ph2d-tool-registry-init/src/lib.rs::register_all` (**em ordem alfabética** — o arch test `architecture_register_all_alphabetical` enforça):
   ```rust
   ph2d_tool_transform::register(reg);
   ```
9. Cria `docs/design/tools/transform.toml` com a spec funcional:
   ```toml
   [tool]
   id          = "transform"
   cluster     = "image_tools"
   zone        = "top_right"
   order       = 50
   a11y_role   = "Button"
   icon_slug   = "transform"
   touches_sim = false

   [label]
   fluent_key   = "tool.transform.label"
   pt_br_inline = "Transform"
   en_us_inline = "Transform"

   [memory_budget]
   vram_mb        = 0
   ram_mb         = 0
   heap_script_mb = 0
   ```
10. Cria `docs/design/icons/transform.svg` (Lucide-style 24×24, `stroke="currentColor"`). Se não tem source, pede ao Enio.
11. Roda `cargo check -p ph2d-tool-transform` — deve compilar verde.
12. Roda `cargo test --workspace --exclude ph2d-asset` rápido — arch tests devem passar (alphabetical, design-sync, etc).
13. Commita: `chore(coord): scaffold ph2d-tool-transform (slot N)`.
14. Entrega briefing pro Enio (vide §2.1).

**Implementador:** preenche `algorithm.rs` com lógica pura + tests, troca `icon.rs` pelo BezPath real do SVG, ajusta MANIFEST se TOML pediu detalhe específico. `cargo test -p ph2d-tool-transform` verde. Reporta.

### 3.2 Painel novo ("vamos criar o painel Outline")

**Coordenador (scaffold):**

1. Decide `slug` (`outline`), `default_visible`, feature flag (`panel-outline`).
2. Cria crate `crates/ph2d-panel-outline/` com `Cargo.toml` (deps em `ph2d-editor-core`, `ph2d-a11y`, `ph2d-tokens`, `ph2d-text`, `ph2d-vector`).
3. Cria `src/lib.rs` com stub do `impl Panel`:
   ```rust
   use ph2d_editor_core::panel::{Panel, PaintCtx, PanelHost};
   use ph2d_a11y::NodeId;

   pub struct OutlinePanel;

   #[derive(Default)]
   pub struct OutlineState { /* placeholder */ }

   impl Panel for OutlinePanel {
       type State = OutlineState;
       const ID: &'static str = "outline";
       const NODE_ID: NodeId = ph2d_a11y::hash_node_id("panel.outline");
       const DEFAULT_VISIBLE: bool = false;

       fn paint(_state: &mut Self::State, _ctx: &mut PaintCtx, _host: &mut dyn PanelHost) {
           // placeholder
       }
       // demais métodos com stubs
   }
   ```
4. Adiciona feature em `crates/ph2d-panel-registry-init/Cargo.toml`:
   ```toml
   [features]
   default = ["panel-inspector", "panel-hierarchy", "panel-widget-gallery", "panel-grid-snap", "panel-outline"]
   panel-outline = ["dep:ph2d-panel-outline"]
   ```
5. Adiciona em `crates/ph2d-panel-registry-init/src/lib.rs::register_all_panels` (alfabético):
   ```rust
   #[cfg(feature = "panel-outline")]
   build.register::<ph2d_panel_outline::OutlinePanel>();
   ```
6. Adiciona member em `Cargo.toml` raiz.
7. `cargo check -p ph2d-panel-outline` + `cargo check -p ph2d-panel-registry-init` verde.
8. Commita + entrega briefing.

**Implementador:** preenche state, paint, apply_event, populate.

### 3.3 Widget primitive novo ("precisamos de um ColorWheel")

Widget primitive é "elemento de UI reutilizável" (botão, slider, dropdown, etc.) — vive em `crates/ph2d-editor-core/src/widget/`. Diferente de tool/painel: **é foundational**, consumido por vários painéis. Coordenador faz o scaffold completo e entrega ao Implementador apenas se for grande o suficiente; caso contrário Coordenador faz inteiro.

**Coordenador (scaffold):**

1. Cria `crates/ph2d-editor-core/src/widget/<slug>.rs` com pattern canônico (vide [`button.rs`](../../crates/ph2d-editor-core/src/widget/button.rs) como template):
   ```rust
   //! <one-liner>
   use crate::interaction::HitIndex;
   use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
   use ph2d_text::TextSystem;
   use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
   use ph2d_vector::VectorScene;
   use crate::paint::resolve;
   use crate::zones::Rect;

   #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
   pub enum ColorWheelState {
       #[default] Normal, Hovered, Pressed, Focused, Disabled,
   }

   #[derive(Clone, Debug)]
   pub struct ColorWheel { pub id: NodeId, /* … */ }

   impl ColorWheel {
       pub fn build_a11y(&self) -> Node {
           NodeBuilder::new(Role::Slider).build()
       }
   }

   pub fn paint_color_wheel(/* args canônicos */) { /* stub */ }
   ```
2. Adiciona em `widget/mod.rs` na ordem alfabética:
   ```rust
   mod color_wheel;
   pub use color_wheel::{ColorWheel, ColorWheelState, paint_color_wheel};
   ```
3. Cria seção no showcase em `widget/showcase/` (copia layout de [`switches.rs`](../../crates/ph2d-editor-core/src/widget/showcase/switches.rs)). Arch test `architecture_widget_showcase_coverage` enforça que o widget aparece no showcase OU em opt-out justificado.
4. `cargo check -p ph2d-editor-core` verde. Os 4 arch tests de widget devem passar:
   - `architecture_widget_loc_cap` (≤500 LOC)
   - `architecture_widget_showcase_coverage`
   - `no_literal_color`
   - `hr12_widgets_a11y`
5. Commita + briefing (se entregar a Implementador).

**Implementador:** preenche `paint_color_wheel` usando **só tokens** (zero hex, zero px literal), adiciona unit tests, ajusta seção do showcase.

### 3.4 TopBar action / chrome affordance ("adicionar botão Settings → Snap toggle")

Chrome (TopBar, LeftRail, BottomHUD, ContextMenu) tem handlers em [`crates/ph2d-editor-core/src/screens/hero/chrome/`](../../crates/ph2d-editor-core/src/screens/hero/chrome/). Cada handler = 1 arquivo (≤80 LOC típico) com função `pub fn apply(hero, event) -> bool`. Dispatcher central em `chrome/mod.rs`.

**Coordenador:**

1. Cria `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` com stub:
   ```rust
   use crate::interaction::WidgetEvent;
   use crate::screens::hero::HeroScreen;

   pub fn apply(_hero: &mut HeroScreen, _event: WidgetEvent) -> bool {
       false  // implementador preenche
   }
   ```
2. Adiciona no dispatcher em `chrome/mod.rs` (ordem alfabética):
   ```rust
   pub mod snap_toggle;
   ```
   E na fn `dispatch_all` adiciona `|| snap_toggle::apply(hero, event)`.
3. Se o handler precisa de NodeIds novos, adiciona em `screens/hero/ids.rs` via `hash_node_id`.
4. Se precisa item de menu/popover, adiciona em `pre_populate.rs` (ou pede ao Implementador).
5. `cargo check -p ph2d-editor-core` verde. Briefing.

**Implementador:** preenche corpo do handler.

### 3.5 Modificar feature existente ("o ícone da tool Trim está errado")

Não há scaffold. A pasta já existe. Coordenador delega direto ao Implementador:

```
Implementador slot N: edite crates/ph2d-tool-trim-transparency/src/icon.rs.
Não toque em nada fora. Reporte quando terminar.
```

### 3.6 Mudança em foundational ("precisamos de um token de cor novo")

Foundational = `ph2d-core`, `ph2d-tokens`, `ph2d-editor-core`, `ph2d-a11y`, `ph2d-host`, `ph2d-vector`, `ph2d-text`, `ph2d-tool-registry`, `ph2d-panel-registry-init`, `shells/*`, arch tests, **+ o contrato de nós `ph2d-nodegraph` + `ph2d-expr` (🔒 CONGELADOS em W2.T4, ADR-0039)**.

**Foundational não é paralelizável.** Coordenador faz **sozinho**. Não delega.

**O contrato de nós é caso especial — congelado.** Mexer em `ph2d-nodegraph`/`ph2d-expr` não é só foundational, é evento **raro Coordenador-only com ADR**: os caps do arch-gate (`architecture_contract_surface.rs`) estão apertados ao tamanho atual, então qualquer crescimento tripa o gate de propósito. Para mudar: bump do cap + ADR novo + (no `ph2d-expr`) re-provar a paridade CPU↔WGSL. Adicionar um NÓ NÃO mexe no contrato — é fan-out (§3.8).

Exemplo: adicionar `ColorToken::AccentTeal`:
1. Coord edita `docs/design/tokens.json` adicionando a chave em todos os 4 temas.
2. Coord roda `cargo check -p ph2d-tokens` (build.rs regenera).
3. Coord edita `crates/ph2d-tokens/src/color.rs` adicionando o variant.
4. Coord roda `cargo test --workspace --exclude ph2d-asset` pra garantir nada quebrou.
5. Coord commita: `feat(tokens): add ColorToken::AccentTeal`.

### 3.7 Trabalho cross-cutting (perf audit, refactor cross-crate, manutenção de tests)

Algumas tarefas não cabem em nenhum dos 6 buckets acima porque tocam múltiplos crates por natureza — perf audit do workspace, deduplicação de pattern, migração de API antiga em N crates consumidores, sweep de lint novo, etc.

**Coordenador autoriza explicitamente a exceção ao isolamento.** O briefing pro Implementador diz literalmente:

> "Você toca tests em vários crates conforme os achados. Exceção autorizada à regra de uma pasta isolada da DIRETRIZ §1.3. Cada commit ainda fica T1 single-crate sempre que possível."

**Regras desse bucket:**

1. **Cada commit valida-se sozinho** — `cargo test -p <crate>` verde para o crate tocado, antes do commit. Pre-commit hook entra em T1 (single-crate, ~30s), não T2.
2. **Não tocar production code de foundational sem motivo claro** — em audit de tests, mexer só nos tests (`tests/`, `#[cfg(test)] mod`). Production `pub fn` fica intocado salvo se a auditoria revelar API faltando (e nesse caso vira novo trabalho discutido com Enio).
3. **Documentar risk surface** — no relatório final, listar todas as mudanças sutis de comportamento que CI pode capturar mas smoke local pode não ver (ex: "função X agora cacheia via `OnceLock` — primeiro caller paga, demais zero-init; tests confirmados que só fazem `&ctx` imutável").
4. **Tokens canônicos continuam valendo** — geralmente N/A em test code, mas se tocar paint/widget, mesma regra de §4.

**Exemplo real (2026-05-19 noite):** perf audit cortou nextest workspace de 14min para 1.5min via 6 commits, cada um T1 single-crate, todos verdes em CI. Detalhe na memória `project_perf_audit_2026_05_19.md`.

### 3.8 Node crate novo — fan-out (o caminho principal de crescimento)

A engine é node-centric (ADR-0030..0039). Uma feature de **conteúdo** (um gerador, um modifier, um cloner, um nó de shader/som/gameplay) é um **node-crate isolado** — e este é o **fan-out mais leve do projeto**, porque o acoplamento é só o contrato congelado e o wiring central é **gerado**, não editado.

**Diferença-chave vs. os baldes de editor (§3.1-3.4):** lá o Coordenador edita um `register_all` central na mão. Aqui **não há edit central** — `cargo run -p ph2d-node-sync` regenera `register_all_nodes` + as deps de `ph2d-node-registry-init` a partir de um scan das pastas `crates/ph2d-node-*`, e o `workspace.members` é glob. Dois agentes adicionando dois nós **não tocam nenhum arquivo em comum, nem o central**.

**O briefing canônico já existe e é pronto-pra-colar:** [`docs/IntegracaoMultiAgente/briefing-node-crate.md`](briefing-node-crate.md). O Coordenador o entrega (preenchendo domínio/slug/spec); o Implementador cria só a sua pasta `crates/ph2d-node-<domínio>-<slug>/`, roda o sync, e fecha o gate local `cargo test -p ph2d-node-registry-init`. Exemplo a copiar: `crates/ph2d-node-debug-wave/` (template) e a vertical Motion `crates/ph2d-node-motion-{grid,transform,clone}/`.

**Regras específicas do balde de nó (somam às 3 obrigações do §1.3):**
1. **Não tocar o contrato congelado** (`ph2d-nodegraph`/`ph2d-expr`, §3.6) — se você acha que precisa, pare e reporte; quase sempre é sinal de que o nó está modelado errado. Mudança de contrato é evento Coordenador-only com ADR.
2. **Ler params via `ctx.param("nome")`** — nunca o default do manifest direto, nunca `unwrap_or(0.0)`. Param que vira contagem/alocação passa por `param_as_count` + cap (vide `motion.grid`/`clone`).
3. **Efeito + membrana:** se o nó não escreve estado de jogo, é `Pure`/`Temporal` (lado pull, isento de HR-5); `Stateful` é só gameplay. `Graph::validate` é quem prova a membrana — rode-o nos testes.
4. **Teste golden** input→output (ADR-0031 §3): grafo source→seu-nó → registra → `validate` → `cook` → asserta a saída.

Estado vivo do sistema de nós + o loop de operação autônoma: [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md).

---

## 4. UI canonical — única fonte de verdade

Tudo de UI passa por tokens. Sem exceção. A cadeia é:

```
docs/design/tokens.json   (designer edita; 4 temas; OKLCH para cores)
        │
        │  (build.rs em ph2d-tokens lê e regenera)
        ▼
crates/ph2d-tokens/src/   (5 enums semânticos)
   ├─ ColorToken          (33 variants — Bg0..Bg3, Text1..3, Accent*, Danger, Selection, etc.)
   ├─ Spacing             (12 variants — Xxs..Xl4)
   ├─ Radius              (7 variants — Xs..Full)
   ├─ TypeToken           (9 sizes + FontWeight + LineHeight + LetterSpacing)
   └─ StrokeToken         (5 variants — Hairline..Heavy)
        │
        │  (widget code consome)
        ▼
let bg = ColorToken::Bg2.resolve(theme);
let pad = Spacing::Lg.px();
let r   = Radius::Md.px();
```

CSS aliases em [`docs/design/styles/tokens.css`](../../docs/design/styles/tokens.css) servem para mockups HTML — o arch test `mockup_tokens_exist` garante que toda `var(--X)` em mockup resolve em `tokens.json` ou em alias CSS.

**Gates ativos que enforçam o canônico:**

| Arch test | O que barra |
|-----------|-------------|
| [`no_literal_color`](../../crates/ph2d-editor-core/tests/no_literal_color.rs) | hex `0xRRGGBB`, `Color::rgba8(...)`, `Color::WHITE`, etc. em `widget/` ou `screens/` |
| `no_magic_numeric` | `f32`/`f64` literais em UI fora do allowlist estrutural (`0.0`, `±0.5`, `±1.0`, `±2.0`) |
| [`hr12_widgets_a11y`](../../crates/ph2d-editor-core/tests/hr12_widgets_a11y.rs) | widget que não emite `Node` AccessKit |
| [`architecture_widget_loc_cap`](../../crates/ph2d-editor-core/tests/architecture_widget_loc_cap.rs) | widget primitive > 500 LOC |
| [`architecture_widget_showcase_coverage`](../../crates/ph2d-editor-core/tests/architecture_widget_showcase_coverage.rs) | widget existe mas não aparece no Widget Gallery showcase nem em opt-out |
| [`mockup_tokens_exist`](../../crates/ph2d-tokens/tests/mockup_tokens_exist.rs) | `var(--X)` em mockup HTML que não resolve em tokens.json/styles |
| [`architecture_register_all_alphabetical`](../../crates/ph2d-tool-registry-init/tests/architecture_register_all_alphabetical.rs) | `register_all` ou Cargo deps fora da ordem alfabética |
| [`architecture_panel_host_surface`](../../crates/ph2d-editor-core/tests/architecture_panel_host_surface.rs) | `PanelHost` cresce além de 12 métodos |
| [`architecture_cycle_prevention`](../../crates/ph2d-editor-core/tests/architecture_cycle_prevention.rs) | `editor-core` ganha dep em `panel-*` ou `ph2d-editor` |
| `tool_manifest_design_sync` | `docs/design/tools/<slug>.toml` divergente do `MANIFEST` const |
| `node_id_collisions` | dois NodeIds chrome colidem |

Violação em qualquer um = build vermelho = Implementador refaz. Não há "vou abrir exceção".

**Exceção declarada legítima:** comentário `// LITERAL-COLOR-OK: <razão>` na mesma linha ou `// LITERAL-PX-OK: <razão>` para magic numeric. Use sparingly — Coordenador valida na revisão se a justificativa procede.

### 4.1 Regras de UI que já queimaram (NÃO repita)

Erros recorrentes que voltaram >1 vez. A regra 1 agora TEM gate automático
(arch-test); as demais (2.x/3) ainda dependem de disciplina + revisão.
Bases de conhecimento: [`docs/UI_Bugs/README.md`](../UI_Bugs/README.md) (UI geral) e [`docs/Image Tools Bugs/README.md`](../Image%20Tools%20Bugs/README.md) (Image Tools). **Leia antes de tocar em painter/dispatch/tool.**

1. **Nenhum glifo fora da fonte bundled (Inter) em string de UI.** Seta/símbolo (`→ ⌘ ↵ ✕ ▸ …`) vira **tofu** (quadrado). Vale pra TODA string visível: toast, tooltip, label, pill. Use ASCII; o único não-ASCII seguro comprovado é `·` (U+00B7). Já queimou 3×: glifos Cmd/Return do topbar (UI_Bugs §9.19), a seta das toasts "Tool → X" (`b62e0c5`) e mais 27 ocorrências varridas no sweep final. **GATE:** [`crates/ph2d-editor-core/tests/no_tofu_glyphs.rs`](../../crates/ph2d-editor-core/tests/no_tofu_glyphs.rs) varre editor-core + shell e barra glifos dos blocos arrow (U+2190–21FF) e technical-symbols (U+2300–23FF) dentro de string literal (ignora comentários). CI vermelho se reincidir.

2. **Estado de MODO e estado DERIVADO não podem viver desacoplados.** Se um toggle/modo de UI (ex.: `image_edit.mode_on`) governa o que aparece (tool ativo, painel, preview), quem desliga o modo é responsável por **desligar tudo** que ele expõe. O lugar certo é uma **reconciliação por frame** sobre estado derivado — **não** um *guard* pontual no caminho de clique. Guard trata "não deixar ligar"; **não** trata "já está ligado". Bug clássico: ferramenta de imagem (Bg Removal/Padding) seguia ativa com Image Tools desligado, painel/preview órfãos persistindo (Image Tools Bugs §2, `3ef9190`). **Implementador:** ao adicionar um modo que liga subsistemas, escreva também a reconciliação que os desliga quando o modo cai, e teste o ciclo liga→usa→desliga.

   2.a **Enumere TODOS os caminhos de ativação.** Uma feature costuma ter >1 via de ligar (pill da TopBar, **tool palette**, atalho de teclado, bus action). Gatear só uma deixa o bug vivo pelas outras — foi exatamente o que aconteceu: pills gateadas, mas a tool palette chamava `set_active` direto e ressuscitava o image tool com o modo off (Image Tools Bugs §2.b, `32460b9`). Faça grep de TODOS os `set_active`/push de action do subsistema e cubra cada um, ou centralize.

   2.c **Hit-test e paint do MESMO widget têm que ser gateados pela MESMA condição.** Um hit-test que roda onde o widget NÃO é pintado = **zona de clique invisível**. Caso real: a tool palette só pinta no caminho demo, mas o hit rodava sempre, sobrepondo o gear de Config no editor — clicar no Config trocava o tool silenciosamente (Image Tools Bugs §2.b, `952dc0c`). Sempre que um widget é condicional, condicione paint E hit juntos (idealmente pela mesma flag/expressão).

   2.b **Pertencimento é data-driven, não lista de ids hardcoded.** "É um image tool?" = está no cluster `"image_tools"` do manifest, resolvido por UM helper (`is_image_edit_tool`) — não `id == "x" || id == "y"` espalhado por N sites. Assim toda tool futura do grupo é coberta de graça e não há lista pra dessincronizar.

3. **Diagnostique medindo, não chutando.** Bug de UI/input com repro: instrumente (env-gate, ex.: `PH2D_UIDBG`) o caminho exato e capture o estado real (id resolvido no hit, flags, frame) antes de propor fix. Reverta a instrumentação no fim. Duas "correções" às cegas do bug do Image Tools falharam por chutar (uma chegou a mover o Config — proibido); a 3ª resolveu medindo.

---

## 5. Codificação rápida

Princípio: **não duplique o pre-commit hook** durante o editing burst. PORÉM o **hook ≠ CI** em dois pontos que já queimaram runs vermelhas — e antes do **push** (não a cada commit) é obrigação do Coordenador fechar esse gap:

1. **clippy `--all-targets`:** o tier **T2 workspace** roda `cargo clippy --workspace -- -D warnings` **SEM `--all-targets`** (cortado no perf audit 2026-05-19 por velocidade). Logo, **lints em código de teste (`#[cfg(test)]`) e atrás de feature NÃO são pegos localmente numa mudança multi-crate.** O CI roda o comando completo (`spike.yml`): `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings`. (O tier T1, 1 crate isolado, ainda roda `--all-targets` — só o T2 workspace não.) **NÃO** use `--all-features` pra "verificar" — ela liga o path flecs do spike (`c11_flecs`) que o CI nem linta (falso-positivo).
2. **arch-gates:** mudança estrutural (novo arquivo de widget, novo campo serializado) dispara arch-tests determinísticos (`*_OPT_OUT`, cook-hash) que só aparecem **depois** dos ~3min de compile do hook — cada miss = ciclo de ~5min perdido. Rode o gate do crate antes (vide §5.1).

**Regra: antes do PUSH, rode o comando exato do CI** — `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` (segundos com build morno). E `git commit` SEMPRE em background (o hook estoura o timeout de 2min do shell em foreground).

### 5.1 Tabela de validação

| Situação | Comando | Tempo |
|----------|---------|-------|
| Editou 1 arquivo, quer ver se compila | `cargo check -p <crate>` | 3-15s |
| Editou crate, quer rodar testes | `cargo test -p <crate>` | 5-30s |
| Quer rodar UM teste | `cargo test -p <crate> -- <pattern>` | 1-5s |
| Editou foundational, quer ver downstream | `cargo check --workspace` (NÃO test) | 30-60s warm |
| Vai commitar (T2 hook vai rodar) | **nada** — deixa o hook validar | 0s |
| Fim do ciclo, antes do push | `cargo test --workspace --exclude ph2d-asset` | 3-5min |
| **Antes do push (paridade CI clippy)** | `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | 1-3min warm |
| Só mudou `.md` | **nada** — hook é T0 (skip) | 0s |

### 5.2 LOC threshold (não interrompa o editing burst)

| LOC editados | Comando OK |
|--------------|-----------|
| 0-400 | nada, continue editando |
| 400-1200 | `cargo check -p <crate>` opcional |
| 1200+ ou módulo inteiro | `cargo check -p <crate>` — sane stop |
| Antes do commit | nada — hook valida |

Não rode `cargo test` durante editing burst. Testes só no hook ou em diagnóstico de falha específica.

### 5.3 O que NÃO fazer

❌ `cargo test --workspace` depois de cada edit
❌ `cargo clippy --workspace --all-targets` a cada COMMIT (mas SIM uma vez antes do PUSH — o T2 workspace NÃO cobre `--all-targets`; vide §5 intro)
❌ Re-rodar testes que já passaram pra "confirmar"
❌ Validar baseline no início da sessão se o último commit já está verde
❌ `cargo build` antes de `cargo test` (test já compila)
❌ Re-`Read` arquivo que acabou de editar (o tool já confirmou sucesso)

### 5.4 Pre-commit hook tiered

| Tier | Ativa quando | Tempo |
|------|--------------|-------|
| **T0** | só docs / `.md` / scripts | ~5s |
| **T1** | arquivos de UM crate isolado | ~30s |
| **T2 escopado** | multi-crate **sem** foundational/Cargo.toml/shells | ~30s-3min (nextest -p escopado) |
| **T2 workspace** | `Cargo.toml/lock`, foundational, `shells/desktop/` | ~5-15min (workspace ripple) |

Se acidentalmente trigar T2 workspace numa pasta isolada, provavelmente está staged junto com algo de outro agente — confira `git status --cached`.

**Cortes A+B (2026-05-19):** o hook NÃO roda mais `cargo test --doc --workspace` nem `clippy --all-targets`. Esses ficam pro CI. Implicações práticas:

- Doctest novo (`/// ```` `` em rustdoc) **só é verificado em CI**. Se você adicionar doctest e ele tiver typo, hook deixa passar; CI pega. Quem cria doctest valida manualmente com `cargo test --doc -p <crate>` antes de commitar.
- Benches e examples (`#[bench]`, `examples/*.rs`) **só clippados em CI**. Mesma lógica: validação manual se alterou.
- Em compensação, T2 caiu de ~40min pra segundos em commits típicos. Vide [`project_perf_audit_2026_05_19`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/project_perf_audit_2026_05_19.md) na memória.

### 5.5 Reads cirúrgicos

- `Read` com `offset` + `limit` em vez de ler arquivo inteiro
- `Bash: grep -n` pra localizar primeiro, `Read` depois
- 5 Reads em paralelo na mesma mensagem em vez de sequenciais
- Para busca larga em código novo: subagent `Explore`

### 5.6 Como NÃO escrever test slow

Test lento mata a cadência. Em 2026-05-19 descobrimos 105 tests `SLOW` no workspace que consumiam 14min cache-quente (~83% disso era CoreText scan de fontes via `TextSystem::new()` em test code, 25-77s por chamada × 48 sites em editor-core). Cortes:

**❌ NÃO faça em test:**

- **`TextSystem::new()`** — enumera fontes do sistema via CoreText/Fontconfig. Pesado. Em test, use **`TextSystem::without_system_fonts()`** (pula scan + força `family_name = "InterVariable"` bundled).
- **Alloc gigante pra exercitar limit-check** — ex: `RgbaImage::new(16384, 16384)` (1 GiB) só pra testar que `image::Limits` rejeita. Use dimensão **1 pixel acima** do limite (8193×1 = 32 KiB). `image::Limits` é checado contra IHDR antes da alloc.
- **GPU init repetido por test** — `pollster::block_on(GpuContext::new_headless())` em cada `#[test]`. Use **`OnceLock<Option<GpuContext>>`** lazy module-level. Primeiro test paga, demais zero-init. `GpuContext: Clone` via `Arc` internals.
- **Font shaping real** quando só precisa do shape de uma palavra fixa. Bundle uma fonte mínima ou use o mock fontique do ph2d-text.

**✅ Faça em test:**

- Setup caro em `OnceLock` lazy, compartilhado entre tests do mesmo binário.
- Input minimal: 1 caso simples + 1 caso edge. Não rode o algoritmo com corpus de 100 entradas.
- IO real (font system, FSEvents watcher, network) → `#[ignore]` com doc-comment explicando + `cargo test -- --ignored` no CI separado, OU mover pra `tests/` integration.

**Slow tests inerentes** (aceitos no chão dos ~99s pós-audit):
- `ph2d-asset watcher_*` (~32s × 2) — FSEvents 5s deadline + 250ms poll cycles, security-critical
- `ph2d-render` GPU init (~14-35s × 4-5 binários) — Metal driver cold load per binary

---

## 6. Disciplina git — anti-colisão entre sessões

`git commit` é serializado pelo índice global do git. Se duas sessões têm arquivos staged ao mesmo tempo e uma roda commit, a segunda agarra os arquivos da primeira junto.

### 6.1 Protocolo atômico stage→commit

```bash
# 1) Antes de stage: confira working tree
git status
#    Há M/?? que não são seus? PARE. Outro agente em vôo.

# 2) Stage só os seus. NUNCA -A / -a / git add .
git add <arquivos-específicos>

# 3) Antes de commit: confere índice
git status --cached
#    Tem arquivo que você não estagiou? Vazamento.
#    git restore --staged <não-meus>

# 4) Commit. Hook tiered roda automaticamente.
git commit -m "<descrição em inglês, imperativo, <70 char>"
```

Stage→commit é **uma operação contínua**. Não pause entre os dois passos.

### 6.2 Proibições

- **Nunca** `git push --force` em main
- **Nunca** `--no-verify` (se hook falha, fix root cause)
- **Nunca** `git commit --amend` (sempre novo commit)
- **Nunca** `git config` mudando settings do repo
- **Nunca** `git restore --staged --worktree` em path fora da sua pasta sem coordenar (memória: feedback_destructive_git_outside_pasta)

### 6.3 Sintomas de colisão

| Sintoma | Recuperação |
|---------|-------------|
| `fatal: cannot lock ref 'HEAD'` no commit | Outra sessão commitou no meio. `git status` → diagnose. |
| `git status` mostra M que você não tocou | Outro agente paralelo. NÃO comite. Reporte. |
| `git log -1` mostra mensagem fundida (2 títulos, corpo truncado) | Colisão. Se NÃO pushado: `git reset --soft HEAD~1` + split + recommit. |
| Hook trigga T2 quando você esperava T1 | `git status --cached` — provavelmente vazamento de outro agente. |

### 6.4 Armadilhas conhecidas

**Typos engine bloqueia palavras pt-BR ambíguas.** O hook roda `typos` (full project, respeita `.typos.toml`). Algumas palavras pt-BR têm forma idêntica a typos comuns em inglês — o engine bloqueia. Casos vistos:

| pt-BR escrito | typos vê como | Solução |
|---------------|---------------|---------|
| `erros` | typo de `errors` | usar `falhas`, `problemas`, ou inglês |
| `usso` | typo de `use` | reescrever ou allowlist |
| `nao` (sem acento) | typo de `not` | usar `não` (com acento) |

**Regra prática:** comentários e doc em pt-BR — prefira palavras sem ambiguidade com inglês. Se a palavra "correta" em pt-BR é necessária e disparou typos, adicione exceção em `.typos.toml` (categoria `[default.extend-words]`) **com justificativa no commit**, não esconda com `--no-verify`.

**Sintoma de cargo lock entre sessões.** Se você rodar `cargo check/build/test` enquanto outra sessão Claude Code paralela está rodando comando cargo, a segunda **espera silenciosamente** pelo `target/` lock. Não é crash — só lentidão inesperada. Se você não estava esperando demora, verifica `ps aux | grep cargo` antes de assumir que travou.

**Hook T2 lento sem motivo.** Se T2 demora muito mais que o esperado (~5-15min com cache quente, ~25-40min full cold), checar:
1. Cache pode ter sido invalidado por mudança em foundational/Cargo.toml recente.
2. Algum teste novo virou slow inadvertidamente — vide §5.6.
3. `target/` em rede/disco lento — mover pra SSD local.

---

## 7. Smoke + Push + CI (Coordenador absorve PRCI)

### 7.0 Fast mode (dia) vs Ship (fim do dia)

**Princípio: separar "implementar" de "entregar".** Validação completa + CI rodam **1× por jornada**, não 1× por commit. Quase todo o "tempo perdido" em commits/push/CI vem de validar a cada mudança — não faça isso.

**De dia — fast mode (implementar sem fricção):**
- Checkpoints com `git commit --no-verify` → **instantâneo**, pula o hook. Salva trabalho, permite reverter, sem o pedágio de ~5min do hook.
- `cargo check -p <crate>` só quando quiser confirmar que compila. Nada de `--workspace`/test em loop (§5).
- **ZERO push, ZERO CI durante o dia.**

**Fim do dia — ship (Enio dispara: "commit" / "push" / "ship" / "fim do dia"):**
O Coordenador entra em **modo observa-e-corrige** e tem a OBRIGAÇÃO de entregar commits + push + CI **verdes, sem falta**:
1. **`./scripts/ship.sh`** — roda a job de lint + test do CI inteira, local, de uma vez (fmt, clippy `--all-targets` com as features do CI, `cargo machete`, `cargo deny`, `cargo audit`, `nextest --workspace`). Paridade EXATA com `spike.yml`; o hook local NÃO cobre isso (§5: o perf audit cortou `--all-targets`, e o hook nunca rodou machete/deny/audit).
2. Para CADA `✗` do ship.sh: diagnostica + corrige + re-roda. **NÃO pusha enquanto o ship.sh não estiver 100% verde.** É aqui que o erro de CI é pego — não no CI vermelho 30min depois.
3. Organiza os checkpoints `--no-verify` do dia em commits limpos (squash se preciso).
4. Push (§7.2) → babysit do CI (§7.3) até verde; em vermelho, diagnostica + corrige + re-push, loop até verde (escalona só após 3 falhas do MESMO job).
5. Entrega: reporta o link da run verde ao Enio.

### 7.1 Smoke local — antes do push

```bash
./play.command
```

Smoke é responsabilidade do **Enio**, sob comando do Coordenador. O Coordenador escreve a checklist concreta:

> "Enio, rode `./play.command` e verifica:
> 1. App abre sem panic.
> 2. Tool Transform aparece na TopBar Image Tools com ícone correto.
> 3. Clique → ação esperada.
> 4. Tools/Actions pré-existentes continuam funcionando.
> 5. Sem regressão visual em Hierarchy / Inspector / Widget Gallery."

Enio confirma item por item ou reporta o que viu de diferente.

### 7.2 Push (Coordenador faz)

Batching policy: **push UMA vez por jornada**, no fim do ciclo. CI matrix (linux + macOS + windows + replay hash + bench) demora ~30min. Não push a cada commit.

```bash
git push origin main
```

Antes do push, Coordenador roda **uma vez** a paridade-CI completa local — via o script único (§7.0), que cobre TODOS os passos da job de lint + test (o comando manual antigo era incompleto: faltavam as features do clippy + machete/deny/audit, o que já reddened CI):

```bash
./scripts/ship.sh
```

`✓ CI-clean` → push. `✗` → corrige e re-roda antes de pushar.

### 7.3 Babysit CI (Coordenador faz)

```bash
gh run list --workflow=spike.yml --limit=1 --json databaseId,url
```

Pega o run id. Polling com intervalo de **15 minutos** (`Monitor` com `sleep 900` ou `gh run watch <id>`):

```bash
bash -c '
RUN_ID=<id>
while true; do
  st=$(gh run view "$RUN_ID" --json status -q .status)
  cc=$(gh run view "$RUN_ID" --json conclusion -q .conclusion)
  echo "[$(date +%H:%M:%S)] status=$st conclusion=$cc"
  [ "$st" = "completed" ] && break
  sleep 900
done
'
```

Cenários:

| Resultado | Resposta |
|-----------|----------|
| Success 9/9 | Coordenador reporta link da run + sha bom novo ao Enio. Ciclo fechado. |
| Falha de código | Coordenador diagnostica (`gh run view --log-failed`), aplica fix mínimo local, commita, push, re-watch. |
| Falha de infra (cache, network, rustup flaky) | Não conta. `gh run rerun --failed` + re-watch. |
| 3 ciclos consecutivos de falha do mesmo job | Escalona pro Enio com diagnose + tentativas. |

CI rule de ouro: **fora do babysit, ninguém polla CI.** Push, link, próxima tarefa.

### 7.4 Comunicação pós-push pro Enio

```
✓ Wave <N> pushed. CI run: https://github.com/dibrioli/PH2D/actions/runs/<id>
Entrei em babysit. Reporto quando concluir.
```

E quando termina:

```
✓ CI verde 9/9 em <duração>. sha bom novo: <sha>.
Ciclo fechado. Disponível para próxima ordem.
```

---

## 8. Quando algo dá errado

| Sintoma | Resposta |
|---------|----------|
| Você não sabe o que fazer | Releia §0 + §1.1 + pergunte ao Enio |
| Arquivo que não tocou aparece em `git status` | §6.3 (colisão entre sessões) |
| Hook falha em fmt/clippy/test | Fix root cause; nunca `--no-verify` |
| Hook trigga T2 quando esperava T1 | `git status --cached` — vazamento de outro agente |
| Smoke quebrou no `./play.command` | Implementador diagnostica + fix local na pasta dele |
| CI failure cíclico (3× mesmo job) | Coordenador escalona pro Enio |
| Implementador descobre bug fora da pasta dele | Reporta ao Enio com diagnose; Coord faz |
| Coord quer editar shared mas Implementador está working | Anuncie via Enio, espere Implementador chegar a estado estável, então edite |
| Coord tem dúvida arquitetural | Apresente opções ao Enio com recomendação + tradeoff |
| Memória diz X mas código diz Y | Confie no código. Atualize memória depois. |

---

## 9. Cheat-sheet

### 9.1 Hard Rules ativas (CI-gated)

| HR | Conteúdo | Onde |
|----|----------|------|
| HR-3 | Zero-alloc no dispatcher hot-path | `interaction_dispatch_no_alloc` |
| HR-5 | Determinism cross-platform | CI replay-hash matrix (3 OS) |
| HR-12 | A11y obrigatória | `hr12_widgets_a11y` |
| HR-13 | Memory budget declarado | manifest `memory_budget` |
| HR-15 | Zero hex + zero hardcoded UI string | `no_literal_color` + `hr15_no_hardcoded_ui_strings` |
| HR-18 | Files em `shells/<plat>/src/` ≤ 600 LOC | `file_loc_caps` |
| (Wave 9) | Widget primitive ≤ 500 LOC | `architecture_widget_loc_cap` |
| (Wave 9) | Widget aparece no showcase | `architecture_widget_showcase_coverage` |

Hard Rules completas em [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md) §HR-1..HR-18.

### 9.2 Caminhos físicos canônicos

| O que | Onde |
|-------|------|
| **Node crate novo (fan-out)** | `crates/ph2d-node-<domínio>-<slug>/` (wiring via `cargo run -p ph2d-node-sync`; gate `cargo test -p ph2d-node-registry-init`) |
| **Contrato de nós (🔒 congelado)** | `crates/ph2d-nodegraph/` + `crates/ph2d-expr/` (Coordenador-only + ADR) |
| Tool nova | `crates/ph2d-tool-<slug>/` |
| Painel novo | `crates/ph2d-panel-<slug>/` |
| Widget primitive | `crates/ph2d-editor-core/src/widget/<slug>.rs` |
| Chrome handler | `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` |
| Tool registry init | `crates/ph2d-tool-registry-init/src/lib.rs::register_all` |
| Panel registry init | `crates/ph2d-panel-registry-init/src/lib.rs::register_all_panels` |
| Widget showcase | `crates/ph2d-editor-core/src/widget/showcase/` |
| Tokens source | `docs/design/tokens.json` |
| Tokens Rust | `crates/ph2d-tokens/src/` (codegen via build.rs) |
| Tool design TOML | `docs/design/tools/<slug>.toml` |
| Icon SVG | `docs/design/icons/<slug>.svg` |
| Mockup HTML | `docs/design/screens/*.html` |
| Arch tests editor | `crates/ph2d-editor-core/tests/` |
| Arch tests tokens | `crates/ph2d-tokens/tests/` |
| Arch tests registry | `crates/ph2d-tool-registry-init/tests/` |

### 9.3 Comandos mais usados

```bash
# Implementador — durante edição
cargo check -p ph2d-tool-<slug>
cargo test  -p ph2d-tool-<slug>
cargo test  -p ph2d-tool-<slug> -- some_pattern

# Coordenador — antes do push
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --exclude ph2d-asset

# Coordenador — push + babysit
git push origin main
gh run list --workflow=spike.yml --limit=1
gh run watch <id> --exit-status
```

---

## 10. Referências canônicas

- **Stack + Hard Rules:** [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md)
- **Operacional dia-a-dia + CI:** [`CLAUDE.md`](../../CLAUDE.md)
- **ADR Convention-by-discovery:** [`docs/architecture/decisions/0027-convention-by-discovery.md`](../architecture/decisions/0027-convention-by-discovery.md)
- **ADR Codegen + design canonical:** [`docs/architecture/decisions/0028-wave-2-codegen-design-canonical.md`](../architecture/decisions/0028-wave-2-codegen-design-canonical.md)
- **ADR Trait-driven panel host:** [`docs/architecture/decisions/0029-trait-driven-panel-host.md`](../architecture/decisions/0029-trait-driven-panel-host.md)
- **Sistema de nós — briefing do fan-out (§3.8):** [`docs/IntegracaoMultiAgente/briefing-node-crate.md`](briefing-node-crate.md)
- **Sistema de nós — estado vivo + loop autônomo:** [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md)
- **ADR FREEZE do contrato de nós (W2.T4):** [`docs/architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md`](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)
- **Memória LLM (auto-loaded):** `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`

---

## 11. Versão + histórico

- **6.5 — 2026-05-22:** Arquitetura node-centric absorvida no doc. §3.8 novo balde "Node crate (fan-out)" — o fan-out mais leve do projeto (wiring gerado por `ph2d-node-sync`, zero edit central), apontando pro `briefing-node-crate.md` como briefing canônico. §3.6 foundational ganha o contrato `ph2d-nodegraph`+`ph2d-expr` (🔒 congelado em W2.T4, ADR-0039 — mudança = Coordenador-only + ADR + re-prova de paridade CPU↔WGSL). TL;DR + §9.2 + §10 atualizados. Os baldes de editor (§3.1-3.4) permanecem válidos (chrome do editor que edita os grafos de nós).
- **6.1 — 2026-05-19 noite:** Perf audit + pre-commit T2 cuts A+B aplicados (commit `10ef2b6` + range `436626e..cb13efe`). Acréscimos: §3.7 trabalho cross-cutting (perf audit / refactor cross-crate); §5.6 como NÃO escrever test slow (com `TextSystem::without_system_fonts()`, `OnceLock` GpuContext, alloc-pequena pra limit-check); §6.4 armadilhas conhecidas (typos pt-BR, cargo lock, hook lento). Tabela §5.4 atualizada com T2 escopado vs T2 workspace.
- **6.0 — 2026-05-19:** Modelo 2 papéis (Coordenador absorvendo PRCI) + fluxo invertido (scaffold central antes do Implementador começar). Condensa em um único doc: v5.0 DIRETRIZ + 4 docs operacionais 01-04 + STATE.md + DIRETRIZ_CODIFICACAO_RAPIDA.md + PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md.
- **5.0 — 2026-05-17** (arquivada): Diretriz unificada substituindo 01-04, ainda com modelo 4 papéis (Enio relay / Coord / Periférico / PRCI) e fluxo Periférico-primeiro.
- **4.0 — 2026-05-13** (arquivada em `ARCHIVE-v4.0-pre-wave-1/` quando criada): modelo Coordenador editava manualmente icons.rs / fixture.rs / ids.rs. Arquitetura morta pós Wave 1.

---

## 12. Quando esta diretriz fica obsoleta

Se a arquitetura mudar materialmente (ex.: surge um terceiro papel, ou o fluxo invertido vira fluxo lateral), atualize esta diretriz in-place e bump a versão. **Não fragmente em múltiplos docs** — a lição dos 4 docs antigos que dessincronizaram é que um doc único é mais fácil de manter atualizado.

Se você é LLM lendo isto depois de uma mudança arquitetural maior e a diretriz contradiz o código, **confie no código**, reporte ao Enio com diagnose, e atualize esta diretriz quando autorizado.
