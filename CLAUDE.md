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

## Plano operacional ativo

[`docs/plans/2026-05-post-spike.md`](docs/plans/2026-05-post-spike.md) — 13
marcos M1..M13 para implementação real do core pós-spike.

**Estado em 2026-05-09:** M1-M12 implementados e mergeados (PRs #1-#28).
M13 em curso: tool palette UI shipada (PR #30), em paralelo com design
library handoff para Claude Design (vide `docs/design/`).

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
