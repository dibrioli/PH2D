# CLAUDE.md — núcleo operacional do PH2D (LEIA INTEIRO — é curto de propósito)

> Toda LLM recebe este arquivo automaticamente. Ele é o **roteador**: os inegociáveis +
> para onde ir por tarefa. Detalhe técnico → [`SKILL_Stack_PH2D_Definitiva.md`](SKILL_Stack_PH2D_Definitiva.md)
> (HR-1..HR-18, stack). Processo → [`DIRETRIZ.md`](docs/IntegracaoMultiAgente/DIRETRIZ.md).
> Não leia esses dois inteiros — use o roteador §1.

## §0 — Inegociáveis (memorize os 7)

1. **Norte arquitetural ([ADR-0075](docs/architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)):** monorepo Rust único; desacoplar por **ECS** (components + events/resources, systems não se chamam), **NÃO** por plugin em runtime nem WASM. Feature nova = **drop-crate** (A). Plugin runtime foi pesquisado e **rejeitado** (sem ABI estável; nem resolve o coupling de schema).
2. **Isolamento:** edite **só a SUA pasta**. Precisou de algo fora (foundational/shell/contrato/outra crate)? **PARE e reporte ao Coordenador** — nunca renegocie direto com outro agente.
3. **UI canônica:** zero hex, zero `f32` literal de UI, zero string hardcoded — tudo via tokens / i18n (HR-15).
4. **Git anti-colisão:** `git add -- <seus paths>` (NUNCA `-A`/`-a`/`git add .`/`git stash`); `git commit --no-verify -m "msg" -- <paths>`; `git status` antes de stage; se houver `M`/`??` alheio, não comite — reporte.
5. **Velocidade (§2):** inner loop = **SÓ `cargo check -p`** (no slot CoW); teste/clippy/auditoria **1× no fechamento do módulo**, nunca por task. **≤3 agentes** compilando por vez (RAM 8 GiB).
6. **Padrão-ouro sem custo:** a melhor opção técnica vence custo de build/cronograma ([feedback-perfection-no-deferrals](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md)); gaps in-scope fecham na sessão atual.
7. **Você NÃO pusha.** Reporta commit local; o **Coordenador** faz ship + push + babysit CI (§3).

## §1 — Roteador leia-por-tarefa (leia SÓ o que sua tarefa exige)

> **A CADA passo de QUALQUER implementação, leia primeiro [DIRETIVA_IMPLEMENTACAO.md](docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).**
> É o antídoto das 4 causas da semana perdida no Painter (costura não-testada · "audit"=compilar ·
> isolamento órfão · alvo irrefutável). Regra-mãe: **verde-de-compilação é velocidade; no audit vale ZERO.**

| Sua tarefa | Leia ISTO (e só isto) |
|---|---|
| **Tool ou node nova** | DIRETRIZ §2 (triagem) + §3.A + [examples-fan-out.md](docs/IntegracaoMultiAgente/examples-fan-out.md) |
| **Painel / widget / chrome** | DIRETRIZ §3.B |
| **Modificar feature existente** | DIRETRIZ §3.D |
| **Foundational / contrato congelado** | DIRETRIZ §3.C + §4 (**Coord-only + ADR**) |
| **Build lento / quero voar** | DIRETRIZ §6 (stack de velocidade) — §2 abaixo é o resumo |
| **Dúvida de stack / Hard Rule** | SKILL_Stack §HR-1..18 (cite por ID) |
| **Quem é o Enio / estado do projeto** | [MEMORY.md](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md) |
| **Quem possui o quê agora** | [SESSION_ACTIVE.md](docs/SESSION_ACTIVE.md) |

## §2 — Velocidade ("agents flying"), resumo (detalhe + configs: DIRETRIZ §6)

- **Inner loop = `cargo check -p <crate>`** (ou `scripts/cargo-check-narrow.sh <crate>` p/ cortar tokens de erro). Nada de test/clippy/auditor por task.
- **Slot warm por CoW:** `bash scripts/slot-seed.sh <slot>` → prefixe cada cargo com o `CARGO_TARGET_DIR` impresso (o Bash-tool não persiste env). Nunca use o `target/` default.
- **Diagnóstico via LSP (maior alavanca):** prefira o **LSP nativo do Claude Code** / `bacon-ls` a ler saída crua do cargo e adivinhar tipos. (Pilotar + medir RAM — vide DIRETRIZ §6.)
- **Gate batched no fim do módulo:** `scripts/nextest-impacted.sh` + clippy `--all-targets` + auditoria ≥2 lentes, **1× sobre o diff acumulado**.
- **≤3 cargos simultâneos** (RAM 8 GiB) — o Coordenador escalona via SESSION_ACTIVE.
- **NÃO use:** Cranelift (ruim p/ check-loop + gaps macOS), `mold` (incompatível macOS — use lld/ld-prime).

## §3 — CI / ship (Coordenador absorve PRCI)

**Implementador:** não faz `git push`, não monitora CI — reporta commit local pronto.
**Coordenador:** push **1× por jornada** (run de CI ~30min: matrix linux+macOS+windows + replay-hash + bench). Protocolo [DIRETRIZ §8](docs/IntegracaoMultiAgente/DIRETRIZ.md):
1. `./scripts/ship.sh` (paridade EXATA com lint+test do CI — fmt, clippy `--all-targets`+features, machete, deny, audit, nextest `--cargo-profile ci-test`, typos). Corrija TODO `✗`, **não pusha antes de verde**.
2. `git push origin main` → babysit (polling 15min, `gh run watch`) até `success`; em vermelho, fix + re-push (escalona após 3 falhas do mesmo job).
3. Forneça SEMPRE o link: `https://github.com/dibrioli/PH2D/actions/runs/<id>` (`gh run list --workflow=spike.yml --limit=1`).

**Fast mode (dia):** `git commit --no-verify` (instantâneo), `cargo check -p` quando quiser, **zero push/CI**. Ship só no fim quando o Enio mandar ("commit"/"push"/"ship"/"fim do dia").

## §4 — Memória persistente

[`MEMORY.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md) — feedback acumulado, perfil do Enio, estado, paths canônicos. **LLM nova lê o índice antes de agir.**

## §5 — Planos operacionais ativos

- [`docs/plans/2026-05-node-waves.md`](docs/plans/2026-05-node-waves.md) — sistema de nós ([ADR-0030..0039](docs/architecture/decisions/)): W1+W2 fechados + contrato CONGELADO; fan-out aberto. Tracker: [`docs/HANDOFF_node_system.md`](docs/HANDOFF_node_system.md).
- [`docs/plans/2026-05-wave-11-carry-overs.md`](docs/plans/2026-05-wave-11-carry-overs.md) — carry-overs pós-Wave 10 ([ADR-0042](docs/architecture/decisions/0042-wave-10-closure.md)).
- [`docs/HANDOFF_painter_brush_engine.md`](docs/HANDOFF_painter_brush_engine.md) — **Painter / Brush Engine** (tracker ÚNICO; [ADR-0043..0053](docs/architecture/decisions/) + [ADR-0097](docs/architecture/decisions/0097-brush-engine-procreate-parity-cpu-first-dab-pipeline.md)): W0-W4 **FECHADOS** (layers panel + compositor GPU 22-modos + persist v2 + dirty-rect + bloom/S-H). **Frontier ATIVO = Brush Engine** (CPU-first dab pipeline, paridade Procreate; Track A provado — golden harness + gate dos 8 sites). Referência dos 14 painéis + plano em [`docs/Novo Painter/`](docs/Novo%20Painter/). Toda etapa segue [`DIRETIVA_IMPLEMENTACAO.md`](docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).
- **Watercolor/fluid/wash — REMOVIDOS** ([ADR-0096](docs/architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md), Enio 2026-06-15): toda a simulação de aquarela (crate `ph2d-painter-wash`, sessões GPU `painter_wash_gpu`/`painter_canvas_gpu`, `wash_pipeline`/settle/bordas-molhadas) foi **deletada** e o canvas voltou a CPU-residente. Backups intactos em `backups/wash_2026-06-14` + `backups/watercolor_v2_2026-06-12`. SCHEMA_VERSION 2→3 (quebra dura de save, postcard posicional). Preservados: layer-stack + compositor GPU + efeitos/ajustes + o **brush default (`apply_stamps_wash` = blend-mode, NÃO sim)**. ADR-0096 supersede ADR-0085..0095 (mantidos como histórico). **Pivot:** mixer-brush (Procreate-style) + Kubelka–Munk/Mixbox, não fluido. **Novo norte ATIVO → Brush Engine** ([ADR-0097](docs/architecture/decisions/0097-brush-engine-procreate-parity-cpu-first-dab-pipeline.md), CPU-first dab pipeline, paridade Procreate): pesquisa + referência exaustiva dos 14 painéis + plano passo-a-passo (cada feature = uma etapa) + matriz de cobertura `[E]` em [`docs/Novo Painter/`](docs/Novo%20Painter/).
- [`docs/Vector Module/17_plano_de_implementacao.md`](docs/Vector%20Module/17_plano_de_implementacao.md) — **Vector Module** ([ADR-0056..0068](docs/architecture/decisions/)): **W1 FECHADA** (auditada 2026-06-01); **W2 ABERTA** (Pencil/Shapes/Select/Color/Undo). Handoff: [`docs/HANDOFF_vector_w2_impl.md`](docs/HANDOFF_vector_w2_impl.md).
- **Sprite Inspector v2** ([ADR-0069..0074](docs/architecture/decisions/)) — W0-W3 + W6 + W10 completos (§0-§9 Inspector + ClipChildren/Visibility/Ordering/Sampling render + widgets + OKLCH picker + Material&Blend); tracker [`docs/HANDOFF_sprite_inspector_v2.md`](docs/HANDOFF_sprite_inspector_v2.md).
- **KTX2 Fase 2** ([ADR-0055](docs/architecture/decisions/)) — W0+W1+W2 fechados (cooker + renderer pipeline + budget); W3 Painter-integration. **imageio AVIF** ([ADR-0054](docs/architecture/decisions/)) — W0-W3 fechado (Path C real encode/decode via libavif, zero RUSTSEC).

Sub-handoffs vivos dos implementadores: `docs/HANDOFF_*_impl.md`. Históricos: [`docs/archive/plans-completed/`](docs/archive/plans-completed/).

## §6 — Contratos congelados (mexer = Coord-only + ADR; DIRETRIZ §4)

- **Nodes** ([ADR-0039](docs/architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)): `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` — gate `architecture_contract_surface`.
- **Tools** ([ADR-0040](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md)+[0041](docs/architecture/decisions/0041-rasteredit-rename-and-deactivate.md)): `Tool=11`/`RasterEditTool=5`/`PanelEvent=4` — gate `architecture_tool_contract_surface`. (`Tool` 10→11 em [ADR-0040-amendment-2](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md): `on_tick` heartbeat p/ aquarela live, ADR-0049/0077-D11.)
- **Painter** ([ADR-0043..0053](docs/architecture/decisions/)): `PainterUiEdit≤24`/`Brush≤168`/`Stamp=96B align(16)`/`RenderingMode=6`/`AdjustmentKind≤32`/`ColorProfile=8`/`DeviceTier=5` — gate `architecture_painter_contract_surface`. (ABIs de superfície/UI — **ficam congelados**.)
- ~~**Watercolor (física)** (ADR-0049/0078-0084)~~ — **REVOGADO** por [ADR-0096](docs/architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md): a sim de aquarela e seus gates (`gpu_parity`/`composite_parity`) foram removidos junto com a crate `ph2d-painter-wash`. O modelo K–M espectral é histórico (backup); o pivot para mixer-brush usa Kubelka–Munk/Mixbox no blend do pigmento, não shallow-water. Nada congelado aqui.
- **Vector** ([ADR-0056..0068](docs/architecture/decisions/)): `VectorOp≤16`/`Vertex`SmallVec32/`Segment`64/`Region.segments`16/`AnimValue` enum/`sample(t:f64)`/18 nodes/`MAX_SPIRAL_TURNS=64`/`MAX_POLYGON_SIDES=128`/`MAX_VERTICES_PER_LLM_GEN=1000` — gate `architecture_vector_contract_surface`. Gate `vello_kurbo_only_in_ph2d_vector` é **W2-deferred (não existe ainda)**.

## §7 — Design system

[`docs/design/PROMPT_CLAUDE_DESIGN.md`](docs/design/PROMPT_CLAUDE_DESIGN.md) (brief: tokens.json + mockups + icons + specs) alimenta os widgets em Vello sobre [`ph2d-editor`](crates/ph2d-editor/) (ADR-0023). Mockup de referência: [`docs/design/component-library.html`](docs/design/component-library.html).
