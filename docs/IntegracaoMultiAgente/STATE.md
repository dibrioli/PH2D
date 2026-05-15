# STATE — Operação Multi-Agente PH2D

`STATE.md` é a **fonte de verdade** sobre a operação multi-agente.
Coordenador escreve; Agentes Periféricos só leem.

---

**Atualizado por:** Coordenador (única sessão autorizada a escrever em STATE.md)
**Última atualização:** 2026-05-15 17:35 BRT

`STATE.md` é a **fonte de verdade** sobre a operação multi-agente.
Coordenador escreve; Agentes Periféricos só leem.

## Slots ativos (máx 4)

| # | Slug | Pastas reservadas | Status | Última atividade |
|---|---|---|---|---|
| 1 | grid-snap | _(a propor pelo Agente)_ | pending-start | 2026-05-15 17:35 |
| 2 | (vago) | — | — | — |
| 3 | (vago) | — | — | — |
| 4 | (vago) | — | — | — |

**Status possíveis:**
- `pending-start` — slot atribuído, agente ainda não começou.
- `proposing-folder` — agente leu briefing, propôs pasta(s), aguardando aprovação do Coordenador.
- `working` — agente codificando dentro das pastas aprovadas.
- `blocked-waiting-coord` — agente parou aguardando ação do Coordenador (dep externa, ícone, etc.).
- `waiting-integration` — feature pronta, na fila de integração.
- `integrating` — Coordenador está integrando agora.
- `done` — integrada com sucesso; slot pode ser liberado.

## Fila de integração (FIFO)

_(vazia)_

Quando um Agente reporta "feature pronta", o slug entra no fim
desta fila. Coordenador processa do topo. Se Coordenador está
idle e fila tem um item no topo, processa imediatamente.

## Pedidos pendentes ao Coordenador

_(nenhum)_

Formato: lista bullet, cada item com: `slug — pedido — recebido em <time>`.
Exemplo: `painter — adicionar dep imageproc 0.25 em ph2d-editor/Cargo.toml — 17:30`.

## Lock de integração

Coordenador: `idle`

Alternativas: `integrando <slug> since <time>` durante uma integração.

## Sha conhecido bom (rollback target)

main @ `09903fcd232b1c6484a5414c1cefa825da95ea13` — 2026-05-15 17:35 — fix(editor) Inspector name refresh after hierarchy rename (HEAD; refactor wave 36 commits + polish fix committed locally, push pendente)

Atualizado pelo Coordenador após cada integração bem-sucedida
(`cargo check --workspace` verde). Em caso de quebra catastrófica,
`git reset --hard <sha>` traz main local de volta ao último ponto
estável conhecido.

## Histórico de operação (append-only — entradas recentes no topo)

| Quando | Evento |
|---|---|
| 2026-05-15 17:35 | slot 1 atribuído a grid-snap (Grid PRO + Snap módulo) |
| 2026-05-15 17:35 | STATE.md inicializado a partir do template; sha bom = `09903fc` |

Entradas típicas:
- `2026-05-13 17:00 — slot 1 atribuído a painter`
- `2026-05-13 17:30 — painter aprovou pasta tools/painter/`
- `2026-05-13 18:15 — painter waiting-integration`
- `2026-05-13 18:20 — integração de painter iniciada`
- `2026-05-13 18:45 — integração de painter done; sha bom atualizado para abc1234`

## Notas operacionais

- **Apenas Coordenador escreve neste arquivo.** Cada mudança gera commit
  `chore(coordenador): <descrição>`.
- **Agentes Periféricos LEEM este arquivo** antes de cada decisão
  importante (qual sua pasta exclusiva? coordenador idle? fila à
  sua frente?).
- **Toda comunicação operacional passa pelo Enio** (relay humano).
  Coordenador não fala direto com Agentes.
- **Build verde em main local** é responsabilidade do Coordenador
  (`cargo check --workspace` após cada integração).
- **Sem branches feature/**, **sem worktrees**, **sem push pro GitHub**
  durante o ciclo. Tudo em main local.
