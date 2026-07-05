---
name: project-multiagent-modo-l-2026-07-05
description: Modelo multi-agente virou função do hardware — workstation=Modo L (linhas por git worktree, SEM coordenador, foundational concorrente via gate testado + Mergiraf); constrained=Modo C (v7.1 shared-tree). Supersede o Coordenador-sempre de [[project-diretriz-v68-2026-05-22]].
metadata:
  type: project
---

Desde 2026-07-05 o **modo de operação multi-agente é função do hardware** (`bash scripts/hw-profile.sh`), não fixo:

- **`workstation` (Linux 128 GB) = Modo L** ([ADR-0106](docs/architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)): N linhas autônomas = N `git worktree` (branch `line/<módulo>`), uma sessão Claude por linha, **SEM Coordenador de plantão**. Cada linha se integra sozinha ao main via `scripts/foundational-integrate.sh` (rebase → gate da árvore combinada `cargo check --workspace` → `git merge --ff-only`). Ship 1×/jornada por quem fecha a ÚLTIMA integração.
- **`constrained` (Mac 8 GB) = Modo C**: o v7.1 preservado (1 Coordenador único + N Implementadores em shared tree) — só p/ sessões de smoke/hotfix.

**Foundational deixou de ser serial no Modo L** ([ADR-0107](docs/architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md)): qualquer linha toca `ph2d-core`/`editor-core`/`tokens`/… sob o gate testado + **Mergiraf** (merge driver sintático via tree-sitter, `scripts/mergiraf-setup.sh`, 1×/máquina). Só **contrato congelado** (§6) e **mesmo-símbolo de tipo-núcleo** seguem seriais → agente PARA e reporta ao Enio (+ ADR). "Camada 0" gerou as 2 últimas superfícies hand-central que restavam: chrome `dispatch_all` (marcador `// ph2d-chrome-sync:z=NN` + `ph2d-chrome-sync`) e `ColorToken` (enum+`key()` de uma `color_tokens!` macro).

**Why:** o Coordenador nasceu do shared-tree do Mac 8 GiB (RAM não cabia N checkouts). No desktop 128 GB o custo sumiu; worktree isola git, pasta disjunta isola merge, Mergiraf+gate cobrem foundational — o gargalo humano é desnecessário.

**How to apply:** LLM nova roda `hw-profile.sh` PRIMEIRO. No `workstation`, **não procure nem espere um Coordenador** — abra a sua linha ([`MODELO_ABERTURA_LINHA.md`](docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md)) e integre sozinho. Guia do operador (Enio): [`GUIA_JORNADA_MODO_L.md`](docs/IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md). As memórias de colisão-git ([[feedback-parallel-agent-collision]], [[feedback-scoped-commit-shared-index]], [[feedback-git-stash-multiagent-danger]], …) valem **só no Modo C** — no Modo L cada linha tem worktree+índice próprios. Supersede o "2 papéis / Coord absorve PRCI" de [[project-diretriz-v68-2026-05-22]] (agora Modo-C-only).
