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

> ⚠️ **CORRIGIDO em 2026-08-18 — duas frases desta memória contradiziam o `CLAUDE.md` e eram
> as únicas do corpus a autorizar push autónomo.**
>
> 1. **«push direto, CI verde» está REVOGADO.** `CLAUDE.md §0.7`: *"Push é 1× por jornada — e
>    NUNCA é seu por conta própria … Integrar/pushar sem ordem = **violação do protocolo**"*.
>    O override que esta memória declara cobria o **SMOKE**, e passou a ser lido como licença
>    de **PUSH** — que nunca foi. Ver [[feedback_ship_only_enio_end_of_all_lines]] e
>    [[feedback_integration_only_enio_command_end_of_all_lines]].
> 2. **«nunca escrever smoke checklist no relatório» está INVERTIDO.** `CLAUDE.md §0.8` manda
>    o oposto, e é lei do Enio: o smoke vai em **passos numerados**, porque é onde ele
>    **aprende a ferramenta** ([[feedback_ready_to_smoke_example]]).
>
> **O que continua válido, e é a lição real:** não **PEDIR** smoke a cada fase. Entregue tudo,
> depois entregue o smoke — de uma vez, escrito para quem nunca viu aquilo.

**How to apply:**
- Diretriz "implemente X" / "siga até o fim" / "complete tudo" →
  fazer TODAS as phases planejadas, **commit local**, e reportar conclusão
  **com o smoke em passos numerados** (§0.8). ⛔ Sem push: quem pusha é o Enio,
  ou o integrador sob ordem explícita dele.
- Bug fix reportado durante o trabalho → fixar + continuar (não
  "fixar + pedir smoke do fix").
- ⚠️ **REVOGADO** (ver o bloco acima): o relatório de conclusão **LEVA** o smoke, em passos
  numerados, com o comando inteiro e o `cd` (§0.8). O que não se faz é **interromper** o
  trabalho a pedir smoke por fase. ⚠️ E o comando **não** é `play.command`: ele faz `cd` para
  a árvore **primária** e fixa um `CARGO_TARGET_DIR` de uma linha morta — em Modo L testa a
  árvore errada **em silêncio** ([[feedback_run_command_include_cd]]).
- Nunca usar "deferred" / "Wave 7 candidato" pra phases planejadas
  sem permissão explícita do Enio. Se a phase exige refactor
  difícil, fazer o refactor, não punt.
- Override total do "Smoke gates each phase" default do CLAUDE.md /
  proposta original. Mantém [[feedback-ci-batching]] (push agrupado
  ok) + [[feedback-commit-cadence]] (commits em blocos).
