---
name: project-event-sheet-module-abandoned-2026-11-13
description: "Dono 13/11 desistiu do módulo Planilha de Eventos (line/EventSheet, ondas 0–3) e mandou apagar a linha; não recomeçar sem ordem dele"
metadata:
  node_type: memory
  type: project
  originSessionId: d8a04fb3-c7ad-41e4-9ce2-b220809c402b
  modified: 2026-11-13T21:01:10.174Z
---

O dono (13/11/2026) desistiu do módulo **Event Sheet / Planilha de eventos** (ADR-0201, `docs/EventSheet/`, crates `ph2d-event-sheet*`, `ph2d-app-event-sheet`) e mandou **apagar a linha**: worktree `Worktrees/line-EventSheet` e ramo `line/EventSheet` removidos (último commit `9343c7baa6`, ondas 0–3, nunca integrado nem enviado; o commit ainda se recupera pelo reflog por algum tempo).

**Why:** pediu uma comparação séria com Construct 3 e GDevelop 5; o resultado foi que eles fazem muito mais (todas as cópias / famílias, gatilhos de jogo prontos, Senão/Ou/sub-eventos, depurador, reaproveitamento), e a planilha não fazia um jogo real sem Luau.

**How to apply:** não propor nem recomeçar a planilha de eventos, nem reaproveitar o ADR-0201, sem ordem explícita do dono. A lógica de jogo segue pelos caminhos que já existem (Luau, componentes de jogo, Signal Actions).
