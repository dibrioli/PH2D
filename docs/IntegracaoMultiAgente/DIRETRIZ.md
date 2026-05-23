# Diretriz de Implementação Universal — PH2D

**Versão:** 6.8 — 2026-05-22 · pós-ADR-0030..0040, princípio transversal **Tool↔Nó simétricos** (uma única receita de fan-out drop-crate em §3.8 cobre as duas famílias). Histórico de versões em §11 (v6.7+v6.8 detalhados; anteriores resumidas + `git log`).
**Audiência:** **toda LLM que entra no projeto.** Lê este doc inteiro antes de tocar em código. Quando o Enio descreve uma tarefa, seu **primeiro output é a TRIAGEM (§1.4)** — classifica e diz se precisa de Coordenador + Implementador ou só Implementador. **Doc único de implementação:** briefing pronto-pra-colar de drop-crate (§3.8) e receitas dos demais baldes vivem aqui; não há doc separado.

---

## TL;DR

> **Triagem primeiro. Dois papéis. Duas famílias-irmãs de fan-out. Zero colisão por construção.**
>
> 0. Enio descreve a tarefa a uma sessão. **Antes de codar, o agente faz a TRIAGEM (§1.4): classifica o balde, diz se toca um contrato congelado, e informa ao Enio como proceder — só Implementador (A), Coordenador + Implementador (B), ou Coordenador-only + ADR (C).** O Enio age conforme a triagem.
> 1. **Caminho (A) — drop-crate (fan-out, §3.8):** tarefa é **node novo OU tool nova**. Uma sessão Implementador sozinha: cria `crates/ph2d-{node,tool}-…/`, roda `cargo run -p ph2d-{node,tool}-sync`, staleness gates fecham o wiring. Zero edit central. Zero coordenação. Pode rodar em paralelo com outros Implementadores na mesma família ou na outra.
> 2. **Caminho (B) — fluxo invertido (scaffold-primeiro):** tarefa é **painel/widget/chrome novo** (§3.2-3.4) — peças que ainda exigem edit central. Coordenador cria pasta + plugues centrais (Cargo.toml/showcase/dispatcher) + stubs verdes, entrega briefing pra Implementador preencher só dentro da pasta. Coordenador revisa, smoke com Enio, commit, push, babysit CI.
> 3. **Caminho (C) — Coordenador-only + ADR:** tarefa toca contrato congelado (nodegraph/expr/Tool/ImageEditTool/PanelEvent — §3.6) ou foundational (tokens, editor-core, shells, arch tests). Evento raro. Não paraleliza, não delega.
>
> **Enio não decide nada operacional.** Coordenador (quando há um) instrui Enio passo a passo. Enio é relay mecânico entre as sessões Claude Code.
>
> **Norte arquitetural:** a engine cresce por **duas famílias de crate isolado simétricas** (ADR-0031, ADR-0040): nós (`crates/ph2d-node-*`, declarativos pull-side / FBP) e tools (`crates/ph2d-tool-*`, imperativos push-side / manipulação direta). Ambas wireadas por codegen (`tools/ph2d-node-sync`, `tools/ph2d-tool-sync`), ambas com contrato congelado por arch-gate. Adicionar feature de conteúdo OU peça de editor manipulando bitmap = drop-crate. Painel/widget/chrome do editor (que renderiza essas tools/nós) ainda passa pelo Coordenador.

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

**Coordenador** — única autoridade global, **convocado quando a triagem (§1.4) pede caminho (B) ou (C)**, OU no momento de ship (fim de jornada, §7) mesmo que tudo tenha sido caminho (A): ship é serializado por uma única sessão. Quando a jornada foi 100% (A) sem Coord, Enio promove um dos Implementadores a Coord pro ship (ou abre uma sessão Coord nova). Centraliza tudo que é "arquivo compartilhado não-codegen'd" (painel/widget/chrome + foundational + contratos congelados). Faz scaffold de feature nova *antes* do Implementador começar (caminho B). Revisa entrega. No ship: ship.sh + commit + push + babysit CI. Absorve o papel que antes era PRCI.

**Implementador** — sessão isolada, uma por feature. **No caminho (A)** (node/tool — §3.8), trabalha sem Coordenador: cria a pasta, roda o sync, testa. **No caminho (B)** (painel/widget/chrome — §3.2-3.4), recebe pasta já plugada na árvore pelo Coordenador e edita **somente** dentro dela. Em ambos os casos, reporta pronto. Pode rodar em paralelo com outros Implementadores sem coordenação direta — a arquitetura física garante que eles não colidem (glob de `workspace.members` + codegen splice em regiões marcadas, pra (A); pasta isolada já plugada, pra (B)).

Enio não é papel. Enio é o humano que orquestra: abre sessões Claude Code, cola mensagens entre elas, roda smoke visual quando o Coordenador pede.

### 1.2 Dois caminhos: fan-out drop-crate vs fluxo invertido

Duas formas de adicionar coisa à árvore, escolhidas pela triagem (§1.4):

**Caminho (A) — drop-crate (fan-out, sem Coordenador).** Aplica a **node novo OU tool nova** — receita única em §3.8. Implementador sozinho:

```
Implementador (uma sessão):
  1. Cria crates/ph2d-{node,tool}-<slug>/ com lib.rs + manifest/register/make
  2. Roda `cargo run -p ph2d-{node,tool}-sync` (regenera o wiring central)
  3. cargo test -p ph2d-{node,tool}-<slug>           ← seu code
     cargo test -p ph2d-{node,tool}-registry-init    ← staleness gate(s) fecham
  4. Commit local, reporta pronto.
```

Zero edit central. Zero conflito git. Dois Implementadores adicionando dois nós (ou dois tools) **não tocam nenhum arquivo em comum** — `workspace.members` é glob (`crates/*`, `tools/*`), e as superfícies centrais são **geradas** entre marcadores codegen pelo `sync` da família: **node** tem 1 superfície (`register_all_nodes` em `ph2d-node-registry-init` + deps `Cargo.toml`); **tool** tem 3 (`register_all` manifests + `register_all_tools` `Box<dyn Tool>` em `ph2d-tool-registry-init` + deps `Cargo.toml`). Detalhe por família em §3.8.1.

**Caminho (B) — fluxo invertido (Coordenador faz scaffold central primeiro).** Aplica a **painel/widget/chrome novo** (§3.2-3.4) — peças que ainda exigem edit central porque o registro é manual (showcase, dispatcher de chrome, features de panel-registry-init):

```
Enio: "Coordenador, vamos criar o painel Outline."
   │
   ▼
Coordenador:
  1. Decide: painel? widget? chrome? (vide §3.2-3.4)
  2. Cria a pasta inteira com stubs verdes
  3. Faz o(s) edit(s) central(is) específico(s) do balde
     (panel-registry-init::register_all_panels alfabético,
      widget/mod.rs, chrome dispatch_all, etc.)
  4. cargo check do crate novo  →  verde
  5. Briefing pro Implementador (§2)
   │
   ▼  (Enio abre nova sessão, cola briefing)
   │
Implementador:
  6. Lê briefing + esta DIRETRIZ + sanity check
  7. Edita SÓ dentro da pasta atribuída
  8. cargo test do crate  →  verde
  9. Commit local, reporta pronto
   │
   ▼
Coordenador:
 10. Revisa diff, pede smoke pro Enio
 11. Em ciclo de ship: push + babysit CI (§7)
```

A partir do passo 5 o Implementador **nunca** toca arquivo fora da pasta dele. Por que dois caminhos? Porque painel/widget/chrome ainda não foram codegen'dos como nó e tool foram (ADR-0040 §2.3 generalizou pra tools, ADR-0031 fez pra nós — painel ainda tem `register_all_panels` editado à mão com `#[cfg(feature = "panel-<slug>")]`; widget tem `widget/mod.rs` e showcase central; chrome tem `dispatch_all`). Quando uma dessas peças virar codegen, migra pra (A).

### 1.3 As 3 obrigações do Implementador (sempre, sem exceção)

1. **ISOLAMENTO.** Edita **só** arquivos dentro da pasta exclusiva atribuída pelo Coordenador. Se precisa de algo fora (dep nova, mudança em foundational, novo NodeId), **reporta** ao Enio — não edita por conta própria.
2. **UI canônica.** Toda cor, espaçamento, raio, tipografia, stroke **passa por tokens** (`ColorToken::X.resolve(theme)`, `Spacing::Lg.px()`, etc.). Zero hex, zero `f32` literal de UI. Vide §4.
3. **Codificação rápida.** Usa `cargo check -p <crate>` durante editing burst. Não duplica trabalho do pre-commit hook. Não roda `--workspace` em loop. Vide §5.

Se você é o Implementador e está pra violar uma das três, **pare e reporte**. Quase certamente significa que o Coordenador não fez o scaffold direito.

### 1.4 Triagem — seu PRIMEIRO output (Coordenador ou só Implementador?)

**Antes de tocar em código, antes de assumir um papel:** quando o Enio te descreve uma tarefa ("quero criar X"), seu primeiro output **não** é começar — é **classificar a tarefa e dizer ao Enio como proceder**. O Enio não sabe de antemão se precisa abrir uma sessão Coordenador + uma Implementador, ou se uma sessão Implementador sozinha resolve. **Você decide isso por ele e informa.**

Responda ao Enio **exatamente** neste formato:

```
TRIAGEM
- Tarefa: <o que o Enio pediu, em 1 linha>
- Balde: <§3.2 painel | §3.3 widget | §3.4 chrome | §3.5 modificar existente |
          §3.6 foundational | §3.7 cross-cutting | §3.8 drop-crate (node|tool)>
- Toca um contrato congelado (nodegraph/expr OU Tool/ImageEditTool/PanelEvent)?
    <Não | Sim — exige ADR + bump de cap>
- COMO PROCEDER:
    (A) Só Implementador — sessão isolada. Drop-crate + sync (§3.8). Sem scaffold central.
    (B) Coordenador + Implementador — scaffold central antes (painel/widget/chrome).
    (C) Coordenador-only — foundational ou contrato congelado; não paraleliza, pode exigir ADR.
- Razão: <por que esse caminho, em 1-2 linhas>
- Se grande/ambíguo: <peças, e o que é isolável vs. compartilhado>
```

Tabela de decisão:

| Tarefa | Balde | Toca contrato congelado? | Como proceder |
|--------|-------|--------------------------|---------------|
| **Nó novo OU tool nova** (any shape) — domínio/contrato existente | §3.8 | Não | **(A) Só Implementador** — drop-crate em `crates/ph2d-{node,tool}-…/` + `cargo run -p ph2d-{node,tool}-sync` + testa. Wiring gerado, zero edit central. Para tool: sem variant novo em `EditorAction`. |
| **Modificar** nó/tool/feature existente (sem novo arquivo central) | §3.5 | Não | **(A) Só Implementador** — a pasta já existe; edite dentro dela. |
| **Painel novo** (`ph2d-panel-<slug>` — docado a uma tool ou genérico) | §3.2 | Não | **(B) Coordenador + Implementador** — Coord plumba feature flag + linha em `register_all_panels` ANTES (peça não-codegen'd). |
| **Widget primitive novo** (em `editor-core/src/widget/`) | §3.3 | Não | **(B) Coordenador + Implementador** — Coord adiciona em `widget/mod.rs` + cria seção do showcase ANTES. |
| **Chrome handler novo** (TopBar/LeftRail/BottomHUD/ContextMenu) | §3.4 | Não | **(B) Coordenador + Implementador** — Coord adiciona em `chrome/mod.rs::dispatch_all` ANTES. |
| **Avaliador novo (Wave-neck)** para um domínio sem avaliador ainda (Shader, Som, Gameplay) | §3.8 + Wave-neck (vide [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md)) | Não (usa o contrato), mas é greenfield grande | **(C) Coordenador-only** durante o neck — trabalho "tipo W2" serial; só depois de fechado, o domínio abre para fan-out (A). |
| **Domínio com avaliador já existindo** (mais nós Motion etc.) | §3.8 | Não | **(A) Só Implementador** — vide primeira linha desta tabela. |
| **Mudar tokens / editor-core (sem ser contrato) / shells / arch tests** | §3.6 | Não | **(C) Coordenador-only** — foundational, não paraleliza. |
| **Mudar contrato de nós** (porta, tipo, formato, EvalCtx, motor) | §3.6 | **Sim — nodegraph/expr** | **(C) Coordenador-only + ADR** — bump do cap em `architecture_contract_surface.rs` + ADR estendendo 0039. |
| **Mudar contrato de tools** (método novo em `Tool`/`ImageEditTool`, variant novo em `PanelEvent` ou `EditorAction::ToolPanelEvent`) | §3.6 | **Sim — Tool/ImageEditTool/PanelEvent** | **(C) Coordenador-only + ADR** — bump do cap em `architecture_tool_contract_surface.rs` + amendment de ADR-0040 §7. |

Heurística de uma frase: **feature de conteúdo (nó) OU peça de editor que manipula bitmap (tool) = drop-crate = (A) Implementador-só (§3.8). Peça do chrome que renderiza essas tools/nós (painel/widget/chrome) = (B) Coord faz o central primeiro. Mudar regra do jogo (contrato congelado, foundational) = (C) Coord-only + ADR.** Na dúvida entre A e B, pergunte: "isso exige editar QUALQUER arquivo fora de uma única pasta nova?" Se sim → (B). Se a única coisa fora da pasta é o wiring **gerado** (`ph2d-{node,tool}-sync`), ainda é **(A)**.

**Nota operacional pra caminho (A):** o `sync` vai sujar `crates/ph2d-{tool,node}-registry-init/src/lib.rs` + `Cargo.toml` (regenera entre marcadores codegen). Esse diff é **esperado e válido** — não viola a regra de "edita só dentro da pasta" da §1.3. O staleness gate em CI exige justamente essa regeneração — esquecer o sync é o que rederia CI, não rodá-lo.

---

## 2. Como Coordenador e Implementador se comunicam

Enio é **relay mecânico**, não decisor.

### 2.1 Coordenador → Implementador (caminho B)

Aplica APENAS ao caminho **(B)** (painel/widget/chrome — §3.2-3.4). Para caminho **(A)** (node/tool — §3.8), o briefing canônico parametrizado por família já está pronto-pra-colar em §3.8.2 — Enio cola direto numa sessão Implementador sem precisar de Coordenador.

Quando o Coordenador precisa que o Implementador comece (caminho B), ele entrega ao Enio um briefing pronto-pra-colar com este formato:

```
═══════════════════════════════════════════════════════════════════
BRIEFING — IMPLEMENTADOR · slot <N> · feature: <slug> (painel/widget/chrome)
═══════════════════════════════════════════════════════════════════

PASTA EXCLUSIVA: <ex: crates/ph2d-panel-<slug>/>

ESTADO INICIAL (já plugado na árvore pelo Coordenador):
- Cargo.toml      (deps prontos)
- src/lib.rs      (impl <stub-do-balde> compilando verde)
- (linha registrada em panel-registry-init / widget/mod.rs /
   chrome dispatch_all — conforme balde)

O QUE VOCÊ FAZ (preenche dentro da pasta):
- <ex: paint do painel, apply_event, populate>
- cargo test -p <crate-novo> verde

O QUE VOCÊ NÃO TOCA (em hipótese alguma):
- Qualquer arquivo fora da pasta atribuída
- Cargo.toml raiz / panel-registry-init / widget/mod.rs / chrome/mod.rs
  (já está feito pelo Coordenador)
- crates/ph2d-tokens/* (foundational)
- crates/ph2d-editor-core/* — exceto a pasta-balde indicada,
  para widget/chrome (foundational por enquanto)
- crates/ph2d-{node,tool}-* (outras famílias / outras sessões)
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

- "Enio, rode `./play.command` e me diga se o painel Outline aparece na zona direita com o cabeçalho correto."
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

## 3. Receitas canônicas — uma por balde

Sete buckets, dois caminhos. O balde §3.8 (drop-crate node OU tool, fan-out simétrico) é **caminho (A)** — sem Coordenador. Os baldes §3.2-§3.4 (Painel/Widget/Chrome) são **fluxo invertido (B)** — Coordenador scaffold-primeiro. Os baldes §3.5-§3.7 cobrem casos transversais (modificar existente, foundational, cross-cutting). §3.1 é só um redirect histórico pro §3.8.

### 3.1 Tool nova → vá direto pro §3.8

> **🔒 ADR-0040 FECHADO (2026-05-22, TG-E `fc23647`).** Tool nova é **caminho (A) drop-crate**, junto com nó — receita única em **§3.8**. Sem scaffold central, sem variant novo em `EditorAction`. `editor-core/src/tools/` foi **deletado em TG-D `c4063b7`** (foundation ⊥ tools por `architecture_cycle_prevention::editor_core_has_no_concrete_tool_deps`). Os 3 sabores de tool (one-shot / palette modal / stateful + panel) estão tabelados em §3.8.3.
>
> O scaffold pré-ADR-0040 (Coord criava Cargo.toml + editava `register_all` à mão + tocava `editor-core/src/tools/`) virou arqueologia — vide ADR-0040 §7 e [`docs/HANDOFF_tool_isolation_close.md`](../HANDOFF_tool_isolation_close.md) (banner: "EXECUTADO em 2026-05-22"; referência histórica do raciocínio, não fluxo vigente).

### 3.2 Painel novo ("vamos criar o painel Outline")

**Coordenador (scaffold):**

1. Decide `slug` (`outline`), `DEFAULT_VISIBLE`, feature flag (`panel-outline`).
2. Cria crate `crates/ph2d-panel-outline/` — `Cargo.toml` adiciona deps em `ph2d-editor-core` (Panel trait + PaintCtx + PanelHostInternal), `ph2d-a11y` (NodeId), `ph2d-tokens`, `ph2d-text`, `ph2d-vector`. Glob `workspace.members` (`crates/*`) cobre o crate sem edit em `Cargo.toml` raiz.
3. Cria `src/lib.rs` com stub do `impl Panel` (vide [`crates/ph2d-panel-inspector/src/lib.rs`](../../crates/ph2d-panel-inspector/src/lib.rs) como template completo):
   ```rust
   #![forbid(unsafe_code)]
   use ph2d_a11y::NodeId;
   use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
   use ph2d_editor_core::panel::{
       EventOutcome, PaintCtx, Panel, PanelHostInternal,
   };
   use ph2d_tool_registry::hash_node_id;

   pub struct OutlinePanel;

   #[derive(Default)]
   pub struct OutlineState { /* placeholder */ }

   impl Panel for OutlinePanel {
       type State = OutlineState;
       const ID: &'static str = "outline";
       const NODE_ID: NodeId = hash_node_id("panel.outline");
       const DEFAULT_VISIBLE: bool = false;

       fn paint(_state: &mut Self::State, _ctx: &mut PaintCtx) {
           // implementador preenche
       }

       fn apply_event(
           _state: &mut Self::State,
           _host: &mut dyn PanelHostInternal,
           _ev: WidgetEvent,
       ) -> EventOutcome {
           EventOutcome::Ignored
       }

       fn populate(_store: &mut WidgetStore) {}
   }
   ```
   **Notas factuais:**
   - `Panel::paint` tem **2 parâmetros** (`state`, `ctx`); o host fica em `ctx.host` (campo de `PaintCtx`), não como parâmetro separado.
   - O trait usado pelo host é `PanelHostInternal` (não `PanelHost`).
   - `hash_node_id` vive em `ph2d-tool-registry`, não em `ph2d-a11y` (`ph2d-a11y` só expõe `NodeId` em si).
4. Adiciona feature em `crates/ph2d-panel-registry-init/Cargo.toml` (o `default` atual lista 6 painéis — bgremoval, padding, inspector, hierarchy, widget-gallery, grid-snap; manter os existentes + acrescentar o novo):
   ```toml
   [features]
   default = [
       "panel-bgremoval", "panel-padding", "panel-inspector",
       "panel-hierarchy", "panel-widget-gallery", "panel-grid-snap",
       "panel-outline",
   ]
   panel-outline = ["dep:ph2d-panel-outline"]
   ```
   E adiciona em `[dependencies]`: `ph2d-panel-outline = { path = "../ph2d-panel-outline", optional = true }`.
5. Adiciona em `crates/ph2d-panel-registry-init/src/lib.rs::build_typed_registry` (a ordem atual NÃO é alfabética — segue a ordem de migração ADR-0029; **não há gate alfabético pra panel-registry-init** diferente do tool-registry-init, então mantenha a ordem que faz sentido pro tipo de painel — image-tool panels primeiro, depois os de editor):
   ```rust
   #[cfg(feature = "panel-outline")]
   reg.push(ErasedPanel::new::<ph2d_panel_outline::OutlinePanel>());
   ```
6. Atualiza `EXPECTED_TYPED` no `#[cfg(test)] mod tests` do `panel-registry-init` (incrementa contador `#[cfg(feature = "panel-outline")] { n += 1; }`).
7. `cargo check -p ph2d-panel-outline` + `cargo test -p ph2d-panel-registry-init` verde.
8. Commita + entrega briefing (vide §2.1).

**Implementador:** preenche `OutlineState`, `paint`, `apply_event`, `populate`.

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
2. Adiciona no dispatcher em `chrome/mod.rs` (preferencialmente alfabético — não há arch-gate alfabético pra chrome, então é convenção de higiene de merge, não regra):
   ```rust
   pub mod snap_toggle;
   ```
   E na fn `dispatch_all` adiciona `|| snap_toggle::apply(hero, event)`.
3. Se o handler precisa de NodeIds novos, adiciona em `screens/hero/ids.rs` via `hash_node_id`.
4. Se precisa item de menu/popover, adiciona em `pre_populate.rs` (ou pede ao Implementador).
5. `cargo check -p ph2d-editor-core` verde. Briefing.

**Implementador:** preenche corpo do handler.

### 3.5 Modificar feature existente ("o ícone da tool Trim está errado", "o algoritmo de Padding tem bug", "o BgRemoval precisa de novo slider")

Não há scaffold. A pasta já existe. **Caminho (A) Implementador-só** — Coordenador (se foi convocado) delega direto, ou o Enio abre uma sessão Implementador no balde correto e cola:

```
Implementador slot N: edite crates/ph2d-tool-<slug>/src/<arquivo>.rs.
Tudo da feature vive no crate isolado (manifest + tool + algorithm + icon + params +
panel docado em ph2d-panel-<slug>/ quando aplicável). Não toque em nada fora.
Se o ajuste exigir edit em algum arquivo central (Cargo.toml raiz, EditorAction,
contrato congelado, foundational), PARE e reporte — quase certo significa que
a tarefa estava mal triada (§1.4).
Reporte quando terminar.
```

Mapa rápido (pasta canônica por feature):

| Feature | Pasta canônica |
|---------|----------------|
| Tool (algo / ícone / manifest / `impl Tool` / `handle_panel_event`) | `crates/ph2d-tool-<slug>/` |
| Vocab UI de um tool (`<Slug>UiEdit`, `<Slug>UiSnapshot`, `<Slug>Params`) | `crates/ph2d-tool-<slug>/src/params.rs` (TG-B/TG-C migrou de editor-core) |
| Panel docado de um tool | `crates/ph2d-panel-<slug>/` |
| Nó | `crates/ph2d-node-<dom>-<slug>/` |
| Painel genérico (Inspector/Hierarchy/etc.) | `crates/ph2d-panel-<slug>/` |
| Widget primitive | `crates/ph2d-editor-core/src/widget/<slug>.rs` |
| Chrome handler | `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` |

A pasta `crates/ph2d-editor-core/src/tools/` **não existe** desde ADR-0040 TG-D (`c4063b7`). Se a memória de uma LLM anterior te apontar pra lá, ignore — confie no `ls`.

### 3.6 Mudança em foundational ("precisamos de um token de cor novo")

Foundational = `ph2d-core`, `ph2d-tokens`, `ph2d-editor-core`, `ph2d-a11y`, `ph2d-host`, `ph2d-vector`, `ph2d-text`, `ph2d-tool-registry`, `ph2d-tool-registry-init`, `ph2d-node-registry`, `ph2d-node-registry-init`, `ph2d-panel-registry-init`, `tools/ph2d-{node,tool}-sync`, `shells/*`, arch tests, **+ os dois contratos congelados abaixo**.

**Foundational não é paralelizável.** Coordenador faz **sozinho**. Não delega.

**Dois contratos congelados — paralelos, com mesma disciplina.** Mexer em qualquer um deles **não é só foundational, é evento raro Coordenador-only com ADR**. Os caps dos arch-gates estão apertados ao tamanho atual sem folga, então qualquer crescimento tripa o gate de propósito.

| Contrato | Arquivos congelados | Arch-gate (cap) | ADR | Mudar exige |
|----------|--------------------|----|-----|-------------|
| **Sistema de nós** (W2.T4, 2026-05-22) | `crates/ph2d-nodegraph/src/{lib.rs,node.rs,port.rs,effect.rs,attr.rs,cook.rs,graph.rs}` + `crates/ph2d-expr/src/lib.rs` | `crates/ph2d-nodegraph/tests/architecture_contract_surface.rs` — `NodeOp ≤ 2 métodos`, `OpResolver ≤ 1 método`, `NodeManifest ≤ 8 campos` | [ADR-0039](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md) | Bump do cap + ADR novo estendendo 0039 + (em `ph2d-expr`) re-provar paridade CPU↔WGSL |
| **Sistema de tools** (TG-E, 2026-05-22) | `crates/ph2d-editor-core/src/tool.rs` (traits `Tool` + `ImageEditTool`, enum `PanelEvent`) + canal genérico em `crates/ph2d-editor-core/src/action_bus.rs` (`EditorAction::{ActivateTool, OneShotImageOp, ToolPanelEvent, CancelActiveTool}`) | `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs` — `Tool ≤ 10 métodos`, `ImageEditTool ≤ 4 métodos`, `PanelEvent ≤ 4 variants` | [ADR-0040 §7](../architecture/decisions/0040-tool-as-isolated-feature-crate.md) | Bump do cap + amendment de ADR-0040 §7 |

**O que NÃO mexe nesses contratos** (é fan-out drop-crate, vide §3.8, sem Coordenador):

- Adicionar **nó novo** num domínio com avaliador existente — `ph2d-node-<dom>-<slug>/` + `cargo run -p ph2d-node-sync`.
- Adicionar **tool novo** (any shape) — `ph2d-tool-<slug>/` + `cargo run -p ph2d-tool-sync`.
- Adicionar **NodeId** novo num panel docado — só edita o crate do tool/panel.
- Adicionar campo novo num `<Slug>UiEdit` enum — vive em `ph2d-tool-<slug>/src/params.rs`.

Exemplo de mudança foundational (não-contrato): adicionar `ColorToken::AccentTeal`:
1. Coord edita `docs/design/tokens.json` adicionando a chave em todos os 4 temas.
2. Coord roda `cargo check -p ph2d-tokens` (build.rs regenera).
3. Coord edita `crates/ph2d-tokens/src/color.rs` adicionando o variant.
4. Coord roda `cargo test --workspace --exclude ph2d-asset` pra garantir nada quebrou.
5. Coord commita: `feat(tokens): add ColorToken::AccentTeal`.

### 3.7 Trabalho cross-cutting (perf audit, refactor cross-crate, manutenção de tests)

Algumas tarefas não cabem em nenhum dos baldes acima (§3.2-3.6) porque tocam múltiplos crates por natureza — perf audit do workspace, deduplicação de pattern, migração de API antiga em N crates consumidores, sweep de lint novo, etc.

**Coordenador autoriza explicitamente a exceção ao isolamento.** O briefing pro Implementador diz literalmente:

> "Você toca tests em vários crates conforme os achados. Exceção autorizada à regra de uma pasta isolada da DIRETRIZ §1.3. Cada commit ainda fica T1 single-crate sempre que possível."

**Regras desse bucket:**

1. **Cada commit valida-se sozinho** — `cargo test -p <crate>` verde para o crate tocado, antes do commit. Pre-commit hook entra em T1 (single-crate, ~30s), não T2.
2. **Não tocar production code de foundational sem motivo claro** — em audit de tests, mexer só nos tests (`tests/`, `#[cfg(test)] mod`). Production `pub fn` fica intocado salvo se a auditoria revelar API faltando (e nesse caso vira novo trabalho discutido com Enio).
3. **Documentar risk surface** — no relatório final, listar todas as mudanças sutis de comportamento que CI pode capturar mas smoke local pode não ver (ex: "função X agora cacheia via `OnceLock` — primeiro caller paga, demais zero-init; tests confirmados que só fazem `&ctx` imutável").
4. **Tokens canônicos continuam valendo** — geralmente N/A em test code, mas se tocar paint/widget, mesma regra de §4.

**Exemplo real (2026-05-19 noite):** perf audit cortou nextest workspace de 14min para 1.5min via 6 commits, cada um T1 single-crate, todos verdes em CI. Detalhe na memória `project_perf_audit_2026_05_19.md`.

### 3.8 Fan-out drop-crate (A) — node OU tool (uma receita, duas famílias)

Esta é a forma simétrica como a engine cresce: largar um crate isolado em `crates/ph2d-{node,tool}-*/`, rodar o `sync` correspondente, gates fecham. Sem edit central, sem coordenação. **Garantia formal de não-colisão em §3.8.4.**

**Diferença-chave vs. baldes (B) §3.2-3.4 (painel/widget/chrome):** lá o registro central é editado à mão (`register_all_panels` com `#[cfg(feature = …)]` / `widget/mod.rs` / `chrome/mod.rs::dispatch_all`), por isso precisam de scaffold-primeiro. Aqui não.

#### 3.8.1 Mapa node ↔ tool (use ao preencher o briefing)

| Aspecto | **Node** (declarativo, pull-side / FBP) | **Tool** (imperativo, push-side / manipulação direta) |
|--------|----------------------------------------|------------------------------------------------------|
| Pasta exclusiva | `crates/ph2d-node-<domínio>-<slug>/` | `crates/ph2d-tool-<slug>/` |
| Codegen | `cargo run -p ph2d-node-sync` | `cargo run -p ph2d-tool-sync` |
| Wiring gerado | `register_all_nodes` + deps `Cargo.toml` (1 superfície) | `register_all` + `register_all_tools` + deps (3 superfícies) |
| Gate de wiring | `cargo test -p ph2d-node-registry-init` (staleness) | `cargo test -p ph2d-tool-registry-init` (3 staleness + 3 alfabéticos) |
| Contrato implementado | `NodeOp` + `NodeManifest` (em `ph2d-nodegraph`) | `Tool` + opcional `ImageEditTool` + `ToolManifest` (em `ph2d-editor-core` + `ph2d-tool-registry`) |
| 🔒 Cap arch-gate | `architecture_contract_surface` — `NodeOp ≤ 2 / OpResolver ≤ 1 / NodeManifest ≤ 8 campos` (ADR-0039) | `architecture_tool_contract_surface` — `Tool ≤ 10 / ImageEditTool ≤ 4 / PanelEvent ≤ 4` (ADR-0040 §7) |
| Entry point(s) do crate | `pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError>` | `pub fn register(reg: &mut Registry)` (manifest) e/ou `pub fn make() -> Box<dyn Tool>` (behavior); 3 sabores — vide §3.8.3 |
| Vocab de canal (foundation) | portas tipadas (`PortType = domínio+dim+CLOCK`) + efeito + clock + params | `EditorAction::{ActivateTool, OneShotImageOp, ToolPanelEvent(PanelEvent), CancelActiveTool}` (4 genéricos — sem variant per-tool) |
| Membrana | `Pure`/`Temporal` (pull, isento HR-5) vs `Stateful` (gameplay, escreve sim) — `Graph::validate` prova | `tool-* → editor-core` OK; `editor-core ⊥ tool-*` gateado (`editor_core_has_no_concrete_tool_deps`) |
| Templates | (Pure trivial) `crates/ph2d-node-debug-const/`; (Temporal + ph2d-expr + golden) `-debug-wave/`; (vertical Stateful-free, generator/cloner/modifier) `-motion-{grid,clone,transform}/` | sabor (1) one-shot: `-make-square/` / `-trim-transparency/` / `-real-size/`; (2) palette modal: `-brush/` (`is_default=true`) / `-move/`; (3) stateful + panel: `-padding/` / `-bgremoval/` (completo) |
| Pegadinhas (gotchas) | `ctx.param("nome")` no eval (NUNCA `MANIFEST.params[..].default`); `param_as_count(v, max)` p/ alocação capada | `apply_ui_edit` = single-source-of-truth de clamps; `handle_panel_event` roteia NodeId → variant; ícone exige IconId variant em ordem alfabética em `ph2d-editor-core/src/icons.rs` |

#### 3.8.2 Briefing pronto-pra-colar (Enio cola numa sessão Implementador nova)

Substitua `<family>` por `node`/`tool`, `<slug>` pelo seu, e se for node preencha `<domínio>` (ex.: `motion`). **Convenção dos marcadores:** linhas/blocos prefixados com `[node]` valem só pra família node; idem `[tool]`. Quem cola apaga os blocos da família errada antes de mandar pro agente.

```
═══════════════════════════════════════════════════════════════════
BRIEFING — <family>-crate · slug: <slug>  [node]  · domínio: <domínio>
═══════════════════════════════════════════════════════════════════

PASTA EXCLUSIVA:
  [node]  crates/ph2d-node-<domínio>-<slug>/
  [tool]  crates/ph2d-tool-<slug>/
(criada por você; o glob de workspace.members a inclui automaticamente —
NÃO edite o Cargo.toml raiz.)

ANTES DE CODAR: leia o mapa node↔tool em DIRETRIZ §3.8.1 (entry points,
contrato, vocab, templates) e copie o template do seu sabor.

O QUE VOCÊ FAZ (só dentro da sua pasta):
1. Cargo.toml: deps mínimas.
   [node]  ph2d-nodegraph, ph2d-node-registry, e ph2d-expr se usar math
           por-elemento.
   [tool]  ph2d-tool-registry (manifest + Zone) + ph2d-editor-core
           (Tool / FloatingPanel se stateful) + ph2d-a11y + ph2d-core +
           dom-específicas (ph2d-vector p/ ícone).
2. src/lib.rs: implemente o contrato (vide §3.8.1 entry point(s)).
   [node]  pub const MANIFEST: NodeManifest { id (NodeTypeId::of(
           "<dom>.<slug>")), name, inputs/outputs (PortSpec), effect
           (Pure | Temporal | Stateful), clock, params, lowerings }
           impl NodeOp { manifest(); eval(ctx) PURO — lê params via
                         ctx.param("nome"); cape produto via
                         param_as_count(v, max) se aloca; }
           pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError>
   [tool]  Escolha o sabor pela tabela §3.8.3 e siga o template:
           - (1) só pub const MANIFEST: ToolManifest + pub fn register
                 + src/algorithm.rs puro.
           - (2) src/tool.rs com impl Tool (id/label/icon_slug/build_panel/
                 handle_panel_event/as_any_mut) + pub fn make.
                 Brush é o único com is_default = true.
           - (3) os dois (manifest + behavior) + crate-irmão
                 ph2d-panel-<slug>/ (vide §3.2). handle_panel_event mapeia
                 NodeIds do panel pra apply_ui_edit(<UiEdit>::X) — único
                 site dos clamps/projeções. Para raster I/O do shell, siga
                 o padrão ATUAL: métodos próprios na concrete type
                 alcançados via as_any_mut downcast (vide BgRemoval /
                 Padding). impl ImageEditTool é fan-out futuro — vide
                 §3.8.3.1; NÃO mova um tool pra esse canal de carona.
3. [node] Teste golden: grafo source→seu-nó, register, g.validate(&ops),
         cook, asserta a saída.
   [tool] Testes do crate: register attaches manifest / make builds /
          panel layout / handle_panel_event clamping.
4. ÍCONE (somente tool — node não tem pill no chrome).
   [node] N/A.
   [tool] src/icon.rs com BezPath (porte do
         docs/design/icons/<slug>.svg, Lucide-style 24×24,
         stroke="currentColor"). Se a tool tem pill no chrome, adicione
         IconId variant em ph2d-editor-core/src/icons.rs em ORDEM
         ALFABÉTICA — o gate enum_order_matches_svgs falha se sair de
         ordem; NUNCA pule via --no-verify (quebra TODOS os ícones).
         Se não tem SVG source, peça ao Enio.

O QUE VOCÊ NÃO TOCA:
- Qualquer arquivo fora da sua pasta.
- 🔒 Contrato congelado (vide §3.8.1 cap). Mudança = Coordenador-only + ADR.
  [node]  ph2d-nodegraph, ph2d-expr, ph2d-node-registry,
          ph2d-node-registry-init/ (GERADO).
  [tool]  crates/ph2d-editor-core/src/tool.rs (Tool/ImageEditTool/
          PanelEvent), action_bus.rs::EditorAction (use os 4 genéricos),
          ph2d-tool-registry, ph2d-tool-registry-init/ (GERADO),
          o resto de ph2d-editor-core (foundational).
- Cargo.toml raiz (glob cobre você).

WIRING (sem colisão, sem edição central):
  cargo run -p ph2d-<family>-sync     # regenera o(s) register_all* + deps
  cargo test -p ph2d-<family>-registry-init     # staleness gate(s) fecham
  (staleness falha se esqueceu o sync; compilação do registry-init falha
   se seu register/make tem assinatura errada.)

VALIDAÇÃO (codificação rápida, §5):
  cargo check  -p ph2d-<family>-<slug>     # durante editing
  cargo test   -p ph2d-<family>-<slug>     # golden / unit
  cargo clippy -p ... --all-targets -- -D warnings
  cargo fmt -p ...

NOMES (gates ativos):
  [node]  type name canônico = "<domínio>.<slug>", único cross-crate
          (colisão pega no boot por RegistryError::Collision); atributos
          de stream e params: identificadores simples (sem espaço/ponto).
  [tool]  manifest id = "<slug>" único cross-crate (colisão pega no
          Registry::build); label_key segue "tool.<slug>.label".

SE PRECISAR DE ALGO FORA DA PASTA (dep externa nova, mudança no contrato
congelado, novo domínio/avaliador, variant novo em EditorAction): PARE e
reporte ao Enio (§2.4). Quase sempre significa que a tarefa não era
fan-out puro — vide triagem §1.4.

QUANDO TERMINAR, reporte ao Enio:
  "<Family> <slug> pronto. Commit local: <sha>. cargo test -p
   ph2d-<family>-<slug> e -p ph2d-<family>-registry-init verdes."
═══════════════════════════════════════════════════════════════════
```

#### 3.8.3 Sabores de tool (escolha o template antes de colar o briefing)

| Sabor | Expõe | Templates | Quando usar |
|-------|-------|-----------|-------------|
| **(1) One-shot stateless** | só `pub fn register` (manifest) | `-make-square/` · `-trim-transparency/` · `-real-size/` | Pill no chrome dispara algoritmo puro no Sprite ativo. Sem `impl Tool`. Shell drena via `EditorAction::OneShotImageOp`. |
| **(2) Palette modal sem manifest** | só `pub fn make` (`Box<dyn Tool>`) | `-brush/` (`is_default=true`) · `-move/` | Cursor de canvas, sem pill no chrome. `impl Tool` + `build_panel` Procreate-style + `handle_panel_event`. Sem `ToolManifest`. |
| **(3) Stateful + panel docado** | ambos `register` E `make` | `-padding/` (leve) · `-bgremoval/` (completo: preview cap + protect-mask + eyedropper via downcast) | Pill no chrome + panel próprio (`ph2d-panel-<slug>/`) + preview/commit raster. (1) + (2) + opcional `impl ImageEditTool` (vide §3.8.3.1). |

O `ph2d-tool-sync` é configurado pelas needles `"pub fn register("` (manifest) e `"pub fn make("` (behavior) — sabor (1) só entra em `register_all`, (2) só em `register_all_tools`, (3) entra nos dois.

#### 3.8.3.1 Status atual do trait `ImageEditTool` (heads-up importante)

O sub-trait `ImageEditTool` (`set_source` / `preview` / `take_pending_commit` / `run_full`) está **definido e congelado no contrato** (ADR-0040 §2.1), mas **nenhum tool de produção implementa hoje** (`grep -rn "impl ImageEditTool" crates/ph2d-tool-*/` retorna zero). BgRemoval e Padding seguem rodando via métodos próprios da concrete type (BgRemoval: `set_source_snapshot` / `run_full_resolution`; Padding: análogo) que o shell alcança via `as_any_mut` downcast. A migração para o canal genérico `ImageEditTool` é fan-out futuro (não bloqueia tool nova; pode ser feito como tarefa separada). **Use BgRemoval/Padding como template do PADRÃO ATUAL (downcast); só implemente `ImageEditTool` se você for o agente migrando algum tool existente pra esse canal — caso em que vire um vertical próprio, não scope creep do tool novo.**

#### 3.8.4 Por que é sem-colisão (a garantia, vale pras duas famílias)

Dois agentes adicionando duas features (mesma família ou não) **não tocam nenhum arquivo em comum**: cada um cria sua pasta; `workspace.members` é glob; superfícies centrais são geradas determinísticamente pelo sync entre marcadores codegen, e staleness gates pegam regen-esquecida. O contrato (`NodeOp`/`NodeManifest` em nodegraph; `Tool`/`ImageEditTool`/`PanelEvent` em editor-core) é o único acoplamento, e está congelado pelo arch-gate. **Para tool em particular**, a foundation `editor-core` está proibida de ganhar dep em qualquer `ph2d-tool-*` concreto (`editor_core_has_no_concrete_tool_deps`) — a única edge permitida é `tool-* → editor-core`.

#### 3.8.5 Checklist do revisor (se houver revisão)

**Comum às duas famílias:**

- [ ] `cargo run -p ph2d-<family>-sync` rodado; `cargo test -p ph2d-<family>-registry-init` verde.
- [ ] arch-gate do contrato congelado verde (sem cap-bust não-autorizado).
- [ ] clippy `--all-targets` + fmt limpos.
- [ ] sem dep fora do contrato; contrato congelado intocado; sem variant novo em `EditorAction` ou em ports/effects.

**Específico de node:**

- [ ] `MANIFEST` completo (params + lowerings preenchidos); nome canônico `"<dom>.<slug>"` único.
- [ ] `eval` puro (sem estado global, sem IO); efeito declarado bate (Stateful só se escreve sim); params lidos via `ctx.param`; alocação capada via `param_as_count`.
- [ ] teste golden presente e verde.

**Específico de tool:**

- [ ] `MANIFEST` completo (id único, cluster/zone/order coerentes com o palette) OU `is_default` correto (sabor 2 só Brush retorna true).
- [ ] Se stateful: `handle_panel_event` cobre 1:1 os NodeIds do panel docado; rota tudo via `apply_ui_edit` (sem duplicar clamps) — disciplina manual, sem arch-gate.
- [ ] Se for um vertical de migração para `ImageEditTool` (raro — vide §3.8.3.1): `as_image_edit_mut` retorna `Some(self)`; quadteto `set_source` / `preview` / `take_pending_commit` / `run_full` honra straight-alpha RGBA8. Caso contrário (padrão atual), raster I/O fica em métodos próprios da concrete type via `as_any_mut` downcast.
- [ ] Ícone: SVG em `docs/design/icons/` + IconId variant em ordem alfabética em `icons.rs`.

Estado vivo + loops autônomos:
- Sistema de nós: [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md) + [`docs/plans/2026-05-node-waves.md`](../plans/2026-05-node-waves.md).
- Sistema de tools: 🔒 CLOSED 2026-05-22 — [`docs/plans/2026-05-tool-isolation-waves.md`](../plans/2026-05-tool-isolation-waves.md); histórico do raciocínio em [`docs/HANDOFF_tool_isolation_close.md`](../HANDOFF_tool_isolation_close.md) (banner: "EXECUTADO em 2026-05-22").

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
| [`architecture_register_all_alphabetical`](../../crates/ph2d-tool-registry-init/tests/architecture_register_all_alphabetical.rs) | `register_all` / `register_all_tools` / Cargo deps fora da ordem alfabética (3 sub-checks) |
| [`staleness`](../../crates/ph2d-tool-registry-init/tests/staleness.rs) (tools) | tool-sync esquecido — `register_all` / `register_all_tools` / Cargo deps divergem do scan de `crates/ph2d-tool-*` (3 sub-checks) |
| [`staleness`](../../crates/ph2d-node-registry-init/tests/staleness.rs) (nodes) | node-sync esquecido — `register_all_nodes` / Cargo deps divergem do scan de `crates/ph2d-node-*` |
| [`architecture_panel_host_surface`](../../crates/ph2d-editor-core/tests/architecture_panel_host_surface.rs) | `PanelHost` cresce além de 12 métodos |
| [`architecture_cycle_prevention`](../../crates/ph2d-editor-core/tests/architecture_cycle_prevention.rs) | 3 invariantes (+ 1 smoke): (1) `editor-core` ⊥ `panel-*` / `ph2d-editor`; (2) `editor-core` ⊥ tool-* (exceto `ph2d-tool-registry` data contract); (3) `panel-*` depende de `editor-core` E não de `ph2d-editor` E não de outro `panel-*` |
| [`architecture_tool_contract_surface`](../../crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs) 🔒 | 🔒 ADR-0040: `Tool > 10 métodos` / `ImageEditTool > 4 métodos` / `PanelEvent > 4 variants` — mudar exige amendment de ADR-0040 §7 |
| [`architecture_contract_surface`](../../crates/ph2d-nodegraph/tests/architecture_contract_surface.rs) 🔒 | 🔒 ADR-0039: `NodeOp > 2 métodos` / `OpResolver > 1 método` / `NodeManifest > 8 campos` — mudar exige ADR estendendo 0039 |
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
| **Antes do push (paridade-CI completa, obrigatório)** | `./scripts/ship.sh` (fmt + clippy `--all-targets --features ph2d-spike/bevy_ecs` + machete + deny + audit + nextest `--workspace`; vide §7.0) | 3-8min warm |
| Diagnóstico isolado de clippy (subset do ship.sh) | `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | 1-3min warm |
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
| **Node crate novo (fan-out — caminho (A))** | `crates/ph2d-node-<domínio>-<slug>/` (wiring via `cargo run -p ph2d-node-sync`; gate `cargo test -p ph2d-node-registry-init`) |
| **Tool crate novo (fan-out — caminho (A))** | `crates/ph2d-tool-<slug>/` (wiring via `cargo run -p ph2d-tool-sync`; gate `cargo test -p ph2d-tool-registry-init`) |
| 🔒 Contrato de nós (congelado, ADR-0039) | `crates/ph2d-nodegraph/` + `crates/ph2d-expr/` (Coordenador-only + ADR) |
| 🔒 Contrato de tools (congelado, ADR-0040 §7) | `crates/ph2d-editor-core/src/tool.rs` (`Tool` + `ImageEditTool` + `PanelEvent`) + canal genérico em `crates/ph2d-editor-core/src/action_bus.rs` (`EditorAction::{ActivateTool, OneShotImageOp, ToolPanelEvent, CancelActiveTool}`) — Coordenador-only + ADR amendment |
| Painel novo (caminho (B)) | `crates/ph2d-panel-<slug>/` |
| Widget primitive (caminho (B)) | `crates/ph2d-editor-core/src/widget/<slug>.rs` |
| Chrome handler (caminho (B)) | `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` |
| Vocab UI de um tool (`<Slug>UiEdit`, `<Slug>UiSnapshot`, `<Slug>Params`) | `crates/ph2d-tool-<slug>/src/params.rs` (TG-B/TG-C — não mais em editor-core) |
| **Tool registry init (GERADO por `ph2d-tool-sync`)** | `crates/ph2d-tool-registry-init/src/lib.rs` (`register_all` manifests + `register_all_tools` `Box<dyn Tool>`) + `Cargo.toml` deps |
| **Node registry init (GERADO por `ph2d-node-sync`)** | `crates/ph2d-node-registry-init/src/lib.rs` (`register_all_nodes`) + `Cargo.toml` deps (2 staleness sub-checks: register_all_nodes + Cargo deps) |
| Codegen tool-sync | `tools/ph2d-tool-sync/` (lib + binário) |
| Codegen node-sync | `tools/ph2d-node-sync/` (lib + binário) |
| Panel registry init (manual — caminho (B)) | `crates/ph2d-panel-registry-init/src/lib.rs::register_all_panels` (features `panel-<slug>`) |
| Widget showcase | `crates/ph2d-editor-core/src/widget/showcase/` |
| Tokens source | `docs/design/tokens.json` |
| Tokens Rust | `crates/ph2d-tokens/src/` (codegen via build.rs) |
| Tool design TOML | `docs/design/tools/<slug>.toml` |
| Icon SVG | `docs/design/icons/<slug>.svg` |
| Mockup HTML | `docs/design/screens/*.html` |
| Workspace members (glob) | `Cargo.toml` raiz — `members = ["crates/*", "tools/*", "shells/desktop", "tests/spike"]` (qualquer pasta `crates/ph2d-{node,tool}-*` é coberta automaticamente) |
| Shell init (registro = 2 chamadas pós-`new`) | `shells/desktop/src/init.rs` — `let mut tools = ToolRegistry::new(); ph2d_tool_registry_init::register_all_tools(&mut tools); tools.activate_default();` (3 linhas, 2 chamadas) |
| Arch tests editor | `crates/ph2d-editor-core/tests/` |
| Arch tests tokens | `crates/ph2d-tokens/tests/` |
| Arch tests tool registry | `crates/ph2d-tool-registry-init/tests/` (3 staleness + 3 alfabéticos) |
| Arch tests node registry | `crates/ph2d-node-registry-init/tests/` (staleness) |
| Arch tests contrato tool 🔒 | `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs` |
| Arch tests contrato nodegraph 🔒 | `crates/ph2d-nodegraph/tests/architecture_contract_surface.rs` |

**Removido com ADR-0040 TG-D (`c4063b7`, 2026-05-22):** a pasta `crates/ph2d-editor-core/src/tools/` (que era onde viviam impl de tools antes do isolamento) **foi deletada**. Foundation ⊥ tools agora é gateado (`architecture_cycle_prevention::editor_core_has_no_concrete_tool_deps`). Se uma memória / doc antigo te apontar pra esse caminho, é stale.

### 9.3 Comandos mais usados

```bash
# Implementador — durante edição
cargo check -p ph2d-tool-<slug>
cargo test  -p ph2d-tool-<slug>
cargo test  -p ph2d-tool-<slug> -- some_pattern

# Coordenador — antes do push (paridade-CI completa, obrigatório)
./scripts/ship.sh   # fmt + clippy --all-targets --features ph2d-spike/bevy_ecs
                    # + machete + deny + audit + nextest --workspace (vide §7.0)

# Coordenador — push + babysit
git push origin main
gh run list --workflow=spike.yml --limit=1
gh run watch <id> --exit-status
```

---

## 10. Referências canônicas

- **Stack + Hard Rules + "Adicionar uma tool" em 3 passos:** [`SKILL_Stack_PH2D_Definitiva.md`](../../SKILL_Stack_PH2D_Definitiva.md)
- **Operacional dia-a-dia + CI:** [`CLAUDE.md`](../../CLAUDE.md)

**ADRs estruturais (ordem cronológica, leitura indispensável pra entender as duas famílias):**

- [ADR-0027 — Convention-by-discovery (tools como crates isolados, semente)](../architecture/decisions/0027-convention-by-discovery.md)
- [ADR-0028 — Wave 2 codegen + design canonical](../architecture/decisions/0028-wave-2-codegen-design-canonical.md)
- [ADR-0029 — Trait-driven panel host (Panel<State> + dual-path → typed-only)](../architecture/decisions/0029-trait-driven-panel-host.md)
- [ADR-0030 — Multi-domain node engine (decisão-mãe do sistema de nós)](../architecture/decisions/0030-multi-domain-node-engine.md)
- [ADR-0031 — Nó E ferramenta como unidade de feature (princípio FBP unificado)](../architecture/decisions/0031-node-and-tool-as-feature-unit.md)
- [ADR-0032 — `ph2d-nodegraph` substrato (7 primitivos + cook)](../architecture/decisions/0032-nodegraph-substrate.md)
- [ADR-0033 — Shared compute `ph2d-expr` (paridade CPU↔WGSL)](../architecture/decisions/0033-shared-compute-expr.md)
- [ADR-0034 — Plural evaluators (shader/audio/motion/gameplay)](../architecture/decisions/0034-plural-evaluators.md)
- [ADR-0035 — Cook vs live + attribute stream](../architecture/decisions/0035-cook-vs-live-and-attribute-stream.md)
- [ADR-0036 — Gameplay authoring (blocks + nodes → Luau)](../architecture/decisions/0036-gameplay-authoring-blocks-and-nodes.md)
- [ADR-0037 — Stable entity wire id + SceneDoc](../architecture/decisions/0037-stable-entity-wire-id-scenedoc.md)
- [ADR-0038 — Artist-first node UX](../architecture/decisions/0038-artist-first-node-ux.md)
- 🔒 [ADR-0039 — Nodegraph contract FREEZE (W2.T4)](../architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)
- 🔒 [ADR-0040 — Tool as isolated feature crate (FREEZE TG-E)](../architecture/decisions/0040-tool-as-isolated-feature-crate.md)

**Briefing de fan-out drop-crate (cobre as duas famílias):**

- **§3.8** — receita única: tabela node↔tool (§3.8.1), briefing parametrizado pronto-pra-colar (§3.8.2), sabores de tool (§3.8.3), garantia sem-colisão (§3.8.4), checklist do revisor (§3.8.5).
- Stub histórico do node-crate briefing original: [`briefing-node-crate.md`](briefing-node-crate.md) (redireciona pro §3.8).
- Histórico do raciocínio que fechou ADR-0040 (não vigente pra implementação nova): [`docs/HANDOFF_tool_isolation_close.md`](../HANDOFF_tool_isolation_close.md).

**Estado vivo + loops autônomos:**

- Sistema de nós — tracker vivo + loop autônomo do fan-out: [`docs/HANDOFF_node_system.md`](../HANDOFF_node_system.md)
- Plano do sistema de nós (status W1+W2 fechados, W3+ aberto): [`docs/plans/2026-05-node-waves.md`](../plans/2026-05-node-waves.md)
- Plano da isolação de tools (🔒 CLOSED 2026-05-22): [`docs/plans/2026-05-tool-isolation-waves.md`](../plans/2026-05-tool-isolation-waves.md)

**Migração + tese arquitetural:**

- [Tese node-centric (substrato unificado + avaliadores plurais)](../Migracao/2026-05-node-centric-architecture.md)
- [Três gargalos do paralelismo foundational](../Migracao/2026-05-foundational-parallelism-three-bottlenecks.md)

**Memória LLM (auto-loaded):** `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`

---

## 11. Versão + histórico

- **6.8 — 2026-05-22:** **Princípio transversal Tool↔Nó simétricos** + condensação. Os antigos §3.8 (node fan-out) e §3.9 (tool fan-out), que viraram clones um do outro depois de TG-E, foram unificados num só **§3.8 "Fan-out drop-crate (A)"** com tabela node↔tool (§3.8.1), briefing parametrizado pronto-pra-colar (§3.8.2), sabores de tool (§3.8.3), garantia sem-colisão (§3.8.4) e checklist do revisor (§3.8.5). §1.4 triagem promove **Tool nova ⇒ (A) Implementador-só** (era (B) por inércia textual). §3.1 reduzida a redirect pro §3.8. §3.5 "modificar existente" agora aponta `crates/ph2d-tool-<slug>/` (a pasta `editor-core/src/tools/` foi deletada em TG-D `c4063b7`) com mapa pasta-canônica-por-feature. §3.6 foundational lista AMBOS os contratos congelados (nodegraph/expr + Tool/ImageEditTool/PanelEvent) com tabela simétrica de caps + ADR. §4 gates ganha `architecture_tool_contract_surface` (🔒 caps 10/4/4) + `architecture_contract_surface` (🔒 caps 2/1/8) + staleness + cycle_prevention anotado com 4 sub-checks. §9.2 caminhos reorganizada (Tool/Node lado-a-lado como (A); contratos congelados explicitados; registry-init marcados GERADO; nota "`editor-core/src/tools/` deletado em TG-D"). §10 referências lista ADRs 0030..0040 + tese node-centric + plano tool-isolation CLOSED. Header + §11 enxutos.
- **6.7 — 2026-05-22:** ADR-0040 FECHADO via TG-A..TG-E — §3.1 neutralizado, §3.9 "Tool crate — fan-out" criado como irmão de §3.8 (unificado em 6.8). Arch-gate de panel auto-discover + cross-panel-dep ban + panel→tool edge codificada como permitida.
- **Histórico anterior (v6.0..v6.6 + v4.0/v5.0 arquivadas):** vide `git log docs/IntegracaoMultiAgente/DIRETRIZ.md`. Resumo: v6.0 (modelo 2 papéis Coord+Impl, fluxo invertido); v6.1 (perf audit + §3.7 + §5.6); v6.3 (§7.0 fast-mode/ship); v6.4 (§4.1 regras UI que queimaram); v6.5 (arquitetura node-centric); v6.6 (doc único + §1.4 triagem).

---

## 12. Quando esta diretriz fica obsoleta

Se a arquitetura mudar materialmente (ex.: surge um terceiro papel, ou o fluxo invertido vira fluxo lateral), atualize esta diretriz in-place e bump a versão. **Não fragmente em múltiplos docs** — a lição dos 4 docs antigos que dessincronizaram é que um doc único é mais fácil de manter atualizado.

Se você é LLM lendo isto depois de uma mudança arquitetural maior e a diretriz contradiz o código, **confie no código**, reporte ao Enio com diagnose, e atualize esta diretriz quando autorizado.
