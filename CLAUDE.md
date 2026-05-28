# CLAUDE.md — diretrizes operacionais para LLM no PH2D

Vide [`SKILL_Stack_PH2D_Definitiva.md`](SKILL_Stack_PH2D_Definitiva.md) para
arquitetura, stack, Hard Rules (HR-1..HR-17), convenções e tudo que é
**técnico do projeto**. Este arquivo é APENAS workflow operacional —
quem faz o quê, quem confere o quê.

## CI / GitHub Actions

**Implementador:** não faz `git push` e não monitora CI. Apenas reporta commit local pronto. Coordenador faz o push.

**Coordenador (absorve o papel antes chamado PRCI):**
A run completa de CI demora ~30min (matrix linux + macOS + windows
+ replay hash + bench). Por isso push pro GitHub é feito **uma
vez por ciclo, ao final**, e o **Coordenador fica responsável por
babysit da CI até ela passar**. Protocolo em
[`docs/IntegracaoMultiAgente/DIRETRIZ.md`](docs/IntegracaoMultiAgente/DIRETRIZ.md) §7:
1. Após `git push`, polling com intervalo de **15min**
   (`Monitor` com `sleep 900` ou `gh run watch`).
2. Se a run falhar, Coordenador diagnostica + corrige + push + retoma
   o polling com a nova run.
3. Loop fecha quando CI conclui `success` ou após **3 ciclos
   consecutivos de falha do mesmo job** (aí escalona pro Enio).

Após `git push`, **forneça SEMPRE o link da run**:
`https://github.com/dibrioli/PH2D/actions/runs/<run-id>`.
Use `gh run list --workflow=spike.yml --limit=1` para pegar o ID.
Se um job falhar, forneça também link direto do job que falhou
(`gh run view --job=<job-id>`).

## Memória persistente da LLM

Vide [`~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md)
para feedback acumulado, perfil do Enio, estado do projeto, paths canônicos.
LLM nova chegando lê esse índice antes de tomar ações.

## Cadência de validação (codificação rápida)

Vide [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](docs/IntegracaoMultiAgente/DIRETRIZ.md) §5 —
quando rodar `cargo check -p <crate>` vs `cargo test --workspace`,
quando confiar no pre-commit hook em vez de duplicar a validação, e
quando granular-commitar vs acumular em blocos durante Waves. **LLM
deve ler antes de começar refactor multi-arquivo** — over-validation
mata produtividade (5-10min de espera por commit quando o hook já
roda a mesma matriz).

## Fluxo de trabalho: fast mode (dia) / ship (fim do dia)

Vide [`DIRETRIZ.md`](docs/IntegracaoMultiAgente/DIRETRIZ.md) §7.0.
**De dia, implemente sem fricção:** checkpoints com `git commit
--no-verify` (instantâneo, pula o hook), `cargo check -p <crate>` quando
quiser, **zero push / zero CI**. **No fim do dia**, quando o Enio mandar
("commit" / "push" / "ship" / "fim do dia"), entre em **modo
observa-e-corrige** e entregue commits + push + CI verdes **sem falta**:
1. `./scripts/ship.sh` (paridade EXATA com a job de lint+test do CI —
   fmt, clippy `--all-targets`+features, machete, deny, audit, nextest;
   o pre-commit hook NÃO cobre isso). 2. Corrija TODO `✗` e re-rode até
verde — **não pusha antes**. 3. Push + babysit do CI (§7.3) até verde,
corrigindo o que aparecer. O erro de CI é pego no ship.sh **antes** do
push, não no CI vermelho 30min depois.

## Planos operacionais ativos

- [`docs/plans/2026-05-node-waves.md`](docs/plans/2026-05-node-waves.md) —
  **sistema de nós node-centric** ([ADR-0030..0039](docs/architecture/decisions/)):
  W1 (neck) e W2 (vertical Motion) fechados + contrato CONGELADO (ADR-0039);
  **fan-out aberto** (Wave 3+: mais nós Motion, Shader, Sound, Gameplay).
  Tracker vivo: [`docs/HANDOFF_node_system.md`](docs/HANDOFF_node_system.md).
- [`docs/plans/2026-05-wave-11-carry-overs.md`](docs/plans/2026-05-wave-11-carry-overs.md) —
  carry-overs pós-Wave 10 ([ADR-0042 §6](docs/architecture/decisions/0042-wave-10-closure.md)).
- [`docs/Painter_projeto/15_plano_de_implementacao.md`](docs/Painter_projeto/15_plano_de_implementacao.md) —
  **Painter (sucessor do Procreate)** — W0 ratificado 2026-05-26 com **11 ADRs Accepted**
  ([ADR-0043..0053](docs/architecture/decisions/)); W1 aberta (T0.8 = criar crate
  `ph2d-painter-contracts/` homestead arch-gate). Mandato §0 padrão-ouro absoluto;
  regra [`feedback-perfection-no-deferrals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md)
  ativa. Handoff: [`docs/HANDOFF_painter.md`](docs/HANDOFF_painter.md).
- [`docs/Vector Module/17_plano_de_implementacao.md`](docs/Vector%20Module/17_plano_de_implementacao.md) —
  **Vector Module (sucessor do Illustrator)** — W0 ratificada 2026-05-29 com **13 ADRs Accepted**
  ([ADR-0056..0068](docs/architecture/decisions/)) + amendments policy ativa; W1 aberta
  com 2 tracks paralelos: **T0.14 shell iPad scaffold** (CRITICAL pre-W1; 5-7d destranca
  cross-platform) + **T1.1 `ph2d-vector-traits` crate** (AnimValue typed enum + t:f64 +
  mocks foundation). PADRÃO-OURO ✓ ~9.7/10 pós-3 audits Antigravity (lentes rotacionadas;
  ENDORSEMENT 9.8/10). 8 inovações extraordinárias incluem Vector-SDF Hybrid GPU,
  Dormant Fracture Edges, LLM-as-graph-node, Painter↔Vector bridge bidirecional,
  Variable Fonts axes como graph inputs. ~9.590 linhas em 19 arquivos. Smoke W1 Day ~7-10
  = clica Pen tool → 3 pontos → triângulo Vello GPU prefix-sum.

**Fan-out drop-crate (node OU tool):** receita única em
[DIRETRIZ §3.8](docs/IntegracaoMultiAgente/DIRETRIZ.md) — briefing parametrizado
+ 2 exemplos pasted-ready em [`docs/IntegracaoMultiAgente/examples-fan-out.md`](docs/IntegracaoMultiAgente/examples-fan-out.md).

**Contratos congelados:**

- Nodes ([ADR-0039](docs/architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md))
  — caps `NodeOp=2` / `OpResolver=1` / `NodeManifest=8`, gate
  [`architecture_contract_surface`](crates/ph2d-nodegraph/tests/architecture_contract_surface.rs).
- Tools ([ADR-0040](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md)
  + [ADR-0041](docs/architecture/decisions/0041-rasteredit-rename-and-deactivate.md))
  — caps `Tool=10` / `RasterEditTool=5` / `PanelEvent=4`, gate
  [`architecture_tool_contract_surface`](crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs).
- Painter ([ADR-0043..0053](docs/architecture/decisions/)) — cascata W0 ratificada
  2026-05-26 com 11 ADRs Accepted. Caps principais: `PainterUiEdit ≤ 24` / `Brush ≤ 168` recursivo /
  `Stamp = 96B align(16) ABI` / `RenderingMode = 6 FROZEN` / `AdjustmentKind ≤ 32 (24 ship v1)` /
  `ColorProfile = 8 FROZEN` / `DeviceTier = 5 FROZEN`. Homestead gates:
  `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs` (W1 T0.8 cria).
- Vector Module ([ADR-0056..0068](docs/architecture/decisions/)) — cascata W0 ratificada
  2026-05-29 com 13 ADRs Accepted + amendments policy. Caps principais: `VectorOp ≤ 16 variants` /
  `Vertex` SmallVec inline 32 / `Segment` SmallVec inline 64 / `Region.segments` SmallVec inline 16 /
  `AnimValue` typed enum {Float/Vec2/Vec3/Color/Bool/Enum} / `AttributeEvaluator::sample(t: f64)` /
  18 nodes geométricos canônicos / 32 crates totais com consolidação seletiva / 8 inovações
  extraordinárias / `MAX_SPIRAL_TURNS=64` / `MAX_POLYGON_SIDES=128` / `MAX_VERTICES_PER_LLM_GEN=1000`
  (security sanitizers). Homestead gates: `crates/ph2d-vector-doc/tests/architecture_vector_contract_surface.rs`
  (W1 T1.2 cria) + `vello_kurbo_only_in_ph2d_vector` arch-gate (long-tail maintenance L6F1).

Planos históricos vivem em [`docs/archive/plans-completed/`](docs/archive/plans-completed/).

## Design system

[`docs/design/PROMPT_CLAUDE_DESIGN.md`](docs/design/PROMPT_CLAUDE_DESIGN.md) —
brief para gerar tokens.json + component-library.html + 17 mockups de tela
+ icons SVG + interactions/gestures/animation/accessibility specs. Output
do Claude Design alimenta a implementação dos widgets em Vello sobre o
[`ph2d-editor`](crates/ph2d-editor/) já existente (4 zonas, FloatingPanel,
ToolRegistry, ZenMode, ToastQueue por ADR-0023).

[`docs/design/component-library.html`](docs/design/component-library.html) —
mockup v2 inspirado em sdf3d-studio (4 temas OKLCH, glass surfaces,
Inter+JetBrains Mono); referência visual até o output canônico do Claude
Design substituí-lo.
