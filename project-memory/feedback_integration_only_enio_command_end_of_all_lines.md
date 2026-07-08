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

**Quem integra (refinamento 2026-07-07):** não é a linha (nem sob comando) — é um
**agente integrador dedicado** que o Enio abre quando decide integrar, munido de um
**handoff de integração** de cada linha (DIRETRIZ §1.5.9: foundational tocado,
ids/consts novos com valores literais, o que só o ship.sh pega, o que smoke-testar).
A linha entrega o handoff e para; o integrador resolve TODOS os conflitos e funde via
`--ff-only`. Um agente que integrou E shipou sem ordem explícita = violação (aconteceu,
motivou este reforço).

**How to apply:** trate `foundational-integrate.sh` como o `ship.sh` do
[[feedback_ship_only_enio_end_of_all_lines]] — Enio-only. O DoD do briefing que
manda "rode o integrate ao fechar" fica **sobrescrito**: feche → escreva o handoff
(§1.5.9) → reporte "linha pronta + handoff" → **PARE**. Vide
[[project_multiagent_modo_l_2026_07_05]] (Modo L),
[[feedback_foundational_editable_design_for_isolation]] (foundational + handoff),
[[feedback_fast_mode_ship]] (commit local sem push de dia).
