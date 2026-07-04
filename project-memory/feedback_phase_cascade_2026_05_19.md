---
name: feedback-phase-cascade-2026-05-19
description: "Em planos com fases sequenciais (Phase C.1..C.N etc), cada fase termina spawnando agente novo para a próxima; última fase faz PR + CI."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e2b00966-00ef-4a2a-af1a-3b19ad1bb036
---

Para planos com fases sequenciais (ex. ADR-0029 Phase C.1..C.4, ou waves), o procedimento operacional padrão é:

1. **Fechar a fase atual:** implementar, validar (cargo check + workspace tests), commitar local.
2. **Preparar handoff:** escrever `docs/HANDOFF_<NOME>_<PROX>.md` (formato igual aos antecessores: §0 verificação rápida, §1 leitura obrigatória, §3 o que a fase anterior entregou, §4 tensões resolvidas, §5 escopo da próxima fase, §6 smoke se aplicável). Commitar o handoff separado (`docs: HANDOFF_<...> — pickup guide for fresh LLM session`).
3. **Atualizar memória:** criar `project_phase_<...>.md` + adicionar linha em `MEMORY.md`.
4. **Spawnar agente novo (Agent tool, subagent_type="claude")** com prompt completo apontando para o handoff e dando GO. O agente novo lê handoff, faz a fase seguinte sozinho.
5. **Cascatear até o fim do plano:** repetir 1-4 até a última fase. NÃO esperar instrução do Enio entre fases — ele já autorizou o cascade.
6. **Última fase:** abrir PR + monitorar CI (papel PRCI per `docs/IntegracaoMultiAgente/04-Agente-PRCI.md`).

**Why:** Enio prefere não micro-gerenciar fases sequenciais; cada uma é mecânica depois do padrão estabelecido pela primeira (Phase C.1 estabeleceu o padrão para C.2/C.3/C.4). Cascatear via agente novo evita estouro de contexto.

**How to apply:** quando um plano tem fases numeradas (C.1, C.2, C.3...) e a primeira já fechou estabelecendo padrão, cada agente fecha sua fase + prepara handoff + chama agente novo para próxima. NÃO criar workdir, NÃO push, NÃO PR exceto na última fase. Mantenha o smoke do Enio agrupado conforme `feedback_smoke_at_end`. Originado em 2026-05-19 durante a transição C.2 → C.3 do ADR-0029.

Vide também: [[feedback-smoke-at-end]], [[feedback-commit-cadence]], [[project-phase-c2-2026-05-18]].
