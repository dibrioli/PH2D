---
name: feedback-smoke-at-end
description: "NUNCA pedir smoke visual no meio da implementação. Rodar TUDO autonomamente até concluir todas as etapas + CI verde. Smoke é responsabilidade exclusiva do Enio, no fim, sem ser pedido."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 3cd59062-48fc-4433-8496-0552be468b98
---

**REGRA ABSOLUTA — não pedir smoke jamais durante implementação.**

Workflow do Enio (re-confirmado com ênfase em 2026-05-18 após eu
pedir smoke entre phases mesmo após o primeiro aviso):

- Quando ele dá uma diretriz tipo "implemente X completo" ou "vai
  até o fim", isso significa: **implementar TODAS as etapas
  sem parar.** Não fragmentar em "Phase X done, smoke?", não
  oferecer checkpoints intermediários, não criar pontos de
  validação visual no plano.
- Smoke (`./play.command`) é **decisão exclusiva do Enio**, no
  momento que ele quiser, sem ser pedido pela LLM.
- Se a wave/feature tem N phases, fazer **todas as N** + push +
  CI verde antes de reportar conclusão. Sem "deferir por escopo"
  unilateralmente — se ele pediu, fazer.

**Why:** Smoke leva tempo do Enio. Ele NÃO quer ser interrompido
a cada milestone pra rodar play.command. Prefere fazer UMA smoke
abrangente quando ele decide, depois que a LLM entregou tudo.
Pedir smoke a cada phase quebra o fluxo dele.

**How to apply:**
- Diretriz "implemente X" / "siga até o fim" / "complete tudo" →
  fazer TODAS as phases planejadas, push direto, CI verde,
  reportar conclusão sem mencionar smoke.
- Bug fix reportado durante o trabalho → fixar + continuar (não
  "fixar + pedir smoke do fix").
- Nunca escrever "smoke checklist" ou "quando rodar play.command,
  teste X" no relatório de conclusão.
- Nunca usar "deferred" / "Wave 7 candidato" pra phases planejadas
  sem permissão explícita do Enio. Se a phase exige refactor
  difícil, fazer o refactor, não punt.
- Override total do "Smoke gates each phase" default do CLAUDE.md /
  proposta original. Mantém [[feedback-ci-batching]] (push agrupado
  ok) + [[feedback-commit-cadence]] (commits em blocos).
