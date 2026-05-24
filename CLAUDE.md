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

## Plano operacional ativo

[`docs/plans/2026-05-node-waves.md`](docs/plans/2026-05-node-waves.md) —
**sistema de nós node-centric** ([ADR-0030..0039](docs/architecture/decisions/)):
W1 (neck) e W2 (vertical Motion) fechados + contrato CONGELADO (ADR-0039);
**fan-out aberto** (Wave 3+: mais nós Motion, Shader, Sound, Gameplay, ferramentas
imperativas). Receita pronta-pra-colar em [DIRETRIZ §3.8](docs/IntegracaoMultiAgente/DIRETRIZ.md#38-node-crate-novo--fan-out-o-caminho-principal-de-crescimento).
Tracker vivo: [`docs/HANDOFF_node_system.md`](docs/HANDOFF_node_system.md).

**Histórico recente (concluído):**

- [`docs/plans/2026-05-tool-isolation-waves.md`](docs/plans/2026-05-tool-isolation-waves.md) —
  **reestruturação tool-as-crate** ([ADR-0040](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md))
  🔒 **CLOSED 2026-05-22** (TG-A..TG-E em uma jornada). Tool agora é satélite
  drop-in: `cargo run -p ph2d-tool-sync` regenera o wiring central, contrato
  `Tool`/`RasterEditTool`/`PanelEvent` congelado por arch-gate. Receita pra
  tool nova em [DIRETRIZ §3.9](docs/IntegracaoMultiAgente/DIRETRIZ.md) + SKILL_Stack §"Adicionar uma tool".
- [`docs/plans/2026-05-post-spike.md`](docs/plans/2026-05-post-spike.md) —
  M1..M13 do core pós-spike, todos mergeados (PRs #1-#30).

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
