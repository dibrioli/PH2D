---
name: project-ci-runs-26-of-313-workspace-members
description: «a suíte está verde» e «o CI está verde» são afirmações sobre populações muito diferentes — o nextest do CI é uma lista de 26 `-p`, não `--workspace`
metadata:
  type: project
---

Medido em 2026-08-30 lendo o passo `workspace unit tests (multi-package nextest)` do
`.github/workflows/spike.yml`: ele **não é `--workspace`**, é uma lista de **26 `-p` escrita à
mão**, contra **313** membros da workspace (`crates/*`, `tools/*`, `shells/desktop`,
`tests/spike`) — **8%**.

Ficam de fora, entre outros: `ph2d-editor-core` (os 53 gates de arquitectura),
`ph2d-tool-painter` (136 k LOC), `ph2d-physics`, `ph2d-physics-ecs`, `ph2d-timeline`, todos os
`ph2d-panel-*` e `shells/desktop`.

⚠️ **A lista estreita NÃO é preguiça — é uma cerca medida.** O `spike.yml` traz o histórico ao
lado do `timeout-minutes: 90`: uma build fria no Windows recompila o grafo inteiro incluindo
dav1d/rav1e e passou de 45 min, e *«um job cancelado NUNCA corre o save do rust-cache»* ⇒ um
estouro deixa o cache frio para sempre.

⛔ **E alargar tem um bloqueador nomeado que não é o tempo:** a família de falso-positivos de
CARGA (`CLAUDE.md` §5.0) — gates que dividem dois relógios ou contam alocações reprovam sob
paralelismo. Um runner de 2 vCPU a correr 20 065 testes é o pior caso possível para eles.
*A cura é a régua, não o corredor.*

**How to apply:** o `scripts/ship.sh` corre `nextest --workspace`, logo o portão LOCAL é mais
largo que o CI — é ele que vale como prova. Nunca diga «o CI valida isto» sobre uma crate sem
confirmar que ela está naquela lista. Detalhe e tabela: `docs/Atualizar Stack/04_registro.md` §23.
