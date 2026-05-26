# Handoff — Painter (sucessor do Procreate, padrão-ouro absoluto)

**Data:** 2026-05-25
**Para:** a LLM que vai começar a implementação do Painter.
**Como usar:** §0 e §1 são INSTRUÇÕES que governam seu comportamento. §2+ é como você se prepara. **Leia tudo + o plano + os docs canônicos antes de escrever uma linha de código.**

---

## 0. MANDATO (lê isto primeiro — governa tudo)

> **Padrão-ouro. Puro-sangue. O definitivo. Sem economias, sem gambiarras.**

- Você está construindo o **sucessor do Procreate** dentro do PH2D — não um clone multiplataforma, mas a **melhor versão que existe** de Paint stack 2D. Quando duas alternativas competem, vence a que entrega excelência sem cortar invariante.
- **Proibido:** corner-cut disfarçado de "v1"; `unwrap`/falha silenciosa onde cabe `Result`/erro real; assumir paridade sem prova; `TODO: depois` em coisa que dá pra fazer certo agora; copiar-colar com gambiarra em vez de extrair a abstração certa; pular smoke do Enio porque "testes passam".
- **Determinismo (HR-5)** respeitado onde aplica; documentado onde é isento. **Contratos minúsculos e gateados.** **Toda superfície pública documentada.** Testes cobrem feliz + edge + a classe-de-bug. **Anti-padrões já catalogados em [`docs/UI_Bugs/`](UI_Bugs/) e [`docs/Image Tools Bugs/`](Image%20Tools%20Bugs/) — releia antes de tocar em painter/dispatch/tool.**
- **A barra é "melhor que Procreate em 2D, com superioridade técnica genuína em pelo menos 5 dimensões"** (vide [`docs/Painter_projeto/14_inovacoes_extraordinarias.md`](Painter_projeto/14_inovacoes_extraordinarias.md)). O Enio confia em você. Gambiarra é "obrigado, mas não".

---

## 1. O LOOP (o protocolo — siga fase a fase, sem parar até precisar de smoke)

Para CADA task do plano ([`docs/Painter_projeto/15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md)):

1. **TRIAGEM primeiro** ([DIRETRIZ v7.0 §2](IntegracaoMultiAgente/DIRETRIZ.md)). Reporte ao Enio:
   ```
   TRIAGEM
   - Tarefa: <T-W.N: descrição em 1 linha>
   - Caminho: (A) drop-crate | (B) scaffold | (C) Coord-only
   - Toca contrato congelado (nodegraph/expr OU Tool/RasterEditTool/PanelEvent)?
       <Não | Sim — exige ADR + bump de cap>
   - Razão: <1-2 linhas>
   - Se grande/ambíguo: <peças isoláveis vs. compartilhadas>
   ```
2. **Sanity check obrigatório (DIRETRIZ v7.0 §0):**
   ```bash
   git log --oneline -5             # confirma HEAD
   git status -sb                   # working tree limpo?
   cargo check --workspace 2>&1 | tail -5    # baseline compila?
   ```
   Algo divergente → **pare e reporte ao Enio.**
3. **Build isolado:** sempre `source scripts/slot-env.sh <slot-id>` no início (DIRETRIZ v7.0 §1.2). Slot recomendado pra Painter: `impl-1`.
4. **Implemente no padrão-ouro** (§0). Cite o princípio no código quando ajudar a próxima LLM.
5. **Auto-verifique tudo verde:**
   - `cargo check -p <crate>`
   - `cargo test -p <crate>`
   - `cargo clippy -p <crate> --all-targets -- -D warnings`
   - `cargo fmt -p <crate> -- --check`
   - Gates relevantes (vide DIRETRIZ v7.0 §5.1).
6. **AUDITE — adversarial e independente.** Lance **≥2 auditores em paralelo** (Agent, `general-purpose`), cada um com lente distinta (corretude/edge-cases · paridade/determinismo · consistência docs↔código · qualidade gold-standard). Lente sugeridas por wave em [`15_plano_de_implementacao.md §21`](Painter_projeto/15_plano_de_implementacao.md). Instrua-os a serem **duros, caçar bugs/lacunas, dar severidade, NÃO validar por cortesia**.
7. **CORRIJA TODOS os achados** (Crítico→Baixo). **Nada adiado** — exceto follow-ups genuinamente não-bloqueantes, registrados explicitamente no plano §follow-ups com justificativa.
8. **RE-AUDITE até erro zero.** "Sem achados" = de verdade nenhum, não "bom o bastante".
9. **Smoke intra-wave do Enio (quando aplicável):** as waves grandes têm smokes 🟦 ao longo do progresso (vide [`15_plano_de_implementacao.md §2`](Painter_projeto/15_plano_de_implementacao.md) tabela). **Pare e peça o smoke** quando atingir um marco 🟦. Não passe sem aprovação.
10. **Commit** (`git commit --no-verify` em background; local; um commit limpo por task ou conjunto coeso de tasks). Atualize plano + todos.
11. **Próxima task.** Volte ao 1.

### Quando PARAR (e só então)
- A task atinge um **smoke 🟦 intra-wave** (vide tabela §2 do plano).
- A wave inteira fechou — pede **smoke do Enio principal**.
- Uma decisão de **FREEZE** do contrato (mudar ADR-0043..0049 = evento Coord-A-only).
- Mudança em **foundational genuinamente compartilhado** que exige coordenação humana (vide DIRETRIZ v7.0 §3.C).
- Ao parar: **relatório curto pro Enio** — o que ficou pronto, o que ele precisa olhar, e por quê.

### NÃO faça autonomamente
- **`git push` / CI** — é o "ship" de fim-de-jornada, sob ordem do Enio. Acumule commits locais; só ele dispara o push (vide DIRETRIZ v7.0 §8).
- **Tocar contratos congelados** (`ph2d-nodegraph`, `ph2d-expr`, `editor-core/src/tool.rs`, `action_bus.rs::EditorAction`) — é Coord-A only + ADR (vide DIRETRIZ v7.0 §4).
- **Tocar shells/* sem necessidade clara** — são foundational/visual, prerrogativa Coord-A.
- **Mudar plano sem autorização** — se descobrir que uma task tá errada/incompleta, **pare e reporte**, não invente reescritas.

---

## 2. ANTES DE TOCAR EM QUALQUER COISA — leitura obrigatória

Leia **nesta ordem**, sem pular. Tempo total estimado: 3-5h para internalização completa.

### 2.1 Tier 1 — Docs canônicos (governa TUDO)

1. [`SKILL_Stack_PH2D_Definitiva.md`](../SKILL_Stack_PH2D_Definitiva.md) — **fonte de verdade da engine**. Foco em §1-§4 (visão + plataformas), §5 (stack), §6 (arquitetura), §9 (HR-1..HR-18 — invariantes inegociáveis), §11.9 (Editor UI), §17 (Definition of done global).
2. [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md) — **v7.0 (2026-05-24)**. Foco em §0 (sanity check), §1 (papéis + infra multi-agente), §2 (TRIAGEM — você fará para cada task), §3.A (drop-crate fan-out), §3.A.4 (`RasterEditTool` padrão crítico), §4 (contratos congelados — NÃO TOQUE), §5 (UI canônica + gates), §6 (codificação rápida), §7 (anti-colisão git), §8 (ship + push + CI).
3. [`CLAUDE.md`](../CLAUDE.md) — operacional dia-a-dia + CI.
4. [`docs/HANDOFF_node_system.md`](HANDOFF_node_system.md) — **modelo do mandato §0 padrão-ouro** que governa este handoff também. Leia §0 + §1.

### 2.2 Tier 2 — Spec do Painter (o que implementar)

Em ordem do índice de [`docs/Painter_projeto/README.md`](Painter_projeto/README.md):

5. [`Painter_projeto/README.md`](Painter_projeto/README.md) — visão + escopo IN/OUT + roadmap 17 waves + decisões estruturais §11 + ADRs prereq §11.C.
6. [`Painter_projeto/01_brush_engine.md`](Painter_projeto/01_brush_engine.md) — motor de pincel (Shape×Grain + Mixbox + Procedural Grain + MCP Stroke Engine + Stroke Vector History).
7. [`Painter_projeto/02_layers.md`](Painter_projeto/02_layers.md) — camadas (7 kinds + 22 blend modes + Adjustment Layers).
8. [`Painter_projeto/03_color.md`](Painter_projeto/03_color.md) — sistema de cor (5 modos + ColorDrop + Eyedropper).
9. [`Painter_projeto/04_canvas_guides.md`](Painter_projeto/04_canvas_guides.md) — canvas + guides + QuickShape + Reference Companion.
10. [`Painter_projeto/05_gestos_input.md`](Painter_projeto/05_gestos_input.md) — gestos + QuickMenu + atalhos desktop.
11. [`Painter_projeto/06_selection_transform_adjustments.md`](Painter_projeto/06_selection_transform_adjustments.md) — selection (4 tipos) + transform (4 modos) + 17 adjustments.
12. [`Painter_projeto/07_pencil_pipeline.md`](Painter_projeto/07_pencil_pipeline.md) — pipeline de input (Pencil/tablet/mouse + curves).
13. [`Painter_projeto/08_performance_memory.md`](Painter_projeto/08_performance_memory.md) — **budgets HR-3/4/13** — Painter cabe em 4.5ms sub-budget Render.
14. [`Painter_projeto/09_export_interop.md`](Painter_projeto/09_export_interop.md) — formats + PSD interop + brush CLI.
15. [`Painter_projeto/10_animation_assist.md`](Painter_projeto/10_animation_assist.md) — timeline frame-based.
16. [`Painter_projeto/11_ux_chrome.md`](Painter_projeto/11_ux_chrome.md) — chrome + sidebar Procreate-style + takeover mode + Settings cascade integration.
17. [`Painter_projeto/12_fora_de_escopo.md`](Painter_projeto/12_fora_de_escopo.md) — 18 não-objetivos explícitos. Leia para saber o que NÃO implementar.
18. [`Painter_projeto/13_referencias.md`](Painter_projeto/13_referencias.md) — fontes externas + crates PH2D referenciados + ADRs.
19. [`Painter_projeto/14_inovacoes_extraordinarias.md`](Painter_projeto/14_inovacoes_extraordinarias.md) — **5 propostas + Crítica A absorvidas integralmente**. Mandato Enio: padrão-ouro absoluto.

### 2.3 Tier 3 — Plano executável (o que fazer)

20. [`Painter_projeto/15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md) — **EXAUSTIVO — 2201 linhas**. Foco em §0 (princípios), §1 (DoD global), §2 (tabela de smokes), §3 (W0 — começa AQUI), §25 (12 anti-padrões transversais), §26 (hierarquia de fontes).

### 2.4 Tier 4 — Estado vivo + bug bases

21. [`docs/UI_Bugs/README.md`](UI_Bugs/README.md) + [`docs/Image Tools Bugs/README.md`](Image%20Tools%20Bugs/README.md) — bugs já catalogados que **NÃO se repete** (anti-padrões em código).
22. [`docs/plans/2026-05-ui-source-of-truth.md`](plans/2026-05-ui-source-of-truth.md) — Fase 1.A ativa: Widget Gallery + Inspector + Hierarchy como **3 painéis canon**. Painter panel copia padrões deles.
23. [`docs/UI_Fonts/2026-05-25-crisp-heavy-implementation.md`](UI_Fonts/2026-05-25-crisp-heavy-implementation.md) — sistema Text rendering que Painter chrome consome.

### 2.5 Tier 5 — Memória LLM (auto-loaded)

24. `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md` — perfil Enio + feedback acumulado. **Importante:** verifica memórias com `feedback_*` para o estilo do Enio (pt-BR direto, opções concretas, decisão recomendada primeiro, sem AskUserQuestion de 4 perguntas, sem anticipar decisões não-pedidas).

### 2.6 Templates canônicos a estudar antes de criar crate novo

Antes de criar `ph2d-tool-painter`, leia o template canônico:

- [`crates/ph2d-tool-bgremoval/`](../crates/ph2d-tool-bgremoval/) — **sabor 3 completo** (Tool + RasterEditTool + Panel + bridge no shell + cache zeroing em deactivate). É o padrão-mestre da Wave 10 — Painter segue.
- [`crates/ph2d-panel-bgremoval/`](../crates/ph2d-panel-bgremoval/) — Panel docado template.
- [`shells/desktop/src/render_loop/bgremoval_preview.rs`](../shells/desktop/src/render_loop/bgremoval_preview.rs) — **bridge canônico** que `painter_bridge.rs` (futuro) deve espelhar.
- [`crates/ph2d-tool-runtime/src/lib.rs`](../crates/ph2d-tool-runtime/src/lib.rs) — 4 drivers helpers (`drive_source_push`, `drive_preview_cache`, `drive_pending_commit`, `drive_deactivate_cleanup`) que Painter consome.
- [`crates/ph2d-panel-widget-gallery/`](../crates/ph2d-panel-widget-gallery/) + [`crates/ph2d-panel-inspector/`](../crates/ph2d-panel-inspector/) + [`crates/ph2d-panel-hierarchy/`](../crates/ph2d-panel-hierarchy/) — **3 painéis canon UI**. Painter panel copia padrões.
- [`crates/ph2d-editor-core/src/tool.rs`](../crates/ph2d-editor-core/src/tool.rs) — contrato `Tool` + `RasterEditTool` + `PanelEvent`. **Congelado por ADR-0040+0041** — NÃO modifique.

---

## 3. PONTO DE ENTRADA — comece AQUI

### Estado atual em 2026-05-26 (W0 ratificado + T1.1+T1.2+T1.3 fechados)

**Cascata W0:** 11 ADRs Accepted (0043..0053). Ratificada 2026-05-26 após 4-5 ondas de auditoria adversarial. Vide `MEMORY.md` → `project_painter_w0_ratified_2026_05_26.md`. Regra ativa: `feedback_perfection_no_deferrals` (gaps conhecidos viram trabalho na sessão atual).

**Spec do Painter:** completo, 17 waves, 11 ADRs em [`docs/architecture/decisions/`](architecture/decisions/), tasks em [`15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md).

**Código já entregue (W1 parcial — esta sessão 2026-05-26):**
- **T0.8** — `crates/ph2d-painter-contracts/` homestead arch-gate (74 tests cumulativos, vacuous-pass conforme crates filhos nascem)
- **T1.1** — `crates/ph2d-tool-painter/` skeleton + `RasterEditTool` stub neutro + IconId::Painter + SVG/TOML
- **T1.2** — pill dispatch data-driven + reconcile `ButtonState` ↔ `tools.active()` shell-side
- **T1.3** — `crates/ph2d-painter-brush/` skeleton (Brush + 12 sub-structs + Stamp 96B ABI freeze + BrushHandle + RenderingMode 6 FROZEN + PigmentMode + GrainSource + ProceduralGrain + library ROUND_HARD + atlas/mixbox stubs). 28 tests + arch-gates ATIVAM corretos cumulativos.

**Próxima task literal:** **T1.4 — Stamp compute pipeline GPU** (ADR-0044 §1.8.2) + **T-color (ADR-0051) paralelo** + **T-tier (ADR-0053) paralelo** (vide [§4.X do plano](Painter_projeto/15_plano_de_implementacao.md)).

### Tasks paralelas obrigatórias em W1 (§4.X — regra perfeição 2026-05-26)

Por audit 2026-05-26 gap 2+3, plano §4.X especifica 5 tasks paralelas que precisam estar disponíveis ANTES do smoke Day 7:

- **T-input** (ADR-0050) — `crates/ph2d-painter-input/` (PointerSource + PressureCurve per-device + PalmRejection + DriverQuirk). **Precede T1.5 brush impl** (Stamp tilt/pressure exigem isso).
- **T-color** (ADR-0051) — extensão `ph2d-color` (ColorProfile FROZEN 8 + LinearSrgb working + Mixbox em LinearSrgb invariant). **Paralelo a T1.3** (já fechado nesta sessão sem T-color — mas T1.4+ exige).
- **T-durability** (ADR-0052) — extensão `ph2d-painter-stroke::durability/` (WAL + CrashRecovery + AutoSave + SuspendHandler). **Paralelo a T1.4** stroke commit.
- **T-tier** (ADR-0053) — extensão `ph2d-host` (DeviceCapability + DeviceTier FROZEN 5 + GpuId + ThermalState). **Paralelo a T1.3** (Mixbox GPU vs CPU choice).
- **T-codegen-stateful** — extensão `tool-sync` para regenerar `STATEFUL_TOOL_IDS` em `chrome/image_actions.rs` + os 2 testes hand-maintained em `tool-registry-init` (memória `feedback_fanout_registry_init_friction`).

Estimativa total W1: 14-18 dias (era 10-14; bumpado pós-audit para acomodar paralelas).

### O caminho do W0 (FECHADO ✅)

~~T0.1 → T0.2 → ... → T0.7 → T0.8 → T0.9 aprovação~~. **Cascata W0 RATIFICADA em 2026-05-26.** 11 ADRs Accepted. T0.9 ✓. W1 abre.

### W1 em andamento

W1 é **caminho (A) drop-crate fan-out**. Implementador cria crates satélites; tool-sync regenera registry-init; arch-gates de `ph2d-painter-contracts` ativam automaticamente.

**Milestone-marker do W1: Day 7 = primeira pintura.** Se Day 7 vier e não há primeira pintura, plano para e revisita.

**Próximo: T1.4** — Stamp compute pipeline GPU (`ph2d-painter-brush::StampPipeline`). Spec em ADR-0044 §2.9 (W1 unified shader rampa) + spec §1.8.2 (workgroup model 8×8).

---

## 4. Arquitetura — onde tudo vive (mapa rápido)

### Crates que o Painter consome (existentes — NÃO crie duplicado)

| Crate | Papel |
|-------|-------|
| `ph2d-editor-core` | `Tool` + `RasterEditTool` + `PanelEvent` traits + chrome handlers + widgets canon |
| `ph2d-tool-runtime` ✨ | **4 drivers helpers** sobre `RasterEditTool` (Wave 10/ADR-0042); consume |
| `ph2d-tool-registry` | `ToolManifest` + `hash_node_id` |
| `ph2d-tokens` | OKLCH colors + Spacing + Radius + TypeToken (inc. TextRendering enum) |
| `ph2d-text` | parley wrapper (com Crisp Heavy axis support) |
| `ph2d-vector` | Vello wrapper + BezPath para ícones |
| `ph2d-a11y` | AccessKit wrapper |
| `ph2d-gpu` | wgpu 28 wrapper para stamp pipeline compute |
| `ph2d-host` | trait PlatformHost (inc. `gyroscope()` para W15 Fluid) |
| `ph2d-input` | Pencil/touch/mouse pure-data |

### Crates novos que o Painter cria (família drop-crate)

| Crate | Wave | Conteúdo |
|-------|------|----------|
| `ph2d-tool-painter` | W1 | impl Tool + impl RasterEditTool + manifest |
| `ph2d-painter-brush` | W1 | brush engine (Shape×Grain + Mixbox + Procedural Grain) |
| `ph2d-painter-stroke` | W1 | Stroke Vector History full + undo + Reproject |
| `ph2d-panel-painter` (sidebar) | W2 | sidebar Procreate-style |
| `ph2d-painter-color` | W7 | 5 color modes |
| `ph2d-panel-painter-layers` | W3 | layer panel docked |
| `ph2d-panel-painter-brush-studio` | W5 | Brush Studio (12 sub-sections) |
| `ph2d-panel-painter-inspector` | W14 | Stroke Inspector retroativo |
| `ph2d-painter-anim` | W11 | Animation Assist |
| `ph2d-painter-fluid` | W15 | Fluid Brushes Extension (opt-in) |

### Bridge no shell

| Arquivo | Wave |
|---------|------|
| `shells/desktop/src/render_loop/painter_bridge.rs` ✨ NEW | W1 (espelha bgremoval_preview.rs) |

### 5 codegens (rodam quando relevante)

- `cargo run -p ph2d-tool-sync` — regenera tool-registry-init quando crate tool novo aparece
- `cargo run -p ph2d-panel-sync` — regenera panel-registry-init
- `cargo run -p ph2d-chrome-sync` — regenera chrome handlers dispatch
- `cargo run -p ph2d-widget-sync` — regenera widget showcase index
- `cargo run -p ph2d-node-sync` — irrelevante pro Painter (não cria nodes em v1)

### Contratos congelados que NÃO se toca (sem ADR amendment)

🔒 **`crates/ph2d-editor-core/src/tool.rs`** (ADR-0040 + ADR-0041) — `Tool ≤ 10` métodos, `RasterEditTool ≤ 5` métodos, `PanelEvent ≤ 4` variants. Cap-bust = build vermelho.

🔒 **`crates/ph2d-editor-core/src/action_bus.rs`** — `EditorAction` 4 variants genéricos (`ActivateTool`, `OneShotImageOp`, `ToolPanelEvent`, `CancelActiveTool`). **Sem variant novo per-tool.**

🔒 **`crates/ph2d-nodegraph/`** + **`crates/ph2d-expr/`** (ADR-0039) — irrelevante pro Painter v1, mas não toque.

### ADRs a criar antes de W1 (W0 do plano)

| ADR | Conteúdo | Task |
|-----|----------|------|
| 0043 | Painter contract | T0.1 |
| 0044 | Brush Engine GPU + Mixbox + Procedural Grain | T0.2 |
| 0045 | Adjustment Layers | T0.3 |
| 0046 | Stroke Vector History | T0.4 |
| 0047 | Painter MCP Stroke Engine | T0.5 |
| 0048 | Stroke Inspector retroativo | T0.6 |
| 0049 | Fluid Brushes Extension | T0.7 |

ADR-0040, 0041, 0042 **já estão aprovados** (vide §10). Pré-requisitos do Painter.

---

## 5. Armadilhas conhecidas (NÃO repita — anti-padrões transversais)

Os 12 anti-padrões mais críticos. Lista completa em [§25 do plano](Painter_projeto/15_plano_de_implementacao.md).

1. **Hex color literal em paint** (`Color::rgba8(0x12, 0x34, ...)`) — quebra `no_literal_color`. **Use `ColorToken::X.resolve(theme)`.**
2. **`f32` literal de UI fora do allowlist** — quebra `no_magic_numeric`. **Use `Spacing::Lg.px()`, `Radius::Md.px()`.**
3. **String hardcoded em UI** — quebra `hr15_no_hardcoded_ui_strings`. **Use `t!()` Fluent.**
4. **Widget sem `Node` AccessKit** — quebra `hr12_widgets_a11y`.
5. **Mirror manual slider↔chip** — **use `store.link_slider_number(slider_id, chip_id)` no populate** (DIRETRIZ v7.0 §5.2 regra 1).
6. **Chip pill sem stepper** se Fase 1.A do `2026-05-ui-source-of-truth.md` fechou — canon mudou; copie do Inspector/Gallery atual.
7. **`std::HashMap` em compositor / state lateral** — **use `BTreeMap`/`EntityHashMap`** (HR-5).
8. **Alocação em hot path** (`Box::new`, `Vec::push` que realoca) — HR-3. Gate `painter_no_alloc_hot_path` (dhat-rs).
9. **Skip `RasterEditTool::deactivate` cache zero** — stale-frame bug (auditoria Wave 10 §A1+A2).
10. **`--no-verify` em commit** — fix root cause (DIRETRIZ v7.0 §7.2).
11. **Hardcoded `id == "painter"` em múltiplos sites** — **use data-driven `is_painter_tool` helper** (Image Tools Bugs §2.b).
12. **Hit-test e paint de widget gateados por condições diferentes** — Image Tools Bugs §2.c. **Use a MESMA flag/expressão pra ambos.**

### Armadilha estrutural específica da Wave 10

13. **Reimplementar `drive_*` helpers no bridge** — **use [`ph2d-tool-runtime`](../crates/ph2d-tool-runtime/)** (`drive_source_push`, `drive_preview_cache`, `drive_pending_commit`, `drive_deactivate_cleanup`). Bridge usa, não duplica.

### Armadilha de fan-out drop-crate (DIRETRIZ v7.0 §3.A.2)

14. **`Cargo.toml` antes de `lib.rs`** — quando você cria `crates/ph2d-tool-painter/`, **lib.rs PRIMEIRO** (mesmo com 1 linha `#![forbid(unsafe_code)]`). Cargo recusa manifest enquanto crate-novo não tem lib.rs; workspace usa glob `crates/*`, então **TODAS as outras sessões paralelas ficam bloqueadas** com "can't find library X" até esse arquivo existir.

### Armadilha do IconId (gate `enum_order_matches_svgs`)

15. **SVG novo em `docs/design/icons/` SEM IconId variant em ordem alfabética** — quebra `ICON_CMDS_BY_ID` e **TODOS os ícones quebram**. Quando adicionar IconId `Painter`, é alfabético em [`crates/ph2d-editor-core/src/icons.rs`](../crates/ph2d-editor-core/src/icons.rs).

---

## 6. Como reportar progresso

### Quando completar uma task

```
T-W.N: <título da task> pronto.
Commit local: <sha>.
cargo test -p <crate> verde.
Próxima task: T-W.N+1.
```

### Quando atingir um smoke 🟦 intra-wave

```
🟦 SMOKE-INTRA W.N — pronto pro Enio testar.
O que ver: <descrição literal>
Como testar: ./play.command + <passos>
Aguardo aprovação ou findings.
```

### Quando wave fechar

```
✓ Wave N - <título> pronta.
Commits locais: <sha1>..<shaN>.
Auditoria adversarial 2 lentes concluída a erro-zero.
Smoke principal: aguardando aprovação do Enio.

DoD checklist (§1 do plano):
  ☑ <itens 1-11 com check ou explicação>
```

### Quando bloqueado

```
Implementador slot N bloqueado em T-W.N.
Razão: <descrição precisa>
Preciso que o Coord faça: <ação específica>
Continuo após scaffold/decisão extra.
```

---

## 7. Coisas que parecem óbvias mas vou repetir

- **TRIAGEM é seu PRIMEIRO output** quando o Enio te passa uma task. Não comece a codar antes de triar (DIRETRIZ v7.0 §2).
- **PT-BR é o idioma de conversa.** Código em inglês, comentários em inglês curto, conversa de design em pt-BR.
- **Não confunda os ADRs:** ADR-0040 / 0041 / 0042 já existem (Tool isolation / RasterEdit rename / Wave 10 closure). ADRs do Painter são **0043-0049** (renumerados em 2026-05-25).
- **`ImageEditTool` foi renomeado para `RasterEditTool`** (ADR-0041). 5 métodos (`set_source` / `current_preview` / `take_pending_commit` / `run_full` / `deactivate`). Se você vir docs antigos com `ImageEditTool`, **trate como stale**.
- **DIRETRIZ v7.0 ganha sobre SKILL_Stack v2.14** quando há contradição (v7.0 é mais novo).
- **O Enio prefere opções concretas com decisão recomendada primeiro** (memória `feedback_communication_style`). Não use AskUserQuestion com 4 perguntas — responda direto.
- **Quando memória diz X mas código diz Y:** confie no código. Atualize memória depois.

---

## 8. Quando você ficar em dúvida (hierarquia de decisão)

Vide [§26 do plano](Painter_projeto/15_plano_de_implementacao.md):

1. **Plano** ([`15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md)) — o quê fazer.
2. **Spec docs** ([`01`](Painter_projeto/01_brush_engine.md) - [`14`](Painter_projeto/14_inovacoes_extraordinarias.md)) — o que implementar.
3. **DIRETRIZ v7.0** — como implementar (triagem, isolamento, gates).
4. **SKILL_Stack** — Hard Rules + arquitetura.
5. **ADRs 0040..0049** — contratos imutáveis.
6. **Code existente** (`bgremoval` template) — padrão visual.
7. **Bug docs** (`UI_Bugs/`, `Image Tools Bugs/`) — o que NÃO repetir.
8. **Memória persistente LLM** — perfil Enio + feedback.

Se a dúvida não cabe em nenhuma das 8: **pergunte ao Enio antes de implementar**. Não adivinhe arquitetura.

Se uma decisão arquitetural surgir que cruza os 8 níveis: apresente **opções concretas com recomendação + tradeoff** (memória `feedback_communication_style`).

---

## 9. Memorando final pro próximo agente

Você está pegando um spec maduro (16 arquivos + plano de 2201 linhas) + uma engine bem estruturada (Wave 10 fechada, ADR-0040/0041/0042 ratificados, 3 tools de produção já implementam RasterEditTool, infra Wave 10 pronta).

**O que você NÃO faz:** "v1 que dá pro gasto", gambiarra, atalhos. Reler §0.

**O que você FAZ:** padrão-ouro absoluto, smoke cedo, audit adversarial obrigatória, reportar honestamente quando dúvida.

**Confiança:** O Enio confia em você. Esse handoff existe pra te dar contexto suficiente para começar com pé direito. Se algo te parece estranho ou contraditório, **pare e pergunte**. É mais barato perguntar 5 minutos do que retrabalhar 5 dias.

A barra é: **Painter PH2D deve ser melhor que Procreate em pelo menos 5 dimensões** (vide [14_inovacoes_extraordinarias §14.8.1](Painter_projeto/14_inovacoes_extraordinarias.md)). Mandato Enio 2026-05-23: tempo maior aceito como custo de superar Procreate em vez de igualar.

**Próximo passo concreto:** leia tudo nesta lista (Tier 1-5, ~3-5h), depois faça TRIAGEM da T0.1 (ADR-0043 Painter contract) e reporte ao Enio.

---

## 10. Referências canônicas (resumo final)

### Docs governantes

- [SKILL_Stack_PH2D_Definitiva.md](../SKILL_Stack_PH2D_Definitiva.md)
- [DIRETRIZ v7.0](IntegracaoMultiAgente/DIRETRIZ.md)
- [CLAUDE.md](../CLAUDE.md)
- [HANDOFF_node_system.md](HANDOFF_node_system.md) (modelo do mandato §0)

### Spec Painter (16 arquivos)

- [README.md](Painter_projeto/README.md) (índice)
- [01_brush_engine.md](Painter_projeto/01_brush_engine.md)
- [02_layers.md](Painter_projeto/02_layers.md)
- [03_color.md](Painter_projeto/03_color.md)
- [04_canvas_guides.md](Painter_projeto/04_canvas_guides.md)
- [05_gestos_input.md](Painter_projeto/05_gestos_input.md)
- [06_selection_transform_adjustments.md](Painter_projeto/06_selection_transform_adjustments.md)
- [07_pencil_pipeline.md](Painter_projeto/07_pencil_pipeline.md)
- [08_performance_memory.md](Painter_projeto/08_performance_memory.md)
- [09_export_interop.md](Painter_projeto/09_export_interop.md)
- [10_animation_assist.md](Painter_projeto/10_animation_assist.md)
- [11_ux_chrome.md](Painter_projeto/11_ux_chrome.md)
- [12_fora_de_escopo.md](Painter_projeto/12_fora_de_escopo.md)
- [13_referencias.md](Painter_projeto/13_referencias.md)
- [14_inovacoes_extraordinarias.md](Painter_projeto/14_inovacoes_extraordinarias.md)
- [15_plano_de_implementacao.md](Painter_projeto/15_plano_de_implementacao.md) ⭐ **plano executável**
- [avaliacao_e_melhorias.md](Painter_projeto/avaliacao_e_melhorias.md) (referência histórica DeepMind)

### ADRs (10 relevantes)

- 0019 Spike scripting (Luau)
- 0020 Surface lifecycle
- 0021 Sim ↔ Present boundary (Painter está em Present)
- 0022 No HashMap in sim
- 0027 Convention-by-discovery
- 0029 Trait-driven panel host
- 🔒 0039 Nodegraph contract FREEZE
- 🔒 0040 Tool as isolated feature crate
- 🔒 0041 RasterEdit rename + deactivate
- 0042 Wave 10 closure (`ph2d-tool-runtime`)

### A criar (W0)

- 0043 Painter contract
- 0044 Brush Engine GPU + Mixbox + Procedural Grain
- 0045 Adjustment Layers
- 0046 Stroke Vector History
- 0047 Painter MCP Stroke Engine
- 0048 Stroke Inspector retroativo
- 0049 Fluid Brushes Extension

### Templates canônicos a estudar

- [`ph2d-tool-bgremoval`](../crates/ph2d-tool-bgremoval/) — sabor 3 completo
- [`ph2d-panel-bgremoval`](../crates/ph2d-panel-bgremoval/) — panel docado template
- [`shells/desktop/src/render_loop/bgremoval_preview.rs`](../shells/desktop/src/render_loop/bgremoval_preview.rs) — bridge canônico
- [`ph2d-tool-runtime/src/lib.rs`](../crates/ph2d-tool-runtime/src/lib.rs) — 4 drivers helpers
- [`ph2d-panel-widget-gallery`](../crates/ph2d-panel-widget-gallery/) + [`ph2d-panel-inspector`](../crates/ph2d-panel-inspector/) + [`ph2d-panel-hierarchy`](../crates/ph2d-panel-hierarchy/) — 3 painéis canon UI

---

**Confiança:** spec maduro + plano detalhado + infra Wave 10 pronta. O caminho está claro. Mandato é padrão-ouro absoluto, sem economias. Vai com tudo e reporta cedo.
