---
name: feedback_integration_only_enio_command_end_of_all_lines
description: Modo L multi-linha — integração ao main SÓ sob comando do Enio, no fim do turno de TODAS as linhas paralelas; não faça self-service integrate
metadata:
  type: feedback
---

Numa jornada Modo L com **N linhas paralelas** (ex.: 5 agentes), a integração
ao main (`scripts/foundational-integrate.sh`) **NÃO é self-service por linha** —
ela acontece **só sob comando explícito do Enio, no fim do turno de trabalho de
TODAS as linhas**. Cada agente-de-linha: implementa → gate batched → **commit
local** → reporta "pronto para integrar" → **PARA e espera**. Nunca rode o
integrate sozinho ao fechar a sua linha.

**Why:** com N linhas vivas, cada `--ff-only` re-serializa e força rebase das
demais; deixar cada agente integrar quando termina vira uma corrida de rebases +
CI parcial. O Enio quer ver o conjunto verde e decidir a ordem/o momento de
fundir tudo de uma vez (mesma lógica do ship, estendida à integração).

**How to apply:** trate `foundational-integrate.sh` como o `ship.sh` do
[[feedback_ship_only_enio_end_of_all_lines]] — Enio-only, fim da rodada de todas
as linhas. O DoD do briefing que manda "rode o integrate ao fechar" fica
**sobrescrito** por este comando quando a jornada é multi-linha. Reporte o SHA do
commit local e aguarde. Vide [[project_multiagent_modo_l_2026_07_05]] (Modo L),
[[feedback_fast_mode_ship]] (commit local sem push de dia).
